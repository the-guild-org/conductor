use conductor_common::serde_utils::{
  JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType, LocalFileReference,
};
use jsonwebtoken::Algorithm;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The `jwt_auth` plugin implements the [JSON Web Tokens](https://jwt.io/introduction) specification.
///
/// It can be used to verify the JWT signature, and optionally validate the token issuer and audience. It can also forward the token and its claims to the upstream service.
///
/// The JWKS configuration can be either a local file on the file-system, or a remote JWKS provider.
///
/// By default, the plugin will look for the JWT token in the `Authorization` header, with the `Bearer` prefix.
///
/// You can also configure the plugin to reject requests that don't have a valid JWT token.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, Default)]
#[schemars(example = "jwt_auth_example_1")]
#[schemars(example = "jwt_auth_example_2")]
#[schemars(example = "jwt_auth_example_3")]
#[schemars(example = "jwt_auth_example_4")]
#[schemars(example = "jwt_auth_example_5")]
pub struct JwtAuthPluginConfig {
  /// A list of JWKS providers to use for verifying the JWT signature.
  /// Can be either a path to a local JSON of the file-system, or a URL to a remote JWKS provider.
  pub jwks_providers: Vec<JwksProviderSourceConfig>,
  /// Specify the [principal](https://tools.ietf.org/html/rfc7519#section-4.1.1) that issued the JWT, usually a URL or an email address.
  /// If specified, it has to match the `iss` field in JWT, otherwise the token's `iss` field is not checked.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub issuers: Option<Vec<String>>,
  /// The list of [JWT audiences](https://tools.ietf.org/html/rfc7519#section-4.1.3) are allowed to access.
  /// If this field is set, the token's `aud` field must be one of the values in this list, otherwise the token's `aud` field is not checked.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audiences: Option<Vec<String>>,
  /// A list of locations to look up for the JWT token in the incoming HTTP request.
  /// The first one that is found will be used.
  #[serde(
    default = "default_lookup_location",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub lookup_locations: Vec<JwtAuthPluginLookupLocation>,
  /// If set to `true`, the entire request will be rejected if the JWT token is not present in the request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reject_unauthenticated_requests: Option<bool>,
  /// List of allowed algorithms for verifying the JWT signature.
  /// If not specified, the default list of all supported algorithms in [`jsonwebtoken` crate](https://crates.io/crates/jsonwebtoken) are used.
  #[serde(
    skip_serializing_if = "Option::is_none",
    default = "default_allowed_algorithms"
  )]
  #[schemars(with = "Option<Vec<String>>")]
  pub allowed_algorithms: Option<Vec<Algorithm>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Forward the JWT token to the upstream service in the specified header.
  pub forward_token_to_upstream_header: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Forward the JWT claims to the upstream service in the specified header.
  pub forward_claims_to_upstream_header: Option<String>,
}

pub fn default_lookup_location() -> Vec<JwtAuthPluginLookupLocation> {
  vec![JwtAuthPluginLookupLocation::Header {
    name: "Authorization".to_string(),
    prefix: Some("Bearer".to_string()),
  }]
}

pub fn default_allowed_algorithms() -> Option<Vec<Algorithm>> {
  Some(vec![
    Algorithm::HS256,
    Algorithm::HS384,
    Algorithm::HS512,
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::ES256,
    Algorithm::ES384,
    Algorithm::PS256,
    Algorithm::PS384,
    Algorithm::PS512,
    Algorithm::EdDSA,
  ])
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum JwtAuthPluginLookupLocation {
  #[serde(rename = "header")]
  #[schemars(title = "header")]
  Header {
    name: String,
    prefix: Option<String>,
  },
  #[serde(rename = "query_param")]
  #[schemars(title = "query_param")]
  QueryParam { name: String },
  #[serde(rename = "cookies")]
  #[schemars(title = "cookies")]
  Cookie { name: String },
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum JwksProviderSourceConfig {
  /// A local file on the file-system. This file will be read once on startup and cached.
  #[serde(rename = "local")]
  #[schemars(title = "local")]
  Local {
    #[serde(rename = "path")]
    /// A path to a local file on the file-system. Relative to the location of the root configuration file.
    file: LocalFileReference,
  },
  /// A remote JWKS provider. The JWKS will be fetched via HTTP/HTTPS and cached.
  #[serde(rename = "remote")]
  #[schemars(title = "remote")]
  Remote {
    /// The URL to fetch the JWKS key set from, via HTTP/HTTPS.
    url: String,
    #[serde(
      deserialize_with = "humantime_serde::deserialize",
      serialize_with = "humantime_serde::serialize",
      default = "default_polling_interval"
    )]
    /// Duration after which the cached JWKS should be expired. If not specified, the default value will be used.
    cache_duration: Option<Duration>,
    /// If set to `true`, the JWKS will be fetched on startup and cached. In case of invalid JWKS, the error will be ignored and the plugin will try to fetch again when server receives the first request.
    /// If set to `false`, the JWKS will be fetched on-demand, when the first request comes in.
    prefetch: Option<bool>,
  },
}
fn default_polling_interval() -> Option<Duration> {
  // Some providers like MS Azure have rate limit configured. So let's use 10 minutes, like Envoy does.
  // and allow users to adjust it if needed.
  // See https://community.auth0.com/t/caching-jwks-signing-key/17654/2
  Some(Duration::from_secs(10 * 60))
}

