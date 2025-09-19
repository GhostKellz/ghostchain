# 🔗 WalletD Integration Guide for Crypto Projects

This guide shows how to integrate **walletd** with your crypto projects in the GhostChain ecosystem, with a focus on **ZVM** and other blockchain components.

## Table of Contents

- [Overview](#overview)
- [ZVM Integration](#zvm-integration)
- [Rust Project Integration](#rust-project-integration)
- [Zig Project Integration](#zig-project-integration)
- [API Reference](#api-reference)
- [ZQUIC Transport](#zquic-transport)
- [Authentication & Security](#authentication--security)
- [Example Projects](#example-projects)
- [Performance Optimization](#performance-optimization)

---

## Overview

**walletd** provides secure wallet operations for the GhostChain ecosystem with multiple integration methods:

| Project Type | Integration Method | Transport | Use Case |
|-------------|-------------------|-----------|----------|
| **ZVM** | ZQUIC + gRPC | Zig ZQUIC | Smart contract wallet operations |
| **Rust Services** | Native ZQUIC | Zig ZQUIC | High-performance blockchain services |
| **Zig Projects** | Direct ZQUIC | Zig ZQUIC | Native Zig blockchain components |
| **Web Apps** | REST API | HTTP/WebSocket | Frontend wallet interfaces |
| **CLI Tools** | gRPC Client | HTTP/2 | Command-line blockchain tools |

### **WalletD Features**

- 🔐 **ZID-based Authentication** - Web5 identity management
- ⚡ **ZQUIC Transport** - High-performance Zig-based networking
- 🔑 **Multi-Algorithm Crypto** - Ed25519, Secp256k1, Secp256r1
- 🤝 **Multi-Signature Support** - Coordinated signing via ZQUIC
- 📡 **Real-time Updates** - Live balance and transaction streams
- 🌐 **Cross-Platform** - Rust, Zig, and Web compatibility

---

## ZVM Integration

### **Smart Contract Wallet Operations**

ZVM (Zig Virtual Machine) can integrate with walletd for secure wallet operations during smart contract execution.

#### **1. Add WalletD Client to ZVM**

```zig
// zvm/src/wallet/client.zig
const std = @import("std");
const zquic = @import("zquic");
const Allocator = std.mem.Allocator;

pub const WalletClient = struct {
    allocator: Allocator,
    zquic_client: zquic.Client,
    walletd_endpoint: []const u8,

    const Self = @This();

    pub fn init(allocator: Allocator, walletd_endpoint: []const u8) !Self {
        const client = try zquic.Client.init(allocator, .{
            .server_name = "walletd.ghostchain.local",
            .alpn_protocols = &[_][]const u8{ "ghostchain-v1", "grpc" },
        });

        return Self{
            .allocator = allocator,
            .zquic_client = client,
            .walletd_endpoint = walletd_endpoint,
        };
    }

    pub fn deinit(self: *Self) void {
        self.zquic_client.deinit();
    }
};
```

#### **2. Wallet Operations for Smart Contracts**

```zig
// zvm/src/wallet/operations.zig
const WalletRequest = struct {
    operation: enum { sign_transaction, get_balance, create_account },
    wallet_id: []const u8,
    data: []const u8,
};

const WalletResponse = struct {
    success: bool,
    result: []const u8,
    error_message: ?[]const u8,
};

pub fn signContractTransaction(
    self: *WalletClient,
    wallet_id: []const u8,
    contract_address: []const u8,
    function_call: []const u8,
    gas_limit: u64,
) !WalletResponse {
    const request = WalletRequest{
        .operation = .sign_transaction,
        .wallet_id = wallet_id,
        .data = try std.json.stringifyAlloc(self.allocator, .{
            .to = contract_address,
            .data = function_call,
            .gas_limit = gas_limit,
            .contract_call = true,
        }),
    };

    const request_bytes = try std.json.stringifyAlloc(self.allocator, request);
    defer self.allocator.free(request_bytes);

    // Send via ZQUIC
    const response_bytes = try self.zquic_client.send(
        self.walletd_endpoint,
        request_bytes,
    );
    defer self.allocator.free(response_bytes);

    return try std.json.parseFromSlice(WalletResponse, self.allocator, response_bytes);
}

pub fn getContractBalance(
    self: *WalletClient,
    wallet_id: []const u8,
    token_contract: ?[]const u8,
) !WalletResponse {
    const request = WalletRequest{
        .operation = .get_balance,
        .wallet_id = wallet_id,
        .data = if (token_contract) |contract| contract else "",
    };

    const request_bytes = try std.json.stringifyAlloc(self.allocator, request);
    defer self.allocator.free(request_bytes);

    const response_bytes = try self.zquic_client.send(
        self.walletd_endpoint,
        request_bytes,
    );
    defer self.allocator.free(response_bytes);

    return try std.json.parseFromSlice(WalletResponse, self.allocator, response_bytes);
}
```

#### **3. ZVM Smart Contract Integration**

```zig
// zvm/src/runtime/wallet_integration.zig
const VM = @import("../vm.zig").VM;
const WalletClient = @import("../wallet/client.zig").WalletClient;

pub const WalletIntegration = struct {
    wallet_client: *WalletClient,

    pub fn init(wallet_client: *WalletClient) WalletIntegration {
        return .{ .wallet_client = wallet_client };
    }

    // Smart contract function: wallet.sign(data)
    pub fn vmWalletSign(vm: *VM, args: []const u8) ![]const u8 {
        const self = vm.getExtension(WalletIntegration);
        
        // Parse arguments from smart contract
        const parsed = try std.json.parseFromSlice(struct {
            wallet_id: []const u8,
            data: []const u8,
        }, vm.allocator, args);

        // Sign via walletd
        const response = try self.wallet_client.signContractTransaction(
            parsed.wallet_id,
            vm.current_contract_address,
            parsed.data,
            vm.gas_limit,
        );

        if (!response.success) {
            return error.WalletSigningFailed;
        }

        return response.result;
    }

    // Smart contract function: wallet.balance()
    pub fn vmWalletBalance(vm: *VM, args: []const u8) ![]const u8 {
        const self = vm.getExtension(WalletIntegration);
        
        const parsed = try std.json.parseFromSlice(struct {
            wallet_id: []const u8,
            token_contract: ?[]const u8 = null,
        }, vm.allocator, args);

        const response = try self.wallet_client.getContractBalance(
            parsed.wallet_id,
            parsed.token_contract,
        );

        if (!response.success) {
            return error.WalletBalanceFailed;
        }

        return response.result;
    }
};
```

#### **4. ZVM Build Configuration**

```zig
// zvm/build.zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    // ... existing ZVM build configuration ...

    // Add walletd integration
    const walletd_integration = b.createModule(.{
        .root_source_file = .{ .path = "src/wallet/integration.zig" },
        .dependencies = &.{
            .{ .name = "zquic", .module = zquic_dep.module("zquic") },
        },
    });

    exe.root_module.addImport("walletd", walletd_integration);
    
    // Link ZQUIC for walletd communication
    exe.linkLibrary(zquic_lib);
}
```

---

## Rust Project Integration

### **Native ZQUIC Integration**

For Rust blockchain services that need high-performance wallet operations:

#### **1. Add WalletD Client Dependency**

```toml
# Cargo.toml
[dependencies]
walletd-client = { git = "https://github.com/ghostkellz/walletd", features = ["zquic-client"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

#### **2. Create WalletD Client**

```rust
// src/wallet_client.rs
use walletd_client::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Clone)]
pub struct BlockchainWalletClient {
    client: ZQuicWalletClient,
}

impl BlockchainWalletClient {
    pub async fn new(walletd_endpoint: &str) -> Result<Self> {
        let client = ZQuicWalletClient::builder()
            .endpoint(walletd_endpoint)
            .server_name("walletd.ghostchain.local")
            .alpn_protocols(vec!["ghostchain-v1", "grpc"])
            .connect()
            .await?;

        Ok(Self { client })
    }

    pub async fn sign_block(
        &self,
        wallet_id: &str,
        block_hash: &[u8],
        passphrase: Option<&str>,
    ) -> Result<Vec<u8>> {
        let request = SignTransactionRequest {
            wallet_id: wallet_id.to_string(),
            data: block_hash.to_vec(),
            use_enhanced_crypto: Some(true),
            passphrase: passphrase.map(|s| s.to_string()),
        };

        let response = self.client.sign_data(request).await?;
        Ok(response.signature)
    }

    pub async fn get_validator_balance(&self, wallet_id: &str) -> Result<u64> {
        let response = self.client.get_balance(wallet_id).await?;
        Ok(response.balance.parse()?)
    }

    pub async fn create_multisig_for_consensus(
        &self,
        validators: Vec<&str>,
        threshold: usize,
    ) -> Result<String> {
        let participants = validators
            .into_iter()
            .map(|wallet_id| MultisigParticipant {
                wallet_id: wallet_id.to_string(),
                public_key: "".to_string(), // Would be retrieved from walletd
                endpoint: None,
            })
            .collect();

        let request = CreateMultisigRequest {
            name: "consensus-validators".to_string(),
            required_signatures: threshold,
            participants,
            use_zquic_coordination: Some(true),
        };

        let response = self.client.create_multisig(request).await?;
        Ok(response.multisig_id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SignTransactionRequest {
    pub wallet_id: String,
    pub data: Vec<u8>,
    pub use_enhanced_crypto: Option<bool>,
    pub passphrase: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MultisigParticipant {
    pub wallet_id: String,
    pub public_key: String,
    pub endpoint: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateMultisigRequest {
    pub name: String,
    pub required_signatures: usize,
    pub participants: Vec<MultisigParticipant>,
    pub use_zquic_coordination: Option<bool>,
}
```

#### **3. Integration Example: Consensus Module**

```rust
// src/consensus/mod.rs
use crate::wallet_client::BlockchainWalletClient;
use anyhow::Result;

pub struct ConsensusEngine {
    wallet_client: BlockchainWalletClient,
    validator_wallet: String,
}

impl ConsensusEngine {
    pub async fn new(
        walletd_endpoint: &str,
        validator_wallet: String,
    ) -> Result<Self> {
        let wallet_client = BlockchainWalletClient::new(walletd_endpoint).await?;

        Ok(Self {
            wallet_client,
            validator_wallet,
        })
    }

    pub async fn sign_block(&self, block_hash: &[u8]) -> Result<Vec<u8>> {
        self.wallet_client
            .sign_block(&self.validator_wallet, block_hash, None)
            .await
    }

    pub async fn setup_validator_multisig(
        &self,
        validators: Vec<String>,
        threshold: usize,
    ) -> Result<String> {
        let validator_refs: Vec<&str> = validators.iter().map(|s| s.as_str()).collect();
        
        self.wallet_client
            .create_multisig_for_consensus(validator_refs, threshold)
            .await
    }
}
```

---

## Zig Project Integration

### **Direct ZQUIC Integration**

For native Zig projects in the GhostChain ecosystem:

#### **1. Project Structure**

```
zig-project/
├── build.zig
├── src/
│   ├── main.zig
│   ├── walletd/
│   │   ├── client.zig
│   │   ├── types.zig
│   │   └── operations.zig
│   └── ...
└── deps/
    └── zquic/          # ZQUIC dependency
```

#### **2. WalletD Client Implementation**

```zig
// src/walletd/client.zig
const std = @import("std");
const zquic = @import("zquic");
const types = @import("types.zig");

pub const WalletDClient = struct {
    allocator: std.mem.Allocator,
    zquic_client: zquic.Client,
    endpoint: []const u8,

    const Self = @This();

    pub fn init(
        allocator: std.mem.Allocator,
        endpoint: []const u8,
    ) !Self {
        const client = try zquic.Client.init(allocator, .{
            .server_name = "walletd.ghostchain.local",
            .alpn_protocols = &[_][]const u8{"ghostchain-v1"},
        });

        return Self{
            .allocator = allocator,
            .zquic_client = client,
            .endpoint = endpoint,
        };
    }

    pub fn deinit(self: *Self) void {
        self.zquic_client.deinit();
    }

    pub fn createWallet(
        self: *Self,
        name: []const u8,
        account_type: types.AccountType,
    ) !types.CreateWalletResponse {
        const request = types.CreateWalletRequest{
            .name = name,
            .account_type = account_type,
            .use_zquic = true,
        };

        const request_json = try std.json.stringifyAlloc(
            self.allocator,
            request,
        );
        defer self.allocator.free(request_json);

        const response_json = try self.zquic_client.post(
            self.endpoint,
            "/api/v1/wallets",
            request_json,
        );
        defer self.allocator.free(response_json);

        return try std.json.parseFromSlice(
            types.CreateWalletResponse,
            self.allocator,
            response_json,
        );
    }

    pub fn signTransaction(
        self: *Self,
        wallet_id: []const u8,
        transaction_data: []const u8,
    ) !types.SignTransactionResponse {
        const request = types.SignTransactionRequest{
            .wallet_id = wallet_id,
            .data = transaction_data,
            .use_enhanced_crypto = true,
        };

        const request_json = try std.json.stringifyAlloc(
            self.allocator,
            request,
        );
        defer self.allocator.free(request_json);

        const response_json = try self.zquic_client.post(
            self.endpoint,
            "/api/v1/transactions/sign",
            request_json,
        );
        defer self.allocator.free(response_json);

        return try std.json.parseFromSlice(
            types.SignTransactionResponse,
            self.allocator,
            response_json,
        );
    }

    pub fn streamBalance(
        self: *Self,
        wallet_id: []const u8,
        callback: fn (types.BalanceUpdate) void,
    ) !void {
        const ws_url = try std.fmt.allocPrint(
            self.allocator,
            "ws://{s}/api/v1/wallets/{s}/balance/stream",
            .{ self.endpoint, wallet_id },
        );
        defer self.allocator.free(ws_url);

        // Connect to WebSocket via ZQUIC
        const ws_connection = try self.zquic_client.connectWebSocket(ws_url);
        defer ws_connection.close();

        while (true) {
            const message = try ws_connection.receive();
            defer self.allocator.free(message);

            const balance_update = try std.json.parseFromSlice(
                types.BalanceUpdate,
                self.allocator,
                message,
            );

            callback(balance_update);
        }
    }
};
```

#### **3. Type Definitions**

```zig
// src/walletd/types.zig
const std = @import("std");

pub const AccountType = enum {
    ed25519,
    secp256k1,
    secp256r1,
};

pub const CreateWalletRequest = struct {
    name: []const u8,
    account_type: AccountType,
    use_zquic: bool,
};

pub const CreateWalletResponse = struct {
    wallet: Wallet,
    zquic_enabled: bool,
};

pub const Wallet = struct {
    id: []const u8,
    name: []const u8,
    address: []const u8,
    public_key: []const u8,
    account_type: []const u8,
    network: []const u8,
    created_at: []const u8,
};

pub const SignTransactionRequest = struct {
    wallet_id: []const u8,
    data: []const u8,
    use_enhanced_crypto: bool,
};

pub const SignTransactionResponse = struct {
    signature: []const u8,
    signed_via_zcrypto: bool,
    status: []const u8,
};

pub const BalanceUpdate = struct {
    wallet_id: []const u8,
    balance: Balance,
    timestamp: []const u8,
    source: []const u8,
    event_type: []const u8,
};

pub const Balance = struct {
    amount: []const u8,
    token_symbol: []const u8,
    decimals: u32,
};
```

---

## API Reference

### **Core Endpoints**

#### **Wallet Operations**
```
POST   /api/v1/wallets                    # Create wallet
GET    /api/v1/wallets/:id               # Get wallet info
GET    /api/v1/wallets/:id/balance       # Get balance
WS     /api/v1/wallets/:id/balance/stream # Stream balance updates
```

#### **Transaction Operations**
```
POST   /api/v1/transactions              # Submit transaction
GET    /api/v1/transactions/:id          # Get transaction
POST   /api/v1/transactions/sign         # Sign transaction
```

#### **Multi-Signature Operations**
```
POST   /api/v1/multisig/create           # Create multisig
POST   /api/v1/multisig/:id/sign         # Sign multisig transaction
POST   /api/v1/multisig/:id/broadcast    # Broadcast multisig
```

#### **ZQUIC Status**
```
GET    /api/v1/zquic/status              # ZQUIC transport status
GET    /api/v1/zquic/peers               # Connected ZQUIC peers
```

### **Request/Response Examples**

#### **Create Wallet**
```json
// POST /api/v1/wallets
{
  "name": "my-project-wallet",
  "account_type": "ed25519",
  "use_zquic": true
}

// Response
{
  "wallet": {
    "id": "wallet_abc123",
    "name": "my-project-wallet",
    "address": "ghost1abc...",
    "public_key": "0x123...",
    "account_type": "ed25519",
    "network": "ghostchain"
  },
  "zquic_enabled": true
}
```

#### **Sign Transaction**
```json
// POST /api/v1/transactions/sign
{
  "wallet_id": "wallet_abc123",
  "data": "0x123...",
  "use_enhanced_crypto": true
}

// Response
{
  "signature": "0xabc...",
  "signed_via_zcrypto": true,
  "status": "signed"
}
```

---

## ZQUIC Transport

### **Connection Configuration**

```rust
// Rust configuration
let client = ZQuicClient::builder()
    .endpoint("127.0.0.1:9090")
    .server_name("walletd.ghostchain.local")
    .alpn_protocols(vec!["ghostchain-v1", "grpc"])
    .max_idle_timeout(30_000)
    .enable_0rtt(true)
    .build()?;
```

```zig
// Zig configuration
const client = try zquic.Client.init(allocator, .{
    .server_name = "walletd.ghostchain.local",
    .alpn_protocols = &[_][]const u8{ "ghostchain-v1", "grpc" },
    .max_idle_timeout = 30000,
    .enable_0rtt = true,
});
```

### **Performance Optimization**

#### **Connection Pooling**
```rust
// Reuse connections for multiple requests
let pool = ConnectionPool::new(PoolConfig {
    max_connections_per_endpoint: 10,
    max_idle_time: Duration::from_secs(300),
    enable_multiplexing: true,
});
```

#### **Batch Operations**
```rust
// Submit multiple operations in a single ZQUIC stream
let batch_request = BatchWalletRequest {
    operations: vec![
        WalletOperation::GetBalance("wallet1".to_string()),
        WalletOperation::GetBalance("wallet2".to_string()),
        WalletOperation::SignTransaction(sign_request),
    ],
};

let responses = client.batch_execute(batch_request).await?;
```

---

## Authentication & Security

### **ZID Authentication**

```rust
// Create authenticated client
let auth_client = AuthenticatedWalletClient::new(
    zquic_client,
    AuthConfig {
        identity: zid_identity,
        session_timeout: Duration::from_secs(3600),
        require_passphrase: true,
    },
).await?;
```

### **Secure Communication**

```toml
# walletd.toml
[quic.tls]
cert_path = "certs/walletd.crt"
key_path = "certs/walletd.key"
use_self_signed = false  # Use proper certs in production

[security]
require_auth = true
enable_hmac_auth = true
session_timeout_seconds = 3600
```

---

## Example Projects

### **Complete ZVM Integration**

```zig
// zvm/src/main.zig
const std = @import("std");
const WalletDClient = @import("walletd").WalletDClient;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Initialize walletd client
    var wallet_client = try WalletDClient.init(
        allocator,
        "127.0.0.1:9090",
    );
    defer wallet_client.deinit();

    // Create wallet for smart contracts
    const wallet_response = try wallet_client.createWallet(
        "zvm-execution-wallet",
        .ed25519,
    );

    std.log.info("Created wallet: {s}", .{wallet_response.wallet.id});

    // Sign smart contract transaction
    const signature = try wallet_client.signTransaction(
        wallet_response.wallet.id,
        "contract_call_data",
    );

    std.log.info("Signed transaction: {s}", .{signature.signature});
}
```

### **Rust Blockchain Service**

```rust
// main.rs
use walletd_client::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize walletd client
    let wallet_client = ZQuicWalletClient::builder()
        .endpoint("127.0.0.1:9090")
        .connect()
        .await?;

    // Create validator wallet
    let wallet = wallet_client.create_wallet(CreateWalletRequest {
        name: "validator-1".to_string(),
        account_type: Some("ed25519".to_string()),
        use_zquic: Some(true),
        ..Default::default()
    }).await?;

    println!("Created validator wallet: {}", wallet.wallet.id);

    // Stream balance updates
    let mut balance_stream = wallet_client
        .stream_balance(&wallet.wallet.id)
        .await?;

    while let Some(update) = balance_stream.next().await {
        println!("Balance update: {:?}", update);
    }

    Ok(())
}
```

---

## Performance Optimization

### **High-Throughput Configuration**

```toml
# walletd.toml - High-performance setup
[quic]
enabled = true
bind_address = "0.0.0.0:9090"
max_concurrent_streams = 5000
max_idle_timeout = 60000
enable_0rtt = true

[features]
enable_metrics = true
enable_batch_operations = true
```

### **Monitoring**

```rust
// Monitor ZQUIC performance
let metrics = client.get_metrics().await?;
println!("Active connections: {}", metrics.active_connections);
println!("Average latency: {:.2}ms", metrics.average_latency_ms);
println!("Throughput: {} req/s", metrics.requests_per_second);
```

---

## Deployment

### **Docker Configuration**

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features full-integration

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/walletd /usr/local/bin/
COPY walletd.toml /etc/walletd/
EXPOSE 8080 9090 50051
CMD ["walletd", "--config", "/etc/walletd/walletd.toml", "start"]
```

### **Production Setup**

```yaml
# docker-compose.yml
version: '3.8'
services:
  walletd:
    build: .
    ports:
      - "8080:8080"   # REST API
      - "9090:9090"   # ZQUIC
      - "50051:50051" # gRPC
    environment:
      - RUST_LOG=info
      - WALLETD_CONFIG=/etc/walletd/walletd.toml
    volumes:
      - ./certs:/etc/walletd/certs
      - ./data:/var/lib/walletd
```

---

This integration guide provides everything needed to connect your crypto projects with walletd's secure wallet infrastructure using high-performance ZQUIC transport. Start with the examples most relevant to your project and scale up based on your specific requirements.

For more detailed examples and troubleshooting, see the [walletd repository](https://github.com/ghostkellz/walletd) and [ZQUIC documentation](https://github.com/ghostkellz/zquic).
