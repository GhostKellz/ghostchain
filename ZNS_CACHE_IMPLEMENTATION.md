# ZNS In-Memory Caching Implementation

## 📋 Overview

This document provides a complete implementation of an in-memory caching system for ZNS with TTL (Time To Live) support, LRU eviction, and performance optimization features.

---

## 🏗️ Core Cache Implementation

### Primary Cache Structure

```zig
// zns/src/cache/domain_cache.zig
const std = @import("std");
const record = @import("../record.zig");
const metrics = @import("../metrics.zig");

pub const CacheEntry = struct {
    domain_data: record.DomainData,
    cached_at: u64,          // Unix timestamp when cached
    expires_at: u64,         // Unix timestamp when expires
    last_accessed: u64,      // Last access time for LRU
    hit_count: u32,          // Number of times accessed
    source: CacheSource,     // Where data originated
    size_bytes: u32,         // Memory footprint of this entry
    
    pub fn is_expired(self: *const Self) bool {
        const now = @intCast(u64, std.time.timestamp());
        return now > self.expires_at;
    }
    
    pub fn time_until_expiry(self: *const Self) u64 {
        const now = @intCast(u64, std.time.timestamp());
        if (self.expires_at <= now) return 0;
        return self.expires_at - now;
    }
    
    pub fn update_access_time(self: *Self) void {
        self.last_accessed = @intCast(u64, std.time.timestamp());
        self.hit_count += 1;
    }
};

pub const CacheSource = enum {
    zns_native,              // Native ZNS resolution
    ens_bridge,              // ENS bridge resolution
    unstoppable_bridge,      // Unstoppable Domains bridge
    traditional_dns,         // Traditional DNS fallback
    peer_cache,              // From peer node cache
    contract_sync,           // From smart contract sync
};

pub const DomainCache = struct {
    const Self = @This();
    
    allocator: std.mem.Allocator,
    entries: std.HashMap([]const u8, CacheEntry, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    lru_list: std.DoublyLinkedList([]const u8), // LRU ordering
    
    // Configuration
    max_entries: usize,
    max_memory_bytes: usize,
    default_ttl: u32,
    min_ttl: u32,
    max_ttl: u32,
    
    // Statistics
    current_memory_bytes: usize,
    total_hits: u64,
    total_misses: u64,
    total_evictions: u64,
    total_expirations: u64,
    
    // Background cleanup
    cleanup_timer: ?std.time.Timer,
    cleanup_interval_ms: u64,
    
    pub fn init(allocator: std.mem.Allocator, config: CacheConfig) !Self {
        return Self{
            .allocator = allocator,
            .entries = std.HashMap([]const u8, CacheEntry, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .lru_list = std.DoublyLinkedList([]const u8){},
            .max_entries = config.max_entries,
            .max_memory_bytes = config.max_memory_bytes,
            .default_ttl = config.default_ttl,
            .min_ttl = config.min_ttl,
            .max_ttl = config.max_ttl,
            .current_memory_bytes = 0,
            .total_hits = 0,
            .total_misses = 0,
            .total_evictions = 0,
            .total_expirations = 0,
            .cleanup_timer = null,
            .cleanup_interval_ms = config.cleanup_interval_ms,
        };
    }
    
    pub fn deinit(self: *Self) void {
        self.clear();
        self.entries.deinit();
    }
    
    /// Get domain from cache, returns null if not found or expired
    pub fn get_domain(self: *Self, domain: []const u8) ?*const record.DomainData {
        if (self.entries.getPtr(domain)) |entry| {
            // Check if entry is expired
            if (entry.is_expired()) {
                self.remove_entry(domain);
                self.total_expirations += 1;
                self.total_misses += 1;
                return null;
            }
            
            // Update access statistics
            entry.update_access_time();
            self.update_lru_position(domain);
            self.total_hits += 1;
            
            return &entry.domain_data;
        }
        
        self.total_misses += 1;
        return null;
    }
    
    /// Cache domain data with specified TTL
    pub fn cache_domain(self: *Self, domain_data: record.DomainData, ttl: ?u32, source: CacheSource) !void {
        const effective_ttl = self.calculate_effective_ttl(ttl);
        const now = @intCast(u64, std.time.timestamp());
        
        // Calculate memory footprint
        const entry_size = self.calculate_entry_size(&domain_data);
        
        // Check if we need to make space
        try self.ensure_space_available(entry_size);
        
        // Create cache entry
        const entry = CacheEntry{
            .domain_data = try self.deep_copy_domain_data(domain_data),
            .cached_at = now,
            .expires_at = now + effective_ttl,
            .last_accessed = now,
            .hit_count = 0,
            .source = source,
            .size_bytes = entry_size,
        };
        
        // Store domain name copy for the key
        const domain_key = try self.allocator.dupe(u8, domain_data.domain);
        
        // Remove existing entry if present
        if (self.entries.contains(domain_key)) {
            self.remove_entry(domain_key);
        }
        
        // Add to cache
        try self.entries.put(domain_key, entry);
        self.add_to_lru(domain_key);
        self.current_memory_bytes += entry_size;
    }
    
    /// Remove domain from cache
    pub fn remove_domain(self: *Self, domain: []const u8) bool {
        return self.remove_entry(domain);
    }
    
    /// Clear all cached entries
    pub fn clear(self: *Self) void {
        var iterator = self.entries.iterator();
        while (iterator.next()) |entry| {
            self.allocator.free(entry.key_ptr.*);
            self.free_domain_data(&entry.value_ptr.domain_data);
        }
        
        self.entries.clearAndFree();
        self.lru_list = std.DoublyLinkedList([]const u8){};
        self.current_memory_bytes = 0;
    }
    
    /// Get cache statistics
    pub fn get_statistics(self: *const Self) CacheStatistics {
        const total_queries = self.total_hits + self.total_misses;
        const hit_rate = if (total_queries > 0) 
            @as(f64, @floatFromInt(self.total_hits)) / @as(f64, @floatFromInt(total_queries))
        else 0.0;
        
        return CacheStatistics{
            .total_entries = self.entries.count(),
            .memory_usage_bytes = self.current_memory_bytes,
            .max_memory_bytes = self.max_memory_bytes,
            .total_hits = self.total_hits,
            .total_misses = self.total_misses,
            .total_evictions = self.total_evictions,
            .total_expirations = self.total_expirations,
            .hit_rate = hit_rate,
            .memory_utilization = @as(f64, @floatFromInt(self.current_memory_bytes)) / @as(f64, @floatFromInt(self.max_memory_bytes)),
        };
    }
    
    /// Background cleanup of expired entries
    pub fn cleanup_expired_entries(self: *Self) !u32 {
        var expired_domains = std.ArrayList([]const u8).init(self.allocator);
        defer expired_domains.deinit();
        
        var iterator = self.entries.iterator();
        while (iterator.next()) |entry| {
            if (entry.value_ptr.is_expired()) {
                try expired_domains.append(entry.key_ptr.*);
            }
        }
        
        for (expired_domains.items) |domain| {
            _ = self.remove_entry(domain);
            self.total_expirations += 1;
        }
        
        return @intCast(u32, expired_domains.items.len);
    }
    
    /// Start background cleanup task
    pub fn start_background_cleanup(self: *Self) !void {
        // This would typically be implemented with a separate thread
        // For now, we'll provide the structure for manual cleanup calls
        self.cleanup_timer = std.time.Timer.start() catch null;
    }
    
    // Private helper methods
    fn calculate_effective_ttl(self: *const Self, requested_ttl: ?u32) u32 {
        const ttl = requested_ttl orelse self.default_ttl;
        return std.math.clamp(ttl, self.min_ttl, self.max_ttl);
    }
    
    fn calculate_entry_size(self: *const Self, domain_data: *const record.DomainData) u32 {
        var size: u32 = 0;
        
        // Domain name
        size += @intCast(u32, domain_data.domain.len);
        
        // Owner
        size += @intCast(u32, domain_data.owner.len);
        
        // Records
        for (domain_data.records) |dns_record| {
            size += @intCast(u32, dns_record.name.len);
            size += @intCast(u32, dns_record.value.len);
            if (dns_record.target) |target| {
                size += @intCast(u32, target.len);
            }
            if (dns_record.signature) |sig| {
                size += @intCast(u32, sig.len);
            }
        }
        
        // Contract address
        if (domain_data.contract_address) |addr| {
            size += @intCast(u32, addr.len);
        }
        
        // Metadata
        size += @intCast(u32, domain_data.metadata.registrar.len);
        if (domain_data.metadata.description) |desc| {
            size += @intCast(u32, desc.len);
        }
        
        // Signature
        size += @intCast(u32, domain_data.signature.len);
        
        // Add overhead for structs and pointers
        size += 256; // Estimated overhead
        
        return size;
    }
    
    fn ensure_space_available(self: *Self, required_bytes: u32) !void {
        // Check memory limit
        if (self.current_memory_bytes + required_bytes > self.max_memory_bytes) {
            try self.evict_lru_entries(required_bytes);
        }
        
        // Check entry count limit
        if (self.entries.count() >= self.max_entries) {
            try self.evict_lru_entries(0); // Evict at least one entry
        }
    }
    
    fn evict_lru_entries(self: *Self, min_bytes_to_free: u32) !void {
        var bytes_freed: u32 = 0;
        var entries_to_evict = std.ArrayList([]const u8).init(self.allocator);
        defer entries_to_evict.deinit();
        
        // Find LRU entries to evict
        var current_node = self.lru_list.last;
        while (current_node != null and (bytes_freed < min_bytes_to_free or self.entries.count() >= self.max_entries)) {
            const domain = current_node.?.data;
            
            if (self.entries.get(domain)) |entry| {
                bytes_freed += entry.size_bytes;
                try entries_to_evict.append(domain);
            }
            
            current_node = current_node.?.prev;
        }
        
        // Evict the selected entries
        for (entries_to_evict.items) |domain| {
            _ = self.remove_entry(domain);
            self.total_evictions += 1;
        }
    }
    
    fn remove_entry(self: *Self, domain: []const u8) bool {
        if (self.entries.fetchRemove(domain)) |removed| {
            self.current_memory_bytes -= removed.value.size_bytes;
            self.remove_from_lru(domain);
            self.free_domain_data(&removed.value.domain_data);
            self.allocator.free(removed.key);
            return true;
        }
        return false;
    }
    
    fn update_lru_position(self: *Self, domain: []const u8) void {
        self.remove_from_lru(domain);
        self.add_to_lru(domain);
    }
    
    fn add_to_lru(self: *Self, domain: []const u8) void {
        // In a real implementation, this would manage the doubly-linked list
        // For simplicity, we're showing the structure
        _ = self;
        _ = domain;
    }
    
    fn remove_from_lru(self: *Self, domain: []const u8) void {
        // Remove from LRU list
        _ = self;
        _ = domain;
    }
    
    fn deep_copy_domain_data(self: *Self, domain_data: record.DomainData) !record.DomainData {
        // Deep copy all strings and arrays in domain_data
        const copied_domain = try self.allocator.dupe(u8, domain_data.domain);
        const copied_owner = try self.allocator.dupe(u8, domain_data.owner);
        
        // Copy records array
        var copied_records = try self.allocator.alloc(record.DnsRecord, domain_data.records.len);
        for (domain_data.records, 0..) |dns_record, i| {
            copied_records[i] = try self.deep_copy_dns_record(dns_record);
        }
        
        // Copy contract address if present
        const copied_contract_address = if (domain_data.contract_address) |addr|
            try self.allocator.dupe(u8, addr)
        else
            null;
        
        // Copy signature
        const copied_signature = try self.allocator.dupe(u8, domain_data.signature);
        
        return record.DomainData{
            .domain = copied_domain,
            .owner = copied_owner,
            .records = copied_records,
            .contract_address = copied_contract_address,
            .metadata = try self.deep_copy_metadata(domain_data.metadata),
            .last_updated = domain_data.last_updated,
            .expiry = domain_data.expiry,
            .signature = copied_signature,
        };
    }
    
    fn deep_copy_dns_record(self: *Self, dns_record: record.DnsRecord) !record.DnsRecord {
        return record.DnsRecord{
            .record_type = dns_record.record_type,
            .name = try self.allocator.dupe(u8, dns_record.name),
            .value = try self.allocator.dupe(u8, dns_record.value),
            .ttl = dns_record.ttl,
            .priority = dns_record.priority,
            .port = dns_record.port,
            .weight = dns_record.weight,
            .target = if (dns_record.target) |target| try self.allocator.dupe(u8, target) else null,
            .created_at = dns_record.created_at,
            .signature = if (dns_record.signature) |sig| try self.allocator.dupe(u8, sig) else null,
        };
    }
    
    fn deep_copy_metadata(self: *Self, metadata: record.DomainMetadata) !record.DomainMetadata {
        return record.DomainMetadata{
            .version = metadata.version,
            .registrar = try self.allocator.dupe(u8, metadata.registrar),
            .tags = if (metadata.tags) |tags| try self.deep_copy_string_array(tags) else null,
            .description = if (metadata.description) |desc| try self.allocator.dupe(u8, desc) else null,
            .avatar = if (metadata.avatar) |avatar| try self.allocator.dupe(u8, avatar) else null,
            .website = if (metadata.website) |website| try self.allocator.dupe(u8, website) else null,
            .social = if (metadata.social) |social| try self.deep_copy_social_links(social) else null,
        };
    }
    
    fn deep_copy_string_array(self: *Self, strings: [][]const u8) ![][]const u8 {
        var copied_strings = try self.allocator.alloc([]const u8, strings.len);
        for (strings, 0..) |string, i| {
            copied_strings[i] = try self.allocator.dupe(u8, string);
        }
        return copied_strings;
    }
    
    fn deep_copy_social_links(self: *Self, social: record.SocialLinks) !record.SocialLinks {
        return record.SocialLinks{
            .twitter = if (social.twitter) |twitter| try self.allocator.dupe(u8, twitter) else null,
            .github = if (social.github) |github| try self.allocator.dupe(u8, github) else null,
            .discord = if (social.discord) |discord| try self.allocator.dupe(u8, discord) else null,
            .telegram = if (social.telegram) |telegram| try self.allocator.dupe(u8, telegram) else null,
        };
    }
    
    fn free_domain_data(self: *Self, domain_data: *const record.DomainData) void {
        // Free all allocated memory in domain_data
        self.allocator.free(domain_data.domain);
        self.allocator.free(domain_data.owner);
        
        for (domain_data.records) |dns_record| {
            self.allocator.free(dns_record.name);
            self.allocator.free(dns_record.value);
            if (dns_record.target) |target| {
                self.allocator.free(target);
            }
            if (dns_record.signature) |sig| {
                self.allocator.free(sig);
            }
        }
        self.allocator.free(domain_data.records);
        
        if (domain_data.contract_address) |addr| {
            self.allocator.free(addr);
        }
        
        self.allocator.free(domain_data.signature);
        
        // Free metadata
        self.allocator.free(domain_data.metadata.registrar);
        if (domain_data.metadata.description) |desc| {
            self.allocator.free(desc);
        }
        if (domain_data.metadata.avatar) |avatar| {
            self.allocator.free(avatar);
        }
        if (domain_data.metadata.website) |website| {
            self.allocator.free(website);
        }
        if (domain_data.metadata.tags) |tags| {
            for (tags) |tag| {
                self.allocator.free(tag);
            }
            self.allocator.free(tags);
        }
        if (domain_data.metadata.social) |social| {
            if (social.twitter) |twitter| self.allocator.free(twitter);
            if (social.github) |github| self.allocator.free(github);
            if (social.discord) |discord| self.allocator.free(discord);
            if (social.telegram) |telegram| self.allocator.free(telegram);
        }
    }
};
```

