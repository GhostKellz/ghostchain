//! GhostChain - Pure Zig Blockchain Implementation
//! 
//! This is the root module for the GhostChain ecosystem, providing a complete
//! blockchain implementation with integrated cryptography, networking, smart contracts,
//! wallet management, and decentralized identity systems.

const std = @import("std");

// Import external dependencies
const zledger = @import("zledger");
const zsig = @import("zsig");
const zquic = @import("zquic");
const ghostnet = @import("ghostnet");
const zsync = @import("zsync");
const zcrypto = @import("zcrypto");
const zwallet = @import("zwallet");
const keystone = @import("keystone");
const zvm = @import("zvm");
const zns = @import("zns");
const cns = @import("cns");
const wraith = @import("wraith");
const shroud = @import("shroud");

// Re-export core modules
pub const blockchain = @import("blockchain/mod.zig");
pub const network = @import("network/mod.zig");
pub const crypto = @import("crypto/mod.zig");
pub const wallet = @import("wallet/mod.zig");
pub const ledger = @import("ledger/mod.zig");
pub const consensus = @import("consensus/mod.zig");
pub const contracts = @import("contracts/mod.zig");
pub const zns = @import("zns/mod.zig");
pub const rpc = @import("rpc/mod.zig");
pub const realid = @import("realid/mod.zig");
pub const daemon = @import("daemon/mod.zig");
pub const tokens = @import("tokens/mod.zig");

