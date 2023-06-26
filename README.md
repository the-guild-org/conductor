## Conductor (take 2)


### Getting Started

1. Clone
2. Run: `cargo run -- temp/config.yaml` to run it locally

Stack:

- `tokio` for async runtime
- `hyper` for network
- `axum` as HTTP server
- `async-graphql` for GraphQL tooling
- `tower` to composing services and building flows