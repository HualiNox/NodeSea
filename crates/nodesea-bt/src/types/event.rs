//! Public domain events emitted by the BitTorrent engine.

use std::{net::SocketAddr, time::Duration};

use crate::{DhtInfoHash, DhtNode, NodeId, TorrentId};

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
        info_hash: DhtInfoHash,
        /// Address of the announcing peer.
        peer_ip: String,
        /// Port of the announcing peer.
        peer_port: u16,
    },
    /// Metadata was successfully received.
    MetadataReceived {
        /// Combined v1/v2 identity of the torrent whose metadata was received.
        torrent_id: TorrentId,
        /// Bencoded torrent info section.
        data: Vec<u8>,
    },
    /// Metadata failed to be received.
    MetadataFailed {
        /// Combined v1/v2 identity of the torrent whose metadata request failed.
        torrent_id: TorrentId,
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
        info_hash: DhtInfoHash,
    },
    /// A torrent was added successfully.
    AddTorrent {
        /// Combined v1/v2 identity of the added torrent.
        torrent_id: TorrentId,
        /// Status message reported by libtorrent.
        message: String,
    },
    /// Adding a torrent failed.
    AddTorrentError {
        /// Combined v1/v2 identity supplied to the add operation.
        torrent_id: TorrentId,
        /// Failure description reported by libtorrent.
        message: String,
        /// Numeric value of the libtorrent error code.
        error_value: i32,
        /// Error category name returned by libtorrent.
        error_category: String,
    },
    /// A torrent entered an error state.
    TorrentError {
        /// Combined v1/v2 identity of the torrent in error.
        torrent_id: TorrentId,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A file operation failed for a torrent.
    FileError {
        /// Combined v1/v2 identity of the affected torrent.
        torrent_id: TorrentId,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// Deleting a torrent failed.
    TorrentDeleteFailed {
        /// Combined v1/v2 identity of the torrent that could not be deleted.
        torrent_id: TorrentId,
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
        samples: Vec<DhtInfoHash>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InfoHashV1;

    #[test]
    fn test_bt_event_variants() {
        let dht_info_hash = DhtInfoHash::from_bytes([0x42; 20]);
        let torrent_id = TorrentId::from(InfoHashV1::from_bytes([0x42; 20]));

        let events = vec![
            BtEvent::DhtAnnounce {
                info_hash: dht_info_hash,
                peer_ip: "10.0.0.1".to_string(),
                peer_port: 8080,
            },
            BtEvent::MetadataReceived {
                torrent_id,
                data: vec![1, 2, 3],
            },
            BtEvent::MetadataFailed {
                torrent_id,
                message: "fetch failed".to_string(),
            },
            BtEvent::DhtStats {
                node_count: 128,
                local_ip: "127.0.0.1".to_string(),
                local_port: 6881,
            },
            BtEvent::DhtBootstrap,
            BtEvent::DhtGetPeers {
                info_hash: dht_info_hash,
            },
            BtEvent::AddTorrent {
                torrent_id,
                message: "torrent added".to_string(),
            },
            BtEvent::AddTorrentError {
                torrent_id,
                message: "add failed".to_string(),
                error_value: 1,
                error_category: "libtorrent".to_string(),
            },
            BtEvent::TorrentError {
                torrent_id,
                message: "torrent error".to_string(),
            },
            BtEvent::FileError {
                torrent_id,
                message: "file error".to_string(),
            },
            BtEvent::TorrentDeleteFailed {
                torrent_id,
                message: "delete failed".to_string(),
            },
            BtEvent::SessionError {
                message: "session error".to_string(),
            },
            BtEvent::ListenFailed {
                message: "listen error".to_string(),
            },
            BtEvent::UdpError {
                message: "udp error".to_string(),
            },
            BtEvent::DhtError {
                message: "dht error".to_string(),
            },
            BtEvent::AlertsDropped {
                message: "dropped alerts".to_string(),
            },
        ];

        for event in events {
            let cloned = event.clone();
            assert_eq!(event, cloned);
            assert!(!format!("{event:?}").is_empty());
        }
    }
}
