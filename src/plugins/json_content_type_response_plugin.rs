use crate::endpoint::endpoint_runtime::EndpointError;

use super::core::Plugin;
use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    Body,
};

pub struct JSONContentTypePlugin {}

impl Plugin for JSONContentTypePlugin {
    fn on_upstream_graphql_response(&self, req: &mut Result<hyper::Response<Body>, EndpointError>) {
        if let Ok(res) = req {
            let headers = res.headers_mut();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
    }
}
