use crate::config::PersistedOperationHttpGetParameterLocation;
use conductor_common::http::HttpHeadersMap;
use tracing::{debug, info};

use super::{ExtractedPersistedDocument, PersistedDocumentsProtocol};
use conductor_common::execute::RequestExecutionContext;
use conductor_common::{
    graphql::GraphQLResponse,
    http::{parse_query_string, ConductorHttpResponse, Method, StatusCode},
};

#[derive(Debug)]
pub struct PersistedDocumentsGetHandler {
    pub document_id_from: PersistedOperationHttpGetParameterLocation,
    pub variables_from: PersistedOperationHttpGetParameterLocation,
    pub operation_name_from: PersistedOperationHttpGetParameterLocation,
}

fn extract_header(header_map: &HttpHeadersMap, header_name: &String) -> Option<String> {
    header_map
        .get(header_name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

fn extract_path_position(path: &str, position: usize) -> Option<String> {
    path.split('/')
        .collect::<Vec<_>>()
        .get(position)
        .map(|v| v.to_string())
}

fn extract_query_param(query: &str, param_name: &String) -> Option<String> {
    parse_query_string(query)
        .get(param_name)
        .map(|v| v.to_string())
}

impl PersistedDocumentsGetHandler {
    fn maybe_document_id(&self, ctx: &RequestExecutionContext) -> Option<String> {
        debug!(
            "trying to extract document id hash from source {:?}",
            self.operation_name_from
        );

        match &self.document_id_from {
            PersistedOperationHttpGetParameterLocation::Header { name } => {
                extract_header(&ctx.downstream_http_request.headers, name)
            }
            PersistedOperationHttpGetParameterLocation::Query { name } => {
                extract_query_param(&ctx.downstream_http_request.query_string, name)
            }
            PersistedOperationHttpGetParameterLocation::Path { position } => {
                extract_path_position(&ctx.downstream_http_request.uri, *position)
            }
        }
    }

    fn maybe_variables(&self, ctx: &RequestExecutionContext) -> Option<String> {
        debug!(
            "trying to extract variables from source {:?}",
            self.operation_name_from
        );

        match &self.variables_from {
            PersistedOperationHttpGetParameterLocation::Header { name } => {
                extract_header(&ctx.downstream_http_request.headers, name)
            }
            PersistedOperationHttpGetParameterLocation::Query { name } => {
                extract_query_param(&ctx.downstream_http_request.query_string, name)
            }
            PersistedOperationHttpGetParameterLocation::Path { position } => {
                extract_path_position(&ctx.downstream_http_request.uri, *position)
            }
        }
    }

    fn maybe_operation_name(&self, ctx: &RequestExecutionContext) -> Option<String> {
        debug!(
            "trying to extract operationName from source {:?}",
            self.operation_name_from
        );

        match &self.operation_name_from {
            PersistedOperationHttpGetParameterLocation::Header { name } => {
                extract_header(&ctx.downstream_http_request.headers, name)
            }
            PersistedOperationHttpGetParameterLocation::Query { name } => {
                extract_query_param(&ctx.downstream_http_request.query_string, name)
            }
            PersistedOperationHttpGetParameterLocation::Path { position } => {
                extract_path_position(&ctx.downstream_http_request.uri, *position)
            }
        }
    }
}

#[async_trait::async_trait]
impl PersistedDocumentsProtocol for PersistedDocumentsGetHandler {
    async fn try_extraction(
        &self,
        ctx: &mut RequestExecutionContext,
    ) -> Option<ExtractedPersistedDocument> {
        if ctx.downstream_http_request.method == Method::GET {
            debug!("request http method is get, trying to extract from body...");

            if let Some(op_id) = self.maybe_document_id(ctx) {
                info!("succuessfully extracted incoming persisted operation from request",);

                return Some(ExtractedPersistedDocument {
                    hash: op_id,
                    variables: self
                        .maybe_variables(ctx)
                        .and_then(|v| serde_json::from_str(&v).ok()),
                    operation_name: self.maybe_operation_name(ctx),
                    extensions: None,
                });
            }
        }

        None
    }

    fn should_prevent_execution(
        &self,
        ctx: &mut RequestExecutionContext,
    ) -> Option<ConductorHttpResponse> {
        if ctx.downstream_http_request.method == Method::GET {
            if let Some(gql_req) = &ctx.downstream_graphql_request {
                if gql_req.is_running_mutation() {
                    debug!(
                        "trying to execute mutation from the persisted document, preventing because of GET request",
                    );

                    return Some(
                        GraphQLResponse::new_error("mutations are not allowed over GET")
                            .into_with_status_code(StatusCode::METHOD_NOT_ALLOWED),
                    );
                }
            }
        }

        None
    }
}
