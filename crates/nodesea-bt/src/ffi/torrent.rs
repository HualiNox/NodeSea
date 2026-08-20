//! Private torrent CXX wire models and their domain conversions.

use crate::{BtEvent, InfoHashV1, InfoHashV2, TorrentId};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// Payload for a metadata-received alert.
    pub(super) struct MetadataReceivedPayload {
        /// Combined torrent identity associated with the metadata.
        torrent_id: TorrentIdPayload,
        /// Bencoded torrent metadata bytes.
        data: Vec<u8>,
    }

    /// Payload for a failed metadata request.
    pub(super) struct MetadataFailedPayload {
        /// Combined torrent identity associated with the alert.
        torrent_id: TorrentIdPayload,
        /// Human-readable alert message.
        message: String,
    }

    /// Payload for a successful torrent-add operation.
    pub(super) struct AddTorrentPayload {
        /// Combined torrent identity supplied to the add operation.
        torrent_id: TorrentIdPayload,
        /// Status message reported by libtorrent.
        message: String,
    }

    /// Payload for a failed torrent-add operation.
    pub(super) struct AddTorrentErrorPayload {
        /// Combined torrent identity supplied to the add operation.
        torrent_id: TorrentIdPayload,
        /// Human-readable failure description.
        message: String,
        /// Numeric libtorrent error value.
        error_value: i32,
        /// Name of the libtorrent error category.
        error_category: String,
    }

    /// Payload for a torrent error alert.
    pub(super) struct TorrentErrorPayload {
        /// Combined torrent identity associated with the alert.
        torrent_id: TorrentIdPayload,
        /// Error description reported by libtorrent.
        message: String,
    }

    /// Payload for a file error alert.
    pub(super) struct FileErrorPayload {
        /// Combined torrent identity associated with the alert.
        torrent_id: TorrentIdPayload,
        /// Error description reported by libtorrent.
        message: String,
    }

    /// Payload for a failed torrent-delete operation.
    pub(super) struct TorrentDeleteFailedPayload {
        /// Combined torrent identity associated with the alert.
        torrent_id: TorrentIdPayload,
        /// Failure description reported by libtorrent.
        message: String,
    }

    /// Wire representation of a v1, v2, or hybrid torrent identity.
    pub(super) struct TorrentIdPayload {
        /// SHA-1 v1 infohash bytes, or zeroes when no v1 hash is present.
        v1: [u8; 20],
        /// SHA-256 v2 infohash bytes, or zeroes when no v2 hash is present.
        v2: [u8; 32],
        /// Whether `v1` contains a valid v1 hash.
        has_v1: bool,
        /// Whether `v2` contains a valid v2 hash.
        has_v2: bool,
    }
}

pub(super) use bridge::{
    AddTorrentErrorPayload, AddTorrentPayload, FileErrorPayload, MetadataFailedPayload,
    MetadataReceivedPayload, TorrentDeleteFailedPayload, TorrentErrorPayload, TorrentIdPayload,
};

impl From<&TorrentId> for bridge::TorrentIdPayload {
    fn from(value: &TorrentId) -> Self {
        Self {
            v1: value.v1().map_or([0; 20], |hash| *hash.as_bytes()),
            v2: value.v2().map_or([0; 32], |hash| *hash.as_bytes()),
            has_v1: value.has_v1(),
            has_v2: value.has_v2(),
        }
    }
}

impl From<bridge::MetadataFailedPayload> for BtEvent {
    fn from(value: bridge::MetadataFailedPayload) -> Self {
        Self::MetadataFailed {
            torrent_id: value.torrent_id.into(),
            message: value.message,
        }
    }
}

impl From<bridge::MetadataReceivedPayload> for BtEvent {
    fn from(value: bridge::MetadataReceivedPayload) -> Self {
        Self::MetadataReceived {
            torrent_id: value.torrent_id.into(),
            data: value.data,
        }
    }
}

impl From<bridge::AddTorrentErrorPayload> for BtEvent {
    fn from(value: bridge::AddTorrentErrorPayload) -> Self {
        Self::AddTorrentError {
            torrent_id: value.torrent_id.into(),
            message: value.message,
            error_value: value.error_value,
            error_category: value.error_category,
        }
    }
}

impl From<bridge::AddTorrentPayload> for BtEvent {
    fn from(value: bridge::AddTorrentPayload) -> Self {
        Self::AddTorrent {
            torrent_id: value.torrent_id.into(),
            message: value.message,
        }
    }
}

impl From<bridge::TorrentErrorPayload> for BtEvent {
    fn from(value: bridge::TorrentErrorPayload) -> Self {
        Self::TorrentError {
            torrent_id: value.torrent_id.into(),
            message: value.message,
        }
    }
}

impl From<bridge::FileErrorPayload> for BtEvent {
    fn from(value: bridge::FileErrorPayload) -> Self {
        Self::FileError {
            torrent_id: value.torrent_id.into(),
            message: value.message,
        }
    }
}

impl From<bridge::TorrentDeleteFailedPayload> for BtEvent {
    fn from(value: bridge::TorrentDeleteFailedPayload) -> Self {
        Self::TorrentDeleteFailed {
            torrent_id: value.torrent_id.into(),
            message: value.message,
        }
    }
}

impl From<bridge::TorrentIdPayload> for TorrentId {
    fn from(value: bridge::TorrentIdPayload) -> Self {
        Self::new(
            if value.has_v1 {
                Some(InfoHashV1::from_bytes(value.v1))
            } else {
                None
            },
            if value.has_v2 {
                Some(InfoHashV2::from_bytes(value.v2))
            } else {
                None
            },
        )
    }
}
