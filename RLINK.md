# RLINK - Rust Link Protocol Specification
## Bidirectional GhostChain Bridge Communication Protocol

*Version: 1.0.0*  
*Date: July 11, 2025*  
*Context: ZNS ↔ GhostChain Integration via GhostBridge*

---

## 🌉 Overview

RLINK (Rust Link) is a bidirectional communication protocol specification for the GhostChain ecosystem bridges. It defines how Zig-based services (ZNS, ZVM) communicate with Rust-based services (GhostChain, GhostLink) through the hybrid GhostBridge architecture.

### Architecture Flow

```
┌─────────────┐    RLINK     ┌──────────────┐    GhostLink    ┌─────────────┐
│ ZNS (Zig)   │ ────────────▶ GhostBridge   │ ──────────────▶ │ GhostChain  │
│ ZVM (Zig)   │   gRPC/QUIC  │ (Zig + Rust) │   Native Rust   │ (Rust)      │
│ Enoc (Zig)  │ ◀────────────│ Hybrid       │ ◀──────────────│ Nodes       │
└─────────────┘              └──────────────┘                 └─────────────┘
```

---

## 🏗️ Bridge Components

### 1. **GhostBridge** - Hybrid Zig/Rust Bridge
- **Server Side (Zig)**: Handles gRPC requests from ZNS/ZVM
- **Client Side (Rust)**: Communicates with GhostChain via GhostLink
- **Location**: `github.com/ghostkellz/ghostbridge`

### 2. **GhostLink** - Pure Rust Client Library
- **Type**: Rust gRPC client for GhostChain communication
- **Used by**: GhostBridge Rust client, GhostChain services
- **Location**: `github.com/ghostkellz/ghostlink`

---

## 📋 RLINK Protocol Specification

### Protocol Stack

```
┌─────────────────┐
│ Application     │  ZNS Domain Resolution, ZVM Execution
├─────────────────┤
│ RLINK Protocol  │  Message Routing, Error Handling, State Sync
├─────────────────┤
│ Transport       │  gRPC over QUIC/HTTP2, TLS encryption
├─────────────────┤
│ Network         │  TCP/UDP, IPv4/IPv6
└─────────────────┘
```

### Message Types

#### 1. **Domain Resolution Messages**
```protobuf
// ZNS → GhostBridge → GhostChain
message RLinkDomainResolveRequest {
    string request_id = 1;              // Unique request identifier
    string domain = 2;                  // Domain to resolve (.ghost, .zns, etc.)
    repeated string record_types = 3;   // DNS record types needed
    RLinkMetadata metadata = 4;         // Request metadata
    uint32 timeout_ms = 5;              // Request timeout
}

message RLinkDomainResolveResponse {
    string request_id = 1;              // Matching request ID
    RLinkStatus status = 2;             // Operation status
    repeated RLinkDNSRecord records = 3; // Resolved records
    RLinkDomainInfo domain_info = 4;    // Domain metadata
    uint64 resolution_time_ms = 5;      // Resolution latency
}
```

#### 2. **Domain Registration Messages**
```protobuf
// ZNS → GhostBridge → GhostChain
message RLinkDomainRegisterRequest {
    string request_id = 1;              // Unique request identifier
    string domain = 2;                  // Domain to register
    string owner_address = 3;           // GhostChain address
    repeated RLinkDNSRecord records = 4; // Initial DNS records
    bytes signature = 5;                // Owner signature
    RLinkMetadata metadata = 6;         // Request metadata
}

message RLinkDomainRegisterResponse {
    string request_id = 1;              // Matching request ID
    RLinkStatus status = 2;             // Operation status
    string transaction_hash = 3;        // Blockchain transaction hash
    uint64 block_height = 4;            // Block height
    string contract_address = 5;        // Smart contract address
}
```

#### 3. **Blockchain State Sync Messages**
```protobuf
// GhostChain → GhostBridge → ZNS
message RLinkStateUpdateEvent {
    string event_id = 1;                // Unique event identifier
    RLinkEventType event_type = 2;      // Type of state change
    string domain = 3;                  // Affected domain
    bytes event_data = 4;               // Event payload
    uint64 block_height = 5;            // Block height
    uint64 timestamp = 6;               // Event timestamp
}

enum RLinkEventType {
    DOMAIN_REGISTERED = 0;
    DOMAIN_TRANSFERRED = 1;
    DOMAIN_UPDATED = 2;
    DOMAIN_EXPIRED = 3;
    DOMAIN_RENEWED = 4;
}
```

