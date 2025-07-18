// 🗄️ GhostChain Blockchain Persistence Layer
// Handles block storage, chain state persistence, and data recovery

const std = @import("std");
const shroud = @import("shroud");
const keystone = shroud.keystone;
const Block = @import("block.zig").Block;
const Transaction = @import("block.zig").Transaction;

pub const StorageEngine = struct {
    allocator: std.mem.Allocator,
    data_dir: []const u8,
    block_db: BlockDatabase,
    state_db: StateDatabase,
    tx_index: TransactionIndex,
    
    pub fn init(allocator: std.mem.Allocator, data_dir: []const u8) !StorageEngine {
        // Ensure data directory exists
        std.fs.cwd().makeDir(data_dir) catch |err| switch (err) {
            error.PathAlreadyExists => {},
            else => return err,
        };
        
        return StorageEngine{
            .allocator = allocator,
            .data_dir = try allocator.dupe(u8, data_dir),
            .block_db = try BlockDatabase.init(allocator, data_dir),
            .state_db = try StateDatabase.init(allocator, data_dir),
            .tx_index = try TransactionIndex.init(allocator, data_dir),
        };
    }
    
    pub fn deinit(self: *StorageEngine) void {
        self.block_db.deinit();
        self.state_db.deinit();
        self.tx_index.deinit();
        self.allocator.free(self.data_dir);
    }
    
    // Store a block permanently
    pub fn storeBlock(self: *StorageEngine, block: Block) !void {
        try self.block_db.store(block);
        
        // Index all transactions in the block
        for (block.transactions) |tx| {
            try self.tx_index.addTransaction(tx, block.index);
        }
        
        std.log.info("📦 Stored block #{} with {} transactions", .{ block.index, block.transactions.len });
    }
    
    // Retrieve a block by height
    pub fn getBlock(self: *StorageEngine, height: u64) !?Block {
        return self.block_db.get(height);
    }
    
    // Retrieve a block by hash
    pub fn getBlockByHash(self: *StorageEngine, hash: [32]u8) !?Block {
        return self.block_db.getByHash(hash);
    }
    
    // Get the latest block height
    pub fn getLatestHeight(self: *StorageEngine) !u64 {
        return self.block_db.getLatestHeight();
    }
    
    // Store blockchain state (token balances, account data)
    pub fn storeState(self: *StorageEngine, key: []const u8, value: []const u8) !void {
        try self.state_db.put(key, value);
    }
    
    // Retrieve blockchain state
    pub fn getState(self: *StorageEngine, key: []const u8) !?[]u8 {
        return self.state_db.get(key);
    }
    
    // Find transaction by hash
    pub fn findTransaction(self: *StorageEngine, tx_hash: [32]u8) !?TransactionLocation {
        return self.tx_index.find(tx_hash);
    }
    
    // Get blocks in range for syncing
    pub fn getBlockRange(self: *StorageEngine, start: u64, end: u64) ![]Block {
        return self.block_db.getRange(start, end);
    }
    
    // Verify blockchain integrity
    pub fn verifyChainIntegrity(self: *StorageEngine) !bool {
        const latest_height = try self.getLatestHeight();
        if (latest_height == 0) return true; // Empty chain is valid
        
        // Verify each block links to the previous one
        for (1..latest_height + 1) |height| {
            const current_block = try self.getBlock(height) orelse return false;
            const prev_block = try self.getBlock(height - 1) orelse return false;
            
            if (!std.mem.eql(u8, &current_block.previous_hash, &prev_block.hash)) {
                std.log.err("❌ Chain integrity violation at block #{}", .{height});
                return false;
            }
        }
        
        std.log.info("✅ Blockchain integrity verified up to block #{}", .{latest_height});
        return true;
    }
};

