use conductor_common::{
  graphql::APPLICATION_GRAPHQL_JSON,
  http::{extract_accept, ConductorHttpResponse, HeaderValue, APPLICATION_JSON, CONTENT_TYPE},
};

use conductor_common::execute::RequestExecutionContext;

use conductor_common::plugin::Plugin;

#[derive(Debug)]
pub struct MatchContentTypePlugin;

#[async_trait::async_trait(?Send)]
impl Plugin for MatchContentTypePlugin {
  fn on_downstream_http_response(
    &self,
    ctx: &mut RequestExecutionContext,
    response: &mut ConductorHttpResponse,
  ) {
    let headers = &mut response.headers;

    if headers.get(CONTENT_TYPE).is_none() {
      let accept_header =
        extract_accept(&ctx.downstream_http_request.headers).unwrap_or(APPLICATION_JSON);

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
