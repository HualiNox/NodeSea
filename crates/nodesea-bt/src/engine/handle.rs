//! Commands and control handles for a running engine.

use std::net::SocketAddr;

use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot, watch,
};

use crate::{
    DhtTarget, TorrentId,
    engine::{errors::EngineError, runner::EngineStatus},
};

/// A command sent from an [`EngineHandle`] to the engine runner.
pub(crate) enum EngineCommand {
    /// Starts a metadata fetch for a torrent identity.
    FetchMetadata {
        /// The v1 and/or v2 identity used for the fetch.
        torrent_id: TorrentId,
        /// Receives whether libtorrent accepted the request.
        reply: oneshot::Sender<bool>,
    },
    /// Cancels a metadata fetch for a torrent identity.
    CancelFetchMetadata {
        /// The identity used to find the active fetch.
        torrent_id: TorrentId,
        /// Receives whether an active fetch was cancelled.
        reply: oneshot::Sender<bool>,
    },
    /// Requests an asynchronous DHT statistics alert.
    PostDhtStats {
        /// Receives whether the request was accepted.
        reply: oneshot::Sender<bool>,
    },
    /// Requests asynchronous session statistics.
    PostSessionStats {
        /// Receives whether the request was accepted.
        reply: oneshot::Sender<bool>,
    },
    /// Requests BEP 51 infohash samples from a remote DHT endpoint.
    PostDhtSampleInfohashes {
        /// The remote DHT endpoint to query.
        endpoint: SocketAddr,
        /// The key-space traversal target.
        target: DhtTarget,
        /// Receives whether the request was accepted.
        reply: oneshot::Sender<bool>,
    },
    /// Requests live-node snapshots from the local DHT routing tables.
    PostDhtLiveNodes {
        /// Receives whether the request was accepted.
        reply: oneshot::Sender<bool>,
    },
    /// Requests orderly shutdown of the engine runner.
    Shutdown {
        /// Completes after the runner clears its native session.
        reply: oneshot::Sender<Result<(), EngineError>>,
    },
}

/// A control flow signal sent from the engine runner to its task.
pub(crate) enum ControlFlow {
    /// Continue processing commands and native alerts.
    Continue,
    /// Stop processing commands and enter the shutdown sequence.
    Shutdown(oneshot::Sender<Result<(), EngineError>>),
}

/// Sender used by handles to submit commands to the engine runner.
pub(crate) type CommandSender = Sender<EngineCommand>;

/// Receiver owned by the engine runner for submitted commands.
pub(crate) type CommandReceiver = Receiver<EngineCommand>;

pub(crate) type StatusSender = watch::Sender<EngineStatus>;
/// Receives lifecycle updates from an engine runner.
pub type StatusReceiver = watch::Receiver<EngineStatus>;

#[derive(Clone)]
/// A cloneable handle for controlling and observing a BitTorrent engine.
///
/// Command methods wait while the engine is `Idle` or `Starting`, and return
/// [`EngineError::EngineNotRunning`] once shutdown has reached `Stopping` or
/// `Stopped`, or startup has reached `Failed`.
pub struct EngineHandle {
    commands: CommandSender,
    status: StatusReceiver,
}

impl EngineHandle {
    /// Creates a handle from an engine command sender and status receiver.
    ///
    /// This is primarily used by [`super::Engine::handle`].
    pub(crate) fn new(commands: CommandSender, status: StatusReceiver) -> Self {
        Self { commands, status }
    }

    async fn request<F>(&self, build: F) -> Result<bool, EngineError>
    where
        F: FnOnce(oneshot::Sender<bool>) -> EngineCommand,
    {
        let mut status = self.status.clone();

        let status = *status
            .wait_for(|status| {
                matches!(
                    status,
                    EngineStatus::Running
                        | EngineStatus::Stopping
                        | EngineStatus::Stopped
                        | EngineStatus::Failed
                )
            })
            .await
            .map_err(|_| EngineError::EngineNotRunning)?;

        if matches!(
            status,
            EngineStatus::Stopping | EngineStatus::Stopped | EngineStatus::Failed
        ) {
            return Err(EngineError::EngineNotRunning);
        }

        let (reply_tx, reply_rx) = oneshot::channel();

        self.commands
            .send(build(reply_tx))
            .await
            .map_err(|_| EngineError::CommandChannelClosed)?;

        reply_rx
            .await
            .map_err(|_| EngineError::CommandResponseClosed)
    }

