# Conductor (take 2) (Rust GraphQL Gateway)


## Getting Started

1. Clone
2. Run: `cargo run -- temp/config.yaml` to run it locally


## Usage

When no arguments are specified, it defaults the config path to `./config.json`
```
cargo run ./temp/config.yaml
# or
cargo run ./temp/config.json
```

## Architecture

This document will describe the architecture of the GraphQL gateway written in Rust.

### Project Structure

The GraphQL Gateway's source code is divided into four main modules:

1. `config`: Handles the loading and parsing of the Gateway configuration files.

2. `endpoint`: The endpoint module is responsible for managing endpoints, their runtime, and processing incoming GraphQL requests.

3. `gateway`: The gateway module contains the engine which manages the lifecycle of the gateway server.

4. `source`: This module includes the source service that processes the GraphQL requests to their respective sources.


### Modules

#### `config`

The `config` module parses the configuration for the gateway from a specified file. The configuration includes server settings, logger settings, source definitions, and endpoint definitions. It supports both `JSON` and `YAML` formats for the schema.

### `endpoint`

The endpoint module defines the `EndpointRuntime` struct which handles incoming requests for a specific endpoint. The `EndpointRuntime` contains the endpoint's configuration and a reference to the service that will process its requests (`upstream_service`). The `call` method is used to process incoming requests.


### `gateway`

The gateway module contains the main `Gateway` struct that manages the lifecycle of the gateway server. When the Gateway is initialized, it creates a map of `GraphQLSourceService` structs for each source defined in the configuration and a map of `EndpointRuntime` structs for each endpoint. These maps are then used to route incoming requests to the correct service.

### `source`

The source module contains the `SourceService` trait that defines the interface for services that process GraphQL requests, this is there so that later on other Sources than plain GraphQL like loading Supergraph from Hive/Apollo Studio.


### `graphql_source`

The `graphql_source` module includes the `GraphQLSourceService` which is an implementation of the `SourceService` trait for GraphQL sources.

### Server Startup Process

The startup process of the server is handled in `src/main.rs`. The steps are as follows:

1. Load the configuration file. `config.json` or `config.yaml`
2. Create a new `Gateway` instance using the loaded configuration.
3. Create a new `Router` and attach a route for each endpoint defined in the gateway's configuration.
4. Start the server and await incoming requests.

### Request Processing Flow

1. The router routes the request to the correct `EndpointRuntime`.
2. The endpoint's `call` method is invoked with the request `body` as a parameter.
3. The call function creates a new `SourceRequest` from the body, which includes the body fields for a valid graphql request (`query`, `variables`, and `operationName`).
4. The `SourceRequest` is sent to the endpoint's `upstream_service`.
5. The `upstream_service` processes the `SourceRequest` and returns a `SourceResponse`.
6. The `SourceResponse` is returned as the response to the client's request.

### Error Handling

Errors are primarily handled through the `SourceError` enum, which includes variants for different types of errors that can occur when processing a request.


## Tech Stack

- `tokio` for async runtime
- `hyper` for network
- `axum` as HTTP server
- `async-graphql` for GraphQL tooling
- `tower` to composing services and building flows
