//! Private torrent CXX wire models and their domain conversions.

use crate::{
    AddTorrent, AddTorrentError, BtEvent, BtEventKind, FileError, InfoHashV1, InfoHashV2,
    MetadataFailed, MetadataReceived, TorrentDeleteFailed, TorrentError, TorrentId,
};

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

    /// Payload for a torrent-add operation.
    pub(super) struct AddTorrentPayload {
        /// Combined torrent identity supplied to the add operation.
        torrent_id: TorrentIdPayload,
        /// Status message reported by libtorrent.
        message: String,
        /// Whether the add operation failed.
        has_error: bool,
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
    AddTorrentPayload, FileErrorPayload, MetadataFailedPayload, MetadataReceivedPayload,
    TorrentDeleteFailedPayload, TorrentErrorPayload, TorrentIdPayload,
};

impl bridge::TorrentIdPayload {
    pub(super) fn from_torrent_id(value: &TorrentId) -> Self {
        Self {
            v1: value.v1().map_or([0; 20], |hash| *hash.as_bytes()),
            v2: value.v2().map_or([0; 32], |hash| *hash.as_bytes()),
            has_v1: value.has_v1(),
            has_v2: value.has_v2(),
        }
    }

    fn into_torrent_id(self) -> TorrentId {
        TorrentId::new(
            self.has_v1.then(|| InfoHashV1::from_bytes(self.v1)),
            self.has_v2.then(|| InfoHashV2::from_bytes(self.v2)),
        )
    }
}

impl From<bridge::MetadataFailedPayload> for BtEvent {
    fn from(value: bridge::MetadataFailedPayload) -> Self {
        Self::new(BtEventKind::MetadataFailed(MetadataFailed::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::MetadataReceivedPayload> for BtEvent {
    fn from(value: bridge::MetadataReceivedPayload) -> Self {
        Self::new(BtEventKind::MetadataReceived(MetadataReceived::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.data,
        )))
    }
}

impl From<bridge::AddTorrentPayload> for BtEvent {
    fn from(value: bridge::AddTorrentPayload) -> Self {
        Self::new(BtEventKind::AddTorrent(AddTorrent::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
            value
                .has_error
                .then(|| AddTorrentError::from_ffi(value.error_value, value.error_category)),
        )))
    }
}

impl From<bridge::TorrentErrorPayload> for BtEvent {
    fn from(value: bridge::TorrentErrorPayload) -> Self {
        Self::new(BtEventKind::TorrentError(TorrentError::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::FileErrorPayload> for BtEvent {
    fn from(value: bridge::FileErrorPayload) -> Self {
        Self::new(BtEventKind::FileError(FileError::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::TorrentDeleteFailedPayload> for BtEvent {
    fn from(value: bridge::TorrentDeleteFailedPayload) -> Self {
        Self::new(BtEventKind::TorrentDeleteFailed(
            TorrentDeleteFailed::from_ffi(value.torrent_id.into_torrent_id(), value.message),
        ))
    }
}
