# ZNS gRPC Resolver Interface

## 📋 Overview

This document defines the gRPC service interface for ZNS (Zig Name Service) domain resolution, registration, and management. The interface enables seamless communication between ZNS components and external clients (including GhostBridge).

---

## 🌉 Protocol Buffer Definitions

### Core ZNS Service

```protobuf
// zns/proto/zns.proto
syntax = "proto3";

package zns.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/empty.proto";

// Primary ZNS service interface
service ZNSService {
  // Domain resolution
  rpc ResolveDomain(ZNSResolveRequest) returns (ZNSResolveResponse);
  rpc ResolveBatch(ZNSBatchResolveRequest) returns (ZNSBatchResolveResponse);
  
  // Domain registration and management
  rpc RegisterDomain(ZNSRegisterRequest) returns (ZNSRegisterResponse);
  rpc UpdateDomainRecords(ZNSUpdateRequest) returns (ZNSUpdateResponse);
  rpc TransferDomain(ZNSTransferRequest) returns (ZNSTransferResponse);
  
  // Domain queries
  rpc GetDomainInfo(ZNSGetDomainRequest) returns (ZNSGetDomainResponse);
  rpc ListDomains(ZNSListDomainsRequest) returns (ZNSListDomainsResponse);
  rpc CheckAvailability(ZNSAvailabilityRequest) returns (ZNSAvailabilityResponse);
  
  // Real-time subscriptions
  rpc SubscribeDomainChanges(ZNSDomainSubscription) returns (stream ZNSDomainChangeEvent);
  rpc SubscribeCache(ZNSCacheSubscription) returns (stream ZNSCacheEvent);
  
  // Health and status
  rpc GetStatus(google.protobuf.Empty) returns (ZNSStatusResponse);
  rpc GetMetrics(google.protobuf.Empty) returns (ZNSMetricsResponse);
}

// Administrative service for management operations
service ZNSAdminService {
  rpc FlushCache(ZNSFlushCacheRequest) returns (ZNSFlushCacheResponse);
  rpc UpdateConfig(ZNSConfigUpdateRequest) returns (ZNSConfigUpdateResponse);
  rpc GetResolverStatus(google.protobuf.Empty) returns (ZNSResolverStatusResponse);
  rpc EnableResolver(ZNSEnableResolverRequest) returns (ZNSEnableResolverResponse);
  rpc DisableResolver(ZNSDisableResolverRequest) returns (ZNSDisableResolverResponse);
}
```

### Request/Response Messages

```protobuf
// Domain resolution messages
message ZNSResolveRequest {
  string domain = 1;                    // Domain to resolve (e.g., "ghostkellz.zkellz")
  repeated string record_types = 2;     // Requested record types (A, AAAA, TXT, etc.)
  bool include_metadata = 3;            // Include domain metadata in response
  bool use_cache = 4;                   // Allow cached responses (default: true)
  uint32 max_ttl = 5;                   // Maximum acceptable TTL (0 = no limit)
  ResolverConfig resolver_config = 6;   // Optional resolver-specific config
}

message ZNSResolveResponse {
  string domain = 1;                    // Resolved domain name
  repeated DNSRecord records = 2;       // DNS records found
  DomainMetadata metadata = 3;          // Domain metadata (if requested)
  ResolutionInfo resolution_info = 4;   // Resolution details
  ZNSError error = 5;                   // Error information (if failed)
}

message ZNSBatchResolveRequest {
  repeated ZNSResolveRequest requests = 1;  // Multiple resolution requests
  uint32 max_concurrent = 2;               // Max concurrent resolutions (default: 10)
  uint32 timeout_ms = 3;                   // Per-request timeout in milliseconds
}

message ZNSBatchResolveResponse {
  repeated ZNSResolveResponse responses = 1;  // Responses in same order as requests
  BatchMetrics metrics = 2;                   // Batch processing metrics
}

// Domain registration messages
message ZNSRegisterRequest {
  string domain = 1;                    // Domain to register
  string owner_address = 2;             // Owner's blockchain address
  repeated DNSRecord initial_records = 3; // Initial DNS records
  DomainMetadata metadata = 4;          // Domain metadata
  uint64 expiry_timestamp = 5;          // Expiration time (0 = permanent)
  bytes signature = 6;                  // Owner's signature
  RegistrationOptions options = 7;      // Registration options
}

message ZNSRegisterResponse {
  bool success = 1;                     // Registration success
  string transaction_hash = 2;          // Blockchain transaction hash
  string domain = 3;                    // Registered domain
  string contract_address = 4;          // Smart contract address
  uint64 block_number = 5;              // Block number of registration
  ZNSError error = 6;                   // Error information (if failed)
}

// Domain update messages
message ZNSUpdateRequest {
  string domain = 1;                    // Domain to update
  repeated DNSRecord records = 2;       // New/updated records
  UpdateAction action = 3;              // Update action (ADD, UPDATE, DELETE)
  bytes owner_signature = 4;            // Owner's signature
  string transaction_id = 5;            // Optional transaction reference
}

message ZNSUpdateResponse {
  bool success = 1;                     // Update success
  string transaction_hash = 2;          // Blockchain transaction hash
  repeated DNSRecord updated_records = 3; // Successfully updated records
  ZNSError error = 4;                   // Error information (if failed)
}
```