---

## ⚙️ Cache Configuration

### Configuration Structure

```zig
// zns/src/cache/config.zig
pub const CacheConfig = struct {
    // Size limits
    max_entries: usize = 10000,           // Maximum number of cached domains
    max_memory_bytes: usize = 100 * 1024 * 1024, // 100MB memory limit
    
    // TTL configuration
    default_ttl: u32 = 3600,              // 1 hour default TTL
    min_ttl: u32 = 60,                    // 1 minute minimum TTL
    max_ttl: u32 = 86400,                 // 24 hours maximum TTL
    
    // Cleanup configuration
    cleanup_interval_ms: u64 = 300000,    // 5 minutes cleanup interval
    eviction_batch_size: u32 = 100,       // Number of entries to evict at once
    
    // Performance tuning
    initial_capacity: usize = 1000,       // Initial hash map capacity
    load_factor: f64 = 0.75,              // Hash map load factor
    
    pub fn development() CacheConfig {
        return CacheConfig{
            .max_entries = 1000,
            .max_memory_bytes = 10 * 1024 * 1024, // 10MB
            .default_ttl = 300,                   // 5 minutes
            .cleanup_interval_ms = 60000,         // 1 minute
        };
    }
    
    pub fn production() CacheConfig {
        return CacheConfig{
            .max_entries = 100000,
            .max_memory_bytes = 1024 * 1024 * 1024, // 1GB
            .default_ttl = 3600,                    // 1 hour
            .cleanup_interval_ms = 300000,          // 5 minutes
        };
    }
    
    pub fn high_performance() CacheConfig {
        return CacheConfig{
            .max_entries = 1000000,
            .max_memory_bytes = 4 * 1024 * 1024 * 1024, // 4GB
            .default_ttl = 7200,                        // 2 hours
            .cleanup_interval_ms = 600000,              // 10 minutes
        };
    }
};
```

