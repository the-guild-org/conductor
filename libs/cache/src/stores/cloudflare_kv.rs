use std::sync::{Arc, Mutex};

use crate::{cache_manager::CacheStore, config::CloudflareKVConfig};
use serde_json::Value;
use worker::kv::{KvError, KvStore};

pub struct CloudflareKVCacheStore {
  pub id: String,
  kv_store: Arc<Mutex<KvStore>>,
  cache_ttl_seconds: Option<u64>,
}

impl std::fmt::Debug for CloudflareKVCacheStore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CloudflareKVCacheStore")
      .field("id", &self.id)
      .finish()
  }
}

impl CloudflareKVCacheStore {
  pub fn new(id: String, config: CloudflareKVConfig) -> Result<Self, KvError> {
    let kv_store = KvStore::create(&config.namespace_identifier)?;

    Ok(CloudflareKVCacheStore {
      id,
      kv_store: Arc::new(Mutex::new(kv_store)),
      cache_ttl_seconds: config.cache_ttl_seconds,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl CacheStore for CloudflareKVCacheStore {
  async fn get(&mut self, key: &str) -> Option<Value> {
    let kv_store = self.kv_store.lock().unwrap();
    match kv_store.get(key).text().await {
      Ok(Some(data)) => serde_json::from_str(&data).ok(),
      _ => None,
    }
  }

  async fn set(&mut self, key: String, response: Value) {
    let response_str = serde_json::to_string(&response).ok().unwrap();
    let ttl = self.cache_ttl_seconds.unwrap_or(3600);

    let kv_store = self.kv_store.lock().unwrap();
    let _ = kv_store
      .put(&key, &response_str)
      .expect("Failed to create put request")
      .expiration_ttl(ttl)
      .execute()
      .await
      .unwrap();
  }

  fn id(&self) -> &str {
    &self.id
  }
}
