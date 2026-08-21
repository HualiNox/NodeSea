//! Simple observer example that polls and prints BitTorrent engine events.
use nodesea_bt::{BtEvent, BtEventKind, DhtTarget, Engine, EventSink};
use std::{collections::HashSet, net::SocketAddr};

/// Example event sink that observes DHT activity and live-node snapshots.
struct Observer {
    /// DHT endpoints discovered from live-node snapshots.
    nodes: HashSet<SocketAddr>,
    /// Endpoints sampled once by this example.
    sampled: HashSet<SocketAddr>,

    /// Whether the initial DHT bootstrap event has been observed.
    dht_bootstrapped: bool,
}

impl EventSink for Observer {
    fn on_event(&mut self, event: BtEvent) {
        match event.kind() {
            BtEventKind::DhtAnnounce(event) => {
                println!("dht announce: {}", event.info_hash());
            }

            BtEventKind::DhtGetPeers(event) => {
                println!("dht getpeers: {}", event.info_hash());
            }

            BtEventKind::DhtStats(event) => {
                println!(
                    "DHT stats: {} nodes, listening on {}:{}",
                    event.node_count(),
                    event.local_ip(),
                    event.local_port()
                );
            }

            BtEventKind::DhtBootstrap(_) => {
                println!("DHT bootstrap completed");
                self.dht_bootstrapped = true;
            }

            BtEventKind::DhtSampleInfohashes(event) => {
                println!(
                    "dht sample: node={}, interval={:?}, num_infohashes={}, samples={}, nodes={}",
                    event.node().endpoint(),
                    event.interval(),
                    event.num_infohashes(),
                    event.samples().len(),
                    event.nodes().len()
                );
            }
            BtEventKind::DhtPkt(event) => {
                self.nodes.insert(*event.endpoint());
                println!(
                    "dht pkt: endpoint={}, direction={:?}, packet_len={}",
                    event.endpoint(),
                    event.direction(),
                    event.packet().len()
                );
            }
            BtEventKind::DhtLiveNodes(event) => {
                println!(
                    "dht live nodes: local_node_id={}, nodes={}",
                    event.local_node_id(),
                    event.nodes().len()
                );
                for node in event.nodes() {
                    self.nodes.insert(*node.endpoint());
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Create the BitTorrent engine and start polling for DHT events
    let mut engine = Engine::new().expect("failed to initialize engine");
    let mut observer = Observer {
        dht_bootstrapped: false,
        sampled: HashSet::new(),
        nodes: HashSet::new(),
    };

    // Start polling loop
    loop {
        engine.post_dht_stats();

        if observer.dht_bootstrapped {
            engine.post_dht_live_nodes();
            if let Some(node) = observer
                .nodes
                .iter()
                .find(|node| !observer.sampled.contains(node))
                .copied()
            {
                println!("sampling from {node}");

                if engine.post_dht_sample_infohashes(&node, &DhtTarget::from_bytes([0; 20])) {
                    observer.sampled.insert(node);
                }
            }
        }

        engine.poll_events(&mut observer);

        // Wait before next poll
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
