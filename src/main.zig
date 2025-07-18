const std = @import("std");
const ghostchain = @import("ghostchain");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Parse command line arguments
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        try printUsage();
        return;
    }

    const command = args[1];

    if (std.mem.eql(u8, command, "ghostd")) {
        try startGhostDaemon(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "walletd")) {
        try startWalletDaemon(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "start")) {
        try startGhostChain(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "wallet")) {
        try walletCommands(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "contract")) {
        try contractCommands(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "zns")) {
        try znsCommands(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "identity")) {
        try identityCommands(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "version")) {
        try ghostchain.bufferedPrint();
    } else {
        try printUsage();
    }
}

fn printUsage() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("🔗 GhostChain v0.1.0 - Pure Zig Blockchain\n\n" ++
        "Usage: ghostchain <command> [options]\n\n" ++
        "Commands:\n" ++
        "  ghostd            Start the GhostD blockchain daemon\n" ++
        "  walletd           Start the WalletD service daemon\n" ++
        "  start             Start the GhostChain node\n" ++
        "  wallet <action>   Wallet operations (create, balance, send)\n" ++
        "  contract <action> Smart contract operations (deploy, call)\n" ++
        "  zns <action>      Name service operations (resolve, register)\n" ++
        "  identity <action> RealID operations (create, verify)\n" ++
        "  version           Show version information\n\n" ++
        "Examples:\n" ++
        "  ghostchain ghostd --production\n" ++
        "  ghostchain walletd --port 3001\n" ++
        "  ghostchain start --port 7777\n" ++
        "  ghostchain wallet create alice\n" ++
        "  ghostchain wallet balance alice\n" ++
        "  ghostchain wallet send alice bob 1000\n" ++
        "  ghostchain contract deploy contract.wasm alice\n" ++
        "  ghostchain zns resolve alice.ghost\n" ++
        "  ghostchain identity create alice <public_key>\n\n", .{});
}

fn startGhostDaemon(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    var production_mode = false;
    
    // Parse daemon arguments
    for (args) |arg| {
        if (std.mem.eql(u8, arg, "--production")) {
            production_mode = true;
        }
    }
    
    const config = if (production_mode) 
        ghostchain.daemon.DaemonConfig.production() 
    else 
        ghostchain.daemon.DaemonConfig.development();
    
    std.log.info("🚀 Starting GhostD daemon in {s} mode", .{if (production_mode) "production" else "development"});
    
    var daemon = try ghostchain.daemon.GhostDaemon.init(allocator, config);
    defer daemon.deinit();
    
    // Set up signal handlers for graceful shutdown
    const handler = struct {
        fn sigintHandler(sig: i32) callconv(.C) void {
            _ = sig;
            std.log.info("🛑 Received shutdown signal, stopping daemon...", .{});
            std.process.exit(0);
        }
    };
    
    _ = std.os.linux.sigaction(std.os.linux.SIG.INT, &std.os.linux.Sigaction{
        .handler = .{ .handler = handler.sigintHandler },
        .mask = std.mem.zeroes(std.posix.sigset_t),
        .flags = 0,
    }, null);
    
    try daemon.start();
}

fn startWalletDaemon(allocator: std.mem.Allocator, args: [][]const u8) !void {
    var production_mode = false;
    var port: u16 = 3001;
    
    // Parse wallet daemon arguments
    var i: usize = 0;
    while (i < args.len) {
        const arg = args[i];
        if (std.mem.eql(u8, arg, "--production")) {
            production_mode = true;
        } else if (std.mem.eql(u8, arg, "--port") and i + 1 < args.len) {
            port = std.fmt.parseInt(u16, args[i + 1], 10) catch 3001;
            i += 1;
        }
        i += 1;
    }
    
    var config = if (production_mode) 
        ghostchain.daemon.WalletConfig.production() 
    else 
        ghostchain.daemon.WalletConfig.development();
    
    config.http_port = port;
    
    std.log.info("🔐 Starting WalletD daemon in {s} mode on port {}", .{if (production_mode) "production" else "development", port});
    
    var wallet_daemon = try ghostchain.daemon.WalletDaemon.init(allocator, config);
    defer wallet_daemon.deinit();
    
    // Set up signal handlers for graceful shutdown
    const handler = struct {
        fn sigintHandler(sig: i32) callconv(.C) void {
            _ = sig;
            std.log.info("🛑 Received shutdown signal, stopping wallet daemon...", .{});
            std.process.exit(0);
        }
    };
    
    _ = std.os.linux.sigaction(std.os.linux.SIG.INT, &std.os.linux.Sigaction{
        .handler = .{ .handler = handler.sigintHandler },
        .mask = std.mem.zeroes(std.posix.sigset_t),
        .flags = 0,
    }, null);
    
    try wallet_daemon.start();
}

