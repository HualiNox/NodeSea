//! Private CXX bridges and the small Rust facade used by the domain layer.

//===----------------------------------------------------------------------===//
// Private bridge modules
//===----------------------------------------------------------------------===//

#[macro_use]
mod macros;
mod dht;
mod session;
mod sink;
mod torrent;

use std::net::SocketAddr;

use crate::{BtEvent, DhtTarget, EventSink, InfoHash, ffi::dht::UdpEndpoint};
use sink::FfiEventSink;

//===----------------------------------------------------------------------===//
// Canonical Engine and callback bridge
//===----------------------------------------------------------------------===//

/// The canonical bridge owns the native Engine binding and the one C++ sink
/// type. Domain-specific wire payloads are owned by the private child bridges.
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
        fn on_dht_sample_infohashes(self: &mut FfiEventSink, event: DhtSampleInfohashesPayload);
        fn on_dht_pkt(self: &mut FfiEventSink, event: DhtPktPayload);
        fn on_dht_live_nodes(self: &mut FfiEventSink, event: DhtLiveNodes);
    }

    unsafe extern "C++" {
        include!("src/ffi/dht.rs.h");
        include!("src/ffi/session.rs.h");
        include!("src/ffi/torrent.rs.h");
        include!("nodesea_bt/engine.hpp");

        type DhtAnnouncePayload = crate::ffi::dht::DhtAnnouncePayload;
        type DhtStatsPayload = crate::ffi::dht::DhtStatsPayload;
        type DhtGetPeersPayload = crate::ffi::dht::DhtGetPeersPayload;
        type MetadataReceivedPayload = crate::ffi::torrent::MetadataReceivedPayload;
        type InfoMessagePayload = crate::ffi::torrent::InfoMessagePayload;
        type AddTorrentErrorPayload = crate::ffi::torrent::AddTorrentErrorPayload;
        type MessagePayload = crate::ffi::session::MessagePayload;
        type UdpEndpoint = crate::ffi::dht::UdpEndpoint;
        type DhtSampleInfohashesPayload = crate::ffi::dht::DhtSampleInfohashesPayload;
        type DhtPktPayload = crate::ffi::dht::DhtPktPayload;
        type DhtLiveNodes = crate::ffi::dht::DhtLiveNodes;

        /// Opaque native engine owned by the Rust facade wrapper.
        type Engine;

        fn new_engine() -> UniquePtr<Engine>;
        fn poll_events(engine: Pin<&mut Engine>, sink: &mut FfiEventSink) -> usize;
        fn fetch_metadata(engine: Pin<&mut Engine>, info_hash: &[u8; 20]) -> bool;
        fn cancel_fetch(engine: Pin<&mut Engine>, info_hash: &[u8; 20]) -> bool;
        fn post_dht_stats(engine: &Engine) -> bool;
        fn post_dht_sample_infohashes(
            engine: &Engine,
            endpoint: &UdpEndpoint,
            target: &[u8; 20],
        ) -> bool;
        fn post_dht_live_nodes(engine: &Engine) -> bool;
    }
}

//===----------------------------------------------------------------------===//
// Callback dispatch
//===----------------------------------------------------------------------===//

impl FfiEventSink {
    fn on_dht_announce(&mut self, event: bridge::DhtAnnouncePayload) {
        self.emit(event.into());
    }

    fn on_metadata_received(&mut self, event: bridge::MetadataReceivedPayload) {
        self.emit(event.into());
    }

    info_message_callback!(on_metadata_failed, MetadataFailed);

    fn on_dht_stats(&mut self, event: bridge::DhtStatsPayload) {
        self.emit(event.into());
    }

    fn on_dht_bootstrap(&mut self) {
        self.emit(BtEvent::DhtBootstrap);
    }

    fn on_dht_get_peers(&mut self, event: bridge::DhtGetPeersPayload) {
        self.emit(event.into());
    }

    info_message_callback!(on_add_torrent, AddTorrent);

    fn on_add_torrent_error(&mut self, event: bridge::AddTorrentErrorPayload) {
        self.emit(event.into());
    }

    info_message_callback!(on_torrent_error, TorrentError);
    info_message_callback!(on_file_error, FileError);
    info_message_callback!(on_torrent_delete_failed, TorrentDeleteFailed);
    message_callback!(on_session_error, SessionError);
    message_callback!(on_listen_failed, ListenFailed);
    message_callback!(on_udp_error, UdpError);
    message_callback!(on_dht_error, DhtError);
    message_callback!(on_alerts_dropped, AlertsDropped);

    fn on_dht_sample_infohashes(&mut self, event: bridge::DhtSampleInfohashesPayload) {
        self.emit(event.into());
    }

    fn on_dht_pkt(&mut self, event: bridge::DhtPktPayload) {
        self.emit(event.into());
    }

    fn on_dht_live_nodes(&mut self, event: bridge::DhtLiveNodes) {
        self.emit(event.into());
    }
}

//===----------------------------------------------------------------------===//
// Rust-owned native wrapper
//===----------------------------------------------------------------------===//

/// Rust-owned wrapper around the private generated CXX Engine binding.
pub(super) struct Engine {
    inner: cxx::UniquePtr<bridge::Engine>,
}

//===----------------------------------------------------------------------===//
// Rust FFI facade
//===----------------------------------------------------------------------===//

/// Creates a Rust-owned wrapper around the native engine.
pub(super) fn new_engine() -> Option<Engine> {
    let inner = bridge::new_engine();
    (!inner.is_null()).then_some(Engine { inner })
}

/// Polls native alerts and dispatches them to the supplied domain sink.
pub(super) fn poll_events<S: EventSink>(engine: &mut Engine, sink: &mut S) -> usize {
    let mut ffi_sink = FfiEventSink::new(sink);
    bridge::poll_events(engine.inner.pin_mut(), &mut ffi_sink)
}

/// Starts metadata fetching for an info hash.
pub(super) fn fetch_metadata(engine: &mut Engine, info_hash: &InfoHash) -> bool {
    bridge::fetch_metadata(engine.inner.pin_mut(), info_hash.as_bytes())
}

/// Cancels metadata fetching for an info hash.
pub(super) fn cancel_fetch(engine: &mut Engine, info_hash: &InfoHash) -> bool {
    bridge::cancel_fetch(engine.inner.pin_mut(), info_hash.as_bytes())
}

/// Requests an asynchronous DHT statistics alert.
pub(super) fn post_dht_stats(engine: &Engine) -> bool {
    bridge::post_dht_stats(&engine.inner)
}

/// Requests an asynchronous DHT sample infohashes alert.
pub(super) fn post_dht_sample_infohashes(
    engine: &Engine,
    endpoint: &SocketAddr,
    target: &DhtTarget,
) -> bool {
    bridge::post_dht_sample_infohashes(
        &engine.inner,
        &UdpEndpoint::from_socket_addr(endpoint),
        target.as_bytes(),
    )
}

/// Requests live nodes from each local DHT routing table.
pub(super) fn post_dht_live_nodes(engine: &Engine) -> bool {
    bridge::post_dht_live_nodes(&engine.inner)
}