// Core GhostChain functionality
pub const GhostChain = struct {
    allocator: std.mem.Allocator,
    blockchain: blockchain.Block,
    storage_engine: blockchain.StorageEngine,
    network_node: network.Node,
    p2p_manager: network.P2PManager,
    wallet_manager: wallet.GhostWallet,
    ledger_system: ledger.GhostLedger,
    consensus_engine: consensus.ProofOfStake,
    contract_executor: contracts.SmartContractExecutor,
    name_service: zns.ZNSResolver,
    rpc_server: rpc.GhostRPCServer,
    identity_system: realid.RealIDSystem,
    token_system: tokens.TokenSystem,
    
    pub fn init(allocator: std.mem.Allocator, config: GhostChainConfig) !GhostChain {
        // Initialize core components
        const ledger_system = ledger.GhostLedger.init(allocator, config.ledger_path);
        const wallet_manager = try wallet.GhostWallet.init(allocator);
        const consensus_engine = consensus.ProofOfStake.init(allocator, config.minimum_stake);
        const name_service = zns.ZNSResolver.init(allocator);
        const identity_system = realid.RealIDSystem.init(allocator);
        const network_node = network.Node.init(allocator, config.bind_address, config.port);
        
        // Initialize P2P networking with QUIC + IPv6
        const p2p_config = network.P2PConfig{
            .bind_address = config.bind_address,
            .port = config.p2p_port,
            .max_peers = config.max_peers,
            .enable_ipv6 = true,
            .enable_multicast = true,
        };
        const p2p_manager = try network.P2PManager.init(allocator, p2p_config);
        
        // Initialize 4-token system (GCC + SPIRIT + MANA + GHOST)
        const token_system = tokens.TokenSystem.init(allocator);
        
        // Initialize blockchain storage engine
        const storage_engine = try blockchain.StorageEngine.init(allocator, config.data_dir);
        
        // Initialize contract executor with ledger integration
        var ledger_system_mut = ledger_system;
        const contract_executor = try contracts.SmartContractExecutor.init(
            allocator,
            &ledger_system_mut,
            config.gas_limit,
        );
        
        // Initialize RPC server with all components
        // TEMP: Disable RPC server until blockchain state management is fixed
        const rpc_server = rpc.GhostRPCServer{
            .allocator = allocator,
            .address = config.rpc_address,
            .port = config.rpc_port,
            .blockchain = undefined,
            .wallet = undefined,
            .zns = undefined,
        };
        
        // Create genesis block
        const genesis_transactions = [_]blockchain.Transaction{};
        const genesis_block = try blockchain.Block.init(
            allocator,
            0, // Genesis block index
            std.mem.zeroes([32]u8), // No previous hash
            &genesis_transactions,
        );
        
        var ghost_chain = GhostChain{
            .allocator = allocator,
            .blockchain = genesis_block,
            .storage_engine = storage_engine,
            .network_node = network_node,
            .p2p_manager = p2p_manager,
            .wallet_manager = wallet_manager,
            .ledger_system = ledger_system,
            .consensus_engine = consensus_engine,
            .contract_executor = contract_executor,
            .name_service = name_service,
            .rpc_server = rpc_server,
            .identity_system = identity_system,
            .token_system = token_system,
        };
        
        // Initialize genesis token distribution
        try tokens.createGenesisDistribution(&ghost_chain.token_system);
        
        return ghost_chain;
    }
    
    pub fn deinit(self: *GhostChain) void {
        self.network_node.deinit();
        self.p2p_manager.deinit();
        self.storage_engine.deinit();
        self.ledger_system.deinit();
        self.consensus_engine.deinit();
        self.contract_executor.deinit();
        self.name_service.deinit();
        self.identity_system.deinit();
        self.token_system.deinit();
    }
    
    pub fn start(self: *GhostChain) !void {
        std.log.info("🚀 Starting GhostChain...", .{});
        
        // Start P2P networking
        try self.p2p_manager.start();
        
        // Start network node
        try self.network_node.start();
        
        // Start RPC server
        try self.rpc_server.start();
        
        std.log.info("✅ GhostChain is running with QUIC + IPv6 networking!", .{});
    }
    
    pub fn createAccount(self: *GhostChain, name: []const u8) ![]const u8 {
        return self.wallet_manager.createGhostAccount(name);
    }
    
    pub fn getBalance(self: *GhostChain, account: []const u8, token: tokens.TokenType) !u64 {
        var account_array: [20]u8 = undefined;
        @memcpy(account_array[0..@min(20, account.len)], account[0..@min(20, account.len)]);
        return self.token_system.getBalance(account_array, token);
    }
    
    pub fn getGhostBalance(self: *GhostChain, account: []const u8) !u64 {
        return self.getBalance(account, .GHOST);
    }
    
    pub fn sendTransaction(
        self: *GhostChain,
        from: []const u8,
        to: []const u8,
        token: tokens.TokenType,
        amount: u64,
        gas_fee: u64,
        memo: ?[]const u8,
    ) !void {
        // Convert addresses to fixed arrays
        var from_array: [20]u8 = undefined;
        var to_array: [20]u8 = undefined;
        @memcpy(from_array[0..@min(20, from.len)], from[0..@min(20, from.len)]);
        @memcpy(to_array[0..@min(20, to.len)], to[0..@min(20, to.len)]);
        
        // Validate gas fee payment in GCC
        try self.token_system.payGasFee(from_array, gas_fee);
        
        // Process token transfer
        try self.token_system.transfer(from_array, to_array, token, amount);
        
        // Process transaction through ledger (for audit trail)
        _ = memo; // TODO: Use memo in transaction processing
        // try self.ledger_system.processGhostTransaction(from, to, amount, memo); // Temporarily disabled
        
        // Create blockchain transaction
        
        const tx = tokens.TokenTransaction{
            .from = from_array,
            .to = to_array,
            .token = token,
            .amount = amount,
            .gas_fee = gas_fee,
            .nonce = 0, // TODO: Get from account
            .signature = std.mem.zeroes([64]u8), // TODO: Sign transaction
        };
        
        // TODO: Add transaction to mempool and create new block
        _ = tx;
    }
    
    pub fn deployContract(
        self: *GhostChain,
        bytecode: []const u8,
        constructor_args: []const u8,
        deployer: []const u8,
    ) !contracts.ContractDeploymentResult {
        return self.contract_executor.deployContract(bytecode, constructor_args, deployer);
    }
    
    pub fn callContract(
        self: *GhostChain,
        contract_address: []const u8,
        function_selector: []const u8,
        args: []const u8,
        caller: []const u8,
        value: u64,
    ) !contracts.ContractCallResult {
        return self.contract_executor.callContract(
            contract_address,
            function_selector,
            args,
            caller,
            value,
        );
    }
    
    pub fn resolveDomain(self: *GhostChain, domain: []const u8) !zns.DomainRecord {
        return self.name_service.resolveDomain(domain, &[_][]const u8{"A"});
    }
    
    pub fn registerDomain(self: *GhostChain, domain: []const u8, owner: []const u8) !void {
        return self.name_service.registerDomain(domain, owner, &[_]zns.DomainRecord{});
    }
    
    pub fn createIdentity(self: *GhostChain, username: []const u8, public_key: []const u8) !realid.Identity {
        return self.identity_system.createIdentity(username, public_key);
    }
    
    pub fn verifyIdentity(self: *GhostChain, did: []const u8, proof: realid.IdentityProof) !bool {
        return self.identity_system.verifyIdentity(did, proof);
    }
    
    // Blockchain persistence methods
    pub fn storeBlock(self: *GhostChain, block: blockchain.Block) !void {
        try self.storage_engine.storeBlock(block);
        std.log.info("🔒 Block #{} permanently stored", .{block.index});
    }
    
    pub fn getBlock(self: *GhostChain, height: u64) !?blockchain.Block {
        return self.storage_engine.getBlock(height);
    }
    
    pub fn getBlockByHash(self: *GhostChain, hash: [32]u8) !?blockchain.Block {
        return self.storage_engine.getBlockByHash(hash);
    }
    
    pub fn getLatestBlockHeight(self: *GhostChain) !u64 {
        return self.storage_engine.getLatestHeight();
    }
    
    pub fn verifyChainIntegrity(self: *GhostChain) !bool {
        return self.storage_engine.verifyChainIntegrity();
    }
    
    pub fn persistTokenBalance(self: *GhostChain, account: [20]u8, token: tokens.TokenType, balance: u64) !void {
        const key = try std.fmt.allocPrint(self.allocator, "balance_{any}_{any}", .{ account, token });
        defer self.allocator.free(key);
        
        const value = try std.fmt.allocPrint(self.allocator, "{}", .{balance});
        defer self.allocator.free(value);
        
        try self.storage_engine.storeState(key, value);
    }
    
    pub fn loadTokenBalance(self: *GhostChain, account: [20]u8, token: tokens.TokenType) !?u64 {
        const key = try std.fmt.allocPrint(self.allocator, "balance_{any}_{any}", .{ account, token });
        defer self.allocator.free(key);
        
        const value = try self.storage_engine.getState(key) orelse return null;
        defer self.allocator.free(value);
        
        return std.fmt.parseInt(u64, value, 10);
    }
    
    // P2P networking methods
    pub fn broadcastBlock(self: *GhostChain, block: blockchain.Block) !void {
        try self.p2p_manager.broadcastBlock(block);
    }
    
    pub fn broadcastTransaction(self: *GhostChain, tx: tokens.TokenTransaction) !void {
        try self.p2p_manager.broadcastTransaction(tx);
    }
    
    pub fn connectToPeer(self: *GhostChain, address: []const u8, port: u16) !void {
        _ = try self.p2p_manager.connectToPeer(address, port);
        std.log.info("🤝 Connected to peer: {s}:{}", .{ address, port });
    }
    
    pub fn getConnectedPeerCount(self: *GhostChain) u32 {
        return self.p2p_manager.getConnectedPeerCount();
    }
    
    pub fn syncWithPeers(self: *GhostChain) !void {
        const latest_height = try self.getLatestBlockHeight();
        try self.p2p_manager.requestBlocks(latest_height + 1, latest_height + 100);
        std.log.info("🔄 Syncing with peers from block #{}", .{latest_height + 1});
    }
};

