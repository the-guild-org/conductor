use std::{
  ops::Deref,
  sync::{Arc, RwLock, RwLockReadGuard},
};

use conductor_common::{
  graphql::{parse_graphql_schema, GraphQLRequest, GraphQLResponse, ParsedGraphQLSchema},
  http::HttpHeadersMap,
  introspection::{introspection_to_sdl, IntrospectionQueryResponse, INTROSPECTION_QUERY},
  parse_introspection_str, SchemaParseError,
};
use conductor_config::{
  SchemaAwarenessConfig, SchemaAwarenessConfigOnError, SchemaAwarenessFormat, SchemaAwarenessSource,
};
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use wasm_polyfills::create_http_client;

#[derive(Debug)]
pub struct SchemaAwarenessRecord<ProcessedValue> {
  raw: Arc<String>,
  schema: Arc<ParsedGraphQLSchema>,
  processed: Arc<ProcessedValue>,
}

impl<ProcessedValue> SchemaAwarenessRecord<ProcessedValue> {
  pub fn raw(&self) -> &Arc<String> {
    &self.raw
  }

  pub fn schema(&self) -> &Arc<ParsedGraphQLSchema> {
    &self.schema
  }

  pub fn processed(&self) -> &Arc<ProcessedValue> {
    &self.processed
  }
}

type ProcessorFn<ProcessedValue> =
  fn(raw: &str, parsed: &ParsedGraphQLSchema) -> Result<ProcessedValue, anyhow::Error>;