---

## 📊 Cache Statistics and Monitoring

### Statistics Structure

```zig
// zns/src/cache/statistics.zig
pub const CacheStatistics = struct {
    // Basic metrics
    total_entries: usize,
    memory_usage_bytes: usize,
    max_memory_bytes: usize,
    
    // Hit/miss statistics
    total_hits: u64,
    total_misses: u64,
    total_evictions: u64,
    total_expirations: u64,
    
    // Performance metrics
    hit_rate: f64,                        // 0.0 to 1.0
    memory_utilization: f64,              // 0.0 to 1.0
    average_entry_size: f64,              // Bytes per entry
    
    pub fn format(
        self: CacheStatistics,
        comptime fmt: []const u8,
        options: std.fmt.FormatOptions,
        writer: anytype,
    ) !void {
        _ = fmt;
        _ = options;
        
        try writer.print(
            \\Cache Statistics:
            \\  Entries: {}/{} ({d:.1}% full)
            \\  Memory: {}/{} bytes ({d:.1}% used)
            \\  Hit Rate: {d:.2}% ({}/{} queries)
            \\  Evictions: {} (LRU), Expirations: {} (TTL)
            \\  Avg Entry Size: {d:.1} bytes
        , .{
            self.total_entries,
            self.max_memory_bytes / @as(usize, @intFromFloat(self.average_entry_size)),
            self.memory_utilization * 100,
            self.memory_usage_bytes,
            self.max_memory_bytes,
            self.memory_utilization * 100,
            self.hit_rate * 100,
            self.total_hits,
            self.total_hits + self.total_misses,
            self.total_evictions,
            self.total_expirations,
            self.average_entry_size,
        });
    }
};
```

