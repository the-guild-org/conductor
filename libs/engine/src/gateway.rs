use std::{fmt::Debug, sync::Arc};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{ExtractGraphQLOperationError, GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::{ConductorHttpRequest, ConductorHttpResponse, Method, StatusCode, Url},
  plugin::PluginError,
};
use conductor_config::{ConductorConfig, EndpointDefinition, SourceDefinition};
use conductor_tracing::{
  manager::TracingManager, minitrace_mgr::MinitraceManager, otel_utils::create_graphql_span,
};
use minitrace::trace;
use tracing::{error, info_span, Instrument, Span};

use crate::{
  plugin_manager::PluginManager,
  source::{
    federation_source::FederationSourceRuntime,
    graphql_source::GraphQLSourceRuntime,
    mock_source::MockedSourceRuntime,
    runtime::{SourceError, SourceRuntime},
  },
};

#[derive(Debug)]
pub struct ConductorGatewayRouteData {
  pub endpoint: String,
  pub plugin_manager: Arc<PluginManager>,
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
}

impl ConductorGateway {
  pub fn match_route(&self, route: &Url) -> Result<&ConductorGatewayRouteData, GatewayError> {
    for conductor_route in &self.routes {
      if route.path().starts_with(&conductor_route.base_path) {
        return Ok(&conductor_route.route_data);
      }
    }

    Err(GatewayError::MissingEndpoint(route.path().to_string()))
  }

  fn create_source(lookup: &str, def: &SourceDefinition) -> Option<Box<dyn SourceRuntime>> {
    match def {
      SourceDefinition::GraphQL { id, config } if id == lookup => Some(Box::new(
        GraphQLSourceRuntime::new(id.clone(), config.clone()),
      )),
      SourceDefinition::Federation { id, config } if id == lookup => Some(Box::new(
        FederationSourceRuntime::new(id.clone(), config.clone()),
      )),
      SourceDefinition::Mock { id, config } if id == lookup => Some(Box::new(
        MockedSourceRuntime::new(id.clone(), config.clone()),
      )),
      _ => None,
    }
  }

