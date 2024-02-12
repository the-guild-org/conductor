use crate::{
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  serde_utils::LocalFileReference,
  vrl_functions::vrl_fns,
};
use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use vrl::{
  compiler::{state::RuntimeState, TargetValue},
  value,
};
use vrl::{
  compiler::{Context, TimeZone},
  value::KeyString,
};
use vrl::{
  compiler::{Program, Resolved},
  diagnostic::DiagnosticList,
  prelude::NotNan,
  value::Value,
};

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

#[derive(Debug)]
pub struct VrlProgramProxy {
  program: Program,
}

impl VrlProgramProxy {
  pub fn new(program: Program) -> Self {
    Self { program }
  }

  pub fn resolve_with_state(
    &self,
    write_value: Value,
    read_value: Value,
    state: &mut RuntimeState,
  ) -> Resolved {
    let mut target = TargetValue {
      metadata: read_value,
      value: write_value,
      secrets: Default::default(),
    };
    self
      .program
      .resolve(&mut Context::new(&mut target, state, &TimeZone::default()))
  }

  pub fn resolve(&self, write_value: Value, read_value: Value) -> Resolved {
    self.resolve_with_state(write_value, read_value, &mut RuntimeState::default())
  }
}

impl VrlConfigReference {
  pub fn contents(&self) -> &String {
    match self {
      VrlConfigReference::Inline { content } => content,
      VrlConfigReference::File { path } => &path.contents,
    }
  }

  pub fn program(&self) -> Result<VrlProgramProxy, DiagnosticList> {
    let contents = self.contents();

    match vrl::compiler::compile(contents, &vrl_fns()) {
      Err(err) => Err(err),
      Ok(result) => {
        if result.warnings.len() > 0 {
          tracing::warn!("vrl compiler warning: {:?}", result.warnings);
        }

        let r = VrlProgramProxy::new(result.program);

        Ok(r)
      }
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

pub fn conductor_response_to_value(res: &ConductorHttpResponse) -> Value {
  let body = res.body.clone();
  let status = res.status.as_u16();
  let mut headers_map: BTreeMap<KeyString, Value> = BTreeMap::new();

  for (k, v) in res.headers.iter() {
    headers_map.insert(k.to_string().into(), v.as_bytes().into());
  }

  let headers = Value::Object(headers_map);

  value!({
      body: body,
      status: status,
      headers: headers,
  })
}

pub fn conductor_graphql_request_to_value(gql_req: &GraphQLRequest) -> Result<Value> {
  let operation = gql_req.operation.as_bytes();
  let operation_name = gql_req.operation_name.as_ref().map(|v| v.as_bytes());
  let variables = match gql_req.variables.as_ref() {
    Some(v) => Some(serde_value_to_vrl_value(&serde_json::Value::Object(
      v.clone(),
    ))?),
    None => None,
  };

  let extensions = match gql_req.extensions.as_ref() {
    Some(v) => Some(serde_value_to_vrl_value(&serde_json::Value::Object(
      v.clone(),
    ))?),
    None => None,
  };

  Ok(value!({
      operation: operation,
      operation_name: operation_name,
      variables: variables,
      extensions: extensions,
  }))
}

pub fn conductor_request_to_value(req: &ConductorHttpRequest) -> Value {
  let body = req.body.clone();
  let uri = req.uri.as_bytes();
  let query_string = req.query_string.as_bytes();
  let method_str = req.method.to_string();
  let method = method_str.as_bytes();
  let mut headers_map: BTreeMap<KeyString, Value> = BTreeMap::new();

  for (k, v) in req.headers.iter() {
    headers_map.insert(k.to_string().into(), v.as_bytes().into());
  }

  let headers = Value::Object(headers_map);

  value!({
      body: body,
      uri: uri,
      query_string: query_string,
      method: method,
      headers: headers,
  })
}