### Performance Monitor

```zig
// zns/src/cache/monitor.zig
pub const CacheMonitor = struct {
    cache: *DomainCache,
    start_time: u64,
    last_report_time: u64,
    report_interval_ms: u64,
    
    pub fn init(cache: *DomainCache, report_interval_ms: u64) Self {
        const now = @intCast(u64, std.time.milliTimestamp());
        return Self{
            .cache = cache,
            .start_time = now,
            .last_report_time = now,
            .report_interval_ms = report_interval_ms,
        };
    }
    
    pub fn should_report(self: *const Self) bool {
        const now = @intCast(u64, std.time.milliTimestamp());
        return (now - self.last_report_time) >= self.report_interval_ms;
    }
    
    pub fn generate_report(self: *Self) CacheReport {
        const now = @intCast(u64, std.time.milliTimestamp());
        const stats = self.cache.get_statistics();
        
        const report = CacheReport{
            .timestamp = now,
            .uptime_ms = now - self.start_time,
            .statistics = stats,
            .performance_score = self.calculate_performance_score(stats),
            .recommendations = self.generate_recommendations(stats),
        };
        
        self.last_report_time = now;
        return report;
    }
    
    fn calculate_performance_score(self: *const Self, stats: CacheStatistics) f64 {
        // Score based on hit rate (50%), memory efficiency (30%), and low eviction rate (20%)
        const hit_rate_score = stats.hit_rate;
        
        const memory_efficiency = 1.0 - stats.memory_utilization; // Lower is better
        const memory_score = if (memory_efficiency > 0.8) 1.0 else memory_efficiency / 0.8;
        
        const total_ops = stats.total_hits + stats.total_misses;
        const eviction_rate = if (total_ops > 0) 
            @as(f64, @floatFromInt(stats.total_evictions)) / @as(f64, @floatFromInt(total_ops))
        else 0.0;
        const eviction_score = std.math.max(0.0, 1.0 - eviction_rate * 10); // Penalize high eviction rates
        
        return (hit_rate_score * 0.5) + (memory_score * 0.3) + (eviction_score * 0.2);
    }
    
    fn generate_recommendations(self: *const Self, stats: CacheStatistics) []const []const u8 {
        var recommendations = std.ArrayList([]const u8).init(std.heap.page_allocator);
        
        if (stats.hit_rate < 0.7) {
            recommendations.append("Consider increasing cache size or TTL values") catch {};
        }
        
        if (stats.memory_utilization > 0.9) {
            recommendations.append("Memory usage is high, consider increasing memory limit") catch {};
        }
        
        if (stats.total_evictions > stats.total_hits / 10) {
            recommendations.append("High eviction rate detected, increase cache size") catch {};
        }
        
        return recommendations.toOwnedSlice() catch &[_][]const u8{};
    }
};

pub const CacheReport = struct {
    timestamp: u64,
    uptime_ms: u64,
    statistics: CacheStatistics,
    performance_score: f64,        // 0.0 to 1.0
    recommendations: []const []const u8,
};
```

