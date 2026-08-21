//! Rust bindings and event model for the Nodesea BitTorrent engine.
#![warn(missing_docs)]

mod engine;
mod ffi;
mod types;

pub use engine::Engine;
pub use types::{
    AddTorrent, AddTorrentError, AlertsDropped, BtEvent, BtEventKind, DhtAnnounce, DhtBootstrap,
    DhtDirection, DhtError, DhtGetPeers, DhtInfoHash, DhtLiveNodes, DhtNode, DhtPkt,
    DhtSampleInfohashes, DhtStats, DhtTarget, EventCollector, EventSink, FileError, InfoHashV1,
    InfoHashV2, ListenFailed, MetadataFailed, MetadataReceived, NodeId, SessionError,
    TorrentDeleteFailed, TorrentError, TorrentId, UdpError,
};
