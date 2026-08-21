//! Rust bindings and event model for the Nodesea BitTorrent engine.
#![warn(missing_docs)]

mod engine;
mod ffi;
mod types;

pub use engine::Engine;
pub use types::{
    BtEvent, DhtDirection, DhtInfoHash, DhtNode, DhtTarget, EventCollector, EventSink, InfoHashV1,
    InfoHashV2, NodeId, TorrentId,
};
