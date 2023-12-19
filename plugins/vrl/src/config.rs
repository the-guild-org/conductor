use conductor_common::{
    serde_utils::{
        JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType,
        LocalFileReference,
    },
    vrl_utils::VrlConfigReference,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

fn vrl_plugin_example_inline() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Inline",
            Some("Load and execute VRL plugins using inline configuration."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
            on_upstream_http_request: Some(VrlConfigReference::Inline {
                content: r#".upstream_http_req.headers."x-authorization" = "some-value"
                "#
                .to_string(),
            }),
            ..Default::default()
        },
    }
}

fn vrl_plugin_example_file() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "File",
            Some("Load and execute VRL plugins using an external '.vrl' file."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
            on_upstream_http_request: Some(VrlConfigReference::File {
                path: LocalFileReference {
                    contents: "".to_string(),
                    path: "my_plugin.vrl".to_string(),
                },
            }),
            ..Default::default()
        },
    }
}

fn vrl_plugin_example_shared_state() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Shared State",
            Some("The following example is configuring a variable, and use it later"),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
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
    }
}

fn vrl_plugin_example_extraction() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Custom GraphQL Extraction",
            Some("The following example is using a custom GraphQL extraction, overriding the default gateway behavior. In this example, we parse the incoming body as JSON and use the parsed value to find the GraphQL operation. Assuming the body structure is: `{ \"runThisQuery\": \"query { __typename }\", \"variables\": {  }`."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: r#"parsed_body = parse_json!(%downstream_http_req.body)
.graphql.operation = parsed_body.runThisQuery
.graphql.variables = parsed_body.variables
                "#
                .to_string(),
            }),
            ..Default::default()
        }
    }
}

fn vrl_plugin_example_short_circuit() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Short Circuit",
            Some("The following example rejects all incoming requests that doesn't have the \"authorization\" header set."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: r#"if %downstream_http_req.headers.authorization == null {
short_circuit!(403, "Missing authorization header")
}
                "#
                .to_string(),
            }),
            ..Default::default()
        },
    }
}

fn vrl_plugin_example_headers() -> JsonSchemaExample<VrlPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Headers Passthrough", Some("This example is using the shared-state feature to store the headers from the incoming HTTP request, and it pass it through to upstream calls.")),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "vrl".to_string(),
        }),
        example: VrlPluginConfig {
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
        }
    }
}
