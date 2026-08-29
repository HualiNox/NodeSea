//! Shared platform helpers for NodeSea processes.
//!
//! This crate is the single owner of process-local address conventions so the
//! daemon and its clients resolve the same endpoint without duplicating
//! platform-specific rules.
//!
//! The public API intentionally stays small: callers can resolve the current
//! user's home directory on macOS and the default local daemon socket on the
//! supported Unix platforms.

#![warn(missing_docs)]

mod platform;

pub use platform::{HelperError, default_socket_path};

#[cfg(target_os = "macos")]
pub use platform::current_user_home;
