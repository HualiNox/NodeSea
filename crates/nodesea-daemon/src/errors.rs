use nodesea_bt::EngineError;

use crate::TransportError;

#[derive(Debug, thiserror::Error)]
/// Errors returned while running the NodeSea daemon.
pub enum DaemonError {
    /// A local transport operation failed.
    #[error(transparent)]
    Transport(#[from] TransportError),

    /// The BitTorrent engine returned an error.
    #[error(transparent)]
    Engine(#[from] EngineError),

    /// The local engine task could not be joined.
    #[error("engine task failed: {0}")]
    EngineTask(#[from] tokio::task::JoinError),
}
