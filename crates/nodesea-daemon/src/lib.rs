//! Local daemon runtime for NodeSea.
//!
//! The daemon owns the local transport and BitTorrent engine lifecycle.
#![warn(missing_docs)]

mod daemon;
mod errors;
mod transport;

pub use daemon::run;
pub use errors::DaemonError;
pub use transport::{Endpoint, TransportError};
