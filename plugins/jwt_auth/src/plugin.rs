use std::str::FromStr;

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::GraphQLResponse,
  http::{parse_query_string, ConductorHttpRequest, StatusCode},
  plugin::{CreatablePlugin, Plugin, PluginError},
};
use cookie::Cookie;
use futures::future::join_all;
use jsonwebtoken::{
  decode, decode_header,
  jwk::{Jwk, JwkSet},
  Algorithm, DecodingKey, Header, TokenData, Validation,
};
use reqwest::header::{InvalidHeaderValue, ToStrError};
use serde_json::Value;
use tracing::{error, warn};

use crate::{
  config::{JwtAuthPluginConfig, JwtAuthPluginLookupLocation},
  jwks_provider::JwksProvider,
};

type TokenPayload = TokenData<Value>;

#[derive(Debug)]
pub struct JwtAuthPlugin {
  config: JwtAuthPluginConfig,
  providers: Vec<JwksProvider>,
}

static CLAIMS_CONTEXT_KEY: &str = "jwt_auth:upstream:claims";
static TOKEN_CONTEXT_KEY: &str = "jwt_auth:upstream:token";

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
  #[error("failed to locate the value in the incoming request")]
  LookupFailed,
  #[error("prefix does not match the found value")]
  MismatchedPrefix,
  #[error("failed to convert header to string")]
  FailedToStringifyHeader(ToStrError),
  #[error("failed to parse header value")]
  FailedToParseHeader(InvalidHeaderValue),
}

impl PartialEq for LookupError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::LookupFailed, Self::LookupFailed) => true,
      (Self::MismatchedPrefix, Self::MismatchedPrefix) => true,
      (Self::FailedToStringifyHeader(s1), Self::FailedToStringifyHeader(s2)) => {
        s1.to_string() == s2.to_string()
      }
      _ => false,
    }
  }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum JwtError {
  #[error("jwt header lookup failed: {0}")]
  LookupFailed(LookupError),
  #[error("failed to parse JWT header: {0}")]
  InvalidJwtHeader(jsonwebtoken::errors::Error),
  #[error("failed to decode JWK: {0}")]
  InvalidDecodingKey(jsonwebtoken::errors::Error),
  #[error("token is not supported by any of the configured providers")]
  FailedToLocateProvider,
  #[error("failed to locate algorithm in jwk")]
  JwkMissingAlgorithm,
  #[error("jwk algorithm is not supported: {0}")]
  JwkAlgorithmNotSupported(jsonwebtoken::errors::Error),
  #[error("failed to decode token: {0}")]
  FailedToDecodeToken(jsonwebtoken::errors::Error),
  #[error("all jwk failed to decode token: {0:?}")]
  AllProvidersFailedToDecode(Vec<JwtError>),
  #[error("http request parsing error: {0:?}")]
  HTTPRequestParsingError(String),
}

