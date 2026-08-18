use crate::{BtEvent, EventSink};

/// Rust-owned adapter passed to C++ for synchronous alert dispatch.
pub(crate) struct FfiEventSink {
    data: *mut (),
    emit_fn: unsafe fn(*mut (), BtEvent),
}

impl FfiEventSink {
    /// Creates a synchronous adapter for a concrete Rust event sink.
    ///
    /// The raw pointer is valid only while the caller is inside the native
    /// `poll_events` call; C++ must not store the adapter or invoke it later.
    pub(crate) fn new<S: EventSink>(sink: &mut S) -> Self {
        Self {
            data: sink as *mut S as *mut (),
            emit_fn: emit::<S>,
        }
    }

    fn emit(&mut self, event: BtEvent) {
        // The adapter is created and borrowed only for the duration of one
        // synchronous poll_events call. C++ never stores this pointer.
        unsafe { (self.emit_fn)(self.data, event) }
    }
}

unsafe fn emit<S: EventSink>(data: *mut (), event: BtEvent) {
    // `data` points to the `S` passed to `FfiEventSink::new`. The adapter is
    // used synchronously, so that value remains valid for every callback.
    unsafe { (&mut *data.cast::<S>()).on_event(event) };
}

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    extern "Rust" {
        type FfiEventSink;

        fn on_dht_announce(self: &mut FfiEventSink, event: DhtAnnouncePayload);
        fn on_metadata_received(self: &mut FfiEventSink, event: MetadataReceivedPayload);
        fn on_metadata_failed(self: &mut FfiEventSink, event: InfoMessagePayload);
        fn on_dht_stats(self: &mut FfiEventSink, event: DhtStatsPayload);
        fn on_dht_bootstrap(self: &mut FfiEventSink);
        fn on_dht_get_peers(self: &mut FfiEventSink, event: DhtGetPeersPayload);
        fn on_add_torrent(self: &mut FfiEventSink, event: InfoMessagePayload);
        fn on_add_torrent_error(self: &mut FfiEventSink, event: AddTorrentErrorPayload);
        fn on_torrent_error(self: &mut FfiEventSink, event: InfoMessagePayload);
        fn on_file_error(self: &mut FfiEventSink, event: InfoMessagePayload);
        fn on_torrent_delete_failed(self: &mut FfiEventSink, event: InfoMessagePayload);
        fn on_session_error(self: &mut FfiEventSink, event: MessagePayload);
        fn on_listen_failed(self: &mut FfiEventSink, event: MessagePayload);
        fn on_udp_error(self: &mut FfiEventSink, event: MessagePayload);
        fn on_dht_error(self: &mut FfiEventSink, event: MessagePayload);
        fn on_alerts_dropped(self: &mut FfiEventSink, event: MessagePayload);
    }

    /// Payload for a DHT announce alert.
    pub(crate) struct DhtAnnouncePayload {
        info_hash: [u8; 20],
        peer_ip: String,
        peer_port: u16,
    }

    /// Payload for a metadata-received alert.
    pub(crate) struct MetadataReceivedPayload {
        info_hash: [u8; 20],
        data: Vec<u8>,
    }

    /// Payload containing an info hash and message.
    pub(crate) struct InfoMessagePayload {
        info_hash: [u8; 20],
        message: String,
    }

    /// Payload containing a message only.
    pub(crate) struct MessagePayload {
        message: String,
    }

    /// Payload for DHT statistics.
    pub(crate) struct DhtStatsPayload {
        node_count: u32,
    }

    /// Payload for a DHT get peers alert.
    pub(crate) struct DhtGetPeersPayload {
        info_hash: [u8; 20],
    }

    /// Payload for a failed torrent-add operation.
    pub(crate) struct AddTorrentErrorPayload {
        info_hash: [u8; 20],
        message: String,
        error_value: i32,
        error_category: String,
    }

    unsafe extern "C++" {
        include!("nodesea_bt/engine.hpp");

        type Engine;

        /// Creates a new BitTorrent engine instance.
        fn new_engine() -> UniquePtr<Engine>;

        /// Polls alerts and dispatches each one to the supplied Rust sink.
        fn poll_events(engine: Pin<&mut Engine>, sink: &mut FfiEventSink) -> usize;

        /// Fetches metadata for a torrent identified by its info hash.
        fn fetch_metadata(engine: Pin<&mut Engine>, info_hash: &[u8; 20]) -> bool;

        /// Cancels metadata fetching for a torrent identified by its info hash.
        fn cancel_fetch(engine: Pin<&mut Engine>, info_hash: &[u8; 20]) -> bool;

        /// Requests an asynchronous DHT statistics alert.
        fn post_dht_stats(engine: &Engine) -> bool;
    }
}

