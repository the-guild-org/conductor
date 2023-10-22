use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::serde_utils::LocalFileReference;

use super::store::fs::PersistedDocumentsFileFormat;

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct PersistedOperationsPluginConfig {
    pub store: PersistedOperationsPluginStoreConfig,
    pub allow_non_persisted: Option<bool>,
    pub protocols: Vec<PersistedOperationsProtocolConfig>,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationsPluginStoreConfig {
    #[serde(rename = "file")]
    File {
        #[serde(rename = "path")]
        file: LocalFileReference,
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
    #[serde(rename = "apollo_manifest_extensions")]
    ApolloManifestExtensions,
    #[serde(rename = "document_id")]
    DocumentId {
        #[serde(default = "document_id_default_field_name")]
        field_name: String,
    },
    #[serde(rename = "http_get")]
    HttpGet {
        #[serde(default = "PersistedOperationHttpGetParameterLocation::document_id_default")]
        document_id_from: PersistedOperationHttpGetParameterLocation,
        #[serde(default = "PersistedOperationHttpGetParameterLocation::variables_default")]
        variables_from: PersistedOperationHttpGetParameterLocation,
        #[serde(default = "PersistedOperationHttpGetParameterLocation::operation_name_default")]
        operation_name_from: PersistedOperationHttpGetParameterLocation,
    },
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source")]
pub enum PersistedOperationHttpGetParameterLocation {
    // TODO: This doesn't work when parsed from config
    #[serde(rename = "search_query")]
    Query { name: String },
    #[serde(rename = "path")]
    Path { position: usize },
    #[serde(rename = "header")]
    Header { name: String },
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
