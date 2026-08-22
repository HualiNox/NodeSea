//! Private DHT CXX wire models and their domain conversions.

use std::{net::SocketAddr, time::Duration};

use crate::{
    BtEvent, BtEventKind, DhtAnnounce, DhtGetPeers, DhtInfoHash, DhtLiveNodes, DhtLog, DhtNode,
    DhtPkt, DhtSampleInfohashes, DhtStats, NodeId,
};

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
    pub(super) struct UdpEndpointPayload {
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
        endpoint: UdpEndpointPayload,
    }

    /// Private fixed-size wire adapter for a sampled infohash.
    ///
    /// CXX cannot represent `Vec<[u8; 20]>` as a shared field. This wrapper
    /// exists only for the FFI payload and is converted to [`DhtInfoHash`] before
    /// reaching the domain event.
    pub(super) struct SampleInfoHashPayload {
        /// Raw 20-byte sampled infohash.
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
        num_infohashes: i64,
        /// Sampled infohashes represented through the CXX fixed-array adapter.
        samples: Vec<SampleInfoHashPayload>,
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
        endpoint: UdpEndpointPayload,
        /// Verbatim packet bytes.
        packet: Vec<u8>,
    }

    /// Wire payload containing one local DHT routing table snapshot.
    pub(super) struct DhtLiveNodesPayload {
        /// Local DHT node ID owning the routing table.
        local_node_id: [u8; 20],
        /// Nodes currently present in the routing table.
        nodes: Vec<DhtNodePayload>,
    }

    /// Payload for a DHT log alert.
    pub(super) struct DhtLogPayload {
        /// Log message reported by libtorrent.
        message: String,
    }
}

// These are narrow, named entries for the canonical callback bridge. The
// bridge module itself remains private and no wildcard re-export is used.
pub(super) use bridge::{
    DhtAnnouncePayload, DhtGetPeersPayload, DhtLiveNodesPayload, DhtLogPayload, DhtPktPayload,
    DhtSampleInfohashesPayload, DhtStatsPayload, UdpEndpointPayload,
};

impl From<bridge::DhtAnnouncePayload> for BtEvent {
    fn from(value: bridge::DhtAnnouncePayload) -> Self {
        Self::new(BtEventKind::DhtAnnounce(DhtAnnounce::from_ffi(
            DhtInfoHash::from_bytes(value.info_hash),
            value.peer_ip,
            value.peer_port,
        )))
    }
}

impl From<bridge::DhtStatsPayload> for BtEvent {
    fn from(value: bridge::DhtStatsPayload) -> Self {
        Self::new(BtEventKind::DhtStats(DhtStats::from_ffi(
            value.node_count,
            value.local_ip,
            value.local_port,
        )))
    }
}

impl From<bridge::DhtGetPeersPayload> for BtEvent {
    fn from(value: bridge::DhtGetPeersPayload) -> Self {
        Self::new(BtEventKind::DhtGetPeers(DhtGetPeers::from_ffi(
            DhtInfoHash::from_bytes(value.info_hash),
        )))
    }
}

impl bridge::UdpEndpointPayload {
    pub(super) fn from_socket_addr(addr: &SocketAddr) -> Self {
        Self {
            address: addr.ip().to_string(),
            port: addr.port(),
        }
    }
}

impl bridge::UdpEndpointPayload {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::new(self.address.parse().unwrap(), self.port)
    }
}

impl bridge::SampleInfoHashPayload {
    fn into_dht_info_hash(self) -> DhtInfoHash {
        DhtInfoHash::from_bytes(self.bytes)
    }
}

impl bridge::DhtNodePayload {
    fn into_dht_node(self) -> DhtNode {
        DhtNode::from_ffi(
            NodeId::from_bytes(self.node_id),
            self.endpoint.into_socket_addr(),
        )
    }
}

impl From<bridge::DhtSampleInfohashesPayload> for BtEvent {
    fn from(value: bridge::DhtSampleInfohashesPayload) -> Self {
        Self::new(BtEventKind::DhtSampleInfohashes(
            DhtSampleInfohashes::from_ffi(
                value.node.into_dht_node(),
                Duration::from_secs(value.interval_secs as u64),
                value.num_infohashes,
                value
                    .samples
                    .into_iter()
                    .map(bridge::SampleInfoHashPayload::into_dht_info_hash)
                    .collect(),
                value
                    .nodes
                    .into_iter()
                    .map(bridge::DhtNodePayload::into_dht_node)
                    .collect(),
            ),
        ))
    }
}

impl From<bridge::DhtPktPayload> for BtEvent {
    fn from(value: bridge::DhtPktPayload) -> Self {
        Self::new(BtEventKind::DhtPkt(DhtPkt::from_ffi(
            match value.direction {
                bridge::DhtDirectionPayload::Incoming => crate::DhtDirection::Incoming,
                bridge::DhtDirectionPayload::Outgoing => crate::DhtDirection::Outgoing,
                // unreachable
                _ => unreachable!("invalid DHT direction from C++"),
            },
            value.endpoint.into_socket_addr(),
            value.packet,
        )))
    }
}

impl From<bridge::DhtLiveNodesPayload> for BtEvent {
    fn from(value: bridge::DhtLiveNodesPayload) -> Self {
        Self::new(BtEventKind::DhtLiveNodes(DhtLiveNodes::from_ffi(
            NodeId::from_bytes(value.local_node_id),
            value
                .nodes
                .into_iter()
                .map(bridge::DhtNodePayload::into_dht_node)
                .collect(),
        )))
    }
}

impl From<bridge::DhtLogPayload> for BtEvent {
    fn from(value: bridge::DhtLogPayload) -> Self {
        Self::new(BtEventKind::DhtLog(DhtLog::from_ffi(value.message)))
    }
}
