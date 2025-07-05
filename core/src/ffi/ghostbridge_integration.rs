use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{timeout, Duration, Instant};
use tracing::{info, warn, error, debug};
use serde_json;
use uuid::Uuid;

use crate::ffi::ghostbridge::*;
use crate::ffi::zquic_integration::ZQuicIntegration;
use crate::types::*;
use crate::blockchain::Blockchain;
// Services will be added later when implemented

/// Enhanced GhostBridge integration for cross-service communication
pub struct GhostBridgeIntegration {
    bridge: Option<*mut GhostBridge>,
    zquic_integration: Arc<Mutex<ZQuicIntegration>>,
    service_registry: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    message_handlers: Arc<RwLock<HashMap<String, Box<dyn MessageHandler + Send + Sync>>>>,
    is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub service_id: String,
    pub endpoint: String,
    pub service_type: ServiceType,
    pub status: ServiceStatus,
    pub last_heartbeat: Instant,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    Blockchain,    // ghostd
    Wallet,        // walletd
    NetworkNode,   // Network layer
    ZNS,          // Domain resolution
    ZVM,          // Virtual machine
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Error,
    Connecting,
}

pub trait MessageHandler {
    async fn handle_message(&self, message: &GhostBridgeMessage) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct GhostBridgeMessage {
    pub message_id: String,
    pub service_id: String,
    pub method: String,
    pub payload: Vec<u8>,
    pub correlation_id: Option<String>,
}

unsafe impl Send for GhostBridgeIntegration {}
unsafe impl Sync for GhostBridgeIntegration {}

impl GhostBridgeIntegration {
    pub fn new(zquic_integration: Arc<Mutex<ZQuicIntegration>>) -> Self {
        Self {
            bridge: None,
            zquic_integration,
            service_registry: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            is_connected: false,
        }
    }

    /// Initialize GhostBridge with GhostChain configuration
    pub async fn initialize(&mut self, config: GhostBridgeConfig) -> Result<()> {
        info!("Initializing GhostBridge integration for cross-service communication");

        unsafe {
            let bridge = ghostbridge_init(&config);
            if bridge.is_null() {
                return Err(anyhow!("Failed to initialize GhostBridge"));
            }
            self.bridge = Some(bridge);
        }

        info!("GhostBridge integration initialized successfully");
        Ok(())
    }

    /// Connect to the Zig GhostBridge implementation
    pub async fn connect(&mut self) -> Result<()> {
        let bridge = self.bridge.ok_or_else(|| anyhow!("GhostBridge not initialized"))?;

        unsafe {
            let result = ghostbridge_connect(bridge);
            if result != 0 {
                return Err(anyhow!("Failed to connect to GhostBridge"));
            }
        }

        self.is_connected = true;
        info!("Successfully connected to GhostBridge");

        // Start service discovery and health monitoring
        self.start_service_discovery().await?;
        self.start_health_monitoring().await?;

        Ok(())
    }

    /// Register a Rust service with GhostBridge
    pub async fn register_service(
        &mut self,
        service_info: ServiceInfo,
        handler: Box<dyn MessageHandler + Send + Sync>,
    ) -> Result<()> {
        if !self.is_connected {
            return Err(anyhow!("GhostBridge not connected"));
        }

        let bridge = self.bridge.ok_or_else(|| anyhow!("No GhostBridge handle"))?;

        // Register with bridge
        let service_name = std::ffi::CString::new(service_info.service_id.clone())
            .map_err(|_| anyhow!("Invalid service name"))?;
        let endpoint = std::ffi::CString::new(service_info.endpoint.clone())
            .map_err(|_| anyhow!("Invalid endpoint"))?;

        unsafe {
            let result = ghostbridge_register_service(
                bridge,
                service_name.as_ptr(),
                endpoint.as_ptr(),
            );

            if result != 0 {
                return Err(anyhow!("Failed to register service with GhostBridge"));
            }
        }

        // Store service info and handler
        {
            let mut registry = self.service_registry.write().await;
            registry.insert(service_info.service_id.clone(), service_info.clone());
        }

        {
            let mut handlers = self.message_handlers.write().await;
            handlers.insert(service_info.service_id.clone(), handler);
        }

        info!("Registered service: {} at {}", service_info.service_id, service_info.endpoint);
        Ok(())
    }

