const std = @import("std");
const ghostchain = @import("../root.zig");
const shroud = @import("shroud");
const wraith = @import("wraith");
const zsync = @import("zsync");

pub const GhostDaemon = struct {
    allocator: std.mem.Allocator,
    config: DaemonConfig,
    blockchain: ghostchain.GhostChain,
    async_runtime: zsync.Runtime,
    quic_server: ?*anyopaque, // Will be replaced with zquic integration
    wraith_proxy: ?*anyopaque, // Will be replaced with wraith integration
    mempool: TransactionMempool,
    peer_manager: PeerManager,
    block_producer: BlockProducer,
    running: bool,
    
    pub fn init(allocator: std.mem.Allocator, config: DaemonConfig) !GhostDaemon {
        // Initialize core blockchain
        const ghost_config = ghostchain.GhostChainConfig{
            .bind_address = config.bind_address,
            .port = config.p2p_port,
            .rpc_address = config.rpc_address,
            .rpc_port = config.rpc_port,
            .minimum_stake = config.minimum_stake,
            .gas_limit = config.gas_limit,
        };
        
        var blockchain = try ghostchain.GhostChain.init(allocator, ghost_config);
        
        // Initialize async runtime for high-performance operations
        var async_runtime = try zsync.Runtime.init(allocator);
        
        // Initialize QUIC server for P2P networking (stub for now)
        const quic_server: ?*anyopaque = null;
        
        // Initialize Wraith proxy for HTTP3/Web5 services (stub for now)
        const wraith_proxy: ?*anyopaque = null;
        
        return GhostDaemon{
            .allocator = allocator,
            .config = config,
            .blockchain = blockchain,
            .async_runtime = async_runtime,
            .quic_server = quic_server,
            .wraith_proxy = wraith_proxy,
            .mempool = TransactionMempool.init(allocator, &blockchain.token_system),
            .peer_manager = PeerManager.init(allocator),
            .block_producer = BlockProducer.init(allocator),
            .running = false,
        };
    }
    
    pub fn deinit(self: *GhostDaemon) void {
        self.blockchain.deinit();
        self.async_runtime.deinit();
        // self.quic_server.deinit();
        // self.wraith_proxy.deinit();
        self.mempool.deinit();
        self.peer_manager.deinit();
        self.block_producer.deinit();
    }
    
    pub fn start(self: *GhostDaemon) !void {
        std.log.info("🚀 Starting GhostD daemon v0.1.0", .{});
        std.log.info("🌐 P2P QUIC server: {s}:{}", .{ self.config.bind_address, self.config.p2p_port });
        std.log.info("🔌 RPC server: {s}:{}", .{ self.config.rpc_address, self.config.rpc_port });
        std.log.info("🌍 HTTP3 Wraith proxy: {s}:{}", .{ self.config.bind_address, self.config.http3_port });
        
        self.running = true;
        
        // Start core blockchain
        try self.blockchain.start();
        
        // Set up Wraith routes for Web5 services
        try self.setupWraithRoutes();
        
        // Start QUIC P2P server (disabled until properly implemented)
        // try self.quic_server.start();
        
        // Start Wraith HTTP3 proxy (disabled until properly implemented)  
        // try self.wraith_proxy.start();
        
        // Start background services
        try self.startBackgroundServices();
        
        std.log.info("✅ GhostD is running!", .{});
        
        // Main event loop
        try self.runEventLoop();
    }
    
    pub fn stop(self: *GhostDaemon) void {
        std.log.info("🛑 Stopping GhostD daemon...", .{});
        self.running = false;
        // self.quic_server.stop();
        // self.wraith_proxy.stop();
    }
    
    fn setupWraithRoutes(self: *GhostDaemon) !void {
        _ = self;
        // TEMP: Wraith Web5 gateway routes disabled until wraith_proxy is properly implemented
        // When implemented, this will handle:
        // - Web5 domain landing pages (*.ghost domains)
        // - Blockchain RPC API endpoints
        // - Contract deployment and interaction
        // - ZNS domain resolution
        // - RealID identity services
        // - Health and metrics endpoints
    }
    
    fn startBackgroundServices(self: *GhostDaemon) !void {
        // Start mempool processor
        _ = try std.Thread.spawn(.{}, mempoolProcessor, .{self});
        
        // Start block producer
        _ = try std.Thread.spawn(.{}, blockProducerLoop, .{self});
        
        // Start peer discovery
        _ = try std.Thread.spawn(.{}, peerDiscoveryLoop, .{self});
        
        // Start sync service
        _ = try std.Thread.spawn(.{}, blockSyncLoop, .{self});
    }
    
    fn runEventLoop(self: *GhostDaemon) !void {
        while (self.running) {
            // Process QUIC connections (disabled until properly implemented)
            // try self.quic_server.processEvents(100); // 100ms timeout
            
            // Process HTTP3 requests (disabled until properly implemented)
            // try self.wraith_proxy.processRequests(100);
            
            // Update metrics
            self.updateMetrics();
            
            std.time.sleep(10_000_000); // 10ms sleep
        }
    }
    
    // Route handlers
    fn handleDomainLanding(self: *GhostDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const domain = request.getParam("domain");
        
        // Resolve domain using ZNS
        const domain_record = self.blockchain.resolveDomain(domain) catch |err| switch (err) {
            error.DomainNotFound => {
                response.setStatus(404);
                try response.writeBody("Domain not found");
                return;
            },
            else => return err,
        };
        
        // Generate Web5 landing page
        const landing_page = try self.generateDomainLandingPage(domain, domain_record);
        defer self.allocator.free(landing_page);
        
        response.setHeader("Content-Type", "text/html; charset=utf-8");
        try response.writeBody(landing_page);
    }
    
    fn handleRPCRequest(self: *GhostDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const body = try request.readBody(self.allocator);
        defer self.allocator.free(body);
        
        // Parse JSON-RPC request
        const rpc_request = try std.json.parseFromSlice(JSONRPCRequest, self.allocator, body, .{});
        defer rpc_request.deinit();
        
        // Process RPC method
        const rpc_response = try self.processRPCMethod(rpc_request.value);
        defer self.allocator.free(rpc_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(rpc_response);
    }
    
    fn handleContractAPI(self: *GhostDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const path_parts = std.mem.split(u8, request.path, "/");
        _ = path_parts.next(); // skip empty
        _ = path_parts.next(); // skip "api"
        _ = path_parts.next(); // skip "v1" 
        _ = path_parts.next(); // skip "contracts"
        
        const action = path_parts.next() orelse "list";
        
        if (std.mem.eql(u8, action, "deploy")) {
            try self.handleContractDeploy(request, response);
        } else if (std.mem.eql(u8, action, "call")) {
            try self.handleContractCall(request, response);
        } else {
            response.setStatus(400);
            try response.writeBody("Invalid contract action");
        }
    }
    
    fn handleZNSResolution(self: *GhostDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const domain = request.getParam("domain");
        
        const record = self.blockchain.resolveDomain(domain) catch |err| switch (err) {
            error.DomainNotFound => {
                response.setStatus(404);
                try response.writeBody("{\"error\": \"Domain not found\"}");
                return;
            },
            else => return err,
        };
        
        const json_response = try std.json.stringifyAlloc(self.allocator, record, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(json_response);
    }
    
    fn handleIdentityAPI(_: *GhostDaemon, _: *wraith.Request, response: *wraith.Response) !void {
        // TODO: Implement RealID API handlers
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"RealID API coming soon\"}");
    }
    
    fn handleHealthCheck(self: *GhostDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = request;
        
        const health_status = HealthStatus{
            .status = "healthy",
            .uptime = self.getUptime(),
            .peers_connected = self.peer_manager.getConnectedPeerCount(),
            .latest_block = self.blockchain.blockchain.index,
            .mempool_size = self.mempool.getTransactionCount(),
        };
        
        const json_response = try std.json.stringifyAlloc(self.allocator, health_status, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(json_response);
    }
    
    fn handleMetrics(_: *GhostDaemon, _: *wraith.Request, response: *wraith.Response) !void {
        // TODO: Implement Prometheus metrics
        try response.writeBody("# Prometheus metrics coming soon\n");
    }
    
    // Background service functions
    fn mempoolProcessor(self: *GhostDaemon) void {
        while (self.running) {
            self.mempool.processPendingTransactions() catch |err| {
                std.log.err("Mempool processing error: {}", .{err});
            };
            std.time.sleep(1_000_000_000); // 1 second
        }
    }
    
    fn blockProducerLoop(self: *GhostDaemon) void {
        while (self.running) {
            self.block_producer.produceBlock() catch |err| {
                std.log.err("Block production error: {}", .{err});
            };
            std.time.sleep(self.config.block_time_ms * 1_000_000);
        }
    }
    
    fn peerDiscoveryLoop(self: *GhostDaemon) void {
        while (self.running) {
            self.peer_manager.discoverPeers() catch |err| {
                std.log.err("Peer discovery error: {}", .{err});
            };
            std.time.sleep(30_000_000_000); // 30 seconds
        }
    }
    
    fn blockSyncLoop(self: *GhostDaemon) void {
        while (self.running) {
            self.peer_manager.syncBlocks() catch |err| {
                std.log.err("Block sync error: {}", .{err});
            };
            std.time.sleep(5_000_000_000); // 5 seconds
        }
    }
    
    // Helper functions
    fn generateDomainLandingPage(self: *GhostDaemon, domain: []const u8, record: ghostchain.zns.DomainRecord) ![]u8 {
        const template = 
            \\<!DOCTYPE html>
            \\<html>
            \\<head>
            \\    <title>{s}.ghost - Web5 Domain</title>
            \\    <meta charset="utf-8">
            \\    <meta name="viewport" content="width=device-width, initial-scale=1">
            \\    <style>
            \\        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 0; padding: 20px; background: #0a0a0a; color: #fff; }}
            \\        .container {{ max-width: 800px; margin: 0 auto; }}
            \\        .header {{ text-align: center; margin-bottom: 40px; }}
            \\        .domain {{ font-size: 3em; font-weight: bold; color: #00ff88; }}
            \\        .record {{ background: #1a1a1a; padding: 20px; margin: 10px 0; border-radius: 8px; }}
            \\        .powered-by {{ text-align: center; margin-top: 40px; opacity: 0.6; }}
            \\    </style>
            \\</head>
            \\<body>
            \\    <div class="container">
            \\        <div class="header">
            \\            <div class="domain">{s}.ghost</div>
            \\            <p>Decentralized Web5 Domain on GhostChain</p>
            \\        </div>
            \\        <div class="record">
            \\            <h3>Domain Record</h3>
            \\            <p><strong>Type:</strong> {}</p>
            \\            <p><strong>Value:</strong> {s}</p>
            \\            <p><strong>TTL:</strong> {} seconds</p>
            \\        </div>
            \\        <div class="powered-by">
            \\            <p>Powered by GhostChain • QUIC • IPv6 • Web5</p>
            \\        </div>
            \\    </div>
            \\</body>
            \\</html>
        ;
        
        return std.fmt.allocPrint(self.allocator, template, .{ domain, domain, record.record_type, record.value, record.ttl });
    }
    
    fn processRPCMethod(self: *GhostDaemon, request: JSONRPCRequest) ![]u8 {
        // TODO: Implement full JSON-RPC 2.0 support
        const response = JSONRPCResponse{
            .jsonrpc = "2.0",
            .id = request.id,
            .result = "Method not implemented yet",
            .@"error" = null,
        };
        
        return std.json.stringifyAlloc(self.allocator, response, .{});
    }
    
    fn getUptime(self: *GhostDaemon) u64 {
        _ = self;
        // TODO: Track actual start time
        return 0;
    }
    
    fn updateMetrics(self: *GhostDaemon) void {
        _ = self;
        // TODO: Update Prometheus metrics
    }
};

pub const DaemonConfig = struct {
    bind_address: []const u8 = "::",
    p2p_port: u16 = 7777,
    rpc_address: []const u8 = "::",
    rpc_port: u16 = 8545,
    http3_port: u16 = 8443,
    minimum_stake: u64 = 1000000,
    gas_limit: u64 = 1000000,
    block_time_ms: u64 = 12000, // 12 seconds
    max_connections: u32 = 1000,
    tls_cert_path: ?[]const u8 = null,
    tls_key_path: ?[]const u8 = null,
    
    pub fn development() DaemonConfig {
        return DaemonConfig{
            .bind_address = "::1",
        };
    }
    
    pub fn production() DaemonConfig {
        return DaemonConfig{
            .minimum_stake = 10000000,
            .gas_limit = 10000000,
            .max_connections = 10000,
        };
    }
};

// Supporting structures
const TransactionMempool = struct {
    allocator: std.mem.Allocator,
    transactions: std.ArrayList(ghostchain.blockchain.Transaction),
    token_system: *ghostchain.tokens.TokenSystem,
    pending_transactions: std.HashMap([32]u8, ghostchain.blockchain.Transaction, HashContext, std.hash_map.default_max_load_percentage),
    account_nonces: std.HashMap([20]u8, u64, AddressHashContext, std.hash_map.default_max_load_percentage),
    max_mempool_size: usize,
    
    fn init(allocator: std.mem.Allocator, token_system: *ghostchain.tokens.TokenSystem) TransactionMempool {
        return TransactionMempool{
            .allocator = allocator,
            .transactions = std.ArrayList(ghostchain.blockchain.Transaction).init(allocator),
            .token_system = token_system,
            .pending_transactions = std.HashMap([32]u8, ghostchain.blockchain.Transaction, HashContext, std.hash_map.default_max_load_percentage).init(allocator),
            .account_nonces = std.HashMap([20]u8, u64, AddressHashContext, std.hash_map.default_max_load_percentage).init(allocator),
            .max_mempool_size = 10000, // Max 10k pending transactions
        };
    }
    
    fn deinit(self: *TransactionMempool) void {
        self.transactions.deinit();
        self.pending_transactions.deinit();
        self.account_nonces.deinit();
    }
    
    // Add transaction to mempool with full validation
    fn addTransaction(self: *TransactionMempool, tx: ghostchain.blockchain.Transaction) !void {
        // Check mempool size limit
        if (self.pending_transactions.count() >= self.max_mempool_size) {
            return error.MempoolFull;
        }
        
        // Validate transaction structure
        try self.validateTransaction(tx);
        
        // Check if transaction already exists
        const tx_hash = tx.hash();
        if (self.pending_transactions.contains(tx_hash)) {
            return error.DuplicateTransaction;
        }
        
        // Validate nonce
        try self.validateNonce(tx.from, tx.nonce);
        
        // Validate signature
        try self.validateSignature(tx);
        
        // Validate account has sufficient balance for gas and amount
        try self.validateBalance(tx);
        
        // Add to pending transactions
        try self.pending_transactions.put(tx_hash, tx);
        try self.transactions.append(tx);
        
        // Update account nonce
        try self.account_nonces.put(tx.from, tx.nonce + 1);
        
        std.log.info("📥 Added transaction to mempool: {} {any} from {any} to {any}", .{ tx.amount, tx.token, tx.from, tx.to });
    }
    
    // Validate transaction structure and content
    fn validateTransaction(self: *TransactionMempool, tx: ghostchain.blockchain.Transaction) !void {
        _ = self;
        
        // Check for zero amounts (gas fee must be > 0)
        if (tx.gas_fee == 0) {
            return error.ZeroGasFee;
        }
        
        // Check for self-transfer
        if (std.mem.eql(u8, &tx.from, &tx.to)) {
            return error.SelfTransfer;
        }
        
        // Validate token type
        switch (tx.token) {
            .GCC, .SPIRIT, .MANA, .GHOST => {}, // Valid tokens
            // Add validation for any future token types
        }
    }
    
    // Validate account nonce
    fn validateNonce(self: *TransactionMempool, from: [20]u8, nonce: u64) !void {
        const expected_nonce = self.account_nonces.get(from) orelse 0;
        if (nonce != expected_nonce) {
            std.log.warn("Invalid nonce: expected {}, got {}", .{ expected_nonce, nonce });
            return error.InvalidNonce;
        }
    }
    
    // Validate transaction signature
    fn validateSignature(self: *TransactionMempool, tx: ghostchain.blockchain.Transaction) !void {
        _ = self;
        _ = tx;
        // TODO: Implement proper signature validation using ghostcipher
        // For now, accept all signatures (insecure)
        std.log.debug("🔐 Signature validation (stub): OK");
    }
    
    // Validate account has sufficient balance
    fn validateBalance(self: *TransactionMempool, tx: ghostchain.blockchain.Transaction) !void {
        // Check GCC balance for gas fee
        const gcc_balance = try self.token_system.getBalance(tx.from, .GCC);
        if (gcc_balance < tx.gas_fee) {
            return error.InsufficientGasBalance;
        }
        
        // Check token balance for transfer amount
        if (tx.token != .GCC) {
            const token_balance = try self.token_system.getBalance(tx.from, tx.token);
            if (token_balance < tx.amount) {
                return error.InsufficientTokenBalance;
            }
        } else {
            // For GCC transfers, check total (amount + gas_fee)
            if (gcc_balance < tx.amount + tx.gas_fee) {
                return error.InsufficientGasBalance;
            }
        }
    }
    
    // Process all valid pending transactions
    fn processPendingTransactions(self: *TransactionMempool) !void {
        std.log.info("🔄 Processing {} pending transactions", .{self.transactions.items.len});
        
        // Process all pending transactions
        var i: usize = 0;
        var processed_count: usize = 0;
        while (i < self.transactions.items.len) {
            const tx = self.transactions.items[i];
            
            // Execute gas fee payment
            self.token_system.payGasFee(tx.from, tx.gas_fee) catch |err| {
                std.log.warn("Transaction gas fee payment failed: {}", .{err});
                self.removeTransaction(i);
                continue;
            };
            
            // Execute token transfer (if not just a gas fee payment)
            if (tx.amount > 0) {
                self.token_system.transfer(tx.from, tx.to, tx.token, tx.amount) catch |err| {
                    std.log.warn("Transaction transfer failed: {}", .{err});
                    self.removeTransaction(i);
                    continue;
                };
            }
            
            std.log.info("✅ Processed transaction: {} {any} from {any} to {any}", .{ tx.amount, tx.token, tx.from, tx.to });
            processed_count += 1;
            i += 1;
        }
        
        // Clear all processed transactions
        self.transactions.clearRetainingCapacity();
        self.pending_transactions.clearRetainingCapacity();
        
        std.log.info("🎯 Successfully processed {} transactions", .{processed_count});
    }
    
    // Remove transaction from mempool
    fn removeTransaction(self: *TransactionMempool, index: usize) void {
        if (index >= self.transactions.items.len) return;
        
        const tx = self.transactions.items[index];
        const tx_hash = tx.hash();
        
        // Remove from pending transactions map
        _ = self.pending_transactions.remove(tx_hash);
        
        // Remove from transaction list
        _ = self.transactions.orderedRemove(index);
    }
    
    // Get transactions ready for block inclusion
    fn getTransactionsForBlock(self: *TransactionMempool, max_count: usize) ![]ghostchain.blockchain.Transaction {
        const count = @min(max_count, self.transactions.items.len);
        const selected = try self.allocator.alloc(ghostchain.blockchain.Transaction, count);
        
        // For now, take transactions in FIFO order
        // TODO: Implement priority queue based on gas fees
        for (0..count) |i| {
            selected[i] = self.transactions.items[i];
        }
        
        return selected;
    }
    
    fn getTransactionCount(self: *TransactionMempool) usize {
        return self.transactions.items.len;
    }
    
    fn getPendingTransactionCount(self: *TransactionMempool) usize {
        return self.pending_transactions.count();
    }
    
    fn getMempoolStats(self: *TransactionMempool) MempoolStats {
        return MempoolStats{
            .pending_count = self.transactions.items.len,
            .total_capacity = self.max_mempool_size,
            .utilization_percent = @as(u8, @intCast((self.transactions.items.len * 100) / self.max_mempool_size)),
        };
    }
};

const PeerManager = struct {
    allocator: std.mem.Allocator,
    
    fn init(allocator: std.mem.Allocator) PeerManager {
        return PeerManager{
            .allocator = allocator,
        };
    }
    
    fn deinit(self: *PeerManager) void {
        _ = self;
    }
    
    fn discoverPeers(self: *PeerManager) !void {
        _ = self;
        // TODO: Implement peer discovery
    }
    
    fn syncBlocks(self: *PeerManager) !void {
        _ = self;
        // TODO: Implement block synchronization
    }
    
    fn getConnectedPeerCount(self: *PeerManager) u32 {
        _ = self;
        return 0; // TODO: Return actual peer count
    }
};

const BlockProducer = struct {
    allocator: std.mem.Allocator,
    
    fn init(allocator: std.mem.Allocator) BlockProducer {
        return BlockProducer{
            .allocator = allocator,
        };
    }
    
    fn deinit(self: *BlockProducer) void {
        _ = self;
    }
    
    fn produceBlock(self: *BlockProducer) !void {
        _ = self;
        // TODO: Implement block production
    }
};

const JSONRPCRequest = struct {
    jsonrpc: []const u8,
    method: []const u8,
    params: ?std.json.Value,
    id: ?std.json.Value,
};

const JSONRPCResponse = struct {
    jsonrpc: []const u8,
    result: ?[]const u8,
    @"error": ?[]const u8,
    id: ?std.json.Value,
};

const HealthStatus = struct {
    status: []const u8,
    uptime: u64,
    peers_connected: u32,
    latest_block: u64,
    mempool_size: usize,
};

const ContractDeployRequest = struct {
    bytecode: []const u8,
    constructor_args: []const u8,
    deployer: []const u8,
    gas_fee: u64,
};

const ContractCallRequest = struct {
    contract_address: []const u8,
    function_selector: []const u8,
    args: []const u8,
    caller: []const u8,
    value: u64,
    gas_fee: u64,
};

// Hash context for transaction hashes
const HashContext = struct {
    pub fn hash(self: @This(), key: [32]u8) u64 {
        _ = self;
        return std.hash_map.hashString(&key);
    }
    
    pub fn eql(self: @This(), a: [32]u8, b: [32]u8) bool {
        _ = self;
        return std.mem.eql(u8, &a, &b);
    }
};

// Hash context for addresses
const AddressHashContext = struct {
    pub fn hash(self: @This(), key: [20]u8) u64 {
        _ = self;
        return std.hash_map.hashString(&key);
    }
    
    pub fn eql(self: @This(), a: [20]u8, b: [20]u8) bool {
        _ = self;
        return std.mem.eql(u8, &a, &b);
    }
};

// Mempool statistics
const MempoolStats = struct {
    pending_count: usize,
    total_capacity: usize,
    utilization_percent: u8,
};