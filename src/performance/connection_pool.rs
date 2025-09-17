use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// Connection pool for managing network connections
pub struct ConnectionPool {
    max_connections: usize,
    connections: Arc<RwLock<HashMap<String, Arc<Mutex<PooledConnection>>>>>,
    connection_count: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone)]
pub struct PooledConnection {
    pub endpoint: String,
    pub created_at: Instant,
    pub last_used: Instant,
    pub use_count: usize,
    pub is_active: bool,
}

pub struct ConnectionHandle {
    pub connection: Arc<Mutex<PooledConnection>>,
    pool: Arc<ConnectionPool>,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn get_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        let connections = self.connections.read().await;
        
        // Check if we already have a connection to this endpoint
        if let Some(connection) = connections.get(endpoint) {
            let mut conn = connection.lock().await;
            if conn.is_active {
                conn.last_used = Instant::now();
                conn.use_count += 1;
                return Ok(ConnectionHandle {
                    connection: connection.clone(),
                    pool: Arc::new(self.clone()),
                });
            }
        }
        
        drop(connections);
        
        // Create new connection
        self.create_connection(endpoint).await
    }

    async fn create_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        let mut count = self.connection_count.lock().await;
        
        // Check if we've reached the maximum number of connections
        if *count >= self.max_connections {
            // Try to find and remove an inactive connection
            let mut connections = self.connections.write().await;
            let inactive_key = connections
                .iter()
                .find(|(_, conn)| {
                    if let Ok(c) = conn.try_lock() {
                        !c.is_active
                    } else {
                        false
                    }
                })
                .map(|(key, _)| key.clone());
            
            if let Some(key) = inactive_key {
                connections.remove(&key);
                *count -= 1;
            } else {
                return Err(anyhow!("Connection pool is full"));
            }
        }

        // Create new connection
        let connection = Arc::new(Mutex::new(PooledConnection {
            endpoint: endpoint.to_string(),
            created_at: Instant::now(),
            last_used: Instant::now(),
            use_count: 1,
            is_active: true,
        }));

        // Add to pool
        let mut connections = self.connections.write().await;
        connections.insert(endpoint.to_string(), connection.clone());
        *count += 1;

        info!("Created new connection to {}", endpoint);

        Ok(ConnectionHandle {
            connection,
            pool: Arc::new(self.clone()),
        })
    }

    pub async fn close_connection(&self, endpoint: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get(endpoint) {
            let mut conn = connection.lock().await;
            conn.is_active = false;
        }
        
        connections.remove(endpoint);
        
        let mut count = self.connection_count.lock().await;
        if *count > 0 {
            *count -= 1;
        }
        
        info!("Closed connection to {}", endpoint);
        Ok(())
    }

    pub async fn active_connections(&self) -> usize {
        let connections = self.connections.read().await;
        let mut active = 0;
        
        for connection in connections.values() {
            if let Ok(conn) = connection.try_lock() {
                if conn.is_active {
                    active += 1;
                }
            }
        }
        
        active
    }

    pub async fn cleanup_inactive(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        let now = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes
        
        let inactive_keys: Vec<String> = connections
            .iter()
            .filter_map(|(key, conn)| {
                if let Ok(c) = conn.try_lock() {
                    if !c.is_active || now.duration_since(c.last_used) > timeout {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        for key in inactive_keys {
            connections.remove(&key);
            let mut count = self.connection_count.lock().await;
            if *count > 0 {
                *count -= 1;
            }
        }
        
        Ok(())
    }

    pub async fn optimize(&self) -> Result<()> {
        debug!("Optimizing connection pool...");
        
        // Clean up inactive connections
        self.cleanup_inactive().await?;
        
        // Get connection statistics
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        let active_connections = self.active_connections().await;
        
        info!(
            "Connection pool optimized: {} total, {} active",
            total_connections, active_connections
        );
        
        Ok(())
    }

    pub async fn get_stats(&self) -> ConnectionPoolStats {
        let connections = self.connections.read().await;
        let total = connections.len();
        let active = self.active_connections().await;
        
        let mut total_use_count = 0;
        let mut oldest_connection = None;
        
        for connection in connections.values() {
            if let Ok(conn) = connection.try_lock() {
                total_use_count += conn.use_count;
                if oldest_connection.is_none() || conn.created_at < oldest_connection.unwrap() {
                    oldest_connection = Some(conn.created_at);
                }
            }
        }
        
        ConnectionPoolStats {
            total_connections: total,
            active_connections: active,
            max_connections: self.max_connections,
            total_use_count,
            oldest_connection_age: oldest_connection
                .map(|created| Instant::now().duration_since(created)),
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            max_connections: self.max_connections,
            connections: self.connections.clone(),
            connection_count: self.connection_count.clone(),
        }
    }
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        // Connection is automatically returned to pool when handle is dropped
        // The connection remains in the pool for reuse
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub max_connections: usize,
    pub total_use_count: usize,
    pub oldest_connection_age: Option<Duration>,
}

/// High-level connection manager for different service types
pub struct ServiceConnectionManager {
    ghostd_pool: ConnectionPool,
    walletd_pool: ConnectionPool,
    zvm_pool: ConnectionPool,
    ghostbridge_pool: ConnectionPool,
    quinn_pool: ConnectionPool,
}

impl ServiceConnectionManager {
    pub fn new(max_connections_per_service: usize) -> Self {
        Self {
            ghostd_pool: ConnectionPool::new(max_connections_per_service),
            walletd_pool: ConnectionPool::new(max_connections_per_service),
            zvm_pool: ConnectionPool::new(max_connections_per_service),
            ghostbridge_pool: ConnectionPool::new(max_connections_per_service),
            quinn_pool: ConnectionPool::new(max_connections_per_service),
        }
    }

    pub async fn get_ghostd_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.ghostd_pool.get_connection(endpoint).await
    }

    pub async fn get_walletd_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.walletd_pool.get_connection(endpoint).await
    }

    pub async fn get_zvm_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.zvm_pool.get_connection(endpoint).await
    }

    pub async fn get_ghostbridge_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.ghostbridge_pool.get_connection(endpoint).await
    }

    pub async fn get_quinn_connection(&self, endpoint: &str) -> Result<ConnectionHandle> {
        self.quinn_pool.get_connection(endpoint).await
    }

    pub async fn optimize_all(&self) -> Result<()> {
        self.ghostd_pool.optimize().await?;
        self.walletd_pool.optimize().await?;
        self.zvm_pool.optimize().await?;
        self.ghostbridge_pool.optimize().await?;
        self.zquic_pool.optimize().await?;
        Ok(())
    }

    pub async fn get_total_stats(&self) -> ServiceConnectionStats {
        let ghostd_stats = self.ghostd_pool.get_stats().await;
        let walletd_stats = self.walletd_pool.get_stats().await;
        let zvm_stats = self.zvm_pool.get_stats().await;
        let ghostbridge_stats = self.ghostbridge_pool.get_stats().await;
        let zquic_stats = self.zquic_pool.get_stats().await;

        ServiceConnectionStats {
            ghostd: ghostd_stats,
            walletd: walletd_stats,
            zvm: zvm_stats,
            ghostbridge: ghostbridge_stats,
            zquic: zquic_stats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceConnectionStats {
    pub ghostd: ConnectionPoolStats,
    pub walletd: ConnectionPoolStats,
    pub zvm: ConnectionPoolStats,
    pub ghostbridge: ConnectionPoolStats,
    pub zquic: ConnectionPoolStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_connection_pool_basic() {
        let pool = ConnectionPool::new(5);
        
        let conn1 = pool.get_connection("localhost:8080").await.unwrap();
        let conn2 = pool.get_connection("localhost:8081").await.unwrap();
        
        assert_eq!(pool.active_connections().await, 2);
        
        // Get same connection again
        let conn3 = pool.get_connection("localhost:8080").await.unwrap();
        assert_eq!(pool.active_connections().await, 2); // Should reuse connection
    }

    #[tokio::test]
    async fn test_connection_pool_limit() {
        let pool = ConnectionPool::new(2);
        
        let _conn1 = pool.get_connection("localhost:8080").await.unwrap();
        let _conn2 = pool.get_connection("localhost:8081").await.unwrap();
        
        // This should fail because pool is full
        let result = pool.get_connection("localhost:8082").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_connection_manager() {
        let manager = ServiceConnectionManager::new(10);
        
        let ghostd_conn = manager.get_ghostd_connection("localhost:8080").await.unwrap();
        let walletd_conn = manager.get_walletd_connection("localhost:8081").await.unwrap();
        
        let stats = manager.get_total_stats().await;
        assert_eq!(stats.ghostd.active_connections, 1);
        assert_eq!(stats.walletd.active_connections, 1);
    }
}