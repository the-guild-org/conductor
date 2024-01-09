use crate::{
  protocols::{
    apollo_manifest::ApolloManifestPersistedDocumentsProtocol,
    document_id::DocumentIdPersistedDocumentsProtocol, get_handler::PersistedDocumentsGetHandler,
  },
  store::fs::PersistedDocumentsFilesystemStore,
};

use super::{protocols::PersistedDocumentsProtocol, store::PersistedDocumentsStore};
use crate::config::{
  PersistedOperationsPluginConfig, PersistedOperationsPluginStoreConfig,
  PersistedOperationsProtocolConfig,
};
use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{ExtractGraphQLOperationError, GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::StatusCode,
  plugin::{CreatablePlugin, Plugin, PluginError},
};
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct PersistedOperationsPlugin {
  config: PersistedOperationsPluginConfig,
  incoming_message_handlers: Vec<Box<dyn PersistedDocumentsProtocol>>,
  store: Box<dyn PersistedDocumentsStore>,
}

#[derive(Debug, thiserror::Error)]
pub enum PersistedOperationsPluginError {
  #[error("failed to create store: {0}")]
  StoreCreationError(String),
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for PersistedOperationsPlugin {
  type Config = PersistedOperationsPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<dyn Plugin>, PluginError> {
    debug!("creating persisted operations plugin");

    let store: Box<dyn PersistedDocumentsStore> = match &config.store {
      PersistedOperationsPluginStoreConfig::File { file, format } => {
        let fs_store =
          PersistedDocumentsFilesystemStore::new_from_file_contents(&file.contents, format)
            .map_err(|e| PluginError::InitError {
              source: PersistedOperationsPluginError::StoreCreationError(e.to_string()).into(),
            })?;

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

    Ok(Box::new(Self {
      config,
      store,
      incoming_message_handlers,
    }))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for PersistedOperationsPlugin {
  async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
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
            operation: Some(op.clone()),
            operation_name: extracted.operation_name,
            variables: extracted.variables,
            extensions: extracted.extensions,
          }) {
            Ok(parsed) => {
              debug!(
                "extracted persisted operation is valid, updating request context: {:?}",
                parsed
              );

              ctx.downstream_graphql_request = Some(parsed);
              return;
            }
            Err(e) => {
              warn!(
                "failed to parse GraphQL request from a store object with key {:?}, error: {:?}",
                e, extracted.hash
              );

              ctx.short_circuit(
                ExtractGraphQLOperationError::GraphQLParserError(e).into_response(None),
              );
              return;
            }
          }
        } else {
          warn!("persisted operation with id {:?} not found", extracted.hash);
        }
      }
    }

    if self.config.allow_non_persisted != Some(true) {
      error!("non-persisted operations are not allowed, short-circute with an error");

      ctx.short_circuit(
        GraphQLResponse::new_error("persisted operation not found")
          .into_with_status_code(StatusCode::NOT_FOUND),
      );

      return;
    }
  }

  async fn on_downstream_graphql_request(&self, ctx: &mut RequestExecutionContext) {
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
