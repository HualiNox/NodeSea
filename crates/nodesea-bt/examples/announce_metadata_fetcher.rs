//! Fetch metadata from DHT announces and sample infohashes from live DHT peers.
//!
//! A DHT announce carries an unmarked 20-byte key. This example treats it as
//! a v1 info hash only to demonstrate the metadata-fetch flow.
//!
//! The event path is:
//!
//! - `DhtAnnounce` -> `post_fetch_metadata`
//! - `DhtLiveNodes` -> choose one peer -> `post_dht_sample_infohashes`
//! - a periodic timer -> refresh DHT statistics and live-node snapshots

use std::{collections::HashSet, net::SocketAddr, time::Duration};

use nodesea_bt::{
    AlertCategory, BoolSetting, BtEvent, BtEventKind, DhtInfoHash, DhtTarget, Engine,
    EngineExtension, EngineStatus, InfoHashV1, IntSetting, SettingsPack, StringSetting, TorrentId,
};
use tokio::sync::mpsc;

const BOOTSTRAP_NODES: &str =
    "dht.libtorrent.org:25401,router.bittorrent.com:6881,dht.transmissionbt.com:6881";

enum Command {
    /// Fetch metadata for the 20-byte key from a DHT announce.
    FetchMetadata(DhtInfoHash),
    /// Ask one known DHT peer for BEP 51 infohash samples.
    SampleInfohashes(SocketAddr),
}

/// Synchronous event adapter. Async engine commands are forwarded to `run`.
struct Observer {
    commands: mpsc::UnboundedSender<Command>,
}

impl EngineExtension for Observer {
    fn on_event(&mut self, event: &BtEvent) {
        match event.kind() {
            BtEventKind::DhtAnnounce(announce) => {
                let info_hash = *announce.info_hash();
                println!("DHT announce: {info_hash}");
                let _ = self.commands.send(Command::FetchMetadata(info_hash));
            }
            BtEventKind::DhtBootstrap(_) => println!("DHT bootstrap completed"),
            BtEventKind::DhtStats(stats) => println!(
                "DHT stats: {} nodes, local endpoint {}:{}",
                stats.node_count(),
                stats.local_ip(),
                stats.local_port()
            ),
            BtEventKind::DhtLiveNodes(snapshot) => {
                println!(
                    "DHT live nodes: local={}, {} nodes",
                    snapshot.local_node_id(),
                    snapshot.nodes().len()
                );
                // A live-node snapshot provides a concrete endpoint for BEP 51.
                if let Some(peer) = snapshot.nodes().first() {
                    let _ = self
                        .commands
                        .send(Command::SampleInfohashes(*peer.endpoint()));
                }
            }
            BtEventKind::DhtSampleInfohashes(sample) => {
                println!(
                    "DHT samples: peer={}, remote_count={}, samples={}, forwarded_nodes={}",
                    sample.node().endpoint(),
                    sample.num_infohashes(),
                    sample.samples().len(),
                    sample.nodes().len()
                );

                // BEP 51 samples are unmarked 20-byte values. This example
                // treats them as v1 keys and sends them through the same
                // metadata-fetch path as DHT announces.
                for info_hash in sample.samples() {
                    let _ = self.commands.send(Command::FetchMetadata(*info_hash));
                }
            }
            BtEventKind::AddTorrent(add) => println!("metadata fetch added: {add:?}"),
            BtEventKind::MetadataReceived(metadata) => println!(
                "metadata received: {} ({} bytes)",
                metadata.torrent_id(),
                metadata.data().len()
            ),
            BtEventKind::MetadataFailed(failure) => {
                println!("metadata fetch failed: {failure:?}")
            }
            BtEventKind::DhtError(error) => eprintln!("DHT error: {error:?}"),
            BtEventKind::ListenFailed(error) => eprintln!("listen failed: {error:?}"),
            BtEventKind::AlertsDropped(error) => eprintln!("alerts dropped: {error:?}"),
            _ => {}
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::task::LocalSet::new().run_until(run()).await;
}

async fn run() {
    let (command_tx, mut command_rx) = mpsc::unbounded_channel();

    let mut settings = SettingsPack::new();
    settings.set_bool(BoolSetting::EnableDht, true);
    settings.set_bool(BoolSetting::EnableIncomingTcp, true);
    settings.set_bool(BoolSetting::EnableIncomingUtp, true);
    // Keep DHT, status, and error alerts enabled. STATUS is needed for
    // add-torrent and metadata-received events.
    settings.set_int(
        IntSetting::AlertMask,
        AlertCategory::DHT_NOTIFICATION
            | AlertCategory::DHT_OPERATION_NOTIFICATION
            | AlertCategory::STATUS_NOTIFICATION
            | AlertCategory::ERROR_NOTIFICATION,
    );
    settings.set_string(StringSetting::ListenInterfaces, "0.0.0.0:6881,[::]:6881");
    settings.set_string(StringSetting::DhtBootstrapNodes, BOOTSTRAP_NODES);

    let engine = Engine::builder()
        .set_settings_pack(settings)
        .add_extension(Observer {
            commands: command_tx,
        })
        .build();
    let handle = engine.handle();
    let mut status = handle.subscribe_status();
    let runner = tokio::task::spawn_local(engine.run());

    status
        .wait_for(|value| *value == EngineStatus::Running)
        .await
        .expect("engine failed to start");

    println!("engine is running");
    println!(
        "session stats requested: {:?}",
        handle.post_session_stats().await
    );

    // The native runner owns the session; this loop executes commands emitted
    // by the extension on the dedicated event worker.
    let mut sampled_peers = HashSet::new();
    // Keep the example observable even when no new announce arrives.
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else { break };
                match command {
                    Command::FetchMetadata(dht_info_hash) => {
                        // DhtInfoHash is unmarked; v1 is used here only for the
                        // demonstration flow.
                        let v1 = InfoHashV1::from_bytes(*dht_info_hash.as_bytes());
                        let torrent_id = TorrentId::new(Some(v1), None);
                        println!("metadata fetch request: {:?}", handle.post_fetch_metadata(torrent_id).await);
                    }
                    Command::SampleInfohashes(peer) if sampled_peers.insert(peer) => {
                        // Avoid repeatedly querying the same peer from later
                        // live-node snapshots.
                        println!(
                            "sample request to {peer}: {:?}",
                            handle.post_dht_sample_infohashes(peer, DhtTarget::default()).await
                        );
                    }
                    Command::SampleInfohashes(_) => {}
                }
            }
            _ = interval.tick() => {
                println!("DHT stats request: {:?}", handle.post_dht_stats().await);
                println!("live-node request: {:?}", handle.post_dht_live_nodes().await);
            }
        }
    }

    if !runner.is_finished() {
        handle.shutdown().await.expect("engine shutdown failed");
    }
    runner
        .await
        .expect("engine runner task panicked")
        .expect("engine runner stopped unexpectedly");
}
