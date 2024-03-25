use crate::cache_manager::CacheStore;
use crate::config::RedisConfig;
use r2d2::{Pool, PooledConnection};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use redis::RedisError;
use serde_json::{self, Value};
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub struct RedisCacheStore {
  pub id: String,
  pool: Pool<RedisConnectionManager>,
  cache_ttl_seconds: Option<u64>,
}

impl RedisCacheStore {
  pub fn new(id: String, config: RedisConfig) -> Result<Self, RedisError> {
    let manager = RedisConnectionManager::new(config.connection_string).unwrap();

    let mut builder = Pool::builder();
    if let Some(pool_size) = config.pool_size {
      builder = builder.max_size(pool_size);
    }
    if let Some(pool_timeout) = config.pool_timeout {
      builder = builder.connection_timeout(Duration::from_secs(pool_timeout));
    }

    let pool = builder.build(manager).unwrap();

    Ok(RedisCacheStore {
      id,
      pool,
      cache_ttl_seconds: config.cache_ttl_seconds,
    })
  }

  fn get_con(&self) -> Result<PooledConnection<RedisConnectionManager>, Box<dyn Error>> {
    self.pool.get().map_err(|e| e.into())
  }
}

#[async_trait::async_trait(?Send)]
impl CacheStore for RedisCacheStore {
  async fn get(&mut self, key: &str) -> Option<Value> {
    let mut con = match self.get_con() {
      Ok(con) => con,
      Err(_) => return None,
    };

    let value: String = match con.get(key) {
      Ok(val) => val,
      Err(_) => return None,
    };

    serde_json::from_str(&value).ok()
  }

  async fn set(&mut self, key: String, response: Value) {
    let mut con = match self.get_con() {
      Ok(con) => con,
      Err(_) => return,
    };

    let response_str = match serde_json::to_string(&response) {
      Ok(str) => str,
      Err(_) => return,
    };

    let ttl_seconds = self
      .cache_ttl_seconds
      .expect("Couldn't find a default value for cache_ttl_seconds");
    let _: Result<(), _> = con.set_ex(key, response_str, ttl_seconds as usize);
  }

  fn id(&self) -> &str {
    &self.id
  }
}
