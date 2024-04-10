use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
pub struct HeaderManipulationPluginConfig {
  pub upstream: Vec<HeaderManipulationAction>,
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum HeaderManipulationAction {
  #[serde(rename = "passthrough")]
  Passthrough { name: String },
  #[serde(rename = "remove")]
  Remove { name: String },
  #[serde(rename = "add")]
  Add { name: String, value: String },
  #[serde(rename = "copy")]
  Copy { to: String, from: String },
}
