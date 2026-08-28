//! Public BitTorrent engine facade.

mod builder;
mod config;
mod dispatcher;
mod errors;
mod extension;
mod handle;
mod runner;

pub use builder::EngineBuilder;
pub use config::*;
pub use errors::EngineError;
pub use extension::EngineExtension;
pub use handle::{EngineHandle, StatusReceiver};
pub use runner::EngineStatus;
use tokio::sync::watch;

use crate::engine::{
    dispatcher::EventDispatcher,
    extension::EngineExtensionBox,
    handle::{CommandReceiver, CommandSender},
    runner::EngineRunner,
};

/// A BitTorrent engine backed by a libtorrent session.
pub struct Engine {
    // These values are consumed by `run`; keeping them here allows the public
    // handle to be created before the runner starts.
    config: EngineConfig,
    extensions: Vec<EngineExtensionBox>,

    // Sender used by handles to submit commands to the engine runner.
    command_tx: CommandSender,
    command_rx: CommandReceiver,

    // Keeps the engine status updated for external observers.
    status_tx: watch::Sender<EngineStatus>,
    status_rx: watch::Receiver<EngineStatus>,
}

impl Engine {
    /// Creates a builder for a BitTorrent engine.
    ///
    /// The builder starts with an empty [`SettingsPack`]. Configure it with
    /// [`EngineBuilder::set_settings_pack`] before calling
    /// [`EngineBuilder::build`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::Engine;
    ///
    /// let _engine = Engine::builder().build();
    /// ```
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    pub(crate) fn new(
        config: EngineConfig,
        extensions: Vec<EngineExtensionBox>,
        command_tx: CommandSender,
        command_rx: CommandReceiver,
        status_tx: watch::Sender<EngineStatus>,
        status_rx: watch::Receiver<EngineStatus>,
    ) -> Self {
        // The builder owns the channels until `run` transfers the receiver to
        // EngineRunner.
        Self {
            config,
            extensions,
            command_rx,
            command_tx,
            status_tx,
            status_rx,
        }
    }

    /// Returns a cloneable handle for sending commands and observing the
    /// running engine.
    ///
    /// The handle does not own the native session. Commands are executed by
    /// the command worker owned by [`Engine::run`].
    pub fn handle(&self) -> EngineHandle {
        EngineHandle::new(self.command_tx.clone(), self.status_rx.clone())
    }

    /// Starts the native session and runs the engine event loop.
    ///
    /// The session is created lazily here so that alert notification is
    /// installed before the engine begins consuming alerts. The future ends
    /// after orderly shutdown or an engine error. If the command channel is
    /// closed, the runner follows the same shutdown path as an explicit
    /// shutdown request.
    pub async fn run(self) -> Result<(), EngineError> {
        // Extensions move into the event worker and are not called from the
        // native session runner.
        let runner = EngineRunner::new(
            self.config.settings_pack(),
            EventDispatcher::new(self.extensions),
            self.command_rx,
            self.status_tx,
        );

        runner.run().await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use super::*;
    use crate::{BtEvent, BtEventKind, InfoHashV1, InfoHashV2, TorrentId};

    struct EventRecorder {
        events: Arc<Mutex<Vec<BtEventKind>>>,
    }

    impl EngineExtension for EventRecorder {
        fn on_event(&mut self, event: &BtEvent) {
            self.events.lock().unwrap().push(event.kind().clone());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn engine_handles_commands_and_shutdown() {
        tokio::task::LocalSet::new()
            .run_until(engine_handles_commands_and_shutdown_inner())
            .await;
    }

    async fn engine_handles_commands_and_shutdown_inner() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let engine = Engine::builder()
            .add_extension(EventRecorder {
                events: Arc::clone(&events),
            })
            .build();
        let handle = engine.handle();
        let mut status = handle.subscribe_status();
        let runner = tokio::task::spawn_local(engine.run());

        status
            .wait_for(|value| *value == EngineStatus::Running)
            .await
            .unwrap();

        assert!(handle.post_dht_stats().await.unwrap());

        let v1_id = TorrentId::new(Some(InfoHashV1::from_bytes([0xef; 20])), None);
        assert!(handle.post_fetch_metadata(v1_id).await.unwrap());

        let add_completed = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if events.lock().unwrap().iter().any(|event| {
                    matches!(
                        event,
                        BtEventKind::AddTorrent(add)
                            if add.torrent_id() == &v1_id && add.error().is_none()
                    )
                }) {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or(false);
        assert!(
            add_completed,
            "metadata add success event was not dispatched"
        );
        assert!(!handle.post_fetch_metadata(v1_id).await.unwrap());
        assert!(handle.post_cancel_fetch_metadata(v1_id).await.unwrap());

        let v2_id = TorrentId::new(None, Some(InfoHashV2::from_bytes([0xcd; 32])));
        assert!(handle.post_fetch_metadata(v2_id).await.unwrap());
        assert!(!handle.post_fetch_metadata(v2_id).await.unwrap());
        assert!(handle.post_cancel_fetch_metadata(v2_id).await.unwrap());

        let hybrid_id = TorrentId::new(
            Some(InfoHashV1::from_bytes([0xab; 20])),
            Some(InfoHashV2::from_bytes([0xcd; 32])),
        );
        assert!(handle.post_fetch_metadata(hybrid_id).await.unwrap());
        assert!(!handle.post_fetch_metadata(hybrid_id).await.unwrap());
        assert!(handle.post_cancel_fetch_metadata(hybrid_id).await.unwrap());

        handle.shutdown().await.unwrap();
        assert!(runner.await.unwrap().is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn engine_dispatches_dht_stats_event_to_extension() {
        tokio::task::LocalSet::new()
            .run_until(engine_dispatches_dht_stats_event_to_extension_inner())
            .await;
    }

    async fn engine_dispatches_dht_stats_event_to_extension_inner() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let engine = Engine::builder()
            .add_extension(EventRecorder {
                events: Arc::clone(&events),
            })
            .build();
        let handle = engine.handle();
        let mut status = handle.subscribe_status();
        let runner = tokio::task::spawn_local(engine.run());

        status
            .wait_for(|value| *value == EngineStatus::Running)
            .await
            .unwrap();

        assert!(handle.post_dht_stats().await.unwrap());

        let received = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if events
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|event| matches!(event, BtEventKind::DhtStats(_)))
                {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or(false);

        handle.shutdown().await.unwrap();
        assert!(runner.await.unwrap().is_ok());
        assert!(received, "DhtStats event was not dispatched");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn engine_dispatches_session_stats_event_to_extension() {
        tokio::task::LocalSet::new()
            .run_until(engine_dispatches_session_stats_event_to_extension_inner())
            .await;
    }

    async fn engine_dispatches_session_stats_event_to_extension_inner() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let engine = Engine::builder()
            .add_extension(EventRecorder {
                events: Arc::clone(&events),
            })
            .build();
        let handle = engine.handle();
        let mut status = handle.subscribe_status();
        let runner = tokio::task::spawn_local(engine.run());

        status
            .wait_for(|value| *value == EngineStatus::Running)
            .await
            .unwrap();

        assert!(handle.post_session_stats().await.unwrap());

        let received = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if events
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|event| matches!(event, BtEventKind::SessionStats(_)))
                {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or(false);

        handle.shutdown().await.unwrap();
        assert!(runner.await.unwrap().is_ok());
        assert!(received, "SessionStats event was not dispatched");
    }
}
