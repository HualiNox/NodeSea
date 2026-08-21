// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include "libtorrent/info_hash.hpp"
#include "libtorrent/sha1_hash.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

namespace nodesea::bt {
struct TorrentIdPayload;
}

// Converts a libtorrent digest32 hash object into a std::array of bytes.
template <std::ptrdiff_t N>
inline std::array<std::uint8_t, N / 8> digest_to_array(libtorrent::digest32<N> const& digest) {
  std::array<std::uint8_t, N / 8> out{};
  std::copy(digest.begin(), digest.end(), out.begin());
  return out;
}

// Generate a unique key for a torrent based on its identity.
std::string torrent_id_key(nodesea::bt::TorrentIdPayload const& torrent_id);
// Generate a unique key for a torrent based on its info hash.
std::string torrent_id_key(lt::info_hash_t const& hashes);
// Convert a libtorrent info hash to a TorrentIdPayload.
nodesea::bt::TorrentIdPayload convert_to_torrent_id(lt::info_hash_t const& hashs);
// Convert a TorrentIdPayload to a libtorrent info hash.
lt::info_hash_t convert_to_info_hash(nodesea::bt::TorrentIdPayload const& torrent_id);
