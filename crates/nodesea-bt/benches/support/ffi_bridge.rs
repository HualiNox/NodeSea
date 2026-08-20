use nodesea_bt::{BtEvent, DhtInfoHash, EventSink};

/// Rust-owned adapter used only by the benchmark bridge.
pub(super) struct FfiBenchSink {
    /// Erased pointer to the benchmark event sink.
    data: *mut (),
    /// Type-specialized callback used to restore the sink type.
    emit_fn: unsafe fn(*mut (), BtEvent),
}

impl FfiBenchSink {
    /// Creates a synchronous benchmark callback adapter.
    pub(super) fn new<S: EventSink>(sink: &mut S) -> Self {
        Self {
            data: sink as *mut S as *mut (),
            emit_fn: emit::<S>,
        }
    }

    fn emit(&mut self, event: BtEvent) {
        unsafe { (self.emit_fn)(self.data, event) }
    }
}

unsafe fn emit<S: EventSink>(data: *mut (), event: BtEvent) {
    unsafe { (&mut *data.cast::<S>()).on_event(event) };
}

#[cxx::bridge(namespace = "nodesea::bt::bench")]
mod bridge {
    extern "Rust" {
        type FfiBenchSink;

        fn on_dht_get_peers(self: &mut FfiBenchSink, event: DhtGetPeersPayload);
        fn on_dht_announce(self: &mut FfiBenchSink, event: DhtAnnouncePayload);
    }

    /// Payload for a benchmarked DHT get-peers event.
    pub(super) struct DhtGetPeersPayload {
        /// 20-byte DHT infohash associated with the lookup.
        info_hash: [u8; 20],
    }

    /// Payload for a benchmarked DHT announce event.
    pub(super) struct DhtAnnouncePayload {
        /// 20-byte DHT infohash associated with the announce.
        info_hash: [u8; 20],
        /// Announced peer IP address.
        peer_ip: String,
        /// Announced peer port.
        peer_port: u16,
    }

    unsafe extern "C++" {
        include!("nodesea_bt/bench.hpp");

        /// Dispatches a batch of synthetic DHT get-peers events.
        fn bench_dht_get_peers_batch(sink: &mut FfiBenchSink, count: usize) -> usize;

        /// Dispatches a batch of synthetic DHT announce events.
        fn bench_dht_announce_batch(sink: &mut FfiBenchSink, count: usize) -> usize;
    }
}

impl FfiBenchSink {
    fn on_dht_get_peers(&mut self, event: bridge::DhtGetPeersPayload) {
        self.emit(BtEvent::DhtGetPeers {
            info_hash: DhtInfoHash::from(event.info_hash),
        });
    }

    fn on_dht_announce(&mut self, event: bridge::DhtAnnouncePayload) {
        self.emit(BtEvent::DhtAnnounce {
            info_hash: DhtInfoHash::from(event.info_hash),
            peer_ip: event.peer_ip,
            peer_port: event.peer_port,
        });
    }
}

pub(super) use bridge::{bench_dht_announce_batch, bench_dht_get_peers_batch};