#### 4. **Health and Status Messages**
```protobuf
message RLinkHealthCheckRequest {
    string service_id = 1;              // Service identifier
    uint64 timestamp = 2;               // Request timestamp
}

message RLinkHealthCheckResponse {
    string service_id = 1;              // Service identifier
    RLinkServiceStatus status = 2;      // Service health status
    RLinkMetrics metrics = 3;           // Service metrics
    uint64 uptime_seconds = 4;          // Service uptime
}
```

### Common Data Structures

```protobuf
// Request/Response metadata
message RLinkMetadata {
    string client_version = 1;          // Client version
    string protocol_version = 2;        // RLINK protocol version
    map<string, string> headers = 3;    // Custom headers
    uint64 timestamp = 4;               // Request timestamp
}

// Operation status
message RLinkStatus {
    RLinkStatusCode code = 1;           // Status code
    string message = 2;                 // Status message
    string details = 3;                 // Detailed error info
}

enum RLinkStatusCode {
    SUCCESS = 0;
    INVALID_REQUEST = 1;
    DOMAIN_NOT_FOUND = 2;
    PERMISSION_DENIED = 3;
    TIMEOUT = 4;
    INTERNAL_ERROR = 5;
    BRIDGE_UNAVAILABLE = 6;
}

// DNS record representation
message RLinkDNSRecord {
    string name = 1;                    // Record name
    string type = 2;                    // Record type (A, AAAA, TXT, etc.)
    string value = 3;                   // Record value
    uint32 ttl = 4;                     // Time to live
    bytes signature = 5;                // Record signature
}

// Domain information
message RLinkDomainInfo {
    string owner = 1;                   // Domain owner address
    uint64 expiry_date = 2;             // Domain expiry timestamp
    string registrar = 3;               // Registrar information
    map<string, string> metadata = 4;   // Additional metadata
}
```

---

## 🔧 Implementation Guide

### 1. **ZNS Integration** (Zig Side)

```zig
// src/rlink/client.zig
const std = @import("std");
const grpc = @import("grpc");

pub const RLinkClient = struct {
    allocator: std.mem.Allocator,
    grpc_client: grpc.Client,
    bridge_endpoint: []const u8,
    
    pub fn init(allocator: std.mem.Allocator, bridge_endpoint: []const u8) !RLinkClient {
        return RLinkClient{
            .allocator = allocator,
            .grpc_client = try grpc.Client.init(allocator, bridge_endpoint),
            .bridge_endpoint = bridge_endpoint,
        };
    }
    
    pub fn deinit(self: *RLinkClient) void {
        self.grpc_client.deinit();
    }
    
    // Domain resolution via RLINK
    pub fn resolveDomain(self: *RLinkClient, domain: []const u8, record_types: []const []const u8) !RLinkDomainResolveResponse {
        const request = RLinkDomainResolveRequest{
            .request_id = try self.generateRequestId(),
            .domain = domain,
            .record_types = record_types,
            .metadata = try self.createMetadata(),
            .timeout_ms = 5000,
        };
        
        const response_data = try self.grpc_client.unaryCall(
            "ghostbridge.RLinkService",
            "ResolveDomain",
            try self.serializeRequest(request)
        );
        defer self.allocator.free(response_data);
        
        return try self.parseResolveResponse(response_data);
    }
    
    // Domain registration via RLINK
    pub fn registerDomain(self: *RLinkClient, domain: []const u8, owner: []const u8, records: []const RLinkDNSRecord) !RLinkDomainRegisterResponse {
        const request = RLinkDomainRegisterRequest{
            .request_id = try self.generateRequestId(),
            .domain = domain,
            .owner_address = owner,
            .records = records,
            .signature = try self.signRequest(domain, owner),
            .metadata = try self.createMetadata(),
        };
        
        const response_data = try self.grpc_client.unaryCall(
            "ghostbridge.RLinkService",
            "RegisterDomain",
            try self.serializeRequest(request)
        );
        defer self.allocator.free(response_data);
        
        return try self.parseRegisterResponse(response_data);
    }
    
    // Subscribe to blockchain events
    pub fn subscribeStateUpdates(self: *RLinkClient, callback: fn(RLinkStateUpdateEvent) void) !void {
        const stream = try self.grpc_client.streamCall(
            "ghostbridge.RLinkService",
            "SubscribeStateUpdates",
            &.{}
        );
        
        // Handle streaming responses
        while (try stream.next()) |event_data| {
            const event = try self.parseStateUpdateEvent(event_data);
            callback(event);
        }
    }
    
    // Helper functions
    fn generateRequestId(self: *RLinkClient) ![]u8 {
        const timestamp = std.time.milliTimestamp();
        return std.fmt.allocPrint(self.allocator, "rlink_{d}_{d}", .{ timestamp, @ptrToInt(self) });
    }
    
    fn createMetadata(self: *RLinkClient) !RLinkMetadata {
        return RLinkMetadata{
            .client_version = "zns-v0.4.0",
            .protocol_version = "rlink-v1.0.0",
            .headers = std.StringHashMap([]const u8).init(self.allocator),
            .timestamp = @intCast(u64, std.time.milliTimestamp()),
        };
    }
    
    fn signRequest(self: *RLinkClient, domain: []const u8, owner: []const u8) ![]u8 {
        // Mock signature - integrate with ZCrypto
        return self.allocator.dupe(u8, "0x123456789abcdef0123456789abcdef01234567");
    }
};
```

