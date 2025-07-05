# ZNS Record Format Schema

## 📋 Overview

This document defines the standardized record format for the Zig Name Service (ZNS), including data structures, validation rules, and serialization formats.

---

## 🏗️ Core ZNS Record Structure

### Primary Record Format

```zig
// zns/src/record.zig
pub const DnsRecordType = enum {
    A,      // IPv4 address
    AAAA,   // IPv6 address  
    CNAME,  // Canonical name
    MX,     // Mail exchange
    TXT,    // Text record
    SRV,    // Service record
    NS,     // Name server
    SOA,    // Start of authority
    PTR,    // Pointer record
    GHOST,  // GhostChain-specific metadata
    CONTRACT, // Smart contract address
    WALLET,   // Wallet address mapping
};

pub const DnsRecord = struct {
    record_type: DnsRecordType,
    name: []const u8,        // Domain name (e.g., "ghostkellz.zkellz")
    value: []const u8,       // Record value
    ttl: u32,               // Time to live in seconds
    priority: ?u16,         // For MX, SRV records
    port: ?u16,             // For SRV records
    weight: ?u16,           // For SRV records
    target: ?[]const u8,    // For SRV, CNAME records
    created_at: u64,        // Unix timestamp
    signature: ?[]const u8, // Ed25519 signature for validation
};

pub const DomainData = struct {
    domain: []const u8,           // Full domain name
    owner: []const u8,            // Owner address/GhostID
    records: []DnsRecord,         // Array of DNS records
    contract_address: ?[]const u8, // Associated smart contract
    metadata: DomainMetadata,      // Additional domain metadata
    last_updated: u64,            // Unix timestamp
    expiry: ?u64,                 // Domain expiration (null for permanent)
    signature: []const u8,        // Owner signature for integrity
};

pub const DomainMetadata = struct {
    version: u8,                  // Schema version
    registrar: []const u8,        // Registration source (ZNS, ENS, etc.)
    tags: ?[][]const u8,          // Optional tags/categories
    description: ?[]const u8,     // Optional description
    avatar: ?[]const u8,          // Optional avatar/logo URL
    website: ?[]const u8,         // Optional website URL
    social: ?SocialLinks,         // Optional social media links
};

pub const SocialLinks = struct {
    twitter: ?[]const u8,
    github: ?[]const u8,
    discord: ?[]const u8,
    telegram: ?[]const u8,
};
```

---

## 🔗 Record Validation Rules

### Domain Name Validation

```zig
// zns/src/validation.zig
pub const DomainValidator = struct {
    pub fn is_valid_domain(domain: []const u8) bool {
        // 1. Length: 1-253 characters total
        if (domain.len == 0 or domain.len > 253) return false;
        
        // 2. Must not start/end with dot or hyphen
        if (domain[0] == '.' or domain[0] == '-') return false;
        if (domain[domain.len - 1] == '.' or domain[domain.len - 1] == '-') return false;
        
        // 3. Valid TLDs: .ghost, .zkellz, .kz, .eth, .crypto, etc.
        return is_supported_tld(domain);
    }
    
    pub fn is_supported_tld(domain: []const u8) bool {
        const supported_tlds = [_][]const u8{
            ".ghost", ".zkellz", ".kz",  // Native ZNS
            ".eth",                       // ENS bridge
            ".crypto", ".nft", ".x",      // Unstoppable Domains
            ".wallet", ".bitcoin",        // Additional Web3 TLDs
        };
        
        for (supported_tlds) |tld| {
            if (std.mem.endsWith(u8, domain, tld)) {
                return true;
            }
        }
        return false;
    }
};
```

### Record Type Validation

