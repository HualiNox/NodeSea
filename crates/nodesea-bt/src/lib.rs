//! Rust bindings and event model for the Nodesea BitTorrent engine.
#![warn(missing_docs)]

use std::collections::VecDeque;
use std::fmt;
use std::str::FromStr;

mod ffi;

/// Represents a Bittorrent event.
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
        /// Failure description reported by libtorrent.
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

/// Receives events produced by the BitTorrent engine.
pub trait EventSink {
    /// Handles one event synchronously.
    ///
    /// The event is owned by the sink after this call returns. Implementations
    /// should not retain references into the FFI payload because the payload
    /// is dropped when the callback returns.
    fn on_event(&mut self, event: BtEvent);
}

/// A simple event sink that stores received events in a vector.
#[derive(Debug, Default)]
pub struct EventCollector {
    events: Vec<BtEvent>,
}

impl EventSink for EventCollector {
    fn on_event(&mut self, event: BtEvent) {
        self.events.push(event);
    }
}

impl EventCollector {
    /// Creates a new, empty event collector.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Creates a new event collector with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Returns a slice of the collected events.
    pub fn events(&self) -> &[BtEvent] {
        &self.events
    }

    /// Takes the collected events out of the collector, leaving it empty.
    pub fn take_events(&mut self) -> Vec<BtEvent> {
        std::mem::take(&mut self.events)
    }

    /// Clears the collector.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Represents a 20-byte info hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct InfoHash([u8; 20]);

impl InfoHash {
    /// Creates an info hash from a 20-byte array.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The 20-byte array to create the info hash from.
    ///
    /// # Returns
    ///
    /// The created info hash.
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Returns the info hash as a byte slice.
    ///
    /// # Returns
    ///
    /// A reference to the 20-byte array representing the info hash.
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Returns the info hash as a hex string.
    ///
    /// # Returns
    ///
    /// The hex-encoded representation of the info hash.
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Creates an info hash from a hex string.
    ///
    /// # Arguments
    ///
    /// * `s` - The hex string to parse into an info hash.
    ///
    /// # Returns
    ///
    /// `Ok(InfoHash)` if the hex string was successfully decoded, or `Err` if decoding failed.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl FromStr for InfoHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl TryFrom<&str> for InfoHash {
    type Error = hex::FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_hex(value)
    }
}

impl TryFrom<&[u8]> for InfoHash {
    type Error = std::array::TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; 20] = slice.try_into()?;
        Ok(Self(bytes))
    }
}

impl fmt::Debug for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InfoHash({})", self.to_hex())
    }
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 20]> for InfoHash {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 20]> for InfoHash {
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

/// A BitTorrent engine backed by a libtorrent session.
pub struct Engine {
    inner: cxx::UniquePtr<ffi::Engine>,
    buffer: VecDeque<BtEvent>,
}

impl Engine {
    /// Creates a new BitTorrent engine instance.
    ///
    /// # Returns
    ///
    /// `None` if the underlying C++ engine could not be created.
    pub fn new() -> Option<Self> {
        let inner = ffi::new_engine();
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                buffer: VecDeque::new(),
            })
        }
    }

    /// Polls for a batch of events and sends them to the supplied sink.
    ///
    /// # Arguments
    ///
    /// * `sink` - Receives events as they are dispatched.
    ///
    /// # Returns
    ///
    /// The number of events delivered, including events buffered by a previous
    /// call to [`Engine::poll_event`].
    pub fn poll_events<S: EventSink>(&mut self, sink: &mut S) -> usize {
        let mut dispatched = 0;
        while let Some(event) = self.buffer.pop_front() {
            sink.on_event(event);
            dispatched += 1;
        }

        let mut ffi_sink = ffi::FfiEventSink::new(sink);
        dispatched + ffi::poll_events(self.inner.pin_mut(), &mut ffi_sink)
    }

    /// Polls for the next single event from the BitTorrent engine.
    ///
    /// # Returns
    ///
    /// An event if one is available, or `None` if no events are pending.
    ///
    /// This method polls all currently available native alerts once and keeps
    /// the remaining events in an internal queue for subsequent calls.
    pub fn poll_event(&mut self) -> Option<BtEvent> {
        if let Some(event) = self.buffer.pop_front() {
            return Some(event);
        }

        {
            let mut queue_sink = QueueSink {
                buffer: &mut self.buffer,
            };
            let mut ffi_sink = ffi::FfiEventSink::new(&mut queue_sink);
            ffi::poll_events(self.inner.pin_mut(), &mut ffi_sink);
        }

        self.buffer.pop_front()
    }

    /// Fetches metadata for a torrent using its info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The 20-byte info hash identifying the torrent.
    ///
    /// # Returns
    ///
    /// `true` if the fetch request was successfully initiated, `false` otherwise.
    pub fn fetch_metadata(&mut self, info_hash: &InfoHash) -> bool {
        let pin = self.inner.pin_mut();
        ffi::fetch_metadata(pin, info_hash.as_bytes())
    }

    /// Cancels a metadata fetch request for a torrent using its info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The 20-byte info hash identifying the torrent.
    ///
    /// # Returns
    ///
    /// `true` if the cancellation request was successfully initiated, `false` otherwise.
    pub fn cancel_fetch_metadata(&mut self, info_hash: &InfoHash) -> bool {
        let pin = self.inner.pin_mut();
        ffi::cancel_fetch(pin, info_hash.as_bytes())
    }

    /// Requests an asynchronous DHT statistics alert.
    ///
    /// # Returns
    ///
    /// Returns `true` if the request was posted successfully.
    ///
    /// The resulting node count is delivered later through [`BtEvent::DhtStats`].
    pub fn post_dht_stats(&self) -> bool {
        ffi::post_dht_stats(&self.inner)
    }
}

