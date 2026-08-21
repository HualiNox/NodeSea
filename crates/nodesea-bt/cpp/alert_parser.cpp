// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/alert_parser.hpp"

#include "nodesea_bt/helper.hpp"
#include "src/ffi.rs.h"
#include "src/ffi/dht.rs.h"
#include "src/ffi/session.rs.h"
#include "src/ffi/torrent.rs.h"

#include <cstdint>
#include <libtorrent/alert.hpp>
#include <libtorrent/alert_types.hpp>
#include <libtorrent/error_code.hpp>
#include <libtorrent/info_hash.hpp>
#include <libtorrent/span.hpp>
#include <libtorrent/sha1_hash.hpp>
#include <utility>

namespace nodesea::bt {

namespace lt = libtorrent;

std::size_t dispatch_alerts(
    std::vector<lt::alert*> const& alerts, FfiEventSink& sink, lt::session& session,
    std::unordered_map<std::string, lt::torrent_handle>& archive_fetches) {
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
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .data = std::move(data),
      });

      ++dispatched;

      // Clean up the fetch entry.
      std::string key = torrent_id_key(a->handle.info_hashes());
      session.remove_torrent(a->handle);
      archive_fetches.erase(key);
      break;
    }

    // Metadata failed alert.
    case lt::metadata_failed_alert::alert_type: {
      auto* a = static_cast<lt::metadata_failed_alert*>(alert);

      sink.on_metadata_failed(MetadataFailedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
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

      sink.on_add_torrent(AddTorrentPayload{
          .torrent_id = convert_to_torrent_id(a->params.info_hashes),
          .message = rust::String(a->message()),
          .has_error = a->error != lt::errors::no_error,
          .error_value = static_cast<std::int32_t>(a->error.value()),
          .error_category = rust::String(a->error.category().name()),
      });

      ++dispatched;

      break;
    }

    // Torrent error alert.
    case lt::torrent_error_alert::alert_type: {
      auto* a = static_cast<lt::torrent_error_alert*>(alert);

      sink.on_torrent_error(TorrentErrorPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // File error alert.
    case lt::file_error_alert::alert_type: {
      auto* a = static_cast<lt::file_error_alert*>(alert);

      sink.on_file_error(FileErrorPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;

      break;
    }

    // Torrent delete failed alert.
    case lt::torrent_delete_failed_alert::alert_type: {
      auto* a = static_cast<lt::torrent_delete_failed_alert*>(alert);

      sink.on_torrent_delete_failed(TorrentDeleteFailedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
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

    // Session stats alert.
    case lt::session_stats_alert::alert_type: {
      auto* a = static_cast<lt::session_stats_alert*>(alert);
      rust::Vec<std::int64_t> counters;
      for (std::int64_t value : a->counters()) {
        counters.push_back(value);
      }

      sink.on_session_stats(SessionStatsPayload{
          .counters = std::move(counters),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // External IP alert.
    case lt::external_ip_alert::alert_type: {
      auto* a = static_cast<lt::external_ip_alert*>(alert);

      sink.on_external_ip(ExternalIpPayload{
          .address = rust::String(a->external_address.to_string()),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Torrent removed alert.
    case lt::torrent_removed_alert::alert_type: {
      auto* a = static_cast<lt::torrent_removed_alert*>(alert);

      sink.on_torrent_removed(TorrentRemovedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Peer connect alert.
    case lt::peer_connect_alert::alert_type: {
      auto* a = static_cast<lt::peer_connect_alert*>(alert);

      sink.on_peer_connect(PeerConnectPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Peer disconnected alert.
    case lt::peer_disconnected_alert::alert_type: {
      auto* a = static_cast<lt::peer_disconnected_alert*>(alert);

      sink.on_peer_disconnected(PeerDisconnectedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Peer error alert.
    case lt::peer_error_alert::alert_type: {
      auto* a = static_cast<lt::peer_error_alert*>(alert);

      sink.on_peer_error(PeerErrorPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Session log alert.
    case lt::log_alert::alert_type: {
      auto* a = static_cast<lt::log_alert*>(alert);

      sink.on_session_log(SessionLogPayload{
          .message = rust::String(a->log_message()),
      });

      ++dispatched;
      break;
    }

    // Torrent log alert.
    case lt::torrent_log_alert::alert_type: {
      auto* a = static_cast<lt::torrent_log_alert*>(alert);

      sink.on_torrent_log(TorrentLogPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->log_message()),
      });

      ++dispatched;
      break;
    }

    // Peer log alert.
    case lt::peer_log_alert::alert_type: {
      auto* a = static_cast<lt::peer_log_alert*>(alert);

      sink.on_peer_log(PeerLogPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->log_message()),
      });

      ++dispatched;
      break;
    }

    // DHT log alert.
    case lt::dht_log_alert::alert_type: {
      auto* a = static_cast<lt::dht_log_alert*>(alert);

      sink.on_dht_log(DhtLogPayload{
          .message = rust::String(a->log_message()),
      });

      ++dispatched;
      break;
    }

    // Piece finished alert.
    case lt::piece_finished_alert::alert_type: {
      auto* a = static_cast<lt::piece_finished_alert*>(alert);

      sink.on_piece_finished(PieceFinishedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .piece_index = static_cast<std::int32_t>(a->piece_index),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Block finished alert.
    case lt::block_finished_alert::alert_type: {
      auto* a = static_cast<lt::block_finished_alert*>(alert);

      sink.on_block_finished(BlockFinishedPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .piece_index = static_cast<std::int32_t>(a->piece_index),
          .block_index = static_cast<std::int32_t>(a->block_index),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Read piece alert.
    case lt::read_piece_alert::alert_type: {
      auto* a = static_cast<lt::read_piece_alert*>(alert);
      rust::Vec<std::uint8_t> data;
      if (a->buffer && a->size > 0) {
        data.reserve(static_cast<std::size_t>(a->size));
        for (int index = 0; index < a->size; ++index) {
          data.push_back(static_cast<std::uint8_t>(a->buffer[index]));
        }
      }

      sink.on_read_piece(ReadPiecePayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .piece_index = static_cast<std::int32_t>(a->piece),
          .size = static_cast<std::int32_t>(a->size),
          .data = std::move(data),
          .message = rust::String(a->message()),
      });

      ++dispatched;
      break;
    }

    // Save resume data alert.
    case lt::save_resume_data_alert::alert_type: {
      auto* a = static_cast<lt::save_resume_data_alert*>(alert);

      sink.on_save_resume_data(SaveResumeDataPayload{
          .torrent_id = convert_to_torrent_id(a->handle.info_hashes()),
          .message = rust::String(a->message()),
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

} // namespace nodesea::bt
