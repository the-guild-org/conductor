// https://opentelemetry.io/docs/specs/semconv/http/http-spans/
pub const HTTP_METHOD: &str = "http.method";
pub const HTTP_SCHEME: &str = "http.scheme";
pub const HTTP_HOST: &str = "http.host";
pub const HTTP_URL: &str = "http.url";
pub const HTTP_ROUTE: &str = "http.route";
pub const HTTP_FLAVOR: &str = "http.flavor";
pub const HTTP_CLIENT_IP: &str = "http.client_ip";
pub const NET_HOST_PORT: &str = "net.host.port";
pub const OTEL_KIND: &str = "otel.kind";
pub const SPAN_KIND: &str = "span.kind";
pub const SPAN_TYPE: &str = "span.type";
pub const HTTP_STATUS_CODE: &str = "http.status_code";
pub const HTTP_USER_AGENT: &str = "http.user_agent";
pub const HTTP_TARGET: &str = "http.target";
pub const REQUEST_ID: &str = "request_id";
pub const TRACE_ID: &str = "trace_id";

// https://opentelemetry.io/docs/specs/semconv/attributes-registry/error/
pub const ERROR_TYPE: &str = "error.type";
pub const ERROR_MESSAGE: &str = "error.message";
pub const ERROR_CAUSE_CHAIN: &str = "error.cause_chain";
pub const OTEL_STATUS_CODE: &str = "otel.status_code"; // "ERROR" / "OK"
pub const EXCEPTION_MESSAGE: &str = "exception.message";
pub const EXCEPTION_DETAILS: &str = "exception.message";

// Specific to Jaeger
pub const ERROR_INDICATOR: &str = "error"; // "true"

// https://opentelemetry.io/docs/specs/semconv/database/graphql/
pub const GRAPHQL_DOCUMENT: &str = "graphql.document";
pub const GRAPHQL_OPERATION_TYPE: &str = "graphql.operation.type";
pub const GRAPHQL_OPERATION_NAME: &str = "graphql.operation.name";
pub const GRAPHQL_ERROR_COUNT: &str = "graphql.error.count";

// Conductor-specific
pub const CONDUCTOR_ENDPOINT: &str = "conductor.endpoint";
pub const CONDUCTOR_SOURCE: &str = "conductor.source";
