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

use crate::{BtEvent, BtEventKind, DhtBootstrap, DhtTarget, EventSink, TorrentId};
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
        fn on_metadata_failed(self: &mut FfiEventSink, event: MetadataFailedPayload);
        fn on_dht_stats(self: &mut FfiEventSink, event: DhtStatsPayload);
        fn on_dht_bootstrap(self: &mut FfiEventSink);
        fn on_dht_get_peers(self: &mut FfiEventSink, event: DhtGetPeersPayload);
        fn on_add_torrent(self: &mut FfiEventSink, event: AddTorrentPayload);
        fn on_add_torrent_error(self: &mut FfiEventSink, event: AddTorrentErrorPayload);
        fn on_torrent_error(self: &mut FfiEventSink, event: TorrentErrorPayload);
        fn on_file_error(self: &mut FfiEventSink, event: FileErrorPayload);
        fn on_torrent_delete_failed(self: &mut FfiEventSink, event: TorrentDeleteFailedPayload);
        fn on_session_error(self: &mut FfiEventSink, event: SessionErrorPayload);
        fn on_listen_failed(self: &mut FfiEventSink, event: ListenFailedPayload);
        fn on_udp_error(self: &mut FfiEventSink, event: UdpErrorPayload);
        fn on_dht_error(self: &mut FfiEventSink, event: DhtErrorPayload);
        fn on_alerts_dropped(self: &mut FfiEventSink, event: AlertsDroppedPayload);
        fn on_dht_sample_infohashes(self: &mut FfiEventSink, event: DhtSampleInfohashesPayload);
        fn on_dht_pkt(self: &mut FfiEventSink, event: DhtPktPayload);
        fn on_dht_live_nodes(self: &mut FfiEventSink, event: DhtLiveNodesPayload);
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
        type MetadataFailedPayload = crate::ffi::torrent::MetadataFailedPayload;
        type AddTorrentPayload = crate::ffi::torrent::AddTorrentPayload;
        type AddTorrentErrorPayload = crate::ffi::torrent::AddTorrentErrorPayload;
        type TorrentErrorPayload = crate::ffi::torrent::TorrentErrorPayload;
        type FileErrorPayload = crate::ffi::torrent::FileErrorPayload;
        type TorrentDeleteFailedPayload = crate::ffi::torrent::TorrentDeleteFailedPayload;
        type TorrentIdPayload = crate::ffi::torrent::TorrentIdPayload;
        type SessionErrorPayload = crate::ffi::session::SessionErrorPayload;
        type ListenFailedPayload = crate::ffi::session::ListenFailedPayload;
        type UdpErrorPayload = crate::ffi::session::UdpErrorPayload;
        type DhtErrorPayload = crate::ffi::session::DhtErrorPayload;
        type AlertsDroppedPayload = crate::ffi::session::AlertsDroppedPayload;
        type UdpEndpointPayload = crate::ffi::dht::UdpEndpointPayload;
        type DhtSampleInfohashesPayload = crate::ffi::dht::DhtSampleInfohashesPayload;
        type DhtPktPayload = crate::ffi::dht::DhtPktPayload;
        type DhtLiveNodesPayload = crate::ffi::dht::DhtLiveNodesPayload;

        /// Opaque native engine owned by the Rust facade wrapper.
        type Engine;

        fn new_engine() -> UniquePtr<Engine>;
        fn poll_events(engine: Pin<&mut Engine>, sink: &mut FfiEventSink) -> usize;
        fn fetch_metadata(engine: Pin<&mut Engine>, torrent_id: &TorrentIdPayload) -> bool;
        fn cancel_fetch(engine: Pin<&mut Engine>, torrent_id: &TorrentIdPayload) -> bool;
        fn post_dht_stats(engine: &Engine) -> bool;
        fn post_dht_sample_infohashes(
            engine: &Engine,
            endpoint: &UdpEndpointPayload,
            target: &[u8; 20],
        ) -> bool;
        fn post_dht_live_nodes(engine: &Engine) -> bool;
    }
}

//===----------------------------------------------------------------------===//
// Callback dispatch
//===----------------------------------------------------------------------===//

impl FfiEventSink {
    // Every payload has one explicit conversion to its corresponding domain event.
    event_callback!(on_dht_announce, bridge::DhtAnnouncePayload);
    event_callback!(on_metadata_received, bridge::MetadataReceivedPayload);
    event_callback!(on_metadata_failed, bridge::MetadataFailedPayload);
    event_callback!(on_dht_stats, bridge::DhtStatsPayload);
    event_callback!(on_dht_get_peers, bridge::DhtGetPeersPayload);
    event_callback!(on_add_torrent, bridge::AddTorrentPayload);
    event_callback!(on_add_torrent_error, bridge::AddTorrentErrorPayload);
    event_callback!(on_torrent_error, bridge::TorrentErrorPayload);
    event_callback!(on_file_error, bridge::FileErrorPayload);
    event_callback!(on_torrent_delete_failed, bridge::TorrentDeleteFailedPayload);
    event_callback!(on_session_error, bridge::SessionErrorPayload);
    event_callback!(on_listen_failed, bridge::ListenFailedPayload);
    event_callback!(on_udp_error, bridge::UdpErrorPayload);
    event_callback!(on_dht_error, bridge::DhtErrorPayload);
    event_callback!(on_alerts_dropped, bridge::AlertsDroppedPayload);
    event_callback!(on_dht_sample_infohashes, bridge::DhtSampleInfohashesPayload);
    event_callback!(on_dht_pkt, bridge::DhtPktPayload);
    event_callback!(on_dht_live_nodes, bridge::DhtLiveNodesPayload);

    // Callbacks without a payload that emit a fixed domain event.
    fn on_dht_bootstrap(&mut self) {
        self.emit(BtEvent::new(BtEventKind::DhtBootstrap(
            DhtBootstrap::from_ffi(),
        )));
    }
}

//===----------------------------------------------------------------------===//
// Rust-owned native wrapper
//===----------------------------------------------------------------------===//

/// Rust-owned wrapper around the private generated CXX Engine binding.
pub(super) struct Engine {
    /// Opaque native engine allocation owned by this Rust wrapper.
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

/// Starts metadata fetching for a torrent identity.
pub(super) fn fetch_metadata(engine: &mut Engine, torrent_id: &TorrentId) -> bool {
    let payload = bridge::TorrentIdPayload::from_torrent_id(torrent_id);
    bridge::fetch_metadata(engine.inner.pin_mut(), &payload)
}

/// Cancels metadata fetching for a torrent identity.
pub(super) fn cancel_fetch(engine: &mut Engine, torrent_id: &TorrentId) -> bool {
    let payload = bridge::TorrentIdPayload::from_torrent_id(torrent_id);
    bridge::cancel_fetch(engine.inner.pin_mut(), &payload)
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
        &bridge::UdpEndpointPayload::from_socket_addr(endpoint),
        target.as_bytes(),
    )
}

/// Requests live nodes from each local DHT routing table.
pub(super) fn post_dht_live_nodes(engine: &Engine) -> bool {
    bridge::post_dht_live_nodes(&engine.inner)
}
