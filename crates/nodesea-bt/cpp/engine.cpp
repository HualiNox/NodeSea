// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/engine.hpp"

#include "libtorrent/address.hpp"
#include "libtorrent/session_handle.hpp"
#include "libtorrent/session_params.hpp"
#include "nodesea_bt/alert_parser.hpp"
#include "nodesea_bt/fetch_registry.hpp"
#include "nodesea_bt/helper.hpp"
#include "src/ffi.rs.h"
#include "src/ffi/config.rs.h"
#include "src/ffi/dht.rs.h"
#include "src/ffi/torrent.rs.h"

#include <cstddef>
#include <cstdint>
#include <libtorrent/add_torrent_params.hpp>
#include <libtorrent/alert.hpp>
#include <libtorrent/alert_types.hpp>
#include <libtorrent/error_code.hpp>
#include <libtorrent/info_hash.hpp>
#include <libtorrent/session.hpp>
#include <libtorrent/settings_pack.hpp>
#include <libtorrent/sha1_hash.hpp>
#include <libtorrent/span.hpp>
#include <libtorrent/torrent_flags.hpp>
#include <libtorrent/torrent_handle.hpp>
#include <libtorrent/torrent_info.hpp>
#include <memory>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

// ----------------------------------------------------------------------------
// Session Implementation
// -----------------------------------------------------------------------------
namespace lt = libtorrent;

