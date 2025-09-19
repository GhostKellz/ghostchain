// CNS (Crypto Name Service) - Multi-domain resolution system
//
// Supports native GhostChain domains (.ghost, .gcc, .warp, .arc, .gcp)
// Bridges to external systems (ENS, Unstoppable Domains, Web5 DIDs)
// Provides unified resolution API for all domain types

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use ghostchain_shared::types::Address;

/// Supported domain types in the CNS system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainType {
    // Native GhostChain domains
    Ghost,    // .ghost
    GCC,      // .gcc
    Warp,     // .warp
    Arc,      // .arc
    GCP,      // .gcp

    // External domain bridges
    ENS,      // .eth
    Unstoppable, // .crypto, .nft, .x, etc.
    Web5,     // did: identifiers

    // Future extensions
    Custom(String),
}

/// Token types in the GhostChain ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    GCC,    // Gas & transaction fees
    SPIRIT, // Governance & voting
    MANA,   // Utility & rewards
    GHOST,  // Brand & collectibles
}

/// Domain resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResolution {
    pub domain: String,
    pub owner: Address,
    pub domain_type: DomainType,
    pub records: BTreeMap<String, String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub registered_at: DateTime<Utc>,
    pub token_type: TokenType,
    pub service_endpoints: Vec<ServiceEndpoint>,
    pub metadata: DomainMetadata,
}

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_type: ServiceType,
    pub endpoint: String,
    pub protocol: String,
    pub priority: u32,
}

/// Service types supported by CNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    Blockchain,
    Wallet,
    Storage,
    Web5Proxy,
    L2,
    Custom(String),
}

/// Domain metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainMetadata {
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
    pub tags: HashSet<String>,
}

/// Domain registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistration {
    pub domain: String,
    pub owner: Address,
    pub duration_years: u32,
    pub records: BTreeMap<String, String>,
    pub metadata: DomainMetadata,
}

/// CNS resolver error types
#[derive(Debug, thiserror::Error)]
pub enum CNSError {
    #[error("Domain not found: {domain}")]
    DomainNotFound { domain: String },

    #[error("Unsupported TLD: {tld}")]
    UnsupportedTLD { tld: String },

    #[error("Domain expired: {domain}")]
    DomainExpired { domain: String },

    #[error("Insufficient payment for domain: {domain}")]
    InsufficientPayment { domain: String },

    #[error("Bridge error: {source}")]
    BridgeError { source: String },

    #[error("Cache error: {source}")]
    CacheError { source: String },
}

/// Domain cache for performance optimization
pub struct DomainCache {
    cache: RwLock<HashMap<String, (DomainResolution, DateTime<Utc>)>>,
    ttl_seconds: u64,
}

impl DomainCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl_seconds,
        }
    }

    pub async fn get(&self, domain: &str) -> Option<DomainResolution> {
        let cache = self.cache.read().await;
        if let Some((resolution, cached_at)) = cache.get(domain) {
            let now = Utc::now();
            if now.signed_duration_since(*cached_at).num_seconds() < self.ttl_seconds as i64 {
                return Some(resolution.clone());
            }
        }
        None
    }

    pub async fn insert(&self, domain: &str, resolution: &DomainResolution) {
        let mut cache = self.cache.write().await;
        cache.insert(domain.to_string(), (resolution.clone(), Utc::now()));
    }

    pub async fn invalidate(&self, domain: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(domain);
    }
}

/// Native GhostChain domain resolver
pub struct GhostNativeResolver {
    domains: RwLock<BTreeMap<String, DomainResolution>>,
}

impl GhostNativeResolver {
    pub fn new() -> Self {
        Self {
            domains: RwLock::new(BTreeMap::new()),
        }
    }

    pub async fn resolve(&self, domain: &str) -> Result<DomainResolution> {
        let domains = self.domains.read().await;
        domains.get(domain)
            .cloned()
            .ok_or_else(|| CNSError::DomainNotFound { domain: domain.to_string() }.into())
    }

