use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use vrl::value::Value;

use crate::serde_utils::LocalFileReference;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "from")]
pub enum VrlConfigReference {
  #[serde(rename = "inline")]
  #[schemars(title = "inline")]
  /// Inline string for a VRL code snippet. The string is parsed and executed as a VRL plugin.
  Inline { content: String },
  #[serde(rename = "file")]
  #[schemars(title = "file")]
  /// File reference to a VRL file. The file is loaded and executed as a VRL plugin.
  File { path: LocalFileReference },
}

impl VrlConfigReference {
  pub fn contents(&self) -> &String {
    match self {
      VrlConfigReference::Inline { content } => content,
      VrlConfigReference::File { path } => &path.contents,
    }
  }
}

pub fn vrl_value_to_serde_value(value: &Value) -> serde_json::Value {
  match value {
    Value::Null => serde_json::Value::Null,
    Value::Object(v) => v
      .iter()
      .map(|(k, v)| (k.to_owned().into(), vrl_value_to_serde_value(v)))
      .collect::<serde_json::Map<_, _>>()
      .into(),
    Value::Boolean(v) => v.to_owned().into(),
    Value::Float(f) => {
      serde_json::Value::Number(serde_json::Number::from_f64(f.into_inner()).unwrap())
    }
    Value::Integer(v) => {
      serde_json::Value::Number(serde_json::Number::from_str(&v.to_string()).unwrap())
    }
    Value::Bytes(v) => serde_json::Value::String(String::from_utf8(v.to_vec()).unwrap()),
    Value::Regex(v) => serde_json::Value::String(v.to_string()),
    Value::Timestamp(v) => serde_json::Value::Number(
      serde_json::Number::from_str(&v.timestamp_millis().to_string()).unwrap(),
    ),
    Value::Array(v) => v
      .iter()
      .map(vrl_value_to_serde_value)
      .collect::<Vec<_>>()
      .into(),
  }
}

pub fn serde_value_to_vrl_value(value: &serde_json::Value) -> Value {
  match value {
    serde_json::Value::Null => Value::Null,
    serde_json::Value::Object(v) => v
      .iter()
      .map(|(k, v)| (k.to_owned().into(), serde_value_to_vrl_value(v)))
      .collect::<BTreeMap<_, _>>()
      .into(),
    serde_json::Value::Bool(v) => v.to_owned().into(),
    serde_json::Value::Number(v) if v.is_f64() => Value::from_f64_or_zero(v.as_f64().unwrap()),
    serde_json::Value::Number(v) => v.as_i64().unwrap_or(i64::MAX).into(),
    serde_json::Value::String(v) => v.to_owned().into(),
    serde_json::Value::Array(v) => v
      .iter()
      .map(serde_value_to_vrl_value)
      .collect::<Vec<_>>()
      .into(),
  }
}
