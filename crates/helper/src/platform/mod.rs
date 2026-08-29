//! Platform-specific NodeSea path resolution.
//!
//! The endpoint is intentionally derived from the effective user rather than
//! from the current working directory. This keeps CLI and daemon addressing
//! stable when either process is launched by a service manager.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, thiserror::Error)]
/// Errors returned while resolving platform paths for NodeSea.
pub enum HelperError {
    /// The current user's home directory could not be resolved.
    #[error("home directory is not available")]
    MissingHomeDirectory,

    /// The current user's runtime directory is not available.
    #[error("XDG_RUNTIME_DIR is not available")]
    MissingRuntimeDirectory,
}

#[cfg(target_os = "linux")]
pub use linux::default_socket_path;

#[cfg(target_os = "macos")]
pub use macos::{current_user_home, default_socket_path};
