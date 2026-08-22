//! Private CXX bridges and the small Rust facade used by the domain layer.

//===----------------------------------------------------------------------===//
// Private bridge modules
//===----------------------------------------------------------------------===//

#[macro_use]
mod macros;
mod config;
mod dht;
mod notifier;
mod peer;
mod session;
mod sink;
mod torrent;

use std::net::SocketAddr;

use crate::{BtEvent, BtEventKind, DhtBootstrap, DhtTarget, EventSink, SettingsPack, TorrentId};
use sink::FfiEventSink;

pub(crate) use notifier::AlertNotifier;

//===----------------------------------------------------------------------===//
// Canonical native Session and callback bridge
//===----------------------------------------------------------------------===//

/// The canonical bridge owns the native Session binding and the one C++ sink
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
        fn on_session_stats(self: &mut FfiEventSink, event: SessionStatsPayload);
        fn on_external_ip(self: &mut FfiEventSink, event: ExternalIpPayload);
        fn on_torrent_removed(self: &mut FfiEventSink, event: TorrentRemovedPayload);
        fn on_peer_connect(self: &mut FfiEventSink, event: PeerConnectPayload);
        fn on_peer_disconnected(self: &mut FfiEventSink, event: PeerDisconnectedPayload);
        fn on_peer_error(self: &mut FfiEventSink, event: PeerErrorPayload);
        fn on_session_log(self: &mut FfiEventSink, event: SessionLogPayload);
        fn on_torrent_log(self: &mut FfiEventSink, event: TorrentLogPayload);
        fn on_peer_log(self: &mut FfiEventSink, event: PeerLogPayload);
        fn on_dht_log(self: &mut FfiEventSink, event: DhtLogPayload);
        fn on_piece_finished(self: &mut FfiEventSink, event: PieceFinishedPayload);
        fn on_block_finished(self: &mut FfiEventSink, event: BlockFinishedPayload);
        fn on_read_piece(self: &mut FfiEventSink, event: ReadPiecePayload);
        fn on_save_resume_data(self: &mut FfiEventSink, event: SaveResumeDataPayload);
    }

    unsafe extern "C++" {
        include!("src/ffi/config.rs.h");
        include!("src/ffi/dht.rs.h");
        include!("src/ffi/session.rs.h");
        include!("src/ffi/torrent.rs.h");
        include!("src/ffi/peer.rs.h");
        include!("nodesea_bt/engine.hpp");

        type DhtAnnouncePayload = crate::ffi::dht::DhtAnnouncePayload;
        type DhtStatsPayload = crate::ffi::dht::DhtStatsPayload;
        type DhtGetPeersPayload = crate::ffi::dht::DhtGetPeersPayload;
        type MetadataReceivedPayload = crate::ffi::torrent::MetadataReceivedPayload;
        type MetadataFailedPayload = crate::ffi::torrent::MetadataFailedPayload;
        type AddTorrentPayload = crate::ffi::torrent::AddTorrentPayload;
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
        type DhtLogPayload = crate::ffi::dht::DhtLogPayload;
        type SessionStatsPayload = crate::ffi::session::SessionStatsPayload;
        type ExternalIpPayload = crate::ffi::session::ExternalIpPayload;
        type SessionLogPayload = crate::ffi::session::SessionLogPayload;
        type TorrentRemovedPayload = crate::ffi::torrent::TorrentRemovedPayload;
        type TorrentLogPayload = crate::ffi::torrent::TorrentLogPayload;
        type ReadPiecePayload = crate::ffi::torrent::ReadPiecePayload;
        type SaveResumeDataPayload = crate::ffi::torrent::SaveResumeDataPayload;
        type PeerConnectPayload = crate::ffi::peer::PeerConnectPayload;
        type PeerDisconnectedPayload = crate::ffi::peer::PeerDisconnectedPayload;
        type PeerErrorPayload = crate::ffi::peer::PeerErrorPayload;
        type PeerLogPayload = crate::ffi::peer::PeerLogPayload;
        type PieceFinishedPayload = crate::ffi::peer::PieceFinishedPayload;
        type BlockFinishedPayload = crate::ffi::peer::BlockFinishedPayload;
        type SettingsPackPayload = crate::ffi::config::SettingsPackPayload;

        /// Opaque native engine owned by the Rust facade wrapper.
        type Session;

        fn start_session(settings_pack: &SettingsPackPayload) -> Result<UniquePtr<Session>>;
        fn poll_events(session: Pin<&mut Session>, sink: &mut FfiEventSink) -> usize;
        fn fetch_metadata(session: Pin<&mut Session>, torrent_id: &TorrentIdPayload) -> bool;
        fn cancel_fetch(session: Pin<&mut Session>, torrent_id: &TorrentIdPayload) -> bool;
        fn post_dht_stats(session: &Session) -> bool;
        fn post_dht_sample_infohashes(
            session: &Session,
            endpoint: &UdpEndpointPayload,
            target: &[u8; 20],
        ) -> bool;
        fn post_dht_live_nodes(session: &Session) -> bool;
    }

    extern "Rust" {
        type AlertNotifier;

        fn notify(self: &AlertNotifier);
    }

    unsafe extern "C++" {
        fn set_alert_notify(session: Pin<&mut Session>, notifier: &AlertNotifier);
        fn clear_alert_notify(session: Pin<&mut Session>);
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
    event_callback!(on_session_stats, bridge::SessionStatsPayload);
    event_callback!(on_external_ip, bridge::ExternalIpPayload);
    event_callback!(on_torrent_removed, bridge::TorrentRemovedPayload);
    event_callback!(on_peer_connect, bridge::PeerConnectPayload);
    event_callback!(on_peer_disconnected, bridge::PeerDisconnectedPayload);
    event_callback!(on_peer_error, bridge::PeerErrorPayload);
    event_callback!(on_session_log, bridge::SessionLogPayload);
    event_callback!(on_torrent_log, bridge::TorrentLogPayload);
    event_callback!(on_peer_log, bridge::PeerLogPayload);
    event_callback!(on_dht_log, bridge::DhtLogPayload);
    event_callback!(on_piece_finished, bridge::PieceFinishedPayload);
    event_callback!(on_block_finished, bridge::BlockFinishedPayload);
    event_callback!(on_read_piece, bridge::ReadPiecePayload);
    event_callback!(on_save_resume_data, bridge::SaveResumeDataPayload);

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

/// Rust-owned wrapper around the private generated CXX Session binding.
pub(super) struct Session {
    /// Opaque native engine allocation owned by this Rust wrapper.
    inner: cxx::UniquePtr<bridge::Session>,
}

//===----------------------------------------------------------------------===//
// Rust FFI facade
//===----------------------------------------------------------------------===//

/// Creates a Rust-owned wrapper around the native engine.
pub(super) fn start_session(settings_pack: SettingsPack) -> Result<Session, String> {
    let settings_pack = bridge::SettingsPackPayload::from(settings_pack);
    bridge::start_session(&settings_pack)
        .map(|inner| Session { inner })
        .map_err(|error| error.what().to_owned())
}

/// Polls native alerts and dispatches them to the supplied domain sink.
pub(super) fn poll_events<S: EventSink>(session: &mut Session, sink: &mut S) -> usize {
    let mut ffi_sink = FfiEventSink::new(sink);
    bridge::poll_events(session.inner.pin_mut(), &mut ffi_sink)
}

/// Starts metadata fetching for a torrent identity.
pub(super) fn fetch_metadata(session: &mut Session, torrent_id: &TorrentId) -> bool {
    let payload = bridge::TorrentIdPayload::from_torrent_id(torrent_id);
    bridge::fetch_metadata(session.inner.pin_mut(), &payload)
}

/// Cancels metadata fetching for a torrent identity.
pub(super) fn cancel_fetch(session: &mut Session, torrent_id: &TorrentId) -> bool {
    let payload = bridge::TorrentIdPayload::from_torrent_id(torrent_id);
    bridge::cancel_fetch(session.inner.pin_mut(), &payload)
}

/// Requests an asynchronous DHT statistics alert.
pub(super) fn post_dht_stats(session: &Session) -> bool {
    bridge::post_dht_stats(&session.inner)
}

/// Requests an asynchronous DHT sample infohashes alert.
pub(super) fn post_dht_sample_infohashes(
    session: &Session,
    endpoint: &SocketAddr,
    target: &DhtTarget,
) -> bool {
    bridge::post_dht_sample_infohashes(
        &session.inner,
        &bridge::UdpEndpointPayload::from_socket_addr(endpoint),
        target.as_bytes(),
    )
}

/// Requests live nodes from each local DHT routing table.
pub(super) fn post_dht_live_nodes(session: &Session) -> bool {
    bridge::post_dht_live_nodes(&session.inner)
}

pub(super) fn set_alert_notify(session: &mut Session, notifier: &notifier::AlertNotifier) {
    bridge::set_alert_notify(session.inner.pin_mut(), notifier);
}

pub(super) fn clear_alert_notify(session: &mut Session) {
    bridge::clear_alert_notify(session.inner.pin_mut());
}
