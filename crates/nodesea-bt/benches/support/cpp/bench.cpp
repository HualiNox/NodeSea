// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/bench.hpp"

#include "benches/support/ffi_bridge.rs.h"

#include <array>
#include <cstdint>

namespace nodesea::bt::bench {

std::size_t bench_dht_get_peers_batch(FfiBenchSink& sink, std::size_t count) {
  const std::array<std::uint8_t, 20> hash{};

  for (std::size_t i = 0; i < count; ++i) {
    sink.on_dht_get_peers(DhtGetPeersPayload{
        .info_hash =
            InfoHash{
                .bytes = hash,
            },
    });
  }

  return count;
}

std::size_t bench_dht_announce_batch(FfiBenchSink& sink, std::size_t count) {
  const std::array<std::uint8_t, 20> hash{};

  for (std::size_t i = 0; i < count; ++i) {
    sink.on_dht_announce(DhtAnnouncePayload{
        .info_hash =
            InfoHash{
                .bytes = hash,
            },
        .peer_ip = rust::String("192.168.1.10"),
        .peer_port = 6881,
    });
  }

  return count;
}

} // namespace nodesea::bt::bench
