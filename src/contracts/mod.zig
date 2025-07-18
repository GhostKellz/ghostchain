// Re-export ZVM functionality for ghostchain smart contracts
const std = @import("std");
const zvm = @import("../stubs/zvm.zig");

pub const SmartContractExecutor = @import("executor.zig").SmartContractExecutor;
pub const ExecutionContext = @import("executor.zig").ExecutionContext;
pub const ContractDeploymentResult = @import("executor.zig").ContractDeploymentResult;
pub const ContractCallResult = @import("executor.zig").ContractCallResult;
pub const WasmExecutionResult = @import("executor.zig").WasmExecutionResult;

// Re-export ZVM core functionality
pub const VirtualMachine = zvm.VirtualMachine;
pub const WasmEngine = zvm.WasmEngine;
pub const WasmContext = zvm.WasmContext;

// GhostChain specific contract features
pub const GhostContract = struct {
    address: []const u8,
    bytecode: []const u8,
    storage: std.HashMap([]const u8, []const u8, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    deployer: []const u8,
    deployed_at: u64,
    
    pub fn init(allocator: std.mem.Allocator, address: []const u8, bytecode: []const u8, deployer: []const u8) GhostContract {
        return GhostContract{
            .address = address,
            .bytecode = bytecode,
            .storage = std.HashMap([]const u8, []const u8, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .deployer = deployer,
            .deployed_at = @as(u64, @intCast(std.time.milliTimestamp())),
        };
    }
    
    pub fn deinit(self: *GhostContract) void {
        self.storage.deinit();
    }
    
    pub fn getStorageValue(self: *GhostContract, key: []const u8) ?[]const u8 {
        return self.storage.get(key);
    }
    
    pub fn setStorageValue(self: *GhostContract, key: []const u8, value: []const u8) !void {
        try self.storage.put(key, value);
    }
};

pub const ContractRegistry = struct {
    allocator: std.mem.Allocator,
    contracts: std.HashMap([]const u8, GhostContract, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator) ContractRegistry {
        return ContractRegistry{
            .allocator = allocator,
            .contracts = std.HashMap([]const u8, GhostContract, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    pub fn deinit(self: *ContractRegistry) void {
        var iterator = self.contracts.iterator();
        while (iterator.next()) |entry| {
            entry.value_ptr.deinit();
        }
        self.contracts.deinit();
    }
    
    pub fn registerContract(self: *ContractRegistry, contract: GhostContract) !void {
        try self.contracts.put(contract.address, contract);
    }
    
    pub fn getContract(self: *ContractRegistry, address: []const u8) ?*GhostContract {
        return self.contracts.getPtr(address);
    }
    
    pub fn contractExists(self: *ContractRegistry, address: []const u8) bool {
        return self.contracts.contains(address);
    }
};