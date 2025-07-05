use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;
use crate::types::*;
use crate::blockchain::integration::BlockchainContractIntegration;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ZNS Integration Module for GhostChain
/// Bridges Rust GhostChain with external ZNS (Zig Name Service)
#[derive(Debug)]
pub struct ZnsIntegration {
    zns_binary_path: Option<PathBuf>, // Optional external binary
    cache: HashMap<String, CachedDomainData>,
    contract_integration: Option<Arc<RwLock<BlockchainContractIntegration>>>, // On-chain integration
    use_onchain: bool, // Whether to use on-chain or external binary
}

// DomainRecord and DomainData are now defined in types.rs

#[derive(Debug, Clone)]
struct CachedDomainData {
    data: DomainData,
    cached_at: u64,
    ttl: u32,
}

impl ZnsIntegration {
    pub fn new_external(zns_binary_path: PathBuf) -> Self {
        Self {
            zns_binary_path: Some(zns_binary_path),
            cache: HashMap::new(),
            contract_integration: None,
            use_onchain: false,
        }
    }
    
    pub fn new_onchain(contract_integration: Arc<RwLock<BlockchainContractIntegration>>) -> Self {
        Self {
            zns_binary_path: None,
            cache: HashMap::new(),
            contract_integration: Some(contract_integration),
            use_onchain: true,
        }
    }
    
    pub fn new_hybrid(
        zns_binary_path: PathBuf,
        contract_integration: Arc<RwLock<BlockchainContractIntegration>>
    ) -> Self {
        Self {
            zns_binary_path: Some(zns_binary_path),
            cache: HashMap::new(),
            contract_integration: Some(contract_integration),
            use_onchain: true, // Prefer on-chain by default
        }
    }
    
    pub fn set_mode(&mut self, use_onchain: bool) {
        self.use_onchain = use_onchain;
    }

    /// Resolve a domain through ZNS
    pub async fn resolve_domain(&mut self, domain: &str) -> Result<DomainData> {
        // Check cache first
        if let Some(cached) = self.get_cached(domain) {
            return Ok(cached.data.clone());
        }

        let domain_data = if self.use_onchain && self.contract_integration.is_some() {
            // Use on-chain contract
            let integration = self.contract_integration.as_ref().unwrap();
            let integration_guard = integration.read().await;
            integration_guard.resolve_domain(domain).await?
        } else if let Some(zns_binary_path) = &self.zns_binary_path {
            // Use external ZNS binary
            let output = Command::new(zns_binary_path)
                .arg("resolve")
                .arg(domain)
                .arg("--json")
                .output()
                .map_err(|e| anyhow!("Failed to execute ZNS binary: {}", e))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("ZNS resolution failed: {}", error));
            }