```zig
pub const RecordValidator = struct {
    pub fn validate_record(record: *const DnsRecord) ValidationResult {
        switch (record.record_type) {
            .A => return validate_ipv4(record.value),
            .AAAA => return validate_ipv6(record.value),
            .CNAME => return validate_domain_name(record.value),
            .MX => return validate_mx_record(record),
            .TXT => return validate_txt_record(record.value),
            .SRV => return validate_srv_record(record),
            .GHOST => return validate_ghost_metadata(record.value),
            .CONTRACT => return validate_contract_address(record.value),
            .WALLET => return validate_wallet_address(record.value),
            else => return .valid,
        }
    }
    
    fn validate_ipv4(address: []const u8) ValidationResult {
        // IPv4 format: xxx.xxx.xxx.xxx
        var parts = std.mem.split(u8, address, ".");
        var count: u8 = 0;
        
        while (parts.next()) |part| {
            count += 1;
            if (count > 4) return .invalid_format;
            
            const num = std.fmt.parseInt(u8, part, 10) catch return .invalid_format;
            if (num > 255) return .invalid_format;
        }
        
        return if (count == 4) .valid else .invalid_format;
    }
    
    fn validate_ipv6(address: []const u8) ValidationResult {
        // Basic IPv6 validation (simplified)
        if (address.len < 2 or address.len > 39) return .invalid_format;
        
        // Must contain colons for IPv6
        if (std.mem.indexOf(u8, address, ":") == null) return .invalid_format;
        
        return .valid; // Full RFC 4291 validation would be more complex
    }
    
    fn validate_contract_address(address: []const u8) ValidationResult {
        // GhostChain contract address validation
        if (address.len != 42) return .invalid_format; // 0x + 40 hex chars
        if (!std.mem.startsWith(u8, address, "0x")) return .invalid_format;
        
        // Validate hex characters
        for (address[2..]) |c| {
            if (!std.ascii.isHex(c)) return .invalid_format;
        }
        
        return .valid;
    }
};

pub const ValidationResult = enum {
    valid,
    invalid_format,
    invalid_length,
    unsupported_type,
    signature_invalid,
};
```

---

## 📦 Serialization Format

### JSON Serialization

```zig
// zns/src/serialization.zig
pub const JsonSerializer = struct {
    pub fn serialize_domain(allocator: std.mem.Allocator, domain: *const DomainData) ![]u8 {
        var json_obj = std.json.ObjectMap.init(allocator);
        defer json_obj.deinit();
        
        try json_obj.put("domain", std.json.Value{ .String = domain.domain });
        try json_obj.put("owner", std.json.Value{ .String = domain.owner });
        try json_obj.put("last_updated", std.json.Value{ .Integer = @intCast(domain.last_updated) });
        
        // Serialize records array
        var records_array = std.json.Array.init(allocator);
        for (domain.records) |record| {
            var record_obj = std.json.ObjectMap.init(allocator);
            try record_obj.put("type", std.json.Value{ .String = @tagName(record.record_type) });
            try record_obj.put("name", std.json.Value{ .String = record.name });
            try record_obj.put("value", std.json.Value{ .String = record.value });
            try record_obj.put("ttl", std.json.Value{ .Integer = @intCast(record.ttl) });
            
            try records_array.append(std.json.Value{ .Object = record_obj });
        }
        try json_obj.put("records", std.json.Value{ .Array = records_array });
        
        return try std.json.stringify(std.json.Value{ .Object = json_obj }, .{}, allocator);
    }
    
    pub fn deserialize_domain(allocator: std.mem.Allocator, json_data: []const u8) !DomainData {
        var parser = std.json.Parser.init(allocator, false);
        defer parser.deinit();
        
        var tree = try parser.parse(json_data);
        defer tree.deinit();
        
        const root = tree.root.Object;
        
        return DomainData{
            .domain = try allocator.dupe(u8, root.get("domain").?.String),
            .owner = try allocator.dupe(u8, root.get("owner").?.String),
            .records = try parse_records_array(allocator, root.get("records").?.Array),
            .contract_address = null, // Parse if present
            .metadata = DomainMetadata{
                .version = 1,
                .registrar = "ZNS",
                .tags = null,
                .description = null,
                .avatar = null,
                .website = null,
                .social = null,
            },
            .last_updated = @intCast(root.get("last_updated").?.Integer),
            .expiry = null,
            .signature = &[_]u8{}, // Empty for now
        };
    }
};
```

