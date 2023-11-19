pub mod plugins;
pub mod serde_utils;

use plugins::{
    GraphiQLPluginConfig, HttpGetPluginConfig, PersistedOperationsPluginConfig,
    PersistedOperationsProtocolConfig,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_utils::{JsonSchemaExample, JsonSchemaExampleMetadata, LocalFileReference};
use std::{
    cell::RefCell,
    fs::read_to_string,
    path::{Path, PathBuf},
};

/// This section describes the top-level configuration object for Conductor gateway.
///
/// Conductor supports both YAML and JSON format for the configuration file.
///
/// ## Loading the config file
///
/// The configuration is loaded when server starts, based on the runtime environment you are using:
///
/// ### Binary
///
/// If you are running the Conductor binary directly, you can specify the configuration file path using the first argument:
///
/// ```
///
/// ./conductor my-config-file.json
///
/// ```
///
/// > By default, Conductor will look for a file named `config.json` in the current directory.
///
/// ### Docker
///
/// If you are using Docker environment, you can mount the configuration file into the container, and then point the Conductor binary to it:
///
/// ```
///
/// docker run -v my-config-file.json:/app/config.json the-guild-org/conductor-t2:latest /app/config.json
///
/// ```
///
/// ### CloudFlare Worker
///
/// WASM runtime doesn't allow filesystem access, so you need to load the configuration file into an environment variable named `CONDUCTOR_CONFIG`.
///
/// ## Autocomplete/validation in VSCode
///
/// For JSON files, you can specify the `$schema` property to enable autocomplete and validation in VSCode:
///
/// ```json filename="config.json"
///
/// {
///  "$schema": "https://raw.githubusercontent.com/the-guild-org/conductor-t2/master/libs/config/conductor.schema.json"
/// }
///
/// ```
///
/// For YAML auto-complete and validation, you can install the [YAML Language Support](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) extension and enable it by adding the following to your YAML file:
///
/// ```yaml filename="config.yaml"
///
/// $schema: "https://raw.githubusercontent.com/the-guild-org/conductor-t2/master/libs/config/conductor.schema.json"
///
/// ```
///
/// ### JSONSchema
///
/// As part of the release flow of Conductor, we are publishing the configuration schema as a JSONSchema file.
///
/// You can find [here the latest version of the schema](https://github.com/the-guild-org/conductor-t2/releases).
///
///
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct ConductorConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]

    /// Configuration for the HTTP server.
    pub server: Option<ServerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Conductor logger configuration.
    pub logger: Option<LoggerConfig>,
    /// List of sources to be used by the gateway. Each source is a GraphQL endpoint or multiple endpoints grouped using a federated implementation.
    ///
    /// For additional information, please refer to the [Sources section](./sources/graphql).
    pub sources: Vec<SourceDefinition>,
    /// List of GraphQL endpoints to be exposed by the gateway.
    /// Each endpoint is a GraphQL schema that is backed by one or more sources and can have a unique set of plugins applied to.
    ///
    /// For additional information, please refer to the [Endpoints section](./endpoints).
    pub endpoints: Vec<EndpointDefinition>,
    /// List of global plugins to be applied to all endpoints. Global plugins are applied before endpoint-specific plugins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<PluginDefinition>>,
}

/// The `Endpoint` object exposes a GraphQL source with set of plugins applied to it.
///
/// Each Endpoint can have its own set of plugins, which are applied after the global plugins. Endpoints can expose the same source with different plugins applied to it, to create different sets of features for different clients or consumers.
///
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "endpoint_definition_example1")]
#[schemars(example = "endpoint_definition_example2")]
pub struct EndpointDefinition {
    /// A valid HTTP path to listen on for this endpoint.
    /// This will be used for the main GraphQL endpoint as well as for the GraphiQL endpoint.
    /// In addition, plugins that extends the HTTP layer will use this path as a base path.
    pub path: String,
    /// The identifier of the `Source` to be used.
    ///
    /// This must match the `id` field of a `Source` definition.
    pub from: String,
    /// A list of unique plugins to be applied to this endpoint. These plugins will be applied after the global plugins.
    ///
    /// Order of plugins is important: plugins are applied in the order they are defined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<PluginDefinition>>,
}

