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

} // namespace nodesea::bt

namespace libtorrent {
struct session;
}

namespace nodesea::bt {

// -----------------------------------------------------------------------------
// Session
// -----------------------------------------------------------------------------

// Owns one libtorrent session and exposes the native operations used by Rust.
class Session {
private:
  struct Impl;
  std::unique_ptr<Impl> impl_;

  // CommandPort is the sole native command owner outside the event runner.
  friend class CommandPort;
  // Exposes the underlying session only to the session-owned command facade.
  libtorrent::session& native_session();

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

  void set_alert_notify(AlertNotifier const& notifier);
  void clear_alert_notify();
};

// -----------------------------------------------------------------------------
// Command Port
// -----------------------------------------------------------------------------

// Provides command operations through a session-owned native facade. The
// facade must not outlive the Session that created it.
class CommandPort {
private:
  Session& session_;

public:
  explicit CommandPort(Session& session);
  ~CommandPort();

  CommandPort(const CommandPort&) = delete;
  CommandPort& operator=(const CommandPort&) = delete;

  bool fetch_metadata(const TorrentIdPayload& torrent_id);
  bool cancel_fetch(const TorrentIdPayload& torrent_id);
  // The following operations enqueue asynchronous libtorrent requests.
  bool post_dht_stats();
  bool post_session_stats();
  bool post_dht_sample_infohashes(const UdpEndpointPayload& endpoint,
                                  const std::array<std::uint8_t, 20>& target);
  bool post_dht_live_nodes();
};

// -----------------------------------------------------------------------------
// Native Session Lifecycle
// -----------------------------------------------------------------------------

// Creates a new BitTorrent engine instance.
std::unique_ptr<Session> start_session(SettingsPackPayload const& settings_pack);
std::uint64_t start_command_port(Session& session);
void destroy_command_port(std::uint64_t port);

// -----------------------------------------------------------------------------
// CXX FFI Bridge Functions
// -----------------------------------------------------------------------------

std::size_t poll_events(Session& session, FfiEventSink& sink);

bool fetch_metadata_from_port(std::uint64_t port, const TorrentIdPayload& torrent_id);

bool cancel_fetch_from_port(std::uint64_t port, const TorrentIdPayload& torrent_id);

bool post_dht_stats_from_port(std::uint64_t port);

bool post_session_stats_from_port(std::uint64_t port);

bool post_dht_sample_infohashes_from_port(std::uint64_t port,
                                          const UdpEndpointPayload& endpoint,
                                          const std::array<std::uint8_t, 20>& target);

bool post_dht_live_nodes_from_port(std::uint64_t port);

void set_alert_notify(Session& session, AlertNotifier const& notifier);
void clear_alert_notify(Session& session);
} // namespace nodesea::bt