### Binary Serialization (MessagePack)

```zig
// For high-performance inter-service communication
pub const BinarySerializer = struct {
    pub fn serialize_domain_binary(allocator: std.mem.Allocator, domain: *const DomainData) ![]u8 {
        // Use MessagePack or custom binary format for efficiency
        // Implementation details...
        return &[_]u8{}; // Placeholder
    }
    
    pub fn deserialize_domain_binary(allocator: std.mem.Allocator, data: []const u8) !DomainData {
        // Deserialize from binary format
        // Implementation details...
        return DomainData{}; // Placeholder
    }
};
```

---

## 🔐 Cryptographic Signatures

### Record Signing

```zig
// zns/src/signing.zig
pub const RecordSigner = struct {
    private_key: [32]u8,
    
    pub fn sign_domain_data(self: *const Self, domain: *DomainData) !void {
        // Create canonical representation for signing
        const canonical_data = try create_canonical_representation(domain);
        defer std.heap.page_allocator.free(canonical_data);
        
        // Sign with Ed25519
        const signature = try zsig.ed25519_sign(canonical_data, self.private_key);
        
        // Store signature in domain data
        domain.signature = try std.heap.page_allocator.dupe(u8, &signature);
    }
    
    pub fn verify_domain_signature(domain: *const DomainData, public_key: [32]u8) !bool {
        if (domain.signature.len == 0) return false;
        
        const canonical_data = try create_canonical_representation(domain);
        defer std.heap.page_allocator.free(canonical_data);
        
        return zsig.ed25519_verify(canonical_data, domain.signature, public_key);
    }
    
    fn create_canonical_representation(domain: *const DomainData) ![]u8 {
        // Create deterministic byte representation for signing
        // Format: domain|owner|records_hash|last_updated
        var buffer = std.ArrayList(u8).init(std.heap.page_allocator);
        defer buffer.deinit();
        
        try buffer.appendSlice(domain.domain);
        try buffer.append('|');
        try buffer.appendSlice(domain.owner);
        try buffer.append('|');
        
        // Hash all records for consistency
        const records_hash = try hash_records(domain.records);
        try buffer.appendSlice(&records_hash);
        try buffer.append('|');
        
        const timestamp_str = try std.fmt.allocPrint(std.heap.page_allocator, "{}", .{domain.last_updated});
        defer std.heap.page_allocator.free(timestamp_str);
        try buffer.appendSlice(timestamp_str);
        
        return buffer.toOwnedSlice();
    }
};
```

---

## 🔧 Cache Schema

### TTL-Based Caching

```zig
// zns/src/cache.zig
pub const CacheEntry = struct {
    domain_data: DomainData,
    cached_at: u64,     // Unix timestamp when cached
    expires_at: u64,    // When cache entry expires
    hit_count: u32,     // Number of times accessed
    source: CacheSource, // Where data came from
};

pub const CacheSource = enum {
    zns_native,         // Native ZNS resolution
    ens_bridge,         // ENS bridge
    unstoppable_bridge, // Unstoppable Domains bridge
    traditional_dns,    // Fallback DNS
};

pub const DomainCache = struct {
    entries: std.HashMap([]const u8, CacheEntry, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    max_entries: usize,
    default_ttl: u32,
    
    pub fn get_cached_domain(self: *Self, domain: []const u8) ?*const DomainData {
        if (self.entries.get(domain)) |entry| {
            const now = std.time.timestamp();
            
            if (now <= entry.expires_at) {
                // Update hit count
                var mutable_entry = self.entries.getPtr(domain).?;
                mutable_entry.hit_count += 1;
                
                return &entry.domain_data;
            } else {
                // Entry expired, remove it
                _ = self.entries.remove(domain);
            }
        }
        
        return null;
    }
    
    pub fn cache_domain(self: *Self, domain_data: DomainData, ttl: ?u32) !void {
        const now = std.time.timestamp();
        const effective_ttl = ttl orelse self.default_ttl;
        
        const entry = CacheEntry{
            .domain_data = domain_data,
            .cached_at = @intCast(now),
            .expires_at = @intCast(now + effective_ttl),
            .hit_count = 0,
            .source = .zns_native,
        };
        
        // If cache is full, remove least recently used entry
        if (self.entries.count() >= self.max_entries) {
            try self.evict_lru_entry();
        }
        
        try self.entries.put(domain_data.domain, entry);
    }
};
```

