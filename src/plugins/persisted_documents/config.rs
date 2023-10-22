use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::serde_utils::LocalFileReference;

use super::store::fs::PersistedDocumentsFileFormat;

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct PersistedOperationsPluginConfig {
    /// The store defines the source of persisted documents.
    pub store: PersistedOperationsPluginStoreConfig,
    /// By default, enabling this plugin does not allow non-persisted operations to be executed.
    /// This is a security measure to prevent accidental exposure of operations that are not persisted.
    pub allow_non_persisted: Option<bool>,
    /// A list of protocols to be used to execute persisted operations.
    pub protocols: Vec<PersistedOperationsProtocolConfig>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationsPluginStoreConfig {
    #[serde(rename = "file")]
    /// File-based store configuration.
    File {
        #[serde(rename = "path")]
        /// A path to a local file on the file-system.
        file: LocalFileReference,
        /// The format and the expected structure of the loaded store file.
        format: PersistedDocumentsFileFormat,
    },
}

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

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum PersistedOperationsProtocolConfig {
    /// This protocol is based on Apollo's Persisted Query Extensions (see https://www.apollographql.com/docs/kotlin/advanced/persisted-queries/#2-publish-operation-manifest)
    /// The GraphQL operation key is sent over POST and contains "extensions" field with the GraphQL document hash.
    ///
    /// Example: POST /graphql {"extensions": {"persistedQuery": {"version": 1, "sha256Hash": "123"}}
    #[serde(rename = "apollo_manifest_extensions")]
    ApolloManifestExtensions,
    /// This protocol is based on a POST request with a JSON body containing a field with the document ID.
    /// By default, the field name is `documentId`.
    ///
    /// Example: POST /graphql {"documentId": "123", "variables": {"code": "AF"}, "operationName": "test"}
    #[serde(rename = "document_id")]
    DocumentId {
        #[serde(default = "document_id_default_field_name")]
        field_name: String,
    },
    /// This protocol is based on a GET request. You can customize where to fetch each one of the parameters from.
    /// Each request parameter can be obtained from a different source: query, path, or header.
    /// By defualt, all parameters are obtained from the query string.
    ///
    /// Example: GET /graphql?documentId=123&variables=%7B%22code%22%3A%22AF%22%7D&operationName=test
    #[serde(rename = "http_get")]
    HttpGet {
        // Instructions for fetching the document ID parameter from the incoming HTTP request.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::document_id_default")]
        document_id_from: PersistedOperationHttpGetParameterLocation,
        // Instructions for fetching the variables parameter from the incoming HTTP request.
        // GraphQL variables must be passed as a JSON-encoded string.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::variables_default")]
        variables_from: PersistedOperationHttpGetParameterLocation,
        // Instructions for fetching the operationName parameter from the incoming HTTP request.
        #[serde(default = "PersistedOperationHttpGetParameterLocation::operation_name_default")]
        operation_name_from: PersistedOperationHttpGetParameterLocation,
    },
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationHttpGetParameterLocation {
    /// The parameter is obtained from the query string of the HTTP request.
    #[serde(rename = "search_query")]
    Query {
        // The name of the query parameter.
        name: String,
    },
    /// The parameter is obtained from the path of the HTTP request.
    #[serde(rename = "path")]
    Path {
        /// The numeric value specific the location of the argument (starting from 0).
        position: usize,
    },
    /// The parameter is obtained from a header in the HTTP request.
    #[serde(rename = "header")]
    Header {
        // The name of the HTTP header.
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
