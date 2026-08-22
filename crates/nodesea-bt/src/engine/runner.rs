//! The single-owner async runtime for a native libtorrent session.

use crate::{
    SettingsPack,
    engine::{
        dispatcher::EventDispatcher,
        errors::EngineError,
        handle::{CommandReceiver, EngineCommand},
    },
    ffi,
};

/// Owns the native session and serializes commands with alert consumption.
pub(crate) struct EngineRunner {
    settings_pack: SettingsPack,
    session: Option<ffi::Session>,
    dispatcher: EventDispatcher,
    command_rx: CommandReceiver,

    // Keeps the notifier address stable while the native callback is registered.
    alert_notifier: Box<ffi::AlertNotifier>,
}

impl EngineRunner {
    /// Creates a runner that will lazily create its native session in `run`.
    pub(crate) fn new(
        settings_pack: SettingsPack,
        dispatcher: EventDispatcher,
        command_rx: CommandReceiver,
    ) -> Self {
        Self {
            dispatcher,
            settings_pack,
            session: None,
            command_rx,
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

    fn handle_command(&mut self, command: EngineCommand) -> Result<bool, EngineError> {
        match command {
            EngineCommand::PostDhtStats { reply } => {
                let result = self
                    .session
                    .as_ref()
                    .map(ffi::post_dht_stats)
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(false)
            }

            EngineCommand::PostDhtLiveNodes { reply } => {
                let result = self
                    .session
                    .as_ref()
                    .map(ffi::post_dht_live_nodes)
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(false)
            }

            EngineCommand::FetchMetadata { torrent_id, reply } => {
                let result = self
                    .session
                    .as_mut()
                    .map(|session| ffi::fetch_metadata(session, &torrent_id))
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(false)
            }

            EngineCommand::CancelFetchMetadata { torrent_id, reply } => {
                let result = self
                    .session
                    .as_mut()
                    .map(|session| ffi::cancel_fetch(session, &torrent_id))
                    .unwrap_or(false);
                let _ = reply.send(result);

                Ok(false)
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

                Ok(false)
            }

            EngineCommand::Shutdown { reply } => {
                self.stop_session();
                let _ = reply.send(());

                Ok(true)
            }
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

    /// Runs the session until shutdown, command-channel closure, or an error.
    pub(crate) async fn run(mut self) -> Result<(), EngineError> {
        self.start_session()?;

        loop {
            tokio::select! {
                command = self.command_rx.recv() => {
                    match command {
                        Some(command) => {
                            if self.handle_command(command)? {
                                return Ok(());
                            }
                        }
                        None => {
                            return Ok(())
                        },
                    }
                }

                _ = self.alert_notifier.notified() => {
                    self.poll_events()?
                }
            }
        }
    }
}

impl Drop for EngineRunner {
    fn drop(&mut self) {
        self.stop_session();
    }
}
