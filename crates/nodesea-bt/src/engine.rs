//! Public BitTorrent engine facade.

mod builder;
mod config;
mod queue;

pub use builder::EngineBuilder;
pub use config::*;

use std::{collections::VecDeque, net::SocketAddr};

use crate::{BtEvent, DhtTarget, EventSink, TorrentId, ffi};
use queue::QueueSink;

/// A BitTorrent engine backed by a libtorrent session.
pub struct Engine {
    /// Rust facade around the private native CXX engine.
    inner: ffi::Engine,
    /// Events buffered by single-event polling.
    buffer: VecDeque<BtEvent>,
}

impl Engine {
    /// Creates a builder for a BitTorrent engine.
    ///
    /// The builder starts with an empty [`SettingsPack`]. Configure it with
    /// [`EngineBuilder::set_settings_pack`] before calling
    /// [`EngineBuilder::build`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::Engine;
    ///
    /// let _engine = Engine::builder().build();
    /// ```
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Creates a new BitTorrent engine instance from the builder.
    ///
    /// This constructor is kept inside the engine module. Callers should use
    /// [`Engine::builder`] to construct an engine.
    pub(crate) fn new(settings_pack: SettingsPack) -> Option<Self> {
        ffi::new_engine(settings_pack).map(|inner| Self {
            inner,
            buffer: VecDeque::new(),
        })
    }

    /// Polls for a batch of events and sends them to the supplied sink.
    ///
    /// # Arguments
    ///
    /// - `sink` - Receives events as they are dispatched.
    ///
    /// # Returns
    ///
    /// The number of events delivered, including events buffered by a previous
    /// call to [`Engine::poll_event`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::{Engine, EventCollector};
    ///
    /// let mut engine = Engine::builder().build().unwrap();
    /// let mut sink = EventCollector::new();
    /// let _ = engine.poll_events(&mut sink);
    /// ```
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
    /// This method polls all currently available native alerts once and keeps
    /// the remaining events in an internal queue for subsequent calls.
    ///
    /// # Returns
    ///
    /// The next event, or `None` if no events are pending.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::Engine;
    ///
    /// let mut engine = Engine::builder().build().unwrap();
    /// let _event = engine.poll_event();
    /// ```
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

    /// Fetches metadata for a torrent using its v1, v2, or hybrid identity.
    ///
    /// # Arguments
    ///
    /// - `torrent_id` - The torrent identity to use for the metadata request.
    ///
    /// # Returns
    ///
    /// `true` if the fetch request was successfully initiated, otherwise
    /// `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::{Engine, InfoHashV1, TorrentId};
    ///
    /// let mut engine = Engine::builder().build().unwrap();
    /// let torrent_id = TorrentId::new(Some(InfoHashV1::from_bytes([0; 20])), None);
    /// let _ = engine.fetch_metadata(&torrent_id);
    /// ```
    pub fn fetch_metadata(&mut self, torrent_id: &TorrentId) -> bool {
        ffi::fetch_metadata(&mut self.inner, torrent_id)
    }

    /// Cancels a metadata fetch request for a torrent identity.
    ///
    /// # Arguments
    ///
    /// - `torrent_id` - The torrent identity used by the metadata request.
    ///
    /// # Returns
    ///
    /// `true` if the cancellation request was successfully initiated,
    /// otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::{Engine, InfoHashV1, TorrentId};
    ///
    /// let mut engine = Engine::builder().build().unwrap();
    /// let torrent_id = TorrentId::new(Some(InfoHashV1::from_bytes([0; 20])), None);
    /// let _ = engine.cancel_fetch_metadata(&torrent_id);
    /// ```
    pub fn cancel_fetch_metadata(&mut self, torrent_id: &TorrentId) -> bool {
        ffi::cancel_fetch(&mut self.inner, torrent_id)
    }

    /// Requests an asynchronous DHT statistics alert.
    ///
    /// The resulting node count is delivered later through
    /// [`crate::BtEventKind::DhtStats`].
    ///
    /// # Returns
    ///
    /// `true` if the request was posted successfully, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::Engine;
    ///
    /// let engine = Engine::builder().build().unwrap();
    /// let _ = engine.post_dht_stats();
    /// ```
    pub fn post_dht_stats(&self) -> bool {
        ffi::post_dht_stats(&self.inner)
    }

