## Smoke Tests

The purpose of these tests is to execute different flows that are connected to real data sources and other third-party dependencies.

Failure in one of these tests during a PR, might indicate that one of the crucial flows is broken.

## CI Setup

Check the `.github/workflows/ci.yaml` (job: `smoke-test`). We are running the same tests against WASM and binary builds.

## Running Locally

To run locally, following these instructions:

1. Make sure you have Docker engine installed and running.
2. Start the third-pary dependencies by running: `docker compose -f docker-compose.yaml up -d --remove-orphans --wait --force-recreate` inside the `libs/smoke_tests` directory.
3. Start a real GraphQL server by running: `cargo run` inside `tests/test-server` directory.

Now, run Conductor with one of the configurations:

- For binary runtime, use: `cargo run --bin conductor -- ./libs/smoke_tests/test_gw.yaml` in the root workspace dir.
- For WASM runtime, use: `pnpm start:smoke` inside `bin/cloudflare_worker`.

Run the smoke tests build using:

```
CONDUCTOR_URL="http://127.0.0.1:9000" cargo test --features binary -- --nocapture
```
