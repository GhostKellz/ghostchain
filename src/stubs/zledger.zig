// ZLedger integration for GhostChain - replaces temporary stub
const std = @import("std");
const zledger = @import("zledger");

// Re-export zledger types for compatibility
pub const Ledger = zledger.Ledger;
pub const Transaction = zledger.Transaction;
pub const Journal = zledger.Journal;
pub const Auditor = zledger.Auditor;
pub const AuditReport = zledger.AuditReport;

// GhostChain-specific ledger operations
pub const GhostLedger = struct {
    ledger: Ledger,
    journal: Journal,
    auditor: Auditor,
    
    pub fn init(allocator: std.mem.Allocator, journal_path: []const u8) !GhostLedger {
        return GhostLedger{
            .ledger = Ledger.init(allocator),
            .journal = Journal.init(allocator, journal_path),
            .auditor = Auditor.init(allocator),
        };
    }
    
    pub fn deinit(self: *GhostLedger) void {
        self.ledger.deinit();
        self.journal.deinit();
        self.auditor.deinit();
    }
    
    pub fn processGhostTransaction(self: *GhostLedger, from: []const u8, to: []const u8, amount: i64, currency: []const u8, memo: ?[]const u8) !void {
        const tx = try Transaction.init(self.ledger.allocator, amount, currency, from, to, memo);
        defer tx.deinit(self.ledger.allocator);
        
        try self.ledger.processTransaction(tx);
        try self.journal.append(tx);
    }
    
    pub fn getBalance(self: *GhostLedger, account: []const u8) i64 {
        return self.ledger.getBalance(account);
    }
    
    pub fn performAudit(self: *GhostLedger) !AuditReport {
        return self.auditor.auditLedger(&self.ledger, &self.journal);
    }
};