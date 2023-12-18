use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    serde_utils::{JsonSchemaExample, JsonSchemaExampleMetadata, LocalFileReference},
    PluginDefinition,
};

#[derive(Default, Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "disable_introspection_example1")]
#[schemars(example = "disable_introspection_example2")]
/// The `disable_introspection` plugin allows you to disable introspection for your GraphQL API.
///
/// A [GraphQL introspection query](https://graphql.org/learn/introspection/) is a special GraphQL query that returns information about the GraphQL schema of your API.
///
/// It it [recommended to disable introspection for production environments](https://escape.tech/blog/should-i-disable-introspection-in-graphql/), unless you have a specific use-case for it.
///
/// It can either disable introspection for all requests, or only for requests that match a specific condition (using VRL scripting language).
///
pub struct DisableIntrospectionPluginConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// A VRL condition that determines whether to disable introspection for the request. This condition is evaluated only if the incoming GraphQL request is detected as an introspection query.
    ///
    /// The condition is evaluated in the context of the incoming request and have access to the metadata field `%downstream_http_req` (fields: `body`, `uri`, `query_string`, `method`, `headers`).
    ///
    /// The condition must return a boolean value: return `true` to continue and disable the introspection, and `false` to allow the introspection to run.
    ///
    /// In case of a runtime error, or an unexpected return value, the script will be ignored and introspection will be disabled for the incoming request.
    pub condition: Option<VrlConfigReference>,
}

fn disable_introspection_example1() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Disable Introspection",
            Some(
                "This example disables introspection for all requests for the configured Endpoint.",
            ),
        ),
        example: PluginDefinition::DisableItrospectionPlugin {
            enabled: Some(true),
            config: Some(DisableIntrospectionPluginConfig {
                ..Default::default()
            }),
        },
    }
}

fn disable_introspection_example2() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Conditional",
            Some(
                "This example disables introspection for all requests that doesn't have the \"bypass-introspection\" HTTP header.",
            ),
        ),
        example: PluginDefinition::DisableItrospectionPlugin {
            enabled: Some(true),
            config: Some(DisableIntrospectionPluginConfig {
                condition: Some(VrlConfigReference::Inline { content: "%downstream_http_req.headers.\"bypass-introspection\" != \"1\"".to_string() }),
            }),
        },
    }
}

/// The `cors` plugin enables [Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) configuration for your GraphQL API.
///
/// By using this plugin, you can define rules for allowing cross-origin requests to your GraphQL server. This is essential for web applications that need to interact with your API from different domains.
///
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "cors_plugin_example1")]
#[schemars(example = "cors_plugin_example2")]
#[schemars(example = "cors_plugin_example3")]
pub struct CorsPluginConfig {
    /// `Access-Control-Allow-Credentials`: Specifies whether to include credentials in the CORS headers. Credentials can include cookies, authorization headers, or TLS client certificates.
    /// Indicates whether the response to the request can be exposed when the credentials flag is true.
    #[serde(
        default = "default_boolean_false",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_credentials: Option<bool>,

    /// `Access-Control-Allow-Methods`: Defines the HTTP methods allowed when accessing the resource. This is used in response to a CORS preflight request.
    /// Specifies the method or methods allowed when accessing the resource in response to a preflight request.
    /// You can also specify a special value "*" to allow any HTTP method to access the resource.
    #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
    pub allowed_methods: Option<String>,

    /// `Access-Control-Allow-Origin`: Determines which origins are allowed to access the resource. It can be a specific origin or a wildcard for allowing any origin.
    /// You can also specify a special value "*" to allow any origin to access the resource.
    /// You can also specify a special value "reflect" to allow the origin of the incoming request to access the resource.
    #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
    pub allowed_origin: Option<String>,

    /// `Access-Control-Allow-Headers`: Lists the headers allowed in actual requests. This helps in specifying which headers can be used when making the actual request.
    /// Used in response to a preflight request to indicate which HTTP headers can be used when making the actual request.
    /// You can also specify a special value "*" to allow any headers to be used when making the actual request, and the `Access-Control-Request-Headers` will be used from the incoming request.
    #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
    pub allowed_headers: Option<String>,

