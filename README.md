# NodeSea

NodeSea is a BitTorrent application under development.

## Daemon

The current executable is `nodesead`. It owns the local daemon transport and
will host the BitTorrent service.

```sh
cargo run --bin nodesead
```

On Linux, `nodesead` is intended to run as a system service and currently
requires root. Its Unix socket is:

```text
/run/nodesea/nodesead.sock
```

Persistent application data will be stored under `/var/lib/nodesea` when the
database layer is added.

On macOS, the socket is stored at:

```text
~/Library/Application Support/NodeSea/nodesead.sock
```

## Workspace crates

- `nodesea-bt` provides the libtorrent-backed engine.
- `nodesea-daemon` provides the daemon runtime and local transport.

## License

MIT
