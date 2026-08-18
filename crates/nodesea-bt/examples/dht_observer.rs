//! Simple observer example that polls and prints BitTorrent engine events.
use nodesea_bt::{BtEvent, Engine, EventSink, InfoHash};
use std::collections::HashSet;

struct Observer {
    pending: Vec<InfoHash>,
    seen: HashSet<InfoHash>,
}

impl EventSink for Observer {
    fn on_event(&mut self, event: BtEvent) {
        match event {
            BtEvent::DhtAnnounce { info_hash, .. } => {
                println!("dht announce: {info_hash}");
                if self.seen.insert(info_hash) {
                    self.pending.push(info_hash);
                }
            }

            BtEvent::DhtGetPeers { info_hash } => {
                println!("dht getpeers: {info_hash}");
                if self.seen.insert(info_hash) {
                    self.pending.push(info_hash);
                }
            }

            BtEvent::MetadataReceived { info_hash, data } => {
                println!("metadata: {info_hash}, {} bytes", data.len());
            }

            BtEvent::MetadataFailed { info_hash, message } => {
                println!("metadata failed: {info_hash}: {message}");
            }

            BtEvent::DhtStats {
                node_count,
                local_ip,
                local_port,
            } => {
                println!("DHT stats: {node_count} nodes, listening on {local_ip}:{local_port}");
            }

            BtEvent::DhtBootstrap => {
                println!("DHT bootstrap completed");
            }

            _ => {}
        }
    }
}

fn main() {
    // Create the BitTorrent engine and start polling for DHT events
    let mut engine = Engine::new().expect("failed to initialize engine");
    let mut observer = Observer {
        pending: Vec::new(),
        seen: HashSet::new(),
    };

    // Start polling loop
    loop {
        engine.post_dht_stats();
        engine.poll_events(&mut observer);

        for hash in observer.pending.drain(..) {
            if engine.fetch_metadata(&hash) {
                println!("fetching {hash}");
            }
        }

        // Wait before next poll
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
