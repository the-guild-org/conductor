## Conductor e2e Benchmarks

This benchmark setup is using K6 to run concurrent users that query the main binary of Conductor. The purpose of this test is to measure the stats of Conductor's overhead on the hot path.

### Running locally

1. Run in background the gateway: `cargo run --bin conductor -- ./benchmark/gw.yaml` (in the root dir)
1. Run K6: `cd benchmark` and then `k6 run k6.js`

### CI

See [Benchmark CI pipeline](../.github/workflows/benchmark.yaml)