    /// Requests BEP 51 infohash samples from a remote DHT node.
    ///
    /// This request does not fetch torrent metadata. A successful response is
    /// delivered later as [`crate::BtEventKind::DhtSampleInfohashes`].
    ///
    /// # Arguments
    ///
    /// - `endpoint` - The remote UDP endpoint to query.
    /// - `target` - The key-space traversal target.
    ///
    /// # Returns
    ///
    /// `true` if the request was accepted by the local native session,
    /// otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::SocketAddr;
    /// use nodesea_bt::{DhtTarget, Engine};
    ///
    /// let engine = Engine::builder().build().unwrap();
    /// let endpoint: SocketAddr = "127.0.0.1:6881".parse().unwrap();
    /// let target = DhtTarget::from_bytes([0; 20]);
    /// let _ = engine.post_dht_sample_infohashes(&endpoint, &target);
    /// ```
    pub fn post_dht_sample_infohashes(&self, endpoint: &SocketAddr, target: &DhtTarget) -> bool {
        ffi::post_dht_sample_infohashes(&self.inner, endpoint, target)
    }

    /// Requests live nodes from each local DHT routing table.
    ///
    /// This operation only reads the local routing tables; it does not send a
    /// network request. The resulting lists are delivered later as
    /// [`crate::BtEventKind::DhtLiveNodes`].
    ///
    /// # Returns
    ///
    /// `true` if the request was accepted by the local native session,
    /// otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodesea_bt::Engine;
    ///
    /// let engine = Engine::builder().build().unwrap();
    /// let _ = engine.post_dht_live_nodes();
    /// ```
    pub fn post_dht_live_nodes(&self) -> bool {
        ffi::post_dht_live_nodes(&self.inner)
    }
}

// Engine encapsulates an internal libtorrent session which manages its own
// thread pool and IO context. Moving ownership across threads is safe.
unsafe impl Send for Engine {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BtEventKind, EventCollector, InfoHashV1, InfoHashV2};

    #[test]
    fn test_engine_lifecycle() {
        let mut engine = Engine::builder()
            .build()
            .expect("Failed to initialize Engine");

        assert_eq!(engine.poll_event(), None);
        assert!(engine.post_dht_stats());

        let dummy_hash = TorrentId::new(Some(InfoHashV1::from_bytes([0xef; 20])), None);
        assert!(engine.fetch_metadata(&dummy_hash));
        assert!(!engine.fetch_metadata(&dummy_hash));
        assert!(engine.cancel_fetch_metadata(&dummy_hash));
        assert!(!engine.cancel_fetch_metadata(&dummy_hash));

        let v2_id = TorrentId::new(None, Some(InfoHashV2::from_bytes([0xcd; 32])));
        assert!(engine.fetch_metadata(&v2_id));
        assert!(!engine.fetch_metadata(&v2_id));
        assert!(engine.cancel_fetch_metadata(&v2_id));

        let hybrid_id = TorrentId::new(
            Some(InfoHashV1::from_bytes([0xab; 20])),
            Some(InfoHashV2::from_bytes([0xcd; 32])),
        );
        assert!(engine.fetch_metadata(&hybrid_id));
        assert!(!engine.fetch_metadata(&hybrid_id));
        assert!(engine.cancel_fetch_metadata(&hybrid_id));
    }

    #[test]
    fn test_engine_poll_events_with_collector() {
        use std::thread;
        use std::time::{Duration, Instant};

        let mut engine = Engine::builder().build().expect("Engine should initialize");
        let mut collector = EventCollector::with_capacity(16);

        assert!(engine.post_dht_stats());

        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() <= deadline {
            engine.poll_events(&mut collector);

            if collector
                .events()
                .iter()
                .any(|event| matches!(event.kind(), BtEventKind::DhtStats(_)))
            {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }

        panic!("DhtStats event was not received before timeout");
    }
}