    pub async fn register_domain(&self, registration: DomainRegistration) -> Result<()> {
        let mut domains = self.domains.write().await;

        // Check if domain already exists
        if domains.contains_key(&registration.domain) {
            return Err(anyhow!("Domain already exists: {}", registration.domain));
        }

        let domain_type = Self::get_domain_type(&registration.domain)?;
        let token_type = Self::get_token_type_for_domain(&domain_type);

        let resolution = DomainResolution {
            domain: registration.domain.clone(),
            owner: registration.owner,
            domain_type,
            records: registration.records,
            expires_at: Some(Utc::now() + chrono::Duration::days(365 * registration.duration_years as i64)),
            registered_at: Utc::now(),
            token_type,
            service_endpoints: vec![],
            metadata: registration.metadata,
        };

        domains.insert(registration.domain, resolution);
        Ok(())
    }

    fn get_domain_type(domain: &str) -> Result<DomainType> {
        if domain.ends_with(".ghost") {
            Ok(DomainType::Ghost)
        } else if domain.ends_with(".gcc") {
            Ok(DomainType::GCC)
        } else if domain.ends_with(".warp") {
            Ok(DomainType::Warp)
        } else if domain.ends_with(".arc") {
            Ok(DomainType::Arc)
        } else if domain.ends_with(".gcp") {
            Ok(DomainType::GCP)
        } else {
            Err(CNSError::UnsupportedTLD {
                tld: domain.split('.').last().unwrap_or("unknown").to_string()
            }.into())
        }
    }

    fn get_token_type_for_domain(domain_type: &DomainType) -> TokenType {
        match domain_type {
            DomainType::Ghost => TokenType::GHOST,
            DomainType::GCC => TokenType::GCC,
            DomainType::Warp => TokenType::MANA,
            DomainType::Arc => TokenType::SPIRIT,
            DomainType::GCP => TokenType::GCC,
            _ => TokenType::GCC, // Default to GCC
        }
    }
}

/// ENS bridge resolver for .eth domains
pub struct ENSBridgeResolver {
    // TODO: Implement Ethereum bridge
}

impl ENSBridgeResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve(&self, _domain: &str) -> Result<DomainResolution> {
        // TODO: Implement ENS resolution via Ethereum bridge
        Err(anyhow!("ENS bridge not yet implemented"))
    }
}

/// Unstoppable Domains bridge resolver
pub struct UnstoppableBridge {
    // TODO: Implement Unstoppable Domains bridge
}

impl UnstoppableBridge {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve(&self, _domain: &str) -> Result<DomainResolution> {
        // TODO: Implement Unstoppable Domains resolution
        Err(anyhow!("Unstoppable Domains bridge not yet implemented"))
    }
}

/// Web5 DID resolver
pub struct Web5DIDResolver {
    // TODO: Implement Web5 DID resolution
}

impl Web5DIDResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve(&self, _did: &str) -> Result<DomainResolution> {
        // TODO: Implement Web5 DID resolution
        Err(anyhow!("Web5 DID resolver not yet implemented"))
    }
}

/// Main CNS resolver combining all resolution methods
pub struct CNSResolver {
    native_resolver: GhostNativeResolver,
    ens_bridge: ENSBridgeResolver,
    unstoppable_bridge: UnstoppableBridge,
    web5_resolver: Web5DIDResolver,
    cache: DomainCache,
}

impl CNSResolver {
    pub fn new() -> Self {
        Self {
            native_resolver: GhostNativeResolver::new(),
            ens_bridge: ENSBridgeResolver::new(),
            unstoppable_bridge: UnstoppableBridge::new(),
            web5_resolver: Web5DIDResolver::new(),
            cache: DomainCache::new(300), // 5 minute cache TTL
        }
    }

    /// Resolve a domain through the appropriate resolver
    pub async fn resolve_domain(&self, domain: &str) -> Result<DomainResolution> {
        // Check cache first
        if let Some(cached) = self.cache.get(domain).await {
            return Ok(cached);
        }

        // Route to appropriate resolver based on domain/identifier type
        let result = if domain.starts_with("did:") {
            self.web5_resolver.resolve(domain).await
        } else if domain.ends_with(".eth") {
            self.ens_bridge.resolve(domain).await
        } else if domain.ends_with(".crypto") || domain.ends_with(".nft") || domain.ends_with(".x") {
            self.unstoppable_bridge.resolve(domain).await
        } else if domain.ends_with(".ghost") || domain.ends_with(".gcc") ||
                  domain.ends_with(".warp") || domain.ends_with(".arc") ||
                  domain.ends_with(".gcp") {
            self.native_resolver.resolve(domain).await
        } else {
            Err(CNSError::UnsupportedTLD {
                tld: domain.split('.').last().unwrap_or("unknown").to_string()
            }.into())
        };

        // Cache successful results
        if let Ok(ref resolution) = result {
            self.cache.insert(domain, resolution).await;
        }

        result
    }

