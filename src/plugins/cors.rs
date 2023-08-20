use std::time::Duration;

use http::{HeaderValue, Method};
use serde::{Deserialize, Deserializer, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

use super::core::Plugin;

pub struct CorsPlugin(pub CorsPluginConfig);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CorsListStringConfig {
    #[serde(deserialize_with = "deserialize_wildcard")]
    Wildcard,
    List(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CorsStringConfig {
    #[serde(deserialize_with = "deserialize_wildcard")]
    Wildcard,
    Value(String),
}

fn deserialize_wildcard<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "*")]
        Wildcard,
    }

    Helper::deserialize(deserializer).map(|_| ())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CorsPluginConfig {
    allow_credentials: Option<bool>,
    allowed_methods: Option<CorsListStringConfig>,
    allowed_origin: Option<CorsStringConfig>,
    allowed_headers: Option<CorsListStringConfig>,
    allow_private_network: Option<bool>,
    max_age: Option<Duration>,
}

impl CorsPluginConfig {
    pub fn is_empty_config(&self) -> bool {
        self.allow_credentials.is_none()
            && self.allowed_methods.is_none()
            && self.allowed_origin.is_none()
            && self.allowed_headers.is_none()
            && self.allow_private_network.is_none()
            && self.max_age.is_none()
    }
}

impl Plugin for CorsPlugin {
    fn on_endpoint_creation(&self, router: axum::Router<()>) -> axum::Router<()> {
        info!("CORS plugin registered, modifying route...");
        debug!("using object config for CORS plugin, config: {:?}", self.0);

        match self.0.is_empty_config() {
            true => {
                info!("CORS plugin configs are empty, using default config (permissive)...");
                let layer = CorsLayer::new()
                    .allow_credentials(false)
                    .allow_headers(Any)
                    .allow_methods(Any)
                    .allow_origin(Any)
                    .allow_private_network(false);

                debug!("CORS layer configuration: {:?}", layer);

                router.route_layer(layer)
            }
            false => {
                let mut layer = CorsLayer::new();
                if self.0.allow_credentials.is_some() {
                    layer = layer.allow_credentials(self.0.allow_credentials.unwrap());
                }

                if self.0.allow_private_network.is_some() {
                    layer = layer.allow_private_network(self.0.allow_private_network.unwrap());
                }

                if self.0.max_age.is_some() {
                    layer = layer.max_age(self.0.max_age.unwrap());
                }

                layer = match self.0.allowed_origin {
                    Some(CorsStringConfig::Value(ref v)) => layer.allow_origin(
                        v.parse::<HeaderValue>()
                            .expect("invalid origin passed to CORS plugin"),
                    ),
                    Some(CorsStringConfig::Wildcard) => layer.allow_origin(Any),
                    None => layer,
                };

                layer = match self.0.allowed_headers {
                    Some(CorsListStringConfig::List(ref v)) => layer.allow_headers(
                        v.iter()
                            .map(|v| {
                                v.parse::<http::header::HeaderName>()
                                    .expect("invalid header passed to CORS plugin")
                            })
                            .collect::<Vec<http::header::HeaderName>>(),
                    ),
                    Some(CorsListStringConfig::Wildcard) => layer.allow_headers(Any),
                    None => layer,
                };

                layer = match self.0.allowed_methods {
                    Some(CorsListStringConfig::List(ref v)) => layer.allow_methods(
                        v.iter()
                            .map(|v| {
                                v.parse::<Method>()
                                    .expect("invalid HTTP method specific for CORS plugin")
                            })
                            .collect::<Vec<Method>>(),
                    ),
                    Some(CorsListStringConfig::Wildcard) => layer.allow_methods(Any),
                    None => layer,
                };

                debug!("CORS layer configuration: {:?}", layer);

                router.route_layer(layer)
            }
        }
    }
}
