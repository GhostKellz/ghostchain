// Re-export gwallet functionality for ghostchain
const std = @import("std");
const shroud = @import("shroud");
const gwallet = shroud.gwallet;

pub const Wallet = gwallet.Wallet;
pub const Account = gwallet.Account;
pub const KeyPair = gwallet.KeyPair;

// GhostChain specific wallet extensions using gwallet
pub const GhostWallet = struct {
    allocator: std.mem.Allocator,
    wallet: gwallet.Wallet,
    
    pub fn init(allocator: std.mem.Allocator) !GhostWallet {
        // Create a default wallet with a generic passphrase
        // In production, this should be configurable
        const wallet = try gwallet.Wallet.create(allocator, "default_passphrase", .hybrid, "ghost_wallet");
        return GhostWallet{
            .allocator = allocator,
            .wallet = wallet,
        };
    }
    
    pub fn create(allocator: std.mem.Allocator, passphrase: []const u8, name: ?[]const u8) !GhostWallet {
        const wallet = try gwallet.Wallet.create(allocator, passphrase, .hybrid, name);
        return GhostWallet{
            .allocator = allocator,
            .wallet = wallet,
        };
    }
    
    pub fn load(allocator: std.mem.Allocator, stored_data: []const u8, passphrase: []const u8) !GhostWallet {
        const wallet = try gwallet.Wallet.load(allocator, stored_data, passphrase);
        return GhostWallet{
            .allocator = allocator,
            .wallet = wallet,
        };
    }
    
    pub fn fromMnemonic(allocator: std.mem.Allocator, mnemonic: []const u8, password: ?[]const u8) !GhostWallet {
        const wallet = try gwallet.Wallet.fromMnemonic(allocator, mnemonic, password, .hybrid);
        return GhostWallet{
            .allocator = allocator,
            .wallet = wallet,
        };
    }
    
    pub fn deinit(self: *GhostWallet) void {
        self.wallet.deinit();
    }
    
    pub fn createGhostAccount(self: *GhostWallet, name: []const u8) ![]const u8 {
        _ = name; // TODO: Use name in account creation
        const account = try self.wallet.createAccount(.ghostchain, .ed25519);
        // Convert account to address string
        return try std.fmt.allocPrint(self.allocator, "ghost:{s}", .{account.address});
    }
    
    pub fn getGhostBalance(self: *GhostWallet) ?u64 {
        return self.wallet.getBalance(.ghostchain, "GHOST");
    }
    
    pub fn updateGhostBalance(self: *GhostWallet, amount: u64) !void {
        return self.wallet.updateBalance(.ghostchain, "GHOST", amount, 8);
    }
};