const std = @import("std");

pub const ProofOfStake = struct {
    validators: std.ArrayList(Validator),
    minimum_stake: u64,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator, minimum_stake: u64) ProofOfStake {
        return ProofOfStake{
            .validators = std.ArrayList(Validator).init(allocator),
            .minimum_stake = minimum_stake,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *ProofOfStake) void {
        self.validators.deinit();
    }
    
    pub fn addValidator(self: *ProofOfStake, address: []const u8, stake: u64) !void {
        if (stake < self.minimum_stake) return error.InsufficientStake;
        
        const validator = Validator{
            .address = address,
            .stake = stake,
            .active = true,
        };
        
        try self.validators.append(validator);
    }
    
    pub fn selectValidator(self: *ProofOfStake, seed: u64) ?*Validator {
        if (self.validators.items.len == 0) return null;
        
        var total_stake: u64 = 0;
        for (self.validators.items) |validator| {
            if (validator.active) {
                total_stake += validator.stake;
            }
        }
        
        if (total_stake == 0) return null;
        
        const random_value = seed % total_stake;
        var current_stake: u64 = 0;
        
        for (self.validators.items) |*validator| {
            if (!validator.active) continue;
            current_stake += validator.stake;
            if (current_stake > random_value) {
                return validator;
            }
        }
        
        return null;
    }
    
    pub fn validateBlock(self: *ProofOfStake, validator_address: []const u8, block_hash: [32]u8) bool {
        for (self.validators.items) |validator| {
            if (std.mem.eql(u8, validator.address, validator_address) and validator.active) {
                // TODO: Implement actual block validation logic
                _ = block_hash;
                return true;
            }
        }
        return false;
    }
};

pub const Validator = struct {
    address: []const u8,
    stake: u64,
    active: bool,
};