//! Private DHT CXX wire models and their domain conversions.

use crate::{BtEvent, InfoHash};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// DHT-specific wire representation of an info hash.
    pub(super) struct DhtInfoHash {
        bytes: [u8; 20],
    }

    /// Payload for a DHT announce alert.
    pub(super) struct DhtAnnouncePayload {
        info_hash: DhtInfoHash,
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
        info_hash: DhtInfoHash,
    }
}

// These are narrow, named entries for the canonical callback bridge. The
// bridge module itself remains private and no wildcard re-export is used.
pub(super) use bridge::{DhtAnnouncePayload, DhtGetPeersPayload, DhtStatsPayload};

impl From<bridge::DhtInfoHash> for InfoHash {
    fn from(value: bridge::DhtInfoHash) -> Self {
        value.bytes.into()
    }
}

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