// Block storage database
const BlockDatabase = struct {
    allocator: std.mem.Allocator,
    blocks_dir: []const u8,
    index_file: std.fs.File,
    block_index: std.HashMap(u64, BlockMetadata, std.hash_map.AutoContext(u64), std.hash_map.default_max_load_percentage),
    hash_to_height: std.HashMap([32]u8, u64, HashContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator, data_dir: []const u8) !BlockDatabase {
        const blocks_dir = try std.fs.path.join(allocator, &[_][]const u8{ data_dir, "blocks" });
        
        // Create blocks directory
        std.fs.cwd().makeDir(blocks_dir) catch |err| switch (err) {
            error.PathAlreadyExists => {},
            else => return err,
        };
        
        // Open or create index file
        const index_path = try std.fs.path.join(allocator, &[_][]const u8{ blocks_dir, "index.dat" });
        defer allocator.free(index_path);
        
        const index_file = std.fs.cwd().createFile(index_path, .{ .read = true, .truncate = false }) catch |err| switch (err) {
            error.FileNotFound => try std.fs.cwd().createFile(index_path, .{ .read = true }),
            else => return err,
        };
        
        var db = BlockDatabase{
            .allocator = allocator,
            .blocks_dir = blocks_dir,
            .index_file = index_file,
            .block_index = std.HashMap(u64, BlockMetadata, std.hash_map.AutoContext(u64), std.hash_map.default_max_load_percentage).init(allocator),
            .hash_to_height = std.HashMap([32]u8, u64, HashContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
        
        // Load existing index
        try db.loadIndex();
        
        return db;
    }
    
    pub fn deinit(self: *BlockDatabase) void {
        self.index_file.close();
        self.block_index.deinit();
        self.hash_to_height.deinit();
        self.allocator.free(self.blocks_dir);
    }
    
    pub fn store(self: *BlockDatabase, block: Block) !void {
        // Create block file
        const block_filename = try std.fmt.allocPrint(self.allocator, "block_{d:0>10}.dat", .{block.index});
        defer self.allocator.free(block_filename);
        
        const block_path = try std.fs.path.join(self.allocator, &[_][]const u8{ self.blocks_dir, block_filename });
        defer self.allocator.free(block_path);
        
        const block_file = try std.fs.cwd().createFile(block_path, .{});
        defer block_file.close();
        
        // Serialize and write block
        try self.writeBlock(block_file, block);
        
        // Update index
        const metadata = BlockMetadata{
            .height = block.index,
            .hash = block.hash,
            .timestamp = block.timestamp,
            .tx_count = block.transactions.len,
            .file_offset = 0, // Simple single-block files for now
        };
        
        try self.block_index.put(block.index, metadata);
        try self.hash_to_height.put(block.hash, block.index);
        
        // Persist index
        try self.saveIndex();
    }
    
    pub fn get(self: *BlockDatabase, height: u64) !?Block {
        const metadata = self.block_index.get(height) orelse return null;
        
        const block_filename = try std.fmt.allocPrint(self.allocator, "block_{d:0>10}.dat", .{height});
        defer self.allocator.free(block_filename);
        
        const block_path = try std.fs.path.join(self.allocator, &[_][]const u8{ self.blocks_dir, block_filename });
        defer self.allocator.free(block_path);
        
        const block_file = std.fs.cwd().openFile(block_path, .{}) catch |err| switch (err) {
            error.FileNotFound => return null,
            else => return err,
        };
        defer block_file.close();
        
        return try self.readBlock(block_file, metadata);
    }
    
    pub fn getByHash(self: *BlockDatabase, hash: [32]u8) !?Block {
        const height = self.hash_to_height.get(hash) orelse return null;
        return self.get(height);
    }
    
    pub fn getLatestHeight(self: *BlockDatabase) !u64 {
        var max_height: u64 = 0;
        var iterator = self.block_index.iterator();
        while (iterator.next()) |entry| {
            if (entry.key_ptr.* > max_height) {
                max_height = entry.key_ptr.*;
            }
        }
        return max_height;
    }
    
    pub fn getRange(self: *BlockDatabase, start: u64, end: u64) ![]Block {
        const count = end - start + 1;
        const blocks = try self.allocator.alloc(Block, count);
        
        for (start..end + 1, 0..) |height, i| {
            blocks[i] = try self.get(height) orelse return error.BlockNotFound;
        }
        
        return blocks;
    }
    
    fn writeBlock(self: *BlockDatabase, file: std.fs.File, block: Block) !void {
        _ = self;
        var writer = file.writer();
        
        // Write block header
        try writer.writeInt(u64, block.index, .little);
        try writer.writeInt(u64, block.timestamp, .little);
        try writer.writeAll(&block.previous_hash);
        try writer.writeAll(&block.merkle_root);
        try writer.writeInt(u64, block.nonce, .little);
        try writer.writeAll(&block.hash);
        
        // Write transaction count
        try writer.writeInt(u32, @intCast(block.transactions.len), .little);
        
        // Write transactions
        for (block.transactions) |tx| {
            try writer.writeAll(&tx.from);
            try writer.writeAll(&tx.to);
            try writer.writeInt(u8, @intFromEnum(tx.token), .little);
            try writer.writeInt(u64, tx.amount, .little);
            try writer.writeInt(u64, tx.gas_fee, .little);
            try writer.writeInt(u64, tx.nonce, .little);
            try writer.writeAll(&tx.signature);
        }
    }
    
    fn readBlock(self: *BlockDatabase, file: std.fs.File, metadata: BlockMetadata) !Block {
        _ = metadata;
        var reader = file.reader();
        
        // Read block header
        const index = try reader.readInt(u64, .little);
        const timestamp = try reader.readInt(u64, .little);
        
        var previous_hash: [32]u8 = undefined;
        _ = try reader.readAll(&previous_hash);
        
        var merkle_root: [32]u8 = undefined;
        _ = try reader.readAll(&merkle_root);
        
        const nonce = try reader.readInt(u64, .little);
        
        var hash: [32]u8 = undefined;
        _ = try reader.readAll(&hash);
        
        // Read transactions
        const tx_count = try reader.readInt(u32, .little);
        const transactions = try self.allocator.alloc(Transaction, tx_count);
        
        for (transactions) |*tx| {
            _ = try reader.readAll(&tx.from);
            _ = try reader.readAll(&tx.to);
            
            const token_value = try reader.readInt(u8, .little);
            tx.token = @enumFromInt(token_value);
            
            tx.amount = try reader.readInt(u64, .little);
            tx.gas_fee = try reader.readInt(u64, .little);
            tx.nonce = try reader.readInt(u64, .little);
            
            _ = try reader.readAll(&tx.signature);
        }
        
        return Block{
            .index = index,
            .timestamp = timestamp,
            .previous_hash = previous_hash,
            .merkle_root = merkle_root,
            .nonce = nonce,
            .transactions = transactions,
            .hash = hash,
        };
    }
    
    fn loadIndex(self: *BlockDatabase) !void {
        _ = self;
        // TODO: Load index from persistent storage
        // For now, scan directory and rebuild index
    }
    
    fn saveIndex(self: *BlockDatabase) !void {
        _ = self;
        // TODO: Save index to persistent storage
        // For now, just log the operation
        std.log.debug("💾 Block index saved");
    }
};

// State database for account data and chain state
const StateDatabase = struct {
    allocator: std.mem.Allocator,
    state_dir: []const u8,
    kv_store: std.HashMap([]const u8, []const u8, StringContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator, data_dir: []const u8) !StateDatabase {
        const state_dir = try std.fs.path.join(allocator, &[_][]const u8{ data_dir, "state" });
        
        // Create state directory
        std.fs.cwd().makeDir(state_dir) catch |err| switch (err) {
            error.PathAlreadyExists => {},
            else => return err,
        };
        
        return StateDatabase{
            .allocator = allocator,
            .state_dir = state_dir,
            .kv_store = std.HashMap([]const u8, []const u8, StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    pub fn deinit(self: *StateDatabase) void {
        // Free all keys and values
        var iterator = self.kv_store.iterator();
        while (iterator.next()) |entry| {
            self.allocator.free(entry.key_ptr.*);
            self.allocator.free(entry.value_ptr.*);
        }
        self.kv_store.deinit();
        self.allocator.free(self.state_dir);
    }
    
    pub fn put(self: *StateDatabase, key: []const u8, value: []const u8) !void {
        const owned_key = try self.allocator.dupe(u8, key);
        const owned_value = try self.allocator.dupe(u8, value);
        
        // Free existing value if key already exists
        if (self.kv_store.get(key)) |existing_value| {
            self.allocator.free(existing_value);
        }
        
        try self.kv_store.put(owned_key, owned_value);
    }
    
    pub fn get(self: *StateDatabase, key: []const u8) !?[]u8 {
        const value = self.kv_store.get(key) orelse return null;
        return try self.allocator.dupe(u8, value);
    }
};

// Transaction index for fast lookups
const TransactionIndex = struct {
    allocator: std.mem.Allocator,
    index_dir: []const u8,
    tx_locations: std.HashMap([32]u8, TransactionLocation, HashContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator, data_dir: []const u8) !TransactionIndex {
        const index_dir = try std.fs.path.join(allocator, &[_][]const u8{ data_dir, "tx_index" });
        
        // Create index directory
        std.fs.cwd().makeDir(index_dir) catch |err| switch (err) {
            error.PathAlreadyExists => {},
            else => return err,
        };
        
        return TransactionIndex{
            .allocator = allocator,
            .index_dir = index_dir,
            .tx_locations = std.HashMap([32]u8, TransactionLocation, HashContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    pub fn deinit(self: *TransactionIndex) void {
        self.tx_locations.deinit();
        self.allocator.free(self.index_dir);
    }
    
    pub fn addTransaction(self: *TransactionIndex, tx: Transaction, block_height: u64) !void {
        const tx_hash = tx.hash();
        const location = TransactionLocation{
            .block_height = block_height,
            .tx_index = 0, // TODO: Calculate actual index within block
        };
        
        try self.tx_locations.put(tx_hash, location);
    }
    
    pub fn find(self: *TransactionIndex, tx_hash: [32]u8) !?TransactionLocation {
        return self.tx_locations.get(tx_hash);
    }
};

// Supporting structures
const BlockMetadata = struct {
    height: u64,
    hash: [32]u8,
    timestamp: u64,
    tx_count: usize,
    file_offset: u64,
};

const TransactionLocation = struct {
    block_height: u64,
    tx_index: u32,
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

// String context for state database
const StringContext = struct {
    pub fn hash(self: @This(), key: []const u8) u64 {
        _ = self;
        return std.hash_map.hashString(key);
    }
    
    pub fn eql(self: @This(), a: []const u8, b: []const u8) bool {
        _ = self;
        return std.mem.eql(u8, a, b);
    }
};