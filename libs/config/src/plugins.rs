use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    serde_utils::{JsonSchemaExample, JsonSchemaExampleMetadata, LocalFileReference},
    PluginDefinition,
};

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum CorsListStringConfig {
    Wildcard,
    List(Vec<String>),
}

impl Serialize for CorsListStringConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CorsListStringConfig::Wildcard => serializer.serialize_str("*"),
            CorsListStringConfig::List(list) => list.serialize(serializer),
        }
    }
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum CorsStringConfig {
    #[serde(deserialize_with = "deserialize_wildcard")]
    Wildcard,
    Value(String),
}

impl Serialize for CorsStringConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CorsStringConfig::Wildcard => serializer.serialize_str("*"),
            CorsStringConfig::Value(value) => value.serialize(serializer),
        }
    }
}

fn deserialize_wildcard<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename = "*")]
    struct Wildcard;

    let _ = Wildcard::deserialize(deserializer)?;
    Ok(())
}

/// The `cors` plugin enables Cross-Origin Resource Sharing (CORS) configuration for your GraphQL API.
///
/// By using this plugin, you can define rules for allowing cross-origin requests to your GraphQL server. This is essential for web applications that need to interact with your API from different domains.
///
/// ### Configuration Options
///
/// - `allow_credentials`: Specifies whether to include credentials in the CORS headers. Credentials can include cookies, authorization headers, or TLS client certificates.
///
/// - `allowed_methods`: Defines the HTTP methods allowed when accessing the resource. This is used in response to a CORS preflight request.
///
/// - `allowed_origin`: Determines which origins are allowed to access the resource. It can be a specific origin or a wildcard for allowing any origin.
///
/// - `allowed_headers`: Lists the headers allowed in actual requests. This helps in specifying which headers can be used when making the actual request.
///
/// - `allow_private_network`: Indicates whether requests from private networks are allowed when originating from public networks.
///
/// - `max_age`: Sets the duration for which the results of a preflight request can be cached.
///
/// ### Example Usage
///
/// ```yaml
/// cors:
///   enabled: true
///   config:
///     allow_credentials: true
///     allowed_methods: ["GET", "POST"]
///     allowed_origin: "https://example.com"
///     allowed_headers: ["Content-Type", "Authorization"]
///     allow_private_network: false
///     max_age: 3600
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "cors_plugin_example")]
pub struct CorsPluginConfig {
    /// Access-Control-Allow-Credentials (default: false)
    /// Indicates whether the response to the request can be exposed when the credentials flag is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_credentials: Option<bool>,

    /// Access-Control-Allow-Methods (default: Any)
    /// Specifies the method or methods allowed when accessing the resource in response to a preflight request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_methods: Option<CorsListStringConfig>,

    /// Access-Control-Allow-Origin (default: Any)
    /// Specifies a URI that may access the resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_origin: Option<CorsStringConfig>,

    /// Access-Control-Allow-Headers (default: Any)
    /// Used in response to a preflight request to indicate which HTTP headers can be used when making the actual request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_headers: Option<CorsListStringConfig>,

    /// Access-Control-Allow-Private-Network (default: false)
    /// Indicates whether the resource allows requests from private networks initiating from a public network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_private_network: Option<bool>,

    /// Access-Control-Max-Age in seconds (default: empty)
    /// Indicates how long the results of a preflight request can be cached.
    /// This field represents the duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
}

impl Default for CorsPluginConfig {
    fn default() -> Self {
        CorsPluginConfig {
            allow_credentials: Some(false),
            allowed_methods: Some(CorsListStringConfig::Wildcard),
            allowed_origin: Some(CorsStringConfig::Wildcard),
            allowed_headers: Some(CorsListStringConfig::Wildcard),
            allow_private_network: Some(false),
            max_age: None,
        }
    }
}

fn cors_plugin_example() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "CORS Plugin Example",
            Some("This example demonstrates how to configure the CORS plugin."),
        ),
        example: PluginDefinition::CorsPlugin {
            enabled: Some(true),
            config: Some(CorsPluginConfig {
                allow_credentials: Some(true),
                allowed_methods: Some(CorsListStringConfig::List(vec![
                    "GET".into(),
                    "POST".into(),
                ])),
                allowed_origin: Some(CorsStringConfig::Value("https://example.com".into())),
                allowed_headers: Some(CorsListStringConfig::List(vec![
                    "Content-Type".into(),
                    "Authorization".into(),
                ])),
                allow_private_network: Some(false),
                max_age: Some(3600),
            }),
        },
    }
}

