use serde::{Deserialize, Serialize};

// At some point, it might be worth supporting more options. see:
// https://github.com/dotansimha/graphql-yoga/blob/main/packages/graphiql/src/YogaGraphiQL.tsx#L35
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphiQLSource {
    pub endpoint: String,
    pub query: String,
    #[serde(rename = "isHeadersEditorEnabled")]
    pub headers_editor_enabled: bool,
}

const YOGA_GRAPHIQL_VERSION: &str = "4.1.1";

impl GraphiQLSource {
    pub fn new(endpoint: &String) -> GraphiQLSource {
        GraphiQLSource {
            endpoint: endpoint.clone(),
            query: String::from(""),
            headers_editor_enabled: true,
        }
    }

    pub fn finish(&self) -> String {
        format!(
            r#"<!doctype html>
        <html lang="en">
          <head>
            <meta charset="utf-8" />
            <title>Conductor</title>
            <link
              rel="stylesheet"
              href="https://unpkg.com/@graphql-yoga/graphiql@{}/dist/style.css"
            />
          </head>
          <body id="body" class="no-focus-outline">
            <noscript>You need to enable JavaScript to run this app.</noscript>
            <div id="root"></div>
        
            <script type="module">
              import {{ renderYogaGraphiQL }} from 'https://unpkg.com/@graphql-yoga/graphiql@{}/dist/yoga-graphiql.es.js';
        
              renderYogaGraphiQL(root, {});
            </script>
          </body>
        </html>"#,
            YOGA_GRAPHIQL_VERSION,
            YOGA_GRAPHIQL_VERSION,
            serde_json::to_string(self).unwrap()
        )
    }
}