impl From<JwtError> for StatusCode {
  fn from(val: JwtError) -> Self {
    match val {
      JwtError::InvalidJwtHeader(_)
      | JwtError::LookupFailed(_)
      | JwtError::JwkAlgorithmNotSupported(_)
      | JwtError::HTTPRequestParsingError(_) => StatusCode::BAD_REQUEST,
      JwtError::JwkMissingAlgorithm
      | JwtError::FailedToLocateProvider
      | JwtError::InvalidDecodingKey(_) => StatusCode::INTERNAL_SERVER_ERROR,
      JwtError::AllProvidersFailedToDecode(_) | JwtError::FailedToDecodeToken(_) => {
        StatusCode::UNAUTHORIZED
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for JwtAuthPlugin {
  type Config = JwtAuthPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    let providers = config
      .jwks_providers
      .iter()
      .map(|provider_config| JwksProvider::new(provider_config.clone()))
      .collect::<Vec<JwksProvider>>();

    for provider in providers.iter().filter(|provider| provider.can_prefetch()) {
      if provider.retrieve_jwk_set().await.is_err() {
        error!("jwt plugin failed to prefetch jwks, ignoring and will try again on first request");
      }
    }

    Ok(Box::new(Self { config, providers }))
  }
}

impl JwtAuthPlugin {
  #[cfg(test)]
  pub(crate) fn new_from_config(config: JwtAuthPluginConfig) -> Self {
    Self {
      config,
      providers: vec![],
    }
  }

  pub(crate) fn find_matching_jwks<'a>(
    &'a self,
    jwt_header: &Header,
    jwks: &'a Vec<&'a JwkSet>,
  ) -> Result<&JwkSet, JwtError> {
    // If `kid` is vailable on the header, we can try to match it to the `kid` on the available JWKs.
    if let Some(jwt_kid) = &jwt_header.kid {
      for jwk in jwks {
        for key in &jwk.keys {
          if key.common.key_id.as_ref().is_some_and(|v| v == jwt_kid) {
            return Ok(jwk);
          }
        }
      }
    }

    // If we don't have `kid` on the token, we should try to match the `alg` field.
    for jwk in jwks {
      for key in &jwk.keys {
        if let Some(key_alg) = key.common.key_algorithm {
          let key_alg_cmp = Algorithm::from_str(&key_alg.to_string())
            .map_err(JwtError::JwkAlgorithmNotSupported)?;
          if key_alg_cmp == jwt_header.alg {
            return Ok(jwk);
          }
        }
      }
    }

    Err(JwtError::FailedToLocateProvider)
  }

  pub(crate) fn lookup(&self, req: &ConductorHttpRequest) -> Result<String, LookupError> {
    for lookup_config in &self.config.lookup_locations {
      match lookup_config {
        JwtAuthPluginLookupLocation::Header { name, prefix } => {
          if let Some(header_value) = req.headers.get(name) {
            let header_str = match header_value.to_str() {
              Ok(s) => s,
              Err(e) => return Err(LookupError::FailedToStringifyHeader(e)),
            };

            let header_value: conductor_common::http::HeaderValue = match header_str.parse() {
              Ok(v) => v,
              Err(e) => return Err(LookupError::FailedToParseHeader(e)),
            };

            match prefix {
              Some(prefix) => match header_value
                .to_str()
                .ok()
                .and_then(|s| s.strip_prefix(prefix))
              {
                Some(stripped_value) => {
                  return Ok(stripped_value.trim().to_string());
                }
                None => {
                  return Err(LookupError::MismatchedPrefix);
                }
              },
              None => {
                return Ok(header_value.to_str().unwrap_or("").to_string());
              }
            }
          }
        }
        JwtAuthPluginLookupLocation::QueryParam { name } => {
          if let Some(query_value) = parse_query_string(&req.query_string).get(name) {
            return Ok(query_value.clone());
          }
        }
        JwtAuthPluginLookupLocation::Cookie { name } => {
          if let Some(cookie_raw) = req.headers.get("cookie") {
            let raw_cookies = match cookie_raw.to_str() {
              Ok(cookies) => cookies.split(';'),
              Err(e) => {
                warn!("jwt plugin failed to convert cookie header to string, ignoring cookie. error: {}", e);
                continue;
              }
            };

            for item in raw_cookies {
              match Cookie::parse_encoded(item) {
                Ok(v) => {
                  let (cookie_name, cookie_value) = v.name_value_trimmed();

                  if cookie_name == name {
                    return Ok(cookie_value.to_string());
                  }
                }
                Err(e) => {
                  // Should we reject the entire request in case of invalid cookies?
                  // I think it's better to consider this as a user error? maybe return 400?
                  warn!(
                    "jwt plugin failed to parse cookie value, ignoring cookie. error: {}",
                    e
                  );
                }
              }
            }
          }
        }
      }
    }

    Err(LookupError::LookupFailed)
  }

  fn try_decode_from_jwk(&self, token: &str, jwk: &Jwk) -> Result<TokenPayload, JwtError> {
    let decoding_key = DecodingKey::from_jwk(jwk).map_err(JwtError::InvalidDecodingKey)?;
    let key_alg = jwk
      .common
      .key_algorithm
      .ok_or(JwtError::JwkMissingAlgorithm)?;

    let alg =
      Algorithm::from_str(&key_alg.to_string()).map_err(JwtError::JwkAlgorithmNotSupported)?;

    let mut validation = Validation::new(alg);

    // This only validates the existence of the claim, it does not validate the values, we'll do it after decoding.
    if let Some(iss) = &self.config.issuers {
      validation.set_issuer(iss);
    }

    // This only validates the existence of the claim, it does not validate the values, we'll do it after decoding.
    if let Some(aud) = &self.config.audiences {
      validation.set_audience(aud);
    }

    let token_data = match decode::<Value>(token, &decoding_key, &validation) {
      Ok(data) => data,
      Err(e) => return Err(JwtError::FailedToDecodeToken(e)),
    };

    match (&self.config.issuers, token_data.claims.get("iss")) {
      (Some(issuers), Some(Value::String(token_iss))) => {
        if !issuers.contains(token_iss) {
          return Err(JwtError::FailedToDecodeToken(
            jsonwebtoken::errors::ErrorKind::InvalidIssuer.into(),
          ));
        }
      }
      (Some(_), None) => {
        return Err(JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidIssuer.into(),
        ));
      }
      _ => {}
    };

    match (&self.config.audiences, token_data.claims.get("aud")) {
      (Some(audiences), Some(Value::Array(token_aud))) => {
        let all_valid = token_aud.iter().all(|v| match v {
          Value::String(token_aud) => audiences.contains(token_aud),
          _ => false,
        });

        if !all_valid {
          return Err(JwtError::FailedToDecodeToken(
            jsonwebtoken::errors::ErrorKind::InvalidAudience.into(),
          ));
        }
      }
      (Some(_), None) => {
        return Err(JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidAudience.into(),
        ));
      }
      _ => {}
    };

    Ok(token_data)
  }

