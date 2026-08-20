// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/helper.hpp"

#include "libtorrent/info_hash.hpp"
#include "libtorrent/sha1_hash.hpp"
#include "src/ffi/torrent.rs.h"

#include <cstdint>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>

const uint8_t INFO_HASH_V1_SIZE = 20;
const uint8_t INFO_HASH_V2_SIZE = 32;
const uint8_t KEY_FLAG_SIZE = 2;

bool is_port_available(uint16_t port) {
  // Create a UDP socket to probe local port availability
  int sock = ::socket(AF_INET, SOCK_DGRAM, 0);
  if (sock < 0) {
    return false;
  }

  sockaddr_in addr{};
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = INADDR_ANY;
  addr.sin_port = htons(port);

  int res = ::bind(sock, reinterpret_cast<const struct sockaddr*>(&addr), sizeof(addr));
  ::close(sock);

  return res == 0;
}

nodesea::bt::TorrentIdPayload convert_to_torrent_id(lt::info_hash_t const& hashs) {
  nodesea::bt::TorrentIdPayload torrent_id{};

  if (hashs.has_v1()) {
    torrent_id.v1 = digest_to_array(hashs.v1);
    torrent_id.has_v1 = true;
  }
  if (hashs.has_v2()) {
    torrent_id.v2 = digest_to_array(hashs.v2);
    torrent_id.has_v2 = true;
  }

  return torrent_id;
}

std::string torrent_id_key(nodesea::bt::TorrentIdPayload const& torrent_id) {
  std::string key;
  key.reserve(KEY_FLAG_SIZE + INFO_HASH_V1_SIZE + INFO_HASH_V2_SIZE);

  key.push_back(static_cast<char>(torrent_id.has_v1));
  key.push_back(static_cast<char>(torrent_id.has_v2));
  key.append(torrent_id.v1.data(), torrent_id.v1.data() + INFO_HASH_V1_SIZE);
  key.append(torrent_id.v2.data(), torrent_id.v2.data() + INFO_HASH_V2_SIZE);

  return key;
}

std::string torrent_id_key(lt::info_hash_t const& hashes) {
  std::string key;
  key.reserve(2 + 20 + 32);
  key.push_back(static_cast<char>(hashes.has_v1()));
  key.push_back(static_cast<char>(hashes.has_v2()));
  key.append(hashes.v1.data(), hashes.v1.data() + hashes.v1.size());
  key.append(hashes.v2.data(), hashes.v2.data() + hashes.v2.size());
  return key;
}

lt::info_hash_t convert_to_info_hash(nodesea::bt::TorrentIdPayload const& torrent_id) {
  lt::info_hash_t hashs;
  if (torrent_id.has_v1) {
    hashs.v1 = lt::sha1_hash(reinterpret_cast<const char*>(torrent_id.v1.data()));
  }
  if (torrent_id.has_v2) {
    hashs.v2 = lt::sha256_hash(reinterpret_cast<const char*>(torrent_id.v2.data()));
  }
  return hashs;
}