#[derive(Debug)]
pub struct SchemaAwareness<ProcessedValue = ()> {
  schema: Arc<RwLock<Option<Arc<SchemaAwarenessRecord<ProcessedValue>>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum SchemaAwarenessError {
  #[error("failed to fetch remote schema")]
  FailedToFetchRemoteSchema { source: reqwest::Error },
  #[error("failed to read remote body")]
  FailedToReadRemoteBody { source: reqwest::Error },
  #[error("failed to parse schema")]
  FailedToParseSchema { source: SchemaParseError },
  #[error("failed to process")]
  FailedToProcessSchema { source: anyhow::Error },
  #[error("failed to load introspection")]
  FailedToParseIntrospection { source: Option<serde_json::Error> },
}

impl<ProcessedValue> SchemaAwareness<ProcessedValue>
where
  ProcessedValue: Send + Sync + 'static,
{
  pub async fn new(
    source_id: String,
    config: SchemaAwarenessConfig,
    processor: ProcessorFn<ProcessedValue>,
  ) -> Result<Self, SchemaAwarenessError> {
    tracing::info!("Initializing schema awareness for source '{}'", source_id);
    let initial_schema = match Self::load_schema(&config.format, &config.source, processor).await {
      Ok(schema) => Some(Arc::new(schema)),
      Err(e) => {
        tracing::error!(
          "Failed to load initial schema awareness for id '{}': {:?}",
          source_id,
          e
        );

        match config.on_error {
          SchemaAwarenessConfigOnError::Ignore => {
            tracing::error!(
              "Failed to load schema awareness for source '{}'. Ignoring.",
              source_id
            );

            None
          }
          SchemaAwarenessConfigOnError::Terminate => {
            return Err(e);
          }
        }
      }
    };

    let instance = Self {
      schema: Arc::new(RwLock::new(initial_schema)),
    };

    if let Some(_polling_interval_duration) = config.polling_interval {
      #[cfg(target_arch = "wasm32")]
      {
        tracing::error!(
          "Schema awareness polling interval is not supported in WASM runtime, ignoring."
        )
      }

      #[cfg(not(target_arch = "wasm32"))]
      {
        let source_id = source_id.clone();
        let source = config.source.clone();
        let format = config.format.clone();
        let handle = instance.schema.clone();

        tokio::spawn(async move {
          Self::fetch_periodically(
            source_id,
            format,
            source,
            handle,
            processor,
            _polling_interval_duration,
          )
          .await;
        });
      }
    }

    Ok(instance)
  }

  #[cfg(not(target_arch = "wasm32"))]
  async fn fetch_periodically(
    source_id: String,
    format: SchemaAwarenessFormat,
    source: SchemaAwarenessSource,
    handle: Arc<RwLock<Option<Arc<SchemaAwarenessRecord<ProcessedValue>>>>>,
    processor: ProcessorFn<ProcessedValue>,
    duration: std::time::Duration,
  ) {
    let mut interval_timer = tokio::time::interval(duration);

    loop {
      interval_timer.tick().await;
      Self::load_and_update_schema(
        source_id.clone(),
        format.clone(),
        source.clone(),
        handle.clone(),
        processor,
      )
      .await;
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  async fn load_and_update_schema(
    source_id: String,
    format: SchemaAwarenessFormat,
    source: SchemaAwarenessSource,
    handle: Arc<RwLock<Option<Arc<SchemaAwarenessRecord<ProcessedValue>>>>>,
    processor: ProcessorFn<ProcessedValue>,
  ) {
    tracing::debug!(
      "fetching schema awareness for source '{}', format: {:?} source config: {:?}",
      source_id,
      format,
      source
    );
    let schema = Self::load_schema(&format, &source, processor).await;

    match schema {
      Ok(schema) => match handle.write() {
        Ok(mut t) => {
          tracing::debug!(
            "successfully loaded schema awareness for source '{}', updating local record",
            source_id
          );
          t.replace(Arc::new(schema));
        }
        Err(e) => {
          tracing::error!(
            "Failed to accquire lock for schema awareness for source '{}': {:?}",
            source_id,
            e
          );
        }
      },
      Err(e) => tracing::error!(
        "Failed to load schema awareness for id '{}': {:?}",
        source_id,
        e
      ),
    };
  }

  async fn load_schema<'a>(
    format: &'a SchemaAwarenessFormat,
    source: &'a SchemaAwarenessSource,
    processor: ProcessorFn<ProcessedValue>,
  ) -> Result<SchemaAwarenessRecord<ProcessedValue>, SchemaAwarenessError> {
    let result = match (format, source) {
      (SchemaAwarenessFormat::Sdl, SchemaAwarenessSource::File { file }) => {
        parse_graphql_schema(&file.contents).map(|schema| (file.contents.clone(), schema))
      }
      (SchemaAwarenessFormat::Sdl, SchemaAwarenessSource::Inline { content }) => {
        parse_graphql_schema(&content).map(|schema| (content.clone(), schema))
      }
      (
        SchemaAwarenessFormat::Sdl,
        SchemaAwarenessSource::Remote {
          url,
          headers,
          method,
        },
      ) => {
        let http_client = create_http_client().build().unwrap();
        let res = http_client
          .request(method.clone(), url)
          .headers(headers.clone())
          .send()
          .await
          .map_err(|source| SchemaAwarenessError::FailedToFetchRemoteSchema { source })?
          .text()
          .await
          .map_err(|source| SchemaAwarenessError::FailedToReadRemoteBody { source })?;

        parse_graphql_schema(&res).map(|schema| (res, schema))
      }
      (
        SchemaAwarenessFormat::Introspection,
        SchemaAwarenessSource::Remote {
          url,
          headers,
          method,
        },
      ) => {
        let http_client = create_http_client().build().unwrap();
        let mut headers = HttpHeadersMap::from(headers.clone());
        headers
          .entry(CONTENT_TYPE)
          .or_insert(HeaderValue::from_static("application/json"));
        headers
          .entry(ACCEPT)
          .or_insert(HeaderValue::from_static("application/json"));

        let gql_response = http_client
          .request(method.clone(), url)
          .headers(headers.clone())
          .body(
            GraphQLRequest {
              operation: INTROSPECTION_QUERY.to_string(),
              operation_name: Some(String::from("IntrospectionQuery")),
              extensions: None,
              variables: None,
            }
            .to_string(),
          )
          .send()
          .await
          .map_err(|source| SchemaAwarenessError::FailedToFetchRemoteSchema { source })?
          .json::<GraphQLResponse<IntrospectionQueryResponse>>()
          .await
          .map_err(|source| SchemaAwarenessError::FailedToReadRemoteBody { source })?;

        match gql_response.data {
          None => return Err(SchemaAwarenessError::FailedToParseIntrospection { source: None }),
          Some(data) => {
            let as_sdl_obj = introspection_to_sdl(data);

            Ok((as_sdl_obj.to_string(), as_sdl_obj))
          }
        }
      }
      (SchemaAwarenessFormat::Introspection, SchemaAwarenessSource::File { file }) => {
        let introspection = parse_introspection_str(&file.contents).map_err(|source| {
          SchemaAwarenessError::FailedToParseIntrospection {
            source: Some(source),
          }
        })?;

        let as_sdl_obj = introspection_to_sdl(introspection);

        Ok((as_sdl_obj.to_string(), as_sdl_obj))
      }
      (SchemaAwarenessFormat::Introspection, SchemaAwarenessSource::Inline { content }) => {
        let introspection = parse_introspection_str(&content).map_err(|source| {
          SchemaAwarenessError::FailedToParseIntrospection {
            source: Some(source),
          }
        })?;

        let as_sdl_obj = introspection_to_sdl(introspection);

        Ok((as_sdl_obj.to_string(), as_sdl_obj))
      }
    }
    .map_err(|source| SchemaAwarenessError::FailedToParseSchema { source })?;

    let processed = processor(&result.0, &result.1)
      .map_err(|source| SchemaAwarenessError::FailedToProcessSchema { source })?;

    Ok(SchemaAwarenessRecord {
      raw: Arc::new(result.0),
      schema: Arc::new(result.1),
      processed: Arc::new(processed),
    })
  }

  fn record(&self) -> RwLockReadGuard<Option<Arc<SchemaAwarenessRecord<ProcessedValue>>>> {
    self.schema.read().unwrap()
  }

  pub fn schema(&self) -> Option<Arc<ParsedGraphQLSchema>> {
    if let Some(record) = self.record().deref() {
      return Some(record.schema().clone());
    }

    None
  }

  pub fn raw(&self) -> Option<Arc<String>> {
    if let Some(record) = self.record().deref() {
      return Some(record.raw().clone());
    }

    None
  }

  pub fn processed(&self) -> Option<Arc<ProcessedValue>> {
    if let Some(record) = self.record().deref() {
      return Some(record.processed().clone());
    }

    None
  }
}
