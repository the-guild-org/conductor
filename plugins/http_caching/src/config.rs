use conductor_common::vrl_utils::VrlConfigReference;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct HttpCachePluginConfig {
  #[serde(rename = "cache")]
  pub store_id: String,
  #[serde(default = "defualt_max_age")]
  pub max_age: u64,
  pub session_builder: Option<VrlConfigReference>,
}

fn defualt_max_age() -> u64 {
  60
}
