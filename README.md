# NodeSea

NodeSea is a BitTorrent application under development.

## Daemon

The current executable is `nodesead`. It owns the local daemon transport and
will host the BitTorrent service.

```sh
cargo run --bin nodesead
```

On Linux, a root service uses the standard Unix socket path:

```text
/run/nodesea/socket
```

When run as a regular user, the socket is created at:

```text
$XDG_RUNTIME_DIR/nodesea/socket
```

Persistent application data will be stored under the user's XDG data or state
directory when the database layer is added.

On macOS, root services use `/var/run/nodesea/socket`. Regular users use:

```text
~/Library/Application Support/NodeSea/socket
```

The daemon creates and manages the socket itself. Homebrew only provides the
installation and service management; it does not affect socket discovery.


## Workspace crates

- `nodesea-bt` provides the libtorrent-backed engine.
- `nodesea-daemon` provides the daemon runtime and local transport.

## License

MIT
