Run locally
```bash
cargo run --release --package example -- --local
```

Run DAS6

```bash
cargo run --release --package example
```

Run Liacs labs

```bash
cargo run --release --package example -- --liacs
```

Benchmarks
```bash
cargo bench --features bench
cargo bench --features bench_tcp
```

Tracing
```bash
cargo run --features tracing -- --local
```
