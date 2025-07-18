# 🔗 gquic Integration Guide for GhostChain Ecosystem

This guide shows how to integrate **gquic** with your Zig and Rust blockchain projects in the GhostChain ecosystem.

## Table of Contents

- [Overview](#overview)
- [Rust Integration](#rust-integration)
- [Zig Integration (FFI)](#zig-integration-ffi)
- [WalletD Integration](#walletd-integration)
- [GhostD Integration](#ghostd-integration)
- [GhostBridge Integration](#ghostbridge-integration)
- [Example Projects](#example-projects)
- [Performance Considerations](#performance-considerations)

## Overview

**gquic** provides multiple integration points for GhostChain ecosystem projects:

| Project Type | Integration Method | Use Case |
|-------------|-------------------|----------|
| Rust Services | Native crate dependency | `walletd`, `ghostd`, `ghostbridge` |
| Zig Projects | FFI (C-compatible) | `zwallet`, `realid`, `enoc`, `wraith` |
| gRPC Services | Built-in protobuf support | Service-to-service communication |
| Daemon Mode | `gquicd` binary | Standalone QUIC proxy/gateway |

## Rust Integration

### Adding gquic to Your Rust Project

Add to your `Cargo.toml`:

```toml
[dependencies]
gquic = { path = "../gquic", features = ["gcrypt-integration", "metrics"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Basic Client Integration

```rust
// src/quic_client.rs
use gquic::prelude::*;
use anyhow::Result;

pub struct GhostChainClient {
    client: QuicClient,
    pool: ConnectionPool,
}

impl GhostChainClient {
    pub fn new() -> Result<Self> {
        let config = QuicClientConfig::builder()
            .server_name("ghostchain.local".to_string())
            .with_alpn("ghostchain-v1")
            .with_alpn("grpc")
            .max_idle_timeout(30_000)
            .build();

        let client = QuicClient::new(config)?;
        let pool = ConnectionPool::new(PoolConfig::default());

        Ok(Self { client, pool })
    }

    pub async fn send_transaction(&self, addr: SocketAddr, tx_data: &[u8]) -> Result<Vec<u8>> {
        let conn = match self.pool.get_connection(addr).await {
            Some(conn) => conn,
            None => {
                let conn = self.client.connect(addr).await?;
                self.pool.return_connection(addr, conn.clone()).await;
                conn
            }
        };

        let mut stream = self.client.open_bi_stream(&conn).await?;
        stream.write_all(tx_data).await?;
        stream.finish().await?;
        
        let response = stream.read_to_end(64 * 1024).await?;
        Ok(response)
    }
}
```

### Basic Server Integration

```rust
// src/quic_server.rs
use gquic::prelude::*;
use gquic::server::handler::{ConnectionHandler, DefaultHandler};
use async_trait::async_trait;

pub struct GhostChainHandler {
    // Your blockchain state, database connections, etc.
}

#[async_trait]
impl ConnectionHandler for GhostChainHandler {
    async fn handle_connection(
        &self,
        connection: NewConnection,
        _config: Arc<QuicServerConfig>,
    ) -> Result<()> {
        let remote_addr = connection.connection.remote_address();
        tracing::info!("New blockchain connection from {}", remote_addr);

        while let Ok((mut send, mut recv)) = connection.bi_streams.accept().await {
            let handler = self.clone();
            tokio::spawn(async move {
                // Read request
                let request_data = recv.read_to_end(1024 * 1024).await?; // 1MB max
                
                // Process blockchain request
                let response = handler.process_request(&request_data).await?;
                
                // Send response
                send.write_all(&response).await?;
                send.finish().await?;
                
                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(())
    }
}

impl GhostChainHandler {
    async fn process_request(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Parse request (protobuf, JSON, etc.)
        // Execute blockchain operation
        // Return serialized response
        Ok(b"response".to_vec())
    }
}

pub async fn start_ghostchain_server() -> Result<()> {
    let handler = GhostChainHandler::new();
    
    let server = QuicServer::builder()
        .bind("0.0.0.0:9090".parse()?)
        .with_tls_files("certs/server.crt", "certs/server.key")?
        .with_alpn("ghostchain-v1")
        .with_handler(Arc::new(handler))
        .max_concurrent_bidi_streams(2000)
        .build()?;

    server.run().await
}
```

## Zig Integration (FFI)

### Building gquic for FFI

```bash
# Build with FFI support
cargo build --release --features ffi
```

This creates `libgquic.so` (Linux) or `libgquic.dylib` (macOS) in `target/release/`.

### Zig FFI Bindings

Create `src/gquic.zig`:

```zig
const std = @import("std");
const c = @cImport({
    @cInclude("gquic_ffi.h");
});

pub const GQuicError = error{
    InvalidParam,
    ConnectionFailed,
    StreamError,
    InitFailed,
};

pub const GQuicClient = struct {
    handle: ?*c.GQuicClient,

    const Self = @This();

    pub fn init(server_name: []const u8) !Self {
        var client: ?*c.GQuicClient = null;
        const server_name_cstr = try std.cstr.addNullByte(std.heap.c_allocator, server_name);
        defer std.heap.c_allocator.free(server_name_cstr);

        const result = c.gquic_client_new(server_name_cstr.ptr, &client);
        if (result != c.GQUIC_OK) {
            return GQuicError.InitFailed;
        }

        return Self{ .handle = client };
    }

    pub fn connect(self: *Self, addr: []const u8) !?*anyopaque {
        const addr_cstr = try std.cstr.addNullByte(std.heap.c_allocator, addr);
        defer std.heap.c_allocator.free(addr_cstr);

        var connection: ?*anyopaque = null;
        const result = c.gquic_client_connect(self.handle, addr_cstr.ptr, &connection);
        
        return switch (result) {
            c.GQUIC_OK => connection,
            c.GQUIC_CONNECTION_FAILED => GQuicError.ConnectionFailed,
            else => GQuicError.InvalidParam,
        };
    }

    pub fn sendData(self: *Self, connection: *anyopaque, data: []const u8) !void {
        const result = c.gquic_client_send_data(
            self.handle,
            connection,
            data.ptr,
            data.len,
        );

        if (result != c.GQUIC_OK) {
            return GQuicError.StreamError;
        }
    }

    pub fn deinit(self: *Self) void {
        if (self.handle) |handle| {
            c.gquic_client_destroy(handle);
            self.handle = null;
        }
    }
};

pub const GQuicServer = struct {
    handle: ?*c.GQuicServer,

    const Self = @This();

    pub fn init(config: ServerConfig) !Self {
        const c_config = c.GQuicConfig{
            .bind_addr = config.bind_addr.ptr,
            .cert_path = config.cert_path.ptr,
            .key_path = config.key_path.ptr,
            .alpn_protocols = config.alpn_protocols.ptr,
            .alpn_count = config.alpn_protocols.len,
            .max_connections = config.max_connections,
            .use_self_signed = if (config.use_self_signed) 1 else 0,
        };

        var server: ?*c.GQuicServer = null;
        const result = c.gquic_server_new(&c_config, &server);
        
        if (result != c.GQUIC_OK) {
            return GQuicError.InitFailed;
        }

        return Self{ .handle = server };
    }

    pub fn start(self: *Self, callback: c.GQuicConnectionCallback, user_data: ?*anyopaque) !void {
        const result = c.gquic_server_start(self.handle, callback, user_data);
        if (result != c.GQUIC_OK) {
            return GQuicError.InitFailed;
        }
    }

    pub fn deinit(self: *Self) void {
        if (self.handle) |handle| {
            c.gquic_server_destroy(handle);
            self.handle = null;
        }
    }
};

pub const ServerConfig = struct {
    bind_addr: [*:0]const u8,
    cert_path: [*:0]const u8,
    key_path: [*:0]const u8,
    alpn_protocols: [*][*:0]const u8,
    max_connections: u32,
    use_self_signed: bool,
};
```

### C Header File

Create `include/gquic_ffi.h`:

```c
#ifndef GQUIC_FFI_H
#define GQUIC_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Error codes
#define GQUIC_OK 0
#define GQUIC_ERROR -1
#define GQUIC_INVALID_PARAM -2
#define GQUIC_CONNECTION_FAILED -3
#define GQUIC_STREAM_ERROR -4

// Opaque types
typedef struct GQuicClient GQuicClient;
typedef struct GQuicServer GQuicServer;

// Callback types
typedef void (*GQuicConnectionCallback)(const void* user_data, int status);
typedef void (*GQuicDataCallback)(const void* user_data, const uint8_t* data, size_t len);

// Configuration
typedef struct {
    const char* bind_addr;
    const char* cert_path;
    const char* key_path;
    const char* const* alpn_protocols;
    size_t alpn_count;
    uint32_t max_connections;
    int use_self_signed;
} GQuicConfig;

// Client API
int gquic_client_new(const char* server_name, GQuicClient** client_out);
int gquic_client_connect(GQuicClient* client, const char* addr, void** connection_out);
int gquic_client_send_data(GQuicClient* client, void* connection, const uint8_t* data, size_t data_len);
void gquic_client_destroy(GQuicClient* client);

// Server API
int gquic_server_new(const GQuicConfig* config, GQuicServer** server_out);
int gquic_server_start(GQuicServer* server, GQuicConnectionCallback callback, const void* user_data);
void gquic_server_destroy(GQuicServer* server);

// Utility
const char* gquic_version(void);
int gquic_init_logging(int level);

#ifdef __cplusplus
}
#endif

#endif // GQUIC_FFI_H
```

### Example Zig Usage

```zig
// main.zig
const std = @import("std");
const gquic = @import("gquic.zig");

pub fn main() !void {
    // Initialize logging
    _ = gquic.c.gquic_init_logging(2); // INFO level

    // Create client
    var client = try gquic.GQuicClient.init("ghostchain.local");
    defer client.deinit();

    // Connect to server
    const connection = try client.connect("127.0.0.1:9090");

    // Send blockchain transaction
    const tx_data = "transaction_data_here";
    try client.sendData(connection, tx_data);

    std.debug.print("Transaction sent successfully\\n");
}
```

## WalletD Integration

### gRPC-over-QUIC for WalletD

```rust
// walletd/src/quic_service.rs
use gquic::prelude::*;
use gquic::proto::walletd::{
    wallet_service_server::{WalletService, WalletServiceServer},
    *,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct WalletServiceImpl {
    // Your wallet state
}

#[tonic::async_trait]
impl WalletService for WalletServiceImpl {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountResponse>, Status> {
        let req = request.into_inner();
        
        // Use gquic crypto backend for key generation
        let backend = gquic::crypto::default_backend();
        let keypair = backend.generate_keypair(
            match req.key_type() {
                KeyType::Ed25519 => gquic::crypto::KeyType::Ed25519,
                KeyType::Secp256k1 => gquic::crypto::KeyType::Secp256k1,
                _ => return Err(Status::invalid_argument("Unsupported key type")),
            }
        ).map_err(|e| Status::internal(e.to_string()))?;

        let response = CreateAccountResponse {
            account_id: uuid::Uuid::new_v4().to_string(),
            public_key: hex::encode(&keypair.public_key.data),
            address: derive_address(&keypair.public_key),
        };

        Ok(Response::new(response))
    }

    async fn sign_data(
        &self,
        request: Request<SignDataRequest>,
    ) -> Result<Response<SignDataResponse>, Status> {
        let req = request.into_inner();
        
        // Load private key for account
        let private_key = load_account_key(&req.account_id)
            .map_err(|e| Status::not_found(e.to_string()))?;

        // Sign with gquic crypto backend
        let backend = gquic::crypto::default_backend();
        let signature = backend.sign(&private_key, &req.data)
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = SignDataResponse {
            signature: signature.data,
            public_key: hex::encode(&private_key.data), // Get public key
            signature_type: match signature.signature_type {
                gquic::crypto::SignatureType::Ed25519 => SignatureType::Ed25519,
                gquic::crypto::SignatureType::EcdsaSecp256k1 => SignatureType::EcdsaSecp256k1,
                _ => return Err(Status::internal("Unsupported signature type")),
            } as i32,
        };

        Ok(Response::new(response))
    }

    // Implement other methods...
}

pub async fn start_walletd_quic_server() -> anyhow::Result<()> {
    let wallet_service = WalletServiceImpl::default();

    // Create gRPC handler for QUIC
    let handler = GrpcHandler::new(WalletServiceServer::new(wallet_service));

    let server = QuicServer::builder()
        .bind("0.0.0.0:9090".parse()?)
        .with_tls_files("certs/walletd.crt", "certs/walletd.key")?
        .with_alpn("grpc")
        .with_handler(Arc::new(handler))
        .build()?;

    tracing::info!("WalletD QUIC server starting on :9090");
    server.run().await
}
```

## GhostD Integration

### P2P Communication over QUIC

```rust
// ghostd/src/p2p/quic_transport.rs
use gquic::prelude::*;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct QuicP2PTransport {
    client: QuicClient,
    server: QuicServer,
    peers: Arc<RwLock<HashMap<PeerId, Connection>>>,
    pool: ConnectionPool,
}

impl QuicP2PTransport {
    pub async fn new(bind_addr: SocketAddr) -> anyhow::Result<Self> {
        let client = QuicClient::builder()
            .server_name("ghostchain.p2p".to_string())
            .with_alpn("ghostchain-p2p")
            .build_client()?;

        let handler = P2PHandler::new();
        let server = QuicServer::builder()
            .bind(bind_addr)
            .with_self_signed_cert()? // Use proper certs in production
            .with_alpn("ghostchain-p2p")
            .with_handler(Arc::new(handler))
            .build()?;

        Ok(Self {
            client,
            server,
            peers: Arc::new(RwLock::new(HashMap::new())),
            pool: ConnectionPool::new(PoolConfig::default()),
        })
    }

    pub async fn connect_peer(&self, peer_id: PeerId, addr: SocketAddr) -> anyhow::Result<()> {
        let connection = self.client.connect(addr).await?;
        
        // Store connection for this peer
        self.peers.write().await.insert(peer_id, connection.clone());
        self.pool.return_connection(addr, connection).await;

        tracing::info!("Connected to peer {} at {}", peer_id, addr);
        Ok(())
    }

    pub async fn broadcast_block(&self, block: &Block) -> anyhow::Result<()> {
        let block_data = serialize_block(block)?;
        let peers = self.peers.read().await;

        let mut tasks = Vec::new();
        for (peer_id, connection) in peers.iter() {
            let peer_id = *peer_id;
            let connection = connection.clone();
            let data = block_data.clone();
            
            tasks.push(tokio::spawn(async move {
                let mut stream = connection.open_uni().await?;
                
                // Send message type + data
                stream.write_all(&[MSG_TYPE_BLOCK]).await?;
                stream.write_all(&data).await?;
                stream.finish().await?;
                
                tracing::debug!("Broadcasted block to peer {}", peer_id);
                Ok::<(), anyhow::Error>(())
            }));
        }

        // Wait for all broadcasts to complete
        for task in tasks {
            if let Err(e) = task.await? {
                tracing::warn!("Failed to broadcast to peer: {}", e);
            }
        }

        Ok(())
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.server.run().await
    }
}

struct P2PHandler;

#[async_trait]
impl ConnectionHandler for P2PHandler {
    async fn handle_connection(
        &self,
        connection: NewConnection,
        _config: Arc<QuicServerConfig>,
    ) -> anyhow::Result<()> {
        let remote_addr = connection.connection.remote_address();
        tracing::info!("New P2P connection from {}", remote_addr);

        // Handle incoming unidirectional streams (broadcasts)
        while let Ok(mut recv) = connection.uni_streams.accept().await {
            tokio::spawn(async move {
                // Read message type
                let mut msg_type = [0u8; 1];
                recv.read_exact(&mut msg_type).await?;

                match msg_type[0] {
                    MSG_TYPE_BLOCK => {
                        let block_data = recv.read_to_end(10 * 1024 * 1024).await?; // 10MB max
                        let block = deserialize_block(&block_data)?;
                        handle_received_block(block).await?;
                    }
                    MSG_TYPE_TRANSACTION => {
                        let tx_data = recv.read_to_end(1024 * 1024).await?; // 1MB max
                        let tx = deserialize_transaction(&tx_data)?;
                        handle_received_transaction(tx).await?;
                    }
                    _ => {
                        tracing::warn!("Unknown message type: {}", msg_type[0]);
                    }
                }

                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(())
    }
}
```

## GhostBridge Integration

### QUIC-to-HTTP/gRPC Proxy

```rust
// ghostbridge/src/proxy.rs
use gquic::prelude::*;
use hyper::{Body, Client, Request, Response};

pub struct QuicHttpBridge {
    quic_server: QuicServer,
    http_client: Client<hyper::client::HttpConnector>,
}

impl QuicHttpBridge {
    pub async fn new() -> anyhow::Result<Self> {
        let handler = BridgeHandler::new();
        
        let server = QuicServer::builder()
            .bind("0.0.0.0:443".parse()?)
            .with_tls_files("certs/bridge.crt", "certs/bridge.key")?
            .with_alpn("h3")        // HTTP/3
            .with_alpn("grpc")      // gRPC-over-QUIC
            .with_alpn("bridge")    // Custom protocol
            .with_handler(Arc::new(handler))
            .build()?;

        Ok(Self {
            quic_server: server,
            http_client: Client::new(),
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("GhostBridge QUIC proxy starting");
        self.quic_server.run().await
    }
}

struct BridgeHandler {
    http_client: Client<hyper::client::HttpConnector>,
}

#[async_trait]
impl ConnectionHandler for BridgeHandler {
    async fn handle_connection(
        &self,
        connection: NewConnection,
        _config: Arc<QuicServerConfig>,
    ) -> anyhow::Result<()> {
        // Determine protocol from ALPN
        let protocol = detect_protocol(&connection)?;

        match protocol {
            Protocol::Http3 => self.handle_http3_connection(connection).await?,
            Protocol::Grpc => self.handle_grpc_connection(connection).await?,
            Protocol::Bridge => self.handle_bridge_connection(connection).await?,
        }

        Ok(())
    }
}

impl BridgeHandler {
    async fn handle_grpc_connection(&self, connection: NewConnection) -> anyhow::Result<()> {
        while let Ok((mut send, mut recv)) = connection.bi_streams.accept().await {
            let client = self.http_client.clone();
            
            tokio::spawn(async move {
                // Read gRPC request from QUIC stream
                let grpc_data = recv.read_to_end(10 * 1024 * 1024).await?;
                
                // Forward to HTTP/2 gRPC backend
                let http_request = Request::builder()
                    .method("POST")
                    .uri("http://walletd-backend:8080/WalletService/CreateAccount")
                    .header("content-type", "application/grpc")
                    .body(Body::from(grpc_data))?;

                let response = client.request(http_request).await?;
                let response_data = hyper::body::to_bytes(response.into_body()).await?;

                // Send response back over QUIC
                send.write_all(&response_data).await?;
                send.finish().await?;

                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(())
    }
}
```

## Example Projects

### Complete WalletD Service

```rust
// walletd-quic/src/main.rs
use gquic::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let bind_addr = env::var("WALLETD_BIND")
        .unwrap_or_else(|_| "0.0.0.0:9090".to_string())
        .parse()?;

    let wallet_service = WalletServiceImpl::new().await?;
    
    let server = QuicServer::builder()
        .bind(bind_addr)
        .with_tls_files(
            &env::var("TLS_CERT_PATH").unwrap_or_else(|_| "certs/server.crt".to_string()),
            &env::var("TLS_KEY_PATH").unwrap_or_else(|_| "certs/server.key".to_string()),
        )?
        .with_alpn("grpc")
        .with_handler(Arc::new(GrpcHandler::new(wallet_service)))
        .max_concurrent_bidi_streams(1000)
        .build()?;

    tracing::info!("WalletD QUIC server listening on {}", bind_addr);
    server.run().await
}
```

### Zig Client Example

```zig
// zwallet/src/quic_client.zig
const std = @import("std");
const gquic = @import("gquic.zig");

pub const WalletClient = struct {
    client: gquic.GQuicClient,
    connection: ?*anyopaque,

    const Self = @This();

    pub fn init(server_addr: []const u8) !Self {
        var client = try gquic.GQuicClient.init("walletd.ghostchain.local");
        const connection = try client.connect(server_addr);

        return Self{
            .client = client,
            .connection = connection,
        };
    }

    pub fn createAccount(self: *Self, name: []const u8, passphrase: []const u8) ![]u8 {
        // Serialize gRPC request
        const request = try serializeCreateAccountRequest(name, passphrase);
        defer std.heap.c_allocator.free(request);

        // Send over QUIC
        try self.client.sendData(self.connection.?, request);

        // TODO: Read response
        return "account_created";
    }

    pub fn deinit(self: *Self) void {
        self.client.deinit();
    }
};

// Usage
pub fn main() !void {
    var wallet_client = try WalletClient.init("127.0.0.1:9090");
    defer wallet_client.deinit();

    const account_id = try wallet_client.createAccount("test_account", "secure_passphrase");
    std.debug.print("Created account: {}\\n", .{account_id});
}
```

## Performance Considerations

### Connection Pooling for High Throughput

```rust
// High-performance setup
let pool_config = PoolConfig::builder()
    .max_connections_per_endpoint(100)     // Scale with server capacity
    .max_connection_age(Duration::from_secs(3600))  // 1 hour
    .max_idle_time(Duration::from_secs(300))        // 5 minutes
    .enable_multiplexing(true)
    .max_concurrent_streams(500)           // High concurrency
    .build();

let server_config = QuicServerConfig::builder()
    .max_concurrent_bidi_streams(5000)     // Scale with hardware
    .max_concurrent_uni_streams(5000)
    .max_idle_timeout(Duration::from_secs(60))
    .keep_alive_interval(Duration::from_secs(20))
    .build()?;
```

### Monitoring and Metrics

```rust
#[cfg(feature = "metrics")]
{
    use gquic::metrics::get_metrics;
    
    // Periodic metrics reporting
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            let metrics = get_metrics().get_metrics().await;
            
            tracing::info!(
                "QUIC metrics - Active: {}, Total: {}, Failed: {}, Latency: {:.2}ms",
                metrics.connection.active_connections,
                metrics.connection.total_connections,
                metrics.connection.failed_connections,
                metrics.connection.average_latency_ms
            );
        }
    });
}
```

### OS-Level Tuning

```bash
# /etc/sysctl.conf
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.core.rmem_default = 16777216
net.core.wmem_default = 16777216
net.ipv4.udp_mem = 102400 873800 16777216
net.ipv4.udp_rmem_min = 8192
net.ipv4.udp_wmem_min = 8192

# Apply changes
sudo sysctl -p
```

## Integration Checklist

### For Rust Projects
- [ ] Add gquic dependency with appropriate features
- [ ] Implement connection pooling for clients
- [ ] Use appropriate ALPN protocols
- [ ] Configure TLS certificates properly
- [ ] Add metrics monitoring
- [ ] Handle connection failures gracefully

### For Zig Projects
- [ ] Build gquic with FFI support
- [ ] Create Zig bindings for required functions
- [ ] Link against libgquic in build.zig
- [ ] Handle C memory management properly
- [ ] Test FFI integration thoroughly

### For gRPC Services
- [ ] Define protobuf schemas
- [ ] Implement gRPC-over-QUIC handlers
- [ ] Configure ALPN for "grpc" protocol
- [ ] Test with various gRPC clients
- [ ] Monitor performance vs HTTP/2

### Production Deployment
- [ ] Use proper TLS certificates
- [ ] Configure firewalls for UDP traffic
- [ ] Monitor connection metrics
- [ ] Set up health checks
- [ ] Configure log rotation
- [ ] Plan for graceful shutdowns

## Troubleshooting

### Common Integration Issues

1. **FFI Compilation Errors**
   ```bash
   # Ensure proper feature flags
   cargo build --release --features ffi
   
   # Check library output
   ldd target/release/libgquic.so
   ```

2. **Connection Failures**
   ```rust
   // Add detailed error logging
   match client.connect(addr).await {
       Err(e) => {
           tracing::error!("Connection failed: {}", e);
           // Check network, firewall, certificates
       }
       Ok(conn) => { /* success */ }
   }
   ```

3. **Performance Issues**
   ```rust
   // Monitor metrics
   let metrics = get_metrics().get_metrics().await;
   if metrics.connection.average_latency_ms > 100.0 {
       tracing::warn!("High latency detected: {:.2}ms", 
                     metrics.connection.average_latency_ms);
   }
   ```

For more detailed troubleshooting, see [DOCS.md](DOCS.md#troubleshooting).

---

This integration guide provides the foundation for connecting your GhostChain ecosystem projects with high-performance QUIC transport. Start with the basic examples and scale up based on your specific requirements.