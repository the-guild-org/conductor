use std::sync::Arc;

use crate::config::HttpCachePluginConfig;
use anyhow::anyhow;
use conductor_cache::cache_manager::{CacheManager, CacheStoreProxy};
use conductor_common::{
  execute::RequestExecutionContext,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  plugin::{CreatablePlugin, Plugin, PluginError},
  vrl_utils::{conductor_request_to_value, VrlProgramProxy},
};
use hex::encode as hex_encode;
use sha2::{Digest, Sha256};
use vrl::value;
#[derive(Debug)]
pub struct HttpCachingPlugin {
  config: HttpCachePluginConfig,
  session_builder: Option<VrlProgramProxy>,
  store: Option<CacheStoreProxy<ConductorHttpResponse>>,
}

impl HttpCachingPlugin {
  fn generate_cache_key(request: &ConductorHttpRequest) -> String {
    let query_body = String::from_utf8_lossy(&request.body);
    format!("{}:{}", request.uri, query_body)
  }

  pub fn configure_caching(&mut self, mgr: Arc<CacheManager>) -> Result<(), PluginError> {
    if let Some(store) = mgr.get_store(&self.config.store_id) {
      self.store = Some(store);
      Ok(())
    } else {
      Err(PluginError::InitError {
        source: anyhow::anyhow!("Cache store not found: {}", self.config.store_id),
      })
    }
  }
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for HttpCachingPlugin {
  type Config = HttpCachePluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    let session_builder = match &config.session_builder {
      Some(condition) => match condition.program() {
        Ok(program) => Some(program),
        Err(e) => {
          return Err(PluginError::InitError {
            source: anyhow::anyhow!("vrl compiler error: {:?}", e),
          })
        }
      },
      None => None,
    };

    Ok(Box::new(Self {
      config,
      store: None,
      session_builder,
    }))
  }
}

impl HttpCachingPlugin {
  fn default_session_builder(ctx: &mut RequestExecutionContext) -> String {
    "".to_string()
  }

  fn build_session_from_request(&self, ctx: &mut RequestExecutionContext) -> Option<String> {
    if let Some(session_builder) = &self.session_builder {
      let downstream_http_req = conductor_request_to_value(&ctx.downstream_http_request);

      let maybe_session = match session_builder.resolve_with_state(
        value::Value::Null,
        value!({
          downstream_http_req: downstream_http_req,
        }),
        ctx.vrl_shared_state(),
      ) {
        Ok(ret) => {
          let t = match ret {
            vrl::value::Value::Bytes(v) => String::from_utf8(v.to_vec()).ok(),
            _ => {
              tracing::error!("HttpCachingPlugin::vrl::session_builder must return a string, but returned a non-string value: {:?}, ignoring...", ret);

              None
            }
          };

          t
        }
        Err(err) => {
          tracing::error!(
            "HttpCachingPlugin::vrl::session_builder resolve error: {:?}, ignoring",
            err
          );

          None
        }
      };

      return maybe_session;
    }

    None
  }

  fn build_cache_key(ctx: &RequestExecutionContext, session_id: String) -> String {
    sha256(
      format!(
        "{}|{}|{}|{}",
        ctx
          .downstream_graphql_request
          .as_ref()
          .map(|v| v.request.operation.clone())
          .unwrap_or_default(),
        ctx
          .downstream_graphql_request
          .as_ref()
          .and_then(|v| v.request.operation_name.clone())
          .unwrap_or_default(),
        ctx
          .downstream_graphql_request
          .as_ref()
          .map(|v| format!("{:?}", v.request.variables))
          .unwrap_or_default(),
        session_id
      )
      .as_bytes(),
    )
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for HttpCachingPlugin {
  async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
    if let Some(store) = &self.store {
      let session_id = self
        .build_session_from_request(ctx)
        .unwrap_or_else(|| Self::default_session_builder(ctx));
      let cache_key = Self::build_cache_key(ctx, session_id);

      if let Some(record) = store.get(&cache_key).await {
        ctx.short_circuit(record);

        return;
      }
    } else {
      tracing::warn!(
        "Cache store '{}' is not configured correctly for http_caching plugin, plugin is skipped.",
        self.config.store_id
      );
    }
  }
}

fn sha256(body: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(body);
  let result = hasher.finalize();
  hex_encode(result)
}
