//! Rust bindings and event model for the Nodesea BitTorrent engine.
#![warn(missing_docs)]

use std::{collections::VecDeque, net::SocketAddr};
mod ffi;
mod types;

pub use types::{
    BtEvent, DhtDirection, DhtNode, DhtTarget, EventCollector, EventSink, InfoHash, NodeId,
};

/// A BitTorrent engine backed by a libtorrent session.
pub struct Engine {
    /// Rust facade around the private native CXX engine.
    inner: ffi::Engine,
    /// Events buffered by single-event polling.
    buffer: VecDeque<BtEvent>,
}

impl Engine {
    /// Creates a new BitTorrent engine instance.
    ///
    /// # Returns
    ///
    /// `None` if the underlying C++ engine could not be created.
    pub fn new() -> Option<Self> {
        ffi::new_engine().map(|inner| Self {
            inner,
            buffer: VecDeque::new(),
        })
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

        dispatched + ffi::poll_events(&mut self.inner, sink)
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
            ffi::poll_events(&mut self.inner, &mut queue_sink);
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
        ffi::fetch_metadata(&mut self.inner, info_hash)
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
        ffi::cancel_fetch(&mut self.inner, info_hash)
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

    /// Requests BEP 51 infohash samples from a remote DHT node.
    ///
    /// This request does not fetch torrent metadata. A successful response is
    /// delivered later as [`BtEvent::DhtSampleInfohashes`].
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The remote UDP endpoint to query.
    /// * `target` - The key-space traversal target. It does not affect the
    ///   samples returned by the remote node.
    ///
    /// # Returns
    ///
    /// `true` if the request was accepted by the local native session, `false`
    /// otherwise. It does not guarantee that the remote node supports BEP 51
    /// or returns a response.
    pub fn post_dht_sample_infohashes(&self, endpoint: &SocketAddr, target: &DhtTarget) -> bool {
        ffi::post_dht_sample_infohashes(&self.inner, endpoint, target)
    }

    /// Requests live nodes from each local DHT routing table.
    ///
    /// This operation only reads the local routing tables; it does not send a
    /// network request. A separate [`BtEvent::DhtLiveNodes`] is produced for
    /// each local DHT instance whose state is available.
    ///
    /// # Returns
    ///
    /// `true` if the request was accepted by the local native session, `false`
    /// otherwise. The resulting lists are delivered later as
    /// [`BtEvent::DhtLiveNodes`]. If the DHT is not started or has no local
    /// routing-table state, no live-nodes event may be produced.
    pub fn post_dht_live_nodes(&self) -> bool {
        ffi::post_dht_live_nodes(&self.inner)
    }
}

/// A queue-based event sink that buffers events in a deque.
/// Temporarily collects native callbacks while implementing single-event
/// polling on top of the batch polling interface.
struct QueueSink<'a> {
    /// Destination buffer for callbacks received during one poll.
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
