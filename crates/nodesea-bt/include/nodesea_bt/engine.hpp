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

// BitTorrent observation and metadata fetching engine wrapper.
class Engine {
private:
  struct Impl;
  std::unique_ptr<Impl> impl_;

public:
  Engine(SettingsPackPayload const& settings_pack);
  ~Engine();

  Engine(const Engine&) = delete;
  Engine& operator=(const Engine&) = delete;

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

  // Requests BEP 51 infohash samples from a remote DHT endpoint. The target
  // directs key-space traversal and does not affect the returned samples.
  bool post_dht_sample_infohashes(const UdpEndpointPayload& endpoint,
                                  const std::array<std::uint8_t, 20>& target) const;

  // Requests snapshots of the live nodes in each local DHT routing table.
  // Results are dispatched asynchronously as DHT live-nodes alerts.
  bool post_dht_live_nodes() const;
};

// Creates a new BitTorrent engine instance.
std::unique_ptr<Engine> new_engine(SettingsPackPayload const& settings_pack);

// -----------------------------------------------------------------------------
// CXX FFI Bridge Functions
// -----------------------------------------------------------------------------

std::size_t poll_events(Engine& engine, FfiEventSink& sink);

bool fetch_metadata(Engine& engine, const TorrentIdPayload& torrent_id);

bool cancel_fetch(Engine& engine, const TorrentIdPayload& torrent_id);

bool post_dht_stats(const Engine& engine);

bool post_dht_sample_infohashes(const Engine& engine, const UdpEndpointPayload& endpoint,
                                const std::array<std::uint8_t, 20>& target);

// Requests live-node snapshots from the local DHT routing tables.
bool post_dht_live_nodes(const Engine& engine);

} // namespace nodesea::bt
