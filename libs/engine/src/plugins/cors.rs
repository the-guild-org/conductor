use conductor_common::http::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS, CONTENT_LENGTH, ORIGIN, VARY,
};
use conductor_common::http::{HttpHeadersMap, Method};
use conductor_config::plugins::CorsPluginConfig;

use super::core::Plugin;
use crate::request_execution_context::RequestExecutionContext;
use conductor_common::http::{ConductorHttpResponse, StatusCode};

pub struct CorsPlugin(pub CorsPluginConfig);

static WILDCARD: &str = "*";
static ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK: &str = "Access-Control-Allow-Private-Network";

impl CorsPlugin {
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin#browser_compatibility
    pub fn configure_origin(
        &self,
        request_headers: &HttpHeadersMap,
        response_headers: &mut HttpHeadersMap,
    ) {
        if let Some(origin) = &self.0.allowed_origin {
            let value = match origin.as_str() {
                "*" => WILDCARD,
                "reflect" => request_headers
                    .get(ORIGIN)
                    .map(|v| v.to_str().unwrap())
                    .unwrap_or(WILDCARD),
                v => v,
            };

            response_headers.append(ACCESS_CONTROL_ALLOW_ORIGIN, value.parse().unwrap());
            response_headers.append(VARY, "Origin".parse().unwrap());
        }
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials
    pub fn configure_credentials(&self, response_headers: &mut HttpHeadersMap) {
        if let Some(credentials) = &self.0.allow_credentials {
            if *credentials {
                response_headers.append(ACCESS_CONTROL_ALLOW_CREDENTIALS, "true".parse().unwrap());
            }
        }
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods
    pub fn configure_methods(&self, response_headers: &mut HttpHeadersMap) {
        let value = match self.0.allowed_methods.as_deref() {
            None | Some("*") => WILDCARD,
            Some(v) => v,
        };

        response_headers.append(ACCESS_CONTROL_ALLOW_METHODS, value.parse().unwrap());
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers
    pub fn configure_allowed_headers(
        &self,
        request_headers: &HttpHeadersMap,
        response_headers: &mut HttpHeadersMap,
    ) {
        match self.0.allowed_headers.as_deref() {
            // We are not going to use "*" because Safari does not support it, so let's just reflect the request headers
            // see https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers#browser_compatibility
            None | Some("*") => {
                if let Some(source_header) = request_headers.get(ACCESS_CONTROL_REQUEST_HEADERS) {
                    response_headers.append(ACCESS_CONTROL_ALLOW_HEADERS, source_header.clone());
                    response_headers.append(
                        VARY,
                        ACCESS_CONTROL_REQUEST_HEADERS.to_string().parse().unwrap(),
                    );
                }
            }
            Some(list) => {
                response_headers.append(ACCESS_CONTROL_ALLOW_HEADERS, list.parse().unwrap());
            }
        }
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers
    pub fn configure_exposed_headers(&self, response_headers: &mut HttpHeadersMap) {
        if let Some(exposed_headers) = &self.0.exposed_headers {
            response_headers.insert(
                ACCESS_CONTROL_EXPOSE_HEADERS,
                HeaderValue::from_str(exposed_headers).unwrap(),
            );
        }
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age
    pub fn configure_max_age(&self, response_headers: &mut HttpHeadersMap) {
        if let Some(max_age) = &self.0.max_age {
            response_headers.insert(ACCESS_CONTROL_MAX_AGE, max_age.to_string().parse().unwrap());
        }
    }

    pub fn configred_allow_private_netowkr(&self, response_headers: &mut HttpHeadersMap) {
        if let Some(allow_private_network) = &self.0.allow_private_network {
            if *allow_private_network {
                response_headers.insert(
                    ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK,
                    "true".parse().unwrap(),
                );
            }
        }
    }
}

#[async_trait::async_trait]
impl Plugin for CorsPlugin {
    async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
        if ctx.downstream_http_request.method == Method::OPTIONS {
            let request_headers = &ctx.downstream_http_request.headers;
            let mut response_headers = HttpHeadersMap::new();
            self.configure_origin(request_headers, &mut response_headers);
            self.configure_credentials(&mut response_headers);
            self.configure_methods(&mut response_headers);
            self.configure_exposed_headers(&mut response_headers);
            self.configure_max_age(&mut response_headers);
            self.configure_allowed_headers(request_headers, &mut response_headers);
            response_headers.insert(CONTENT_LENGTH, "0".parse().unwrap());

            ctx.short_circuit(ConductorHttpResponse {
                status: StatusCode::OK,
                headers: response_headers,
                body: Default::default(),
            })
        }
    }

    fn on_downstream_http_response(
        &self,
        ctx: &mut RequestExecutionContext,
        response: &mut ConductorHttpResponse,
    ) {
        let request_headers = &ctx.downstream_http_request.headers;
        self.configure_origin(request_headers, &mut response.headers);
        self.configure_credentials(&mut response.headers);
        self.configure_exposed_headers(&mut response.headers);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        endpoint_runtime::EndpointRuntime, gateway::ConductorGateway,
        source::graphql_source::GraphQLSourceRuntime,
    };

    use super::*;
    use conductor_common::http::ConductorHttpRequest;
    use conductor_config::GraphQLSourceConfig;
    use http::header::ORIGIN;
    use httpmock::{Method::POST, MockServer};
    use serde_json::json;
    use tokio::test;

    async fn prepare(
        config: Option<CorsPluginConfig>,
        method: Method,
        headers: Option<HttpHeadersMap>,
    ) -> ConductorHttpResponse {
        let plugin = CorsPlugin(config.unwrap_or_default());
        let request = match method {
            Method::OPTIONS => ConductorHttpRequest {
                body: Default::default(),
                uri: String::from("/test"),
                query_string: String::from(""),
                method,
                headers: headers.unwrap_or_default(),
            },
            Method::POST => ConductorHttpRequest {
                body: "{\"query\": \"query { __typename }\"}".into(),
                uri: String::from("/test"),
                query_string: String::from(""),
                method,
                headers: headers.unwrap_or_default(),
            },
            _ => unimplemented!(),
        };
        let http_mock = MockServer::start();
        http_mock.mock(|when, then| {
            when.method(POST).path("/graphql");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    &json!({
                        "data": {
                            "__typename": "Query"
                        },
                        "errors": null
                    })
                    .to_string(),
                );
        });

        let source = GraphQLSourceRuntime::new(GraphQLSourceConfig {
            endpoint: http_mock.url("/graphql"),
        });

        ConductorGateway::execute_test(
            EndpointRuntime::dummy(),
            Arc::new(source),
            vec![Box::new(plugin)],
            request,
        )
        .await
    }

    #[test]
    async fn options_zero_content_length() {
        let response = prepare(None, Method::OPTIONS, None).await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(CONTENT_LENGTH),
            Some(&"0".parse().unwrap())
        );
    }

    #[test]
    async fn default_methods() {
        let response = prepare(None, Method::OPTIONS, None).await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".parse().unwrap())
        );
    }

    #[test]
    async fn override_methods() {
        let response = prepare(
            Some(CorsPluginConfig {
                allowed_methods: Some("GET, POST".into()),
                ..Default::default()
            }),
            Method::OPTIONS,
            None,
        )
        .await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"GET, POST".parse().unwrap())
        );
    }

    #[test]
    async fn post_default_options_allow_all_origins() {
        let response = prepare(None, Method::POST, None).await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"*".parse().unwrap())
        );
    }

    #[test]
    async fn options_default_options_allow_all_origins() {
        let response = prepare(None, Method::OPTIONS, None).await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"*".parse().unwrap())
        );
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".parse().unwrap())
        );
    }

    #[test]
    async fn wildcard_config_reflects_origin() {
        let mut req_headers = HttpHeadersMap::new();
        req_headers.insert(
            ACCESS_CONTROL_REQUEST_HEADERS,
            "x-header-1, x-header-2".parse().unwrap(),
        );
        let response = prepare(
            Some(CorsPluginConfig {
                allowed_origin: Some("*".to_string()),
                ..Default::default()
            }),
            Method::OPTIONS,
            Some(req_headers),
        )
        .await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"*".parse().unwrap())
        );
        assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".parse().unwrap())
        );
    }

    #[test]
    async fn override_origin() {
        let response = prepare(
            Some(CorsPluginConfig {
                allowed_origin: Some("http://my-server.com".to_string()),
                ..Default::default()
            }),
            Method::OPTIONS,
            None,
        )
        .await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"http://my-server.com".parse().unwrap())
        );
        assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".parse().unwrap())
        );
    }

    #[test]
    async fn reflects_origin() {
        let mut req_headers = HttpHeadersMap::new();
        req_headers.insert(ORIGIN, "http://my-server.com".parse().unwrap());
        let response = prepare(
            Some(CorsPluginConfig {
                allowed_origin: Some("reflect".to_string()),
                ..Default::default()
            }),
            Method::OPTIONS,
            Some(req_headers),
        )
        .await;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"http://my-server.com".parse().unwrap())
        );
        assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
        assert_eq!(
            response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".parse().unwrap())
        );
    }
}