### 2. **GhostBridge Implementation** (Zig Server + Rust Client)

```zig
// ghostbridge/src/rlink_server.zig
const std = @import("std");
const grpc = @import("grpc");
const ghostlink = @import("ghostlink");

pub const RLinkServer = struct {
    allocator: std.mem.Allocator,
    grpc_server: grpc.Server,
    ghostlink_client: ghostlink.GhostClient,
    
    pub fn init(allocator: std.mem.Allocator, listen_port: u16, ghostchain_endpoint: []const u8) !RLinkServer {
        return RLinkServer{
            .allocator = allocator,
            .grpc_server = try grpc.Server.init(allocator, listen_port),
            .ghostlink_client = try ghostlink.GhostClient.connect(ghostchain_endpoint),
        };
    }
    
    pub fn start(self: *RLinkServer) !void {
        // Register RLINK service handlers
        try self.grpc_server.registerService("ghostbridge.RLinkService", self);
        
        // Start gRPC server
        try self.grpc_server.start();
        
        std.log.info("RLINK Bridge server started on port {d}", .{self.grpc_server.port});
    }
    
    // Handle domain resolution requests
    pub fn handleResolveDomain(self: *RLinkServer, request_data: []const u8) ![]u8 {
        const request = try self.parseResolveRequest(request_data);
        
        // Forward to GhostChain via GhostLink
        const ghostchain_response = try self.ghostlink_client.resolveDomain(
            request.domain,
            request.record_types
        );
        
        // Convert to RLINK response
        const rlink_response = try self.convertToRLinkResponse(ghostchain_response);
        
        return try self.serializeResponse(rlink_response);
    }
    
    // Handle domain registration requests
    pub fn handleRegisterDomain(self: *RLinkServer, request_data: []const u8) ![]u8 {
        const request = try self.parseRegisterRequest(request_data);
        
        // Validate signature
        if (!try self.validateSignature(request)) {
            return try self.createErrorResponse("Invalid signature");
        }
        
        // Forward to GhostChain via GhostLink
        const ghostchain_response = try self.ghostlink_client.registerDomain(
            request.domain,
            request.owner_address,
            request.records
        );
        
        // Convert to RLINK response
        const rlink_response = try self.convertToRLinkRegisterResponse(ghostchain_response);
        
        return try self.serializeResponse(rlink_response);
    }
    
    // Stream blockchain events back to clients
    pub fn handleSubscribeStateUpdates(self: *RLinkServer, stream: *grpc.Stream) !void {
        // Subscribe to GhostChain events via GhostLink
        const event_stream = try self.ghostlink_client.subscribeEvents();
        
        while (try event_stream.next()) |ghostchain_event| {
            // Convert to RLINK event
            const rlink_event = try self.convertToRLinkEvent(ghostchain_event);
            
            // Send to client
            try stream.send(try self.serializeEvent(rlink_event));
        }
    }
};
```

### 3. **GhostLink Integration** (Rust Client)

