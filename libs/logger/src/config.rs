use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, PartialEq)]
/// A source definition for a GraphQL endpoint or a federated GraphQL implementation.
pub enum LoggerConfigFormat {
  /// This logging format outputs minimal, compact logs. It focuses on the essential parts of the log message and its fields, making it suitable for production environments where performance and log size are crucial.
  ///
  /// Pros:
  ///
  ///   - Efficient in terms of space and performance.
  ///
  ///   - Easy to read for brief messages and simple logs.
  ///
  /// Cons:
  ///
  ///   - May lack detailed context, making debugging a bit more challenging.
  #[serde(rename = "compact")]
  #[schemars(title = "compact")]
  Compact,

  /// The pretty format is designed for enhanced readability, featuring more verbose output including well-formatted fields and context. Ideal for development and debugging purposes.
  ///
  /// Pros:
  ///
  ///   - Highly readable and provides detailed context.
  ///
  ///   - Easier to understand complex log messages.
  ///
  /// Cons:
  ///
  ///   - More verbose, resulting in larger log sizes.
  ///
  ///   - Potentially slower performance due to the additional formatting overhead.
  #[serde(rename = "pretty")]
  #[schemars(title = "pretty")]
  Pretty,

  /// This format outputs logs in JSON. It is particularly useful when integrating with tools that consume or process JSON logs, such as log aggregators and analysis systems.
  ///
  /// Pros:
  ///
  ///   - Structured format makes it easy to parse and integrate with various tools.
  ///
  ///   - Consistent and predictable output.
  ///
  /// Cons:
  ///
  ///   - Can be verbose and harder to read directly by developers.
  ///
  ///   - Slightly more overhead compared to simpler formats like compact.
  #[serde(rename = "json")]
  #[schemars(title = "json")]
  Json,
}

impl Default for LoggerConfigFormat {
  // In development, we wish to see some more details and code locations.
  #[cfg(debug_assertions)]
  fn default() -> Self {
    LoggerConfigFormat::Pretty
  }

  #[cfg(not(debug_assertions))]
  fn default() -> Self {
    if atty::is(atty::Stream::Stdout) {
      LoggerConfigFormat::Compact
    } else {
      LoggerConfigFormat::Json
    }
  }
}
