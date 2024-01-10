use conductor_common::serde_utils::{
  JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType, LocalFileReference,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct ApolloPersistedQueryManifest {
  pub format: String,
  pub version: i32,
  pub operations: Vec<ApolloPersistedQueryManifestRecord>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApolloPersistedQueryManifestRecord {
  pub id: String,
  pub body: String,
  pub name: String,
  #[serde(rename = "type")]
  pub operation_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "trusted_documents_example_1")]
#[schemars(example = "trusted_documents_example_2")]
pub struct TrustedDocumentsPluginConfig {
  /// The store defines the source of trusted documents.
  /// The store contents is a list of hashes and GraphQL documents that are allowed to be executed.
  pub store: TrustedDocumentsPluginStoreConfig,
  /// A list of protocols to be exposed by this plugin. Each protocol defines how to obtain the document ID from the incoming request.
  /// You can specify multiple kinds of protocols, if needed.
  pub protocols: Vec<TrustedDocumentsProtocolConfig>,
  /// By default, this plugin does not allow untrusted operations to be executed.
  /// This is a security measure to prevent accidental exposure of operations that are not persisted.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_untrusted: Option<bool>,
}

fn trusted_documents_example_1() -> JsonSchemaExample<TrustedDocumentsPluginConfig> {
  JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Local File Store", Some("This example is using a local file called `trusted_documents.json` as a store, using the Key->Value map format. The protocol exposed is based on HTTP `POST`, using the `documentId` parameter from the request body.")),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "trusted_documents".to_string(),
        }),
        example: TrustedDocumentsPluginConfig {
            store: TrustedDocumentsPluginStoreConfig::File {
                file: LocalFileReference {
                    path: "trusted_documents.json".to_string(),
                    contents: "".to_string(),
                },
                format: TrustedDocumentsFileFormat::JsonKeyValue,
            },
            allow_untrusted: None,
            protocols: vec![TrustedDocumentsProtocolConfig::DocumentId {
                field_name: "documentId".to_string(),
            }],
        }
    }
}

fn trusted_documents_example_2() -> JsonSchemaExample<TrustedDocumentsPluginConfig> {
  JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("HTTP GET", Some("This example uses a local file store called `trusted_documents.json`, using the Key->Value map format. The protocol exposed is based on HTTP `GET`, and extracts all parameters from the query string.")),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "trusted_documents".to_string(),
        }),
        example: TrustedDocumentsPluginConfig {
            store: TrustedDocumentsPluginStoreConfig::File {
                file: LocalFileReference {
                    path: "trusted_documents.json".to_string(),
                    contents: "".to_string(),
                },
                format: TrustedDocumentsFileFormat::JsonKeyValue,
            },
            allow_untrusted: None,
            protocols: vec![TrustedDocumentsProtocolConfig::HttpGet {
                document_id_from: TrustedDocumentHttpGetParameterLocation::document_id_default(),
                variables_from: TrustedDocumentHttpGetParameterLocation::variables_default(),
                operation_name_from:
                    TrustedDocumentHttpGetParameterLocation::operation_name_default(),
            }],
        },
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum TrustedDocumentsPluginStoreConfig {
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
    format: TrustedDocumentsFileFormat,
  },
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum TrustedDocumentsProtocolConfig {
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
    #[serde(default = "TrustedDocumentHttpGetParameterLocation::document_id_default")]
    document_id_from: TrustedDocumentHttpGetParameterLocation,
    /// Instructions for fetching the variables parameter from the incoming HTTP request.
    /// GraphQL variables must be passed as a JSON-encoded string.
    #[serde(default = "TrustedDocumentHttpGetParameterLocation::variables_default")]
    variables_from: TrustedDocumentHttpGetParameterLocation,
    /// Instructions for fetching the operationName parameter from the incoming HTTP request.
    #[serde(default = "TrustedDocumentHttpGetParameterLocation::operation_name_default")]
    operation_name_from: TrustedDocumentHttpGetParameterLocation,
  },
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum TrustedDocumentHttpGetParameterLocation {
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

impl TrustedDocumentHttpGetParameterLocation {
  pub fn document_id_default() -> Self {
    TrustedDocumentHttpGetParameterLocation::Query {
      name: document_id_default_field_name(),
    }
  }

  pub fn variables_default() -> Self {
    TrustedDocumentHttpGetParameterLocation::Query {
      name: "variables".to_string(),
    }
  }

  pub fn operation_name_default() -> Self {
    TrustedDocumentHttpGetParameterLocation::Query {
      name: "operationName".to_string(),
    }
  }
}

fn document_id_default_field_name() -> String {
  "documentId".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub enum TrustedDocumentsFileFormat {
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