```rust
// ghostlink/src/rlink_adapter.rs
use crate::{GhostClient, GhostClientConfig, TransportProtocol};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct RLinkAdapter {
    ghost_client: GhostClient,
}

impl RLinkAdapter {
    pub async fn new(endpoint: String) -> Result<Self> {
        let config = GhostClientConfig::builder()
            .endpoint(endpoint)
            .transport(TransportProtocol::Quic)
            .with_tls()
            .build();
            
        let ghost_client = GhostClient::new(config).await?;
        
        Ok(Self { ghost_client })
    }
    
    // Handle domain resolution from bridge
    pub async fn resolve_domain(&self, domain: &str, record_types: &[&str]) -> Result<RLinkDomainResolveResponse> {
        // Query GhostChain for domain
        let ghostchain_response = self.ghost_client
            .ghostchain()
            .resolve_domain(domain)
            .await?;
        
        // Convert to RLINK format
        let rlink_response = RLinkDomainResolveResponse {
            request_id: "".to_string(), // Set by bridge
            status: RLinkStatus::success(),
            records: ghostchain_response.records.into_iter()
                .map(|r| RLinkDNSRecord::from_ghostchain_record(r))
                .collect(),
            domain_info: RLinkDomainInfo::from_ghostchain_info(ghostchain_response.domain_info),
            resolution_time_ms: ghostchain_response.resolution_time_ms,
        };
        
        Ok(rlink_response)
    }
    
    // Handle domain registration from bridge
    pub async fn register_domain(&self, domain: &str, owner: &str, records: &[RLinkDNSRecord]) -> Result<RLinkDomainRegisterResponse> {
        // Submit registration to GhostChain
        let ghostchain_response = self.ghost_client
            .ghostchain()
            .register_domain(domain, owner, records)
            .await?;
        
        // Convert to RLINK format
        let rlink_response = RLinkDomainRegisterResponse {
            request_id: "".to_string(), // Set by bridge
            status: RLinkStatus::success(),
            transaction_hash: ghostchain_response.transaction_hash,
            block_height: ghostchain_response.block_height,
            contract_address: ghostchain_response.contract_address,
        };
        
        Ok(rlink_response)
    }
    
    // Stream blockchain events to bridge
    pub async fn subscribe_events(&self) -> Result<impl Stream<Item = Result<RLinkStateUpdateEvent>>> {
        let event_stream = self.ghost_client
            .ghostchain()
            .subscribe_domain_events()
            .await?;
        
        Ok(event_stream.map(|event| {
            event.map(|e| RLinkStateUpdateEvent::from_ghostchain_event(e))
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RLinkDomainResolveResponse {
    pub request_id: String,
    pub status: RLinkStatus,
    pub records: Vec<RLinkDNSRecord>,
    pub domain_info: RLinkDomainInfo,
    pub resolution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RLinkStatus {
    pub code: RLinkStatusCode,
    pub message: String,
    pub details: String,
}

impl RLinkStatus {
    pub fn success() -> Self {
        Self {
            code: RLinkStatusCode::Success,
            message: "OK".to_string(),
            details: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RLinkStatusCode {
    Success,
    InvalidRequest,
    DomainNotFound,
    PermissionDenied,
    Timeout,
    InternalError,
    BridgeUnavailable,
}
```

---

## 🚀 Integration Timeline

### Week 1: Foundation
- [x] **ZNS gRPC Service**: Basic gRPC interface implemented
- [x] **GhostBridge Architecture**: Hybrid Zig/Rust structure defined
- [ ] **RLINK Protocol**: Core message definitions
- [ ] **Basic Client**: ZNS → GhostBridge communication

### Week 2: Core Implementation
- [ ] **GhostBridge Server**: Zig-based gRPC server
- [ ] **GhostLink Adapter**: Rust client for GhostChain
- [ ] **Domain Resolution**: End-to-end domain resolution
- [ ] **State Sync**: Blockchain event streaming

### Week 3: Advanced Features
- [ ] **Error Handling**: Comprehensive error propagation
- [ ] **Performance**: Connection pooling, caching
- [ ] **Security**: Signature validation, TLS
- [ ] **Monitoring**: Metrics and health checks

### Week 4: Production Ready
- [ ] **Load Testing**: Stress testing under load
- [ ] **Documentation**: API documentation
- [ ] **Integration Tests**: End-to-end testing
- [ ] **Deployment**: Production deployment guide

---

## 📊 Performance Targets

### Latency Requirements
- **ZNS → GhostBridge**: < 5ms
- **GhostBridge → GhostChain**: < 50ms
- **End-to-end Resolution**: < 100ms
- **Event Propagation**: < 1s

### Throughput Requirements
- **Concurrent Connections**: > 1000
- **Requests per Second**: > 10,000
- **Event Streaming**: > 100,000 events/s
- **Domain Registrations**: > 100/s

### Reliability Requirements
- **Uptime**: > 99.9%
- **Error Rate**: < 0.1%
- **Data Consistency**: > 99.99%
- **Failover Time**: < 30s

---

## 🔒 Security Considerations

### Authentication
- **Request Signing**: Ed25519 signatures for all requests
- **TLS Encryption**: All communications encrypted
- **Rate Limiting**: Per-client rate limits
- **Access Control**: Role-based permissions

### Data Integrity
- **Message Signing**: All messages cryptographically signed
- **Replay Protection**: Timestamp-based replay prevention
- **Data Validation**: Comprehensive input validation
- **Audit Logging**: Complete audit trail

---

## 📋 Next Steps

1. **Implement RLINK Protocol**: Define protobuf schemas
2. **Build GhostBridge Server**: Zig-based gRPC server
3. **Integrate GhostLink**: Rust client adapter
4. **Add Error Handling**: Comprehensive error management
5. **Performance Testing**: Load testing and optimization
6. **Security Audit**: Comprehensive security review

This RLINK specification provides a robust foundation for bidirectional communication between ZNS/ZVM and GhostChain via the hybrid GhostBridge architecture.