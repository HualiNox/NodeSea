//! The session runtime and command worker for a native libtorrent session.

use std::thread::{self, JoinHandle};

use tokio::sync::{mpsc, oneshot};

use crate::{
    SettingsPack,
    engine::{
        dispatcher::EventDispatcher,
        errors::EngineError,
        handle::{CommandReceiver, EngineCommand, StatusSender},
    },
    ffi,
};

/// Lifecycle state reported by the engine runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    /// The engine has been built but has not started running.
    Idle,
    /// The native session is being created.
    Starting,
    /// The native session is running and accepts commands.
    Running,
    /// Shutdown is in progress and no new commands are processed.
    Stopping,
    /// The native session and runner have completed shutdown.
    Stopped,
    /// The engine failed while starting and cannot accept commands.
    Failed,
}

/// Owns the native session and coordinates it with the command worker.
pub(crate) struct EngineRunner {
    settings_pack: SettingsPack,
    session: Option<ffi::Session>,
    dispatcher: EventDispatcher,
    command_rx: Option<CommandReceiver>,
    shutdown_rx: mpsc::Receiver<ShutdownRequest>,
    shutdown_tx: mpsc::Sender<ShutdownRequest>,
    command_worker: Option<CommandWorker>,

    status_tx: StatusSender,

    // Keeps the notifier address stable while the native callback is registered.
    alert_notifier: Box<ffi::AlertNotifier>,
}

struct ShutdownRequest {
    /// Completes the public shutdown request, if one initiated this stop.
    reply: Option<oneshot::Sender<Result<(), EngineError>>>,
}

/// Owns the native command port and guarantees that it is destroyed on the
/// same worker thread that executes command calls.
struct CommandWorker {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<JoinHandle<()>>,
}

impl CommandWorker {
    /// Starts the command loop on its own OS thread and current-thread runtime.
    fn spawn(
        port: ffi::CommandPort,
        command_rx: CommandReceiver,
        shutdown_tx: mpsc::Sender<ShutdownRequest>,
    ) -> Self {
        let (stop_tx, stop_rx) = oneshot::channel();
        let join = thread::Builder::new()
            .name("nodesea-command".to_owned())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("command worker runtime should build");
                runtime.block_on(command_loop(port, command_rx, shutdown_tx, stop_rx));
            })
            .expect("command worker thread should start");

        Self {
            stop_tx: Some(stop_tx),
            join: Some(join),
        }
    }

    fn stop_and_join(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// Serializes all native command calls independently from alert polling.
async fn command_loop(
    mut port: ffi::CommandPort,
    mut command_rx: CommandReceiver,
    shutdown_tx: mpsc::Sender<ShutdownRequest>,
    mut stop_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else {
                    let _ = shutdown_tx.send(ShutdownRequest { reply: None }).await;
                    break;
                };

                match command {
                    EngineCommand::FetchMetadata { torrent_id, reply } => {
                        let _ = reply.send(ffi::fetch_metadata_from_port(&mut port, &torrent_id));
                    }
                    EngineCommand::CancelFetchMetadata { torrent_id, reply } => {
                        let _ = reply.send(ffi::cancel_fetch_from_port(&mut port, &torrent_id));
                    }
                    EngineCommand::PostDhtStats { reply } => {
                        let _ = reply.send(ffi::post_dht_stats_from_port(&port));
                    }
                    EngineCommand::PostSessionStats { reply } => {
                        let _ = reply.send(ffi::post_session_stats_from_port(&port));
                    }
                    EngineCommand::PostDhtSampleInfohashes { endpoint, target, reply } => {
                        let _ = reply.send(ffi::post_dht_sample_infohashes_from_port(
                            &port, &endpoint, &target,
                        ));
                    }
                    EngineCommand::PostDhtLiveNodes { reply } => {
                        let _ = reply.send(ffi::post_dht_live_nodes_from_port(&port));
                    }
                    EngineCommand::Shutdown { reply } => {
                        let _ = shutdown_tx.send(ShutdownRequest { reply: Some(reply) }).await;
                        break;
                    }
                }
            }
            _ = &mut stop_rx => break,
        }
    }
}

impl EngineRunner {
    /// Creates a runner that will lazily create its native session in `run`.
    pub(crate) fn new(
        settings_pack: SettingsPack,
        dispatcher: EventDispatcher,
        command_rx: CommandReceiver,
        status_tx: StatusSender,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        Self {
            dispatcher,
            settings_pack,
            session: None,
            command_rx: Some(command_rx),
            shutdown_rx,
            shutdown_tx,
            command_worker: None,
            status_tx,
            alert_notifier: Box::new(ffi::AlertNotifier::new()),
        }
    }

