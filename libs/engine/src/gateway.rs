use std::{collections::HashMap, fmt::Debug, sync::Arc};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{ExtractGraphQLOperationError, GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::{ConductorHttpRequest, ConductorHttpResponse, Url},
  logging_locks::LoggingRwLock,
  plugin::PluginError,
  plugin_manager::PluginManager,
  source::{GraphQLSourceInitError, SourceError, SourceRuntime},
};
use conductor_config::{ConductorConfig, EndpointDefinition, SourceDefinition};
use conductor_tracing::{
  minitrace_mgr::MinitraceManager,
  otel_attrs::CONDUCTOR_SOURCE,
  otel_utils::{create_graphql_error_span_properties, create_graphql_span},
};
use minitrace::{future::FutureExt, trace, Span};
use reqwest::{Method, StatusCode};
use tracing::error;

use crate::{
  plugin_manager::PluginManagerImpl,
  source::{
    federation_source::FederationSourceRuntime, graphql_source::GraphQLSourceRuntime,
    mock_source::MockedSourceRuntime,
  },
};

#[derive(Debug)]
pub struct ConductorGatewayRouteData {
  pub endpoint: String,
  pub tenant_id: u32,
  pub plugin_manager: Arc<Box<dyn PluginManager>>,
  pub to: Arc<Box<dyn SourceRuntime>>,
}

#[derive(Debug)]
pub struct ConductorGatewayRoute {
  pub base_path: String,
  pub route_data: Arc<ConductorGatewayRouteData>,
}

#[derive(Debug)]
pub struct ConductorGateway {
  pub routes: Vec<ConductorGatewayRoute>,
}

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
  #[error("failed to initialize plugins manager")]
  PluginManagerInitError(PluginError),
  #[error("failed to match route to endpoint: \"{0}\"")]
  MissingEndpoint(String),
  #[error("failed to locate source named \"{0}\", or it's not configured correctly.")]
  MissingSource(String),
  #[error("failed to initialize source '{0}': {1}")]
  SourceInitFailed(String, GraphQLSourceInitError),
}

impl ConductorGateway {
  pub fn match_route(&self, route: &Url) -> Result<&ConductorGatewayRouteData, GatewayError> {
    // TODO: This function should probably use a more sophisticated matching algorithm.
    for conductor_route in &self.routes {
      if route.path() == conductor_route.base_path {
        return Ok(&conductor_route.route_data);
      }
    }

    for conductor_route in &self.routes {
      if route.path().starts_with(&conductor_route.base_path) {
        return Ok(&conductor_route.route_data);
      }
    }

    Err(GatewayError::MissingEndpoint(route.path().to_string()))
  }

  async fn create_source(
    def: &SourceDefinition,
  ) -> Result<Box<dyn SourceRuntime>, GraphQLSourceInitError> {
    Ok(match def {
      SourceDefinition::GraphQL { id, config } => {
        Box::new(GraphQLSourceRuntime::new(id.clone(), config.clone()).await?)
      }
      SourceDefinition::Federation { id, config } => {
        Box::new(FederationSourceRuntime::new(id.clone(), config.clone()).await?)
      }
      SourceDefinition::Mock { id, config } => {
        Box::new(MockedSourceRuntime::new(id.clone(), config.clone()))
      }
    })
  }

