// https://opentelemetry.io/docs/specs/semconv/http/http-spans/
pub const HTTP_METHOD: &str = "http.method";
pub const HTTP_SCHEME: &str = "http.scheme";
pub const HTTP_HOST: &str = "http.host";
pub const HTTP_URL: &str = "http.url";
pub const NET_HOST_PORT: &str = "net.host.port";
pub const OTEL_KIND: &str = "otel.kind";
pub const OTEL_NAME: &str = "otel.name";
pub const SPAN_KIND: &str = "span.kind";
pub const HTTP_STATUS_CODE: &str = "http.status_code";
pub const HTTP_USER_AGENT: &str = "http.user_agent";

// https://opentelemetry.io/docs/specs/semconv/attributes-registry/error/
pub const ERROR_TYPE: &str = "error.type";
pub const ERROR_MESSAGE: &str = "error.message";
pub const ERROR_CAUSE_CHAIN: &str = "error.cause_chain";
pub const OTEL_STATUS_CODE: &str = "otel.status_code"; // "ERROR"

// Specific to Jaeger
pub const ERROR_INDICATOR: &str = "error"; // "true"

// https://opentelemetry.io/docs/specs/semconv/database/graphql/
pub const GRAPHQL_DOCUMENT: &str = "graphql.document";
pub const GRAPHQL_OPERATION_TYPE: &str = "graphql.operation.type";
pub const GRAPHQL_OPERATION_NAME: &str = "graphql.operation.name";
pub const GRAPHQL_ERROR_COUNT: &str = "graphql.error.count";