    fn start_session(&mut self) -> Result<(), EngineError> {
        let mut session = ffi::start_session(self.settings_pack.clone())
            .map_err(EngineError::SessionStartError)?;
        ffi::set_alert_notify(&mut session, self.alert_notifier.as_ref());
        self.session = Some(session);

        // The command receiver is moved into the worker so the runner never
        // competes with it for command-channel polling.
        let command_rx = self
            .command_rx
            .take()
            .expect("command receiver should only be taken once");
        let port = match self
            .session
            .as_mut()
            .map(ffi::start_command_port)
            .transpose()
            .map_err(EngineError::SessionStartError)?
        {
            Some(port) => port,
            None => {
                self.stop_session();
                return Err(EngineError::SessionStartError(
                    "session did not start".to_owned(),
                ));
            }
        };
        self.command_worker = Some(CommandWorker::spawn(
            port,
            command_rx,
            self.shutdown_tx.clone(),
        ));
        Ok(())
    }

    fn stop_session(&mut self) {
        // The command worker must already be joined before the native session
        // is dropped; CommandPort contains a reference to this session.
        if let Some(mut session) = self.session.take() {
            ffi::clear_alert_notify(&mut session);
        }
    }

    fn stop_command_worker(&mut self) {
        if let Some(mut worker) = self.command_worker.take() {
            worker.stop_and_join();
        }
    }

    fn poll_events(&mut self) -> Result<(), EngineError> {
        let Some(session) = &mut self.session else {
            return Err(EngineError::EngineNotRunning);
        };

        // Alert polling remains owned by the runner because the notifier and
        // native Session are not shared across competing consumers.
        ffi::poll_events(session, &mut self.dispatcher);

        Ok(())
    }

    fn send_status(&mut self, status: EngineStatus) {
        let _ = self.status_tx.send(status);
    }

    async fn shutdown(&mut self) -> Result<(), EngineError> {
        self.send_status(EngineStatus::Stopping);

        // Stop in dependency order: port, session, then queued extensions.
        self.stop_command_worker();
        self.stop_session();
        self.dispatcher.shutdown();

        self.send_status(EngineStatus::Stopped);

        Ok(())
    }

    /// Runs the session until shutdown, command-channel closure, or an error.
    ///
    /// The native session is stopped before returning after a loop error. A
    /// failure while starting the session publishes [`EngineStatus::Failed`]
    /// and returns the original startup error.
    pub(crate) async fn run(mut self) -> Result<(), EngineError> {
        self.send_status(EngineStatus::Starting);

        if let Err(error) = self.start_session() {
            self.send_status(EngineStatus::Failed);
            return Err(error);
        }

        self.send_status(EngineStatus::Running);

        // The runner waits for a shutdown request from the command worker and
        // for native alert notifications. Commands themselves are received
        // and executed by the dedicated command worker.
        let loop_result = loop {
            let result = tokio::select! {
                _ = self.alert_notifier.notified() => {
                    self.poll_events().map(|_| ())
                }

                request = self.shutdown_rx.recv() => {
                    break Ok(request);
                }
            };

            match result {
                Ok(()) => {}
                Err(error) => break Err(error),
            }
        };

        // The loop has exited due to a shutdown command, a closed command
        // channel, or an error.
        match loop_result {
            Ok(shutdown_request) => {
                let shutdown_result = self.shutdown().await;

                if let Some(ShutdownRequest { reply: Some(reply) }) = shutdown_request {
                    let _ = reply.send(shutdown_result.clone());
                }

                shutdown_result
            }

            Err(error) => {
                let _ = self.shutdown().await;
                Err(error)
            }
        }
    }
}

impl Drop for EngineRunner {
    fn drop(&mut self) {
        self.stop_command_worker();
        self.stop_session();
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{mpsc, watch};

    use super::*;
    use crate::{SettingsPack, engine::dispatcher::EventDispatcher};

    #[tokio::test(flavor = "current_thread")]
    async fn closed_command_channel_runs_shutdown_fallback() {
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_tx);
        let (status_tx, status_rx) = watch::channel(EngineStatus::Idle);
        let runner = EngineRunner::new(
            SettingsPack::new(),
            EventDispatcher::new(Vec::new()),
            command_rx,
            status_tx,
        );

        assert_eq!(runner.run().await, Ok(()));
        assert_eq!(*status_rx.borrow(), EngineStatus::Stopped);
    }
}