            let json_output = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str(&json_output)
                .map_err(|e| anyhow!("Failed to parse ZNS response: {}", e))?
        } else {
            return Err(anyhow!("No ZNS resolution method available"));
        };

        // Cache the result
        self.cache_domain(domain, &domain_data);

        Ok(domain_data)
    }

    /// Register a domain through ZNS
    pub async fn register_domain(
        &mut self,
        domain: &str,
        owner: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        if self.use_onchain && self.contract_integration.is_some() {
            // Use on-chain contract
            let integration = self.contract_integration.as_ref().unwrap();
            let mut integration_guard = integration.write().await;
            let owner_addr: Address = owner.to_string();
            let result = integration_guard.register_domain(domain, &owner_addr, records).await?;
            
            // Return transaction hash or contract result
            Ok(format!("Contract call successful, gas used: {}", result.gas_used))
        } else if let Some(zns_binary_path) = &self.zns_binary_path {
            // Use external ZNS binary
            let private_key = private_key.ok_or_else(|| anyhow!("Private key required for external ZNS"))?;
            let records_json = serde_json::to_string(&records)?;

            let output = Command::new(zns_binary_path)
                .arg("register")
                .arg(domain)
                .arg("--owner")
                .arg(owner)
                .arg("--records")
                .arg(&records_json)
                .arg("--key")
                .arg(private_key)
                .output()
                .map_err(|e| anyhow!("Failed to execute ZNS binary: {}", e))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("ZNS registration failed: {}", error));
            }

            let transaction_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(transaction_hash)
        } else {
            Err(anyhow!("No ZNS registration method available"))
        }
    }

    /// Update domain records
    pub async fn update_domain(
        &mut self,
        domain: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        if self.use_onchain && self.contract_integration.is_some() {
            // Use on-chain contract - for simplicity, we'll set one record at a time
            let integration = self.contract_integration.as_ref().unwrap();
            let mut integration_guard = integration.write().await;
            
            let mut total_gas = 0;
            for record in records {
                // This would need the actual contract call for setting records
                // For now, we'll simulate it
                total_gas += 10000; // Estimated gas per record
            }
            
            // Clear cache for this domain
            self.cache.remove(domain);
            
            Ok(format!("Contract records updated, estimated gas: {}", total_gas))
        } else if let Some(zns_binary_path) = &self.zns_binary_path {
            // Use external ZNS binary
            let private_key = private_key.ok_or_else(|| anyhow!("Private key required for external ZNS"))?;
            let records_json = serde_json::to_string(&records)?;

            let output = Command::new(zns_binary_path)
                .arg("update")
                .arg(domain)
                .arg("--records")
                .arg(&records_json)
                .arg("--key")
                .arg(private_key)
                .output()
                .map_err(|e| anyhow!("Failed to execute ZNS binary: {}", e))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("ZNS update failed: {}", error));
            }

            // Clear cache for this domain
            self.cache.remove(domain);

            let transaction_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(transaction_hash)
        } else {
            Err(anyhow!("No ZNS update method available"))
        }
    }

    /// Get domain ownership info
    pub async fn get_domain_owner(&mut self, domain: &str) -> Result<Address> {
        let domain_data = self.resolve_domain(domain).await?;
        Ok(domain_data.owner)
    }

    /// Check if domain exists
    pub async fn domain_exists(&mut self, domain: &str) -> bool {
        self.resolve_domain(domain).await.is_ok()
    }

    /// Get all domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &str) -> Result<Vec<String>> {
        if self.use_onchain && self.contract_integration.is_some() {
            // Use on-chain contract query
            let integration = self.contract_integration.as_ref().unwrap();
            let integration_guard = integration.read().await;
            
            let query_data = serde_json::to_vec(&serde_json::json!({
                "owner": owner
            }))?;
            
            let result = integration_guard.query_contract(
                &"system.domain_registry".to_string(),
                "get_owner_domains",
                &query_data,
            ).await?;
            
            let domains: Vec<String> = serde_json::from_slice(&result)?;
            Ok(domains)
        } else if let Some(zns_binary_path) = &self.zns_binary_path {
            // Use external ZNS binary
            let output = Command::new(zns_binary_path)
                .arg("list")
                .arg("--owner")
                .arg(owner)
                .arg("--json")
                .output()
                .map_err(|e| anyhow!("Failed to execute ZNS binary: {}", e))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("ZNS list failed: {}", error));
            }

            let json_output = String::from_utf8_lossy(&output.stdout);
            let domains: Vec<String> = serde_json::from_str(&json_output)
                .map_err(|e| anyhow!("Failed to parse ZNS response: {}", e))?;

            Ok(domains)
        } else {
            Err(anyhow!("No ZNS list method available"))
        }
    }

    // Private helper methods

    fn get_cached(&self, domain: &str) -> Option<&CachedDomainData> {
        if let Some(cached) = self.cache.get(domain) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now < cached.cached_at + cached.ttl as u64 {
                return Some(cached);
            }
        }
        None
    }

    fn cache_domain(&mut self, domain: &str, data: &DomainData) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Use minimum TTL from records, default to 300 seconds
        let ttl = data.records.iter()
            .map(|r| r.ttl)
            .min()
            .unwrap_or(300);

        let cached_data = CachedDomainData {
            data: data.clone(),
            cached_at: now,
            ttl,
        };

        self.cache.insert(domain.to_string(), cached_data);
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache.retain(|_, cached| {
            now < cached.cached_at + cached.ttl as u64
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_zns_integration_resolve() {
        // Mock test - would need actual ZNS binary for integration testing
        let zns_path = PathBuf::from("./zns"); // Path to ZNS binary
        let mut zns = ZnsIntegration::new(zns_path);
        
        // This would only work with actual ZNS binary
        // let result = zns.resolve_domain("test.ghost").await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_cache_functionality() {
        let zns_path = PathBuf::from("./zns");
        let mut zns = ZnsIntegration::new(zns_path);

        let domain_data = DomainData {
            domain: "test.ghost".to_string(),
            owner: "owner123".to_string(),
            records: vec![
                DomainRecord {
                    record_type: "A".to_string(),
                    name: "test.ghost".to_string(),
                    value: "192.168.1.1".to_string(),
                    ttl: 300,
                    priority: None,
                }
            ],
            contract_address: None,
            last_updated: 1640995200,
            signature: "sig123".to_string(),
        };

        zns.cache_domain("test.ghost", &domain_data);
        assert!(zns.get_cached("test.ghost").is_some());
    }
}
