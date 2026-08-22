//! Errors produced while starting, controlling, or stopping the engine.

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Hash)]
/// Errors returned by the BitTorrent engine facade.
pub enum EngineError {
    /// The native libtorrent session could not be created.
    #[error("Failed to start the engine session: {0}")]
    SessionStartError(String),

    /// An operation required a session that is no longer running.
    #[error("Engine is not running")]
    EngineNotRunning,

    /// The engine command receiver has been dropped.
    #[error("Engine command channel is closed")]
    CommandChannelClosed,

    /// The runner stopped before replying to a command.
    #[error("Engine command response channel is closed")]
    CommandResponseClosed,
}
