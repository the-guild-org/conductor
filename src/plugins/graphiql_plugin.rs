use axum::response::{self};
use mime::{Mime, APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};

use crate::{
    graphql_utils::APPLICATION_GRAPHQL_JSON,
    http_utils::{extract_accept, extract_content_type},
};

use super::{core::Plugin, flow_context::FlowContext};

pub struct GraphiQLPlugin {}

impl Plugin for GraphiQLPlugin {
    fn on_downstream_http_request(&self, ctx: &mut FlowContext) {
        if ctx.downstream_http_request.method() == axum::http::Method::GET {
            let headers = ctx.downstream_http_request.headers();
            let content_type = extract_content_type(headers);

            if content_type.is_none() || content_type != Some(APPLICATION_WWW_FORM_URLENCODED) {
                let accept: Option<mime::Mime> = extract_accept(headers);

                if accept != Some(APPLICATION_JSON)
                    && accept != Some(APPLICATION_GRAPHQL_JSON.parse::<Mime>().unwrap())
                {
                    ctx.short_circuit(response::Html(ctx.endpoint.compose_graphiql().finish()));
                }
            }
        }
    }
}

#[tokio::test]
async fn graphiql_plugin_render_cases() {
    use crate::config::{EndpointDefinition, PluginDefinition};
    use http::{header::CONTENT_TYPE, StatusCode};

    let server = crate::test::utils::ConductorTest::empty()
        .mocked_source()
        .endpoint(EndpointDefinition {
            from: "s".to_string(),
            path: "/graphql".to_string(),
            plugins: Some(vec![PluginDefinition::GraphiQLPlugin]),
        })
        .finish();

    // try GET
    let response = server.get("/graphql").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(
        response
            .header(CONTENT_TYPE)
            .to_str()
            .expect("content type is missing"),
        "text/html; charset=utf-8"
    );

    // try POST
    let response = server.post("/graphql").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(
        response
            .header(CONTENT_TYPE)
            .to_str()
            .expect("content type is missing"),
        "application/json"
    );
}
