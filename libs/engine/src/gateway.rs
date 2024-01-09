use std::{fmt::Debug, sync::Arc};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{ExtractGraphQLOperationError, GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::{ConductorHttpRequest, ConductorHttpResponse, Method, StatusCode, Url},
};
use conductor_config::{ConductorConfig, EndpointDefinition, SourceDefinition};
use futures::future::join_all;
use tracing::{debug, error};

use crate::{
  plugin_manager::PluginManager,
  source::{
    graphql_source::GraphQLSourceRuntime,
    mock_source::MockedSourceRuntime,
    runtime::{SourceError, SourceRuntime},
  },
};

#[derive(Debug)]
pub struct ConductorGatewayRouteData {
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
  PluginManagerInitError,
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
      SourceDefinition::GraphQL { id, config } if id == lookup => {
        Some(Box::new(GraphQLSourceRuntime::new(config.clone())))
      }
      SourceDefinition::Mock { id, config } if id == lookup => {
        Some(Box::new(MockedSourceRuntime::new(config.clone())))
      }
      _ => None,
    }
  }

  async fn construct_endpoint(
    config_object: &ConductorConfig,
    endpoint_config: &EndpointDefinition,
  ) -> Result<ConductorGatewayRouteData, GatewayError> {
    let global_plugins = &config_object.plugins;
    let combined_plugins = global_plugins
      .iter()
      .chain(&endpoint_config.plugins)
      .flat_map(|vec| vec.iter())
      .cloned()
      .collect::<Vec<_>>();

    let plugin_manager = PluginManager::new(&Some(combined_plugins))
      .await
      .map_err(|_| GatewayError::PluginManagerInitError)?;

    let upstream_source: Box<dyn SourceRuntime> = config_object
      .sources
      .iter()
      .find_map(|def| ConductorGateway::create_source(&endpoint_config.from, def))
      .ok_or(GatewayError::MissingSource(endpoint_config.from.clone()))?;

    let route_data = ConductorGatewayRouteData {
      to: Arc::new(upstream_source),
      plugin_manager: Arc::new(plugin_manager),
    };

    Ok(route_data)
  }

  pub async fn new(config_object: &ConductorConfig) -> Result<Self, GatewayError> {
    let route_mapping = join_all(
      config_object
        .endpoints
        .iter()
        .map(move |endpoint_config| async move {
          match Self::construct_endpoint(config_object, endpoint_config).await {
            Ok(route_data) => ConductorGatewayRoute {
              base_path: endpoint_config.path.clone(),
              route_data: Arc::new(route_data),
            },
            Err(e) => panic!("failed to construct endpoint: {:?}", e),
          }
        })
        .collect::<Vec<_>>(),
    )
    .await;

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
      plugin_manager: Arc::new(plugin_manager),
      to: source,
    };
    let gw = Self {
      routes: vec![ConductorGatewayRoute {
        base_path: "/".to_string(),
        route_data: Arc::new(route_data),
      }],
    };

    ConductorGateway::execute(request, &gw.routes[0].route_data).await
  }

  #[tracing::instrument(skip(request, route_data), name = "ConductorGateway::execute")]
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
      let mut sc_response = request_ctx.short_circuit_response.unwrap();
      request_ctx.short_circuit_response = None;

      route_data
        .plugin_manager
        .on_downstream_http_response(&mut request_ctx, &mut sc_response);

      return sc_response;
    }

    // Step 2: Default handling flow for GraphQL request using POST
    // If plugins didn't extract anything from the request, we can try to do that here.
    // Plugins might have set it before, so we can avoid extraction.
    if request_ctx.downstream_graphql_request.is_none()
      && request_ctx.downstream_http_request.method == Method::POST
    {
      debug!("captured POST request, trying to handle as GraphQL POST flow");
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

    // Step 2.5: In case of invalid request at this point, we can fail and return an error.
    if request_ctx.has_failed_extraction() {
      return ConductorHttpResponse {
        body: GraphQLResponse::new_error("failed to extract GraphQL request from HTTP request")
          .into(),
        status: StatusCode::BAD_REQUEST,
        headers: Default::default(),
      };
    }

    // Step 3: Execute plugins on the extracted GraphQL request.
    route_data
      .plugin_manager
      .on_downstream_graphql_request(&mut request_ctx)
      .await;

    // Step 3.5: In case of short circuit, return the response right now.
    if request_ctx.is_short_circuit() {
      let mut sc_response = request_ctx.short_circuit_response.unwrap();
      request_ctx.short_circuit_response = None;

      route_data
        .plugin_manager
        .on_downstream_http_response(&mut request_ctx, &mut sc_response);

      return sc_response;
    }

    let upstream_response = route_data.to.execute(route_data, &mut request_ctx).await;
    let final_response = match upstream_response {
      Ok(response) => response,
      Err(e) => match e {
        SourceError::ShortCircuit => {
          return request_ctx.short_circuit_response.unwrap();
        }
        e => e.into(),
      },
    };

    let mut http_response: ConductorHttpResponse = final_response.into();

    route_data
      .plugin_manager
      .on_downstream_http_response(&mut request_ctx, &mut http_response);

    http_response
  }
}
