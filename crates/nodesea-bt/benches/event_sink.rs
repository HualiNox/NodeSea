use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use nodesea_bt::{BtEvent, EventCollector, EventSink, InfoHash};

const ANNOUNCE_BATCHES: &[usize] = &[1, 64, 1_024, 16_384];
const METADATA_SIZES: &[usize] = &[256, 1_024, 16_384];
const METADATA_BATCH: usize = 64;

// Use a fixed, repeatable workload so results from different runs can be
// compared while still giving Criterion enough samples to expose variance.
fn configure_group<M: criterion::measurement::Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, M>,
) {
    group.sample_size(100);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
}

fn bench_dht_announce(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_sink/dht_announce");
    configure_group(&mut group);
    let info_hash = InfoHash::from_bytes([0xab; 20]);

    for &batch_size in ANNOUNCE_BATCHES {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut collector = EventCollector::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        collector.on_event(BtEvent::DhtAnnounce {
                            info_hash,
                            peer_ip: String::from("192.168.1.100"),
                            peer_port: 6881,
                        });
                    }
                    black_box(collector.take_events());
                });
            },
        );
    }
    group.finish();
}

fn bench_metadata_received(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_sink/metadata_received");
    configure_group(&mut group);
    let info_hash = InfoHash::from_bytes([0xab; 20]);

    for &payload_size in METADATA_SIZES {
        let total_bytes = payload_size * METADATA_BATCH;
        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{payload_size}B x {METADATA_BATCH}")),
            &payload_size,
            |b, &payload_size| {
                b.iter(|| {
                    let mut collector = EventCollector::with_capacity(METADATA_BATCH);
                    for _ in 0..METADATA_BATCH {
                        collector.on_event(BtEvent::MetadataReceived {
                            info_hash,
                            data: vec![0x42; payload_size],
                        });
                    }
                    black_box(collector.take_events());
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_dht_announce, bench_metadata_received,);
criterion_main!(benches);