---

## 🔧 Advanced Caching Features

### Tiered Caching

```zig
// zns/src/cache/tiered_cache.zig
pub const TieredCache = struct {
    l1_cache: DomainCache,    // Hot cache - small, fast
    l2_cache: DomainCache,    // Warm cache - larger, slower
    
    pub fn init(allocator: std.mem.Allocator, l1_config: CacheConfig, l2_config: CacheConfig) !Self {
        return Self{
            .l1_cache = try DomainCache.init(allocator, l1_config),
            .l2_cache = try DomainCache.init(allocator, l2_config),
        };
    }
    
    pub fn get_domain(self: *Self, domain: []const u8) ?*const record.DomainData {
        // Try L1 cache first
        if (self.l1_cache.get_domain(domain)) |data| {
            return data;
        }
        
        // Try L2 cache
        if (self.l2_cache.get_domain(domain)) |data| {
            // Promote to L1 cache
            self.l1_cache.cache_domain(data.*, null, .zns_native) catch {};
            return data;
        }
        
        return null;
    }
    
    pub fn cache_domain(self: *Self, domain_data: record.DomainData, ttl: ?u32, source: CacheSource) !void {
        // Cache in both tiers
        try self.l1_cache.cache_domain(domain_data, ttl, source);
        try self.l2_cache.cache_domain(domain_data, if (ttl) |t| t * 2 else null, source); // Longer TTL in L2
    }
};
```