  async fn construct_endpoint(
    config_object: &ConductorConfig,
    endpoint_config: &EndpointDefinition,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<ConductorGatewayRouteData, GatewayError> {
    let global_plugins = &config_object.plugins;
    let combined_plugins = global_plugins
      .iter()
      .chain(&endpoint_config.plugins)
      .flat_map(|vec| vec.iter())
      .cloned()
      .collect::<Vec<_>>();

    let plugin_manager = PluginManager::new(
      &endpoint_config.path,
      &Some(combined_plugins),
      tracing_manager,
    )
    .await
    .map_err(GatewayError::PluginManagerInitError)?;

    let upstream_source: Box<dyn SourceRuntime> = config_object
      .sources
      .iter()
      .find_map(|def| ConductorGateway::create_source(&endpoint_config.from, def))
      .ok_or(GatewayError::MissingSource(endpoint_config.from.clone()))?;

    let route_data = ConductorGatewayRouteData {
      endpoint: endpoint_config.path.clone(),
      to: Arc::new(upstream_source),
      plugin_manager: Arc::new(plugin_manager),
    };

    Ok(route_data)
  }

  pub async fn new(
    config_object: &ConductorConfig,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<Self, GatewayError> {
    let mut route_mapping: Vec<ConductorGatewayRoute> = vec![];

    for endpoint_config in config_object.endpoints.iter() {
      let route_data =
        match Self::construct_endpoint(config_object, endpoint_config, tracing_manager).await {
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
    let plugin_manager = PluginManager::new_from_vec(plugins);
    let route_data = ConductorGatewayRouteData {
      endpoint: "/".to_string(),
      plugin_manager: Arc::new(plugin_manager),
      to: source,
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

  #[trace]
  pub async fn execute(
    request: ConductorHttpRequest,
    route_data: &ConductorGatewayRouteData,
  ) -> ConductorHttpResponse {
    let mut request_ctx = RequestExecutionContext::new(request);

    // Step 1: Trigger "on_downstream_http_request" on all plugins
    route_data
      .plugin_manager
      .on_downstream_http_request(&mut request_ctx)
      .await;

    // Step 1.5: In case of short circuit, return the response right now.
    if request_ctx.is_short_circuit() {
      if let Some(mut sc_response) = request_ctx.short_circuit_response.take() {
        route_data
          .plugin_manager
          .on_downstream_http_response(&mut request_ctx, &mut sc_response);

        return sc_response;
      } else {
        return ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None);
      }
    }

    // Step 2: Default handling flow for GraphQL request using POST
    // If plugins didn't extract anything from the request, we can try to do that here.
    // Plugins might have set it before, so we can avoid extraction.
    if request_ctx.downstream_graphql_request.is_none()
      && request_ctx.downstream_http_request.method == Method::POST
    {
      let (_, accept, result) =
        GraphQLRequest::new_from_http_post(&request_ctx.downstream_http_request);

      match result {
        Ok(gql_request) => match ParsedGraphQLRequest::create_and_parse(gql_request) {
          Ok(parsed) => {
            request_ctx.downstream_graphql_request = Some(parsed);
          }
          Err(e) => {
            let mut error_response =
              ExtractGraphQLOperationError::GraphQLParserError(e).into_response(accept);
            route_data
              .plugin_manager
              .on_downstream_http_response(&mut request_ctx, &mut error_response);

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
            .on_downstream_http_response(&mut request_ctx, &mut error_response);

          return error_response;
        }
      }
    }

    // Verify that we have a GraphQL request at this point.
    match request_ctx.downstream_graphql_request.as_ref() {
      Some(gql_operation) => {
        let graphql_span = create_graphql_span(gql_operation);

        async move {
          // Step 3: Execute plugins on the extracted GraphQL request.
          route_data
            .plugin_manager
            .on_downstream_graphql_request(&mut request_ctx)
            .await;

          // Step 3.5: In case of short circuit, return the response right now.
          if request_ctx.is_short_circuit() {
            if let Some(mut sc_response) = request_ctx.short_circuit_response.take() {
              route_data
                .plugin_manager
                .on_downstream_http_response(&mut request_ctx, &mut sc_response);

              return sc_response;
            } else {
              return ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None);
            }
          }

          let span = info_span!("upstream_http", source = route_data.to.name());
          let upstream_response = route_data
            .to
            .execute(route_data, &mut request_ctx)
            .instrument(span)
            .await;

          let final_response = match upstream_response {
            Ok(response) => response,
            Err(e) => match e {
              SourceError::ShortCircuit => {
                return match request_ctx.short_circuit_response {
                  Some(e) => e,
                  None => {
                    ExtractGraphQLOperationError::FailedToCreateResponseBody.into_response(None)
                  }
                }
              }
              e => e.into(),
            },
          };

          if let Some(errors) = final_response.errors.as_ref() {
            if errors.len() > 0 {
              let span = Span::current();
              span.record("graphql.error.count", &errors.len());
              span.record("error.type", "graphql");

              let errors_str = errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join(", ");

              span.record("error.message", errors_str);
            }
          }

          let mut http_response: ConductorHttpResponse = final_response.into();

          route_data
            .plugin_manager
            .on_downstream_http_response(&mut request_ctx, &mut http_response);

          http_response
        }
        .instrument(graphql_span)
        .await
      }
      None => {
        // Step 2.5: In case of invalid request at this point, we can fail and return an error.
        return ConductorHttpResponse {
          body: GraphQLResponse::new_error("failed to extract GraphQL request from HTTP request")
            .into(),
          status: StatusCode::BAD_REQUEST,
          headers: Default::default(),
        };
      }
    }
  }
}
