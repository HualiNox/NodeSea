//! Commands and control handles for a running engine.

use std::net::SocketAddr;

use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot,
};

use crate::{DhtTarget, TorrentId, engine::errors::EngineError};

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
        reply: oneshot::Sender<()>,
    },
}

/// Sender used by handles to submit commands to the engine runner.
pub(crate) type CommandSender = Sender<EngineCommand>;

/// Receiver owned by the engine runner for submitted commands.
pub(crate) type CommandReceiver = Receiver<EngineCommand>;

#[derive(Clone)]
/// A cloneable handle for controlling a running BitTorrent engine.
pub struct EngineHandle {
    commands: CommandSender,
}

impl EngineHandle {
    /// Creates a handle from an engine command sender.
    ///
    /// This is primarily used by [`super::Engine::handle`].
    pub(crate) fn new(commands: CommandSender) -> Self {
        Self { commands }
    }

    async fn request<F>(&self, build: F) -> Result<bool, EngineError>
    where
        F: FnOnce(oneshot::Sender<bool>) -> EngineCommand,
    {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.commands
            .send(build(reply_tx))
            .await
            .map_err(|_| EngineError::CommandChannelClosed)?;

        reply_rx
            .await
            .map_err(|_| EngineError::CommandResponseClosed)
    }

    /// Starts fetching metadata for a torrent identity.
    ///
    /// Returns `true` when libtorrent accepts the request and `false` when it
    /// rejects it, for example because an equivalent fetch is already active.
    pub async fn fetch_metadata(&self, torrent_id: TorrentId) -> Result<bool, EngineError> {
        self.request(|reply| EngineCommand::FetchMetadata { torrent_id, reply })
            .await
    }

    /// Cancels an active metadata fetch for a torrent identity.
    pub async fn cancel_fetch_metadata(&self, torrent_id: TorrentId) -> Result<bool, EngineError> {
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
            .map_err(|_| EngineError::CommandResponseClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn command_returns_channel_closed_when_runner_is_gone() {
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_rx);

        let handle = EngineHandle::new(command_tx);
        let error = handle.post_dht_stats().await.unwrap_err();

        assert_eq!(error, EngineError::CommandChannelClosed);
    }

    #[tokio::test]
    async fn shutdown_waits_for_runner_response() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let handle = EngineHandle::new(command_tx);

        let shutdown = tokio::spawn(async move { handle.shutdown().await });

        let Some(EngineCommand::Shutdown { reply }) = command_rx.recv().await else {
            panic!("expected shutdown command");
        };
        reply.send(()).unwrap();

        assert_eq!(shutdown.await.unwrap(), Ok(()));
    }
}