    /// Posts a request to start fetching metadata for a torrent identity.
    ///
    /// Returns `true` when libtorrent accepts the request and `false` when it
    /// rejects it, for example because an equivalent fetch is already active.
    pub async fn post_fetch_metadata(&self, torrent_id: TorrentId) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::FetchMetadata { torrent_id, reply })
            .await
    }

    /// Posts a request to cancel an active metadata fetch for a torrent identity.
    pub async fn post_cancel_fetch_metadata(
        &self,
        torrent_id: TorrentId,
    ) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::CancelFetchMetadata { torrent_id, reply })
            .await
    }

    /// Requests an asynchronous DHT statistics alert.
    ///
    /// The returned `bool` indicates whether the request was accepted by the
    /// native session; the resulting statistics arrive as an engine event.
    pub async fn post_dht_stats(&self) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::PostDhtStats { reply })
            .await
    }

    /// Requests an asynchronous session statistics alert.
    ///
    /// The resulting statistics arrive as an engine event.
    pub async fn post_session_stats(&self) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::PostSessionStats { reply })
            .await
    }

    /// Requests BEP 51 infohash samples from a remote DHT endpoint.
    ///
    /// The target controls traversal through the remote DHT key space. The
    /// resulting samples arrive asynchronously as engine events.
    pub async fn post_dht_sample_infohashes(
        &self,
        endpoint: SocketAddr,
        target: DhtTarget,
    ) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::PostDhtSampleInfohashes {
            endpoint,
            target,
            reply,
        })
        .await
    }

    /// Requests live-node snapshots from the local DHT routing tables.
    ///
    /// Returns `false` when the local session has no active DHT node and no
    /// snapshot request can be submitted. Results arrive asynchronously as
    /// engine events.
    pub async fn post_dht_live_nodes(&self) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::PostDhtLiveNodes { reply })
            .await
    }

    /// Requests an orderly shutdown of the engine runner.
    pub async fn shutdown(&self) -> Result<(), EngineError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.commands
            .send(EngineCommand::Shutdown { reply: reply_tx })
            .await
            .map_err(|_| EngineError::CommandChannelClosed)?;

        reply_rx
            .await
            .map_err(|_| EngineError::CommandResponseClosed)?
    }

    /// Returns the latest observed engine status.
    ///
    /// The value may be stale if the runner has not published a newer
    /// transition yet.
    pub fn status(&self) -> EngineStatus {
        *self.status.borrow()
    }

    /// Returns a receiver for future engine status updates.
    pub fn subscribe_status(&self) -> StatusReceiver {
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::{mpsc, watch};

    #[tokio::test]
    async fn command_returns_channel_closed_when_runner_is_gone() {
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_rx);

        let handle = EngineHandle::new(command_tx, watch::channel(EngineStatus::Running).1);
        let error = handle.post_dht_stats().await.unwrap_err();

        assert_eq!(error, EngineError::CommandChannelClosed);
    }

    #[tokio::test]
    async fn command_waits_until_engine_is_running() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let (status_tx, status_rx) = watch::channel(EngineStatus::Idle);
        let handle = EngineHandle::new(command_tx, status_rx);

        let request = tokio::spawn(async move { handle.post_dht_stats().await });
        tokio::task::yield_now().await;
        assert!(!request.is_finished());
        assert!(command_rx.try_recv().is_err());

        status_tx.send(EngineStatus::Running).unwrap();

        let Some(EngineCommand::PostDhtStats { reply }) = command_rx.recv().await else {
            panic!("expected post DHT stats command");
        };
        reply.send(true).unwrap();

        assert_eq!(request.await.unwrap(), Ok(true));
    }

    #[tokio::test]
    async fn command_is_rejected_once_engine_is_stopping() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let handle = EngineHandle::new(command_tx, watch::channel(EngineStatus::Stopping).1);

        assert_eq!(
            handle.post_dht_stats().await,
            Err(EngineError::EngineNotRunning)
        );
        assert!(command_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn command_is_rejected_after_startup_failed() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let handle = EngineHandle::new(command_tx, watch::channel(EngineStatus::Failed).1);

        assert_eq!(
            handle.post_dht_stats().await,
            Err(EngineError::EngineNotRunning)
        );
        assert!(command_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn shutdown_waits_for_runner_response() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let handle = EngineHandle::new(command_tx, watch::channel(EngineStatus::Idle).1);

        let shutdown = tokio::spawn(async move { handle.shutdown().await });

        let Some(EngineCommand::Shutdown { reply }) = command_rx.recv().await else {
            panic!("expected shutdown command");
        };
        reply.send(Ok(())).unwrap();

        assert_eq!(shutdown.await.unwrap(), Ok(()));
    }
}
