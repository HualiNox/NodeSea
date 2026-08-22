//! The single-owner async runtime for a native libtorrent session.

use crate::{
    SettingsPack,
    engine::{
        dispatcher::EventDispatcher,
        errors::EngineError,
        handle::{CommandReceiver, ControlFlow, EngineCommand, StatusSender},
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

/// Owns the native session and serializes commands with alert consumption.
pub(crate) struct EngineRunner {
    settings_pack: SettingsPack,
    session: Option<ffi::Session>,
    dispatcher: EventDispatcher,
    command_rx: CommandReceiver,

    status_tx: StatusSender,

    // Keeps the notifier address stable while the native callback is registered.
    alert_notifier: Box<ffi::AlertNotifier>,
}

impl EngineRunner {
    /// Creates a runner that will lazily create its native session in `run`.
    pub(crate) fn new(
        settings_pack: SettingsPack,
        dispatcher: EventDispatcher,
        command_rx: CommandReceiver,
        status_tx: StatusSender,
    ) -> Self {
        Self {
            dispatcher,
            settings_pack,
            session: None,
            command_rx,
            status_tx,
            alert_notifier: Box::new(ffi::AlertNotifier::new()),
        }
    }

    fn start_session(&mut self) -> Result<(), EngineError> {
        let mut session = ffi::start_session(self.settings_pack.clone())
            .map_err(EngineError::SessionStartError)?;
        ffi::set_alert_notify(&mut session, self.alert_notifier.as_ref());
        self.session = Some(session);
        Ok(())
    }

    fn handle_command(&mut self, command: EngineCommand) -> Result<ControlFlow, EngineError> {
        match command {
            EngineCommand::PostDhtStats { reply } => {
                let result = self
                    .session
                    .as_ref()
                    .map(ffi::post_dht_stats)
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::PostSessionStats { reply } => {
                let result = self
                    .session
                    .as_ref()
                    .map(ffi::post_session_stats)
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::PostDhtLiveNodes { reply } => {
                let result = self
                    .session
                    .as_ref()
                    .map(ffi::post_dht_live_nodes)
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::FetchMetadata { torrent_id, reply } => {
                let result = self
                    .session
                    .as_mut()
                    .map(|session| ffi::fetch_metadata(session, &torrent_id))
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::CancelFetchMetadata { torrent_id, reply } => {
                let result = self
                    .session
                    .as_mut()
                    .map(|session| ffi::cancel_fetch(session, &torrent_id))
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::PostDhtSampleInfohashes {
                endpoint,
                target,
                reply,
            } => {
                let result = self
                    .session
                    .as_ref()
                    .map(|session| ffi::post_dht_sample_infohashes(session, &endpoint, &target))
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(ControlFlow::Continue)
            }

            EngineCommand::Shutdown { reply } => Ok(ControlFlow::Shutdown(reply)),
        }
    }

    fn stop_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            ffi::clear_alert_notify(&mut session);
        }
    }

    fn poll_events(&mut self) -> Result<(), EngineError> {
        let Some(session) = &mut self.session else {
            return Err(EngineError::EngineNotRunning);
        };

        ffi::poll_events(session, &mut self.dispatcher);

        Ok(())
    }

    fn send_status(&mut self, status: EngineStatus) {
        let _ = self.status_tx.send(status);
    }

    async fn shutdown(&mut self) -> Result<(), EngineError> {
        self.send_status(EngineStatus::Stopping);

        self.stop_session();

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

        // The main loop waits for commands and native alerts. A closed command
        // channel enters the shutdown sequence. An error also enters the
        // shutdown sequence before the error is returned.
        let loop_result = loop {
            let result = tokio::select! {
                command = self.command_rx.recv() => {
                    match command {
                        Some(command) => self.handle_command(command),
                        None => break Ok(None),
                    }
                }

                _ = self.alert_notifier.notified() => {
                    self.poll_events()
                        .map(|_| ControlFlow::Continue)
                }
            };

            match result {
                Ok(ControlFlow::Continue) => {}
                Ok(ControlFlow::Shutdown(reply)) => break Ok(Some(reply)),
                Err(error) => break Err(error),
            }
        };

        // The loop has exited due to a shutdown command, a closed command
        // channel, or an error.
        match loop_result {
            Ok(shutdown_reply) => {
                let shutdown_result = self.shutdown().await;

                if let Some(reply) = shutdown_reply {
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