    /// `Access-Control-Expose-Headers`: The "Access-Control-Expose-Headers" response header allows a server to indicate which response headers should be made available to scripts running in the browser, in response to a cross-origin request.
    /// You can also specify a special value "*" to allow any headers to be exposed to scripts running in the browser.
    #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
    pub exposed_headers: Option<String>,

    /// `Access-Control-Allow-Private-Network`: Indicates whether requests from private networks are allowed when originating from public networks.
    #[serde(
        default = "default_boolean_false",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_private_network: Option<bool>,

    /// `Access-Control-Max-Age`: Indicates how long the results of a preflight request can be cached.
    /// This field represents the duration in seconds.
    #[serde(default = "defualt_max_age", skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
}

impl Default for CorsPluginConfig {
    fn default() -> Self {
        Self {
            allow_credentials: default_boolean_false(),
            allowed_methods: default_wildcard(),
            allowed_origin: default_wildcard(),
            allowed_headers: default_wildcard(),
            exposed_headers: default_wildcard(),
            allow_private_network: default_boolean_false(),
            max_age: defualt_max_age(),
        }
    }
}

fn default_wildcard() -> Option<String> {
    Some("*".to_string())
}

fn default_boolean_false() -> Option<bool> {
    Some(false)
}

fn defualt_max_age() -> Option<u64> {
    None
}

fn cors_plugin_example1() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Strict CORS",
            Some("This example demonstrates how to configure the CORS plugin with a strict list of methods, headers and origins."),
        ),
        example: PluginDefinition::CorsPlugin {
            enabled: Some(true),
            config: Some(CorsPluginConfig {
                allow_credentials: Some(true),
                exposed_headers: None,
                allowed_methods: Some("GET, POST".into()),
                allowed_origin: Some("https://example.com".into()),
                allowed_headers: Some("Content-Type, Authorization".into()),
                allow_private_network: Some(false),
                max_age: Some(3600),
            }),
        },
    }
}

fn cors_plugin_example2() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Permissive CORS",
            Some("This example demonstrates how to configure the CORS plugin with a permissive setup."),
        ),
        example: PluginDefinition::CorsPlugin {
            enabled: Some(true),
            config: Some(CorsPluginConfig {
                allow_credentials: Some(true),
                exposed_headers: Some("*".into()),
                allowed_methods: Some("*".into()),
                allowed_origin: Some("*".into()),
                allowed_headers: Some("*".into()),
                allow_private_network: Some(true),
                max_age: Some(3600),
            }),
        },
    }
}

