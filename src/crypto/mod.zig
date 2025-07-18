// GhostChain crypto integration - combines zcrypto and shroud
const std = @import("std");
const zcrypto = @import("zcrypto");
const zsig = @import("zsig");
const shroud = @import("shroud");

// Re-export zcrypto functionality
pub const hash = zcrypto.hash;
pub const sym = zcrypto.sym;
pub const kdf = zcrypto.kdf;
pub const rand = zcrypto.rand;
pub const util = zcrypto.util;

// GhostChain specific crypto extensions
pub const GhostCrypto = struct {
    pub fn deriveGhostAddress(allocator: std.mem.Allocator, public_key: []const u8) ![]u8 {
        // Use SHA-256 hash for address derivation
        const addr_hash = zcrypto.hash.sha256(public_key);
        
        // Take last 20 bytes of hash for address
        const address = try allocator.alloc(u8, 20);
        @memcpy(address, addr_hash[12..32]);
        return address;
    }
    
    pub fn generateKeyPair(allocator: std.mem.Allocator) !KeyPair {
        // Generate Ed25519 key pair using zsig
        const keypair = try zsig.Ed25519.generateKeyPair(allocator);
        
        return KeyPair{
            .public_key = keypair.public_key,
            .private_key = keypair.private_key,
            .key_type = .ed25519,
        };
    }
    
    pub fn signMessage(message: []const u8, private_key: []const u8, key_type: KeyType) ![]u8 {
        switch (key_type) {
            .ed25519 => return zsig.Ed25519.sign(message, private_key),
            .secp256k1 => return zsig.Secp256k1.sign(message, private_key),
        }
    }
    
    pub fn verifyGhostSignature(message: []const u8, signature: []const u8, public_key: []const u8, key_type: KeyType) !bool {
        switch (key_type) {
            .ed25519 => return zsig.Ed25519.verify(message, signature, public_key),
            .secp256k1 => return zsig.Secp256k1.verify(message, signature, public_key),
        }
    }
    
    pub fn hashTransaction(tx: *const Transaction) [32]u8 {
        var hasher = std.crypto.hash.Sha256.init(.{});
        hasher.update(tx.from);
        hasher.update(tx.to);
        hasher.update(std.mem.asBytes(&tx.amount));
        hasher.update(std.mem.asBytes(&tx.fee));
        hasher.update(std.mem.asBytes(&tx.timestamp));
        hasher.update(tx.data);
        return hasher.finalResult();
    }
    
    pub fn hashBlock(block: *const Block) [32]u8 {
        var hasher = std.crypto.hash.Sha256.init(.{});
        hasher.update(std.mem.asBytes(&block.index));
        hasher.update(std.mem.asBytes(&block.timestamp));
        hasher.update(&block.previous_hash);
        hasher.update(&block.merkle_root);
        hasher.update(std.mem.asBytes(&block.nonce));
        return hasher.finalResult();
    }
    
    pub fn encryptData(allocator: std.mem.Allocator, data: []const u8, key: []const u8) ![]u8 {
        return zcrypto.sym.encryptAesGcm(allocator, data, key);
    }
    
    pub fn decryptData(allocator: std.mem.Allocator, encrypted_data: []const u8, key: []const u8) ![]u8 {
        return zcrypto.sym.decryptAesGcm(allocator, encrypted_data, key);
    }
    
    pub fn deriveKey(password: []const u8, salt: []const u8, iterations: u32, key_length: u32) ![64]u8 {
        return zcrypto.kdf.pbkdf2(password, salt, iterations, key_length);
    }
    
    pub fn generateSecureRandom(buffer: []u8) !void {
        try zcrypto.rand.fillBytes(buffer);
    }
    
    pub fn constantTimeCompare(a: []const u8, b: []const u8) bool {
        return zcrypto.util.constantTimeCompare(a, b);
    }
};

pub const KeyPair = struct {
    public_key: []const u8,
    private_key: []const u8,
    key_type: KeyType,
    
    pub fn deinit(self: *KeyPair, allocator: std.mem.Allocator) void {
        allocator.free(self.public_key);
        zcrypto.util.secureZero(self.private_key);
        allocator.free(self.private_key);
    }
};

pub const KeyType = enum {
    ed25519,
    secp256k1,
};

// Forward declarations (these should be defined in blockchain module)
const Transaction = struct {
    from: []const u8,
    to: []const u8,
    amount: u64,
    fee: u64,
    timestamp: u64,
    data: []const u8,
};

const Block = struct {
    index: u64,
    timestamp: u64,
    previous_hash: [32]u8,
    merkle_root: [32]u8,
    nonce: u64,
};