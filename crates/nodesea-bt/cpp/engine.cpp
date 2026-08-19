// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/engine.hpp"

#include "libtorrent/address.hpp"
#include "libtorrent/session_handle.hpp"
#include "libtorrent/session_params.hpp"
#include "libtorrent/time.hpp"
#include "nodesea_bt/helper.hpp"
#include "src/ffi.rs.h"
#include "src/ffi/dht.rs.h"
#include "src/ffi/session.rs.h"
#include "src/ffi/torrent.rs.h"

#include <cstddef>
#include <cstdint>
#include <format>
#include <libtorrent/add_torrent_params.hpp>
#include <libtorrent/alert.hpp>
#include <libtorrent/alert_types.hpp>
#include <libtorrent/error_code.hpp>
#include <libtorrent/session.hpp>
#include <libtorrent/settings_pack.hpp>
#include <libtorrent/sha1_hash.hpp>
#include <libtorrent/span.hpp>
#include <libtorrent/torrent_flags.hpp>
#include <libtorrent/torrent_handle.hpp>
#include <libtorrent/torrent_info.hpp>
#include <memory>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>
#include <vector>

// ----------------------------------------------------------------------------
// Configuration
// -----------------------------------------------------------------------------

// Default listening port for BitTorrent connections.
constexpr std::uint16_t DEFAULT_PORT = 6881;

// Random port selection fallback when default port is unavailable.
constexpr std::uint16_t RANDOM_PORT = 0;

// Format string for listening interfaces: IPv4 and IPv6 addresses.
constexpr std::string_view LISTEN_ADDR_FORMAT = "0.0.0.0:{0},[::]:{0}";

// DHT bootstrap nodes for peer discovery.
constexpr std::string_view DHT_BOOTSTRAP_NODES = "dht.libtorrent.org:25401,"
                                                 "router.bittorrent.com:6881,"
                                                 "dht.transmissionbt.com:6881";

// Alert categories to enable for the session.
constexpr auto ALERT_CATEGORIES = lt::alert_category::dht | lt::alert_category::dht_operation |
                                  lt::alert_category::status | lt::alert_category::error;

// ----------------------------------------------------------------------------
// Engine Implementation
// -----------------------------------------------------------------------------
namespace lt = libtorrent;