pub const GhostChainConfig = struct {
    bind_address: []const u8 = "::1", // IPv6 localhost
    port: u16 = 7777,
    p2p_port: u16 = 7778, // QUIC P2P port
    rpc_address: []const u8 = "::1",
    rpc_port: u16 = 8545,
    ledger_path: []const u8 = "ghostchain.ledger",
    data_dir: []const u8 = "ghostchain_data", // Blockchain storage directory
    minimum_stake: u64 = 1000000, // 1M SPIRIT minimum stake
    gas_limit: u64 = 1000000,
    max_peers: u32 = 50, // Maximum P2P connections
    
    pub fn development() GhostChainConfig {
        return GhostChainConfig{};
    }
    
    pub fn production() GhostChainConfig {
        return GhostChainConfig{
            .bind_address = "::",
            .port = 7777,
            .rpc_address = "::",
            .rpc_port = 8545,
            .minimum_stake = 10000000, // 10 GHOST minimum stake for production
            .gas_limit = 10000000,
        };
    }
};

// Utility function for basic buffered output
pub fn bufferedPrint() !void {
    const stdout_file = std.io.getStdOut().writer();
    var bw = std.io.bufferedWriter(stdout_file);
    const stdout = bw.writer();

    try stdout.print("🔗 GhostChain v0.1.0 - Pure Zig Blockchain\n", .{});
    try stdout.print("🚀 Ready for integration with ZCRYPTO, ZLEDGER, ZWALLET, ZQUIC, ZSIG, and ZVM\n", .{});

    try bw.flush();
}

// Basic tests
test "ghostchain basic functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const config = GhostChainConfig.development();
    var ghostchain = try GhostChain.init(allocator, config);
    defer ghostchain.deinit();
    
    // Test basic initialization
    try std.testing.expect(ghostchain.consensus_engine.minimum_stake == 1000000);
}

test "basic blockchain operations" {
    const tx = blockchain.Transaction.init("alice", "bob", 1000, 100);
    try std.testing.expect(tx.amount == 1000);
    try std.testing.expect(tx.fee == 100);
}
