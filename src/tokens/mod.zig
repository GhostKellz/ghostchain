// 🪙 GhostChain Four-Token System: GCC + SPIRIT + MANA + GHOST 👻
const std = @import("std");
const shroud = @import("shroud");
const ghostcipher = shroud.ghostcipher;

pub const TokenType = enum {
    GCC,     // Native gas & transaction fees
    SPIRIT,  // Governance & voting rights 
    MANA,    // Utility & rewards
    GHOST,   // Brand & collectibles 👻
};

pub const Address = [20]u8;

// Account state with all four token balances
pub const Account = struct {
    gcc_balance: u64,      // Gas & transaction fees
    spirit_balance: u64,   // Governance voting power
    mana_balance: u64,     // Utility & rewards  
    ghost_balance: u64,    // Brand & collectibles 👻
    nonce: u64,
    
    pub fn init() Account {
        return Account{
            .gcc_balance = 0,
            .spirit_balance = 0,
            .mana_balance = 0,
            .ghost_balance = 0,
            .nonce = 0,
        };
    }
    
    pub fn getBalance(self: *const Account, token: TokenType) u64 {
        return switch (token) {
            .GCC => self.gcc_balance,
            .SPIRIT => self.spirit_balance,
            .MANA => self.mana_balance,
            .GHOST => self.ghost_balance,
        };
    }
    
    pub fn setBalance(self: *Account, token: TokenType, amount: u64) void {
        switch (token) {
            .GCC => self.gcc_balance = amount,
            .SPIRIT => self.spirit_balance = amount,
            .MANA => self.mana_balance = amount,
            .GHOST => self.ghost_balance = amount,
        }
    }
    
    pub fn addBalance(self: *Account, token: TokenType, amount: u64) !void {
        const current = self.getBalance(token);
        if (current + amount < current) return error.Overflow;
        self.setBalance(token, current + amount);
    }
    
    pub fn subBalance(self: *Account, token: TokenType, amount: u64) !void {
        const current = self.getBalance(token);
        if (current < amount) return error.InsufficientBalance;
        self.setBalance(token, current - amount);
    }
};

// Token system manager
pub const TokenSystem = struct {
    allocator: std.mem.Allocator,
    accounts: std.HashMap(Address, Account, AddressContext, std.hash_map.default_max_load_percentage),
    total_supply: [4]u64, // GCC, SPIRIT, MANA, GHOST
    
    pub fn init(allocator: std.mem.Allocator) TokenSystem {
        return TokenSystem{
            .allocator = allocator,
            .accounts = std.HashMap(Address, Account, AddressContext, std.hash_map.default_max_load_percentage).init(allocator),
            .total_supply = [4]u64{ 0, 100_000_000, 0, 1_000_000 }, // Fixed SPIRIT & GHOST supply (base units)
        };
    }
    
    pub fn deinit(self: *TokenSystem) void {
        self.accounts.deinit();
    }
    
    // Get account (creates if doesn't exist)
    pub fn getAccount(self: *TokenSystem, address: Address) !*Account {
        const result = try self.accounts.getOrPut(address);
        if (!result.found_existing) {
            result.value_ptr.* = Account.init();
        }
        return result.value_ptr;
    }
    
    // Core token operations
    pub fn getBalance(self: *TokenSystem, address: Address, token: TokenType) !u64 {
        const account = try self.getAccount(address);
        return account.getBalance(token);
    }
    
    pub fn transfer(self: *TokenSystem, from: Address, to: Address, token: TokenType, amount: u64) !void {
        if (amount == 0) return;
        
        var from_account = try self.getAccount(from);
        var to_account = try self.getAccount(to);
        
        try from_account.subBalance(token, amount);
        try to_account.addBalance(token, amount);
    }
    
    // GCC operations (gas token)
    pub fn payGasFee(self: *TokenSystem, from: Address, gcc_amount: u64) !void {
        var account = try self.getAccount(from);
        try account.subBalance(.GCC, gcc_amount);
        // GCC fees are burned (deflationary)
        self.total_supply[0] -= gcc_amount;
    }
    
    pub fn mintGCC(self: *TokenSystem, to: Address, amount: u64) !void {
        var account = try self.getAccount(to);
        try account.addBalance(.GCC, amount);
        self.total_supply[0] += amount;
    }
    
    // SPIRIT operations (governance)
    pub fn delegateVotingPower(self: *TokenSystem, from: Address, to: Address, amount: u64) !void {
        // TODO: Implement delegation tracking
        try self.transfer(from, to, .SPIRIT, amount);
    }
    
    // MANA operations (utility)
    pub fn spendMANA(self: *TokenSystem, user: Address, amount: u64) !void {
        var account = try self.getAccount(user);
        try account.subBalance(.MANA, amount);
        // MANA is burned when spent
        self.total_supply[2] -= amount;
    }
    
    pub fn earnMANA(self: *TokenSystem, user: Address, amount: u64) !void {
        var account = try self.getAccount(user);
        try account.addBalance(.MANA, amount);
        self.total_supply[2] += amount;
    }
    
    // GHOST operations (brand & collectibles) 👻
    pub fn mintGHOST(self: *TokenSystem, to: Address, amount: u64) !void {
        if (self.total_supply[3] + amount > 1_000_000) return error.MaxSupplyExceeded;
        var account = try self.getAccount(to);
        try account.addBalance(.GHOST, amount);
        self.total_supply[3] += amount;
    }
    
    pub fn burnGHOST(self: *TokenSystem, from: Address, amount: u64) !void {
        var account = try self.getAccount(from);
        try account.subBalance(.GHOST, amount);
        self.total_supply[3] -= amount;
    }
    
    // Supply tracking
    pub fn getTotalSupply(self: *TokenSystem, token: TokenType) u64 {
        return switch (token) {
            .GCC => self.total_supply[0],
            .SPIRIT => self.total_supply[1], 
            .MANA => self.total_supply[2],
            .GHOST => self.total_supply[3],
        };
    }
};

// Hash context for Address type
const AddressContext = struct {
    pub fn hash(self: @This(), addr: Address) u64 {
        _ = self;
        return std.hash_map.hashString(@as(*const [20:0]u8, @ptrCast(&addr))[0..]);
    }
    
    pub fn eql(self: @This(), a: Address, b: Address) bool {
        _ = self;
        return std.mem.eql(u8, &a, &b);
    }
};

// Transaction structure with token fees
pub const TokenTransaction = struct {
    from: Address,
    to: Address,
    token: TokenType,
    amount: u64,
    gas_fee: u64,  // Always paid in GCC
    nonce: u64,
    signature: [64]u8,
    
    pub fn hash(self: *const TokenTransaction) [32]u8 {
        var hasher = std.crypto.hash.sha2.Sha256.init(.{});
        hasher.update(&self.from);
        hasher.update(&self.to);
        hasher.update(@as(*const [1]u8, @ptrCast(&self.token))[0..]);
        hasher.update(std.mem.asBytes(&self.amount));
        hasher.update(std.mem.asBytes(&self.gas_fee));
        hasher.update(std.mem.asBytes(&self.nonce));
        return hasher.finalResult();
    }
};

// Genesis token distribution
pub fn createGenesisDistribution(token_system: *TokenSystem) !void {
    const genesis_address = std.mem.zeroes(Address);
    
    // Initial GCC for validators
    try token_system.mintGCC(genesis_address, 1_000_000);
    
    // Initial MANA for ecosystem bootstrap
    try token_system.earnMANA(genesis_address, 10_000_000);
    
    // Initial GHOST for community
    try token_system.mintGHOST(genesis_address, 100_000);
}