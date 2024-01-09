use serde_json::Value;
use tracing::{debug, info};

use super::{ExtractedPersistedDocument, PersistedDocumentsProtocol};
use conductor_common::execute::RequestExecutionContext;
use conductor_common::http::Method;

#[derive(Debug)]
pub struct DocumentIdPersistedDocumentsProtocol {
  pub field_name: String,
}

#[async_trait::async_trait(?Send)]
impl PersistedDocumentsProtocol for DocumentIdPersistedDocumentsProtocol {
  async fn try_extraction(
    &self,
    ctx: &mut RequestExecutionContext,
  ) -> Option<ExtractedPersistedDocument> {
    if ctx.downstream_http_request.method == Method::POST {
      debug!("request http method is post, trying to extract from body...");

      if let Ok(root_object) = ctx.downstream_http_request.json_body::<Value>() {
        debug!(
                    "found valid JSON body in request, trying to extract the document id using field_name: {}",
                    self.field_name
                );

        if let Some(op_id) = root_object
          .get(self.field_name.as_str())
          .and_then(|v| v.as_str())
          .map(|v| v.to_string())
        {
          info!("succuessfully extracted incoming persisted operation from request",);

          return Some(ExtractedPersistedDocument {
            hash: op_id,
            variables: root_object
              .get("variables")
              .and_then(|v| v.as_object())
              .cloned(),
            operation_name: root_object
              .get("operationName")
              .and_then(|v| v.as_str())
              .map(|v| v.to_string()),
            extensions: root_object
              .get("extensions")
              .and_then(|v| v.as_object())
              .cloned(),
          });
        }
      }
    }

    None
  }
}
