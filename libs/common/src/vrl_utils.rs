use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use vrl::{prelude::NotNan, value::Value};

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

pub fn vrl_value_to_serde_value(value: &Value) -> Result<serde_json::Value> {
  Ok(match value {
    Value::Null => serde_json::Value::Null,
    Value::Object(v) => serde_json::Value::Object(
      v.iter()
        .map(|(k, v)| Ok((k.to_owned().into(), vrl_value_to_serde_value(v)?)))
        .collect::<Result<_>>()?,
    ),
    Value::Boolean(v) => serde_json::Value::Bool(*v),
    Value::Float(f) => serde_json::Number::from_f64(f.into_inner())
      .map(serde_json::Value::Number)
      .ok_or_else(|| anyhow!("Failed to convert float to serde_json::Number"))?,
    Value::Integer(v) => serde_json::Value::Number(
      serde_json::Number::from_str(&v.to_string())
        .map_err(|e| anyhow!("Failed to convert integer to serde_json::Number: {}", e))?,
    ),
    Value::Bytes(v) => serde_json::Value::String(
      String::from_utf8(v.to_vec())
        .map_err(|e| anyhow!("Failed to convert bytes to string: {}", e))?,
    ),
    Value::Regex(v) => serde_json::Value::String(v.to_string()),
    Value::Timestamp(v) => serde_json::Value::Number(
      serde_json::Number::from_str(&v.timestamp_millis().to_string())
        .map_err(|e| anyhow!("Failed to convert timestamp to serde_json::Number: {}", e))?,
    ),
    Value::Array(v) => serde_json::Value::Array(
      v.iter()
        .map(vrl_value_to_serde_value)
        .collect::<Result<_>>()?,
    ),
  })
}

pub fn serde_value_to_vrl_value(value: &serde_json::Value) -> Result<Value> {
  Ok(match value {
    serde_json::Value::Null => Value::Null,
    serde_json::Value::Object(v) => Value::Object(
      v.iter()
        .map(|(k, v)| Ok((k.to_owned().into(), serde_value_to_vrl_value(v)?)))
        .collect::<Result<_>>()?,
    ),
    serde_json::Value::Bool(v) => Value::Boolean(*v),
    serde_json::Value::Number(v) if v.is_f64() => {
      let float_value = v
        .as_f64()
        .ok_or_else(|| anyhow!("Failed to convert serde_json::Number to f64"))?;
      let not_nan_float = NotNan::new(float_value)
        .map_err(|e| anyhow!("Failed to convert f64 to NotNan<f64>: {}", e))?;
      Value::Float(not_nan_float)
    }
    serde_json::Value::Number(v) => Value::Integer(v.as_i64().unwrap_or(i64::MAX)),
    serde_json::Value::String(v) => Value::Bytes(v.to_owned().into()),
    serde_json::Value::Array(v) => Value::Array(
      v.iter()
        .map(serde_value_to_vrl_value)
        .collect::<Result<_>>()?,
    ),
  })
}
