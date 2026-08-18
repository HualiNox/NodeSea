// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>

#include "libtorrent/sha1_hash.hpp"

// Checks if a UDP port is available for local binding.
bool is_port_available(uint16_t port);

// Converts a libtorrent digest32 hash object into a std::array of bytes.
template <std::ptrdiff_t N>
inline std::array<std::uint8_t, N / 8>
digest_to_array(libtorrent::digest32<N> const &digest) {
  std::array<std::uint8_t, N / 8> out{};
  std::copy(digest.begin(), digest.end(), out.begin());
  return out;
}
