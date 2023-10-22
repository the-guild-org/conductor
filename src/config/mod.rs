use schemars::JsonSchema;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path};

use crate::plugins::{
    cors::CorsPluginConfig, http_get_plugin::HttpGetPluginConfig,
    persisted_documents::config::PersistedOperationsPluginConfig,
};

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct ConductorConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub logger: LoggerConfig,
    pub sources: Vec<SourceDefinition>,
    pub endpoints: Vec<EndpointDefinition>,
    pub plugins: Option<Vec<PluginDefinition>>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct EndpointDefinition {
    pub path: String,
    pub from: String,
    pub plugins: Option<Vec<PluginDefinition>>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum PluginDefinition {
    #[serde(rename = "cors")]
    CorsPlugin { config: CorsPluginConfig },

    #[serde(rename = "graphiql")]
    GraphiQLPlugin,

    #[serde(rename = "http_get")]
    HttpGetPlugin { config: Option<HttpGetPluginConfig> },

    #[serde(rename = "persisted_operations")]
    PersistedOperationsPlugin {
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
    pub level: Level,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_server_host")]
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
pub enum SourceDefinition {
    #[serde(rename = "graphql")]
    GraphQL {
        id: String,
        config: GraphQLSourceConfig,
    },
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct GraphQLSourceConfig {
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
