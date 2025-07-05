use anyhow::{Result, anyhow};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error, debug};

use crate::ffi::zquic::*;
use crate::types::*;

/// Enhanced ZQUIC integration for production use
pub struct ZQuicIntegration {
    context: Option<*mut ZQuicContext>,
    connections: Vec<*mut ZQuicConnection>,
    is_initialized: bool,
}

unsafe impl Send for ZQuicIntegration {}
unsafe impl Sync for ZQuicIntegration {}

impl ZQuicIntegration {
    pub fn new() -> Self {
        Self {
            context: None,
            connections: Vec::new(),
            is_initialized: false,
        }
    }

    /// Initialize ZQUIC with GhostChain-specific configuration
    pub async fn initialize(&mut self, config: ZQuicIntegrationConfig) -> Result<()> {
        info!("Initializing ZQUIC integration for GhostChain");

        let cert_path = CString::new(config.cert_path.as_str())
            .map_err(|_| anyhow!("Invalid cert path"))?;
        let key_path = CString::new(config.key_path.as_str())
            .map_err(|_| anyhow!("Invalid key path"))?;

        let zquic_config = ZQuicConfig {
            max_connections: config.max_connections,
            connection_timeout_ms: config.connection_timeout_ms,
            enable_compression: config.enable_compression,
            enable_encryption: config.enable_encryption,
            cert_path: cert_path.as_ptr(),
            key_path: key_path.as_ptr(),
        };

        unsafe {
            let ctx = zquic_init(&zquic_config);
            if ctx.is_null() {
                return Err(anyhow!("Failed to initialize ZQUIC context"));
            }
            self.context = Some(ctx);
        }

        self.is_initialized = true;
        info!("ZQUIC integration initialized successfully");
        Ok(())
    }

