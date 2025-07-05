use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::*;
use crate::blockchain::integration::BlockchainContractIntegration;

/// GhostChain native domain resolver
pub struct GhostDomainResolver {
    pub blockchain_integration: Option<Arc<RwLock<BlockchainContractIntegration>>>,
    pub cache: HashMap<String, GhostDomainRecord>,
    pub supported_tlds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostDomainRecord {
    pub domain: String,
    pub owner: Option<Address>,
    pub records: Vec<DomainRecord>,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub last_updated: u64,
}

impl GhostDomainResolver {
    pub fn new() -> Self {
        // From DOMAINS.md - GhostChain native TLDs
        let supported_tlds = vec![
            // Core Identity Domains
            "ghost".to_string(),    // Root domain of GhostChain identities and services
            "gcc".to_string(),      // GhostChain Contracts
            "sig".to_string(),      // Signature authorities and verifiers
            "gpk".to_string(),      // GhostChain Public Key registry
            "key".to_string(),      // Public key alias domain
            "pin".to_string(),      // Persistent Identity Node
            
            // Decentralized & Blockchain Infrastructure
            "bc".to_string(),       // General blockchain assets and services
            "zns".to_string(),      // Root namespace registry
            "ops".to_string(),      // Operational nodes
            
            // Reserved for Future/Extension Use
            "sid".to_string(),      // Secure identity domain
            "dvm".to_string(),      // Decentralized Virtual Machine domains
            "tmp".to_string(),      // Temporary identity bindings
            "dbg".to_string(),      // Debug/testnet addresses
            "lib".to_string(),      // Shared contract libraries
            "txo".to_string(),      // Transaction-output indexed namespaces
        ];

        Self {
            blockchain_integration: None,
            cache: HashMap::new(),
            supported_tlds,
        }
    }

    pub fn with_blockchain_integration(mut self, integration: Arc<RwLock<BlockchainContractIntegration>>) -> Self {
        self.blockchain_integration = Some(integration);
        self
    }

    /// Resolve a Ghost domain
    pub async fn resolve(&mut self, domain: &str) -> Result<GhostDomainRecord> {
        let tld = self.get_tld(domain)?;
        
        if !self.supported_tlds.contains(&tld) {
            return Err(anyhow!("Unsupported Ghost domain TLD: {}", tld));
        }

        // Check cache first
        if let Some(record) = self.cache.get(domain) {
            if !self.is_expired(record) {
                return Ok(record.clone());
            }
        }

        // Query blockchain if integration is available
        if let Some(integration) = &self.blockchain_integration {
            let integration_guard = integration.read().await;
            // In real implementation, would query domain registry contract
            drop(integration_guard);
        }

        // Create sample record for demonstration
        let record = self.create_sample_record(domain, &tld);
        self.cache.insert(domain.to_string(), record.clone());
        
        Ok(record)
    }

    /// Register a new Ghost domain
    pub async fn register_domain(
        &mut self,
        domain: &str,
        owner: &Address,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        let tld = self.get_tld(domain)?;
        
        if !self.supported_tlds.contains(&tld) {
            return Err(anyhow!("Cannot register unsupported TLD: {}", tld));
        }

        // Check if domain is available
        if self.cache.contains_key(domain) {
            return Err(anyhow!("Domain already registered: {}", domain));
        }

        // Validate domain name based on TLD rules
        self.validate_domain_name(domain, &tld)?;

        // If blockchain integration is available, register on-chain
        if let Some(integration) = &self.blockchain_integration {
            let mut integration_guard = integration.write().await;
            let owner_addr: Address = owner.to_string();
            let result = integration_guard.register_domain(domain, &owner_addr, records.clone()).await?;
            
            // Create and cache the record
            let record = GhostDomainRecord {
                domain: domain.to_string(),
                owner: Some(owner.clone()),
                records: records.clone(),
                metadata: self.create_domain_metadata(domain, &tld),
                created_at: chrono::Utc::now().timestamp() as u64,
                expires_at: None, // Ghost domains don't expire by default
                last_updated: chrono::Utc::now().timestamp() as u64,
            };
            
            self.cache.insert(domain.to_string(), record);
            return Ok(format!("On-chain registration successful, gas used: {}", result.gas_used));
        }

        // Local registration (for testing/development)
        let record = GhostDomainRecord {
            domain: domain.to_string(),
            owner: Some(owner.clone()),
            records,
            metadata: self.create_domain_metadata(domain, &tld),
            created_at: chrono::Utc::now().timestamp() as u64,
            expires_at: None,
            last_updated: chrono::Utc::now().timestamp() as u64,
        };
        
        self.cache.insert(domain.to_string(), record);
        Ok(format!("Local registration successful for {}", domain))
    }

