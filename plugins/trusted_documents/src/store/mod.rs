use std::fmt::Debug;

pub mod fs;

#[async_trait::async_trait(?Send)]
pub trait TrustedDocumentsStore: Sync + Send + Debug {
  async fn has_document(&self, hash: &str) -> bool;
  async fn get_document(&self, hash: &str) -> Option<&String>;
}