### Data Structure Messages

```protobuf
// DNS record representation
message DNSRecord {
  string name = 1;                      // Record name (usually same as domain)
  DNSRecordType type = 2;               // Record type
  string value = 3;                     // Record value
  uint32 ttl = 4;                       // Time to live in seconds
  uint32 priority = 5;                  // Priority (for MX, SRV)
  uint32 weight = 6;                    // Weight (for SRV)
  uint32 port = 7;                      // Port (for SRV)
  string target = 8;                    // Target (for SRV, CNAME)
  google.protobuf.Timestamp created_at = 9;  // Creation timestamp
  bytes signature = 10;                 // Record signature
}

enum DNSRecordType {
  DNS_RECORD_TYPE_UNSPECIFIED = 0;
  DNS_RECORD_TYPE_A = 1;                // IPv4 address
  DNS_RECORD_TYPE_AAAA = 2;             // IPv6 address
  DNS_RECORD_TYPE_CNAME = 3;            // Canonical name
  DNS_RECORD_TYPE_MX = 4;               // Mail exchange
  DNS_RECORD_TYPE_TXT = 5;              // Text record
  DNS_RECORD_TYPE_SRV = 6;              // Service record
  DNS_RECORD_TYPE_NS = 7;               // Name server
  DNS_RECORD_TYPE_SOA = 8;              // Start of authority
  DNS_RECORD_TYPE_PTR = 9;              // Pointer record
  DNS_RECORD_TYPE_GHOST = 10;           // GhostChain-specific
  DNS_RECORD_TYPE_CONTRACT = 11;        // Smart contract address
  DNS_RECORD_TYPE_WALLET = 12;          // Wallet address
}

// Domain metadata
message DomainMetadata {
  uint32 version = 1;                   // Schema version
  string registrar = 2;                 // Registration source
  repeated string tags = 3;             // Domain tags/categories
  string description = 4;               // Domain description
  string avatar_url = 5;                // Avatar/logo URL
  string website_url = 6;               // Website URL
  SocialLinks social = 7;               // Social media links
  map<string, string> custom_data = 8;  // Custom key-value data
}

message SocialLinks {
  string twitter = 1;
  string github = 2;
  string discord = 3;
  string telegram = 4;
  string linkedin = 5;
  string instagram = 6;
}

// Resolution information
message ResolutionInfo {
  ResolverSource source = 1;            // Resolution source
  uint64 resolution_time_ms = 2;        // Resolution time in milliseconds
  bool was_cached = 3;                  // Whether result was from cache
  google.protobuf.Timestamp resolved_at = 4; // Resolution timestamp
  string resolver_version = 5;          // Resolver version
  repeated string resolution_path = 6;  // Resolution chain (for debugging)
}

enum ResolverSource {
  RESOLVER_SOURCE_UNSPECIFIED = 0;
  RESOLVER_SOURCE_ZNS_NATIVE = 1;       // Native ZNS resolution
  RESOLVER_SOURCE_ENS_BRIDGE = 2;       // ENS bridge
  RESOLVER_SOURCE_UNSTOPPABLE = 3;      // Unstoppable Domains
  RESOLVER_SOURCE_TRADITIONAL_DNS = 4;  // Traditional DNS fallback
  RESOLVER_SOURCE_CACHE = 5;            // Local cache
}

// Error handling
message ZNSError {
  ZNSErrorCode code = 1;                // Error code
  string message = 2;                   // Human-readable error message
  string details = 3;                   // Detailed error information
  repeated string resolution_chain = 4; // Resolution chain for debugging
}

enum ZNSErrorCode {
  ZNS_ERROR_CODE_UNSPECIFIED = 0;
  ZNS_ERROR_CODE_DOMAIN_NOT_FOUND = 1;
  ZNS_ERROR_CODE_INVALID_DOMAIN = 2;
  ZNS_ERROR_CODE_INVALID_RECORD_TYPE = 3;
  ZNS_ERROR_CODE_PERMISSION_DENIED = 4;
  ZNS_ERROR_CODE_SIGNATURE_INVALID = 5;
  ZNS_ERROR_CODE_DOMAIN_EXPIRED = 6;
  ZNS_ERROR_CODE_RESOLVER_UNAVAILABLE = 7;
  ZNS_ERROR_CODE_TIMEOUT = 8;
  ZNS_ERROR_CODE_RATE_LIMITED = 9;
  ZNS_ERROR_CODE_INTERNAL_ERROR = 10;
}
```