fn endpoint_definition_example1() -> JsonSchemaExample<ConductorConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Basic Example", Some("This example demonstrate how to declare a GraphQL source, and expose it as a GraphQL endpoint. The endpoint also exposes a GraphiQL interface.")),
        example: ConductorConfig {
            server: None,
            logger: None,
            plugins: None,
            sources: vec![SourceDefinition::GraphQL {
                id: "my-source".to_string(),
                config: GraphQLSourceConfig {
                    endpoint: "https://my-source.com/graphql".to_string(),
                },
            }],
            endpoints: vec![EndpointDefinition {
                path: "/graphql".to_string(),
                from: "my-source".to_string(),
                plugins: Some(vec![PluginDefinition::GraphiQLPlugin { config: None }]),
            }],
        },
    }
}

fn endpoint_definition_example2() -> JsonSchemaExample<ConductorConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Multiple Endpoints", Some("This example shows how to expose a single GraphQL source with different plugins applied to it. In this example, we expose the same, one time with persised operations, and one time with HTTP GET for arbitrary queries.")),
        example: ConductorConfig {
            server: None,
            logger: None,
            plugins: None,
            sources: vec![SourceDefinition::GraphQL {
                id: "my-source".to_string(),
                config: GraphQLSourceConfig {
                    endpoint: "https://my-source.com/graphql".to_string(),
                },
            }],
            endpoints: vec![EndpointDefinition {
                path: "/persisted".to_string(),
                from: "my-source".to_string(),
                plugins: Some(vec![
                    PluginDefinition::PersistedOperationsPlugin {
                        config: PersistedOperationsPluginConfig {
                            allow_non_persisted: Some(false),
                            store: plugins::PersistedOperationsPluginStoreConfig::File { file: LocalFileReference { path: "store.json".to_string(), contents: "".to_string()}, format: plugins::PersistedDocumentsFileFormat::JsonKeyValue },
                            protocols: vec![
                                PersistedOperationsProtocolConfig::DocumentId { field_name: Default::default() },
                            ]
                        }
                    }
                ]),
            }, EndpointDefinition {
                path: "/data".to_string(),
                from: "my-source".to_string(),
                plugins: Some(vec![
                    PluginDefinition::HttpGetPlugin { config: Some(HttpGetPluginConfig {
                        mutations: Some(false)
                    }) }
                ]),
            }],
        },
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum PluginDefinition {
    #[serde(rename = "graphiql")]
    GraphiQLPlugin {
        config: Option<GraphiQLPluginConfig>,
    },

    #[serde(rename = "http_get")]
    HttpGetPlugin { config: Option<HttpGetPluginConfig> },

    #[serde(rename = "persisted_operations")]
    PersistedOperationsPlugin {
        config: PersistedOperationsPluginConfig,
    },
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, Copy, JsonSchema)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
pub struct LoggerConfig {
    #[serde(default)]
    /// Log level
    pub level: Level,
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
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
            port: Default::default(),
            host: Default::default(),
        }
    }
}

fn default_server_port() -> u16 {
    9000
}
fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
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

/// An upstream based on a simple, single GraphQL endpoint.
///
/// By using this source, you can easily wrap an existing GraphQL upstream server, and enrich it with features and plugins.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "graphql_source_definition_example")]
pub struct GraphQLSourceConfig {
    /// The HTTP(S) endpoint URL for the GraphQL source.
    pub endpoint: String,
}

fn graphql_source_definition_example() -> JsonSchemaExample<SourceDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Simple", None),
        example: SourceDefinition::GraphQL {
            id: "my-source".to_string(),
            config: GraphQLSourceConfig {
                endpoint: "https://my-source.com/graphql".to_string(),
            },
        },
    }
}

thread_local! {
    static BASE_PATH: RefCell<PathBuf> = RefCell::new(PathBuf::new());
}

#[tracing::instrument(level = "trace")]
pub async fn load_config(file_path: &String) -> ConductorConfig {
    let path = Path::new(file_path);
    let contents = read_to_string(file_path).expect("Failed to read config file");

    let base_path = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

    BASE_PATH.with(|bp| {
        *bp.borrow_mut() = base_path;
    });

    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("json") => parse_config_from_json(&contents).expect("Failed to parse config file"),
            Some("yaml") | Some("yml") => {
                parse_config_from_yaml(&contents).expect("Failed to parse config file")
            }
            _ => panic!("Unsupported config file extension"),
        },
        None => panic!("Config file has no extension"),
    }
}

pub fn parse_config_from_yaml(contents: &str) -> Result<ConductorConfig, serde_yaml::Error> {
    serde_yaml::from_str::<ConductorConfig>(contents)
}

pub fn parse_config_from_json(contents: &str) -> Result<ConductorConfig, serde_json::Error> {
    serde_json::from_str::<ConductorConfig>(contents)
}
