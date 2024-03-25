use cached::{Cached, TimedSizedCache};
use serde_json::Value;

use crate::{cache_manager::CacheStore, config::InMemoryConfig};

#[derive(Debug)]
pub struct InMemoryCacheStore {
  pub id: String,
  pub cache: TimedSizedCache<String, Value>,
}

impl InMemoryCacheStore {
  pub fn new(id: String, config: InMemoryConfig) -> Self {
    let cache = TimedSizedCache::with_size_and_lifespan(
      config.max_size.unwrap_or(1000),
      config.cache_ttl_seconds.unwrap_or(600),
    );

    InMemoryCacheStore { id, cache }
  }
}

#[async_trait::async_trait(?Send)]
impl CacheStore for InMemoryCacheStore {
  async fn get(&mut self, key: &str) -> Option<Value> {
    self.cache.cache_get(key).cloned()
  }

  async fn set(&mut self, key: String, response: Value) {
    self.cache.cache_set(key, response);
  }

  fn id(&self) -> &str {
    &self.id
  }
}