impl FfiEventSink {
    fn on_dht_announce(&mut self, event: bridge::DhtAnnouncePayload) {
        self.emit(BtEvent::DhtAnnounce {
            info_hash: event.info_hash.into(),
            peer_ip: event.peer_ip,
            peer_port: event.peer_port,
        });
    }

    fn on_metadata_received(&mut self, event: bridge::MetadataReceivedPayload) {
        self.emit(BtEvent::MetadataReceived {
            info_hash: event.info_hash.into(),
            data: event.data,
        });
    }

    fn on_metadata_failed(&mut self, event: bridge::InfoMessagePayload) {
        self.emit(BtEvent::MetadataFailed {
            info_hash: event.info_hash.into(),
            message: event.message,
        });
    }

    fn on_dht_stats(&mut self, event: bridge::DhtStatsPayload) {
        self.emit(BtEvent::DhtStats {
            node_count: event.node_count,
        });
    }

    fn on_dht_bootstrap(&mut self) {
        self.emit(BtEvent::DhtBootstrap);
    }

    fn on_dht_get_peers(&mut self, event: bridge::DhtGetPeersPayload) {
        self.emit(BtEvent::DhtGetPeers {
            info_hash: event.info_hash.into(),
        });
    }

    fn on_add_torrent(&mut self, event: bridge::InfoMessagePayload) {
        self.emit(BtEvent::AddTorrent {
            info_hash: event.info_hash.into(),
            message: event.message,
        });
    }

    fn on_add_torrent_error(&mut self, event: bridge::AddTorrentErrorPayload) {
        self.emit(BtEvent::AddTorrentError {
            info_hash: event.info_hash.into(),
            message: event.message,
            error_value: event.error_value,
            error_category: event.error_category,
        });
    }

    fn on_torrent_error(&mut self, event: bridge::InfoMessagePayload) {
        self.emit(BtEvent::TorrentError {
            info_hash: event.info_hash.into(),
            message: event.message,
        });
    }

    fn on_file_error(&mut self, event: bridge::InfoMessagePayload) {
        self.emit(BtEvent::FileError {
            info_hash: event.info_hash.into(),
            message: event.message,
        });
    }

    fn on_torrent_delete_failed(&mut self, event: bridge::InfoMessagePayload) {
        self.emit(BtEvent::TorrentDeleteFailed {
            info_hash: event.info_hash.into(),
            message: event.message,
        });
    }

    fn on_session_error(&mut self, event: bridge::MessagePayload) {
        self.emit(BtEvent::SessionError {
            message: event.message,
        });
    }

    fn on_listen_failed(&mut self, event: bridge::MessagePayload) {
        self.emit(BtEvent::ListenFailed {
            message: event.message,
        });
    }

    fn on_udp_error(&mut self, event: bridge::MessagePayload) {
        self.emit(BtEvent::UdpError {
            message: event.message,
        });
    }

    fn on_dht_error(&mut self, event: bridge::MessagePayload) {
        self.emit(BtEvent::DhtError {
            message: event.message,
        });
    }

    fn on_alerts_dropped(&mut self, event: bridge::MessagePayload) {
        self.emit(BtEvent::AlertsDropped {
            message: event.message,
        });
    }
}

pub(crate) use bridge::*;
