use conductor_common::{
    graphql::APPLICATION_GRAPHQL_JSON,
    http::{
        extract_accept, extract_content_type, Method, Mime, APPLICATION_JSON,
        APPLICATION_WWW_FORM_URLENCODED,
    },
};
use conductor_config::plugins::GraphiQLPluginConfig;

use crate::request_execution_context::RequestExecutionContext;

use super::core::Plugin;

pub struct GraphiQLPlugin(pub GraphiQLPluginConfig);

#[async_trait::async_trait]
impl Plugin for GraphiQLPlugin {
    async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
        if ctx.downstream_http_request.method == Method::GET {
            let headers = &ctx.downstream_http_request.headers;
            let content_type = extract_content_type(headers);

            if content_type.is_none() || content_type != Some(APPLICATION_WWW_FORM_URLENCODED) {
                let accept: Option<Mime> = extract_accept(headers);

                if accept != Some(APPLICATION_JSON)
                    && accept != Some(APPLICATION_GRAPHQL_JSON.parse::<Mime>().unwrap())
                {
                    ctx.short_circuit(ctx.endpoint.render_graphiql(&self.0));
                }
            }
        }
    }
}
