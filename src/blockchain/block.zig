const std = @import("std");
const shroud = @import("shroud");
const ghostcipher = shroud.ghostcipher;
const tokens = @import("../tokens/mod.zig");

pub const Block = struct {
    index: u64,
    timestamp: u64,
    previous_hash: [32]u8,
    merkle_root: [32]u8,
    nonce: u64,
    transactions: []Transaction,
    hash: [32]u8,

    pub fn init(_: std.mem.Allocator, index: u64, previous_hash: [32]u8, transactions: []Transaction) !Block {
        const timestamp = @as(u64, @intCast(std.time.milliTimestamp()));
        const merkle_root = calculateMerkleRoot(transactions);
        
        var block = Block{
            .index = index,
            .timestamp = timestamp,
            .previous_hash = previous_hash,
            .merkle_root = merkle_root,
            .nonce = 0,
            .transactions = transactions,
            .hash = undefined,
        };
        
        block.hash = block.calculateHash();
        return block;
    }

    pub fn calculateHash(self: *const Block) [32]u8 {
        var hasher = std.crypto.hash.sha2.Sha256.init(.{});
        hasher.update(std.mem.asBytes(&self.index));
        hasher.update(std.mem.asBytes(&self.timestamp));
        hasher.update(&self.previous_hash);
        hasher.update(&self.merkle_root);
        hasher.update(std.mem.asBytes(&self.nonce));
        return hasher.finalResult();
    }

    pub fn mine(self: *Block, difficulty: u32) void {
        const target_prefix = [_]u8{0} ** (difficulty / 8);
        
        while (true) {
            self.hash = self.calculateHash();
            if (std.mem.startsWith(u8, &self.hash, &target_prefix)) {
                break;
            }
            self.nonce += 1;
        }
    }

    fn calculateMerkleRoot(transactions: []Transaction) [32]u8 {
        if (transactions.len == 0) {
            return std.mem.zeroes([32]u8);
        }
        
        var hasher = std.crypto.hash.sha2.Sha256.init(.{});
        for (transactions) |tx| {
            const tx_hash = tx.hash();
            hasher.update(&tx_hash);
        }
        return hasher.finalResult();
    }
};

// Re-export token transaction as main transaction type
pub const Transaction = tokens.TokenTransaction;

// Legacy transaction structure for compatibility
pub const LegacyTransaction = struct {
    from: []const u8,
    to: []const u8,
    amount: u64,
    fee: u64,
    timestamp: u64,
    signature: []const u8,
    hash: [32]u8,

    pub fn init(from: []const u8, to: []const u8, amount: u64, fee: u64) LegacyTransaction {
        const timestamp = @as(u64, @intCast(std.time.milliTimestamp()));
        var tx = LegacyTransaction{
            .from = from,
            .to = to,
            .amount = amount,
            .fee = fee,
            .timestamp = timestamp,
            .signature = "",
            .hash = undefined,
        };
        tx.hash = tx.calculateHash();
        return tx;
    }

    pub fn calculateHash(self: *const LegacyTransaction) [32]u8 {
        var hasher = std.crypto.hash.sha2.Sha256.init(.{});
        hasher.update(self.from);
        hasher.update(self.to);
        hasher.update(std.mem.asBytes(&self.amount));
        hasher.update(std.mem.asBytes(&self.fee));
        hasher.update(std.mem.asBytes(&self.timestamp));
        return hasher.finalResult();
    }
};