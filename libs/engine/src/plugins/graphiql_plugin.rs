use conductor_common::{
    graphql::APPLICATION_GRAPHQL_JSON,
    http::{
        extract_accept, extract_content_type, Method, Mime, APPLICATION_JSON,
        APPLICATION_WWW_FORM_URLENCODED,
    },
};

use crate::request_execution_context::RequestExecutionContext;

use super::core::Plugin;

pub struct GraphiQLPlugin {}

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
                    ctx.short_circuit(ctx.endpoint.render_graphiql());
                }
            }
        }
    }
}

// #[tokio::test]
// async fn graphiql_plugin_input_output() {
//     use crate::endpoint::endpoint_runtime::EndpointRuntime;
//     use http::header::{ACCEPT, CONTENT_TYPE};
//     use http::Request;

//     let plugin = GraphiQLPlugin {};
//     let endpoint = EndpointRuntime::mocked_endpoint();

//     // Empty Content-Type -> GraphiQL
//     let mut req = Request::builder()
//         .method("GET")
//         .body(axum::body::Body::empty())
//         .unwrap();
//     let mut ctx = FlowContext::new(&endpoint, &mut req);
//     plugin.on_downstream_http_request(&mut ctx).await;
//     assert_eq!(ctx.is_short_circuit(), true);
//     assert_eq!(
//         ctx.short_circuit_response
//             .unwrap()
//             .headers()
//             .get(CONTENT_TYPE)
//             .unwrap()
//             .to_str()
//             .unwrap(),
//         "text/html; charset=utf-8"
//     );

//     // Should never render GraphiQL when non-GET is used
//     let mut req = Request::builder()
//         .method("POST")
//         .body(axum::body::Body::empty())
//         .unwrap();
//     let mut ctx = FlowContext::new(&endpoint, &mut req);
//     plugin.on_downstream_http_request(&mut ctx).await;
//     assert_eq!(ctx.is_short_circuit(), false);

//     // Should never render GraphiQL when Content-Type is set to APPLICATION_WWW_FORM_URLENCODED
//     let mut req = Request::builder()
//         .method("GET")
//         .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.to_string())
//         .body(axum::body::Body::empty())
//         .unwrap();
//     let mut ctx = FlowContext::new(&endpoint, &mut req);
//     plugin.on_downstream_http_request(&mut ctx).await;
//     assert_eq!(ctx.is_short_circuit(), false);

//     // Should never render GraphiQL when Accept is set to APPLICATION_JSON
//     let mut req = Request::builder()
//         .method("GET")
//         .header(ACCEPT, APPLICATION_JSON.to_string())
//         .body(axum::body::Body::empty())
//         .unwrap();
//     let mut ctx = FlowContext::new(&endpoint, &mut req);
//     plugin.on_downstream_http_request(&mut ctx).await;
//     assert_eq!(ctx.is_short_circuit(), false);
// }

// #[tokio::test]
// async fn graphiql_plugin_render_cases() {
//     use conductor_config::{EndpointDefinition, PluginDefinition};
//     use http::{header::CONTENT_TYPE, StatusCode};

//     let server = crate::test::utils::ConductorTest::empty()
//         .mocked_source()
//         .endpoint(EndpointDefinition {
//             from: "s".to_string(),
//             path: "/graphql".to_string(),
//             plugins: Some(vec![PluginDefinition::GraphiQLPlugin]),
//         })
//         .finish();

//     // try GET
//     let response = server.get("/graphql").await;
//     assert_eq!(response.status_code(), StatusCode::OK);
//     assert_eq!(
//         response
//             .header(CONTENT_TYPE)
//             .to_str()
//             .expect("content type is missing"),
//         "text/html; charset=utf-8"
//     );

//     // try POST
//     let response = server.post("/graphql").await;
//     assert_eq!(response.status_code(), StatusCode::OK);
//     assert_eq!(
//         response
//             .header(CONTENT_TYPE)
//             .to_str()
//             .expect("content type is missing"),
//         "application/json"
//     );
// }