/// The `http_get` plugin allows you to expose your GraphQL API over HTTP `GET` requests. This feature is fully compliant with the [GraphQL over HTTP specification](https://graphql.github.io/graphql-over-http/).
///
/// By enabling this plugin, you can execute GraphQL queries and mutations over HTTP `GET` requests, using HTTP query parameters, for example:
///
/// `GET /graphql?query=query%20%7B%20__typename%20%7D`
///
/// ### Query Parameters
///
/// For complete documentation of the supported query parameters, see the [GraphQL over HTTP specification](https://graphql.github.io/graphql-over-http/draft/#sec-GET).
///
/// - `query`: The GraphQL query to execute
///
/// - `variables` (optional): A JSON-encoded string containing the GraphQL variables
///
/// - `operationName` (optional): The name of the GraphQL operation to execute
///
/// ### Headers
///
/// To execute GraphQL queries over HTTP `GET` requests, you must set the `Content-Type` header to `application/json`, **or** the `Accept` header to `application/x-www-form-urlencoded` / `application/graphql-response+json`.
///
#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
#[schemars(example = "http_get_example_1")]
#[schemars(example = "http_get_example_2")]
pub struct HttpGetPluginConfig {
    /// Allow mutations over GET requests.
    ///
    /// **The option is disabled by default:** this restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
    #[serde(
        default = "mutations_default_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutations: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
#[schemars(example = "graphiql_example")]
/// This plugin adds a GraphiQL interface to your Endpoint.
///
/// This plugin is rendering the GraphiQL interface for HTTP `GET` requests, that are not intercepted by other plugins.
pub struct GraphiQLPluginConfig {
    #[serde(
        default = "headers_editor_enabled_default_value",
        skip_serializing_if = "Option::is_none"
    )]
    /// Enable/disable the HTTP headers editor in the GraphiQL interface.
    pub headers_editor_enabled: Option<bool>,
}

fn graphiql_example() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Enable GraphiQL", None),
        example: PluginDefinition::GraphiQLPlugin {
            enabled: Default::default(),
            config: Some(GraphiQLPluginConfig {
                headers_editor_enabled: Default::default(),
            }),
        },
    }
}

fn headers_editor_enabled_default_value() -> Option<bool> {
    Some(true)
}

fn http_get_example_1() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Simple", None),
        example: PluginDefinition::HttpGetPlugin {
            enabled: Default::default(),
            config: Some(HttpGetPluginConfig { mutations: None }),
        },
    }
}

fn http_get_example_2() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Enable Mutations",
            Some("This example enables mutations over HTTP GET requests."),
        ),
        example: PluginDefinition::HttpGetPlugin {
            enabled: Default::default(),
            config: Some(HttpGetPluginConfig {
                mutations: Some(true),
            }),
        },
    }
}

fn mutations_default_value() -> Option<bool> {
    Some(false)
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "persisted_operations_example_1")]
#[schemars(example = "persisted_operations_example_2")]
pub struct PersistedOperationsPluginConfig {
    /// The store defines the source of persisted documents.
    /// The store contents is a list of hashes and GraphQL documents that are allowed to be executed.
    pub store: PersistedOperationsPluginStoreConfig,
    /// A list of protocols to be exposed by this plugin. Each protocol defines how to obtain the document ID from the incoming request.
    /// You can specify multiple kinds of protocols, if needed.
    pub protocols: Vec<PersistedOperationsProtocolConfig>,
    /// By default, this plugin does not allow non-persisted operations to be executed.
    /// This is a security measure to prevent accidental exposure of operations that are not persisted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_non_persisted: Option<bool>,
}

fn persisted_operations_example_1() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Local File Store", Some("This example is using a local file called `persisted_operations.json` as a store, using the Key->Value map format. The protocol exposed is based on HTTP `POST`, using the `documentId` parameter from the request body.")),
        example: PluginDefinition::PersistedOperationsPlugin { enabled: Default::default(), config: PersistedOperationsPluginConfig {
            store: PersistedOperationsPluginStoreConfig::File {
                file: LocalFileReference {
                    path: "persisted_operations.json".to_string(),
                    contents: "".to_string(),
                },
                format: PersistedDocumentsFileFormat::JsonKeyValue,
            },
            allow_non_persisted: None,
            protocols: vec![PersistedOperationsProtocolConfig::DocumentId {
                field_name: "documentId".to_string(),
            }],
        } },
    }
}

