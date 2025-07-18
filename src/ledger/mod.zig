// GhostChain ledger integration - combines keystone and zledger
const std = @import("std");
const keystone = @import("keystone");
const zledger = @import("zledger");

pub const Ledger = keystone.Ledger;
pub const Transaction = keystone.Transaction;
pub const Journal = keystone.Journal;
pub const Auditor = keystone.Auditor;

// GhostChain specific ledger extensions using Keystone and ZLedger
pub const GhostLedger = struct {
    keystone_ledger: keystone.Ledger,
    zledger_instance: zledger.Ledger,
    journal: keystone.Journal,
    
    pub fn init(allocator: std.mem.Allocator, journal_path: []const u8) GhostLedger {
        return GhostLedger{
            .keystone_ledger = keystone.Ledger.init(allocator),
            .zledger_instance = zledger.Ledger.init(allocator),
            .journal = keystone.Journal.init(allocator, journal_path),
        };
    }
    
    pub fn deinit(self: *GhostLedger) void {
        self.keystone_ledger.deinit();
        self.zledger_instance.deinit();
        self.journal.deinit();
    }
    
    pub fn processGhostTransaction(self: *GhostLedger, from: []const u8, to: []const u8, amount: u64, memo: ?[]const u8) !void {
        // Process in keystone for auditing
        const keystone_tx = try keystone.Transaction.init(
            self.keystone_ledger.allocator,
            @intCast(amount),
            "GHOST",
            from,
            to,
            memo,
        );
        defer keystone_tx.deinit(self.keystone_ledger.allocator);
        
        // Process in zledger for precision accounting
        const zledger_tx = try zledger.Transaction.init(
            self.zledger_instance.allocator,
            @intCast(amount),
            "GHOST",
            from,
            to,
            memo,
        );
        defer zledger_tx.deinit(self.zledger_instance.allocator);
        
        try self.keystone_ledger.processTransaction(keystone_tx);
        try self.zledger_instance.processTransaction(zledger_tx);
        try self.journal.append(keystone_tx);
    }
    
    pub fn getAccountBalance(self: *GhostLedger, account: []const u8) i64 {
        // Use zledger for precision balance queries
        return self.zledger_instance.getBalance(account);
    }
    
    pub fn auditLedger(self: *GhostLedger) !bool {
        var auditor = keystone.Auditor.init(self.keystone_ledger.allocator);
        var report = try auditor.auditLedger(&self.keystone_ledger, &self.journal);
        defer report.deinit(self.keystone_ledger.allocator);
        return report.isValid();
    }
};