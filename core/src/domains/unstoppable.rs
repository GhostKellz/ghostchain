use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::*;

/// Unstoppable Domains resolver
pub struct UnstoppableResolver {
    pub polygon_rpc_url: String,
    pub ethereum_rpc_url: String,
    pub registry_contracts: HashMap<String, String>, // TLD -> contract address
    pub cache: HashMap<String, UnstoppableDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstoppableDomain {
    pub name: String,
    pub owner: Address,
    pub resolver: Address,
    pub registry: String,
    pub records: HashMap<String, String>, // key -> value mapping
    pub standard_records: Vec<DomainRecord>,
}

impl UnstoppableResolver {
    pub fn new() -> Self {
        let mut registry_contracts = HashMap::new();
        
        // Unstoppable Domains registry contracts (simplified)
        registry_contracts.insert("crypto".to_string(), "0xD1E5b0Ff1287aA9f9A268759062E4Ab08b9dacbe".to_string());
        registry_contracts.insert("nft".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("blockchain".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("888".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("wallet".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("x".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("klever".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("hi".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("kresus".to_string(), "0x049aba7510f45BA5b64ea9E658E342F904DB358D".to_string());
        registry_contracts.insert("polygon".to_string(), "0xa9a6A3626993D487d2Dbda3173cf58cA1a9D9e9f".to_string());

        Self {
            polygon_rpc_url: "https://polygon-mainnet.g.alchemy.com/v2/YOUR-API-KEY".to_string(),
            ethereum_rpc_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY".to_string(),
            registry_contracts,
            cache: HashMap::new(),
        }
    }

    pub fn with_rpc_urls(mut self, polygon_url: String, ethereum_url: String) -> Self {
        self.polygon_rpc_url = polygon_url;
        self.ethereum_rpc_url = ethereum_url;
        self
    }

    /// Resolve Unstoppable Domain to cryptocurrency addresses
    pub async fn resolve(&mut self, domain: &str) -> Result<Address> {
        let tld = self.get_tld(domain)?;
        
        if !self.registry_contracts.contains_key(&tld) {
            return Err(anyhow!("Unsupported Unstoppable Domain TLD: {}", tld));
        }

        // Check cache first
        if let Some(ud_domain) = self.cache.get(domain) {
            // Return ETH address by default, or first available address
            if let Some(eth_addr) = ud_domain.records.get("crypto.ETH.address") {
                return Ok(eth_addr.clone());
            }
            if let Some(btc_addr) = ud_domain.records.get("crypto.BTC.address") {
                return Ok(btc_addr.clone());
            }
        }

        // In real implementation, this would:
        // 1. Connect to appropriate blockchain (Polygon for newer domains, Ethereum for .crypto)
        // 2. Query registry contract for domain token ID
        // 3. Query resolver contract for records
        
        // Simulate resolution with some example domains
        let (eth_address, records) = match domain {
            "brad.crypto" => (
                "0x8aaD44321A86b170879d7A244c1e8d360c99DdA8".to_string(),
                self.create_sample_records("brad.crypto", "0x8aaD44321A86b170879d7A244c1e8d360c99DdA8")
            ),
            "unstoppabledomains.crypto" => (
                "0x1C42088b82f6B222B9C70aE4e6B522F87beEB5E0".to_string(),
                self.create_sample_records("unstoppabledomains.crypto", "0x1C42088b82f6B222B9C70aE4e6B522F87beEB5E0")
            ),
            _ => {
                // Generate deterministic address for simulation
                let address = format!("0x{:040x}", self.hash_domain(domain));
                (address.clone(), self.create_sample_records(domain, &address))
            }
        };

        // Cache the result
        let ud_domain = UnstoppableDomain {
            name: domain.to_string(),
            owner: eth_address.clone(),
            resolver: self.registry_contracts.get(&tld).unwrap().clone(),
            registry: tld.clone(),
            records,
            standard_records: vec![
                DomainRecord {
                    record_type: "A".to_string(),
                    name: domain.to_string(),
                    value: eth_address.clone(),
                    ttl: 3600,
                    priority: None,
                },
            ],
        };
        
        self.cache.insert(domain.to_string(), ud_domain);
        Ok(eth_address)
    }

    /// Get all records for an Unstoppable Domain
    pub async fn get_records(&self, domain: &str) -> Result<Vec<DomainRecord>> {
        if let Some(ud_domain) = self.cache.get(domain) {
            return Ok(ud_domain.standard_records.clone());
        }

        // In real implementation, would query all records from resolver contract
        Ok(vec![
            DomainRecord {
                record_type: "A".to_string(),
                name: domain.to_string(),
                value: format!("0x{:040x}", self.hash_domain(domain)),
                ttl: 3600,
                priority: None,
            },
        ])
    }

    /// Get specific record value (like crypto.ETH.address, ipfs.html.value, etc.)
    pub async fn get_record(&self, domain: &str, key: &str) -> Result<Option<String>> {
        if let Some(ud_domain) = self.cache.get(domain) {
            return Ok(ud_domain.records.get(key).cloned());
        }

        // In real implementation, would query specific record from resolver contract
        Ok(None)
    }

    /// Get domain owner
    pub async fn get_owner(&self, domain: &str) -> Result<Option<Address>> {
        if let Some(ud_domain) = self.cache.get(domain) {
            return Ok(Some(ud_domain.owner.clone()));
        }

        // In real implementation, would query registry contract for owner
        Ok(Some(format!("0x{:040x}", self.hash_domain(&format!("owner_{}", domain)))))
    }

    /// Get domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &Address) -> Result<Vec<String>> {
        // In real implementation, would use indexing service or events
        let domains: Vec<String> = self.cache
            .values()
            .filter(|domain| &domain.owner == owner)
            .map(|domain| domain.name.clone())
            .collect();
        
        Ok(domains)
    }

    /// Set domain records (requires ownership)
    pub async fn set_records(
        &mut self,
        domain: &str,
        records: HashMap<String, String>,
        private_key: &str,
    ) -> Result<String> {
        // In real implementation, would:
        // 1. Verify domain ownership
        // 2. Create transaction to update resolver records
        // 3. Sign and broadcast transaction
        
        if let Some(ud_domain) = self.cache.get_mut(domain) {
            ud_domain.records.extend(records);
            Ok(format!("0x{:064x}", self.hash_domain(&format!("update_{}", domain))))
        } else {
            Err(anyhow!("Domain not found: {}", domain))
        }
    }

    /// Check if domain is available for registration
    pub async fn is_available(&self, domain: &str) -> Result<bool> {
        let tld = self.get_tld(domain)?;
        
        if !self.registry_contracts.contains_key(&tld) {
            return Ok(false);
        }

        // In real implementation, would check registry contract
        Ok(!self.cache.contains_key(domain))
    }

    /// Get supported cryptocurrency addresses for a domain
    pub async fn get_crypto_addresses(&self, domain: &str) -> Result<HashMap<String, String>> {
        if let Some(ud_domain) = self.cache.get(domain) {
            let crypto_records: HashMap<String, String> = ud_domain.records
                .iter()
                .filter(|(key, _)| key.starts_with("crypto.") && key.ends_with(".address"))
                .map(|(key, value)| {
                    let currency = key.strip_prefix("crypto.").unwrap()
                        .strip_suffix(".address").unwrap();
                    (currency.to_string(), value.clone())
                })
                .collect();
            return Ok(crypto_records);
        }

        Ok(HashMap::new())
    }

    /// Get IPFS content hash for domain
    pub async fn get_ipfs_hash(&self, domain: &str) -> Result<Option<String>> {
        if let Some(ud_domain) = self.cache.get(domain) {
            return Ok(ud_domain.records.get("ipfs.html.value").cloned());
        }
        Ok(None)
    }

    fn get_tld(&self, domain: &str) -> Result<String> {
        domain.split('.').last()
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow!("Invalid domain format: {}", domain))
    }

    fn create_sample_records(&self, domain: &str, eth_address: &str) -> HashMap<String, String> {
        let mut records = HashMap::new();
        
        // Cryptocurrency addresses
        records.insert("crypto.ETH.address".to_string(), eth_address.to_string());
        records.insert("crypto.BTC.address".to_string(), format!("bc1q{:x}", self.hash_domain(&format!("btc_{}", domain))));
        records.insert("crypto.LTC.address".to_string(), format!("ltc1q{:x}", self.hash_domain(&format!("ltc_{}", domain))));
        
        // Web records
        records.insert("ipfs.html.value".to_string(), format!("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"));
        records.insert("ipfs.redirect_domain.value".to_string(), format!("{}.on.fleek.co", domain.split('.').next().unwrap()));
        
        // Social records
        records.insert("social.twitter.username".to_string(), format!("@{}", domain.split('.').next().unwrap()));
        records.insert("social.github.username".to_string(), domain.split('.').next().unwrap().to_string());
        
        records
    }

    fn hash_domain(&self, domain: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        hasher.finish()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}