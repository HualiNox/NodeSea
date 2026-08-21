//! Rust bindings and event model for the Nodesea BitTorrent engine.
#![warn(missing_docs)]

mod engine;
mod ffi;
mod types;

#[cfg(feature = "bench-internals")]
#[path = "../benches/support/ffi_bridge.rs"]
/// Internal CXX bridge used by the FFI dispatch benchmark.
pub mod bench;

pub use engine::Engine;
pub use types::{
    AddTorrent, AddTorrentError, AlertsDropped, BtEvent, BtEventKind, DhtAnnounce, DhtBootstrap,
    DhtDirection, DhtError, DhtGetPeers, DhtInfoHash, DhtLiveNodes, DhtNode, DhtPkt,
    DhtSampleInfohashes, DhtStats, DhtTarget, EventCollector, EventSink, FileError, InfoHashV1,
    InfoHashV2, ListenFailed, MetadataFailed, MetadataReceived, NodeId, SessionError,
    TorrentDeleteFailed, TorrentError, TorrentId, UdpError,
};
