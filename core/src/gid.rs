// GID (Ghost Identity) - Decentralized identity layer for GhostChain
//
// Ghost Identity provides zero-trust, DID-compatible identity management
// with native integration to CNS domains and the 4-token economy

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;

/// GID - Ghost Identity identifier following DID spec
/// Format: did:ghost:{identifier}
/// Examples: did:ghost:alice, did:ghost:0x1234...
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GID {
    pub did: String,
    pub method: String,  // "ghost"
    pub identifier: String,
}

impl GID {
    /// Create a new Ghost Identity
    pub fn new(identifier: impl Into<String>) -> Self {
        let id = identifier.into();
        Self {
            did: format!("did:ghost:{}", id),
            method: "ghost".to_string(),
            identifier: id,
        }
    }

    /// Parse a DID string into a GID
    pub fn parse(did: &str) -> Result<Self> {
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() != 3 || parts[0] != "did" || parts[1] != "ghost" {
            return Err(anyhow!("Invalid GID format. Expected: did:ghost:identifier"));
        }

        Ok(Self {
            did: did.to_string(),
            method: parts[1].to_string(),
            identifier: parts[2].to_string(),
        })
    }

    /// Check if this is a valid Ghost Identity
    pub fn is_valid(&self) -> bool {
        self.method == "ghost" && !self.identifier.is_empty()
    }

    /// Convert to standard DID string
    pub fn to_did_string(&self) -> String {
        self.did.clone()
    }
}

/// Ghost Identity Document - contains identity metadata and verification info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIDDocument {
    pub id: GID,
    pub public_keys: Vec<PublicKeyEntry>,
    pub authentication: Vec<String>,
    pub service_endpoints: Vec<ServiceEndpoint>,
    pub permissions: PermissionSet,
    pub cns_domains: Vec<String>,  // Associated CNS domains
    pub token_balances: TokenBalances,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub recovery_keys: Vec<String>,
    pub metadata: GIDMetadata,
}

/// Public key entry for GID verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyEntry {
    pub id: String,
    pub key_type: KeyType,
    pub public_key: String,
    pub purpose: Vec<KeyPurpose>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
    BLS,
    PostQuantum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyPurpose {
    Authentication,
    Signing,
    KeyAgreement,
    Recovery,
}

/// Service endpoints for GID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub endpoint: String,
    pub priority: u32,
}

/// Permissions for GID operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }

    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Create default permissions for regular users
    pub fn default_user() -> Self {
        let mut set = Self::new();
        set.add(Permission::Send);
        set.add(Permission::Receive);
        set.add(Permission::RegisterDomain);
        set
    }

    /// Create admin permissions
    pub fn admin() -> Self {
        let mut set = Self::new();
        set.add(Permission::Send);
        set.add(Permission::Receive);
        set.add(Permission::RegisterDomain);
        set.add(Permission::CreateIdentities);
        set.add(Permission::ManagePermissions);
        set.add(Permission::ViewAudit);
        set.add(Permission::CreateContracts);
        set.add(Permission::ExecuteContracts);
        set
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Send,
    Receive,
    RegisterDomain,
    CreateIdentities,
    ManagePermissions,
    ViewAudit,
    CreateContracts,
    ExecuteContracts,
    MintTokens,
    BurnTokens,
}

/// Token balances associated with a GID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalances {
    pub gcc: u64,    // Gas currency
    pub spirit: u64, // Governance token
    pub mana: u64,   // Utility token
    pub ghost: u64,  // Brand/collectible token
}

impl Default for TokenBalances {
    fn default() -> Self {
        Self {
            gcc: 0,
            spirit: 0,
            mana: 0,
            ghost: 0,
        }
    }
}

/// Metadata for Ghost Identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIDMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
    pub tags: HashSet<String>,
    pub soulbound: bool,  // If true, identity cannot be transferred
}

impl Default for GIDMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            avatar: None,
            website: None,
            social_links: HashMap::new(),
            tags: HashSet::new(),
            soulbound: true, // Default to soulbound for security
        }
    }
}

/// Access token for GID authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIDAccessToken {
    pub token_id: String,
    pub issuer: GID,
    pub subject: GID,
    pub permissions: PermissionSet,
    pub scope: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: String,
}

impl GIDAccessToken {
    /// Check if token is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// Verify token has required permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.has(permission)
    }

    /// Check if token grants access to scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scope.iter().any(|s| s == scope)
    }
}

/// GID Registry - manages all Ghost Identities
pub struct GIDRegistry {
    identities: Arc<RwLock<HashMap<String, GIDDocument>>>,
    name_registry: Arc<RwLock<HashMap<String, String>>>, // name -> GID mapping
    domain_registry: Arc<RwLock<HashMap<String, String>>>, // domain -> GID mapping
}