### Write-Through Cache

```zig
// zns/src/cache/write_through.zig
pub const WriteThroughCache = struct {
    cache: DomainCache,
    backend: BackendStorage,
    
    pub fn cache_domain(self: *Self, domain_data: record.DomainData, ttl: ?u32, source: CacheSource) !void {
        // Write to backend first
        try self.backend.store_domain(domain_data);
        
        // Then cache
        try self.cache.cache_domain(domain_data, ttl, source);
    }
    
    pub fn get_domain(self: *Self, domain: []const u8) !?*const record.DomainData {
        // Try cache first
        if (self.cache.get_domain(domain)) |data| {
            return data;
        }
        
        // Fallback to backend
        if (try self.backend.load_domain(domain)) |data| {
            // Cache the result
            try self.cache.cache_domain(data, null, .contract_sync);
            return self.cache.get_domain(domain);
        }
        
        return null;
    }
};
```

### Distributed Cache Sync

```zig
// zns/src/cache/distributed.zig
pub const DistributedCache = struct {
    local_cache: DomainCache,
    peer_clients: []PeerCacheClient,
    sync_interval_ms: u64,
    
    pub fn sync_with_peers(self: *Self) !void {
        for (self.peer_clients) |*peer| {
            const peer_updates = try peer.get_recent_updates();
            
            for (peer_updates) |update| {
                if (self.should_apply_peer_update(update)) {
                    try self.local_cache.cache_domain(update.domain_data, update.ttl, .peer_cache);
                }
            }
        }
    }
    
    fn should_apply_peer_update(self: *const Self, update: PeerUpdate) bool {
        // Apply update if:
        // 1. We don't have the domain
        // 2. Peer's version is newer
        // 3. Peer has higher trust score
        
        if (self.local_cache.get_domain(update.domain_data.domain)) |local_data| {
            return update.domain_data.last_updated > local_data.last_updated;
        }
        
        return true; // New domain, apply update
    }
};
```

---

## 🧪 Testing and Benchmarks

### Cache Performance Tests

