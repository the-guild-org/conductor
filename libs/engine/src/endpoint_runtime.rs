use conductor_common::http::{ConductorHttpResponse, HttpHeadersMap, StatusCode, CONTENT_TYPE};
use conductor_config::{plugins::GraphiQLPluginConfig, EndpointDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
}

const YOGA_GRAPHIQL_VERSION: &str = "4.1.1";

// At some point, it might be worth supporting more options. see:
// https://github.com/dotansimha/graphql-yoga/blob/main/packages/graphiql/src/YogaGraphiQL.tsx#L35
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphiQLSource {
    pub endpoint: String,
    pub query: String,
    #[serde(rename = "isHeadersEditorEnabled")]
    pub headers_editor_enabled: bool,
}

impl EndpointRuntime {
    #[cfg(test)]
    pub fn dummy() -> Self {
        EndpointRuntime {
            config: EndpointDefinition {
                from: "dummy".to_string(),
                path: "/".to_string(),
                plugins: None,
            },
        }
    }

    pub fn render_graphiql(&self, config: &GraphiQLPluginConfig) -> ConductorHttpResponse {
        let config = GraphiQLSource {
            endpoint: self.config.path.to_string(),
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
}
