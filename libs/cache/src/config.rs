use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct CachePluginConfig {
  #[serde(rename = "cache")]
  pub store_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum CacheStoreConfig {
  #[serde(rename = "redis")]
  Redis { id: String, config: RedisConfig },
  #[serde(rename = "in_memory")]
  InMemory { id: String, config: InMemoryConfig },
  #[serde(rename = "cloudflare_kv")]
  CloudflareKV {
    id: String,
    config: CloudflareKVConfig,
  },
}

/// Configuration for Redis.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct RedisConfig {
  /// Connection string to the Redis server (e.g: "redis://localhost:6379").
  pub connection_string: String,

  /// Redis database number. Default is 0.
  #[serde(default = "redis_default_database")]
  pub database: Option<i64>,

  /// Time-to-live for cache entries in seconds. Default is 600 seconds (10 minutes).
  #[serde(default = "redis_default_cache_ttl_seconds")]
  pub cache_ttl_seconds: Option<u64>,

  /// The maximum number of connections in the Redis connection pool. Default is 10.
  #[serde(default = "redis_default_pool_size")]
  pub pool_size: Option<u32>,

  /// Timeout for acquiring a connection from the pool in seconds. Default is 5 seconds.
  #[serde(default = "redis_default_pool_timeout")]
  pub pool_timeout: Option<u64>,
  // Yassin: Don't think we need those now, they will overcomplicate things unncessarily
  // /// Master name for Redis Sentinel. No default (None).
  // pub sentinel_master: Option<String>,

  // /// Addresses for Redis Sentinel nodes. No default (None).
  // pub sentinel_addresses: Option<Vec<String>>,

  // /// Enable cluster support. Default is false.
  // #[serde(default = "redis_default_cluster_enabled")]
  // pub cluster_enabled: Option<bool>,

  // /// Addresses for Redis Cluster nodes. No default (None).
  // pub cluster_nodes: Option<Vec<String>>,
}

// Default functions
fn redis_default_database() -> Option<i64> {
  Some(0)
}
fn redis_default_cache_ttl_seconds() -> Option<u64> {
  Some(600)
}
fn redis_default_pool_size() -> Option<u32> {
  Some(10)
}
fn redis_default_pool_timeout() -> Option<u64> {
  Some(5)
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct CloudflareKVConfig {
  /// The Cloudflare account identifier.
  pub account_identifier: String,

  /// The namespace identifier for the KV store.
  pub namespace_identifier: String,

  /// API token for authenticating with the Cloudflare API.
  pub api_token: String,

  /// Optional: Override the default TTL (time-to-live) for cache entries.
  /// If not provided, Cloudflare's default TTL will be used.
  #[serde(default)]
  pub cache_ttl_seconds: Option<u64>,

  /// Optional: Specifies the connection timeout in seconds.
  /// Default: 30 seconds.
  #[serde(default = "default_connection_timeout")]
  pub connection_timeout_seconds: Option<u64>,
}

// Default function for connection timeout
fn default_connection_timeout() -> Option<u64> {
  Some(30)
}

/// Configuration for In-Memory caching, it internally works using an LRU (Least Recently Used) eviction policy
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct InMemoryConfig {
  /// The maximum number of cache entries. Default is 1000 entries.
  /// When the cache reaches this size, it will start evicting entries
  /// based on the eviction policy.
  #[serde(default = "in_memory_default_max_size")]
  pub max_size: Option<usize>,

  /// Time-to-live for cache entries in seconds. Default is 600 seconds (10 minutes).
  /// This is the duration after which a cache entry will be automatically removed.
  #[serde(default = "in_memory_default_cache_ttl_seconds")]
  pub cache_ttl_seconds: Option<u64>,
}

// Default functions
fn in_memory_default_max_size() -> Option<usize> {
  Some(1000)
}

fn in_memory_default_cache_ttl_seconds() -> Option<u64> {
  Some(600)
}
