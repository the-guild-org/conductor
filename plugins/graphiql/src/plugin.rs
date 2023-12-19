use crate::config::{GraphiQLPluginConfig, GraphiQLSource};
use conductor_common::{
    graphql::APPLICATION_GRAPHQL_JSON,
    http::{
        extract_accept, extract_content_type, Method, Mime, APPLICATION_JSON,
        APPLICATION_WWW_FORM_URLENCODED,
    },
};

use conductor_common::execute::RequestExecutionContext;
use conductor_common::plugin::Plugin;

pub struct GraphiQLPlugin {
    config: GraphiQLPluginConfig,
    endpoint: String,
}

impl GraphiQLPlugin {
    pub fn new(config: GraphiQLPluginConfig, endpoint: String) -> Self {
        Self { config, endpoint }
    }
}

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
                    ctx.short_circuit(render_graphiql(&self.config, self.endpoint.clone()));
                }
            }
        }
    }
}

use conductor_common::http::{ConductorHttpResponse, HttpHeadersMap, StatusCode, CONTENT_TYPE};

const YOGA_GRAPHIQL_VERSION: &str = "4.1.1";

pub fn render_graphiql(config: &GraphiQLPluginConfig, endpoint: String) -> ConductorHttpResponse {
    let config = GraphiQLSource {
        endpoint,
        query: String::from(""),
        headers_editor_enabled: config.headers_editor_enabled.unwrap_or_default(),
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
        YOGA_GRAPHIQL_VERSION,
        serde_json::to_string(&config).unwrap()
    );

    let mut header_map = HttpHeadersMap::new();
    header_map.append(CONTENT_TYPE, "text/html".parse().unwrap());

    ConductorHttpResponse {
        body: body.into(),
        status: StatusCode::OK,
        headers: header_map,
    }
}
