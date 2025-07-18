const std = @import("std");
const shroud = @import("shroud");
const ghostwire = shroud.ghostwire;

pub const QuicServer = struct {
    allocator: std.mem.Allocator,
    server: ghostwire.Server,
    bind_address: []const u8,
    port: u16,
    running: bool,
    handlers: std.HashMap([]const u8, RequestHandler, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator, bind_address: []const u8, port: u16) !QuicServer {
        const server = try ghostwire.Server.init(allocator, .{
            .bind_address = bind_address,
            .port = port,
            .use_ipv6 = true,
            .max_connections = 10000,
            .idle_timeout_ms = 300000, // 5 minutes
            .alpn_protocols = &[_][]const u8{ "ghostchain/1.0", "h3" },
        });
        
        return QuicServer{
            .allocator = allocator,
            .server = server,
            .bind_address = bind_address,
            .port = port,
            .running = false,
            .handlers = std.HashMap([]const u8, RequestHandler, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    pub fn deinit(self: *QuicServer) void {
        self.server.deinit();
        self.handlers.deinit();
    }
    
    pub fn registerHandler(self: *QuicServer, method: []const u8, handler: RequestHandler) !void {
        try self.handlers.put(method, handler);
    }
    
    pub fn start(self: *QuicServer) !void {
        std.log.info("🚀 Starting QUIC server on {s}:{}", .{ self.bind_address, self.port });
        
        // Register default handlers
        try self.registerHandler("ping", handlePing);
        try self.registerHandler("get_block", handleGetBlock);
        try self.registerHandler("submit_transaction", handleSubmitTransaction);
        try self.registerHandler("get_balance", handleGetBalance);
        
        self.running = true;
        
        try self.server.start();
        
        std.log.info("✅ QUIC server listening for connections", .{});
        
        // Main connection handling loop
        while (self.running) {
            // Accept new connections
            const connection = self.server.acceptConnection() catch |err| switch (err) {
                error.WouldBlock => {
                    std.time.sleep(1_000_000); // 1ms
                    continue;
                },
                else => return err,
            };
            
            // Handle connection in separate thread
            _ = try std.Thread.spawn(.{}, handleConnection, .{ self, connection });
        }
    }
    
    pub fn stop(self: *QuicServer) void {
        std.log.info("🛑 Stopping QUIC server", .{});
        self.running = false;
        self.server.stop();
    }
    
    fn handleConnection(self: *QuicServer, connection: ghostwire.Connection) void {
        defer connection.close();
        
        std.log.debug("📡 New QUIC connection from {any}", .{connection.remote_address()});
        
        while (connection.isAlive()) {
            // Accept new streams
            const stream = connection.acceptStream() catch |err| switch (err) {
                error.WouldBlock => {
                    std.time.sleep(1_000_000); // 1ms
                    continue;
                },
                error.ConnectionClosed => break,
                else => {
                    std.log.err("Stream accept error: {any}", .{err});
                    continue;
                },
            };
            
            // Handle stream in separate thread for concurrency
            _ = std.Thread.spawn(.{}, handleStream, .{ self, stream }) catch {
                stream.close();
                continue;
            };
        }
        
        std.log.debug("📡 QUIC connection closed", .{});
    }
    
    fn handleStream(self: *QuicServer, stream: ghostwire.Stream) void {
        defer stream.close();
        
        // Read request data
        const request_data = stream.readAll(self.allocator, 1024 * 1024) catch |err| {
            std.log.err("Failed to read stream data: {any}", .{err});
            return;
        };
        defer self.allocator.free(request_data);
        
        // Parse JSON request
        const request = std.json.parseFromSlice(BlockchainRequest, self.allocator, request_data, .{}) catch |err| {
            std.log.err("Failed to parse request JSON: {any}", .{err});
            return;
        };
        defer request.deinit();
        
        // Look up handler
        const handler = self.handlers.get(request.value.method) orelse {
            const error_response = BlockchainResponse{
                .result = null,
                .@"error" = "Method not found",
                .id = request.value.id,
            };
            
            const response_data = std.json.stringifyAlloc(self.allocator, error_response, .{}) catch return;
            defer self.allocator.free(response_data);
            
            stream.write(response_data) catch {};
            return;
        };
        
        // Execute handler
        const response = handler(self.allocator, request.value);
        
        // Send response
        const response_data = std.json.stringifyAlloc(self.allocator, response, .{}) catch return;
        defer self.allocator.free(response_data);
        
        stream.write(response_data) catch {};
        stream.finish() catch {};
    }
};

// Default request handlers
fn handlePing(allocator: std.mem.Allocator, request: BlockchainRequest) BlockchainResponse {
    _ = allocator;
    return BlockchainResponse{
        .result = std.json.Value{ .string = "pong" },
        .@"error" = null,
        .id = request.id,
    };
}

fn handleGetBlock(allocator: std.mem.Allocator, request: BlockchainRequest) BlockchainResponse {
    // TODO: Implement actual block retrieval
    return BlockchainResponse{
        .result = std.json.Value{ .object = std.json.ObjectMap.init(allocator) },
        .@"error" = "Not implemented",
        .id = request.id,
    };
}

fn handleSubmitTransaction(allocator: std.mem.Allocator, request: BlockchainRequest) BlockchainResponse {
    _ = allocator;
    // TODO: Implement transaction submission
    return BlockchainResponse{
        .result = std.json.Value{ .string = "0x1234567890abcdef" },
        .@"error" = "Not implemented",
        .id = request.id,
    };
}

fn handleGetBalance(allocator: std.mem.Allocator, request: BlockchainRequest) BlockchainResponse {
    _ = allocator;
    // TODO: Implement balance retrieval
    return BlockchainResponse{
        .result = std.json.Value{ .integer = 1000000 },
        .@"error" = "Not implemented",
        .id = request.id,
    };
}

// Type definitions
const BlockchainRequest = @import("client.zig").BlockchainRequest;
const BlockchainResponse = @import("client.zig").BlockchainResponse;

pub const RequestHandler = *const fn (allocator: std.mem.Allocator, request: BlockchainRequest) BlockchainResponse;