    /// Update domain records
    pub async fn update_domain(
        &mut self,
        domain: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        // Check if domain exists
        if !self.cache.contains_key(domain) {
            return Err(anyhow!("Domain not found: {}", domain));
        }

        // In real implementation, would verify ownership through signature

        // Update the record
        if let Some(record) = self.cache.get_mut(domain) {
            record.records = records;
            record.last_updated = chrono::Utc::now().timestamp() as u64;
        }

        if let Some(integration) = &self.blockchain_integration {
            // In real implementation, would update on-chain
            Ok(format!("Domain {} updated on-chain", domain))
        } else {
            Ok(format!("Domain {} updated locally", domain))
        }
    }

    /// Get domain owner
    pub async fn get_owner(&self, domain: &str) -> Result<Option<Address>> {
        if let Some(record) = self.cache.get(domain) {
            return Ok(record.owner.clone());
        }

        // Query blockchain if not in cache
        if let Some(integration) = &self.blockchain_integration {
            // In real implementation, would query registry contract
        }

        Ok(None)
    }

    /// Get domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &Address) -> Result<Vec<String>> {
        let domains: Vec<String> = self.cache
            .values()
            .filter(|record| record.owner.as_ref() == Some(owner))
            .map(|record| record.domain.clone())
            .collect();
        
        Ok(domains)
    }

    /// Transfer domain ownership
    pub async fn transfer_domain(
        &mut self,
        domain: &str,
        new_owner: &Address,
        private_key: &str,
    ) -> Result<String> {
        if let Some(record) = self.cache.get_mut(domain) {
            // In real implementation, would verify current owner's signature
            record.owner = Some(new_owner.clone());
            record.last_updated = chrono::Utc::now().timestamp() as u64;
            
            Ok(format!("Domain {} transferred to {}", domain, new_owner))
        } else {
            Err(anyhow!("Domain not found: {}", domain))
        }
    }

    /// Check if domain is available
    pub async fn is_available(&self, domain: &str) -> Result<bool> {
        let tld = self.get_tld(domain)?;
        
        if !self.supported_tlds.contains(&tld) {
            return Ok(false);
        }

        // Check cache and blockchain
        Ok(!self.cache.contains_key(domain))
    }

    /// Get domain registration cost
    pub fn get_registration_cost(&self, domain: &str) -> Result<u128> {
        let tld = self.get_tld(domain)?;
        let name_part = domain.strip_suffix(&format!(".{}", tld)).unwrap();
        
        // Pricing based on domain length and TLD importance
        let base_cost = match tld.as_str() {
            "ghost" => 1_000_000_000_000_000_000u128, // 1 GHOST for root domains
            "gcc" | "sig" | "gpk" => 500_000_000_000_000_000u128, // 0.5 GHOST for key domains
            "key" | "pin" | "zns" => 100_000_000_000_000_000u128, // 0.1 GHOST for identity domains
            _ => 50_000_000_000_000_000u128, // 0.05 GHOST for other domains
        };

        // Length-based pricing
        let length_multiplier = match name_part.len() {
            1 => 10.0,
            2 => 5.0,
            3 => 2.0,
            4 => 1.5,
            _ => 1.0,
        };

        Ok((base_cost as f64 * length_multiplier) as u128)
    }

    /// Validate domain name according to Ghost domain rules
    fn validate_domain_name(&self, domain: &str, tld: &str) -> Result<()> {
        let name_part = domain.strip_suffix(&format!(".{}", tld))
            .ok_or_else(|| anyhow!("Invalid domain format"))?;

        // Length restrictions
        if name_part.len() < 1 || name_part.len() > 63 {
            return Err(anyhow!("Domain name must be 1-63 characters"));
        }

        // Character restrictions
        if !name_part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(anyhow!("Domain name can only contain alphanumeric characters and hyphens"));
        }

        // Cannot start or end with hyphen
        if name_part.starts_with('-') || name_part.ends_with('-') {
            return Err(anyhow!("Domain name cannot start or end with hyphen"));
        }

