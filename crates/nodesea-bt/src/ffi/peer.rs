//! Private peer alert payloads and their domain conversions.

use crate::{
    BlockFinished, BtEvent, BtEventKind, PeerConnect, PeerDisconnected, PeerError, PeerLog,
    PieceFinished,
};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    unsafe extern "C++" {
        include!("src/ffi/torrent.rs.h");

        type TorrentIdPayload = crate::ffi::torrent::TorrentIdPayload;
    }

    /// Payload for a peer connection alert.
    pub(super) struct PeerConnectPayload {
        /// Combined torrent identity associated with the peer.
        torrent_id: TorrentIdPayload,
        /// Alert message reported by libtorrent.
        message: String,
    }

    /// Payload for a peer disconnection alert.
    pub(super) struct PeerDisconnectedPayload {
        /// Combined torrent identity associated with the peer.
        torrent_id: TorrentIdPayload,
        /// Alert message reported by libtorrent.
        message: String,
    }

    /// Payload for a peer error alert.
    pub(super) struct PeerErrorPayload {
        /// Combined torrent identity associated with the peer.
        torrent_id: TorrentIdPayload,
        /// Alert message reported by libtorrent.
        message: String,
    }

    /// Payload for a peer log alert.
    pub(super) struct PeerLogPayload {
        /// Combined torrent identity associated with the peer.
        torrent_id: TorrentIdPayload,
        /// Log message reported by libtorrent.
        message: String,
    }

    /// Payload for a completed piece.
    pub(super) struct PieceFinishedPayload {
        /// Combined torrent identity associated with the piece.
        torrent_id: TorrentIdPayload,
        /// Piece index.
        piece_index: i32,
        /// Alert message reported by libtorrent.
        message: String,
    }

    /// Payload for a completed block.
    pub(super) struct BlockFinishedPayload {
        /// Combined torrent identity associated with the block.
        torrent_id: TorrentIdPayload,
        /// Piece index containing the block.
        piece_index: i32,
        /// Block index within the piece.
        block_index: i32,
        /// Alert message reported by libtorrent.
        message: String,
    }
}

pub(super) use bridge::{
    BlockFinishedPayload, PeerConnectPayload, PeerDisconnectedPayload, PeerErrorPayload,
    PeerLogPayload, PieceFinishedPayload,
};

impl From<bridge::PeerConnectPayload> for BtEvent {
    fn from(value: bridge::PeerConnectPayload) -> Self {
        Self::new(BtEventKind::PeerConnect(PeerConnect::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::PeerDisconnectedPayload> for BtEvent {
    fn from(value: bridge::PeerDisconnectedPayload) -> Self {
        Self::new(BtEventKind::PeerDisconnected(PeerDisconnected::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::PeerErrorPayload> for BtEvent {
    fn from(value: bridge::PeerErrorPayload) -> Self {
        Self::new(BtEventKind::PeerError(PeerError::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::PeerLogPayload> for BtEvent {
    fn from(value: bridge::PeerLogPayload) -> Self {
        Self::new(BtEventKind::PeerLog(PeerLog::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.message,
        )))
    }
}

impl From<bridge::PieceFinishedPayload> for BtEvent {
    fn from(value: bridge::PieceFinishedPayload) -> Self {
        Self::new(BtEventKind::PieceFinished(PieceFinished::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.piece_index,
            value.message,
        )))
    }
}

impl From<bridge::BlockFinishedPayload> for BtEvent {
    fn from(value: bridge::BlockFinishedPayload) -> Self {
        Self::new(BtEventKind::BlockFinished(BlockFinished::from_ffi(
            value.torrent_id.into_torrent_id(),
            value.piece_index,
            value.block_index,
            value.message,
        )))
    }
}