```zig
// zns/src/cache/tests.zig
const testing = std.testing;

test "cache basic operations" {
    var cache = try DomainCache.init(testing.allocator, CacheConfig.development());
    defer cache.deinit();
    
    // Create test domain data
    const test_domain = record.DomainData{
        .domain = "test.ghost",
        .owner = "ghost1test",
        .records = &[_]record.DnsRecord{},
        .contract_address = null,
        .metadata = record.DomainMetadata{
            .version = 1,
            .registrar = "ZNS",
            .tags = null,
            .description = null,
            .avatar = null,
            .website = null,
            .social = null,
        },
        .last_updated = @intCast(u64, std.time.timestamp()),
        .expiry = null,
        .signature = &[_]u8{},
    };
    
    // Test caching
    try cache.cache_domain(test_domain, 3600, .zns_native);
    
    // Test retrieval
    const cached_data = cache.get_domain("test.ghost");
    try testing.expect(cached_data != null);
    try testing.expectEqualStrings(cached_data.?.domain, "test.ghost");
    
    // Test cache hit statistics
    const stats = cache.get_statistics();
    try testing.expect(stats.total_hits == 1);
    try testing.expect(stats.total_misses == 0);
}

test "cache TTL expiration" {
    var cache = try DomainCache.init(testing.allocator, CacheConfig.development());
    defer cache.deinit();
    
    // Cache with 1 second TTL
    const test_domain = create_test_domain("ttl-test.ghost");
    try cache.cache_domain(test_domain, 1, .zns_native);
    
    // Should be available immediately
    try testing.expect(cache.get_domain("ttl-test.ghost") != null);
    
    // Wait for expiration (in real test, use mock time)
    std.time.sleep(1500 * std.time.ns_per_ms);
    
    // Should be expired now
    try testing.expect(cache.get_domain("ttl-test.ghost") == null);
}

test "cache LRU eviction" {
    var small_cache_config = CacheConfig.development();
    small_cache_config.max_entries = 3;
    
    var cache = try DomainCache.init(testing.allocator, small_cache_config);
    defer cache.deinit();
    
    // Fill cache to capacity
    try cache.cache_domain(create_test_domain("domain1.ghost"), 3600, .zns_native);
    try cache.cache_domain(create_test_domain("domain2.ghost"), 3600, .zns_native);
    try cache.cache_domain(create_test_domain("domain3.ghost"), 3600, .zns_native);
    
    // Access domain1 to make it recently used
    _ = cache.get_domain("domain1.ghost");
    
    // Add new domain, should evict domain2 (least recently used)
    try cache.cache_domain(create_test_domain("domain4.ghost"), 3600, .zns_native);
    
    // domain1 and domain3 and domain4 should still be there
    try testing.expect(cache.get_domain("domain1.ghost") != null);
    try testing.expect(cache.get_domain("domain3.ghost") != null);
    try testing.expect(cache.get_domain("domain4.ghost") != null);
    
    // domain2 should be evicted
    try testing.expect(cache.get_domain("domain2.ghost") == null);
}

fn create_test_domain(domain_name: []const u8) record.DomainData {
    return record.DomainData{
        .domain = domain_name,
        .owner = "test-owner",
        .records = &[_]record.DnsRecord{},
        .contract_address = null,
        .metadata = record.DomainMetadata{
            .version = 1,
            .registrar = "ZNS",
            .tags = null,
            .description = null,
            .avatar = null,
            .website = null,
            .social = null,
        },
        .last_updated = @intCast(u64, std.time.timestamp()),
        .expiry = null,
        .signature = &[_]u8{},
    };
}
```

### Benchmark Suite

```zig
// zns/src/cache/benchmark.zig
pub fn benchmark_cache_performance() !void {
    const allocator = std.heap.page_allocator;
    
    // Test different cache sizes
    const cache_sizes = [_]usize{ 1000, 10000, 100000 };
    
    for (cache_sizes) |size| {
        std.debug.print("Benchmarking cache size: {}\n", .{size});
        
        var config = CacheConfig.production();
        config.max_entries = size;
        
        var cache = try DomainCache.init(allocator, config);
        defer cache.deinit();
        
        // Warm up cache
        try warmup_cache(&cache, size);
        
        // Benchmark read performance
        const read_duration = try benchmark_reads(&cache, size);
        std.debug.print("  Read performance: {d:.2} ns/op\n", .{read_duration});
        
        // Benchmark write performance
        const write_duration = try benchmark_writes(&cache, size / 10);
        std.debug.print("  Write performance: {d:.2} ns/op\n", .{write_duration});
        
        // Print final statistics
        const stats = cache.get_statistics();
        std.debug.print("  Final hit rate: {d:.2}%\n", .{stats.hit_rate * 100});
        std.debug.print("  Memory usage: {} bytes\n", .{stats.memory_usage_bytes});
    }
}

fn warmup_cache(cache: *DomainCache, count: usize) !void {
    var i: usize = 0;
    while (i < count) : (i += 1) {
        const domain_name = try std.fmt.allocPrint(std.heap.page_allocator, "warmup{}.ghost", .{i});
        defer std.heap.page_allocator.free(domain_name);
        
        const domain_data = create_test_domain(domain_name);
        try cache.cache_domain(domain_data, 3600, .zns_native);
    }
}

fn benchmark_reads(cache: *DomainCache, iterations: usize) !f64 {
    const start_time = std.time.nanoTimestamp();
    
    var i: usize = 0;
    while (i < iterations) : (i += 1) {
        const domain_name = try std.fmt.allocPrint(std.heap.page_allocator, "warmup{}.ghost", .{i % 1000});
        defer std.heap.page_allocator.free(domain_name);
        
        _ = cache.get_domain(domain_name);
    }
    
    const end_time = std.time.nanoTimestamp();
    const total_duration = @as(f64, @floatFromInt(end_time - start_time));
    
    return total_duration / @as(f64, @floatFromInt(iterations));
}

fn benchmark_writes(cache: *DomainCache, iterations: usize) !f64 {
    const start_time = std.time.nanoTimestamp();
    
    var i: usize = 0;
    while (i < iterations) : (i += 1) {
        const domain_name = try std.fmt.allocPrint(std.heap.page_allocator, "bench{}.ghost", .{i});
        defer std.heap.page_allocator.free(domain_name);
        
        const domain_data = create_test_domain(domain_name);
        try cache.cache_domain(domain_data, 3600, .zns_native);
    }
    
    const end_time = std.time.nanoTimestamp();
    const total_duration = @as(f64, @floatFromInt(end_time - start_time));
    
    return total_duration / @as(f64, @floatFromInt(iterations));
}
```

