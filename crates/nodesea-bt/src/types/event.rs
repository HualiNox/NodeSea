//! Public domain events emitted by the BitTorrent engine.

use std::{
    net::SocketAddr,
    time::{Duration, SystemTime},
};

use crate::{DhtInfoHash, DhtNode, NodeId, TorrentId};

/// Direction of a DHT packet (incoming or outgoing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DhtDirection {
    /// Incoming packet received from a DHT node.
    Incoming,
    /// Outgoing packet sent to a DHT node.
    Outgoing,
}

/// A BitTorrent event with its reception timestamp and event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtEvent {
    timestamp: SystemTime,
    kind: BtEventKind,
}

impl BtEvent {
    /// Returns the time at which the event was created by the Rust bridge.
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    /// Returns the event payload without consuming the event.
    pub fn kind(&self) -> &BtEventKind {
        &self.kind
    }

    pub(crate) fn new(kind: BtEventKind) -> Self {
        Self {
            timestamp: SystemTime::now(),
            kind,
        }
    }
}

macro_rules! event_payload {
    ($doc:literal, $name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            $(
                $field: $ty,
            )*
        }

        impl $name {
            pub(crate) fn from_ffi($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            $(
                #[doc = concat!("Returns the `", stringify!($field), "` field.")]
                pub fn $field(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    };
    ($doc:literal, $name:ident) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name;

        impl $name {
            pub(crate) fn from_ffi() -> Self {
                Self
            }
        }
    };
}

event_payload!(
    "A DHT announce event payload.",
    DhtAnnounce {
        info_hash: DhtInfoHash,
        peer_ip: String,
        peer_port: u16,
    }
);
event_payload!("Metadata received event payload.", MetadataReceived {
    torrent_id: TorrentId,
    data: Vec<u8>,
});
event_payload!(
    "Metadata failed event payload.",
    MetadataFailed {
        torrent_id: TorrentId,
        message: String,
    }
);
event_payload!(
    "DHT statistics event payload.",
    DhtStats {
        node_count: u32,
        local_ip: String,
        local_port: u16,
    }
);
event_payload!("DHT bootstrap event payload.", DhtBootstrap);
event_payload!(
    "DHT get-peers event payload.",
    DhtGetPeers {
        info_hash: DhtInfoHash,
    }
);
event_payload!(
    "Error details reported when adding a torrent fails.",
    AddTorrentError {
        error_value: i32,
        error_category: String,
    }
);
event_payload!(
    "Torrent add result. The `error` field contains failure details when the
    operation fails.",
    AddTorrent {
        torrent_id: TorrentId,
        message: String,
        error: Option<AddTorrentError>,
    }
);
event_payload!(
    "Torrent error event payload.",
    TorrentError {
        torrent_id: TorrentId,
        message: String,
    }
);
event_payload!(
    "File error event payload.",
    FileError {
        torrent_id: TorrentId,
        message: String,
    }
);
event_payload!(
    "Torrent deletion failure event payload.",
    TorrentDeleteFailed {
        torrent_id: TorrentId,
        message: String,
    }
);
event_payload!(
    "Session error event payload.",
    SessionError { message: String }
);
event_payload!(
    "Listen failure event payload.",
    ListenFailed { message: String }
);
event_payload!("UDP error event payload.", UdpError { message: String });
event_payload!("DHT error event payload.", DhtError { message: String });
event_payload!(
    "Dropped alerts event payload.",
    AlertsDropped { message: String }
);
event_payload!("DHT sample infohashes event payload.", DhtSampleInfohashes {
    node: DhtNode,
    interval: Duration,
    num_infohashes: u32,
    samples: Vec<DhtInfoHash>,
    nodes: Vec<DhtNode>,
});
event_payload!("Raw DHT packet event payload.", DhtPkt {
    direction: DhtDirection,
    endpoint: SocketAddr,
    packet: Vec<u8>,
});
event_payload!("DHT live nodes event payload.", DhtLiveNodes {
    local_node_id: NodeId,
    nodes: Vec<DhtNode>,
});
/// A BitTorrent event payload produced by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtEventKind {
    /// A DHT announce event.
    DhtAnnounce(DhtAnnounce),
    /// Metadata was successfully received.
    MetadataReceived(MetadataReceived),
    /// Metadata failed to be received.
    MetadataFailed(MetadataFailed),
    /// DHT statistics were received.
    DhtStats(DhtStats),
    /// DHT bootstrap completed.
    DhtBootstrap(DhtBootstrap),
    /// A DHT get-peers event was received.
    DhtGetPeers(DhtGetPeers),
    /// A torrent-add operation completed.
    AddTorrent(AddTorrent),
    /// A torrent entered an error state.
    TorrentError(TorrentError),
    /// A file operation failed for a torrent.
    FileError(FileError),
    /// Deleting a torrent failed.
    TorrentDeleteFailed(TorrentDeleteFailed),
    /// The session reported an error.
    SessionError(SessionError),
    /// Listening on the configured port failed.
    ListenFailed(ListenFailed),
    /// A UDP operation failed.
    UdpError(UdpError),
    /// A DHT operation failed.
    DhtError(DhtError),
    /// The session dropped alerts before they were polled.
    AlertsDropped(AlertsDropped),
    /// Sample infohashes were received from a DHT node.
    DhtSampleInfohashes(DhtSampleInfohashes),
    /// A raw DHT packet was received or sent.
    DhtPkt(DhtPkt),
    /// Live nodes were reported for one local DHT routing table.
    DhtLiveNodes(DhtLiveNodes),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InfoHashV1;

    #[test]
    fn test_bt_event_variants() {
        let dht_info_hash = DhtInfoHash::from_bytes([0x42; 20]);
        let torrent_id = TorrentId::new(Some(InfoHashV1::from_bytes([0x42; 20])), None);

        let kinds = vec![
            BtEventKind::DhtAnnounce(DhtAnnounce::from_ffi(
                dht_info_hash,
                "10.0.0.1".to_string(),
                8080,
            )),
            BtEventKind::MetadataReceived(MetadataReceived::from_ffi(torrent_id, vec![1, 2, 3])),
            BtEventKind::MetadataFailed(MetadataFailed::from_ffi(
                torrent_id,
                "fetch failed".to_string(),
            )),
            BtEventKind::DhtStats(DhtStats::from_ffi(128, "127.0.0.1".to_string(), 6881)),
            BtEventKind::DhtBootstrap(DhtBootstrap::from_ffi()),
            BtEventKind::DhtGetPeers(DhtGetPeers::from_ffi(dht_info_hash)),
            BtEventKind::AddTorrent(AddTorrent::from_ffi(
                torrent_id,
                "torrent added".to_string(),
                None,
            )),
            BtEventKind::AddTorrent(AddTorrent::from_ffi(
                torrent_id,
                "add failed".to_string(),
                Some(AddTorrentError::from_ffi(1, "libtorrent".to_string())),
            )),
            BtEventKind::TorrentError(TorrentError::from_ffi(
                torrent_id,
                "torrent error".to_string(),
            )),
            BtEventKind::FileError(FileError::from_ffi(torrent_id, "file error".to_string())),
            BtEventKind::TorrentDeleteFailed(TorrentDeleteFailed::from_ffi(
                torrent_id,
                "delete failed".to_string(),
            )),
            BtEventKind::SessionError(SessionError::from_ffi("session error".to_string())),
            BtEventKind::ListenFailed(ListenFailed::from_ffi("listen error".to_string())),
            BtEventKind::UdpError(UdpError::from_ffi("udp error".to_string())),
            BtEventKind::DhtError(DhtError::from_ffi("dht error".to_string())),
            BtEventKind::AlertsDropped(AlertsDropped::from_ffi("dropped alerts".to_string())),
        ];

        for kind in kinds {
            let event = BtEvent::new(kind);
            let cloned = event.clone();
            assert_eq!(event, cloned);
            assert!(!format!("{event:?}").is_empty());
        }
    }
}
