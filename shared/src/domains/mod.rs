use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use crate::types::*;

pub mod ens;
pub mod unstoppable;
pub mod web5;
pub mod ghost;
pub mod domain_testing;

/// Multi-domain resolver supporting ENS, Unstoppable Domains, Web5, and native .ghost domains
pub struct MultiDomainResolver {
    pub ens_resolver: ens::ENSResolver,
    pub unstoppable_resolver: unstoppable::UnstoppableResolver,
    pub web5_resolver: web5::Web5Resolver,
    pub ghost_resolver: ghost::GhostDomainResolver,
    pub cache: HashMap<String, CachedDomainResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDomainResult {
    pub domain: String,
    pub result: DomainResolutionResult,
    pub cached_at: u64,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResolutionResult {
    pub domain: String,
    pub domain_type: DomainType,
    pub resolved_address: Option<Address>,
    pub identity: Option<Web5Identity>,
    pub records: Vec<DomainRecord>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainType {
    ENS,           // .eth domains
    Unstoppable,   // .crypto, .nft, .blockchain, .888, .wallet, .x, .klever, .hi, .kresus, .polygon
    Web5,          // did: identities
    Ghost,         // .ghost, .gcc, .sig, .gpk, .key, .pin, .sid, .dvm, .tmp, .dbg, .lib, .txo
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web5Identity {
    pub did: String,
    pub public_keys: Vec<PublicKeyInfo>,
    pub services: Vec<ServiceEndpoint>,
    pub verification_methods: Vec<VerificationMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub id: String,
    pub key_type: String,
    pub public_key_hex: String,
    pub purposes: Vec<String>, // authentication, assertion, key_agreement, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub method_type: String,
    pub controller: String,
    pub public_key: String,
}

impl MultiDomainResolver {
    pub fn new() -> Self {
        Self {
            ens_resolver: ens::ENSResolver::new(),
            unstoppable_resolver: unstoppable::UnstoppableResolver::new(),
            web5_resolver: web5::Web5Resolver::new(),
            ghost_resolver: ghost::GhostDomainResolver::new(),
            cache: HashMap::new(),
        }
    }

    pub fn get_ghost_resolver(&mut self) -> &mut ghost::GhostDomainResolver {
        &mut self.ghost_resolver
    }

    /// Wrapper for backward compatibility
    pub async fn resolve(&mut self, domain: &str) -> Result<DomainResolutionResult> {
        self.resolve_domain(domain).await
    }

    /// Resolve any domain across all supported systems
    pub async fn resolve_domain(&mut self, domain: &str) -> Result<DomainResolutionResult> {
        // Check cache first
        if let Some(cached) = self.get_cached(domain) {
            return Ok(cached.result.clone());
        }

        let domain_type = self.detect_domain_type(domain);
        
        let result = match domain_type {
            DomainType::ENS => {
                self.resolve_ens_domain(domain).await?
            },
            DomainType::Unstoppable => {
                self.resolve_unstoppable_domain(domain).await?
            },
            DomainType::Web5 => {
                self.resolve_web5_identity(domain).await?
            },
            DomainType::Ghost => {
                self.resolve_ghost_domain(domain).await?
            },
            DomainType::Unknown => {
                return Err(anyhow!("Unknown domain type: {}", domain));
            }
        };

        // Cache the result
        self.cache_result(domain, &result);
        
        Ok(result)
    }

    fn detect_domain_type(&self, domain: &str) -> DomainType {
        if domain.starts_with("did:") {
            return DomainType::Web5;
        }

        if let Some(tld) = domain.split('.').last() {
            match tld.to_lowercase().as_str() {
                // ENS domains
                "eth" => DomainType::ENS,
                
                // Unstoppable Domains
                "crypto" | "nft" | "blockchain" | "888" | "wallet" | "x" | 
                "klever" | "hi" | "kresus" | "polygon" | "unstoppable" => DomainType::Unstoppable,
                
                // Ghost domains (from DOMAINS.md)
                "ghost" | "gcc" | "sig" | "gpk" | "key" | "pin" | "sid" | 
                "dvm" | "tmp" | "dbg" | "lib" | "txo" | "zns" | "bc" | "ops" => DomainType::Ghost,
                
                _ => DomainType::Unknown,
            }
        } else {
            DomainType::Unknown
        }
    }

    async fn resolve_ens_domain(&mut self, domain: &str) -> Result<DomainResolutionResult> {
        let address = self.ens_resolver.resolve(domain).await?;
        let records = self.ens_resolver.get_records(domain).await?;
        
        Ok(DomainResolutionResult {
            domain: domain.to_string(),
            domain_type: DomainType::ENS,
            resolved_address: Some(address),
            identity: None,
            records,
            metadata: HashMap::new(),
        })
    }

    async fn resolve_unstoppable_domain(&mut self, domain: &str) -> Result<DomainResolutionResult> {
        let address = self.unstoppable_resolver.resolve(domain).await?;
        let records = self.unstoppable_resolver.get_records(domain).await?;
        
        Ok(DomainResolutionResult {
            domain: domain.to_string(),
            domain_type: DomainType::Unstoppable,
            resolved_address: Some(address),
            identity: None,
            records,
            metadata: HashMap::new(),
        })
    }

    async fn resolve_web5_identity(&mut self, did: &str) -> Result<DomainResolutionResult> {
        let identity = self.web5_resolver.resolve_did(did).await?;
        
        Ok(DomainResolutionResult {
            domain: did.to_string(),
            domain_type: DomainType::Web5,
            resolved_address: None, // Web5 DIDs don't resolve to addresses directly
            identity: Some(identity),
            records: Vec::new(),
            metadata: HashMap::new(),
        })
    }

    async fn resolve_ghost_domain(&mut self, domain: &str) -> Result<DomainResolutionResult> {
        let ghost_record = self.ghost_resolver.resolve(domain).await?;
        
        Ok(DomainResolutionResult {
            domain: domain.to_string(),
            domain_type: DomainType::Ghost,
            resolved_address: ghost_record.owner.clone(),
            identity: None,
            records: ghost_record.records,
            metadata: ghost_record.metadata,
        })
    }

    /// Register a domain (only for Ghost domains - others are external)
    pub async fn register_domain(
        &mut self,
        domain: &str,
        owner: &Address,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        let domain_type = self.detect_domain_type(domain);
        
        match domain_type {
            DomainType::Ghost => {
                self.ghost_resolver.register_domain(domain, owner, records, private_key).await
            },
            _ => {
                Err(anyhow!("Cannot register {} domains through GhostChain", domain_type.to_string()))
            }
        }
    }

    /// Update domain records (only for Ghost domains)
    pub async fn update_domain(
        &mut self,
        domain: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        let domain_type = self.detect_domain_type(domain);
        
        match domain_type {
            DomainType::Ghost => {
                self.ghost_resolver.update_domain(domain, records, private_key).await
            },
            _ => {
                Err(anyhow!("Cannot update {} domains through GhostChain", domain_type.to_string()))
            }
        }
    }

    /// Get domain owner
    pub async fn get_domain_owner(&self, domain: &str) -> Result<Option<Address>> {
        let domain_type = self.detect_domain_type(domain);
        
        match domain_type {
            DomainType::ENS => self.ens_resolver.get_owner(domain).await,
            DomainType::Unstoppable => self.unstoppable_resolver.get_owner(domain).await,
            DomainType::Ghost => self.ghost_resolver.get_owner(domain).await,
            DomainType::Web5 => {
                // Web5 DIDs don't have traditional owners
                Ok(None)
            },
            DomainType::Unknown => Ok(None),
        }
    }

    /// Get all domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &Address) -> Result<Vec<String>> {
        let mut all_domains = Vec::new();
        
        // Check ENS domains
        if let Ok(ens_domains) = self.ens_resolver.get_domains_by_owner(owner).await {
            all_domains.extend(ens_domains);
        }
        
        // Check Unstoppable domains
        if let Ok(ud_domains) = self.unstoppable_resolver.get_domains_by_owner(owner).await {
            all_domains.extend(ud_domains);
        }
        
        // Check Ghost domains
        if let Ok(ghost_domains) = self.ghost_resolver.get_domains_by_owner(owner).await {
            all_domains.extend(ghost_domains);
        }
        
        Ok(all_domains)
    }

    fn get_cached(&self, domain: &str) -> Option<&CachedDomainResult> {
        if let Some(cached) = self.cache.get(domain) {
            let now = chrono::Utc::now().timestamp() as u64;
            if now - cached.cached_at < cached.ttl as u64 {
                return Some(cached);
            }
        }
        None
    }

    fn cache_result(&mut self, domain: &str, result: &DomainResolutionResult) {
        let cached = CachedDomainResult {
            domain: domain.to_string(),
            result: result.clone(),
            cached_at: chrono::Utc::now().timestamp() as u64,
            ttl: 300, // 5 minutes default TTL
        };
        self.cache.insert(domain.to_string(), cached);
    }

    /// Clear domain cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), self.cache.len() as u64);
        
        let now = chrono::Utc::now().timestamp() as u64;
        let expired = self.cache.values()
            .filter(|cached| now - cached.cached_at >= cached.ttl as u64)
            .count();
        stats.insert("expired_entries".to_string(), expired as u64);
        
        stats
    }
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainType::ENS => write!(f, "ENS"),
            DomainType::Unstoppable => write!(f, "Unstoppable"),
            DomainType::Web5 => write!(f, "Web5"),
            DomainType::Ghost => write!(f, "Ghost"),
            DomainType::Unknown => write!(f, "Unknown"),
        }
    }
}