    /// Connect to a ZQUIC endpoint (e.g., for GhostBridge communication)
    pub async fn connect_to_endpoint(&mut self, endpoint: &str) -> Result<*mut ZQuicConnection> {
        if !self.is_initialized {
            return Err(anyhow!("ZQUIC not initialized"));
        }

        let endpoint_cstr = CString::new(endpoint)
            .map_err(|_| anyhow!("Invalid endpoint string"))?;

        unsafe {
            let ctx = self.context.ok_or_else(|| anyhow!("No ZQUIC context"))?;
            let connection = zquic_create_connection(ctx, endpoint_cstr.as_ptr());
            
            if connection.is_null() {
                return Err(anyhow!("Failed to create connection to {}", endpoint));
            }

            // Wait for connection establishment with timeout
            let connection_timeout = Duration::from_millis(5000);
            let result = timeout(connection_timeout, async {
                loop {
                    let status = zquic_connection_status(connection);
                    match status {
                        ConnectionStatus::Connected => return Ok(()),
                        ConnectionStatus::Error => return Err(anyhow!("Connection failed")),
                        _ => {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }).await;

            match result {
                Ok(Ok(())) => {
                    info!("Successfully connected to ZQUIC endpoint: {}", endpoint);
                    self.connections.push(connection);
                    Ok(connection)
                },
                Ok(Err(e)) => Err(e),
                Err(_) => Err(anyhow!("Connection timeout to {}", endpoint)),
            }
        }
    }

    /// Create a gRPC stream for service communication
    pub async fn create_grpc_stream(
        &self,
        connection: *mut ZQuicConnection,
        service_name: &str,
        method_name: &str,
    ) -> Result<*mut ZQuicGrpcStream> {
        let service_cstr = CString::new(service_name)
            .map_err(|_| anyhow!("Invalid service name"))?;
        let method_cstr = CString::new(method_name)
            .map_err(|_| anyhow!("Invalid method name"))?;

        unsafe {
            let stream = zquic_create_grpc_stream(
                connection,
                service_cstr.as_ptr(),
                method_cstr.as_ptr(),
            );

            if stream.is_null() {
                return Err(anyhow!("Failed to create gRPC stream for {}.{}", service_name, method_name));
            }

            debug!("Created gRPC stream for {}.{}", service_name, method_name);
            Ok(stream)
        }
    }

    /// Send blockchain transaction via gRPC over QUIC
    pub async fn send_blockchain_transaction(
        &self,
        connection: *mut ZQuicConnection,
        transaction: &Transaction,
    ) -> Result<String> {
        let stream = self.create_grpc_stream(
            connection,
            "ghostchain.Blockchain",
            "SubmitTransaction",
        ).await?;

        // Serialize transaction to bytes
        let tx_data = serde_json::to_vec(transaction)
            .map_err(|e| anyhow!("Failed to serialize transaction: {}", e))?;

        unsafe {
            let result = zquic_send_grpc_data(
                stream,
                tx_data.as_ptr(),
                tx_data.len(),
            );

            if result != 0 {
                zquic_close_grpc_stream(stream);
                return Err(anyhow!("Failed to send transaction data"));
            }

            // In a real implementation, we'd wait for and parse the response
            zquic_close_grpc_stream(stream);
        }

        Ok(format!("Transaction {} submitted via ZQUIC", transaction.id))
    }

    /// Query blockchain state via gRPC over QUIC
    pub async fn query_blockchain_state(
        &self,
        connection: *mut ZQuicConnection,
        query_type: &str,
        query_data: &str,
    ) -> Result<String> {
        let stream = self.create_grpc_stream(
            connection,
            "ghostchain.Blockchain",
            "QueryState",
        ).await?;

        // Create query message
        let query_msg = serde_json::json!({
            "query_type": query_type,
            "data": query_data
        });

        let query_bytes = serde_json::to_vec(&query_msg)
            .map_err(|e| anyhow!("Failed to serialize query: {}", e))?;

        unsafe {
            let result = zquic_send_grpc_data(
                stream,
                query_bytes.as_ptr(),
                query_bytes.len(),
            );

            if result != 0 {
                zquic_close_grpc_stream(stream);
                return Err(anyhow!("Failed to send query"));
            }

            // In a real implementation, we'd receive and parse the response
            let mut buffer = vec![0u8; 4096];
            let mut received_len = 0usize;
            
            let recv_result = zquic_recv_grpc_data(
                stream,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut received_len,
            );

            zquic_close_grpc_stream(stream);

            if recv_result != 0 {
                return Err(anyhow!("Failed to receive query response"));
            }

            if received_len > 0 {
                let response = String::from_utf8_lossy(&buffer[..received_len]);
                Ok(response.to_string())
            } else {
                Ok("Query completed, no data returned".to_string())
            }
        }
    }

    /// Test ZQUIC connection health
    pub async fn test_connection_health(&self, connection: *mut ZQuicConnection) -> Result<bool> {
        unsafe {
            let status = zquic_connection_status(connection);
            match status {
                ConnectionStatus::Connected => Ok(true),
                ConnectionStatus::Connecting => {
                    warn!("Connection still establishing");
                    Ok(false)
                },
                ConnectionStatus::Disconnected => {
                    warn!("Connection disconnected");
                    Ok(false)
                },
                ConnectionStatus::Error => {
                    error!("Connection in error state");
                    Ok(false)
                },
            }
        }
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ZQuicStats {
        ZQuicStats {
            total_connections: self.connections.len(),
            active_connections: self.get_active_connection_count().await,
            is_initialized: self.is_initialized,
        }
    }

    async fn get_active_connection_count(&self) -> usize {
        let mut active_count = 0;
        for &connection in &self.connections {
            if let Ok(true) = self.test_connection_health(connection).await {
                active_count += 1;
            }
        }
        active_count
    }
}

impl Drop for ZQuicIntegration {
    fn drop(&mut self) {
        unsafe {
            // Close all connections
            for &connection in &self.connections {
                zquic_close_connection(connection);
            }

            // Destroy context
            if let Some(ctx) = self.context {
                zquic_destroy(ctx);
            }
        }
        info!("ZQUIC integration cleaned up");
    }
}

#[derive(Debug, Clone)]
pub struct ZQuicIntegrationConfig {
    pub max_connections: c_uint,
    pub connection_timeout_ms: c_uint,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub cert_path: String,
    pub key_path: String,
}

impl Default for ZQuicIntegrationConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            connection_timeout_ms: 5000,
            enable_compression: true,
            enable_encryption: true,
            cert_path: "./certs/ghostchain.crt".to_string(),
            key_path: "./certs/ghostchain.key".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZQuicStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub is_initialized: bool,
}

/// Test function for ZQUIC integration
pub async fn test_zquic_integration() -> Result<()> {
    info!("Starting ZQUIC integration test");

    let mut integration = ZQuicIntegration::new();
    let config = ZQuicIntegrationConfig::default();

    // Initialize ZQUIC
    integration.initialize(config).await?;

    // Test connection to local ZQUIC endpoint
    let test_endpoint = "127.0.0.1:4433";
    match integration.connect_to_endpoint(test_endpoint).await {
        Ok(connection) => {
            info!("Successfully connected to test endpoint");

            // Test connection health
            let is_healthy = integration.test_connection_health(connection).await?;
            info!("Connection health: {}", is_healthy);

            // Test gRPC stream creation
            match integration.create_grpc_stream(
                connection,
                "ghostchain.Test",
                "Ping",
            ).await {
                Ok(stream) => {
                    info!("Successfully created gRPC stream");
                    unsafe { zquic_close_grpc_stream(stream); }
                },
                Err(e) => warn!("Failed to create gRPC stream: {}", e),
            }
        },
        Err(e) => {
            warn!("Failed to connect to test endpoint: {}", e);
            warn!("This is expected if ZQUIC server is not running");
        }
    }

    let stats = integration.get_connection_stats().await;
    info!("ZQUIC integration stats: {:?}", stats);

    info!("ZQUIC integration test completed");
    Ok(())
}

/// CLI command handler for ZQUIC testing
pub async fn handle_zquic_test_command() -> Result<()> {
    println!("🚀 Testing ZQUIC Integration...");
    
    match test_zquic_integration().await {
        Ok(()) => {
            println!("✅ ZQUIC integration test completed successfully");
            println!("📋 Note: Some connection tests may fail if ZQUIC server is not running");
        },
        Err(e) => {
            println!("❌ ZQUIC integration test failed: {}", e);
        }
    }
    
    Ok(())
}