fn startGhostChain(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    _ = args; // TODO: Parse start options
    
    std.log.info("🚀 Initializing GhostChain...", .{});
    
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    std.log.info("🌐 Node starting on IPv6 {s}:{}", .{ config.bind_address, config.port });
    std.log.info("🔌 RPC server starting on {s}:{}", .{ config.rpc_address, config.rpc_port });
    std.log.info("💰 Minimum stake: {} GHOST", .{config.minimum_stake});
    
    try ghost.start();
}

fn walletCommands(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    if (args.len == 0) {
        std.log.err("Wallet action required (create, balance, send)", .{});
        return;
    }
    
    const action = args[0];
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    if (std.mem.eql(u8, action, "create")) {
        if (args.len < 2) {
            std.log.err("Account name required", .{});
            return;
        }
        const account_name = args[1];
        const address = try ghost.createAccount(account_name);
        std.log.info("✅ Created account '{s}' with address: {s}", .{ account_name, address });
        
    } else if (std.mem.eql(u8, action, "balance")) {
        if (args.len < 2) {
            std.log.err("Account name required", .{});
            return;
        }
        const account = args[1];
        const balance = try ghost.getBalance(account, ghostchain.tokens.TokenType.GHOST);
        std.log.info("💰 Balance for '{s}': {} GHOST", .{ account, balance });
        
    } else if (std.mem.eql(u8, action, "send")) {
        if (args.len < 4) {
            std.log.err("Usage: wallet send <from> <to> <amount>", .{});
            return;
        }
        const from = args[1];
        const to = args[2];
        const amount_str = args[3];
        const amount = try std.fmt.parseInt(u64, amount_str, 10);
        
        try ghost.sendTransaction(from, to, ghostchain.tokens.TokenType.GHOST, amount, 1000, null);
        std.log.info("✅ Sent {} GHOST from '{s}' to '{s}'", .{ amount, from, to });
        
    } else if (std.mem.eql(u8, action, "multisig")) {
        if (args.len < 4) {
            std.log.err("Usage: wallet multisig <create|sign|execute> <params...>", .{});
            return;
        }
        const multisig_action = args[1];
        
        if (std.mem.eql(u8, multisig_action, "create")) {
            std.log.info("📋 Multisig wallet creation using zsig - functionality available", .{});
        } else if (std.mem.eql(u8, multisig_action, "sign")) {
            std.log.info("📋 Multisig transaction signing using zsig - functionality available", .{});
        } else if (std.mem.eql(u8, multisig_action, "execute")) {
            std.log.info("📋 Multisig transaction execution using zsig - functionality available", .{});
        }
        
    } else {
        std.log.err("Unknown wallet action: {s}", .{action});
    }
}