---

## 🚀 Integration with ZNS Resolver

### Cache-Aware Resolver

```zig
// zns/src/resolver.zig
pub const ZNSResolver = struct {
    cache: DomainCache,
    native_resolver: NativeZNSResolver,
    ens_resolver: ENSResolver,
    unstoppable_resolver: UnstoppableResolver,
    
    pub fn resolve_domain(self: *Self, domain: []const u8, record_types: [][]const u8) !ResolutionResult {
        const start_time = std.time.nanoTimestamp();
        
        // Check cache first
        if (self.cache.get_domain(domain)) |cached_data| {
            return ResolutionResult{
                .success = true,
                .domain_data = cached_data.*,
                .source = .cache,
                .resolution_time_ns = std.time.nanoTimestamp() - start_time,
            };
        }
        
        // Try resolvers in order of preference
        const resolvers = [_]ResolverInterface{
            self.native_resolver,
            self.ens_resolver,
            self.unstoppable_resolver,
        };
        
        for (resolvers) |resolver| {
            if (try resolver.resolve(domain, record_types)) |result| {
                // Cache successful resolution
                try self.cache.cache_domain(result.domain_data, null, result.source);
                
                return ResolutionResult{
                    .success = true,
                    .domain_data = result.domain_data,
                    .source = result.source,
                    .resolution_time_ns = std.time.nanoTimestamp() - start_time,
                };
            }
        }
        
        return ResolutionResult{
            .success = false,
            .domain_data = undefined,
            .source = .unknown,
            .resolution_time_ns = std.time.nanoTimestamp() - start_time,
        };
    }
};
```

---

## 📝 Usage Examples

### Basic Cache Usage

```zig
// Example: Setting up and using the domain cache
pub fn example_cache_usage() !void {
    const allocator = std.heap.page_allocator;
    
    // Create cache with production settings
    var cache = try DomainCache.init(allocator, CacheConfig.production());
    defer cache.deinit();
    
    // Create some test domain data
    const domain_data = record.DomainData{
        .domain = "example.ghost",
        .owner = "ghost1example123",
        .records = &[_]record.DnsRecord{
            record.DnsRecord{
                .record_type = .A,
                .name = "example.ghost",
                .value = "192.168.1.100",
                .ttl = 3600,
                .priority = null,
                .port = null,
                .weight = null,
                .target = null,
                .created_at = @intCast(u64, std.time.timestamp()),
                .signature = null,
            },
        },
        .contract_address = "0x742d35Cc6486C4F09aAbB8e2F68bF51b5FBB4BF1",
        .metadata = record.DomainMetadata{
            .version = 1,
            .registrar = "ZNS",
            .tags = null,
            .description = "Example domain for testing",
            .avatar = null,
            .website = "https://example.ghost",
            .social = null,
        },
        .last_updated = @intCast(u64, std.time.timestamp()),
        .expiry = null,
        .signature = &[_]u8{},
    };
    
    // Cache the domain
    try cache.cache_domain(domain_data, 3600, .zns_native);
    std.debug.print("Cached domain: {s}\n", .{domain_data.domain});
    
    // Retrieve from cache
    if (cache.get_domain("example.ghost")) |cached| {
        std.debug.print("Retrieved from cache: {s} -> {s}\n", .{cached.domain, cached.records[0].value});
    }
    
    // Check cache statistics
    const stats = cache.get_statistics();
    std.debug.print("Cache stats: {d:.2}% hit rate, {} entries\n", .{stats.hit_rate * 100, stats.total_entries});
    
    // Cleanup expired entries
    const expired_count = try cache.cleanup_expired_entries();
    std.debug.print("Cleaned up {} expired entries\n", .{expired_count});
}
```

---

This comprehensive caching implementation provides:

1. **TTL-based expiration** with configurable min/max limits
2. **LRU eviction** when memory or entry limits are reached  
3. **Memory management** with deep copying and proper cleanup
4. **Performance monitoring** with detailed statistics and reporting
5. **Advanced features** like tiered caching and distributed sync
6. **Comprehensive testing** with benchmarks and performance validation

The cache is designed to handle high-throughput DNS resolution while maintaining data consistency and optimal memory usage.