namespace nodesea::bt {

struct Session::Impl {
  std::unique_ptr<lt::session> session_;
  FetchRegistry archive_fetches_;
};

Session::Session(SettingsPackPayload const& settings_pack) : impl_(std::make_unique<Impl>()) {
  lt::settings_pack sp;

  // Apply settings from the provided SettingsPackPayload.
  for (auto const& setting : settings_pack.settings) {
    switch (setting.kind) {
    case SettingKind::Int:
      sp.set_int(setting.key, setting.int_value);
      break;
    case SettingKind::Bool:
      sp.set_bool(setting.key, setting.bool_value);
      break;
    case SettingKind::String:
      sp.set_str(setting.key, std::string(setting.string_value));
      break;
    default:
      throw std::runtime_error("Unknown setting kind: " +
                               std::to_string(static_cast<int>(setting.kind)));
      break;
    }
  }

  // Start the session.
  impl_->session_ = std::make_unique<lt::session>(std::move(sp));
}

Session::~Session() = default;

lt::session& Session::native_session() { return *impl_->session_; }

// -----------------------------------------------------------------------------
// CommandPort Implementation
// -----------------------------------------------------------------------------

CommandPort::CommandPort(Session& session) : session_(session) {}

CommandPort::~CommandPort() = default;

bool CommandPort::fetch_metadata(const TorrentIdPayload& torrent_id) {
  return session_.fetch_metadata(torrent_id);
}

bool CommandPort::cancel_fetch(const TorrentIdPayload& torrent_id) {
  return session_.cancel_fetch(torrent_id);
}

bool CommandPort::post_dht_stats() {
  session_.native_session().post_dht_stats();
  return true;
}

bool CommandPort::post_session_stats() {
  session_.native_session().post_session_stats();
  return true;
}

bool CommandPort::post_dht_sample_infohashes(
    const UdpEndpointPayload& endpoint, const std::array<std::uint8_t, 20>& target) {
  auto& session = session_.native_session();
  lt::error_code ec;
  lt::address endpoint_address = lt::make_address(std::string(endpoint.address), ec);
  if (ec) {
    return false;
  }
  const lt::udp::endpoint remote(endpoint_address, static_cast<std::uint16_t>(endpoint.port));
  const lt::sha1_hash target_hash(reinterpret_cast<const char*>(target.data()));
  session.dht_sample_infohashes(remote, target_hash);
  return true;
}

bool CommandPort::post_dht_live_nodes() {
  auto& session = session_.native_session();
  lt::session_params state = session.session_state(lt::session_handle::save_dht_state);
  if (state.dht_state.nids.empty()) return false;
  for (auto const& [_, nid] : state.dht_state.nids) session.dht_live_nodes(nid);
  return true;
}

// -----------------------------------------------------------------------------
// Native Session Lifecycle
// -----------------------------------------------------------------------------

std::unique_ptr<Session> start_session(SettingsPackPayload const& settings_pack) {
  return std::make_unique<Session>(settings_pack);
}

std::uint64_t start_command_port(Session& session) {
  return reinterpret_cast<std::uint64_t>(new CommandPort(session));
}

void destroy_command_port(std::uint64_t port) {
  // The port must be destroyed before Session because it stores a Session&.
  delete reinterpret_cast<CommandPort*>(port);
}

// -----------------------------------------------------------------------------
// Session Event Operations
// -----------------------------------------------------------------------------

std::size_t Session::poll_events(FfiEventSink& sink) {
  if (!impl_->session_) {
    return 0;
  }

  std::vector<lt::alert*> alerts;
  impl_->session_->pop_alerts(&alerts);
  return dispatch_alerts(alerts, sink, *impl_->session_, impl_->archive_fetches_);
}

// -----------------------------------------------------------------------------
// Session Command Operations
// -----------------------------------------------------------------------------

bool Session::fetch_metadata(const TorrentIdPayload& torrent_id) {
  if (!impl_->session_) {
    return false;
  }

  if (!torrent_id.has_v1 && !torrent_id.has_v2) {
    return false;
  }

  std::string torrent_key = torrent_id_key(torrent_id);
  if (impl_->archive_fetches_.contains(torrent_key)) {
    return false;
  }

  lt::add_torrent_params params;
  params.info_hashes = convert_to_info_hash(torrent_id);
  params.save_path = ".";
  params.flags |= lt::torrent_flags::upload_mode | lt::torrent_flags::default_dont_download |
                  lt::torrent_flags::auto_managed;

  lt::error_code error;
  lt::torrent_handle handle = impl_->session_->add_torrent(std::move(params), error);
  if (error) {
    return false;
  }
  if (!impl_->archive_fetches_.try_insert(std::move(torrent_key), handle)) {
    return false;
  }
  return true;
}

bool Session::cancel_fetch(const TorrentIdPayload& torrent_id) {
  if (!impl_->session_) {
    return false;
  }

  std::string torrent_key = torrent_id_key(torrent_id);
  auto handle = impl_->archive_fetches_.find(torrent_key);
  if (!handle) {
    return false;
  }

  impl_->session_->remove_torrent(*handle, lt::session::delete_files);
  impl_->archive_fetches_.erase(torrent_key);
  return true;
}

// -----------------------------------------------------------------------------
// Alert Notification
// -----------------------------------------------------------------------------

void Session::set_alert_notify(AlertNotifier const& notifier) {
  if (!impl_->session_) {
    return;
  }
  impl_->session_->set_alert_notify([&notifier]() { notifier.notify(); });
}

void Session::clear_alert_notify() {
  if (!impl_->session_) {
    return;
  }

  impl_->session_->set_alert_notify({});
}

// -----------------------------------------------------------------------------
// CXX FFI Bridge Functions
// -----------------------------------------------------------------------------

std::size_t poll_events(Session& session, FfiEventSink& sink) {
  // Only this path consumes the native alert queue.
  return session.poll_events(sink);
}

bool fetch_metadata_from_port(std::uint64_t port, const TorrentIdPayload& torrent_id) {
  return reinterpret_cast<CommandPort*>(port)->fetch_metadata(torrent_id);
}

bool cancel_fetch_from_port(std::uint64_t port, const TorrentIdPayload& torrent_id) {
  return reinterpret_cast<CommandPort*>(port)->cancel_fetch(torrent_id);
}

bool post_dht_stats_from_port(std::uint64_t port) {
  return reinterpret_cast<CommandPort*>(port)->post_dht_stats();
}

bool post_session_stats_from_port(std::uint64_t port) {
  return reinterpret_cast<CommandPort*>(port)->post_session_stats();
}

bool post_dht_sample_infohashes_from_port(
    std::uint64_t port, const UdpEndpointPayload& endpoint,
    const std::array<std::uint8_t, 20>& target) {
  return reinterpret_cast<CommandPort*>(port)->post_dht_sample_infohashes(endpoint, target);
}

bool post_dht_live_nodes_from_port(std::uint64_t port) {
  return reinterpret_cast<CommandPort*>(port)->post_dht_live_nodes();
}

void set_alert_notify(Session& session, AlertNotifier const& notifier) {
  session.set_alert_notify(notifier);
}

void clear_alert_notify(Session& session) {
  session.clear_alert_notify();
}
} // namespace nodesea::bt