### Configuration and Management Messages

```protobuf
// Configuration messages
message ResolverConfig {
  bool enable_cache = 1;                // Enable caching for this request
  uint32 cache_ttl_override = 2;        // Override cache TTL
  repeated ResolverSource preferred_sources = 3; // Preferred resolution sources
  bool enable_dnssec = 4;               // Enable DNSSEC validation
  map<string, string> custom_headers = 5; // Custom headers for HTTP resolvers
}

message RegistrationOptions {
  bool auto_renew = 1;                  // Enable auto-renewal
  uint32 renewal_period_days = 2;       // Renewal period in days
  string payment_token = 3;             // Payment token type (SPIRIT, MANA, etc.)
  string referrer_address = 4;          // Referrer for revenue sharing
}

enum UpdateAction {
  UPDATE_ACTION_UNSPECIFIED = 0;
  UPDATE_ACTION_ADD = 1;                // Add new records
  UPDATE_ACTION_UPDATE = 2;             // Update existing records
  UPDATE_ACTION_DELETE = 3;             // Delete records
  UPDATE_ACTION_REPLACE = 4;            // Replace all records
}

// Status and metrics messages
message ZNSStatusResponse {
  bool healthy = 1;                     // Overall health status
  string version = 2;                   // ZNS version
  uint64 uptime_seconds = 3;            // Uptime in seconds
  repeated ResolverStatus resolvers = 4; // Status of individual resolvers
  CacheStatus cache = 5;                // Cache status
}

message ResolverStatus {
  ResolverSource source = 1;            // Resolver source
  bool enabled = 2;                     // Whether resolver is enabled
  bool healthy = 3;                     // Health status
  uint64 total_queries = 4;             // Total queries processed
  uint64 successful_queries = 5;        // Successful queries
  uint64 failed_queries = 6;            // Failed queries
  double average_response_time_ms = 7;  // Average response time
  google.protobuf.Timestamp last_query = 8; // Last query timestamp
}

message CacheStatus {
  uint64 total_entries = 1;             // Total cached entries
  uint64 memory_usage_bytes = 2;        // Memory usage in bytes
  double hit_rate = 3;                  // Cache hit rate (0.0-1.0)
  uint64 total_hits = 4;                // Total cache hits
  uint64 total_misses = 5;              // Total cache misses
  uint64 evictions = 6;                 // Number of evictions
}

message ZNSMetricsResponse {
  uint64 total_resolutions = 1;         // Total resolutions performed
  uint64 successful_resolutions = 2;    // Successful resolutions
  uint64 failed_resolutions = 3;        // Failed resolutions
  double average_resolution_time_ms = 4; // Average resolution time
  map<string, uint64> resolutions_by_tld = 5; // Resolutions by TLD
  map<string, uint64> resolutions_by_source = 6; // Resolutions by source
  CacheStatus cache_metrics = 7;        // Cache metrics
  google.protobuf.Timestamp metrics_since = 8; // Metrics collection start time
}
```

