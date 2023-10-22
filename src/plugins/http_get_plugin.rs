use http::{Method, StatusCode};
use schemars::JsonSchema;

use crate::{
    graphql_utils::{GraphQLResponse, ParsedGraphQLRequest},
    http_utils::{extract_graphql_from_get_request, ExtractGraphQLOperationError},
};

use super::{core::Plugin, flow_context::FlowContext};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct HttpGetPluginConfig {
    mutations: Option<bool>,
}

impl Default for HttpGetPluginConfig {
    fn default() -> Self {
        Self { mutations: None }
    }
}

pub struct HttpGetPlugin(pub HttpGetPluginConfig);

#[async_trait::async_trait]
impl Plugin for HttpGetPlugin {
    async fn on_downstream_http_request(&self, ctx: &mut FlowContext) {
        if ctx.downstream_http_request.method() == axum::http::Method::GET {
            let (_, accept, result) = extract_graphql_from_get_request(ctx);

            match result {
                Ok(gql_request) => match ParsedGraphQLRequest::create_and_parse(gql_request) {
                    Ok(parsed) => {
                        ctx.downstream_graphql_request = Some(parsed);
                    }
                    Err(e) => {
                        ctx.short_circuit(e.into_response(accept));
                    }
                },
                Err(ExtractGraphQLOperationError::EmptyExtraction) => {
                    // nothing to do here, maybe other plugins (like GraphiQL will take care of this one)
                }
                Err(e) => {
                    ctx.short_circuit(e.into_response(accept));
                }
            }
        }
    }

    async fn on_downstream_graphql_request(&self, ctx: &mut FlowContext) {
        if ctx.downstream_http_request.method() == Method::GET
            && (self.0.mutations.is_none() || self.0.mutations == Some(false))
        {
            if let Some(gql_req) = &ctx.downstream_graphql_request {
                if gql_req.is_running_mutation() {
                    ctx.short_circuit(
                        GraphQLResponse::new_error("mutations are not allowed over GET")
                            .into_response(StatusCode::METHOD_NOT_ALLOWED),
                    );
                }
            }
        }
    }
}
