# nodesea-bt

The BitTorrent engine used by NodeSea.

This crate provides a Rust interface to a libtorrent session. The native
session is kept behind a small C++ bridge; Rust code interacts with it through
the engine handle and receives converted domain events.

## Status

The crate is currently used as an internal workspace dependency. Its public
API is still subject to change.

## Engine model

An `Engine` is built with an optional `SettingsPack` and a set of
`EngineExtension`s. Calling `run` starts the native session and its event
loop. The returned `EngineHandle` can be cloned and used to submit commands
while the runner owns the session.

The runner is the only component that polls libtorrent alerts. Extensions are
called synchronously from the event loop and should return quickly.

## Current API

- engine lifecycle and status updates
- metadata fetching and cancellation
- DHT statistics and live-node snapshots
- BEP 51 infohash samples
- session statistics
- torrent, peer, DHT, and session events

## Examples

Observe DHT events:

```sh
cargo run -p nodesea-bt --example dht_observer
```

Fetch metadata announced by the DHT:

```sh
cargo run -p nodesea-bt --example announce_metadata_fetcher
```

## Building

The crate builds libtorrent and Boost through CMake during the Cargo build.
Building therefore requires:

- Rust and Cargo
- CMake 3.24 or newer
- a C++20 compiler
- network access for the CMake `FetchContent` dependencies

The native build uses libtorrent `v2.1.1` and a static libtorrent library.

## License

MIT