fn cors_plugin_example3() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Reflect Origin",
            Some("This example demonstrates how to configure the CORS plugin with a reflect Origin setup."),
        ),
        example: PluginDefinition::CorsPlugin {
            enabled: Some(true),
            config: Some(CorsPluginConfig {
                allow_credentials: Some(true),
                exposed_headers: Some("*".into()),
                allowed_methods: Some("GET, POST".into()),
                allowed_origin: Some("reflect".into()),
                allowed_headers: Some("*".into()),
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

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
#[schemars(example = "vrl_plugin_example_inline")]
#[schemars(example = "vrl_plugin_example_file")]
#[schemars(example = "vrl_plugin_example_headers")]
#[schemars(example = "vrl_plugin_example_shared_state")]
#[schemars(example = "vrl_plugin_example_short_circuit")]
#[schemars(example = "vrl_plugin_example_extraction")]
/// To simplify the process of extending the functionality of the GraphQL Gateway, we adopted a Rust-based script language called [VRL](https://vector.dev/docs/reference/vrl/).
///
/// VRL language is intended for writing simple scripts that can be executed in the context of the GraphQL Gateway. VRL is focused around safety and performance: the script is compiled into Rust code when the server starts, and executed as a native Rust code ([you can find a comparison between VRL and other scripting languages here](https://github.com/YassinEldeeb/rust-embedded-langs-vs-native-benchmark)).
///
/// > VRL was initially created to allow users to extend [Vector](https://vector.dev/), a high-performance observability data router, and adopted for Conductor to allow developers to extend the functionality of the GraphQL Gateway easily.
///
///
/// ### Writing VRL
///
/// VRL is an expression-oriented language. A VRL program consists entirely of expressions, with every expression returning a value. You can define variables, call functions, and use operators to manipulate values.
///
/// #### Variables and Functions
///
/// The following program defines a variable `myVar` with the value `"myValue"` and prints it to the console:
///
/// ```vrl
///
/// myVar = "my value"
///
/// log(myVar, level:"info")
///
/// ```
///
/// #### Assignment
///
/// The `.` is used to set output values. In this example, we are setting the `x-authorization` header of the upstream HTTP request to `my-value`.
///
/// Here's an example for a VRL program that extends Conductor's behavior by adding a custom HTTP header to all upstream HTTP requests:
///
/// ```vrl
///
/// .upstream_http_req.headers."x-authorization" = "my-value"
///
/// ```
///
/// #### Metadata
///
/// The `%` is used to access metadata values. Note that metadata values are read only.
///
/// The following program is printing a metadata value to the console:
///
/// ```vrl
///
/// log(%downstream_http_req.headers.authorization, level:"info")
///
/// ```
///
/// #### Further Reading
///
/// - [VRL Playground](https://playground.vrl.dev/)
///
/// - [VRL concepts documentation](https://vector.dev/docs/reference/vrl/#concepts)
///
/// - [VRL syntax documentation](https://vector.dev/docs/reference/vrl/expressions/)
///
/// - [Compiler errors documentation](https://vector.dev/docs/reference/vrl/errors/)
///
/// - [VRL program examples](https://vector.dev/docs/reference/vrl/examples/)
///
/// ### Runtime Failure Handling
///
/// Some VRL functions are fallible, meaning that they can error. Any potential errors thrown by fallible functions must be handled, a requirement enforced at compile time.
///
///
/// ```vrl
///
/// # This function is fallible, and can create errors, so it must be handled.
///
/// parsed, err = parse_json("invalid json")
///
/// ```
///
/// VRL function calls can be marked as infallible by adding a `!` suffix to the function call: (note that this might lead to runtime errors)
///
/// ```vrl
///
/// parsed = parse_json!("invalid json")
///
/// ```
///
/// > In case of a runtime error of a fallible function call, an error will be returned to the end-user, and the gateway will not continue with the execution.
///
/// ### Input/Output
///
/// #### `on_downstream_http_request`
///
/// The `on_downstream_http_request` hook is executed when a downstream HTTP request is received to the gateway from the end-user.
///
/// The following metadata inputs are available to the hook:
///
/// - `%downstream_http_req.body` (type: `string`): The body string of the incoming HTTP request.
///
/// - `%downstream_http_req.uri` (type: `string`): The URI of the incoming HTTP request.
///
/// - `%downstream_http_req.query_string` (type: `string`): The query string of the incoming HTTP request.
///
/// - `%downstream_http_req.method` (type: `string`): The HTTP method of the incoming HTTP request.
///
/// - `%downstream_http_req.headers` (type: `object`): The HTTP headers of the incoming HTTP request.
///
/// The following output values are available to the hook:
///
/// - `.graphql.operation` (type: `string`): The GraphQL operation string to be executed. If this value is set, the gateway will skip the lookup phase, and will use this GraphQL operation instead.
///
/// - `.graphql.operation_name` (type: `string`): If multiple GraphQL operations are set in `.graphql.operation`, you can specify the executable operation by setting this value.
///
/// - `.graphql.variables` (type: `object`): The GraphQL variables to be used when executing the GraphQL operation.
///
/// - `.graphql.extensions` (type: `object`): The GraphQL extensions to be used when executing the GraphQL operation.
///
///
/// #### `on_downstream_graphql_request`
///
/// The `on_downstream_graphql_request` hook is executed when a GraphQL operation is extracted from a downstream HTTP request, and before the upstream GraphQL request is sent.
///
/// The following metadata inputs are available to the hook:
///
/// - `%downstream_graphql_req.operation` (type: `string`): The GraphQL operation string, as extracted from the incoming HTTP request.
///
/// - `%downstream_graphql_req.operation_name`(type: `string`) : If multiple GraphQL operations are set in `%downstream_graphql_req.operation`, you can specify the executable operation by setting this value.
///
/// - `%downstream_graphql_req.variables` (type: `object`): The GraphQL variables, as extracted from the incoming HTTP request.
///
/// - `%downstream_graphql_req.extensions` (type: `object`): The GraphQL extensions, as extracted from the incoming HTTP request.
///
/// The following output values are available to the hook:
///
/// - `.graphql.operation` (type: `string`): The GraphQL operation string to be executed. If this value is set, it will override the existing operation.
///
/// - `.graphql.operation_name` (type: `string`): If multiple GraphQL operations are set in `.graphql.operation`, you can override the extracted value by setting this field.
///
/// - `%downstream_graphql_req.variables` (type: `object`): The GraphQL variables, as extracted from the incoming HTTP request. Setting this value will override the existing variables.
///
/// - `%downstream_graphql_req.extensions` (type: `object`): The GraphQL extensions, as extracted from the incoming HTTP request. Setting this value will override the existing extensions.
///
/// #### `on_upstream_http_request`
///
/// The `on_upstream_http_request` hook is executed when an HTTP request is about to be sent to the upstream GraphQL server.
///
/// The following metadata inputs are available to the hook:
///
/// - `%upstream_http_req.body` (type: `string`): The body string of the planned HTTP request.
///
/// - `%upstream_http_req.uri` (type: `string`): The URI of the planned HTTP request.
///
/// - `%upstream_http_req.query_string` (type: `string`): The query string of the planned HTTP request.
///
/// - `%upstream_http_req.method` (type: `string`): The HTTP method of the planned HTTP request.
///
/// - `%upstream_http_req.headers` (type: `object`): The HTTP headers of the planned HTTP request.
///
/// The following output values are available to the hook:
///
/// - `.upstream_http_req.body` (type: `string`): The body string of the planned HTTP request. Setting this value will override the existing body.
///
/// - `.upstream_http_req.uri` (type: `string`): The URI of the planned HTTP request. Setting this value will override the existing URI.
///
/// - `.upstream_http_req.query_string` (type: `string`): The query string of the planned HTTP request. Setting this value will override the existing query string.
///
/// - `.upstream_http_req.method` (type: `string`): The HTTP method of the planned HTTP request. Setting this value will override the existing HTTP method.
///
/// - `.upstream_http_req.headers` (type: `object`): The HTTP headers of the planned HTTP request. Headers set here will only extend the existing headers. You can use `null` value if you wish to remove an existing header.
///
/// #### `on_downstream_http_response`
///
/// The `on_downstream_http_response` hook is executed when a GraphQL response is received from the upstream GraphQL server, and before the response is sent to the end-user.
///
/// The following metadata inputs are available to the hook:
///
/// - `%downstream_http_res.body` (type: `string`): The body string of the HTTP response.
///
/// - `%downstream_http_res.status` (type: `number`): The status code of the HTTP response.
///
/// - `%downstream_http_res.headers` (type: `object`): The HTTP headers of the HTTP response.
///
/// The following output values are available to the hook:
///
/// - `.downstream_http_res.body` (type: `string`): The body string of the HTTP response. Setting this value will override the existing body.
///
/// - `.downstream_http_res.status` (type: `number`): The status code of the HTTP response. Setting this value will override the existing status code.
///
/// - `.downstream_http_res.headers` (type: `object`): The HTTP headers of the HTTP response. Headers set here will only extend the existing headers. You can use `null` value if you wish to remove an existing header.
///
/// ### Shared State
///
/// During the execution of VRL programs, Conductor configures a shared state object for every incoming HTTP request.
///
/// This means that you can create type-safe shared state objects, and use them to share data between different VRL programs and hooks.
///
/// You can find an example for this in the **Examples** section below.
///
/// ### Available Functions
///
pub struct VrlPluginConfig {
    /// A hook executed when a downstream HTTP request is received to the gateway from the end-user.
    /// This hook allow you to extract information from the request, for later use, or to reject a request quickly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_downstream_http_request: Option<VrlConfigReference>,
    /// A hook executed when a GraphQL query is extracted from a downstream HTTP request, and before the upstream GraphQL request is sent.
    /// This hooks allow you to easily manipulate the incoming GraphQL request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_downstream_graphql_request: Option<VrlConfigReference>,
    /// A hook executed when an HTTP request is about to be sent to the upstream GraphQL server.
    /// This hook allow you to manipulate upstream HTTP calls easily.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_upstream_http_request: Option<VrlConfigReference>,
    /// A hook executed when a GraphQL response is received from the upstream GraphQL server, and before the response is sent to the end-user.
    /// This hook allow you to manipulate the end-user response easily.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_downstream_http_response: Option<VrlConfigReference>,
}

fn vrl_plugin_example_inline() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Inline",
            Some("Load and execute VRL plugins using inline configuration."),
        ),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_upstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#".upstream_http_req.headers."x-authorization" = "some-value"
                    "#
                    .to_string(),
                }),
                ..Default::default()
            },
        },
    }
}

