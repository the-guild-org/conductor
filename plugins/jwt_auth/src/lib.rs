mod config;
mod jwks_provider;
mod plugin;

#[cfg(test)]
mod test;

pub use crate::config::JwksProviderSourceConfig as JwksProvider;
pub use crate::config::JwtAuthPluginConfig as Config;
pub use crate::config::JwtAuthPluginLookupLocation as LookupLocation;
pub use crate::plugin::JwtAuthPlugin as Plugin;
pub use jsonwebtoken::{decode, encode, Algorithm, EncodingKey, Header as JwtHeader};
pub use serde_json::Value as ClaimsJsonObject;
