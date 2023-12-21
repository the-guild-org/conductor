use std::str::FromStr;

use conductor_common::{
    graphql::GraphQLResponse,
    http::{ConductorHttpRequest, HeaderName, HeaderValue, Method, StatusCode},
};
use tracing::error;
use vrl::{
    compiler::{Context, Program, TargetValue, TimeZone},
    value,
    value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

use super::{utils::conductor_request_to_value, vrl_functions::ShortCircuitFn};

static METADATA_UPSTREAM_HTTP_REQ: &str = "upstream_http_req";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS: &str = "upstream_http_req.headers";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD: &str = "upstream_http_req.method";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_URI: &str = "upstream_http_req.uri";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING: &str = "upstream_http_req.query_string";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_BODY: &str = "upstream_http_req.body";

pub fn vrl_upstream_http_request(
    program: &Program,
    ctx: &mut RequestExecutionContext,
    req: &mut ConductorHttpRequest,
) {
    let upstream_req_value = conductor_request_to_value(req);
    let mut target = TargetValue {
        value: value!({}),
        metadata: value!({}),
        secrets: Secrets::default(),
    };

    target.value.insert(
        TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS,
        Value::Object(Default::default()),
    );
    target
        .value
        .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD, Value::Null);
    target
        .value
        .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_URI, Value::Null);
    target
        .value
        .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING, Value::Null);
    target
        .metadata
        .insert(METADATA_UPSTREAM_HTTP_REQ, upstream_req_value);

    match program.resolve(&mut Context::new(
        &mut target,
        ctx.vrl_shared_state(),
        &TimeZone::default(),
    )) {
        Ok(ret) => {
            if let Some((error_code, message)) = ShortCircuitFn::check_short_circuit(&ret) {
                ctx.short_circuit(
                    GraphQLResponse::new_error(&String::from_utf8(message.to_vec()).unwrap())
                        .into_with_status_code(StatusCode::from_u16(error_code as u16).unwrap()),
                );

                return;
            }

            if let Some(Value::Object(headers)) = target
                .value
                .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS, false)
            {
                for (k, v) in headers {
                    match v {
                        Value::Bytes(b) => {
                            req.headers.insert(
                                HeaderName::from_str(&k).unwrap(),
                                HeaderValue::from_bytes(&b).unwrap(),
                            );
                        }
                        Value::Null => {
                            req.headers.remove(HeaderName::from_str(&k).unwrap());
                        }
                        _ => {}
                    }
                }
            }

            if let Some(Value::Bytes(method)) = target
                .value
                .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD, false)
            {
                req.method = Method::from_bytes(&method).unwrap();
            }

            if let Some(Value::Bytes(uri)) = target
                .value
                .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_URI, false)
            {
                req.uri = String::from_utf8(uri.into()).unwrap()
            }

            if let Some(Value::Bytes(query_string)) = target
                .value
                .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING, false)
            {
                req.query_string = String::from_utf8(query_string.into()).unwrap()
            }

            if let Some(Value::Bytes(body)) = target
                .value
                .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_BODY, false)
            {
                req.body = body;
            }
        }
        Err(err) => {
            error!("vrl::upstream_http_request resolve error: {:?}", err);

            ctx.short_circuit(
                GraphQLResponse::new_error("vrl runtime error")
                    .into_with_status_code(StatusCode::BAD_GATEWAY),
            );
        }
    }
}
