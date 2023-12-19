use std::collections::BTreeMap;
use std::str::FromStr;

use conductor_common::graphql::GraphQLRequest;
use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse};
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

pub fn conductor_graphql_request_to_value(gql_req: &GraphQLRequest) -> Value {
    let operation = gql_req.operation.as_bytes();
    let operation_name = gql_req.operation_name.as_ref().map(|v| v.as_bytes());
    let variables = gql_req
        .variables
        .as_ref()
        .map(|v| serde_value_to_vrl_value(&serde_json::Value::Object(v.clone())));
    let extensions = gql_req
        .extensions
        .as_ref()
        .map(|v| serde_value_to_vrl_value(&serde_json::Value::Object(v.clone())));

    value!({
        operation: operation,
        operation_name: operation_name,
        variables: variables,
        extensions: extensions,
    })
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
