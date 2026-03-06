use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rrmi::utils::{find_available_port_mine,find_available_port_os}; // adjust import

fn bench_ports(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_available_port");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    for total in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::new("os", total), &total, |b, &total| {
            b.iter(|| {
                let ports: Vec<_> = (0..total)
                    .map(|_| black_box(find_available_port_os().expect("should have available ports")))
                    .collect();
                black_box(ports)
            })
        });

        group.bench_with_input(BenchmarkId::new("mine", total), &total, |b, &total| {
            b.iter(|| {
                let ports: Vec<_> = (0..total)
                    .map( |_| black_box(find_available_port_mine().expect("should have available ports")))
                    .collect(); 
                black_box(ports)
            })
        });
    }

    group.finish();
}
criterion_group!(benches, bench_ports);
criterion_main!(benches);
