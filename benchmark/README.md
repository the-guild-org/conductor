## Conductor e2e Benchmarks

This benchmark setup is using K6 to run concurrent users that query the main binary of Conductor.

### Running locally

1. Run in background the mock server in release mode: `cd benchmark/server` and then `cargo run --release`
2. Run in background the gateway: `cargo run --bin conductor -- ./benchmark/gw.yaml` (in the root dir)
3. Run K6: `cd benchmark` and then `k6 run k6.js`

### CI

See [Benchmark CI pipeline](../.github/workflows/benchmark.yaml)
