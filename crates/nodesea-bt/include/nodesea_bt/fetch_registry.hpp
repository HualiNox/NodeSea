// SPDX-License-Identifier: MIT
// Copyright (c) 2026 HualiNox

#pragma once

#include <cstddef>
#include <libtorrent/torrent_handle.hpp>
#include <mutex>
#include <optional>
#include <string>
#include <unordered_map>

namespace nodesea::bt {

// Thread-safe registry for native metadata-fetch torrent handles.
//
// Each public operation locks the registry for the duration of that operation.
// Returned torrent handles are copies, so no container reference escapes the
// lock.
class FetchRegistry {
private:
  // Protects both the map structure and the lifetime of handles stored in it.
  mutable std::mutex mutex_;
  std::unordered_map<std::string, libtorrent::torrent_handle> entries_;

public:
  // Constructs an empty registry.
  FetchRegistry() = default;

  // A mutex-protected registry cannot be copied safely or meaningfully.
  FetchRegistry(const FetchRegistry&) = delete;
  FetchRegistry& operator=(const FetchRegistry&) = delete;

  // Inserts a fetch only when key is not already registered.
  //
  // Returns true when the entry was inserted and false when key already
  // exists. The check and insertion are one atomic registry operation.
  bool try_insert(std::string key, libtorrent::torrent_handle handle);

  // Returns a copy of the handle registered for key, or nullopt when absent.
  std::optional<libtorrent::torrent_handle> find(std::string const& key) const;

  // Returns whether key is currently registered.
  bool contains(std::string const& key) const;

  // Removes key and returns whether an entry was removed.
  bool erase(std::string const& key);

  // Removes all registered fetches.
  void clear();

  // Returns the number of currently registered fetches.
  std::size_t size() const;
};

} // namespace nodesea::bt
