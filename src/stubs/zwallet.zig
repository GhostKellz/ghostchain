// ZWallet integration for GhostChain - HD wallet and key management
const std = @import("std");
const zwallet = @import("zwallet");

// Re-export zwallet types for compatibility
pub const Wallet = zwallet.Wallet;
pub const Account = zwallet.Account;
pub const KeyPair = zwallet.KeyPair;

// GhostChain-specific wallet operations
pub const GhostWallet = struct {
    wallet: Wallet,
    
    pub fn init(allocator: std.mem.Allocator) !GhostWallet {
        return GhostWallet{
            .wallet = try Wallet.init(allocator),
        };
    }
    
    pub fn deinit(self: *GhostWallet) void {
        self.wallet.deinit();
    }
    
    pub fn createGhostAccount(self: *GhostWallet, name: []const u8) ![]const u8 {
        const account = try self.wallet.createAccount(name, .ghost);
        return account.getAddress();
    }
    
    pub fn getBalance(self: *GhostWallet, account: []const u8, currency: []const u8) !u64 {
        return self.wallet.getBalance(account, currency);
    }
    
    pub fn sendTransaction(self: *GhostWallet, from: []const u8, to: []const u8, amount: u64, currency: []const u8) ![]const u8 {
        return self.wallet.send(from, to, amount, currency);
    }
    
    pub fn signTransaction(self: *GhostWallet, account: []const u8, tx_data: []const u8) ![]const u8 {
        return self.wallet.signTransaction(account, tx_data);
    }
    
    pub fn generateKeyPair(self: *GhostWallet) !KeyPair {
        return self.wallet.generateKeyPair();
    }
};

pub const WalletType = enum {
    ghost,
    standard,
};