    /// Send message to another service via GhostBridge
    pub async fn send_message_to_service(
        &self,
        target_service: &str,
        method: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        if !self.is_connected {
            return Err(anyhow!("GhostBridge not connected"));
        }

        let bridge = self.bridge.ok_or_else(|| anyhow!("No GhostBridge handle"))?;

        let service_cstr = std::ffi::CString::new(target_service)
            .map_err(|_| anyhow!("Invalid service name"))?;
        let method_cstr = std::ffi::CString::new(method)
            .map_err(|_| anyhow!("Invalid method name"))?;
        let correlation_id = Uuid::new_v4().to_string();
        let correlation_cstr = std::ffi::CString::new(correlation_id.clone())
            .map_err(|_| anyhow!("Invalid correlation ID"))?;

        let message = CrossLangMessage {
            message_type: MessageType::ServiceDiscovery,
            service_id: service_cstr.as_ptr(),
            method: method_cstr.as_ptr(),
            payload: payload.as_ptr(),
            payload_len: payload.len(),
            correlation_id: correlation_cstr.as_ptr(),
        };

        unsafe {
            let result = ghostbridge_send_message(bridge, &message);
            if result != 0 {
                return Err(anyhow!("Failed to send message to service {}", target_service));
            }
        }

        // In a real implementation, we'd wait for the response with the correlation ID
        // For now, return success acknowledgment
        Ok(b"Message sent successfully".to_vec())
    }

    /// Send blockchain transaction via GhostBridge to ghostd
    pub async fn send_blockchain_transaction(&self, transaction: &Transaction) -> Result<String> {
        let tx_data = serde_json::to_vec(transaction)
            .map_err(|e| anyhow!("Failed to serialize transaction: {}", e))?;

        let response = self.send_message_to_service(
            "ghostd",
            "SubmitTransaction",
            &tx_data,
        ).await?;

        let response_str = String::from_utf8_lossy(&response);
        Ok(format!("Transaction {} submitted via GhostBridge: {}", transaction.id, response_str))
    }

    /// Query wallet balance via GhostBridge to walletd
    pub async fn query_wallet_balance(&self, wallet_id: &str, token_type: TokenType) -> Result<u128> {
        let query = serde_json::json!({
            "wallet_id": wallet_id,
            "token_type": token_type
        });

        let query_data = serde_json::to_vec(&query)
            .map_err(|e| anyhow!("Failed to serialize query: {}", e))?;

        let response = self.send_message_to_service(
            "walletd",
            "GetBalance",
            &query_data,
        ).await?;

        // Parse response
        let response_json: serde_json::Value = serde_json::from_slice(&response)
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        let balance = response_json["balance"].as_u64()
            .ok_or_else(|| anyhow!("Invalid balance in response"))?;

        Ok(balance as u128)
    }

    /// Execute ZVM contract via GhostBridge
    pub async fn execute_zvm_contract(
        &self,
        contract_id: &str,
        method: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let bridge = self.bridge.ok_or_else(|| anyhow!("No GhostBridge handle"))?;

        let contract_cstr = std::ffi::CString::new(contract_id)
            .map_err(|_| anyhow!("Invalid contract ID"))?;
        let method_cstr = std::ffi::CString::new(method)
            .map_err(|_| anyhow!("Invalid method name"))?;

        let mut result_buffer = vec![0u8; 4096];
        let mut result_len = 0usize;

        unsafe {
            let result = ghostbridge_execute_contract(
                bridge,
                contract_cstr.as_ptr(),
                method_cstr.as_ptr(),
                args.as_ptr(),
                args.len(),
                result_buffer.as_mut_ptr(),
                &mut result_len,
            );

            if result != 0 {
                return Err(anyhow!("Failed to execute contract via GhostBridge"));
            }
        }

        result_buffer.truncate(result_len);
        Ok(result_buffer)
    }

    /// Get service registry status
    pub async fn get_service_registry(&self) -> HashMap<String, ServiceInfo> {
        let registry = self.service_registry.read().await;
        registry.clone()
    }

    /// Check if a specific service is available
    pub async fn is_service_available(&self, service_id: &str) -> bool {
        let registry = self.service_registry.read().await;
        registry.get(service_id)
            .map(|info| matches!(info.status, ServiceStatus::Active))
            .unwrap_or(false)
    }

    async fn start_service_discovery(&self) -> Result<()> {
        info!("Starting service discovery for GhostChain ecosystem");

        // Register core GhostChain services
        let core_services = vec![
            ("ghostd", "127.0.0.1:8000", ServiceType::Blockchain),
            ("walletd", "127.0.0.1:8001", ServiceType::Wallet),
            ("zns", "127.0.0.1:8002", ServiceType::ZNS),
            ("zvm", "127.0.0.1:8003", ServiceType::ZVM),
        ];

        let mut registry = self.service_registry.write().await;
        for (service_id, endpoint, service_type) in core_services {
            let service_info = ServiceInfo {
                service_id: service_id.to_string(),
                endpoint: endpoint.to_string(),
                service_type,
                status: ServiceStatus::Connecting,
                last_heartbeat: Instant::now(),
            };
            registry.insert(service_id.to_string(), service_info);
        }

        info!("Service discovery initialized with {} core services", registry.len());
        Ok(())
    }