  async fn construct_endpoint(
    tenant_id: u32,
    config_object: &ConductorConfig,
    endpoint_config: &EndpointDefinition,
    source_runtime: Arc<Box<dyn SourceRuntime>>,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<ConductorGatewayRouteData, GatewayError> {
    let global_plugins = &config_object.plugins;
    let combined_plugins = global_plugins
      .iter()
      .chain(&endpoint_config.plugins)
      .flat_map(|vec| vec.iter())
      .cloned()
      .collect::<Vec<_>>();

    let plugin_manager =
      PluginManagerImpl::new(&Some(combined_plugins), tracing_manager, tenant_id)
        .await
        .map_err(GatewayError::PluginManagerInitError)?;

    let route_data = ConductorGatewayRouteData {
      endpoint: endpoint_config.path.clone(),
      to: source_runtime,
      plugin_manager: Arc::new(Box::new(plugin_manager)),
      tenant_id,
    };

    Ok(route_data)
  }

  pub async fn new(
    config_object: &ConductorConfig,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<Self, GatewayError> {
    let mut route_mapping: Vec<ConductorGatewayRoute> = vec![];
    let mut sources: HashMap<String, Arc<Box<dyn SourceRuntime>>> = HashMap::new();

    for source_config in config_object.sources.iter() {
      let source = ConductorGateway::create_source(source_config)
        .await
        .map_err(|source| GatewayError::SourceInitFailed(source_config.id().to_owned(), source))?;

      sources.insert(source_config.id().to_owned(), Arc::new(source));
    }

    for (index, endpoint_config) in config_object.endpoints.iter().enumerate() {
      let upstream_source = sources
        .get(&endpoint_config.from)
        .ok_or_else(|| GatewayError::MissingSource(endpoint_config.from.clone()))?;

      let route_data = match Self::construct_endpoint(
        index.try_into().unwrap(),
        config_object,
        endpoint_config,
        upstream_source.clone(),
        tracing_manager,
      )
      .await
      {
        Ok(route_data) => ConductorGatewayRoute {
          base_path: endpoint_config.path.clone(),
          route_data: Arc::new(route_data),
        },
        // @expected: if we are unable to construct the endpoints and attach them onto the gateway's http server, we have to exit
        Err(e) => panic!("failed to construct endpoint: {:?}", e),
      };

      route_mapping.push(route_data);
    }

    Ok(Self {
      routes: route_mapping,
    })
  }

  #[cfg(feature = "test_utils")]
  pub async fn execute_test(
    source: Arc<Box<dyn SourceRuntime>>,
    plugins: Vec<Box<dyn conductor_common::plugin::Plugin>>,
    request: ConductorHttpRequest,
  ) -> ConductorHttpResponse {
    let plugin_manager = PluginManagerImpl::new_from_vec(plugins);
    let route_data = ConductorGatewayRouteData {
      endpoint: "/".to_string(),
      plugin_manager: Arc::new(Box::new(plugin_manager)),
      to: source,
      tenant_id: 0,
    };
    let gw = Self {
      routes: vec![ConductorGatewayRoute {
        base_path: "/".to_string(),
        route_data: Arc::new(route_data),
      }],
    };

    // @expected: we can safely index here, it's inside a test with constant defined fixtures.
    ConductorGateway::execute(request, &gw.routes[0].route_data).await
  }

  #[trace(name = "execute")]
  pub async fn execute(
    request: ConductorHttpRequest,
    route_data: &ConductorGatewayRouteData,
  ) -> ConductorHttpResponse {
    let request_ctx = Arc::new(LoggingRwLock::new(
      "RequestExecutionContext",
      RequestExecutionContext::new(request),
    ));

    // Step 1: Trigger "on_downstream_http_request" on all plugins
    route_data
      .plugin_manager
      .on_downstream_http_request(request_ctx.clone())
      .await;

    // Step 1.5: In case of short circuit, return the response right now.
    if request_ctx.read().await.is_short_circuit() {
      if let Some(mut sc_response) = request_ctx.write().await.short_circuit_response.take() {
        route_data
          .plugin_manager
          .on_downstream_http_response(request_ctx.clone(), &mut sc_response)
          .await;

        return sc_response;
      } else {
        return ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None);
      }
    }

    // Step 2: Default handling flow for GraphQL request using POST
    // If plugins didn't extract anything from the request, we can try to do that here.
    // Plugins might have set it before, so we can avoid extraction.
    if request_ctx
      .read()
      .await
      .downstream_graphql_request
      .is_none()
      && request_ctx.read().await.downstream_http_request.method == Method::POST
    {
      let (_, accept, result) =
        GraphQLRequest::new_from_http_post(&request_ctx.read().await.downstream_http_request);

      match result {
        Ok(gql_request) => match ParsedGraphQLRequest::create_and_parse(gql_request) {
          Ok(parsed) => {
            request_ctx.write().await.downstream_graphql_request = Some(parsed);
          }
          Err(e) => {
            let mut error_response =
              ExtractGraphQLOperationError::GraphQLParserError(e).into_response(accept);
            route_data
              .plugin_manager
              .on_downstream_http_response(request_ctx.clone(), &mut error_response)
              .await;

            return error_response;
          }
        },
        Err(e) => {
          error!(
            "error while trying to extract GraphQL request from POST request: {:?}",
            e
          );

          let mut error_response = e.into_response(accept);
          route_data
            .plugin_manager
            .on_downstream_http_response(request_ctx.clone(), &mut error_response)
            .await;

          return error_response;
        }
      }
    }

    // Verify that we have a GraphQL request at this point.
    let ctx = &mut request_ctx.write().await;
    let downstream_graphql_request = ctx.downstream_graphql_request.as_mut();
    match downstream_graphql_request {
      Some(gql_operation) => {
        let mut _graphql_span = create_graphql_span(&gql_operation);

        // Step 3: Execute plugins on the extracted GraphQL request.
        route_data
          .plugin_manager
          .on_downstream_graphql_request(route_data.to.clone(), request_ctx.clone())
          .await;

        // Step 3.5: In case of short circuit, return the response right now.
        if request_ctx.read().await.is_short_circuit() {
          if let Some(mut sc_response) = request_ctx.write().await.short_circuit_response.take() {
            route_data
              .plugin_manager
              .on_downstream_http_response(request_ctx.clone(), &mut sc_response)
              .await;

            return sc_response;
          } else {
            return ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None);
          }
        }

        let upstream_span = Span::enter_with_parent("upstream_call", &_graphql_span)
          .with_property(|| (CONDUCTOR_SOURCE, route_data.to.name().to_string()));

        let upstream_response = route_data
          .to
          .execute(route_data.plugin_manager.clone(), request_ctx.clone())
          .in_span(upstream_span)
          .await;

        let final_response = match upstream_response {
          Ok(response) => response,
          Err(e) => match e {
            SourceError::ShortCircuit => {
              return match &request_ctx.read().await.short_circuit_response {
                Some(e) => e.clone(),
                None => {
                  ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None)
                }
              }
            }
            e => e.into(),
          },
        };

        if let Some(errors) = final_response.errors.as_ref() {
          _graphql_span =
            _graphql_span.with_properties(|| create_graphql_error_span_properties(errors));
        }

        let mut http_response: ConductorHttpResponse = final_response.into();

        route_data
          .plugin_manager
          .on_downstream_http_response(request_ctx.clone(), &mut http_response)
          .await;

        http_response
      }
      None => {
        // Step 2.5: In case of invalid request at this point, we can fail and return an error.
        ConductorHttpResponse {
          body: GraphQLResponse::new_error("failed to extract GraphQL request from HTTP request")
            .into(),
          status: StatusCode::BAD_REQUEST,
          headers: Default::default(),
        }
      }
    }
  }
}
