use nodesea_bt::{BtEvent, EventSink};

/// Rust-owned adapter used only by the benchmark bridge.
pub(crate) struct FfiBenchSink {
    data: *mut (),
    emit_fn: unsafe fn(*mut (), BtEvent),
}

impl FfiBenchSink {
    /// Creates a synchronous benchmark callback adapter.
    pub(crate) fn new<S: EventSink>(sink: &mut S) -> Self {
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

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    extern "Rust" {
        type FfiBenchSink;

        fn on_dht_get_peers(self: &mut FfiBenchSink, event: DhtGetPeersPayload);
        fn on_dht_announce(self: &mut FfiBenchSink, event: DhtAnnouncePayload);
    }

    /// Payload for a benchmarked DHT get-peers event.
    pub(crate) struct DhtGetPeersPayload {
        info_hash: [u8; 20],
    }

    /// Payload for a benchmarked DHT announce event.
    pub(crate) struct DhtAnnouncePayload {
        info_hash: [u8; 20],
        peer_ip: String,
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
            info_hash: event.info_hash.into(),
        });
    }

    fn on_dht_announce(&mut self, event: bridge::DhtAnnouncePayload) {
        self.emit(BtEvent::DhtAnnounce {
            info_hash: event.info_hash.into(),
            peer_ip: event.peer_ip,
            peer_port: event.peer_port,
        });
    }
}

pub(crate) use bridge::{bench_dht_announce_batch, bench_dht_get_peers_batch};