namespace nodesea::bt {

struct Engine::Impl {
  std::unique_ptr<lt::session> session_;
  std::unordered_map<std::string, lt::torrent_handle> archive_fetches_;
};

Engine::Engine() : impl_(std::make_unique<Impl>()) {
  lt::settings_pack sp;

  // Configure alert categories and enable DHT.
  sp.set_int(lt::settings_pack::alert_mask, ALERT_CATEGORIES);

  // Enable DHT.
  sp.set_bool(lt::settings_pack::enable_dht, true);

  // Configure listening interfaces with fallback.
  if (is_port_available(DEFAULT_PORT)) {
    sp.set_str(lt::settings_pack::listen_interfaces, std::format(LISTEN_ADDR_FORMAT, DEFAULT_PORT));
  } else {
    sp.set_str(lt::settings_pack::listen_interfaces, std::format(LISTEN_ADDR_FORMAT, RANDOM_PORT));
  }

  // Configure DHT bootstrap nodes.
  sp.set_str(lt::settings_pack::dht_bootstrap_nodes, std::string(DHT_BOOTSTRAP_NODES));

  // Start the session.
  impl_->session_ = std::make_unique<lt::session>(std::move(sp));
}

Engine::~Engine() = default;

std::unique_ptr<Engine> new_engine() {
  return std::make_unique<Engine>();
}

std::size_t Engine::poll_events(FfiEventSink& sink) {
  if (!impl_->session_) {
    return 0;
  }

  std::vector<lt::alert*> alerts;
  impl_->session_->pop_alerts(&alerts);
  std::size_t dispatched = 0;

  for (lt::alert* alert : alerts) {
    switch (alert->type()) {

    // DHT announce alert.
    case lt::dht_announce_alert::alert_type: {
      auto* a = static_cast<lt::dht_announce_alert*>(alert);

      sink.on_dht_announce(DhtAnnouncePayload{
          .info_hash = digest_to_array(a->info_hash),
          .peer_ip = rust::String(a->ip.to_string()),
          .peer_port = static_cast<std::uint16_t>(a->port),
      });

      ++dispatched;
      break;
    }

    // Metadata received alert.
    case lt::metadata_received_alert::alert_type: {
      auto* a = static_cast<lt::metadata_received_alert*>(alert);

      auto hash = digest_to_array(a->handle.info_hash());
      auto torrent_file = a->handle.torrent_file();
      rust::Vec<std::uint8_t> data;
      if (torrent_file && torrent_file->is_valid()) {
        lt::span<char const> info_section = torrent_file->info_section();
        data.reserve(info_section.size());
        for (char byte : info_section) {
          data.push_back(static_cast<std::uint8_t>(byte));
        }
      }
      sink.on_metadata_received(MetadataReceivedPayload{
          .info_hash = hash,
          .data = std::move(data),
      });

      ++dispatched;

      // Clean up the fetch entry.
      std::string key(reinterpret_cast<const char*>(hash.data()), 20);
      impl_->session_->remove_torrent(a->handle);
      impl_->archive_fetches_.erase(key);
      break;
    }

    // Metadata failed alert.
    case lt::metadata_failed_alert::alert_type: {
      auto* a = static_cast<lt::metadata_failed_alert*>(alert);

      sink.on_metadata_failed(MetadataFailedPayload{
          .info_hash = digest_to_array(a->handle.info_hash()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      // Don't clean up the fetch entry here. A metadata_failed_alert represents
      // a failed metadata attempt, and libtorrent may retry the retrieval.
      break;
    }

    // DHT bootstrap alert.
    case lt::dht_bootstrap_alert::alert_type: {
      sink.on_dht_bootstrap();

      ++dispatched;

      break;
    }

    // DHT stats alert.
    case lt::dht_stats_alert::alert_type: {
      auto* a = static_cast<lt::dht_stats_alert*>(alert);
      std::uint32_t total = 0;

      for (const auto& bucket : a->routing_table) {
        total += static_cast<std::uint32_t>(bucket.num_nodes);
      }

      sink.on_dht_stats(DhtStatsPayload{
          .node_count = total,
          .local_ip = rust::String(a->local_endpoint.address().to_string()),
          .local_port = static_cast<std::uint16_t>(a->local_endpoint.port()),
      });

      ++dispatched;

      break;
    }

    // DHT get peers alert.
    case lt::dht_get_peers_alert::alert_type: {
      auto* a = static_cast<lt::dht_get_peers_alert*>(alert);

      sink.on_dht_get_peers(DhtGetPeersPayload{
          .info_hash = digest_to_array(a->info_hash),
      });

      ++dispatched;

      break;
    }

    // Session error alert.
    case lt::session_error_alert::alert_type: {
      auto* a = static_cast<lt::session_error_alert*>(alert);

      sink.on_session_error(SessionErrorPayload{
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // Listen failed alert.
    case lt::listen_failed_alert::alert_type: {
      auto* a = static_cast<lt::listen_failed_alert*>(alert);

      sink.on_listen_failed(ListenFailedPayload{
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // UDP error alert.
    case lt::udp_error_alert::alert_type: {
      auto* a = static_cast<lt::udp_error_alert*>(alert);

      sink.on_udp_error(UdpErrorPayload{
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // DHT error alert.
    case lt::dht_error_alert::alert_type: {
      auto* a = static_cast<lt::dht_error_alert*>(alert);

      sink.on_dht_error(DhtErrorPayload{
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // Alerts dropped alert.
    case lt::alerts_dropped_alert::alert_type: {
      auto* a = static_cast<lt::alerts_dropped_alert*>(alert);

      sink.on_alerts_dropped(AlertsDroppedPayload{
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // Add torrent alert.
    case lt::add_torrent_alert::alert_type: {
      auto* a = static_cast<lt::add_torrent_alert*>(alert);

      auto hash = digest_to_array(a->params.info_hashes.get_best());
      if (a->error == lt::errors::no_error) {
        sink.on_add_torrent(AddTorrentPayload{
            .info_hash = hash,
            .message = rust::String(a->message()),
        });
      } else {
        sink.on_add_torrent_error(AddTorrentErrorPayload{
            .info_hash = hash,
            .message = rust::String(a->message()),
            .error_value = static_cast<std::int32_t>(a->error.value()),
            .error_category = rust::String(a->error.category().name()),
        });
      }

      ++dispatched;

      break;
    }

    // Torrent error alert.
    case lt::torrent_error_alert::alert_type: {
      auto* a = static_cast<lt::torrent_error_alert*>(alert);

      sink.on_torrent_error(TorrentErrorPayload{
          .info_hash = digest_to_array(a->handle.info_hash()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // File error alert.
    case lt::file_error_alert::alert_type: {
      auto* a = static_cast<lt::file_error_alert*>(alert);

      sink.on_file_error(FileErrorPayload{
          .info_hash = digest_to_array(a->handle.info_hash()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // Torrent delete failed alert.
    case lt::torrent_delete_failed_alert::alert_type: {
      auto* a = static_cast<lt::torrent_delete_failed_alert*>(alert);

      sink.on_torrent_delete_failed(TorrentDeleteFailedPayload{
          .info_hash = digest_to_array(a->handle.info_hash()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // DHT sample infohashes alert.
    case lt::dht_sample_infohashes_alert::alert_type: {
      auto* a = static_cast<lt::dht_sample_infohashes_alert*>(alert);

      // Convert libtorrent samples to private CXX wire payloads.
      rust::Vec<SampleInfoHashPayload> samples;
      for (lt::sha1_hash sample : a->samples()) {
        samples.push_back(SampleInfoHashPayload{
            .bytes = digest_to_array(sample),
        });
      }

      // Convert response DHT nodes to private CXX wire payloads.
      rust::Vec<DhtNodePayload> nodes;
      for (auto node : a->nodes()) {
        nodes.push_back(DhtNodePayload{
            .node_id = digest_to_array(node.first),
            .endpoint =
                UdpEndpointPayload{
                    .address = rust::String(node.second.address().to_string()),
                    .port = static_cast<uint16_t>(node.second.port()),
                },
        });
      }

      sink.on_dht_sample_infohashes(DhtSampleInfohashesPayload{
          .node =
              DhtNodePayload{
                  .node_id = digest_to_array(a->node_id),
                  .endpoint =
                      UdpEndpointPayload{
                          .address = rust::String(a->endpoint.address().to_string()),
                          .port = static_cast<std::uint16_t>(a->endpoint.port()),
                      },
              },
          .interval_secs = lt::total_seconds(a->interval),
          .num_infohashes = static_cast<std::int32_t>(a->num_infohashes),
          .samples = std::move(samples),
          .nodes = std::move(nodes),
      });

      ++dispatched;

      break;
    }

    // DHT packet alert.
    case lt::dht_pkt_alert::alert_type: {
      auto* a = static_cast<lt::dht_pkt_alert*>(alert);

      // Copy packet data from libtorrent to Rust Vec
      rust::Vec<std::uint8_t> packet;
      packet.reserve(a->pkt_buf().size());
      for (char buf : a->pkt_buf()) {
        packet.push_back(buf);
      }

      sink.on_dht_pkt(DhtPktPayload{
          .direction = a->direction == lt::dht_pkt_alert::incoming ? DhtDirectionPayload::Incoming
                                                                   : DhtDirectionPayload::Outgoing,
          .endpoint =
              UdpEndpointPayload{
                  .address = rust::String(a->node.address().to_string()),
                  .port = static_cast<std::uint16_t>(a->node.port()),
              },
          .packet = std::move(packet),
      });

      ++dispatched;

      break;
    }

    // DHT live nodes alert.
    case lt::dht_live_nodes_alert::alert_type: {
      auto* a = static_cast<lt::dht_live_nodes_alert*>(alert);

      // Convert libtorrent nodes to private CXX wire payloads.
      rust::Vec<DhtNodePayload> nodes;
      nodes.reserve(a->nodes().size());
      for (auto node : a->nodes()) {
        nodes.push_back(DhtNodePayload{
            .node_id = digest_to_array(node.first),
            .endpoint =
                UdpEndpointPayload{
                    .address = rust::String(node.second.address().to_string()),
                    .port = static_cast<std::uint16_t>(node.second.port()),
                },
        });
      }

      sink.on_dht_live_nodes(DhtLiveNodesPayload{
          .local_node_id = digest_to_array(a->node_id),
          .nodes = std::move(nodes),
      });

      ++dispatched;

      break;
    }

    default:
      break;
    }
  }

  return dispatched;
}

bool Engine::fetch_metadata(const std::array<std::uint8_t, 20>& info_hash) {
  if (!impl_->session_) {
    return false;
  }

  lt::sha1_hash sha1_hash(reinterpret_cast<const char*>(info_hash.data()));
  std::string sha1_hash_key = sha1_hash.to_string();

  if (impl_->archive_fetches_.contains(sha1_hash_key)) {
    return false;
  }

  lt::add_torrent_params params;
  params.info_hashes = lt::info_hash_t(sha1_hash);
  params.save_path = ".";
  params.flags |= lt::torrent_flags::upload_mode | lt::torrent_flags::default_dont_download |
                  lt::torrent_flags::auto_managed;

  lt::error_code error;
  lt::torrent_handle handle = impl_->session_->add_torrent(std::move(params), error);
  if (error) {
    return false;
  }
  impl_->archive_fetches_[sha1_hash_key] = handle;
  return true;
}

bool Engine::cancel_fetch(const std::array<std::uint8_t, 20>& info_hash) {
  if (!impl_->session_) {
    return false;
  }

  std::string sha1_hash_key =
      lt::sha1_hash(reinterpret_cast<const char*>(info_hash.data())).to_string();
  auto handle_it = impl_->archive_fetches_.find(sha1_hash_key);
  if (handle_it == impl_->archive_fetches_.end()) {
    return false;
  }

  impl_->session_->remove_torrent(handle_it->second, lt::session::delete_files);
  impl_->archive_fetches_.erase(handle_it);
  return true;
}

bool Engine::post_dht_stats() const {
  if (!impl_->session_) {
    return false;
  }

  impl_->session_->post_dht_stats();
  return true;
}

bool Engine::post_dht_sample_infohashes(const UdpEndpointPayload& endpoint,
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

bool Engine::post_dht_live_nodes() const {
  if (!impl_->session_) {
    return false;
  }

  lt::session_params state = impl_->session_->session_state(lt::session_handle::save_dht_state);

  for (auto const& [_, nid] : state.dht_state.nids) {
    impl_->session_->dht_live_nodes(nid);
  }

  return true;
}
// -----------------------------------------------------------------------------
// CXX FFI Bridge Functions
// -----------------------------------------------------------------------------

std::size_t poll_events(Engine& engine, FfiEventSink& sink) {
  return engine.poll_events(sink);
}

bool fetch_metadata(Engine& engine, const std::array<std::uint8_t, 20>& info_hash) {
  return engine.fetch_metadata(info_hash);
}

bool cancel_fetch(Engine& engine, const std::array<std::uint8_t, 20>& info_hash) {
  return engine.cancel_fetch(info_hash);
}

bool post_dht_stats(const Engine& engine) {
  return engine.post_dht_stats();
}

bool post_dht_sample_infohashes(const Engine& engine, const UdpEndpointPayload& endpoint,
                                const std::array<std::uint8_t, 20>& target) {
  return engine.post_dht_sample_infohashes(endpoint, target);
}

bool post_dht_live_nodes(const Engine& engine) {
  return engine.post_dht_live_nodes();
}

} // namespace nodesea::bt
