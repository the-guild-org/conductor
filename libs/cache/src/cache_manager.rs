use crate::config::CacheStoreConfig;
use crate::stores::in_memory::InMemoryCacheStore;
use crate::stores::redis::RedisCacheStore;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

#[async_trait::async_trait(?Send)]
pub trait CacheStore: std::fmt::Debug {
  async fn get(&mut self, key: &str) -> Option<Value>;
  async fn set(&mut self, key: String, value: Value);

  fn id(&self) -> &str;
}

#[derive(Clone)]
pub struct CacheManager {
  stores: HashMap<String, Arc<Mutex<Box<dyn CacheStore + Send + Sync>>>>,
}

impl fmt::Debug for CacheManager {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "CacheManager",)
  }
}

impl CacheManager {
  pub fn new(config: Vec<CacheStoreConfig>) -> Self {
    let mut stores = CacheManager {
      stores: HashMap::new(),
    };

    for i in config {
      match i {
        CacheStoreConfig::InMemory { id, config } => {
          let cache = InMemoryCacheStore::new(id.clone(), config);

          stores
            .stores
            .insert(id.clone(), Arc::new(Mutex::new(Box::new(cache))));
        }
        CacheStoreConfig::Redis { id, config } => match RedisCacheStore::new(id.clone(), config) {
          Ok(cache) => {
            stores
              .stores
              .insert(id.clone(), Arc::new(Mutex::new(Box::new(cache))));
          }
          Err(e) => eprintln!("Failed to create Redis cache store: {}", e),
        },
        // CacheConfig::CloudflareKV { id, config } => match CloudflareKVCacheStore::new(id, config) {
        //   Ok(cache) => stores.caches.lock().unwrap().push(Box::new(cache)),
        //   Err(e) => eprintln!("Failed to create Cloudflare KV cache store: {}", e),
        // },
        _ => {}
      }
    }

    stores
  }

  pub fn get_store<T>(&self, store_id: &str) -> Option<CacheStoreProxy<T>>
  where
    T: DeserializeOwned + Serialize,
  {
    for (id, store) in self.stores.iter() {
      if id == store_id {
        return Some(CacheStoreProxy::new(store.clone()));
      }
    }

    None
  }
}

#[derive(Debug)]
pub struct CacheStoreProxy<T>
where
  T: DeserializeOwned + Serialize,
{
  store: Arc<Mutex<Box<dyn CacheStore + Send + Sync>>>,
  _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> CacheStoreProxy<T> {
  pub fn new(store: Arc<Mutex<Box<dyn CacheStore + Send + Sync>>>) -> Self {
    CacheStoreProxy {
      store,
      _phantom: std::marker::PhantomData,
    }
  }

  pub async fn get(&self, key: &str) -> Option<T> {
    if let Ok(mut store) = self.store.lock() {
      store
        .get(key)
        .await
        .map(|v| serde_json::from_value(v).unwrap())
    } else {
      None
    }
  }

  pub async fn set(&mut self, key: String, value: T) {
    if let Ok(mut store) = self.store.lock() {
      store.set(key, serde_json::to_value(value).unwrap()).await;
    }
  }
}
