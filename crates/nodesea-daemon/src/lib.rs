//! Local daemon runtime for NodeSea.
//!
//! The daemon owns the local transport and BitTorrent engine lifecycle.
//!
//! It exposes capability-oriented gRPC services over the local transport and
//! is the only process boundary that owns the native BitTorrent session. The
//! command-line client should use this crate's IPC services rather than
//! accessing [`nodesea_bt`] directly.
#![warn(missing_docs)]

mod daemon;
mod engine;
mod errors;
mod transport;

pub use daemon::run;
pub use errors::DaemonError;
pub use transport::{Endpoint, TransportError};
