use serde::Deserialize;

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
