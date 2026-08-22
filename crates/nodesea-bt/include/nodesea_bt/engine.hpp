// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <memory>

namespace nodesea::bt {

struct FfiEventSink;
struct UdpEndpointPayload;
struct TorrentIdPayload;
struct SettingsPackPayload;
struct AlertNotifier;

// Owns one libtorrent session and exposes the native operations used by Rust.
class Session {
private:
  struct Impl;
  std::unique_ptr<Impl> impl_;

public:
  Session(SettingsPackPayload const& settings_pack);
  ~Session();

  Session(const Session&) = delete;
  Session& operator=(const Session&) = delete;

  // Polls for alerts from the BitTorrent session and dispatches them
  // directly into the Rust EventSink adapter. Returns the number of alerts
  // dispatched; alerts with no registered event mapping are not counted.
  std::size_t poll_events(FfiEventSink& sink);

  // Initiates metadata-only download for a torrent v1, v2, or hybrid identity.
  bool fetch_metadata(const TorrentIdPayload& torrent_id);

  // Cancels an ongoing metadata fetch task.
  bool cancel_fetch(const TorrentIdPayload& torrent_id);

  // Posts DHT statistics to update the node count.
  bool post_dht_stats() const;

  // Posts session statistics. Results are dispatched asynchronously as a
  // session-stats alert.
  bool post_session_stats() const;

  // Requests BEP 51 infohash samples from a remote DHT endpoint. The target
  // directs key-space traversal and does not affect the returned samples.
  bool post_dht_sample_infohashes(const UdpEndpointPayload& endpoint,
                                  const std::array<std::uint8_t, 20>& target) const;

  // Requests snapshots of the live nodes in each local DHT routing table.
  // Returns false when no local DHT node is available. Results are dispatched
  // asynchronously as DHT live-nodes alerts.
  bool post_dht_live_nodes() const;

  void set_alert_notify(AlertNotifier const& notifier);
  void clear_alert_notify();
};

// Creates a new BitTorrent engine instance.
std::unique_ptr<Session> start_session(SettingsPackPayload const& settings_pack);

// -----------------------------------------------------------------------------
// CXX FFI Bridge Functions
// -----------------------------------------------------------------------------

std::size_t poll_events(Session& session, FfiEventSink& sink);

bool fetch_metadata(Session& session, const TorrentIdPayload& torrent_id);

bool cancel_fetch(Session& session, const TorrentIdPayload& torrent_id);

bool post_dht_stats(const Session& session);

bool post_session_stats(const Session& session);

bool post_dht_sample_infohashes(const Session& session, const UdpEndpointPayload& endpoint,
                                const std::array<std::uint8_t, 20>& target);

// Requests live-node snapshots from the local DHT routing tables. Returns
// false when no local DHT node is available.
bool post_dht_live_nodes(const Session& session);

void set_alert_notify(Session& session, AlertNotifier const& notifier);
void clear_alert_notify(Session& session);
} // namespace nodesea::bt
