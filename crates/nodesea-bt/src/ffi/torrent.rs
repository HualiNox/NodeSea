//! Private torrent CXX wire models and their domain conversions.

use crate::BtEvent;

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// Payload for a metadata-received alert.
    pub(super) struct MetadataReceivedPayload {
        /// Torrent info hash associated with the metadata.
        info_hash: [u8; 20],
        /// Bencoded torrent metadata bytes.
        data: Vec<u8>,
    }

    /// Payload for a failed metadata request.
    pub(super) struct MetadataFailedPayload {
        /// Torrent info hash associated with the alert.
        info_hash: [u8; 20],
        /// Human-readable alert message.
        message: String,
    }

    /// Payload for a successful torrent-add operation.
    pub(super) struct AddTorrentPayload {
        /// Torrent info hash supplied to the add operation.
        info_hash: [u8; 20],
        /// Status message reported by libtorrent.
        message: String,
    }

    /// Payload for a failed torrent-add operation.
    pub(super) struct AddTorrentErrorPayload {
        /// Torrent info hash supplied to the add operation.
        info_hash: [u8; 20],
        /// Human-readable failure description.
        message: String,
        /// Numeric libtorrent error value.
        error_value: i32,
        /// Name of the libtorrent error category.
        error_category: String,
    }

    /// Payload for a torrent error alert.
    pub(super) struct TorrentErrorPayload {
        /// Torrent info hash associated with the alert.
        info_hash: [u8; 20],
        /// Error description reported by libtorrent.
        message: String,
    }

    /// Payload for a file error alert.
    pub(super) struct FileErrorPayload {
        /// Torrent info hash associated with the alert.
        info_hash: [u8; 20],
        /// Error description reported by libtorrent.
        message: String,
    }

    /// Payload for a failed torrent-delete operation.
    pub(super) struct TorrentDeleteFailedPayload {
        /// Torrent info hash associated with the alert.
        info_hash: [u8; 20],
        /// Failure description reported by libtorrent.
        message: String,
    }
}

pub(super) use bridge::{
    AddTorrentErrorPayload, AddTorrentPayload, FileErrorPayload, MetadataFailedPayload,
    MetadataReceivedPayload, TorrentDeleteFailedPayload, TorrentErrorPayload,
};

impl From<bridge::MetadataFailedPayload> for BtEvent {
    fn from(value: bridge::MetadataFailedPayload) -> Self {
        Self::MetadataFailed {
            info_hash: value.info_hash.into(),
            message: value.message,
        }
    }
}

impl From<bridge::MetadataReceivedPayload> for BtEvent {
    fn from(value: bridge::MetadataReceivedPayload) -> Self {
        Self::MetadataReceived {
            info_hash: value.info_hash.into(),
            data: value.data,
        }
    }
}

impl From<bridge::AddTorrentErrorPayload> for BtEvent {
    fn from(value: bridge::AddTorrentErrorPayload) -> Self {
        Self::AddTorrentError {
            info_hash: value.info_hash.into(),
            message: value.message,
            error_value: value.error_value,
            error_category: value.error_category,
        }
    }
}

impl From<bridge::AddTorrentPayload> for BtEvent {
    fn from(value: bridge::AddTorrentPayload) -> Self {
        Self::AddTorrent {
            info_hash: value.info_hash.into(),
            message: value.message,
        }
    }
}

impl From<bridge::TorrentErrorPayload> for BtEvent {
    fn from(value: bridge::TorrentErrorPayload) -> Self {
        Self::TorrentError {
            info_hash: value.info_hash.into(),
            message: value.message,
        }
    }
}

impl From<bridge::FileErrorPayload> for BtEvent {
    fn from(value: bridge::FileErrorPayload) -> Self {
        Self::FileError {
            info_hash: value.info_hash.into(),
            message: value.message,
        }
    }
}

impl From<bridge::TorrentDeleteFailedPayload> for BtEvent {
    fn from(value: bridge::TorrentDeleteFailedPayload) -> Self {
        Self::TorrentDeleteFailed {
            info_hash: value.info_hash.into(),
            message: value.message,
        }
    }
}
