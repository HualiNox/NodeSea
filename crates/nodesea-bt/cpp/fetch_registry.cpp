// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#include "nodesea_bt/fetch_registry.hpp"

#include <utility>

namespace nodesea::bt {

bool FetchRegistry::try_insert(std::string key, libtorrent::torrent_handle handle) {
  std::lock_guard lock(mutex_);
  return entries_.emplace(std::move(key), std::move(handle)).second;
}

std::optional<libtorrent::torrent_handle> FetchRegistry::find(std::string const& key) const {
  std::lock_guard lock(mutex_);
  auto const it = entries_.find(key);
  if (it == entries_.end()) {
    return std::nullopt;
  }
  return it->second;
}

bool FetchRegistry::contains(std::string const& key) const {
  std::lock_guard lock(mutex_);
  return entries_.contains(key);
}

bool FetchRegistry::erase(std::string const& key) {
  std::lock_guard lock(mutex_);
  return entries_.erase(key) != 0;
}

void FetchRegistry::clear() {
  std::lock_guard lock(mutex_);
  entries_.clear();
}

std::size_t FetchRegistry::size() const {
  std::lock_guard lock(mutex_);
  return entries_.size();
}

} // namespace nodesea::bt