fn vrl_plugin_example_file() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "File",
            Some("Load and execute VRL plugins using an external '.vrl' file."),
        ),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_upstream_http_request: Some(VrlConfigReference::File {
                    path: LocalFileReference {
                        contents: "".to_string(),
                        path: "my_plugin.vrl".to_string(),
                    },
                }),
                ..Default::default()
            },
        },
    }
}

fn vrl_plugin_example_shared_state() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Shared State",
            Some("The following example is configuring a variable, and use it later"),
        ),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_downstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#"authorization_header = %downstream_http_req.headers.authorization
                    "#
                    .to_string(),
                }),
                on_upstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#".upstream_http_req.headers."x-auth" = authorization_header
                    "#
                    .to_string(),
                }),
                ..Default::default()
            },
        },
    }
}

fn vrl_plugin_example_extraction() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Custom GraphQL Extraction",
            Some("The following example is using a custom GraphQL extraction, overriding the default gateway behavior. In this example, we parse the incoming body as JSON and use the parsed value to find the GraphQL operation. Assuming the body structure is: `{ \"runThisQuery\": \"query { __typename }\", \"variables\": {  }`."),
        ),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_downstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#"parsed_body = parse_json!(%downstream_http_req.body)
.graphql.operation = parsed_body.runThisQuery
.graphql.variables = parsed_body.variables
                    "#
                    .to_string(),
                }),
                ..Default::default()
            },
        },
    }
}

