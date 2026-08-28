// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include "nodesea_bt/fetch_registry.hpp"

#include <cstddef>
#include <libtorrent/session.hpp>
#include <vector>

namespace libtorrent {
struct alert;
}

namespace nodesea::bt {

struct FfiEventSink;

// Converts native alerts into Rust FFI callbacks and performs metadata cleanup.
std::size_t dispatch_alerts(std::vector<libtorrent::alert*> const& alerts, FfiEventSink& sink,
                            libtorrent::session& session, FetchRegistry& archive_fetches);

} // namespace nodesea::bt
