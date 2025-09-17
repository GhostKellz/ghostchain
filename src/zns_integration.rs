use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;

/// ZNS Integration Module for GhostChain
/// Bridges Rust GhostChain with external ZNS (Zig Name Service)
#[derive(Debug, Clone)]
pub struct ZnsIntegration {
    zns_binary_path: PathBuf,
    cache: HashMap<String, CachedDomainData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    pub record_type: String,  // A, AAAA, TXT, CNAME, etc.
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainData {
    pub domain: String,
    pub owner: String,
    pub records: Vec<DomainRecord>,
    pub contract_address: Option<String>,
    pub last_updated: u64,
    pub signature: String,
}

#[derive(Debug, Clone)]
struct CachedDomainData {
    data: DomainData,
    cached_at: u64,
    ttl: u32,
}

impl ZnsIntegration {
    pub fn new(zns_binary_path: PathBuf) -> Self {
        Self {
            zns_binary_path,
            cache: HashMap::new(),
        }
    }

    /// Resolve a domain through ZNS
    pub async fn resolve_domain(&mut self, domain: &str) -> Result<DomainData> {
        // Check cache first
        if let Some(cached) = self.get_cached(domain) {
            return Ok(cached.data);
        }

        // Query ZNS binary
        let output = Command::new(&self.zns_binary_path)
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
        let domain_data: DomainData = serde_json::from_str(&json_output)
            .map_err(|e| anyhow!("Failed to parse ZNS response: {}", e))?;

        // Cache the result
        self.cache_domain(domain, &domain_data);

        Ok(domain_data)
    }

    /// Register a domain through ZNS
    pub async fn register_domain(
        &self,
        domain: &str,
        owner: &str,
        records: &[DomainRecord],
        private_key: &str,
    ) -> Result<String> {
        let records_json = serde_json::to_string(records)?;

        let output = Command::new(&self.zns_binary_path)
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
    }

    /// Update domain records
    pub async fn update_domain(
        &mut self,
        domain: &str,
        records: &[DomainRecord],
        private_key: &str,
    ) -> Result<String> {
        let records_json = serde_json::to_string(records)?;

        let output = Command::new(&self.zns_binary_path)
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
    }

    /// Get domain ownership info
    pub async fn get_domain_owner(&mut self, domain: &str) -> Result<String> {
        let domain_data = self.resolve_domain(domain).await?;
        Ok(domain_data.owner)
    }

    /// Check if domain exists
    pub async fn domain_exists(&mut self, domain: &str) -> bool {
        self.resolve_domain(domain).await.is_ok()
    }

    /// Get all domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &str) -> Result<Vec<String>> {
        let output = Command::new(&self.zns_binary_path)
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