impl GIDRegistry {
    pub fn new() -> Self {
        Self {
            identities: Arc::new(RwLock::new(HashMap::new())),
            name_registry: Arc::new(RwLock::new(HashMap::new())),
            domain_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new Ghost Identity
    pub async fn register(&self, identifier: &str, public_key: PublicKeyEntry, metadata: GIDMetadata) -> Result<GID> {
        let gid = GID::new(identifier);

        // Check if already exists
        let identities = self.identities.read().await;
        if identities.contains_key(&gid.did) {
            return Err(anyhow!("GID already exists: {}", gid.did));
        }
        drop(identities);

        // Create GID document
        let now = Utc::now();
        let doc = GIDDocument {
            id: gid.clone(),
            public_keys: vec![public_key],
            authentication: vec![format!("{}#key-1", gid.did)],
            service_endpoints: vec![],
            permissions: PermissionSet::default_user(),
            cns_domains: vec![],
            token_balances: TokenBalances::default(),
            created_at: now,
            updated_at: now,
            recovery_keys: vec![],
            metadata,
        };

        // Store identity
        let mut identities = self.identities.write().await;
        identities.insert(gid.did.clone(), doc.clone());

        // Register name if provided
        if let Some(ref name) = doc.metadata.name {
            let mut name_registry = self.name_registry.write().await;
            name_registry.insert(name.clone(), gid.did.clone());
        }

        Ok(gid)
    }

    /// Resolve a GID to its document
    pub async fn resolve(&self, gid: &GID) -> Result<GIDDocument> {
        let identities = self.identities.read().await;
        identities
            .get(&gid.did)
            .cloned()
            .ok_or_else(|| anyhow!("GID not found: {}", gid.did))
    }

    /// Resolve by name
    pub async fn resolve_by_name(&self, name: &str) -> Result<GIDDocument> {
        let name_registry = self.name_registry.read().await;
        let did = name_registry
            .get(name)
            .ok_or_else(|| anyhow!("Name not registered: {}", name))?;

        let identities = self.identities.read().await;
        identities
            .get(did)
            .cloned()
            .ok_or_else(|| anyhow!("GID document not found for name: {}", name))
    }

    /// Link a CNS domain to a GID
    pub async fn link_domain(&self, gid: &GID, domain: &str) -> Result<()> {
        let mut identities = self.identities.write().await;
        let doc = identities
            .get_mut(&gid.did)
            .ok_or_else(|| anyhow!("GID not found: {}", gid.did))?;

        if !doc.cns_domains.contains(&domain.to_string()) {
            doc.cns_domains.push(domain.to_string());
            doc.updated_at = Utc::now();
        }

        // Update domain registry
        drop(identities);
        let mut domain_registry = self.domain_registry.write().await;
        domain_registry.insert(domain.to_string(), gid.did.clone());

        Ok(())
    }

    /// Grant permission to a GID
    pub async fn grant_permission(&self, gid: &GID, permission: Permission) -> Result<()> {
        let mut identities = self.identities.write().await;
        let doc = identities
            .get_mut(&gid.did)
            .ok_or_else(|| anyhow!("GID not found: {}", gid.did))?;

        doc.permissions.add(permission);
        doc.updated_at = Utc::now();

        Ok(())
    }

    /// Update token balance for a GID
    pub async fn update_balance(&self, gid: &GID, token: TokenType, amount: u64) -> Result<()> {
        let mut identities = self.identities.write().await;
        let doc = identities
            .get_mut(&gid.did)
            .ok_or_else(|| anyhow!("GID not found: {}", gid.did))?;

        match token {
            TokenType::GCC => doc.token_balances.gcc = amount,
            TokenType::SPIRIT => doc.token_balances.spirit = amount,
            TokenType::MANA => doc.token_balances.mana = amount,
            TokenType::GHOST => doc.token_balances.ghost = amount,
        }

        doc.updated_at = Utc::now();
        Ok(())
    }

    /// Create an access token for a GID
    pub async fn create_access_token(
        &self,
        issuer: &GID,
        subject: &GID,
        permissions: PermissionSet,
        scope: Vec<String>,
        duration_seconds: i64,
    ) -> Result<GIDAccessToken> {
        // Verify issuer exists and has permission to create tokens
        let issuer_doc = self.resolve(issuer).await?;
        if !issuer_doc.permissions.has(&Permission::CreateIdentities) {
            return Err(anyhow!("Issuer lacks permission to create access tokens"));
        }

        // Verify subject exists
        let _ = self.resolve(subject).await?;

        let now = Utc::now();
        let token = GIDAccessToken {
            token_id: format!("gid-token-{}", uuid::Uuid::new_v4()),
            issuer: issuer.clone(),
            subject: subject.clone(),
            permissions,
            scope,
            issued_at: now,
            expires_at: now + chrono::Duration::seconds(duration_seconds),
            signature: "mock-signature".to_string(), // TODO: Implement actual signing
        };

        Ok(token)
    }
}

/// Token types in the GhostChain ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    GCC,    // Gas & transaction fees
    SPIRIT, // Governance & voting
    MANA,   // Utility & rewards
    GHOST,  // Brand & collectibles
}

/// GID Verifier - verifies identities and signatures
pub struct GIDVerifier {
    registry: Arc<GIDRegistry>,
}

impl GIDVerifier {
    pub fn new(registry: Arc<GIDRegistry>) -> Self {
        Self { registry }
    }