  fn decode_and_validate_token(&self, token: &str, jwks: &[Jwk]) -> Result<TokenPayload, JwtError> {
    let decode_attempts = jwks.iter().map(|jwk| self.try_decode_from_jwk(token, jwk));

    if let Some(success) = decode_attempts.clone().find(|result| result.is_ok()) {
      return success;
    }

    Err(JwtError::AllProvidersFailedToDecode(
      decode_attempts
        .into_iter()
        .map(|result: Result<TokenData<Value>, JwtError>| result.unwrap_err())
        .collect::<Vec<_>>(),
    ))
  }

  pub(crate) fn authenticate(
    &self,
    jwks: &Vec<&JwkSet>,
    req: &ConductorHttpRequest,
  ) -> Result<(TokenData<Value>, String), JwtError> {
    match self.lookup(req) {
      Ok(token) => {
        // First, we need to decode the header to determine which provider to use.
        let header = decode_header(&token).map_err(JwtError::InvalidJwtHeader)?;
        let jwk = self.find_matching_jwks(&header, jwks)?;

        self
          .decode_and_validate_token(&token, &jwk.keys)
          .map(|token_data| (token_data, token))
      }
      Err(e) => {
        warn!("jwt plugin failed to lookup token. error: {}", e);

        Err(JwtError::LookupFailed(e))
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for JwtAuthPlugin {
  async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
    let jwks = join_all(
      self
        .providers
        .iter()
        .map(|provider| provider.retrieve_jwk_set()),
    )
    .await;

    let valid_jwks = jwks
      .iter()
      .filter_map(|r| match r {
        Ok(result) => Some(result.get_jwk()),
        Err(_) => None,
      })
      .collect::<Vec<_>>();

    match self.authenticate(&valid_jwks, &ctx.downstream_http_request) {
      Ok((token_data, token)) => {
        if self.config.forward_claims_to_upstream_header.is_some() {
          ctx.ctx_insert(CLAIMS_CONTEXT_KEY, token_data.claims);
        }
        if self.config.forward_token_to_upstream_header.is_some() {
          ctx.ctx_insert(TOKEN_CONTEXT_KEY, token);
        }
      }
      Err(e) => {
        warn!("jwt token error: {}", e);

        if self
          .config
          .reject_unauthenticated_requests
          .is_some_and(|v| v)
        {
          ctx.short_circuit(
            GraphQLResponse::new_error("unauthenticated request").into_with_status_code(e.into()),
          );
        }
      }
    }
  }

  async fn on_upstream_http_request(
    &self,
    ctx: &mut RequestExecutionContext,
    upstream_req: &mut ConductorHttpRequest,
  ) {
    if let Some(header_name) = &self.config.forward_claims_to_upstream_header {
      if let Some(claims) = ctx.ctx_get(CLAIMS_CONTEXT_KEY) {
        let parsed_header_name = match header_name
          .to_string()
          .parse::<conductor_common::http::HeaderName>()
        {
          Ok(name) => name,
          Err(_) => {
            ctx.short_circuit(
              GraphQLResponse::new_error("Failed to parse header name for claims")
                .into_with_status_code(StatusCode::BAD_REQUEST),
            );
            return;
          }
        };

        let parsed_header_value = match claims
          .to_string()
          .parse::<conductor_common::http::HeaderValue>()
        {
          Ok(value) => value,
          Err(_) => {
            ctx.short_circuit(
              GraphQLResponse::new_error("Failed to parse claims as header value")
                .into_with_status_code(StatusCode::BAD_REQUEST),
            );
            return;
          }
        };

        // if both parsing operations succeed, append the header
        upstream_req
          .headers
          .append(parsed_header_name, parsed_header_value);
      }
    }

    if let Some(header_name) = &self.config.forward_token_to_upstream_header {
      if let Some(token) = ctx.ctx_get(TOKEN_CONTEXT_KEY) {
        let parsed_header_name = match header_name
          .to_string()
          .parse::<conductor_common::http::HeaderName>()
        {
          Ok(name) => name,
          Err(_) => {
            ctx.short_circuit(
              GraphQLResponse::new_error("Failed to parse header name for token")
                .into_with_status_code(StatusCode::BAD_REQUEST),
            );
            return;
          }
        };

        let parsed_header_value = match token
          .as_str()
          .and_then(|t| t.parse::<conductor_common::http::HeaderValue>().ok())
        {
          Some(value) => value,
          None => {
            ctx.short_circuit(
              GraphQLResponse::new_error("Failed to convert token to header value")
                .into_with_status_code(StatusCode::BAD_REQUEST),
            );
            return;
          }
        };

        // append header if both parsed successfully
        upstream_req
          .headers
          .append(parsed_header_name, parsed_header_value);
      }
    }
  }
}
