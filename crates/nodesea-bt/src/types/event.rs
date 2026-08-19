//! Public domain events emitted by the BitTorrent engine.

use std::{net::SocketAddr, time::Duration};

use crate::{DhtNode, NodeId};

use super::identity::InfoHash;

/// Direction of a DHT packet (incoming or outgoing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DhtDirection {
    /// Incoming packet received from a DHT node.
    Incoming,
    /// Outgoing packet sent to a DHT node.
    Outgoing,
}

/// A BitTorrent event produced by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtEvent {
    /// A DHT announce event was received.
    DhtAnnounce {
        /// Info hash associated with the announce request.
        info_hash: InfoHash,
        /// Address of the announcing peer.
        peer_ip: String,
        /// Port of the announcing peer.
        peer_port: u16,
    },
    /// Metadata was successfully received.
    MetadataReceived {
        /// Info hash of the torrent whose metadata was received.
        info_hash: InfoHash,
        /// Bencoded torrent info section.
        data: Vec<u8>,
    },
    /// Metadata failed to be received.
    MetadataFailed {
        /// Info hash of the torrent whose metadata request failed.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
    },
    /// The DHT stats event containing the number of nodes.
    DhtStats {
        /// Number of nodes currently present in the DHT routing table.
        node_count: u32,
        /// Local IP address used for DHT operations.
        local_ip: String,
        /// Local port used for DHT operations.
        local_port: u16,
    },
    /// The DHT bootstrap process has completed.
    DhtBootstrap,
    /// A DHT get peers event was received.
    DhtGetPeers {
        /// Info hash associated with the get peers request.
        info_hash: InfoHash,
    },
    /// A torrent was added successfully.
    AddTorrent {
        /// Info hash of the added torrent.
        info_hash: InfoHash,
        /// Status message reported by libtorrent.
        message: String,
    },
    /// Adding a torrent failed.
    AddTorrentError {
        /// Info hash supplied to the add operation.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
        /// Numeric value of the libtorrent error code.
        error_value: i32,
        /// Error category name returned by libtorrent.
        error_category: String,
    },
    /// A torrent entered an error state.
    TorrentError {
        /// Info hash of the torrent in error.
        info_hash: InfoHash,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A file operation failed for a torrent.
    FileError {
        /// Info hash of the affected torrent.
        info_hash: InfoHash,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// Deleting a torrent failed.
    TorrentDeleteFailed {
        /// Info hash of the torrent that could not be deleted.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
    },
    /// The session reported an error.
    SessionError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// Listening on the configured port failed.
    ListenFailed {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A UDP operation failed.
    UdpError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A DHT operation failed.
    DhtError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// The session dropped alerts before they were polled.
    AlertsDropped {
        /// Description of the dropped-alert condition.
        message: String,
    },
    /// Sample infohashes were received from a DHT node.
    DhtSampleInfohashes {
        /// The DHT node that sent the sample.
        node: DhtNode,
        /// Minimum interval before requesting another sample from this node.
        interval: Duration,
        /// Number of infohashes currently stored by the responding node.
        num_infohashes: u32,
        /// The sampled infohashes.
        samples: Vec<InfoHash>,
        /// The DHT nodes included in the sample.
        nodes: Vec<DhtNode>,
    },
    /// A raw DHT packet was received or sent for diagnostics.
    DhtPkt {
        /// Direction of the packet (incoming or outgoing).
        direction: DhtDirection,
        /// Address of the DHT node that sent or received the packet.
        endpoint: SocketAddr,
        /// Raw packet data.
        packet: Vec<u8>,
    },
    /// Live nodes were reported for one local DHT routing table.
    DhtLiveNodes {
        /// Node ID identifying the local DHT routing table.
        local_node_id: NodeId,
        /// Nodes currently present in that routing table.
        nodes: Vec<DhtNode>,
    },
}
