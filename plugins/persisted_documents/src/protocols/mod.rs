pub mod apollo_manifest;
pub mod document_id;
pub mod get_handler;

use std::fmt::Debug;

use conductor_common::http::ConductorHttpResponse;
use serde_json::{Map, Value};

use conductor_common::execute::RequestExecutionContext;

#[derive(Debug)]
pub struct ExtractedPersistedDocument {
    pub hash: String,
    pub variables: Option<Map<String, Value>>,
    pub operation_name: Option<String>,
    pub extensions: Option<Map<String, Value>>,
}

#[async_trait::async_trait]
pub trait PersistedDocumentsProtocol: Sync + Send + Debug {
    async fn try_extraction(
        &self,
        ctx: &mut RequestExecutionContext,
    ) -> Option<ExtractedPersistedDocument>;
    fn should_prevent_execution(
        &self,
        _ctx: &mut RequestExecutionContext,
    ) -> Option<ConductorHttpResponse> {
        None
    }
}
