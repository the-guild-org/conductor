use std::collections::BTreeMap;

use anyhow::{Ok, Result};
use conductor_common::graphql::GraphQLRequest;
use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse};
use conductor_common::vrl_utils::serde_value_to_vrl_value;
use vrl::value;
use vrl::value::{KeyString, Value};

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