### Subscription and Events

```protobuf
// Subscription messages
message ZNSDomainSubscription {
  repeated string domains = 1;          // Specific domains to watch (empty = all)
  repeated DNSRecordType record_types = 2; // Record types to watch (empty = all)
  bool include_metadata = 3;            // Include metadata in events
}

message ZNSDomainChangeEvent {
  string domain = 1;                    // Changed domain
  ChangeEventType event_type = 2;       // Type of change
  repeated DNSRecord old_records = 3;   // Previous records (for updates/deletes)
  repeated DNSRecord new_records = 4;   // New records (for adds/updates)
  google.protobuf.Timestamp timestamp = 5; // Event timestamp
  string transaction_hash = 6;          // Associated blockchain transaction
}

enum ChangeEventType {
  CHANGE_EVENT_TYPE_UNSPECIFIED = 0;
  CHANGE_EVENT_TYPE_DOMAIN_REGISTERED = 1; // New domain registered
  CHANGE_EVENT_TYPE_DOMAIN_UPDATED = 2;    // Domain records updated
  CHANGE_EVENT_TYPE_DOMAIN_TRANSFERRED = 3; // Domain ownership transferred
  CHANGE_EVENT_TYPE_DOMAIN_EXPIRED = 4;     // Domain expired
  CHANGE_EVENT_TYPE_DOMAIN_RENEWED = 5;     // Domain renewed
}

message ZNSCacheSubscription {
  bool include_hits = 1;                // Include cache hit events
  bool include_misses = 2;              // Include cache miss events
  bool include_evictions = 3;           // Include cache eviction events
}

message ZNSCacheEvent {
  CacheEventType event_type = 1;        // Type of cache event
  string domain = 2;                    // Affected domain
  google.protobuf.Timestamp timestamp = 3; // Event timestamp
  CacheEventData data = 4;              // Additional event data
}

enum CacheEventType {
  CACHE_EVENT_TYPE_UNSPECIFIED = 0;
  CACHE_EVENT_TYPE_HIT = 1;             // Cache hit
  CACHE_EVENT_TYPE_MISS = 2;            // Cache miss
  CACHE_EVENT_TYPE_EVICTION = 3;        // Cache eviction
  CACHE_EVENT_TYPE_FLUSH = 4;           // Cache flush
}

message CacheEventData {
  uint64 hit_count = 1;                 // Hit count for the entry
  uint64 ttl_remaining = 2;             // TTL remaining in seconds
  ResolverSource original_source = 3;   // Original resolution source
}
```

---

## 🔧 Service Implementation Guide

### Zig Server Implementation

