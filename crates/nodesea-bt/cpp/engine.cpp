// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/engine.hpp"

#include "libtorrent/address.hpp"
#include "libtorrent/session_handle.hpp"
#include "libtorrent/session_params.hpp"
#include "nodesea_bt/alert_parser.hpp"
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
#include <unordered_map>
#include <utility>
#include <vector>

// ----------------------------------------------------------------------------
// Session Implementation
// -----------------------------------------------------------------------------
namespace lt = libtorrent;

namespace nodesea::bt {

struct Session::Impl {
  std::unique_ptr<lt::session> session_;
  std::unordered_map<std::string, lt::torrent_handle> archive_fetches_;
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

std::unique_ptr<Session> start_session(SettingsPackPayload const& settings_pack) {
  return std::make_unique<Session>(settings_pack);
}

std::size_t Session::poll_events(FfiEventSink& sink) {
  if (!impl_->session_) {
    return 0;
  }

  std::vector<lt::alert*> alerts;
  impl_->session_->pop_alerts(&alerts);
  return dispatch_alerts(alerts, sink, *impl_->session_, impl_->archive_fetches_);
}

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
  impl_->archive_fetches_[torrent_key] = handle;
  return true;
}

bool Session::cancel_fetch(const TorrentIdPayload& torrent_id) {
  if (!impl_->session_) {
    return false;
  }

  std::string torrent_key = torrent_id_key(torrent_id);
  auto handle_it = impl_->archive_fetches_.find(torrent_key);
  if (handle_it == impl_->archive_fetches_.end()) {
    return false;
  }

  impl_->session_->remove_torrent(handle_it->second, lt::session::delete_files);
  impl_->archive_fetches_.erase(handle_it);
  return true;
}

bool Session::post_dht_stats() const {
  if (!impl_->session_) {
    return false;
  }

  impl_->session_->post_dht_stats();
  return true;
}

bool Session::post_dht_sample_infohashes(const UdpEndpointPayload& endpoint,
                                        const std::array<std::uint8_t, 20>& target) const {
  if (!impl_->session_) {
    return false;
  }

  // Convert Rust address string to libtorrent address.
  lt::error_code ec;
  lt::address endpoint_address = lt::make_address(std::string(endpoint.address), ec);
  if (ec) {
    return false;
  }

  // Create UDP endpoint.
  const lt::udp::endpoint rp(endpoint_address, static_cast<std::uint16_t>(endpoint.port));

  // Create target hash.
  const lt::sha1_hash target_hash(reinterpret_cast<const char*>(target.data()));

  impl_->session_->dht_sample_infohashes(rp, target_hash);
  return true;
}

bool Session::post_dht_live_nodes() const {
  if (!impl_->session_) {
    return false;
  }

  lt::session_params state = impl_->session_->session_state(lt::session_handle::save_dht_state);

  for (auto const& [_, nid] : state.dht_state.nids) {
    impl_->session_->dht_live_nodes(nid);
  }

  return true;
}

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
  return session.poll_events(sink);
}

bool fetch_metadata(Session& session, const TorrentIdPayload& torrent_id) {
  return session.fetch_metadata(torrent_id);
}

bool cancel_fetch(Session& session, const TorrentIdPayload& torrent_id) {
  return session.cancel_fetch(torrent_id);
}

bool post_dht_stats(const Session& session) {
  return session.post_dht_stats();
}

bool post_dht_sample_infohashes(const Session& session, const UdpEndpointPayload& endpoint,
                                const std::array<std::uint8_t, 20>& target) {
  return session.post_dht_sample_infohashes(endpoint, target);
}

bool post_dht_live_nodes(const Session& session) {
  return session.post_dht_live_nodes();
}

void set_alert_notify(Session& session, AlertNotifier const& notifier) {
  session.set_alert_notify(notifier);
}

void clear_alert_notify(Session& session) {
  session.clear_alert_notify();
}
} // namespace nodesea::bt
