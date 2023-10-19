use crate::{
    graphql_utils::{GraphQLRequest, ParsedGraphQLRequest},
    http_utils::ExtractGraphQLOperationError,
    plugins::{
        core::Plugin,
        flow_context::FlowContext,
        persisted_documents::{
            config::PersistedOperationsPluginStoreConfig,
            protocols::{
                apollo_manifest::ApolloManifestPersistedDocumentsProtocol,
                document_id::DocumentIdPersistedDocumentsProtocol,
                get_handler::PersistedDocumentsGetHandler,
            },
            store::fs::PersistedDocumentsFilesystemStore,
        },
    },
};

use super::{
    config::{PersistedOperationsPluginConfig, PersistedOperationsProtocolConfig},
    protocols::PersistedDocumentsProtocol,
    store::PersistedDocumentsStore,
};
use async_trait::async_trait;
use tracing::{debug, error, info, warn};

pub struct PersistedOperationsPlugin {
    config: PersistedOperationsPluginConfig,
    incoming_message_handlers: Vec<Box<dyn PersistedDocumentsProtocol>>,
    store: Box<dyn PersistedDocumentsStore>,
}

type ErrorMessage = String;

#[derive(Debug)]
pub enum PersistedOperationsPluginError {
    StoreCreationError(ErrorMessage),
}

impl PersistedOperationsPlugin {
    pub fn new_from_config(
        config: PersistedOperationsPluginConfig,
    ) -> Result<Self, PersistedOperationsPluginError> {
        debug!("creating persisted operations plugin");

        let store: Box<dyn PersistedDocumentsStore> = match &config.store {
            PersistedOperationsPluginStoreConfig::File { file, format } => {
                let fs_store = PersistedDocumentsFilesystemStore::new_from_file_contents(
                    &file.contents,
                    format,
                )
                .map_err(|pe| PersistedOperationsPluginError::StoreCreationError(pe.to_string()))?;

                Box::new(fs_store)
            }
        };

        let incoming_message_handlers: Vec<Box<dyn PersistedDocumentsProtocol>> = config
            .protocols
            .iter()
            .map(|protocol| match protocol {
                PersistedOperationsProtocolConfig::DocumentId { field_name } => {
                    debug!("adding persisted documents protocol of type document_id with field_name: {}", field_name);

                    Box::new(DocumentIdPersistedDocumentsProtocol {
                        field_name: field_name.to_string(),
                    }) as Box<dyn PersistedDocumentsProtocol>
                }
                PersistedOperationsProtocolConfig::ApolloManifestExtensions => {
                    debug!("adding persisted documents protocol of type apollo_manifest (extensions) with field_name");

                    Box::new(ApolloManifestPersistedDocumentsProtocol {})
                        as Box<dyn PersistedDocumentsProtocol>
                }
                PersistedOperationsProtocolConfig::HttpGet {
                    document_id_from,
                    variables_from,
                    operation_name_from,
                } => {
                    debug!(
                        "adding persisted documents protocol of type get HTTP with the following sources: {:?}, {:?}, {:?}",
                        document_id_from, variables_from, operation_name_from
                    );

                    Box::new(PersistedDocumentsGetHandler {
                        document_id_from: document_id_from.clone(),
                        variables_from: variables_from.clone(),
                        operation_name_from: operation_name_from.clone(),
                    }) as Box<dyn PersistedDocumentsProtocol>
                }
            })
            .collect();

        Ok(Self {
            config,
            store,
            incoming_message_handlers,
        })
    }
}

#[async_trait]
impl Plugin for PersistedOperationsPlugin {
    async fn on_downstream_http_request(&self, ctx: &mut FlowContext) {
        if ctx.downstream_graphql_request.is_some() {
            return;
        }

        for extractor in &self.incoming_message_handlers {
            debug!(
                "trying to extract persisted operation from incoming request, extractor: {:?}",
                extractor
            );
            if let Some(extracted) = extractor.as_ref().try_extraction(ctx).await {
                info!(
                    "extracted persisted operation from incoming request: {:?}",
                    extracted
                );

                if let Some(op) = self.store.get_document(&extracted.hash).await {
                    debug!("found persisted operation with id {:?}", extracted.hash);

                    match ParsedGraphQLRequest::create_and_parse(GraphQLRequest {
                        operation: op.clone(),
                        operation_name: extracted.operation_name,
                        variables: extracted.variables,
                        extensions: extracted.extensions,
                    }) {
                        Ok(parsed) => {
                            debug!(
                                "extracted persisted operation is valid, updating request context: {:?}", parsed
                            );

                            ctx.downstream_graphql_request = Some(parsed);
                        }
                        Err(e) => {
                            warn!("failed to parse GraphQL request from a store object with key {:?}, error: {:?}", e, extracted.hash);

                            ctx.short_circuit(e.into_response(None));
                        }
                    }
                } else {
                    warn!("persisted operation with id {:?} not found", extracted.hash);

                    if self.config.allow_non_persisted != Some(true) {
                        error!(
                            "non-persisted operations are not allowed, short-circute with an error"
                        );

                        ctx.short_circuit(
                            ExtractGraphQLOperationError::PersistedOperationNotFound
                                .into_response(None),
                        );
                    }
                }
            }
        }
    }

    async fn on_downstream_graphql_request(&self, ctx: &mut FlowContext) {
        for item in self.incoming_message_handlers.iter() {
            if let Some(response) = item.as_ref().should_prevent_execution(ctx) {
                warn!(
                    "persisted operation execution was prevented, due to falsy value returned from should_prevent_execution from extractor {:?}",item
                );
                ctx.short_circuit(response);
            }
        }
    }
}

#[tokio::test]
async fn persisted_documents_plugin() {
    // use crate::endpoint::endpoint_runtime::EndpointRuntime;
    // use serde_json::json;

    // // use http::header::{ACCEPT, CONTENT_TYPE};
    // // use http::Request;

    // let store = PersistedDocumentsFilesystemStore::new_from_file_contents(
    //     json!({
    //         "key": "query { hello }"
    //     })
    //     .to_string(),
    //     &crate::plugins::persisted_documents::store::fs::PersistedDocumentsFileFormat::JsonKeyValue,
    // );
    // let plugin = PersistedOperationsPlugin::new_from_config(config)
    // let endpoint = EndpointRuntime::mocked_endpoint();
}