```zig
// zns/src/grpc_server.zig
const std = @import("std");
const grpc = @import("grpc");
const zns = @import("zns.zig");

pub const ZNSGrpcServer = struct {
    allocator: std.mem.Allocator,
    resolver: *zns.ZNSResolver,
    cache: *zns.DomainCache,
    metrics: *zns.ResolutionMetrics,
    
    pub fn init(allocator: std.mem.Allocator, resolver: *zns.ZNSResolver) !Self {
        return Self{
            .allocator = allocator,
            .resolver = resolver,
            .cache = try zns.DomainCache.init(allocator),
            .metrics = try zns.ResolutionMetrics.init(allocator),
        };
    }
    
    pub fn resolve_domain(self: *Self, request: ZNSResolveRequest) !ZNSResolveResponse {
        const start_time = std.time.milliTimestamp();
        
        // Check cache first if enabled
        if (request.use_cache) {
            if (self.cache.get_cached_domain(request.domain)) |cached_data| {
                return self.create_response_from_cache(cached_data, start_time);
            }
        }
        
        // Perform resolution
        const resolution_result = try self.resolver.resolve_domain(
            request.domain,
            request.record_types,
        );
        
        const end_time = std.time.milliTimestamp();
        const resolution_time = @intCast(u64, end_time - start_time);
        
        // Record metrics
        self.metrics.record_query(
            request.domain,
            false, // Not a cache hit
            resolution_time,
            resolution_result.success,
        );
        
        // Cache the result if successful
        if (resolution_result.success and request.use_cache) {
            try self.cache.cache_domain(resolution_result.domain_data, null);
        }
        
        return self.create_response_from_resolution(resolution_result, resolution_time);
    }
    
    pub fn register_domain(self: *Self, request: ZNSRegisterRequest) !ZNSRegisterResponse {
        // Validate domain name
        if (!zns.DomainValidator.is_valid_domain(request.domain)) {
            return ZNSRegisterResponse{
                .success = false,
                .error = ZNSError{
                    .code = .ZNS_ERROR_CODE_INVALID_DOMAIN,
                    .message = "Invalid domain name format",
                    .details = request.domain,
                    .resolution_chain = &[_][]const u8{},
                },
            };
        }
        
        // Verify signature
        if (!try self.verify_registration_signature(request)) {
            return ZNSRegisterResponse{
                .success = false,
                .error = ZNSError{
                    .code = .ZNS_ERROR_CODE_SIGNATURE_INVALID,
                    .message = "Invalid registration signature",
                    .details = "",
                    .resolution_chain = &[_][]const u8{},
                },
            };
        }
        
        // Register domain via resolver
        const registration_result = try self.resolver.register_domain(request);
        
        return ZNSRegisterResponse{
            .success = registration_result.success,
            .transaction_hash = registration_result.transaction_hash,
            .domain = request.domain,
            .contract_address = registration_result.contract_address,
            .block_number = registration_result.block_number,
            .error = if (registration_result.success) null else registration_result.error,
        };
    }
    
    // Additional service methods...
    pub fn subscribe_domain_changes(self: *Self, subscription: ZNSDomainSubscription) !*grpc.Stream(ZNSDomainChangeEvent) {
        // Implementation for real-time domain change subscription
        // This would typically integrate with the blockchain event system
        return try self.resolver.create_domain_subscription(subscription);
    }
};
```

### Rust Client Implementation

```rust
// ghostchain/src/zns_client.rs
use tonic::{Request, Response, Status};
use tokio_stream::StreamExt;

pub mod zns_v1 {
    tonic::include_proto!("zns.v1");
}

use zns_v1::{
    zns_service_client::ZnsServiceClient,
    ZnsResolveRequest, ZnsResolveResponse,
    ZnsRegisterRequest, ZnsRegisterResponse,
};

pub struct ZNSClient {
    client: ZnsServiceClient<tonic::transport::Channel>,
}

impl ZNSClient {
    pub async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        let client = ZnsServiceClient::connect(endpoint).await?;
        Ok(Self { client })
    }
    
    pub async fn resolve_domain(
        &mut self,
        domain: String,
        record_types: Vec<String>,
    ) -> Result<ZnsResolveResponse, Status> {
        let request = Request::new(ZnsResolveRequest {
            domain,
            record_types,
            include_metadata: true,
            use_cache: true,
            max_ttl: 3600,
            resolver_config: None,
        });
        
        let response = self.client.resolve_domain(request).await?;
        Ok(response.into_inner())
    }
    
    pub async fn register_domain(
        &mut self,
        domain: String,
        owner_address: String,
        initial_records: Vec<zns_v1::DnsRecord>,
    ) -> Result<ZnsRegisterResponse, Status> {
        let request = Request::new(ZnsRegisterRequest {
            domain,
            owner_address,
            initial_records,
            metadata: None,
            expiry_timestamp: 0,
            signature: vec![],
            options: None,
        });
        
        let response = self.client.register_domain(request).await?;
        Ok(response.into_inner())
    }
    
    pub async fn subscribe_domain_changes(
        &mut self,
        domains: Vec<String>,
    ) -> Result<impl StreamExt<Item = Result<zns_v1::ZnsDomainChangeEvent, Status>>, Status> {
        let subscription = zns_v1::ZnsDomainSubscription {
            domains,
            record_types: vec![],
            include_metadata: true,
        };
        
        let request = Request::new(subscription);
        let response = self.client.subscribe_domain_changes(request).await?;
        Ok(response.into_inner())
    }
}
```