fn persisted_operations_example_2() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("HTTP GET", Some("This example uses a local file store called `persisted_operations.json`, using the Key->Value map format. The protocol exposed is based on HTTP `GET`, and extracts all parameters from the query string.")),
        example: PluginDefinition::PersistedOperationsPlugin { enabled: Default::default(), config: PersistedOperationsPluginConfig {
            store: PersistedOperationsPluginStoreConfig::File {
                file: LocalFileReference {
                    path: "persisted_operations.json".to_string(),
                    contents: "".to_string(),
                },
                format: PersistedDocumentsFileFormat::JsonKeyValue,
            },
            allow_non_persisted: None,
            protocols: vec![PersistedOperationsProtocolConfig::HttpGet {
                document_id_from: PersistedOperationHttpGetParameterLocation::document_id_default(),
                variables_from: PersistedOperationHttpGetParameterLocation::variables_default(),
                operation_name_from:
                    PersistedOperationHttpGetParameterLocation::operation_name_default(),
            }],
        } },
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationsPluginStoreConfig {
    #[serde(rename = "file")]
    #[schemars(title = "file")]
    /// File-based store configuration. The path specified is relative to the location of the root configuration file.
    /// The file contents are loaded into memory on startup. The file is not reloaded automatically.
    /// The file format is specified by the `format` field, based on the structure of your file.
    File {
        #[serde(rename = "path")]
        /// A path to a local file on the file-system. Relative to the location of the root configuration file.
        file: LocalFileReference,
        /// The format and the expected structure of the loaded store file.
        format: PersistedDocumentsFileFormat,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum PersistedOperationsProtocolConfig {
    /// This protocol is based on [Apollo's Persisted Query Extensions](https://www.apollographql.com/docs/kotlin/advanced/persisted-queries/#2-publish-operation-manifest).
    /// The GraphQL operation key is sent over `POST` and contains `extensions` field with the GraphQL document hash.
    ///
    /// Example:
    /// `POST /graphql {"extensions": {"persistedQuery": {"version": 1, "sha256Hash": "123"}}`
    #[serde(rename = "apollo_manifest_extensions")]
    #[schemars(title = "apollo_manifest_extensions")]
    ApolloManifestExtensions,
    /// This protocol is based on a `POST` request with a JSON body containing a field with the document ID.
    /// By default, the field name is `documentId`.
    ///
    /// Example:
    /// `POST /graphql {"documentId": "123", "variables": {"code": "AF"}, "operationName": "test"}`
    #[serde(rename = "document_id")]
    #[schemars(title = "document_id")]
    DocumentId {
        /// The name of the JSON field containing the document ID in the incoming request.
        #[serde(default = "document_id_default_field_name")]
        field_name: String,
    },
    /// This protocol is based on a HTTP `GET` request. You can customize where to fetch each one of the parameters from.
    /// Each request parameter can be obtained from a different source: query, path, or header.
    /// By defualt, all parameters are obtained from the query string.
    ///
    /// Unlike other protocols, this protocol does not support sending GraphQL mutations.
    ///
    /// Example:
    /// `GET /graphql?documentId=123&variables=%7B%22code%22%3A%22AF%22%7D&operationName=test`
    #[serde(rename = "http_get")]
    #[schemars(title = "http_get")]
    HttpGet {
        /// Instructions for fetching the document ID parameter from the incoming HTTP request.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::document_id_default")]
        document_id_from: PersistedOperationHttpGetParameterLocation,
        /// Instructions for fetching the variables parameter from the incoming HTTP request.
        /// GraphQL variables must be passed as a JSON-encoded string.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::variables_default")]
        variables_from: PersistedOperationHttpGetParameterLocation,
        /// Instructions for fetching the operationName parameter from the incoming HTTP request.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::operation_name_default")]
        operation_name_from: PersistedOperationHttpGetParameterLocation,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationHttpGetParameterLocation {
    /// Instructs the plugin to extract this parameter from  the query string of the HTTP request.
    #[serde(rename = "search_query")]
    #[schemars(title = "search_query")]
    Query {
        /// The name of the HTTP query parameter.
        name: String,
    },
    /// Instructs the plugin to extract this parameter from the path of the HTTP request.
    #[serde(rename = "path")]
    #[schemars(title = "path")]
    Path {
        /// The numeric value specific the location of the argument (starting from 0).
        position: usize,
    },
    /// Instructs the plugin to extract this parameter from a header in the HTTP request.
    #[serde(rename = "header")]
    #[schemars(title = "header")]
    Header {
        /// The name of the HTTP header.
        name: String,
    },
}

impl PersistedOperationHttpGetParameterLocation {
    pub fn document_id_default() -> Self {
        PersistedOperationHttpGetParameterLocation::Query {
            name: document_id_default_field_name(),
        }
    }

    pub fn variables_default() -> Self {
        PersistedOperationHttpGetParameterLocation::Query {
            name: "variables".to_string(),
        }
    }

    pub fn operation_name_default() -> Self {
        PersistedOperationHttpGetParameterLocation::Query {
            name: "operationName".to_string(),
        }
    }
}

fn document_id_default_field_name() -> String {
    "documentId".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub enum PersistedDocumentsFileFormat {
    #[serde(rename = "apollo_persisted_query_manifest")]
    #[schemars(title = "apollo_persisted_query_manifest")]
    /// JSON file formated based on [Apollo Persisted Query Manifest](https://www.apollographql.com/docs/kotlin/advanced/persisted-queries/#1-generate-operation-manifest).
    ApolloPersistedQueryManifest,
    #[serde(rename = "json_key_value")]
    #[schemars(title = "json_key_value")]
    /// A simple JSON map of key-value pairs.
    ///
    /// Example:
    /// `{"key1": "query { __typename }"}`
    JsonKeyValue,
}
