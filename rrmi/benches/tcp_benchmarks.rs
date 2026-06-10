use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[cfg(feature = "bench_tcp")]
use rrmi::{
    _send_data_combined, _send_data_ioslice, _send_data_separate, _send_data_separate_flush,
    receive_data,
};
use std::hint::black_box; // adjust import
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn spawn_receiver() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        for conn in listener.incoming() {
            let mut stream = conn.unwrap();

            thread::spawn(move || {
                loop {
                    let mut len = [0u8; 4];
                    if stream.read_exact(&mut len).is_err() {
                        break;
                    }

                    let size = u32::from_be_bytes(len) as usize;
                    let mut buf = vec![0u8; size];

                    if stream.read_exact(&mut buf).is_err() {
                        break;
                    }
                }
            });
        }
    });

    addr
}

fn bench_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_send_variants");

    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(1000));

    let addr = spawn_receiver();

    let mut stream_v1 = TcpStream::connect(addr).unwrap();
    let mut stream_v2 = TcpStream::connect(addr).unwrap();
    let mut stream_v3 = TcpStream::connect(addr).unwrap();
    let mut stream_v4 = TcpStream::connect(addr).unwrap();

    for size in [4, 1024, 9000025] {
        let data = vec![0u8; size];
        group.bench_function(BenchmarkId::new("separate", size), |b| {
            b.iter(|| {
                let _ = _send_data_separate(black_box(data.clone()), &mut stream_v1);
            })
        });

        group.bench_function(BenchmarkId::new("separate_flush", size), |b| {
            b.iter(|| {
                let _ = _send_data_separate_flush(black_box(data.clone()), &mut stream_v2);
            })
        });

        group.bench_function(BenchmarkId::new("combined", size), |b| {
            b.iter(|| {
                let _ = _send_data_combined(black_box(data.clone()), &mut stream_v3);
            })
        });

        group.bench_function(BenchmarkId::new("ioslice", size), |b| {
            b.iter(|| {
                let _ = _send_data_ioslice(black_box(data.clone()), &mut stream_v4);
            })
        });
    }
    group.finish();
}

fn spawn_sender(size: usize) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        for conn in listener.incoming() {
            let mut stream = conn.unwrap();

            let payload = vec![0u8; size];
            thread::spawn(move || {
                loop {
                    _ = _send_data_ioslice(payload.clone(), &mut stream);
                }
            });
        }
    });

    addr
}

fn bench_recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_send_variants");

    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_millis(1000));

    let size = 128;
    let addr = spawn_sender(size);

    let mut stream_v1 = TcpStream::connect(addr).unwrap();

    group.bench_function(BenchmarkId::new("stream", size), |b| {
        b.iter(|| {
            let _ = receive_data(&mut stream_v1);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_recv, bench_send);
criterion_main!(benches);
