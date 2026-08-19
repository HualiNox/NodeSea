//! Private DHT CXX wire models and their domain conversions.

use std::{net::SocketAddr, time::Duration};

use crate::{BtEvent, DhtNode, InfoHash};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    enum DhtDirectionPayload {
        Incoming,
        Outgoing,
    }

    /// Payload for a DHT announce alert.
    pub(super) struct DhtAnnouncePayload {
        info_hash: [u8; 20],
        peer_ip: String,
        peer_port: u16,
    }

    /// Payload for DHT statistics.
    pub(super) struct DhtStatsPayload {
        node_count: u32,
        local_ip: String,
        local_port: u16,
    }

    /// Payload for a DHT get-peers alert.
    pub(super) struct DhtGetPeersPayload {
        info_hash: [u8; 20],
    }

    pub(super) struct UdpEndpoint {
        address: String,
        port: u16,
    }

    pub(super) struct DhtNodePayload {
        node_id: [u8; 20],
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
        node: DhtNodePayload,
        interval_secs: i64,
        num_infohashes: i32,
        samples: Vec<SampleInfoHash>,
        nodes: Vec<DhtNodePayload>,
    }

    /// Payload for a DHT packet alert.
    ///
    /// This alert contains a raw DHT packet for diagnostics.
    pub(super) struct DhtPktPayload {
        direction: DhtDirectionPayload,
        endpoint: UdpEndpoint,
        packet: Vec<u8>,
    }
}

// These are narrow, named entries for the canonical callback bridge. The
// bridge module itself remains private and no wildcard re-export is used.
pub(super) use bridge::{
    DhtAnnouncePayload, DhtGetPeersPayload, DhtPktPayload, DhtSampleInfohashesPayload,
    DhtStatsPayload, UdpEndpoint,
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
