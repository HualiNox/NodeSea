use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

use nodesea_bt::{BtEvent, EventSink};

#[path = "support/ffi_bridge.rs"]
mod ffi_bridge;
use ffi_bridge::{FfiBenchSink, bench_dht_announce_batch, bench_dht_get_peers_batch};

/// FFI dispatch benchmarks
const BATCHES: &[usize] = &[1, 64, 1_024, 16_384, 65_536];

struct BlackBoxSink;

impl EventSink for BlackBoxSink {
    fn on_event(&mut self, event: BtEvent) {
        black_box(event);
    }
}

/// Benchmarks FFI dispatch performance.
fn bench_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_dispatch");

    // Configure sampling and timing parameters
    group.sample_size(50);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));

    // Benchmark DHT get peers and announce operations
    for &batch in BATCHES {
        // Set throughput based on batch size
        group.throughput(Throughput::Elements(batch as u64));

        // Benchmark DHT get peers
        group.bench_with_input(
            BenchmarkId::new("dht_get_peers", batch),
            &batch,
            |b, &batch| {
                let mut sink = BlackBoxSink;
                let mut ffi_sink = FfiBenchSink::new(&mut sink);

                b.iter(|| {
                    black_box(bench_dht_get_peers_batch(&mut ffi_sink, black_box(batch)));
                });
            },
        );

        // Benchmark DHT announce
        group.bench_with_input(
            BenchmarkId::new("dht_announce", batch),
            &batch,
            |b, &batch| {
                let mut sink = BlackBoxSink;
                let mut ffi_sink = FfiBenchSink::new(&mut sink);

                b.iter(|| {
                    black_box(bench_dht_announce_batch(&mut ffi_sink, black_box(batch)));
                });
            },
        );
    }

    group.finish();
}

// Criterion benchmark groups
criterion_group!(benches, bench_dispatch);
criterion_main!(benches);