---

## 🚀 Integration with GhostBridge

### Bridge Service Extension

```protobuf
// ghostbridge/proto/bridge_zns.proto
syntax = "proto3";

package ghostbridge.v1;

import "zns/proto/zns.proto";

// Extended bridge service with ZNS integration
service GhostBridgeZNSService {
  // ZNS operations through bridge
  rpc ResolveDomainViaBridge(BridgeResolveRequest) returns (BridgeResolveResponse);
  rpc RegisterDomainViaBridge(BridgeRegisterRequest) returns (BridgeRegisterResponse);
  
  // Sync operations between ZNS and blockchain
  rpc SyncDomainToChain(SyncToChainRequest) returns (SyncToChainResponse);
  rpc SyncDomainFromChain(SyncFromChainRequest) returns (SyncFromChainResponse);
  
  // Bridge health and status
  rpc GetBridgeStatus(google.protobuf.Empty) returns (BridgeStatusResponse);
}

message BridgeResolveRequest {
  zns.v1.ZNSResolveRequest zns_request = 1;   // Original ZNS request
  string bridge_id = 2;                       // Bridge identifier
  map<string, string> bridge_metadata = 3;    // Bridge-specific metadata
}

message BridgeResolveResponse {
  zns.v1.ZNSResolveResponse zns_response = 1; // ZNS response
  string bridge_id = 2;                       // Bridge identifier
  uint64 bridge_processing_time_ms = 3;       // Bridge processing time
  bool used_blockchain_fallback = 4;          // Whether blockchain was queried
}
```

---

## 📊 Performance Metrics

### Expected Performance Targets

```yaml
# zns_performance_targets.yaml
resolution_targets:
  cache_hit_latency: "< 5ms"
  native_zns_latency: "< 50ms"
  ens_bridge_latency: "< 200ms"
  unstoppable_latency: "< 300ms"
  traditional_dns_fallback: "< 1000ms"

throughput_targets:
  concurrent_resolutions: "> 1000 req/s"
  batch_resolutions: "> 100 batches/s"
  subscription_events: "> 10000 events/s"

cache_targets:
  hit_rate: "> 85%"
  memory_efficiency: "< 1GB for 100k domains"
  eviction_rate: "< 10% daily"

reliability_targets:
  uptime: "> 99.9%"
  error_rate: "< 0.1%"
  data_consistency: "> 99.99%"
```

---

## 🚀 Next Implementation Steps

1. **Generate Code**: Use protoc to generate Zig and Rust client/server code
2. **Implement Core Service**: Build the ZNS gRPC server in Zig
3. **Create Client Libraries**: Implement Rust client for GhostBridge integration
4. **Add Subscription Support**: Implement real-time domain change events
5. **Performance Testing**: Benchmark resolution performance and caching
6. **Integration Testing**: Test with GhostBridge and GhostChain components

This gRPC interface provides a comprehensive, type-safe, and high-performance foundation for ZNS domain resolution and management.
