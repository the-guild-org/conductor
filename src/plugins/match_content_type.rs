use crate::{graphql_utils::APPLICATION_GRAPHQL_JSON, http_utils::extract_accept};

use super::{core::Plugin, flow_context::FlowContext};
use axum::body::BoxBody;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use mime::APPLICATION_JSON;

pub struct MatchContentTypePlugin {}

impl Plugin for MatchContentTypePlugin {
    fn on_downstream_http_response(
        &self,
        ctx: &FlowContext,
        response: &mut http::Response<BoxBody>,
    ) {
        let headers = response.headers_mut();

        if headers.get(CONTENT_TYPE).is_none() {
            let accept_header =
                extract_accept(ctx.downstream_http_request.headers()).unwrap_or(APPLICATION_JSON);

            if accept_header == APPLICATION_JSON || accept_header == "*/*" {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            } else if accept_header == APPLICATION_GRAPHQL_JSON {
                headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static(APPLICATION_GRAPHQL_JSON),
                );
            }
        }
    }
}