    /// Verify a GID signature
    pub async fn verify_signature(&self, gid: &GID, signature: &[u8], message: &[u8]) -> Result<bool> {
        let doc = self.registry.resolve(gid).await?;

        // TODO: Implement actual signature verification using public keys
        // For now, return mock verification
        let _ = (signature, message, doc);
        Ok(true)
    }

    /// Verify an access token
    pub async fn verify_token(&self, token: &GIDAccessToken) -> Result<bool> {
        // Check token validity
        if !token.is_valid() {
            return Ok(false);
        }

        // Verify issuer exists
        let _ = self.registry.resolve(&token.issuer).await?;

        // Verify subject exists
        let _ = self.registry.resolve(&token.subject).await?;

        // TODO: Verify token signature
        Ok(true)
    }

    /// Verify domain ownership
    pub async fn verify_domain_ownership(&self, gid: &GID, domain: &str) -> Result<bool> {
        let doc = self.registry.resolve(gid).await?;
        Ok(doc.cns_domains.contains(&domain.to_string()))
    }

    /// Verify permission
    pub async fn verify_permission(&self, gid: &GID, permission: &Permission) -> Result<bool> {
        let doc = self.registry.resolve(gid).await?;
        Ok(doc.permissions.has(permission))
    }
}

impl Default for GIDRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gid_creation() {
        let gid = GID::new("alice");
        assert_eq!(gid.did, "did:ghost:alice");
        assert_eq!(gid.method, "ghost");
        assert_eq!(gid.identifier, "alice");
        assert!(gid.is_valid());
    }

    #[test]
    fn test_gid_parsing() {
        let gid = GID::parse("did:ghost:bob").unwrap();
        assert_eq!(gid.identifier, "bob");
        assert!(gid.is_valid());

        // Invalid format
        assert!(GID::parse("did:eth:alice").is_err());
        assert!(GID::parse("invalid").is_err());
    }

    #[tokio::test]
    async fn test_gid_registry() {
        let registry = GIDRegistry::new();

        // Create public key
        let public_key = PublicKeyEntry {
            id: "key-1".to_string(),
            key_type: KeyType::Ed25519,
            public_key: "ed25519:abc123".to_string(),
            purpose: vec![KeyPurpose::Authentication, KeyPurpose::Signing],
        };

        // Create metadata
        let mut metadata = GIDMetadata::default();
        metadata.name = Some("Alice".to_string());

        // Register GID
        let gid = registry.register("alice", public_key, metadata).await.unwrap();
        assert_eq!(gid.identifier, "alice");

        // Resolve by GID
        let doc = registry.resolve(&gid).await.unwrap();
        assert_eq!(doc.id.identifier, "alice");
        assert_eq!(doc.metadata.name, Some("Alice".to_string()));

        // Resolve by name
        let doc_by_name = registry.resolve_by_name("Alice").await.unwrap();
        assert_eq!(doc_by_name.id.identifier, "alice");

        // Link domain
        registry.link_domain(&gid, "alice.ghost").await.unwrap();
        let updated_doc = registry.resolve(&gid).await.unwrap();
        assert!(updated_doc.cns_domains.contains(&"alice.ghost".to_string()));
    }

    #[tokio::test]
    async fn test_permissions() {
        let registry = GIDRegistry::new();
        let verifier = GIDVerifier::new(Arc::new(registry));

        // Test permission sets
        let user_perms = PermissionSet::default_user();
        assert!(user_perms.has(&Permission::Send));
        assert!(user_perms.has(&Permission::Receive));
        assert!(!user_perms.has(&Permission::MintTokens));

        let admin_perms = PermissionSet::admin();
        assert!(admin_perms.has(&Permission::CreateContracts));
        assert!(admin_perms.has(&Permission::ManagePermissions));
    }

    #[test]
    fn test_access_token_validity() {
        let issuer = GID::new("issuer");
        let subject = GID::new("subject");

        let token = GIDAccessToken {
            token_id: "test-token".to_string(),
            issuer,
            subject,
            permissions: PermissionSet::default_user(),
            scope: vec!["read".to_string(), "write".to_string()],
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            signature: "test-sig".to_string(),
        };

        assert!(token.is_valid());
        assert!(token.has_scope("read"));
        assert!(token.has_scope("write"));
        assert!(!token.has_scope("admin"));
        assert!(token.has_permission(&Permission::Send));
        assert!(!token.has_permission(&Permission::MintTokens));
    }
}