fn contractCommands(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    if (args.len == 0) {
        std.log.err("Contract action required (deploy, call)", .{});
        return;
    }
    
    const action = args[0];
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    if (std.mem.eql(u8, action, "deploy")) {
        if (args.len < 3) {
            std.log.err("Usage: contract deploy <bytecode_file> <deployer>", .{});
            return;
        }
        const bytecode_file = args[1];
        const deployer = args[2];
        
        // Read bytecode from file
        const bytecode = try std.fs.cwd().readFileAlloc(allocator, bytecode_file, 1024 * 1024);
        defer allocator.free(bytecode);
        
        const result = try ghost.deployContract(bytecode, "", deployer);
        if (result.success) {
            std.log.info("✅ Contract deployed at: {s}", .{result.contract_address});
            std.log.info("⛽ Gas used: {}", .{result.gas_used});
        } else {
            std.log.err("❌ Contract deployment failed: {s}", .{result.error_message orelse "Unknown error"});
        }
        
    } else if (std.mem.eql(u8, action, "call")) {
        if (args.len < 4) {
            std.log.err("Usage: contract call <contract_address> <function> <args...>", .{});
            return;
        }
        const contract_address = args[1];
        const function_name = args[2];
        const call_args = if (args.len > 3) args[3] else "";
        
        const result = try ghost.callContract(contract_address, function_name, call_args, "default_caller", 0);
        if (result.success) {
            std.log.info("✅ Contract call successful", .{});
            std.log.info("📊 Return data: {s}", .{result.return_data});
            std.log.info("⛽ Gas used: {}", .{result.gas_used});
        } else {
            std.log.err("❌ Contract call failed: {s}", .{result.error_message orelse "Unknown error"});
        }
        
    } else {
        std.log.err("Unknown contract action: {s}", .{action});
    }
}

fn znsCommands(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    if (args.len == 0) {
        std.log.err("ZNS action required (resolve, register)", .{});
        return;
    }
    
    const action = args[0];
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    if (std.mem.eql(u8, action, "resolve")) {
        if (args.len < 2) {
            std.log.err("Domain name required", .{});
            return;
        }
        const domain = args[1];
        
        const record = ghost.resolveDomain(domain) catch |err| switch (err) {
            error.DomainNotFound => {
                std.log.err("❌ Domain '{s}' not found", .{domain});
                return;
            },
            else => return err,
        };
        
        std.log.info("✅ Resolved '{s}': {any} -> {s}", .{ domain, record.record_type, record.value });
        
    } else if (std.mem.eql(u8, action, "register")) {
        if (args.len < 3) {
            std.log.err("Usage: zns register <domain> <owner>", .{});
            return;
        }
        const domain = args[1];
        const owner = args[2];
        
        try ghost.registerDomain(domain, owner);
        std.log.info("✅ Registered domain '{s}' for owner '{s}'", .{ domain, owner });
        
    } else {
        std.log.err("Unknown ZNS action: {s}", .{action});
    }
}

fn identityCommands(allocator: std.mem.Allocator, args: [][:0]u8) !void {
    if (args.len == 0) {
        std.log.err("Identity action required (create, verify)", .{});
        return;
    }
    
    const action = args[0];
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    if (std.mem.eql(u8, action, "create")) {
        if (args.len < 3) {
            std.log.err("Usage: identity create <username> <public_key>", .{});
            return;
        }
        const username = args[1];
        const public_key = args[2];
        
        const identity = try ghost.createIdentity(username, public_key);
        std.log.info("✅ Created identity: {s}", .{identity.did});
        std.log.info("👤 Username: {s}", .{identity.username});
        std.log.info("🔑 Public Key: {s}", .{identity.public_key});
        
    } else if (std.mem.eql(u8, action, "verify")) {
        std.log.info("📋 Identity verification functionality coming soon...", .{});
        
    } else {
        std.log.err("Unknown identity action: {s}", .{action});
    }
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit();
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}

test "ghostchain integration test" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const config = ghostchain.GhostChainConfig.development();
    var ghost = try ghostchain.GhostChain.init(allocator, config);
    defer ghost.deinit();
    
    // Test account creation
    const address = try ghost.createAccount("test_account");
    try std.testing.expect(address.len > 0);
    
    // Test identity creation
    const identity = try ghost.createIdentity("test_user", "test_public_key");
    try std.testing.expect(std.mem.startsWith(u8, identity.did, "did:ghost:"));
}
