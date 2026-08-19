//! Private torrent CXX wire models and their domain conversions.

use crate::{BtEvent, InfoHash};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// Payload for a metadata-received alert.
    pub(super) struct MetadataReceivedPayload {
        info_hash: [u8; 20],
        data: Vec<u8>,
    }

    /// Payload containing a torrent info hash and message.
    pub(super) struct InfoMessagePayload {
        info_hash: [u8; 20],
        message: String,
    }

    /// Payload for a failed torrent-add operation.
    pub(super) struct AddTorrentErrorPayload {
        info_hash: [u8; 20],
        message: String,
        error_value: i32,
        error_category: String,
    }
}

pub(super) use bridge::{AddTorrentErrorPayload, InfoMessagePayload, MetadataReceivedPayload};

impl From<bridge::InfoMessagePayload> for (InfoHash, String) {
    fn from(value: bridge::InfoMessagePayload) -> Self {
        (value.info_hash.into(), value.message)
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