---

## 📊 Metrics and Analytics

### Resolution Statistics

```zig
// zns/src/metrics.zig
pub const ResolutionMetrics = struct {
    total_queries: u64,
    cache_hits: u64,
    cache_misses: u64,
    successful_resolutions: u64,
    failed_resolutions: u64,
    average_resolution_time_ms: f64,
    queries_by_tld: std.HashMap([]const u8, u64, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    pub fn record_query(self: *Self, domain: []const u8, was_cache_hit: bool, resolution_time_ms: u64, success: bool) void {
        self.total_queries += 1;
        
        if (was_cache_hit) {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
        
        if (success) {
            self.successful_resolutions += 1;
        } else {
            self.failed_resolutions += 1;
        }
        
        // Update moving average for resolution time
        const alpha = 0.1; // Smoothing factor
        self.average_resolution_time_ms = alpha * @as(f64, @floatFromInt(resolution_time_ms)) + 
                                         (1.0 - alpha) * self.average_resolution_time_ms;
        
        // Track queries by TLD
        if (get_tld(domain)) |tld| {
            const current_count = self.queries_by_tld.get(tld) orelse 0;
            self.queries_by_tld.put(tld, current_count + 1) catch {};
        }
    }
    
    pub fn get_cache_hit_rate(self: *const Self) f64 {
        if (self.total_queries == 0) return 0.0;
        return @as(f64, @floatFromInt(self.cache_hits)) / @as(f64, @floatFromInt(self.total_queries));
    }
};
```

---

## 📝 Usage Examples

### Basic Record Creation

```zig
// Example: Creating a domain record
const record = DnsRecord{
    .record_type = .A,
    .name = "ghostkellz.zkellz",
    .value = "192.168.1.100",
    .ttl = 3600,
    .priority = null,
    .port = null,
    .weight = null,
    .target = null,
    .created_at = @intCast(std.time.timestamp()),
    .signature = null,
};

const domain_data = DomainData{
    .domain = "ghostkellz.zkellz",
    .owner = "ghost1abc123def456...",
    .records = &[_]DnsRecord{record},
    .contract_address = "0x742d35Cc6486C4F09aAbB8e2F68bF51b5FBB4BF1",
    .metadata = DomainMetadata{
        .version = 1,
        .registrar = "ZNS",
        .tags = null,
        .description = "GhostKellz personal domain",
        .avatar = null,
        .website = "https://ghostkellz.sh",
        .social = null,
    },
    .last_updated = @intCast(std.time.timestamp()),
    .expiry = null,
    .signature = &[_]u8{},
};
```

### Record Validation

```zig
const validator = RecordValidator{};
const result = validator.validate_record(&record);

switch (result) {
    .valid => std.debug.print("Record is valid\n", .{}),
    .invalid_format => std.debug.print("Invalid record format\n", .{}),
    .invalid_length => std.debug.print("Invalid record length\n", .{}),
    else => std.debug.print("Validation failed\n", .{}),
}
```

---

## 🚀 Next Steps

1. **Implement Core Structures**: Complete the Zig implementation of all record types
2. **Add Validation Layer**: Implement comprehensive validation for all record types
3. **Create Serialization**: Add JSON and binary serialization support
4. **Integrate Caching**: Implement TTL-based caching with LRU eviction
5. **Add Metrics**: Track resolution performance and statistics
6. **Connect to GhostBridge**: Integrate with gRPC interface for Rust interoperability

This schema provides a robust foundation for the ZNS system while maintaining compatibility with existing DNS standards and adding blockchain-specific enhancements.
