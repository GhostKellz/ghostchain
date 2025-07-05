// GhostD REST API Server
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use warp::{Filter, Reply};

use ghostchain_core::{
    blockchain::Blockchain,
    performance::PerformanceManager,
};
use ghostchain_shared::types::*;

use crate::config::GhostdConfig;
use crate::daemon::DaemonStatus;

#[derive(Clone)]
pub struct ApiServer {
    config: GhostdConfig,
    blockchain: Arc<RwLock<Blockchain>>,
    performance_manager: PerformanceManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl ApiServer {
    pub async fn new(
        config: GhostdConfig,
        blockchain: Arc<RwLock<Blockchain>>,
        performance_manager: PerformanceManager,
    ) -> Result<Self> {
        Ok(Self {
            config,
            blockchain,
            performance_manager,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        let api = self.create_routes();
        
        let addr: std::net::SocketAddr = "127.0.0.1:8548".parse()
            .map_err(|e| anyhow!("Invalid API bind address: {}", e))?;
        
        info!("🌐 Starting API server on {}", addr);
        
        warp::serve(api)
            .run(addr)
            .await;
        
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping API server");
        // In a real implementation, we'd have a way to gracefully stop the server
        Ok(())
    }
    
    fn create_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
        
        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&ApiResponse::success("GhostD API is healthy".to_string()))
            });
        
        // Status endpoint
        let status = warp::path("status")
            .and(warp::get())
            .and(self.with_state())
            .and_then(Self::handle_status);
        
        // Blockchain endpoints
        let blockchain_routes = self.blockchain_routes();
        
        // Performance endpoints
        let performance_routes = self.performance_routes();
        
        warp::path("api")
            .and(warp::path("v1"))
            .and(
                health
                    .or(status)
                    .or(blockchain_routes)
                    .or(performance_routes)
            )
            .with(cors)
    }
    
    fn blockchain_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let height = warp::path("blockchain")
            .and(warp::path("height"))
            .and(warp::get())
            .and(self.with_state())
            .and_then(Self::handle_get_height);
        
        let get_block = warp::path("blockchain")
            .and(warp::path("block"))
            .and(warp::path::param::<u64>())
            .and(warp::get())
            .and(self.with_state())
            .and_then(Self::handle_get_block);
        
        height.or(get_block)
    }
    
    fn performance_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let metrics = warp::path("performance")
            .and(warp::path("metrics"))
            .and(warp::get())
            .and(self.with_state())
            .and_then(Self::handle_get_metrics);
        
        metrics
    }
    
    fn with_state(&self) -> impl Filter<Extract = (ApiServer,), Error = std::convert::Infallible> + Clone {
        let server = self.clone();
        warp::any().map(move || server.clone())
    }
    
    async fn handle_status(server: ApiServer) -> Result<impl Reply, warp::Rejection> {
        let blockchain = server.blockchain.read().await;
        
        let status = DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            chain_id: server.config.chain.chain_id.clone(),
            current_height: blockchain.current_height(),
            peer_count: 0, // TODO: Implement peer count
            testnet_mode: server.config.testnet_mode,
            services: crate::daemon::ServiceStatus {
                zquic_enabled: server.config.zquic.enabled,
                rpc_enabled: server.config.rpc.enabled,
                api_enabled: true,
                mining_enabled: server.config.chain.enable_mining,
            },
        };
        
        Ok(warp::reply::json(&ApiResponse::success(status)))
    }
    
    async fn handle_get_height(server: ApiServer) -> Result<impl Reply, warp::Rejection> {
        let blockchain = server.blockchain.read().await;
        let height = blockchain.current_height();
        
        Ok(warp::reply::json(&ApiResponse::success(height)))
    }
    
    async fn handle_get_block(height: u64, server: ApiServer) -> Result<impl Reply, warp::Rejection> {
        let blockchain = server.blockchain.read().await;
        
        if height < blockchain.chain.len() as u64 {
            let block = &blockchain.chain[height as usize];
            Ok(warp::reply::json(&ApiResponse::success(block)))
        } else {
            Ok(warp::reply::json(&ApiResponse::<Block>::error(
                format!("Block not found at height {}", height)
            )))
        }
    }
    
    async fn handle_get_metrics(server: ApiServer) -> Result<impl Reply, warp::Rejection> {
        match server.performance_manager.collect_metrics().await {
            Ok(metrics) => Ok(warp::reply::json(&ApiResponse::success(metrics))),
            Err(e) => Ok(warp::reply::json(&ApiResponse::<ghostchain_core::performance::PerformanceMetrics>::error(
                format!("Failed to collect metrics: {}", e)
            ))),
        }
    }
}