/// A queue-based event sink that buffers events in a deque.
struct QueueSink<'a> {
    buffer: &'a mut VecDeque<BtEvent>,
}

impl EventSink for QueueSink<'_> {
    fn on_event(&mut self, event: BtEvent) {
        self.buffer.push_back(event);
    }
}

// Engine encapsulates an internal libtorrent session which manages its own
// thread pool and IO context. Moving ownership across threads is safe.
unsafe impl Send for Engine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_hash_hex_roundtrip() {
        let hex_str = "0123456789abcdef0123456789abcdef01234567";
        let hash = InfoHash::from_hex(hex_str).expect("Valid 40-character hex string");
        assert_eq!(hash.to_hex(), hex_str);
        assert_eq!(format!("{hash}"), hex_str);
        assert_eq!(format!("{hash:?}"), format!("InfoHash({hex_str})"));
    }

    #[test]
    fn test_info_hash_traits() {
        let hex_str = "0123456789abcdef0123456789abcdef01234567";
        let hash: InfoHash = hex_str.parse().expect("Parse via FromStr");
        assert_eq!(hash.to_hex(), hex_str);

        let hash_try_str = InfoHash::try_from(hex_str).expect("TryFrom &str");
        assert_eq!(hash_try_str, hash);

        let raw = [7u8; 20];
        let hash_try_slice = InfoHash::try_from(&raw[..]).expect("TryFrom &[u8]");
        assert_eq!(hash_try_slice.as_bytes(), &raw);
    }

    #[test]
    fn test_info_hash_invalid_hex() {
        assert!(InfoHash::from_hex("invalid_hex_characters_here!!").is_err());
        assert!(InfoHash::from_hex("12345678").is_err()); // Too short
    }

    #[test]
    fn test_info_hash_from_bytes() {
        let raw_bytes = [42u8; 20];
        let hash = InfoHash::from_bytes(raw_bytes);
        assert_eq!(hash.as_bytes(), &raw_bytes);
        assert_eq!(hash, InfoHash::from(raw_bytes));
    }

    #[test]
    fn test_engine_lifecycle() {
        let mut engine = Engine::new().expect("Failed to initialize Engine");

        // Initial poll should return None on an empty queue
        assert_eq!(engine.poll_event(), None);

        // Request DHT statistics; the node count is delivered as a later event.
        assert!(engine.post_dht_stats());

        // Test metadata fetch lifecycle
        let dummy_hash = InfoHash::from_bytes([0xef; 20]);
        assert!(engine.fetch_metadata(&dummy_hash));

        // Duplicate fetch should return false (already registered)
        assert!(!engine.fetch_metadata(&dummy_hash));

        // Cancel fetch should succeed
        assert!(engine.cancel_fetch_metadata(&dummy_hash));

        // Canceling a non-existent fetch should return false
        assert!(!engine.cancel_fetch_metadata(&dummy_hash));
    }

    #[test]
    fn test_info_hash_default_and_order() {
        let default_hash = InfoHash::default();
        assert_eq!(default_hash.as_bytes(), &[0u8; 20]);
        assert_eq!(default_hash, InfoHash::from_bytes([0u8; 20]));

        let hash_a = InfoHash::from_bytes([1u8; 20]);
        let hash_b = InfoHash::from_bytes([2u8; 20]);
        assert!(hash_a < hash_b);
        assert_eq!(hash_a.as_ref(), &[1u8; 20]);

        // Test TryFrom slice error on invalid length
        let invalid_slice = [0u8; 19];
        assert!(InfoHash::try_from(&invalid_slice[..]).is_err());
    }

    #[test]
    fn test_event_collector_sink() {
        let mut collector = EventCollector::new();
        assert!(collector.events().is_empty());

        let info_hash = InfoHash::from_bytes([0xab; 20]);
        let get_peers_event = BtEvent::DhtGetPeers { info_hash };
        let announce_event = BtEvent::DhtAnnounce {
            info_hash,
            peer_ip: "127.0.0.1".to_string(),
            peer_port: 6881,
        };

        collector.on_event(BtEvent::DhtBootstrap);
        collector.on_event(get_peers_event.clone());
        collector.on_event(announce_event.clone());

        assert_eq!(collector.events().len(), 3);
        assert_eq!(
            collector.events(),
            &[
                BtEvent::DhtBootstrap,
                get_peers_event.clone(),
                announce_event.clone(),
            ]
        );

        let taken = collector.take_events();
        assert_eq!(taken.len(), 3);
        assert!(collector.events().is_empty());

        collector.on_event(get_peers_event);
        assert_eq!(collector.events().len(), 1);
        collector.clear();
        assert!(collector.events().is_empty());
    }

    #[test]
    fn test_bt_event_variants() {
        let info_hash = InfoHash::from_bytes([0x42; 20]);

        let events = vec![
            BtEvent::DhtAnnounce {
                info_hash,
                peer_ip: "10.0.0.1".to_string(),
                peer_port: 8080,
            },
            BtEvent::MetadataReceived {
                info_hash,
                data: vec![1, 2, 3],
            },
            BtEvent::MetadataFailed {
                info_hash,
                message: "fetch failed".to_string(),
            },
            BtEvent::DhtStats {
                node_count: 128,
                local_ip: "127.0.0.1".to_string(),
                local_port: 6881,
            },
            BtEvent::DhtBootstrap,
            BtEvent::DhtGetPeers { info_hash },
            BtEvent::AddTorrent {
                info_hash,
                message: "torrent added".to_string(),
            },
            BtEvent::AddTorrentError {
                info_hash,
                message: "add failed".to_string(),
                error_value: 1,
                error_category: "libtorrent".to_string(),
            },
            BtEvent::TorrentError {
                info_hash,
                message: "torrent error".to_string(),
            },
            BtEvent::FileError {
                info_hash,
                message: "file error".to_string(),
            },
            BtEvent::TorrentDeleteFailed {
                info_hash,
                message: "delete failed".to_string(),
            },
            BtEvent::SessionError {
                message: "session error".to_string(),
            },
            BtEvent::ListenFailed {
                message: "listen error".to_string(),
            },
            BtEvent::UdpError {
                message: "udp error".to_string(),
            },
            BtEvent::DhtError {
                message: "dht error".to_string(),
            },
            BtEvent::AlertsDropped {
                message: "dropped alerts".to_string(),
            },
        ];

        for event in events {
            let cloned = event.clone();
            assert_eq!(event, cloned);
            assert!(!format!("{event:?}").is_empty());
        }
    }

    #[test]
    fn test_engine_poll_events_with_collector() {
        use std::thread;
        use std::time::{Duration, Instant};

        let mut engine = Engine::new().expect("Engine should initialize");
        let mut collector = EventCollector::with_capacity(16);

        assert!(engine.post_dht_stats());

        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() <= deadline {
            engine.poll_events(&mut collector);

            if collector
                .events()
                .iter()
                .any(|e| matches!(e, BtEvent::DhtStats { .. }))
            {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }

        panic!("DhtStats event was not received before timeout");
    }
}
