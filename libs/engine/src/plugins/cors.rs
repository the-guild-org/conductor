use conductor_config::plugins::{CorsListStringConfig, CorsPluginConfig, CorsStringConfig};
use http::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE,
};

use super::core::Plugin;
use crate::request_execution_context::RequestExecutionContext;
use conductor_common::http::ConductorHttpResponse;

pub struct CorsPlugin(pub CorsPluginConfig);

#[async_trait::async_trait]
impl Plugin for CorsPlugin {
    fn on_downstream_http_response(
        &self,
        _ctx: &RequestExecutionContext,
        response: &mut ConductorHttpResponse,
    ) {
        // Apply CORS headers based on the plugin configuration
        let config = &self.0;
        if let Some(ref origin) = config.allowed_origin {
            let value = match origin {
                CorsStringConfig::Wildcard => "*".to_string(),
                CorsStringConfig::Value(ref v) => v.clone(),
            };
            response.headers.insert(
                ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_str(&value).unwrap(),
            );
        }

        if let Some(ref methods) = config.allowed_methods {
            let value = match methods {
                CorsListStringConfig::Wildcard => "*".to_string(),
                CorsListStringConfig::List(ref v) => v.join(", "),
            };
            response.headers.insert(
                ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_str(&value).unwrap(),
            );
        }

        if let Some(ref headers) = config.allowed_headers {
            let value = match headers {
                CorsListStringConfig::Wildcard => "*".to_string(),
                CorsListStringConfig::List(ref v) => v.join(", "),
            };
            response.headers.insert(
                ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_str(&value).unwrap(),
            );
        }

        if let Some(allow_credentials) = config.allow_credentials {
            response.headers.insert(
                ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_str(&allow_credentials.to_string()).unwrap(),
            );
        }

        if let Some(max_age) = config.max_age {
            response.headers.insert(
                ACCESS_CONTROL_MAX_AGE,
                HeaderValue::from_str(&max_age.to_string()).unwrap(),
            );
        }
    }
}