fn vrl_plugin_example_short_circuit() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Short Circuit",
            Some("The following example rejects all incoming requests that doesn't have the \"authorization\" header set."),
        ),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_downstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#"if %downstream_http_req.headers.authorization == null {
  short_circuit!(403, "Missing authorization header")
}
                    "#
                    .to_string(),
                }),
                ..Default::default()
            },
        },
    }
}

fn vrl_plugin_example_headers() -> JsonSchemaExample<PluginDefinition> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Headers Passthrough", Some("This example is using the shared-state feature to store the headers from the incoming HTTP request, and it pass it through to upstream calls.")),
        example: PluginDefinition::VrlPluginConfig {
            enabled: Default::default(),
            config: VrlPluginConfig {
                on_downstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#"incoming_headers = %downstream_http_req.headers
                    "#
                    .to_string(),
                }),
                on_upstream_http_request: Some(VrlConfigReference::Inline {
                    content: r#".upstream_http_req.headers = incoming_headers
                    "#
                    .to_string(),
                }),
                ..Default::default()
            },
        },
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "from")]
pub enum VrlConfigReference {
    #[serde(rename = "inline")]
    #[schemars(title = "inline")]
    /// Inline string for a VRL code snippet. The string is parsed and executed as a VRL plugin.
    Inline { content: String },
    #[serde(rename = "file")]
    #[schemars(title = "file")]
    /// File reference to a VRL file. The file is loaded and executed as a VRL plugin.
    File { path: LocalFileReference },
}

impl VrlConfigReference {
    pub fn contents(&self) -> &String {
        match self {
            VrlConfigReference::Inline { content } => content,
            VrlConfigReference::File { path } => &path.contents,
        }
    }
}