    /// Register a new domain
    pub async fn register_domain(&self, registration: DomainRegistration) -> Result<()> {
        // Only handle native GhostChain domains for registration
        if registration.domain.ends_with(".ghost") || registration.domain.ends_with(".gcc") ||
           registration.domain.ends_with(".warp") || registration.domain.ends_with(".arc") ||
           registration.domain.ends_with(".gcp") {
            self.native_resolver.register_domain(registration).await
        } else {
            Err(anyhow!("Can only register native GhostChain domains"))
        }
    }

    /// Update domain records
    pub async fn update_domain_records(&self, domain: &str, records: BTreeMap<String, String>) -> Result<()> {
        // TODO: Implement domain record updates
        // This would involve checking ownership and updating the domain resolution
        let _ = (domain, records);
        Err(anyhow!("Domain record updates not yet implemented"))
    }

    /// Transfer domain ownership
    pub async fn transfer_domain(&self, domain: &str, new_owner: Address) -> Result<()> {
        // TODO: Implement domain ownership transfers
        // This would involve checking current ownership and transferring to new owner
        let _ = (domain, new_owner);
        Err(anyhow!("Domain transfers not yet implemented"))
    }

    /// Get all domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: Address) -> Result<Vec<String>> {
        // TODO: Implement owner domain lookup
        let _ = owner;
        Err(anyhow!("Owner domain lookup not yet implemented"))
    }
}

impl Default for CNSResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// CNS daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CNSConfig {
    pub bind_address: String,
    pub rpc_port: u16,
    pub dns_port: u16,
    pub cache_ttl_seconds: u64,
    pub enable_ens_bridge: bool,
    pub enable_unstoppable_bridge: bool,
    pub enable_web5_resolver: bool,
}

impl Default for CNSConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            rpc_port: 8553,
            dns_port: 53,
            cache_ttl_seconds: 300,
            enable_ens_bridge: false,
            enable_unstoppable_bridge: false,
            enable_web5_resolver: false,
        }
    }
}

/// CNS daemon service
pub struct CNSDaemon {
    resolver: CNSResolver,
    config: CNSConfig,
}

impl CNSDaemon {
    pub fn new(config: CNSConfig) -> Self {
        Self {
            resolver: CNSResolver::new(),
            config,
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting CNS daemon on {}:{}", self.config.bind_address, self.config.rpc_port);

        // TODO: Start RPC server
        // TODO: Start DNS server
        // TODO: Start gRPC server

        Ok(())
    }

    pub fn resolver(&self) -> &CNSResolver {
        &self.resolver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_domain_cache() {
        let cache = DomainCache::new(60);

        // Cache should be empty initially
        assert!(cache.get("test.ghost").await.is_none());

        // Insert a test resolution
        let resolution = DomainResolution {
            domain: "test.ghost".to_string(),
            owner: Address::default(),
            domain_type: DomainType::Ghost,
            records: BTreeMap::new(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            registered_at: Utc::now(),
            token_type: TokenType::GHOST,
            service_endpoints: vec![],
            metadata: DomainMetadata {
                description: None,
                avatar: None,
                website: None,
                social_links: HashMap::new(),
                tags: HashSet::new(),
            },
        };

        cache.insert("test.ghost", &resolution).await;

        // Should retrieve cached value
        let cached = cache.get("test.ghost").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().domain, "test.ghost");
    }

    #[tokio::test]
    async fn test_native_resolver_domain_types() {
        assert!(matches!(
            GhostNativeResolver::get_domain_type("example.ghost"),
            Ok(DomainType::Ghost)
        ));

        assert!(matches!(
            GhostNativeResolver::get_domain_type("example.gcc"),
            Ok(DomainType::GCC)
        ));

        assert!(matches!(
            GhostNativeResolver::get_domain_type("example.warp"),
            Ok(DomainType::Warp)
        ));
    }
}