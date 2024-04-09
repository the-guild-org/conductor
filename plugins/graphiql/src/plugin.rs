use std::sync::Arc;

use crate::config::{GraphiQLPluginConfig, GraphiQLSource};
use conductor_common::{
  graphql::{ExtractGraphQLOperationError, APPLICATION_GRAPHQL_JSON_MIME},
  http::{
    extract_accept, extract_content_type, HeaderValue, Method, Mime, APPLICATION_JSON,
    APPLICATION_WWW_FORM_URLENCODED,
  },
  logging_locks::LoggingRwLock,
  plugin::{CreatablePlugin, PluginError},
};

use no_deadlocks::RwLock;

use conductor_common::execute::RequestExecutionContext;
use conductor_common::plugin::Plugin;

#[derive(Debug)]
pub struct GraphiQLPlugin {
  config: GraphiQLPluginConfig,
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for GraphiQLPlugin {
  type Config = GraphiQLPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    Ok(Box::new(Self { config }))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for GraphiQLPlugin {
  async fn on_downstream_http_request(&self, ctx: Arc<RwLock<RequestExecutionContext>>) {
    let mut ctx_guard = ctx.write().unwrap();

    if ctx_guard.downstream_http_request.method == Method::GET {
      let headers = &ctx_guard.downstream_http_request.headers;
      let content_type = extract_content_type(headers);

      if content_type.is_none() || content_type != Some(APPLICATION_WWW_FORM_URLENCODED) {
        let accept: Option<Mime> = extract_accept(headers);

        if accept != Some(APPLICATION_JSON)
          && accept != Some(APPLICATION_GRAPHQL_JSON_MIME.to_owned())
        {
          let uri = ctx_guard.downstream_http_request.uri.clone();
          ctx_guard.short_circuit(render_graphiql(&self.config, uri));
        }
      }
    }
  }
}

use conductor_common::http::{ConductorHttpResponse, HttpHeadersMap, StatusCode, CONTENT_TYPE};

const YOGA_GRAPHIQL_VERSION: &str = "4.2.1";

pub fn render_graphiql(config: &GraphiQLPluginConfig, endpoint: String) -> ConductorHttpResponse {
  let config = GraphiQLSource {
    endpoint,
    query: String::from(""),
    headers_editor_enabled: config.headers_editor_enabled.unwrap_or_default(),
  };

  let config_json = match serde_json::to_string(&config) {
    Ok(json) => json,
    Err(e) => return ExtractGraphQLOperationError::SerializationError(e).into_response(None),
  };

  let body = format!(
    r#"<!doctype html>
  <html lang="en">
    <head>
      <meta charset="utf-8" />
      <title>Conductor</title>
      <link
        rel="stylesheet"
        href="https://unpkg.com/@graphql-yoga/graphiql@{0}/dist/style.css"
      />
    </head>
    <body id="body" class="no-focus-outline">
      <noscript>You need to enable JavaScript to run this app.</noscript>
      <div id="root"></div>
  
      <script type="module">
        import {{ renderYogaGraphiQL }} from 'https://unpkg.com/@graphql-yoga/graphiql@{0}/dist/yoga-graphiql.es.js';
  
        renderYogaGraphiQL(root, {1});
      </script>
    </body>
  </html>"#,
    YOGA_GRAPHIQL_VERSION, config_json
  );

  let mut header_map = HttpHeadersMap::new();
  header_map.append(CONTENT_TYPE, HeaderValue::from_static("text/html"));

  ConductorHttpResponse {
    body: body.into(),
    status: StatusCode::OK,
    headers: header_map,
  }
}
