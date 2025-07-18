const std = @import("std");
const shroud = @import("shroud");
const ghostcipher = shroud.ghostcipher;
const sigil = shroud.sigil;

pub const RealIDSystem = struct {
    allocator: std.mem.Allocator,
    identities: std.HashMap([]const u8, Identity, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    did_registry: DIDRegistry,
    
    pub fn init(allocator: std.mem.Allocator) RealIDSystem {
        return RealIDSystem{
            .allocator = allocator,
            .identities = std.HashMap([]const u8, Identity, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .did_registry = DIDRegistry.init(allocator),
        };
    }
    
    pub fn deinit(self: *RealIDSystem) void {
        var iterator = self.identities.iterator();
        while (iterator.next()) |entry| {
            entry.value_ptr.deinit(self.allocator);
        }
        self.identities.deinit();
        self.did_registry.deinit();
    }
    
    pub fn createIdentity(self: *RealIDSystem, username: []const u8, public_key: []const u8) !Identity {
        const did = try self.generateDID(username);
        
        const identity = Identity{
            .did = did,
            .username = username,
            .public_key = public_key,
            .verified = false,
            .created_at = @intCast(std.time.milliTimestamp()),
            .attributes = std.ArrayList(IdentityAttribute).init(self.allocator),
            .credentials = std.ArrayList(VerifiableCredential).init(self.allocator),
        };
        
        try self.identities.put(did, identity);
        try self.did_registry.register(did, public_key);
        
        return identity;
    }
    
    pub fn getIdentity(self: *RealIDSystem, did: []const u8) ?*Identity {
        return self.identities.getPtr(did);
    }
    
    pub fn verifyIdentity(self: *RealIDSystem, did: []const u8, proof: IdentityProof) !bool {
        const identity = self.getIdentity(did) orelse return false;
        
        // Verify signature with identity's public key
        const message = try self.createVerificationMessage(identity, proof.challenge);
        defer self.allocator.free(message);
        
        const is_valid = try sigil.verify(identity.public_key, message, proof.signature);
        
        if (is_valid) {
            identity.verified = true;
        }
        
        return is_valid;
    }
    
    pub fn issueCredential(
        self: *RealIDSystem,
        issuer_did: []const u8,
        subject_did: []const u8,
        credential_type: CredentialType,
        claims: []const Claim,
    ) !VerifiableCredential {
        const issuer = self.getIdentity(issuer_did) orelse return error.IssuerNotFound;
        const subject_identity = self.getIdentity(subject_did) orelse return error.SubjectNotFound;
        
        if (!issuer.verified) return error.IssuerNotVerified;
        
        const credential = VerifiableCredential{
            .id = try self.generateCredentialId(),
            .issuer = issuer_did,
            .subject = subject_did,
            .credential_type = credential_type,
            .claims = claims,
            .issued_at = @intCast(std.time.milliTimestamp()),
            .expires_at = null,
            .revoked = false,
        };
        
        // Add credential to subject's identity
        var subject_mut = self.identities.getPtr(subject_did).?;
        try subject_mut.credentials.append(credential);
        _ = subject_identity;
        
        return credential;
    }
    
    pub fn revokeCredential(self: *RealIDSystem, credential_id: []const u8, issuer_did: []const u8) !void {
        // Find and revoke credential
        var iterator = self.identities.iterator();
        while (iterator.next()) |entry| {
            const identity = entry.value_ptr;
            for (identity.credentials.items) |*credential| {
                if (std.mem.eql(u8, credential.id, credential_id) and
                    std.mem.eql(u8, credential.issuer, issuer_did)) {
                    credential.revoked = true;
                    return;
                }
            }
        }
        return error.CredentialNotFound;
    }
    
    // Private helper functions
    fn generateDID(self: *RealIDSystem, username: []const u8) ![]const u8 {
        const did_prefix = "did:ghost:";
        const did = try self.allocator.alloc(u8, did_prefix.len + username.len);
        @memcpy(did[0..did_prefix.len], did_prefix);
        @memcpy(did[did_prefix.len..], username);
        return did;
    }
    
    fn generateCredentialId(self: *RealIDSystem) ![]const u8 {
        var random_bytes: [16]u8 = undefined;
        try ghostcipher.rand.fillBytes(&random_bytes);
        return ghostcipher.util.hexEncode(self.allocator, &random_bytes);
    }
    
    fn createVerificationMessage(self: *RealIDSystem, identity: *const Identity, challenge: []const u8) ![]const u8 {
        const message_parts = [_][]const u8{ identity.did, identity.username, challenge };
        return std.mem.join(self.allocator, ":", &message_parts);
    }
};

pub const Identity = struct {
    did: []const u8,
    username: []const u8,
    public_key: []const u8,
    verified: bool,
    created_at: u64,
    attributes: std.ArrayList(IdentityAttribute),
    credentials: std.ArrayList(VerifiableCredential),
    
    pub fn deinit(self: *Identity, allocator: std.mem.Allocator) void {
        allocator.free(self.did);
        self.attributes.deinit();
        for (self.credentials.items) |credential| {
            allocator.free(credential.id);
        }
        self.credentials.deinit();
    }
    
    pub fn addAttribute(self: *Identity, attribute: IdentityAttribute) !void {
        try self.attributes.append(attribute);
    }
    
    pub fn getAttribute(self: *const Identity, name: []const u8) ?IdentityAttribute {
        for (self.attributes.items) |attr| {
            if (std.mem.eql(u8, attr.name, name)) {
                return attr;
            }
        }
        return null;
    }
};

pub const IdentityAttribute = struct {
    name: []const u8,
    value: []const u8,
    verified: bool,
    verifier: ?[]const u8,
};

pub const VerifiableCredential = struct {
    id: []const u8,
    issuer: []const u8,
    subject: []const u8,
    credential_type: CredentialType,
    claims: []const Claim,
    issued_at: u64,
    expires_at: ?u64,
    revoked: bool,
};

pub const CredentialType = enum {
    IDENTITY_VERIFICATION,
    AGE_VERIFICATION,
    ADDRESS_VERIFICATION,
    KYC_CREDENTIAL,
    CUSTOM,
};

pub const Claim = struct {
    key: []const u8,
    value: []const u8,
    confidence: f32, // 0.0 to 1.0
};

pub const IdentityProof = struct {
    challenge: []const u8,
    signature: []const u8,
    timestamp: u64,
};

pub const DIDRegistry = struct {
    allocator: std.mem.Allocator,
    registrations: std.HashMap([]const u8, DIDRegistration, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    
    pub fn init(allocator: std.mem.Allocator) DIDRegistry {
        return DIDRegistry{
            .allocator = allocator,
            .registrations = std.HashMap([]const u8, DIDRegistration, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }
    
    pub fn deinit(self: *DIDRegistry) void {
        self.registrations.deinit();
    }
    
    pub fn register(self: *DIDRegistry, did: []const u8, public_key: []const u8) !void {
        const registration = DIDRegistration{
            .did = did,
            .public_key = public_key,
            .registered_at = @intCast(std.time.milliTimestamp()),
            .active = true,
        };
        
        try self.registrations.put(did, registration);
    }
    
    pub fn resolve(self: *DIDRegistry, did: []const u8) ?DIDRegistration {
        return self.registrations.get(did);
    }
    
    pub fn deactivate(self: *DIDRegistry, did: []const u8) !void {
        if (self.registrations.getPtr(did)) |registration| {
            registration.active = false;
        } else {
            return error.DIDNotFound;
        }
    }
};

pub const DIDRegistration = struct {
    did: []const u8,
    public_key: []const u8,
    registered_at: u64,
    active: bool,
};