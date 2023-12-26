pub mod fs;

#[async_trait::async_trait]
pub trait PersistedDocumentsStore: Sync + Send {
  async fn has_document(&self, hash: &str) -> bool;
  async fn get_document(&self, hash: &str) -> Option<&String>;
}
