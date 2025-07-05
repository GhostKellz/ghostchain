// ZQUIC Integration for GhostD
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use ghostchain_core::{
    blockchain::Blockchain,
    performance::PerformanceManager,
};
use ghostchain_shared::{
    types::*,
    ffi::zquic::*,
    ffi::ghostbridge::*,
};

#[derive(Clone)]
pub struct ZQuicDaemonServer {
    config: ZQuicConfig,
    blockchain: Arc<RwLock<Blockchain>>,
    performance_manager: PerformanceManager,
    zquic_context: Option<Arc<ZQuicContext>>,
    ghostbridge_context: Option<Arc<GhostBridgeContext>>,
}

impl ZQuicDaemonServer {
    pub async fn new(
        config: ZQuicConfig,
        blockchain: Arc<RwLock<Blockchain>>,
        performance_manager: PerformanceManager,
    ) -> Result<Self> {
        info!("⚡ Initializing ZQUIC daemon server");
        
        // Initialize ZQUIC context
        let zquic_context = if config.enabled {
            match unsafe { zquic_init(&config as *const ZQuicConfig) } {
                ptr if !ptr.is_null() => {
                    info!("✅ ZQUIC context initialized");
                    Some(Arc::new(ZQuicContext { ptr }))
                },
                _ => {
                    warn!("⚠️  Failed to initialize ZQUIC context");
                    None
                }
            }
        } else {
            None
        };
        
        // Initialize GhostBridge context
        let ghostbridge_context = if config.ghostbridge_enabled {
            let bridge_config = GhostBridgeConfig {
                enabled: true,
                bind_address: config.bind_address.clone(),
                max_connections: config.max_connections,
                timeout_ms: config.timeout_ms,
                enable_compression: true,
                alpn_protocols: vec!["ghostchain/1.0".to_string()],
            };
            
            match unsafe { ghostbridge_init(&bridge_config as *const GhostBridgeConfig) } {
                ptr if !ptr.is_null() => {
                    info!("🌉 GhostBridge context initialized");
                    Some(Arc::new(GhostBridgeContext { ptr }))
                },
                _ => {
                    warn!("⚠️  Failed to initialize GhostBridge context");
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            config,
            blockchain,
            performance_manager,
            zquic_context,
            ghostbridge_context,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        info!("🏃 Starting ZQUIC daemon server");
        
        if self.zquic_context.is_none() {
            return Err(anyhow!("ZQUIC context not initialized"));
        }
        
        // Start QUIC server
        self.start_quic_server().await?;
        
        // Start GhostBridge if enabled
        if self.ghostbridge_context.is_some() {
            self.start_ghostbridge_server().await?;
        }
        
        // Start request handling loop
        self.handle_requests().await?;
        
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping ZQUIC daemon server");
        
        // Cleanup ZQUIC context
        if let Some(context) = &self.zquic_context {
            unsafe {
                zquic_destroy(context.ptr);
            }
            info!("✅ ZQUIC context cleaned up");
        }
        
        // Cleanup GhostBridge context
        if let Some(context) = &self.ghostbridge_context {
            unsafe {
                ghostbridge_destroy(context.ptr);
            }
            info!("🌉 GhostBridge context cleaned up");
        }
        
        Ok(())
    }
    
    async fn start_quic_server(&self) -> Result<()> {
        let context = self.zquic_context.as_ref().unwrap();
        
        let bind_addr = std::ffi::CString::new(self.config.bind_address.clone())?;
        
        let result = unsafe {
            zquic_start_server(context.ptr, bind_addr.as_ptr())
        };
        
        if result == 0 {
            info!("⚡ ZQUIC server started on {}", self.config.bind_address);
            Ok(())
        } else {
            Err(anyhow!("Failed to start ZQUIC server: error code {}", result))
        }
    }
    
    async fn start_ghostbridge_server(&self) -> Result<()> {
        let context = self.ghostbridge_context.as_ref().unwrap();
        
        let bind_addr = std::ffi::CString::new(self.config.bind_address.clone())?;
        
        let result = unsafe {
            ghostbridge_start_server(context.ptr, bind_addr.as_ptr())
        };
        
        if result == 0 {
            info!("🌉 GhostBridge server started on {}", self.config.bind_address);
            Ok(())
        } else {
            Err(anyhow!("Failed to start GhostBridge server: error code {}", result))
        }
    }
    
    async fn handle_requests(&self) -> Result<()> {
        info!("📡 ZQUIC daemon request handler started");
        
        loop {
            // Handle ZQUIC requests
            if let Some(context) = &self.zquic_context {
                match self.process_zquic_requests(context).await {
                    Ok(_) => {},
                    Err(e) => warn!("ZQUIC request error: {}", e),
                }
            }
            
            // Handle GhostBridge requests
            if let Some(context) = &self.ghostbridge_context {
                match self.process_ghostbridge_requests(context).await {
                    Ok(_) => {},
                    Err(e) => warn!("GhostBridge request error: {}", e),
                }
            }
            
            // Small delay to prevent busy loop
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    
    async fn process_zquic_requests(&self, context: &ZQuicContext) -> Result<()> {
        // Check for incoming connections
        let mut connection_ptr: *mut ZQuicConnection = std::ptr::null_mut();
        let result = unsafe {
            zquic_accept_connection(context.ptr, &mut connection_ptr as *mut *mut ZQuicConnection)
        };
        
        if result == 0 && !connection_ptr.is_null() {
            info!("🔗 New ZQUIC connection accepted");
            
            // Handle the connection
            tokio::spawn({
                let blockchain = self.blockchain.clone();
                let perf_manager = self.performance_manager.clone();
                async move {
                    if let Err(e) = Self::handle_zquic_connection(connection_ptr, blockchain, perf_manager).await {
                        error!("ZQUIC connection error: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
    
    async fn process_ghostbridge_requests(&self, context: &GhostBridgeContext) -> Result<()> {
        // Check for gRPC requests over QUIC
        let mut request_ptr: *mut GhostBridgeRequest = std::ptr::null_mut();
        let result = unsafe {
            ghostbridge_receive_request(context.ptr, &mut request_ptr as *mut *mut GhostBridgeRequest)
        };
        
        if result == 0 && !request_ptr.is_null() {
            info!("📨 New GhostBridge request received");
            
            // Handle the gRPC request
            tokio::spawn({
                let blockchain = self.blockchain.clone();
                async move {
                    if let Err(e) = Self::handle_ghostbridge_request(request_ptr, blockchain).await {
                        error!("GhostBridge request error: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
    
    async fn handle_zquic_connection(
        connection: *mut ZQuicConnection,
        blockchain: Arc<RwLock<Blockchain>>,
        _perf_manager: PerformanceManager,
    ) -> Result<()> {
        // Read data from connection
        let mut buffer = vec![0u8; 4096];
        let mut bytes_read = 0;
        
        let result = unsafe {
            zquic_read_data(
                connection,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut bytes_read as *mut usize,
            )
        };
        
        if result == 0 && bytes_read > 0 {
            let request_data = &buffer[..bytes_read];
            
            // Parse and handle request
            let response = Self::process_blockchain_request(request_data, blockchain).await?;
            
            // Send response
            let result = unsafe {
                zquic_send_data(
                    connection,
                    response.as_ptr(),
                    response.len(),
                )
            };
            
            if result != 0 {
                error!("Failed to send ZQUIC response: error code {}", result);
            }
        }
        
        // Close connection
        unsafe {
            zquic_close_connection(connection);
        }
        
        Ok(())
    }
    
    async fn handle_ghostbridge_request(
        request: *mut GhostBridgeRequest,
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<()> {
        // Extract gRPC method and data from request
        let mut method_ptr: *const std::os::raw::c_char = std::ptr::null();
        let mut data_ptr: *const u8 = std::ptr::null();
        let mut data_len = 0;
        
        unsafe {
            ghostbridge_get_request_data(
                request,
                &mut method_ptr as *mut *const std::os::raw::c_char,
                &mut data_ptr as *mut *const u8,
                &mut data_len as *mut usize,
            );
        }
        
        if !method_ptr.is_null() && !data_ptr.is_null() {
            let method = unsafe { std::ffi::CStr::from_ptr(method_ptr) }.to_str()?;
            let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
            
            info!("🔧 Handling gRPC method: {}", method);
            
            // Process gRPC method
            let response = Self::process_grpc_method(method, data, blockchain).await?;
            
            // Send response through GhostBridge
            unsafe {
                ghostbridge_send_response(
                    request,
                    response.as_ptr(),
                    response.len(),
                );
            }
        }
        
        Ok(())
    }
    
    async fn process_blockchain_request(
        request_data: &[u8],
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<Vec<u8>> {
        // Parse request (simplified JSON for demo)
        let request: serde_json::Value = serde_json::from_slice(request_data)?;
        
        let method = request["method"].as_str().unwrap_or("unknown");
        
        match method {
            "get_height" => {
                let chain = blockchain.read().await;
                let height = chain.current_height();
                Ok(serde_json::to_vec(&serde_json::json!({
                    "result": height
                }))?)
            },
            "get_block" => {
                let height = request["params"]["height"].as_u64().unwrap_or(0);
                let chain = blockchain.read().await;
                
                if height < chain.chain.len() as u64 {
                    let block = &chain.chain[height as usize];
                    Ok(serde_json::to_vec(&serde_json::json!({
                        "result": block
                    }))?)
                } else {
                    Ok(serde_json::to_vec(&serde_json::json!({
                        "error": "Block not found"
                    }))?)
                }
            },
            _ => {
                Ok(serde_json::to_vec(&serde_json::json!({
                    "error": "Unknown method"
                }))?)
            }
        }
    }
    
    async fn process_grpc_method(
        method: &str,
        data: &[u8],
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<Vec<u8>> {
        info!("🔧 Processing gRPC method: {}", method);
        
        match method {
            "GetBlockchainInfo" => {
                let chain = blockchain.read().await;
                let info = serde_json::json!({
                    "height": chain.current_height(),
                    "chain_id": "ghostchain", // TODO: Get from config
                    "latest_block_hash": chain.chain.last().map(|b| &b.hash),
                });
                Ok(serde_json::to_vec(&info)?)
            },
            "GetBlock" => {
                // Parse gRPC request for block height
                let height = 0u64; // TODO: Parse from protobuf data
                let chain = blockchain.read().await;
                
                if height < chain.chain.len() as u64 {
                    let block = &chain.chain[height as usize];
                    Ok(serde_json::to_vec(block)?)
                } else {
                    Err(anyhow!("Block not found"))
                }
            },
            _ => {
                warn!("Unknown gRPC method: {}", method);
                Err(anyhow!("Unknown gRPC method: {}", method))
            }
        }
    }
}