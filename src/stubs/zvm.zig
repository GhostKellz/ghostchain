// ZVM integration for GhostChain - Smart contract execution engine
const std = @import("std");
const zvm = @import("zvm");

// Re-export zvm types for compatibility
pub const VirtualMachine = zvm.VirtualMachine;
pub const WasmExecutionResult = zvm.ExecutionResult;

// GhostChain-specific VM operations
pub const GhostVM = struct {
    vm: VirtualMachine,
    
    pub fn init(allocator: std.mem.Allocator) !GhostVM {
        return GhostVM{
            .vm = try VirtualMachine.init(allocator),
        };
    }
    
    pub fn deinit(self: *GhostVM) void {
        self.vm.deinit();
    }
    
    pub fn loadBytecode(self: *GhostVM, bytecode: []const u8) !void {
        return self.vm.loadBytecode(bytecode);
    }
    
    pub fn execute(self: *GhostVM, function: []const u8, args: []const u8) !WasmExecutionResult {
        return self.vm.execute(function, args);
    }
    
    pub fn executeContract(self: *GhostVM, contract_address: []const u8, input_data: []const u8) ![]u8 {
        return self.vm.executeContract(contract_address, input_data);
    }
    
    pub fn deployContract(self: *GhostVM, bytecode: []const u8, constructor_args: []const u8) ![]const u8 {
        return self.vm.deployContract(bytecode, constructor_args);
    }
    
    pub fn getGasUsage(self: *GhostVM) u64 {
        return self.vm.getGasUsage();
    }
    
    pub fn resetGasCounter(self: *GhostVM) void {
        self.vm.resetGasCounter();
    }
};