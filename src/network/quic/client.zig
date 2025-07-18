const std = @import("std");
const shroud = @import("shroud");
const ghostwire = shroud.ghostwire;

pub const QuicClient = struct {
    allocator: std.mem.Allocator,
    connection: ?ghostwire.Connection,
    server_address: []const u8,
    server_port: u16,
    
    pub fn init(allocator: std.mem.Allocator, server_address: []const u8, server_port: u16) QuicClient {
        return QuicClient{
            .allocator = allocator,
            .connection = null,
            .server_address = server_address,
            .server_port = server_port,
        };
    }
    
    pub fn deinit(self: *QuicClient) void {
        if (self.connection) |*conn| {
            conn.close();
        }
    }
    
    pub fn connect(self: *QuicClient) !void {
        std.log.info("📡 Connecting to QUIC server {s}:{}", .{ self.server_address, self.server_port });
        
        // Initialize QUIC connection with IPv6 support
        self.connection = try ghostwire.Connection.connect(self.allocator, .{
            .server_address = self.server_address,
            .server_port = self.server_port,
            .use_ipv6 = true,
            .enable_0rtt = true, // Enable 0-RTT for faster reconnections
            .alpn_protocols = &[_][]const u8{ "ghostchain/1.0", "h3" },
        });
        
        std.log.info("✅ QUIC connection established", .{});
    }
    
    pub fn sendBlockchainRequest(self: *QuicClient, request: BlockchainRequest) !BlockchainResponse {
        const conn = self.connection orelse return error.NotConnected;
        
        // Create bidirectional stream for request/response
        var stream = try conn.openBidirectionalStream();
        defer stream.close();
        
        // Serialize and send request
        const request_data = try std.json.stringifyAlloc(self.allocator, request, .{});
        defer self.allocator.free(request_data);
        
        try stream.write(request_data);
        try stream.finish();
        
        // Read response
        const response_data = try stream.readAll(self.allocator, 1024 * 1024); // 1MB max
        defer self.allocator.free(response_data);
        
        // Parse response
        const response = try std.json.parseFromSlice(BlockchainResponse, self.allocator, response_data, .{});
        defer response.deinit();
        
        return response.value;
    }
    
    pub fn subscribeToBlocks(self: *QuicClient, callback: BlockCallback) !void {
        const conn = self.connection orelse return error.NotConnected;
        
        // Create unidirectional stream for subscription
        var stream = try conn.openUnidirectionalStream();
        
        // Send subscription request
        const subscription = SubscriptionRequest{
            .type = .blocks,
            .filters = null,
        };
        
        const sub_data = try std.json.stringifyAlloc(self.allocator, subscription, .{});
        defer self.allocator.free(sub_data);
        
        try stream.write(sub_data);
        try stream.finish();
        
        // Listen for block events
        while (true) {
            const event_data = stream.read(self.allocator, 1024 * 1024) catch |err| switch (err) {
                error.StreamClosed => break,
                else => return err,
            };
            defer self.allocator.free(event_data);
            
            const block_event = std.json.parseFromSlice(BlockEvent, self.allocator, event_data, .{}) catch continue;
            defer block_event.deinit();
            
            callback(block_event.value);
        }
    }
    
    pub fn ping(self: *QuicClient) !u64 {
        const start_time = std.time.nanoTimestamp();
        
        const ping_request = BlockchainRequest{
            .method = "ping",
            .params = null,
            .id = 1,
        };
        
        _ = try self.sendBlockchainRequest(ping_request);
        
        const end_time = std.time.nanoTimestamp();
        return @as(u64, @intCast(end_time - start_time)) / 1_000_000; // Convert to milliseconds
    }
};

pub const BlockchainRequest = struct {
    method: []const u8,
    params: ?std.json.Value,
    id: u64,
};

pub const BlockchainResponse = struct {
    result: ?std.json.Value,
    @"error": ?[]const u8,
    id: u64,
};

pub const SubscriptionRequest = struct {
    type: SubscriptionType,
    filters: ?std.json.Value,
};

pub const SubscriptionType = enum {
    blocks,
    transactions,
    domains,
};

pub const BlockEvent = struct {
    block_number: u64,
    block_hash: []const u8,
    timestamp: u64,
    transactions: []TransactionSummary,
};

pub const TransactionSummary = struct {
    hash: []const u8,
    from: []const u8,
    to: []const u8,
    amount: u64,
};

pub const BlockCallback = *const fn (event: BlockEvent) void;