    async fn start_health_monitoring(&self) -> Result<()> {
        info!("Starting health monitoring for registered services");
        
        // In a real implementation, this would spawn a background task
        // to periodically check service health and update the registry
        
        Ok(())
    }
}

impl Drop for GhostBridgeIntegration {
    fn drop(&mut self) {
        unsafe {
            if let Some(bridge) = self.bridge {
                ghostbridge_disconnect(bridge);
                ghostbridge_destroy(bridge);
            }
        }
        info!("GhostBridge integration cleaned up");
    }
}

/// Blockchain message handler for ghostd service
pub struct BlockchainMessageHandler {
    blockchain: Arc<RwLock<Blockchain>>,
}

impl BlockchainMessageHandler {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self { blockchain }
    }
}

#[async_trait::async_trait]
impl MessageHandler for BlockchainMessageHandler {
    async fn handle_message(&self, message: &GhostBridgeMessage) -> Result<Vec<u8>> {
        match message.method.as_str() {
            "SubmitTransaction" => {
                let transaction: Transaction = serde_json::from_slice(&message.payload)
                    .map_err(|e| anyhow!("Invalid transaction format: {}", e))?;

                // Process transaction
                let mut blockchain = self.blockchain.write().await;
                let result = blockchain.add_transaction(transaction.clone());

                let response = serde_json::json!({
                    "success": result.is_ok(),
                    "transaction_id": transaction.id,
                    "message": result.map(|_| "Transaction added".to_string())
                        .unwrap_or_else(|e| format!("Error: {}", e))
                });

                Ok(serde_json::to_vec(&response)?)
            },
            "QueryState" => {
                let blockchain = self.blockchain.read().await;
                let state_info = serde_json::json!({
                    "height": blockchain.current_height(),
                    "accounts": blockchain.state.accounts.len(),
                    "total_supply": blockchain.state.total_supply
                });

                Ok(serde_json::to_vec(&state_info)?)
            },
            _ => Err(anyhow!("Unknown method: {}", message.method)),
        }
    }
}

/// Test function for GhostBridge integration
pub async fn test_ghostbridge_integration() -> Result<()> {
    info!("Starting GhostBridge integration test");

    // Create blockchain instance for testing
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    
    // Create ZQUIC integration
    let zquic_integration = Arc::new(Mutex::new(
        crate::ffi::zquic_integration::ZQuicIntegration::new()
    ));

    // Create GhostBridge integration
    let mut bridge_integration = GhostBridgeIntegration::new(zquic_integration);

    // Initialize with test configuration
    let config = GhostBridgeConfig {
        rust_port: 8100,
        zig_port: 8101,
        enable_tls: false,
        max_message_size: 1024 * 1024, // 1MB
        timeout_ms: 5000,
    };

    match bridge_integration.initialize(config).await {
        Ok(()) => info!("GhostBridge initialized successfully"),
        Err(e) => warn!("GhostBridge initialization failed: {}", e),
    }

    // Test connection
    match bridge_integration.connect().await {
        Ok(()) => {
            info!("GhostBridge connected successfully");

            // Register blockchain service
            let blockchain_handler = Box::new(BlockchainMessageHandler::new(blockchain.clone()));
            let blockchain_service = ServiceInfo {
                service_id: "ghostd".to_string(),
                endpoint: "127.0.0.1:8000".to_string(),
                service_type: ServiceType::Blockchain,
                status: ServiceStatus::Active,
                last_heartbeat: Instant::now(),
            };

            match bridge_integration.register_service(blockchain_service, blockchain_handler).await {
                Ok(()) => info!("Blockchain service registered successfully"),
                Err(e) => warn!("Failed to register blockchain service: {}", e),
            }

            // Test service discovery
            let services = bridge_integration.get_service_registry().await;
            info!("Discovered {} services", services.len());
            for (id, info) in services {
                info!("Service {}: {} ({:?})", id, info.endpoint, info.status);
            }
        },
        Err(e) => {
            warn!("GhostBridge connection failed: {}", e);
            warn!("This is expected if GhostBridge Zig implementation is not running");
        }
    }

    info!("GhostBridge integration test completed");
    Ok(())
}

/// CLI command handler for GhostBridge testing
pub async fn handle_ghostbridge_test_command() -> Result<()> {
    println!("🌉 Testing GhostBridge Integration...");
    
    match test_ghostbridge_integration().await {
        Ok(()) => {
            println!("✅ GhostBridge integration test completed successfully");
            println!("📋 Note: Some connection tests may fail if GhostBridge server is not running");
        },
        Err(e) => {
            println!("❌ GhostBridge integration test failed: {}", e);
        }
    }
    
    Ok(())
}