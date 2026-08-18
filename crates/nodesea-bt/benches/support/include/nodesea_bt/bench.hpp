// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include <cstddef>

namespace nodesea::bt {

struct FfiBenchSink;

std::size_t bench_dht_get_peers_batch(FfiBenchSink& sink, std::size_t count);
std::size_t bench_dht_announce_batch(FfiBenchSink& sink, std::size_t count);

} // namespace nodesea::bt
