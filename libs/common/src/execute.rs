use crate::{
  graphql::ParsedGraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  vrl_utils::serde_value_to_vrl_value,
};
use anyhow::Result;
use serde_json::{Map, Value};
use vrl::compiler::state::RuntimeState;

type Context = Map<String, Value>;

#[derive(Debug)]
pub struct RequestExecutionContext {
  pub downstream_http_request: ConductorHttpRequest,
  pub downstream_graphql_request: Option<ParsedGraphQLRequest>,
  pub short_circuit_response: Option<ConductorHttpResponse>,
  vrl_shared_state: RuntimeState,
  context: Context,
}

impl RequestExecutionContext {
  pub fn new(downstream_http_request: ConductorHttpRequest) -> Self {
    RequestExecutionContext {
      downstream_http_request,
      downstream_graphql_request: None,
      short_circuit_response: None,
      vrl_shared_state: RuntimeState::default(),
      context: Context::new(),
    }
  }

  pub fn vrl_shared_state(&mut self) -> &mut RuntimeState {
    &mut self.vrl_shared_state
  }

  pub fn short_circuit(&mut self, response: ConductorHttpResponse) {
    self.short_circuit_response = Some(response);
  }

  pub fn is_short_circuit(&self) -> bool {
    self.short_circuit_response.is_some()
  }

  pub fn has_failed_extraction(&self) -> bool {
    self.downstream_graphql_request.is_none()
  }

  pub fn ctx_insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
    self.context.insert(key.into(), value.into())
  }

  pub fn ctx_get(&self, key: impl Into<String>) -> Option<&Value> {
    self.context.get(&key.into())
  }

  pub fn ctx_for_vrl(&self) -> Result<vrl::value::Value> {
    serde_value_to_vrl_value(&serde_json::Value::Object(self.context.clone()))
  }
}
