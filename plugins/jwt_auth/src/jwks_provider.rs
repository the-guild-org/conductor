use std::{
  sync::{Arc, RwLock},
  time::{Duration, SystemTime},
};

use jsonwebtoken::jwk::JwkSet;

use crate::config::JwksProviderSourceConfig;

#[derive(Debug)]
pub struct JwksProvider {
  config: JwksProviderSourceConfig,
  jwk: RwLock<Option<Arc<TimedJwtSet>>>,
}

#[derive(Debug)]
pub struct TimedJwtSet {
  expiration: Option<SystemTime>,
  set: JwkSet,
}

impl TimedJwtSet {
  pub fn get_jwk(&self) -> &JwkSet {
    &self.set
  }
}

#[derive(thiserror::Error, Debug)]
pub enum JwksProviderError {
  #[error("failed to load remote jwks: {0}")]
  RemoteJwksNetworkError(reqwest::Error),
  #[error("failed to parse jwks json file: {0}")]
  JwksContentInvalidStructure(serde_json::Error),
  #[error("failed to acquire access to jwk handle")]
  FailedToAcquireJwk,
}

impl JwksProvider {
  async fn load_jwks(&self) -> Result<&Self, JwksProviderError> {
    let new_jwk = Some(Arc::new(match &self.config {
      JwksProviderSourceConfig::Remote {
        url,
        cache_duration,
        prefetch: _,
      } => {
        // @expected: if initiating an http client fails, then we have to exit.
        let client = wasm_polyfills::create_http_client().build().unwrap();
        let response_text = client
          .get(url)
          .send()
          .await
          .map_err(JwksProviderError::RemoteJwksNetworkError)?
          .text()
          .await
          .map_err(JwksProviderError::RemoteJwksNetworkError)?;
        let expiration =
          SystemTime::now().checked_add(cache_duration.unwrap_or(Duration::from_secs(10 * 60)));
        let set = serde_json::from_str::<JwkSet>(&response_text)
          .map_err(JwksProviderError::JwksContentInvalidStructure)?;

        TimedJwtSet { expiration, set }
      }
      JwksProviderSourceConfig::Local { file } => TimedJwtSet {
        expiration: None,
        set: serde_json::from_str::<JwkSet>(&file.contents)
          .map_err(JwksProviderError::JwksContentInvalidStructure)?,
      },
    }));

    if let Ok(mut w_jwk) = self.jwk.write() {
      *w_jwk = new_jwk;
    }

    Ok(self)
  }

  pub fn new(config: JwksProviderSourceConfig) -> Self {
    Self {
      config,
      jwk: RwLock::new(None),
    }
  }

  #[cfg(target_arch = "wasm32")]
  pub fn can_prefetch(&self) -> bool {
    use tracing::error;

    error!("jwks prefetching is not supported on wasm32, ignoring");

    false
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn can_prefetch(&self) -> bool {
    match &self.config {
      JwksProviderSourceConfig::Remote { prefetch, .. } => match prefetch {
        Some(prefetch) => *prefetch,
        None => false,
      },
      JwksProviderSourceConfig::Local { .. } => false,
    }
  }

  fn needs_refetch(&self) -> bool {
    if let Ok(jwk) = self.jwk.try_read() {
      return match jwk.as_ref() {
        Some(jwk) => match jwk.expiration {
          Some(expiration) => SystemTime::now() > expiration,
          None => false,
        },
        None => true,
      };
    }

    true
  }

  pub async fn retrieve_jwk_set(&self) -> Result<Arc<TimedJwtSet>, JwksProviderError> {
    if self.needs_refetch() {
      self.load_jwks().await?;
    }

    if let Ok(jwk) = self.jwk.try_read() {
      if let Some(jwk) = jwk.as_ref() {
        return Ok(jwk.clone());
      }
    }

    Err(JwksProviderError::FailedToAcquireJwk)
  }
}
