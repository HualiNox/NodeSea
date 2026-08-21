// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include <cstddef>
#include <libtorrent/session.hpp>
#include <libtorrent/torrent_handle.hpp>
#include <string>
#include <unordered_map>
#include <vector>

namespace libtorrent {
struct alert;
}

namespace nodesea::bt {

struct FfiEventSink;

// Converts native alerts into Rust FFI callbacks and performs metadata cleanup.
std::size_t
dispatch_alerts(std::vector<libtorrent::alert*> const& alerts, FfiEventSink& sink,
                libtorrent::session& session,
                std::unordered_map<std::string, libtorrent::torrent_handle>& archive_fetches);

} // namespace nodesea::bt
