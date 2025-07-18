const std = @import("std");
const ghostchain = @import("../root.zig");
const shroud = @import("shroud");
const ghostwire = shroud.ghostwire;
const ghostcipher = shroud.ghostcipher;
const wraith = @import("wraith");

pub const WalletDaemon = struct {
    allocator: std.mem.Allocator,
    config: WalletConfig,
    wallet_manager: ghostchain.wallet.GhostWallet,
    key_store: SecureKeyStore,
    quic_server: ?*anyopaque, // Stub until ghostwire.Server is available
    wraith_server: ?*anyopaque, // Stub until wraith.WraithGateway is available
    session_manager: SessionManager,
    running: bool,
    
    pub fn init(allocator: std.mem.Allocator, config: WalletConfig) !WalletDaemon {
        const wallet_manager = try ghostchain.wallet.GhostWallet.init(allocator);
        const key_store = try SecureKeyStore.init(allocator, config.keystore_path);
        
        // Initialize QUIC server for secure wallet operations
        const quic_server: ?*anyopaque = null; // Stub until ghostwire.Server is available
        // TODO: Enable when ghostwire.Server is available
        // const quic_server = try ghostwire.Server.init(allocator, .{
        //     .bind_address = config.bind_address,
        //     .port = config.quic_port,
        //     .tls_config = .{
        //         .certificate_path = config.tls_cert_path,
        //         .private_key_path = config.tls_key_path,
        //     },
        // });
        
        // Initialize Wraith server for HTTP API
        const wraith_server: ?*anyopaque = null; // Stub until wraith.WraithGateway API is available
        // TODO: Enable when wraith.WraithGateway is available
        // const wraith_server = try wraith.WraithGateway.init(allocator);
        
        return WalletDaemon{
            .allocator = allocator,
            .config = config,
            .wallet_manager = wallet_manager,
            .key_store = key_store,
            .quic_server = quic_server,
            .wraith_server = wraith_server,
            .session_manager = SessionManager.init(allocator),
            .running = false,
        };
    }
    
    pub fn deinit(self: *WalletDaemon) void {
        self.key_store.deinit();
        if (self.quic_server) |server| { server.deinit(); }
        if (self.wraith_server) |server| { server.deinit(); }
        self.session_manager.deinit();
    }
    
    pub fn start(self: *WalletDaemon) !void {
        std.log.info("🔐 Starting WalletD daemon v0.1.0", .{});
        std.log.info("🌐 QUIC secure server: {s}:{}", .{ self.config.bind_address, self.config.quic_port });
        std.log.info("🌍 HTTP API server: {s}:{}", .{ self.config.bind_address, self.config.http_port });
        
        self.running = true;
        
        // Set up API routes
        // try self.setupAPIRoutes();
        
        // Start QUIC server
        // try self.quic_server.start();
        
        // Start HTTP server
        // try self.wraith_server.start();
        
        // Load existing wallets
        try self.loadExistingWallets();
        
        std.log.info("✅ WalletD is running!", .{});
        
        // Main event loop
        try self.runEventLoop();
    }
    
    pub fn stop(self: *WalletDaemon) void {
        std.log.info("🛑 Stopping WalletD daemon...", .{});
        self.running = false;
        // self.quic_server.stop();
        // self.wraith_server.stop();
        
        // Secure cleanup of sensitive data
        self.key_store.secureCleanup();
    }
    
    fn setupAPIRoutes(self: *WalletDaemon) !void {
        _ = self;
        // TODO: Implement when wraith.WraithGateway API is available
        // Authentication
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/auth/login",
            .method = .POST,
            .handler = self.handleLogin,
            .priority = 100,
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/auth/logout",
            .method = .POST,
            .handler = self.handleLogout,
            .priority = 100,
        });
        
        // Wallet management (requires authentication)
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets",
            .method = .{ .GET, .POST },
            .handler = self.handleWallets,
            .priority = 90,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets/{wallet_id}",
            .method = .{ .GET, .PUT, .DELETE },
            .handler = self.handleWalletById,
            .priority = 90,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        // Account operations
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets/{wallet_id}/accounts",
            .method = .{ .GET, .POST },
            .handler = self.handleAccounts,
            .priority = 80,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets/{wallet_id}/accounts/{account_id}/balance",
            .method = .GET,
            .handler = self.handleAccountBalance,
            .priority = 80,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        // Transaction operations
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets/{wallet_id}/transactions",
            .method = .{ .GET, .POST },
            .handler = self.handleTransactions,
            .priority = 70,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/wallets/{wallet_id}/sign",
            .method = .POST,
            .handler = self.handleSignTransaction,
            .priority = 70,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        // Key management
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/keys/generate",
            .method = .POST,
            .handler = self.handleGenerateKeys,
            .priority = 60,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/keys/import",
            .method = .POST,
            .handler = self.handleImportKeys,
            .priority = 60,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        // Multi-signature operations
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/multisig/*",
            .method = .{ .GET, .POST, .PUT },
            .handler = self.handleMultiSig,
            .priority = 50,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
        
        // Health and status
        try self.wraith_server.addRoute(.{
            .path = "/health",
            .method = .GET,
            .handler = self.handleHealthCheck,
            .priority = 40,
        });
        
        try self.wraith_server.addRoute(.{
            .path = "/api/v1/status",
            .method = .GET,
            .handler = self.handleStatus,
            .priority = 40,
            .middleware = &[_]wraith.Middleware{wraith.AuthMiddleware},
        });
    }
    
    fn runEventLoop(self: *WalletDaemon) !void {
        while (self.running) {
            // Process QUIC connections
            // try self.quic_server.processEvents(100);
            
            // Process HTTP requests
            // try self.wraith_server.processRequests(100);
            
            // Clean up expired sessions
            self.session_manager.cleanupExpiredSessions();
            
            std.time.sleep(10_000_000); // 10ms sleep
        }
    }
    
    // API Route Handlers
    // TODO: Enable when wraith.WraithGateway API is available
    fn handleLogin(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const body = try request.readBody(self.allocator);
        defer self.allocator.free(body);
        
        const login_request = try std.json.parseFromSlice(LoginRequest, self.allocator, body, .{});
        defer login_request.deinit();
        
        // Verify credentials (implement secure authentication)
        const is_valid = try self.verifyCredentials(login_request.value.username, login_request.value.password);
        
        if (is_valid) {
            // Create session
            const session = try self.session_manager.createSession(login_request.value.username);
            
            const login_response = LoginResponse{
                .success = true,
                .session_token = session.token,
                .expires_at = session.expires_at,
                .message = "Login successful",
            };
            
            const json_response = try std.json.stringifyAlloc(self.allocator, login_response, .{});
            defer self.allocator.free(json_response);
            
            response.setHeader("Content-Type", "application/json");
            try response.writeBody(json_response);
        } else {
            response.setStatus(401);
            try response.writeBody("{\"error\": \"Invalid credentials\"}");
        }
    }
    
    fn handleLogout(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const session_token = request.getHeader("Authorization") orelse {
            response.setStatus(401);
            try response.writeBody("{\"error\": \"No session token provided\"}");
            return;
        };
        
        self.session_manager.invalidateSession(session_token);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"message\": \"Logged out successfully\"}");
    }
    
    fn handleWallets(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        switch (request.method) {
            .GET => try self.handleListWallets(request, response),
            .POST => try self.handleCreateWallet(request, response),
            else => {
                response.setStatus(405);
                try response.writeBody("{\"error\": \"Method not allowed\"}");
            },
        }
    }
    
    fn handleListWallets(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = request;
        
        const wallets = try self.key_store.listWallets();
        defer self.allocator.free(wallets);
        
        const json_response = try std.json.stringifyAlloc(self.allocator, wallets, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(json_response);
    }
    
    fn handleCreateWallet(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const body = try request.readBody(self.allocator);
        defer self.allocator.free(body);
        
        const create_request = try std.json.parseFromSlice(CreateWalletRequest, self.allocator, body, .{});
        defer create_request.deinit();
        
        // Generate new wallet
        const wallet = try self.createNewWallet(create_request.value);
        defer wallet.deinit(self.allocator);
        
        // Store in keystore
        try self.key_store.storeWallet(wallet);
        
        const wallet_response = WalletResponse{
            .id = wallet.id,
            .name = wallet.name,
            .address = wallet.address,
            .created_at = wallet.created_at,
            .accounts = wallet.accounts,
        };
        
        const json_response = try std.json.stringifyAlloc(self.allocator, wallet_response, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        response.setStatus(201);
        try response.writeBody(json_response);
    }
    
    fn handleSignTransaction(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        const wallet_id = request.getParam("wallet_id");
        const body = try request.readBody(self.allocator);
        defer self.allocator.free(body);
        
        const sign_request = try std.json.parseFromSlice(SignTransactionRequest, self.allocator, body, .{});
        defer sign_request.deinit();
        
        // Get wallet from keystore
        const wallet = try self.key_store.getWallet(wallet_id);
        
        // Sign transaction
        const signature = try self.signTransactionWithWallet(wallet, sign_request.value);
        defer self.allocator.free(signature);
        
        const sign_response = SignTransactionResponse{
            .transaction_hash = sign_request.value.transaction_hash,
            .signature = signature,
            .signed_at = @intCast(std.time.milliTimestamp()),
        };
        
        const json_response = try std.json.stringifyAlloc(self.allocator, sign_response, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(json_response);
    }
    
    fn handleHealthCheck(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = request;
        
        const health_status = WalletHealthStatus{
            .status = "healthy",
            .wallets_loaded = self.key_store.getWalletCount(),
            .active_sessions = self.session_manager.getActiveSessionCount(),
            .keystore_encrypted = self.key_store.isEncrypted(),
        };
        
        const json_response = try std.json.stringifyAlloc(self.allocator, health_status, .{});
        defer self.allocator.free(json_response);
        
        response.setHeader("Content-Type", "application/json");
        try response.writeBody(json_response);
    }
    
    // Helper functions
    fn loadExistingWallets(self: *WalletDaemon) !void {
        try self.key_store.loadWallets();
        const wallet_count = self.key_store.getWalletCount();
        std.log.info("📁 Loaded {} wallets from keystore", .{wallet_count});
    }
    
    fn verifyCredentials(self: *WalletDaemon, username: []const u8, password: []const u8) !bool {
        // TODO: Implement proper credential verification
        _ = self;
        _ = username;
        _ = password;
        return true; // Placeholder
    }
    
    fn createNewWallet(self: *WalletDaemon, request: CreateWalletRequest) !Wallet {
        // Generate new key pair
        const keypair = try ghostchain.crypto.GhostCrypto.generateKeyPair(self.allocator);
        
        // Derive address from public key
        const address = try ghostchain.crypto.GhostCrypto.deriveGhostAddress(self.allocator, keypair.public_key);
        
        return Wallet{
            .id = try self.generateWalletId(),
            .name = request.name,
            .address = address,
            .public_key = keypair.public_key,
            .private_key = keypair.private_key,
            .key_type = keypair.key_type,
            .created_at = @intCast(std.time.milliTimestamp()),
            .accounts = std.ArrayList(Account).init(self.allocator),
        };
    }
    
    fn signTransactionWithWallet(self: *WalletDaemon, wallet: Wallet, request: SignTransactionRequest) ![]u8 {
        const tx_hash = try std.fmt.allocPrint(self.allocator, "{s}", .{request.transaction_hash});
        defer self.allocator.free(tx_hash);
        
        return ghostchain.crypto.GhostCrypto.signMessage(tx_hash, wallet.private_key, wallet.key_type);
    }
    
    fn generateWalletId(self: *WalletDaemon) ![]u8 {
        var random_bytes: [16]u8 = undefined;
        try ghostchain.crypto.GhostCrypto.generateSecureRandom(&random_bytes);
        return ghostcipher.util.hexEncode(self.allocator, &random_bytes);
    }
    
    // Placeholder handler implementations
    fn handleWalletById(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Wallet by ID endpoint coming soon\"}");
    }
    
    fn handleAccounts(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Accounts endpoint coming soon\"}");
    }
    
    fn handleAccountBalance(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"balance\": \"0\", \"status\": \"Balance endpoint coming soon\"}");
    }
    
    fn handleTransactions(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Transactions endpoint coming soon\"}");
    }
    
    fn handleGenerateKeys(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Key generation endpoint coming soon\"}");
    }
    
    fn handleImportKeys(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Key import endpoint coming soon\"}");
    }
    
    fn handleMultiSig(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Multi-signature endpoint coming soon\"}");
    }
    
    fn handleStatus(self: *WalletDaemon, request: *wraith.Request, response: *wraith.Response) !void {
        _ = self; _ = request;
        response.setHeader("Content-Type", "application/json");
        try response.writeBody("{\"status\": \"Status endpoint coming soon\"}");
    }
};

pub const WalletConfig = struct {
    bind_address: []const u8 = "::",
    quic_port: u16 = 9090,
    http_port: u16 = 3001,
    keystore_path: []const u8 = "./wallets",
    tls_cert_path: ?[]const u8 = null,
    tls_key_path: ?[]const u8 = null,
    session_timeout_minutes: u32 = 60,
    
    pub fn development() WalletConfig {
        return WalletConfig{
            .bind_address = "::1",
        };
    }
    
    pub fn production() WalletConfig {
        return WalletConfig{
            .session_timeout_minutes = 30,
        };
    }
};

// Data structures
const Wallet = struct {
    id: []const u8,
    name: []const u8,
    address: []const u8,
    public_key: []const u8,
    private_key: []const u8,
    key_type: ghostchain.crypto.KeyType,
    created_at: u64,
    accounts: std.ArrayList(Account),
    
    fn deinit(self: *Wallet, allocator: std.mem.Allocator) void {
        allocator.free(self.id);
        allocator.free(self.address);
        allocator.free(self.public_key);
        ghostcipher.util.secureZero(self.private_key);
        allocator.free(self.private_key);
        self.accounts.deinit();
    }
};

const Account = struct {
    id: []const u8,
    name: []const u8,
    address: []const u8,
    balance: u64,
};

const Session = struct {
    token: []const u8,
    username: []const u8,
    created_at: u64,
    expires_at: u64,
};

// Request/Response structures
const LoginRequest = struct {
    username: []const u8,
    password: []const u8,
};

const LoginResponse = struct {
    success: bool,
    session_token: []const u8,
    expires_at: u64,
    message: []const u8,
};

const CreateWalletRequest = struct {
    name: []const u8,
    password: []const u8,
    key_type: ?ghostchain.crypto.KeyType = .ed25519,
};

const WalletResponse = struct {
    id: []const u8,
    name: []const u8,
    address: []const u8,
    created_at: u64,
    accounts: std.ArrayList(Account),
};

const SignTransactionRequest = struct {
    transaction_hash: []const u8,
    account_id: []const u8,
};

const SignTransactionResponse = struct {
    transaction_hash: []const u8,
    signature: []const u8,
    signed_at: u64,
};

const WalletHealthStatus = struct {
    status: []const u8,
    wallets_loaded: u32,
    active_sessions: u32,
    keystore_encrypted: bool,
};

// Supporting components
const SecureKeyStore = struct {
    allocator: std.mem.Allocator,
    keystore_path: []const u8,
    wallets: std.HashMap([]const u8, Wallet, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    fn init(allocator: std.mem.Allocator, path: []const u8) !SecureKeyStore {
        return SecureKeyStore{
            .allocator = allocator,
            .keystore_path = path,
            .wallets = std.HashMap([]const u8, Wallet, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    fn deinit(self: *SecureKeyStore) void {
        var iterator = self.wallets.iterator();
        while (iterator.next()) |entry| {
            entry.value_ptr.deinit(self.allocator);
        }
        self.wallets.deinit();
    }
    
    fn loadWallets(self: *SecureKeyStore) !void {
        // TODO: Load encrypted wallets from disk
        _ = self;
    }
    
    fn storeWallet(self: *SecureKeyStore, wallet: Wallet) !void {
        // TODO: Store encrypted wallet to disk
        try self.wallets.put(wallet.id, wallet);
    }
    
    fn getWallet(self: *SecureKeyStore, wallet_id: []const u8) !Wallet {
        return self.wallets.get(wallet_id) orelse error.WalletNotFound;
    }
    
    fn listWallets(self: *SecureKeyStore) ![]WalletResponse {
        // TODO: Return wallet list without private keys
        _ = self;
        return &[_]WalletResponse{};
    }
    
    fn getWalletCount(self: *SecureKeyStore) u32 {
        return @intCast(self.wallets.count());
    }
    
    fn isEncrypted(self: *SecureKeyStore) bool {
        _ = self;
        return true; // TODO: Check if keystore is encrypted
    }
    
    fn secureCleanup(self: *SecureKeyStore) void {
        // TODO: Securely wipe sensitive data from memory
        _ = self;
    }
};

const SessionManager = struct {
    allocator: std.mem.Allocator,
    sessions: std.HashMap([]const u8, Session, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    fn init(allocator: std.mem.Allocator) SessionManager {
        return SessionManager{
            .allocator = allocator,
            .sessions = std.HashMap([]const u8, Session, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    fn deinit(self: *SessionManager) void {
        self.sessions.deinit();
    }
    
    fn createSession(self: *SessionManager, username: []const u8) !Session {
        var token_bytes: [32]u8 = undefined;
        try ghostchain.crypto.GhostCrypto.generateSecureRandom(&token_bytes);
        const token = try ghostcipher.util.hexEncode(self.allocator, &token_bytes);
        
        const now = @as(u64, @intCast(std.time.milliTimestamp()));
        const session = Session{
            .token = token,
            .username = username,
            .created_at = now,
            .expires_at = now + (60 * 60 * 1000), // 1 hour
        };
        
        try self.sessions.put(token, session);
        return session;
    }
    
    fn invalidateSession(self: *SessionManager, token: []const u8) void {
        _ = self.sessions.remove(token);
    }
    
    fn cleanupExpiredSessions(self: *SessionManager) void {
        const now = @as(u64, @intCast(std.time.milliTimestamp()));
        var iterator = self.sessions.iterator();
        var expired_tokens = std.ArrayList([]const u8).init(self.allocator);
        defer expired_tokens.deinit();
        
        while (iterator.next()) |entry| {
            if (entry.value_ptr.expires_at < now) {
                expired_tokens.append(entry.key_ptr.*) catch continue;
            }
        }
        
        for (expired_tokens.items) |token| {
            _ = self.sessions.remove(token);
        }
    }
    
    fn getActiveSessionCount(self: *SessionManager) u32 {
        return @intCast(self.sessions.count());
    }
};