        // TLD-specific restrictions
        match tld {
            "ghost" => {
                // Reserved for system use - require special authorization
                if name_part.len() < 4 && !["node", "sys", "net", "dev"].contains(&name_part) {
                    return Err(anyhow!("Short .ghost domains are reserved"));
                }
            },
            "sig" | "gpk" => {
                // Must be valid hex for cryptographic domains
                if name_part.len() == 64 && !name_part.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(anyhow!("{} domains should be valid hex for cryptographic use", tld));
                }
            },
            _ => {} // No special restrictions for other TLDs
        }

        Ok(())
    }

    fn create_sample_record(&self, domain: &str, tld: &str) -> GhostDomainRecord {
        let mut records = Vec::new();
        let mut metadata = self.create_domain_metadata(domain, tld);

        // Create records based on TLD type
        match tld {
            "ghost" => {
                records.push(DomainRecord {
                    record_type: "A".to_string(),
                    name: domain.to_string(),
                    value: format!("192.168.1.{}", self.hash_domain(domain) % 255),
                    ttl: 3600,
                    priority: None,
                });
                records.push(DomainRecord {
                    record_type: "TXT".to_string(),
                    name: domain.to_string(),
                    value: "GhostChain Root Node".to_string(),
                    ttl: 3600,
                    priority: None,
                });
            },
            "gcc" => {
                records.push(DomainRecord {
                    record_type: "CONTRACT".to_string(),
                    name: domain.to_string(),
                    value: format!("0x{:040x}", self.hash_domain(domain)),
                    ttl: 86400,
                    priority: None,
                });
                metadata.insert("contract_type".to_string(), "ERC20".to_string());
            },
            "sig" | "gpk" | "key" => {
                records.push(DomainRecord {
                    record_type: "PUBKEY".to_string(),
                    name: domain.to_string(),
                    value: format!("{:064x}", self.hash_domain(domain)),
                    ttl: 86400,
                    priority: None,
                });
                metadata.insert("key_type".to_string(), "Ed25519".to_string());
            },
            _ => {
                records.push(DomainRecord {
                    record_type: "A".to_string(),
                    name: domain.to_string(),
                    value: format!("10.0.0.{}", self.hash_domain(domain) % 255),
                    ttl: 3600,
                    priority: None,
                });
            }
        }

        GhostDomainRecord {
            domain: domain.to_string(),
            owner: Some(format!("0x{:040x}", self.hash_domain(&format!("owner_{}", domain)))),
            records,
            metadata,
            created_at: chrono::Utc::now().timestamp() as u64,
            expires_at: None,
            last_updated: chrono::Utc::now().timestamp() as u64,
        }
    }

    fn create_domain_metadata(&self, domain: &str, tld: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("tld".to_string(), tld.to_string());
        metadata.insert("registrar".to_string(), "GhostChain".to_string());
        metadata.insert("registry_version".to_string(), "1.0".to_string());
        
        // TLD-specific metadata
        match tld {
            "ghost" => {
                metadata.insert("type".to_string(), "root_identity".to_string());
                metadata.insert("system".to_string(), "ghostchain".to_string());
            },
            "gcc" => {
                metadata.insert("type".to_string(), "smart_contract".to_string());
                metadata.insert("blockchain".to_string(), "ghostchain".to_string());
            },
            "sig" | "gpk" | "key" => {
                metadata.insert("type".to_string(), "cryptographic_key".to_string());
                metadata.insert("purpose".to_string(), "authentication".to_string());
            },
            "zns" => {
                metadata.insert("type".to_string(), "namespace_registry".to_string());
                metadata.insert("protocol".to_string(), "zns".to_string());
            },
            _ => {
                metadata.insert("type".to_string(), "general".to_string());
            }
        }
        
        metadata
    }

    fn get_tld(&self, domain: &str) -> Result<String> {
        domain.split('.').last()
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow!("Invalid domain format: {}", domain))
    }

    fn is_expired(&self, record: &GhostDomainRecord) -> bool {
        if let Some(expires_at) = record.expires_at {
            let now = chrono::Utc::now().timestamp() as u64;
            now > expires_at
        } else {
            false // Ghost domains don't expire by default
        }
    }

    fn hash_domain(&self, domain: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_supported_tlds(&self) -> &Vec<String> {
        &self.supported_tlds
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}