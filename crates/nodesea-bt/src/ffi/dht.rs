//! Private DHT CXX wire models and their domain conversions.

use std::{net::SocketAddr, time::Duration};

use crate::{BtEvent, DhtNode, InfoHash};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {

    /// Direction of a raw DHT packet on the wire.
    enum DhtDirectionPayload {
        /// Packet received from a DHT node.
        Incoming,
        /// Packet sent to a DHT node.
        Outgoing,
    }

    /// Payload for a DHT announce alert.
    pub(super) struct DhtAnnouncePayload {
        /// Torrent info hash associated with the announce.
        info_hash: [u8; 20],
        /// IP address announced by the remote peer.
        peer_ip: String,
        /// Port announced by the remote peer.
        peer_port: u16,
    }

    /// Payload for DHT statistics.
    pub(super) struct DhtStatsPayload {
        /// Number of nodes currently in the DHT routing table.
        node_count: u32,
        /// Local IP address used by the DHT socket.
        local_ip: String,
        /// Local UDP port used by the DHT socket.
        local_port: u16,
    }

    /// Payload for a DHT get-peers alert.
    pub(super) struct DhtGetPeersPayload {
        /// Torrent info hash associated with the lookup.
        info_hash: [u8; 20],
    }

    /// Wire representation of a UDP endpoint.
    pub(super) struct UdpEndpoint {
        /// Numeric IP address without a port suffix.
        address: String,
        /// UDP port number.
        port: u16,
    }

    /// Wire representation of a DHT node and its endpoint.
    pub(super) struct DhtNodePayload {
        /// DHT node identifier.
        node_id: [u8; 20],
        /// Network endpoint associated with the node.
        endpoint: UdpEndpoint,
    }

    /// Private fixed-size wire adapter for a sampled infohash.
    ///
    /// CXX cannot represent `Vec<[u8; 20]>` as a shared field. This wrapper
    /// exists only for the FFI payload and is converted to [`InfoHash`] before
    /// reaching the domain event.
    pub(super) struct SampleInfoHash {
        bytes: [u8; 20],
    }

    /// Payload for a DHT sample infohashes alert.
    ///
    /// This alert contains sampled infohashes and DHT nodes from a BEP 51
    /// DHT request.
    pub(super) struct DhtSampleInfohashesPayload {
        /// Remote DHT node that returned the sample.
        node: DhtNodePayload,
        /// Minimum interval before querying this node again, in seconds.
        interval_secs: i64,
        /// Number of infohashes currently stored by the remote node.
        num_infohashes: i32,
        /// Sampled infohashes represented through the CXX fixed-array adapter.
        samples: Vec<SampleInfoHash>,
        /// Additional DHT nodes returned for key-space traversal.
        nodes: Vec<DhtNodePayload>,
    }

    /// Payload for a DHT packet alert.
    ///
    /// This alert contains a raw DHT packet for diagnostics.
    pub(super) struct DhtPktPayload {
        /// Direction in which the packet crossed the DHT socket.
        direction: DhtDirectionPayload,
        /// Remote DHT endpoint associated with the packet.
        endpoint: UdpEndpoint,
        /// Verbatim packet bytes.
        packet: Vec<u8>,
    }

    /// Wire payload containing one local DHT routing table snapshot.
    pub(super) struct DhtLiveNodes {
        /// Local DHT node ID owning the routing table.
        local_node_id: [u8; 20],
        /// Nodes currently present in the routing table.
        nodes: Vec<DhtNodePayload>,
    }
}

// These are narrow, named entries for the canonical callback bridge. The
// bridge module itself remains private and no wildcard re-export is used.
pub(super) use bridge::{
    DhtAnnouncePayload, DhtGetPeersPayload, DhtLiveNodes, DhtPktPayload,
    DhtSampleInfohashesPayload, DhtStatsPayload, UdpEndpoint,
};

impl From<bridge::DhtAnnouncePayload> for BtEvent {
    fn from(value: bridge::DhtAnnouncePayload) -> Self {
        Self::DhtAnnounce {
            info_hash: value.info_hash.into(),
            peer_ip: value.peer_ip,
            peer_port: value.peer_port,
        }
    }
}

impl From<bridge::DhtStatsPayload> for BtEvent {
    fn from(value: bridge::DhtStatsPayload) -> Self {
        Self::DhtStats {
            node_count: value.node_count,
            local_ip: value.local_ip,
            local_port: value.local_port,
        }
    }
}

impl From<bridge::DhtGetPeersPayload> for BtEvent {
    fn from(value: bridge::DhtGetPeersPayload) -> Self {
        Self::DhtGetPeers {
            info_hash: value.info_hash.into(),
        }
    }
}

impl bridge::UdpEndpoint {
    pub(super) fn from_socket_addr(addr: &SocketAddr) -> Self {
        Self {
            address: addr.ip().to_string(),
            port: addr.port(),
        }
    }
}

impl From<bridge::UdpEndpoint> for SocketAddr {
    fn from(value: bridge::UdpEndpoint) -> Self {
        SocketAddr::new(value.address.parse().unwrap(), value.port)
    }
}

impl From<bridge::SampleInfoHash> for InfoHash {
    fn from(value: bridge::SampleInfoHash) -> Self {
        Self::from_bytes(value.bytes)
    }
}

impl From<bridge::DhtNodePayload> for DhtNode {
    fn from(value: bridge::DhtNodePayload) -> Self {
        Self {
            node_id: value.node_id.into(),
            endpoint: value.endpoint.into(),
        }
    }
}

impl From<bridge::DhtSampleInfohashesPayload> for BtEvent {
    fn from(value: bridge::DhtSampleInfohashesPayload) -> Self {
        Self::DhtSampleInfohashes {
            node: value.node.into(),
            interval: Duration::from_secs(value.interval_secs as u64),
            num_infohashes: value.num_infohashes as u32,
            samples: value.samples.into_iter().map(|s| s.into()).collect(),
            nodes: value.nodes.into_iter().map(|n| n.into()).collect(),
        }
    }
}

impl From<bridge::DhtPktPayload> for BtEvent {
    fn from(value: bridge::DhtPktPayload) -> Self {
        Self::DhtPkt {
            direction: match value.direction {
                bridge::DhtDirectionPayload::Incoming => crate::DhtDirection::Incoming,
                bridge::DhtDirectionPayload::Outgoing => crate::DhtDirection::Outgoing,
                // unreachable
                _ => unreachable!("invalid DHT direction from C++"),
            },
            endpoint: value.endpoint.into(),
            packet: value.packet,
        }
    }
}

impl From<bridge::DhtLiveNodes> for BtEvent {
    fn from(value: bridge::DhtLiveNodes) -> Self {
        Self::DhtLiveNodes {
            local_node_id: value.local_node_id.into(),
            nodes: value.nodes.into_iter().map(|n| n.into()).collect(),
        }
    }
}
