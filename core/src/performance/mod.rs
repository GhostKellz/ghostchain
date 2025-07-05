use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub mod cache;
pub mod connection_pool;
pub mod batch_processor;
pub mod metrics;
pub mod integration;

pub use cache::*;
pub use connection_pool::*;
pub use batch_processor::*;
pub use metrics::*;
pub use integration::*;

/// Performance configuration for the blockchain
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub cache_size: usize,
    pub max_connections: usize,
    pub batch_size: usize,
    pub batch_timeout: Duration,
    pub enable_compression: bool,
    pub enable_metrics: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_size: 10_000,
            max_connections: 100,
            batch_size: 1000,
            batch_timeout: Duration::from_millis(100),
            enable_compression: true,
            enable_metrics: true,
        }
    }
}

/// Central performance manager for the blockchain
pub struct PerformanceManager {
    config: PerformanceConfig,
    cache: Arc<RwLock<LRUCache<String, Vec<u8>>>>,
    connection_pool: Arc<ConnectionPool>,
    batch_processor: Arc<BatchProcessor>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
}

impl PerformanceManager {
    pub fn new(config: PerformanceConfig) -> Result<Self> {
        let cache = Arc::new(RwLock::new(LRUCache::new(config.cache_size)));
        let connection_pool = Arc::new(ConnectionPool::new(config.max_connections));
        let batch_processor = Arc::new(BatchProcessor::new(config.batch_size, config.batch_timeout));
        let metrics = Arc::new(RwLock::new(PerformanceMetrics::new()));

        Ok(Self {
            config,
            cache,
            connection_pool,
            batch_processor,
            metrics,
            start_time: Instant::now(),
        })
    }

    pub async fn get_cached<K, V>(&self, key: K) -> Option<V>
    where
        K: ToString,
        V: serde::de::DeserializeOwned,
    {
        let mut cache = self.cache.write().await;
        if let Some(data) = cache.get(&key.to_string()) {
            if let Ok(value) = bincode::deserialize::<V>(&data) {
                drop(cache);
                self.record_cache_hit().await;
                return Some(value);
            }
        }
        drop(cache);
        self.record_cache_miss().await;
        None
    }

    pub async fn set_cached<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: ToString,
        V: serde::Serialize,
    {
        let data = bincode::serialize(&value)?;
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), data);
        Ok(())
    }

    pub async fn batch_process(&self, operations: Vec<batch_processor::BatchOperation>) -> Result<()> {
        self.batch_processor.add_operations(operations).await
    }

    pub async fn get_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.connection_pool.get_connection(endpoint).await
    }

    pub async fn record_operation(&self, operation: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.record_operation(operation, duration);
    }

    pub async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    pub async fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub async fn optimize_storage(&self) -> Result<()> {
        info!("Running storage optimization...");
        
        // Trigger cache cleanup
        let mut cache = self.cache.write().await;
        cache.cleanup();
        
        // Optimize connection pool
        self.connection_pool.optimize().await?;
        
        // Process any pending batches
        self.batch_processor.flush().await?;
        
        info!("Storage optimization completed");
        Ok(())
    }

    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.metrics.read().await;
        let cache_hit_rate = if metrics.cache_hits + metrics.cache_misses > 0 {
            metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64
        } else {
            0.0
        };

        HealthStatus {
            uptime: self.uptime(),
            cache_hit_rate,
            active_connections: self.connection_pool.active_connections().await,
            pending_batches: self.batch_processor.pending_count().await,
            memory_usage: self.estimate_memory_usage().await,
        }
    }

    async fn estimate_memory_usage(&self) -> usize {
        let cache = self.cache.read().await;
        cache.estimated_size()
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub uptime: Duration,
    pub cache_hit_rate: f64,
    pub active_connections: usize,
    pub pending_batches: usize,
    pub memory_usage: usize,
}