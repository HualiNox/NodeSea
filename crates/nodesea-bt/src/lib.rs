//! Rust bindings and event model for the NodeSea BitTorrent engine.
//!
//! This crate owns the native libtorrent session and exposes the safe engine
//! handle, settings, domain types, and events used by the daemon. The native
//! session remains behind the engine boundary; clients should communicate
//! with the daemon instead of constructing a session directly.
#![warn(missing_docs)]

mod engine;
mod ffi;
mod types;

pub use engine::*;
pub use types::*;
