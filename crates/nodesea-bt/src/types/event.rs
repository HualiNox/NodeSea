use super::identity::InfoHash;

/// A BitTorrent event produced by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtEvent {
    /// A DHT announce event was received.
    DhtAnnounce {
        /// Info hash associated with the announce request.
        info_hash: InfoHash,
        /// Address of the announcing peer.
        peer_ip: String,
        /// Port of the announcing peer.
        peer_port: u16,
    },
    /// Metadata was successfully received.
    MetadataReceived {
        /// Info hash of the torrent whose metadata was received.
        info_hash: InfoHash,
        /// Bencoded torrent info section.
        data: Vec<u8>,
    },
    /// Metadata failed to be received.
    MetadataFailed {
        /// Info hash of the torrent whose metadata request failed.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
    },
    /// The DHT stats event containing the number of nodes.
    DhtStats {
        /// Number of nodes currently present in the DHT routing table.
        node_count: u32,
        /// Local IP address used for DHT operations.
        local_ip: String,
        /// Local port used for DHT operations.
        local_port: u16,
    },
    /// The DHT bootstrap process has completed.
    DhtBootstrap,
    /// A DHT get peers event was received.
    DhtGetPeers {
        /// Info hash associated with the get peers request.
        info_hash: InfoHash,
    },
    /// A torrent was added successfully.
    AddTorrent {
        /// Info hash of the added torrent.
        info_hash: InfoHash,
        /// Status message reported by libtorrent.
        message: String,
    },
    /// Adding a torrent failed.
    AddTorrentError {
        /// Info hash supplied to the add operation.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
        /// Numeric value of the libtorrent error code.
        error_value: i32,
        /// Error category name returned by libtorrent.
        error_category: String,
    },
    /// A torrent entered an error state.
    TorrentError {
        /// Info hash of the torrent in error.
        info_hash: InfoHash,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A file operation failed for a torrent.
    FileError {
        /// Info hash of the affected torrent.
        info_hash: InfoHash,
        /// Error description reported by libtorrent.
        message: String,
    },
    /// Deleting a torrent failed.
    TorrentDeleteFailed {
        /// Info hash of the torrent that could not be deleted.
        info_hash: InfoHash,
        /// Failure description reported by libtorrent.
        message: String,
    },
    /// The session reported an error.
    SessionError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// Listening on the configured port failed.
    ListenFailed {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A UDP operation failed.
    UdpError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// A DHT operation failed.
    DhtError {
        /// Error description reported by libtorrent.
        message: String,
    },
    /// The session dropped alerts before they were polled.
    AlertsDropped {
        /// Description of the dropped-alert condition.
        message: String,
    },
}