fn jwt_auth_example_1() -> JsonSchemaExample<JwtAuthPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Local JWKS",
      Some("This example is loading a JWKS file from the local file-system. The token is looked up in the `Authorization` header."),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "jwt_auth".to_string(),
    }),
    example: JwtAuthPluginConfig {
      jwks_providers: vec![JwksProviderSourceConfig::Local {
        file: LocalFileReference {
          path: "jwks.json".to_string(),
          contents: "".to_string(),
        },
      }],
      lookup_locations: vec![JwtAuthPluginLookupLocation::Header {
        name: "Authorization".to_string(),
        prefix: Some("Bearer".to_string()),
      }],
      ..Default::default()
    },
  }
}

fn jwt_auth_example_2() -> JsonSchemaExample<JwtAuthPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Remote JWKS with prefetch",
      Some(
        "This example is loading a remote JWKS, when the server starts (prefetch). The token is looked up in the `Authorization` header.",
      ),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "jwt_auth".to_string(),
    }),
    example: JwtAuthPluginConfig {
      jwks_providers: vec![JwksProviderSourceConfig::Remote {
        url: "https://example.com/jwks.json".to_string(),
        cache_duration: Some(Duration::from_secs(10 * 60)),
        prefetch: Some(true),
      }],
      lookup_locations: vec![JwtAuthPluginLookupLocation::Header {
        name: "Authorization".to_string(),
        prefix: Some("Bearer".to_string()),
      }],
      ..Default::default()
    },
  }
}

fn jwt_auth_example_3() -> JsonSchemaExample<JwtAuthPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Reject Unauthenticated",
      Some(
        "This example is loading a remote JWKS, and looks for the token in the `auth` cookie. If the token is not present, the request will be rejected.",
      ),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "jwt_auth".to_string(),
    }),
    example: JwtAuthPluginConfig {
      jwks_providers: vec![JwksProviderSourceConfig::Remote {
        url: "https://example.com/jwks.json".to_string(),
        cache_duration: Some(Duration::from_secs(10 * 60)),
        prefetch: Some(true),
      }],
      lookup_locations: vec![JwtAuthPluginLookupLocation::Cookie {
        name: "auth".to_string(),
      }],
      reject_unauthenticated_requests: Some(true),
      ..Default::default()
    },
  }
}

fn jwt_auth_example_4() -> JsonSchemaExample<JwtAuthPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Claims Forwarding",
      Some(
        "This example is loading a remote JWKS, and looks for the token in the `jwt` cookie. If the token is not present, the request will be rejected. The token and its claims will be forwarded to the upstream service in the `X-Auth-Token` and `X-Auth-Claims` headers.",
      ),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "jwt_auth".to_string(),
    }),
    example: JwtAuthPluginConfig {
      jwks_providers: vec![JwksProviderSourceConfig::Remote {
        url: "https://example.com/jwks.json".to_string(),
        cache_duration: Some(Duration::from_secs(10 * 60)),
        prefetch: Some(true),
      }],
      lookup_locations: vec![JwtAuthPluginLookupLocation::Cookie {
        name: "jwt".to_string(),
      }],
      reject_unauthenticated_requests: Some(true),
      forward_claims_to_upstream_header: Some("X-Auth-Claims".to_string()),
      forward_token_to_upstream_header: Some("X-Auth-Token".to_string()),
      ..Default::default()
    },
  }
}

fn jwt_auth_example_5() -> JsonSchemaExample<JwtAuthPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Strict Validation",
      Some(
        "This example is using strict validation, where the token issuer and audience are checked.",
      ),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "jwt_auth".to_string(),
    }),
    example: JwtAuthPluginConfig {
      jwks_providers: vec![JwksProviderSourceConfig::Remote {
        url: "https://example.com/jwks.json".to_string(),
        cache_duration: Some(Duration::from_secs(10 * 60)),
        prefetch: None,
      }],
      lookup_locations: vec![JwtAuthPluginLookupLocation::Cookie {
        name: "jwt".to_string(),
      }],
      audiences: Some(vec!["realm.myapp.com".to_string()]),
      issuers: Some(vec!["https://example.com".to_string()]),
      ..Default::default()
    },
  }
}
