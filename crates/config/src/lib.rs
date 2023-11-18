pub mod plugins;
pub mod serde_utils;

use plugins::{CorsPluginConfig, HttpGetPluginConfig, PersistedOperationsPluginConfig};
use schemars::JsonSchema;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path};

#[derive(Deserialize, Debug, Clone, JsonSchema)]
/// The top-level configuration object for Conductor gateway.
pub struct ConductorConfig {
    #[serde(default)]
    /// Configuration for the HTTP server.
    pub server: ServerConfig,
    #[serde(default)]
    /// Conductor logger configuration.
    pub logger: LoggerConfig,
    /// List of sources to be used by the gateway. Each source is a GraphQL endpoint or multiple endpoints grouped using a federated implementation.
    pub sources: Vec<SourceDefinition>,
    /// List of GraphQL endpoints to be exposed by the gateway.
    /// Each endpoint is a GraphQL schema that is backed by one or more sources and can have a unique set of plugins applied to.
    pub endpoints: Vec<EndpointDefinition>,
    /// List of global plugins to be applied to all endpoints. Global plugins are applied before endpoint-specific plugins.
    pub plugins: Option<Vec<PluginDefinition>>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct EndpointDefinition {
    /// A valid HTTP path to listen on for this endpoint.
    /// This will be used for the main GraphQL endpoint as well as for the GraphiQL endpoint.
    /// In addition, plugins that extends the HTTP layer will use this path as a base path.
    pub path: String,
    /// The identifier of the source to be used. This must match the `id` field of a source definition.
    pub from: String,
    /// A list of unique plugins to be applied to this endpoint.
    pub plugins: Option<Vec<PluginDefinition>>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum PluginDefinition {
    #[serde(rename = "cors")]
    /// CORS plugin
    CorsPlugin {
        /// CORS configuration object. You may also specify an empty object ( {} ) to use the default permissive configuration.
        config: CorsPluginConfig,
    },

    #[serde(rename = "graphiql")]
    /// GraphiQL over HTTP GET plugin.
    GraphiQLPlugin,

    #[serde(rename = "http_get")]
    HttpGetPlugin {
        /// HTTP-GET GraphQL execution, based on GraphQL-Over-HTTP specification: https://graphql.github.io/graphql-over-http/draft/
        config: Option<HttpGetPluginConfig>,
    },

    #[serde(rename = "persisted_operations")]
    PersistedOperationsPlugin {
        /// Persisted Documents plugin for improved performance, reduced network traffic and hardened GraphQL layer.
        config: PersistedOperationsPluginConfig,
    },
}

#[derive(Deserialize, Default, Debug, Clone, Copy, JsonSchema)]
pub enum Level {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    #[default]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl Level {
    pub fn into_level(self) -> tracing::Level {
        match self {
            Level::Trace => tracing::Level::TRACE,
            Level::Debug => tracing::Level::DEBUG,
            Level::Info => tracing::Level::INFO,
            Level::Warn => tracing::Level::WARN,
            Level::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct LoggerConfig {
    #[serde(default)]
    /// Log level
    pub level: Level,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    /// The port to listen on, default to 9000
    pub port: u16,
    #[serde(default = "default_server_host")]
    /// The host to listen on, default to 127.0.0.1
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_server_port(),
            host: default_server_host(),
        }
    }
}

fn default_server_port() -> u16 {
    9000
}
fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
/// A source definition for a GraphQL endpoint or a federated GraphQL implementation.
pub enum SourceDefinition {
    #[serde(rename = "graphql")]
    /// A simple, single GraphQL endpoint
    GraphQL {
        /// The identifier of the source. This is used to reference the source in the `from` field of an endpoint definition.
        id: String,
        /// The configuration for the GraphQL source.
        config: GraphQLSourceConfig,
    },
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct GraphQLSourceConfig {
    /// The endpoint URL for the GraphQL source.
    pub endpoint: String,
}

#[tracing::instrument(level = "trace")]
pub async fn load_config(file_path: &String) -> ConductorConfig {
    let path = Path::new(file_path);
    let contents = read_to_string(file_path).expect("Failed to read config file");

    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("json") => serde_json::from_str::<ConductorConfig>(&contents)
                .expect("Failed to parse config file"),
            Some("yaml") | Some("yml") => serde_yaml::from_str::<ConductorConfig>(&contents)
                .expect("Failed to parse config file"),
            _ => panic!("Unsupported config file extension"),
        },
        None => panic!("Config file has no extension"),
    }
}

pub fn from_yaml(contents: &str) -> Result<ConductorConfig, serde_yaml::Error> {
    serde_yaml::from_str::<ConductorConfig>(contents)
}
