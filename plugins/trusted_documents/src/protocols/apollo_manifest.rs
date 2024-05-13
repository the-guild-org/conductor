use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::{debug, info};

use super::{ExtractedTrustedDocument, TrustedDocumentsProtocol};
use conductor_common::execute::RequestExecutionContext;
use conductor_common::http::Method;
use no_deadlocks::RwLock;

#[derive(Debug)]
pub struct ApolloManifestPersistedDocumentsProtocol;

#[derive(Deserialize, Debug)]

struct ApolloPersistedOperationsIncomingMessage {
  variables: Option<Map<String, Value>>,
  #[serde(rename = "operationName")]
  operation_name: Option<String>,
  extensions: Extensions,
}

#[derive(Deserialize, Debug)]
struct Extensions {
  #[serde(rename = "persistedQuery")]
  persisted_query: PersistedQuery,
  #[serde(flatten)]
  other: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
struct PersistedQuery {
  #[serde(rename = "sha256Hash")]
  hash: String,
}

#[async_trait::async_trait(?Send)]
impl TrustedDocumentsProtocol for ApolloManifestPersistedDocumentsProtocol {
  async fn try_extraction(
    &self,
    ctx: Arc<RwLock<RequestExecutionContext>>,
  ) -> Option<ExtractedTrustedDocument> {
    if ctx.read().unwrap().downstream_http_request.method == Method::POST {
      debug!("request http method is post, trying to extract from body...");

      if let Ok(message) = ctx
        .read()
        .unwrap()
        .downstream_http_request
        .json_body::<ApolloPersistedOperationsIncomingMessage>()
      {
        info!(
          "succuessfully extracted incoming persisted operation from request: {:?}",
          message
        );

        return Some(ExtractedTrustedDocument {
          hash: message.extensions.persisted_query.hash,
          variables: message.variables,
          operation_name: message.operation_name,
          extensions: Some(message.extensions.other),
        });
      }
    }

    None
  }
}
