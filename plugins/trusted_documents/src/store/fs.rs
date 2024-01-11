use crate::config::TrustedDocumentsFileFormat;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::ApolloPersistedQueryManifest;

use super::TrustedDocumentsStore;

#[derive(Debug)]
pub struct TrustedDocumentsFilesystemStore {
  known_documents: HashMap<String, String>,
}

#[async_trait::async_trait(?Send)]
impl TrustedDocumentsStore for TrustedDocumentsFilesystemStore {
  async fn has_document(&self, hash: &str) -> bool {
    self.known_documents.contains_key(hash)
  }

  async fn get_document(&self, hash: &str) -> Option<&String> {
    self.known_documents.get(hash)
  }
}

impl TrustedDocumentsFilesystemStore {
  pub fn new_from_file_contents(
    contents: &str,
    file_format: &TrustedDocumentsFileFormat,
  ) -> Result<Self, serde_json::Error> {
    debug!(
      "creating trusted documents store from a local FS file, the expected file format is: {:?}",
      file_format
    );

    let result = match file_format {
      TrustedDocumentsFileFormat::ApolloPersistedQueryManifest => {
        let parsed = serde_json::from_str::<ApolloPersistedQueryManifest>(contents)?;

        Self {
          known_documents: parsed
            .operations
            .into_iter()
            .fold(HashMap::new(), |mut acc, record| {
              acc.insert(record.id, record.body);
              acc
            }),
        }
      }
      TrustedDocumentsFileFormat::JsonKeyValue => Self {
        known_documents: serde_json::from_str(contents)?,
      },
    };

    info!(
      "loaded trusted documents store from file, total records: {:?}",
      result.known_documents.len()
    );

    Ok(result)
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[tokio::test]
  async fn fs_store_apollo_manifest_value() {
    // valid JSON structure with empty array
    let store_result = TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!({
          "format": "apollo",
          "version": 1,
          "operations": []
      })
      .to_string(),
      &TrustedDocumentsFileFormat::ApolloPersistedQueryManifest,
    );
    assert!(store_result.is_ok());
    if let Ok(store) = store_result {
      assert_eq!(store.known_documents.len(), 0);
    }

    // valid store mapping
    let store_result = TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!({
          "format": "apollo",
          "version": 1,
          "operations": [
              {
                  "id": "key1",
                  "body": "query test { __typename }",
                  "name": "test",
                  "type": "query"
              }
          ]
      })
      .to_string(),
      &TrustedDocumentsFileFormat::ApolloPersistedQueryManifest,
    );
    assert!(store_result.is_ok());
    if let Ok(store) = store_result {
      assert_eq!(store.known_documents.len(), 1);
      assert!(store.has_document("key1").await);
      assert_eq!(
        store.get_document("key1").await.cloned(),
        Some("query test { __typename }".to_string())
      );
    }

    // Invalid JSON
    assert!(TrustedDocumentsFilesystemStore::new_from_file_contents(
      "{",
      &TrustedDocumentsFileFormat::ApolloPersistedQueryManifest,
    )
    .is_err());

    // invalid JSON structure
    assert!(TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!({}).to_string(),
      &TrustedDocumentsFileFormat::ApolloPersistedQueryManifest,
    )
    .is_err());
  }

  #[tokio::test]
  async fn fs_store_json_key_value() {
    // Valid empty JSON map
    let store_result = TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!({}).to_string(),
      &TrustedDocumentsFileFormat::JsonKeyValue,
    );
    assert!(store_result.is_ok());
    if let Ok(store) = store_result {
      assert_eq!(store.known_documents.len(), 0);
    }

    // Valid JSON map
    let store_result = TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!({
          "key1": "query { __typename }"
      })
      .to_string(),
      &TrustedDocumentsFileFormat::JsonKeyValue,
    );
    assert!(store_result.is_ok());
    if let Ok(store) = store_result {
      assert_eq!(store.known_documents.len(), 1);
    }

    // Invalid object structure
    assert!(TrustedDocumentsFilesystemStore::new_from_file_contents(
      &serde_json::json!([]).to_string(),
      &TrustedDocumentsFileFormat::JsonKeyValue,
    )
    .is_err());

    // Invalid JSON
    assert!(TrustedDocumentsFilesystemStore::new_from_file_contents(
      "{",
      &TrustedDocumentsFileFormat::JsonKeyValue,
    )
    .is_err());
  }
}
