use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// A simple LRU cache implementation with time-based expiration
pub struct LRUCache<K, V> {
    capacity: usize,
    data: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
}

impl<K, V> LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self::with_ttl(capacity, Duration::from_secs(3600)) // 1 hour default TTL
    }

    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        Self {
            capacity,
            data: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            ttl,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let now = Instant::now();
        
        // Check if key exists and is not expired
        if let Some(entry) = self.data.get(key) {
            if now.duration_since(entry.created_at) < self.ttl {
                // Update access time and order in a separate step
                let value = entry.value.clone();
                if let Some(entry) = self.data.get_mut(key) {
                    entry.last_accessed = now;
                }
                self.update_access_order(key);
                return Some(value);
            } else {
                // Mark for removal by collecting expired keys
                let expired_key = key.clone();
                self.data.remove(&expired_key);
                self.access_order.retain(|k| k != &expired_key);
            }
        }
        
        None
    }

    pub fn insert(&mut self, key: K, value: V) {
        let now = Instant::now();
        
        // If key already exists, update it
        if self.data.contains_key(&key) {
            let entry = CacheEntry {
                value,
                created_at: now,
                last_accessed: now,
            };
            self.data.insert(key.clone(), entry);
            self.update_access_order(&key);
            return;
        }

        // If at capacity, remove least recently used
        if self.data.len() >= self.capacity {
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.data.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        // Insert new entry
        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
        };
        self.data.insert(key.clone(), entry);
        self.access_order.push(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.remove(key) {
            self.access_order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.access_order.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<K> = self.data
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) >= self.ttl)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            self.data.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
    }

    pub fn estimated_size(&self) -> usize {
        // Rough estimate: assume each entry takes about 100 bytes
        self.data.len() * 100
    }

    fn update_access_order(&mut self, key: &K) {
        // Move key to end (most recently used)
        self.access_order.retain(|k| k != key);
        self.access_order.push(key.clone());
    }
}

/// Multi-level cache for different data types
pub struct MultiLevelCache {
    block_cache: LRUCache<u64, Vec<u8>>,
    account_cache: LRUCache<String, Vec<u8>>,
    transaction_cache: LRUCache<String, Vec<u8>>,
    contract_cache: LRUCache<String, Vec<u8>>,
}

impl MultiLevelCache {
    pub fn new() -> Self {
        Self {
            block_cache: LRUCache::new(1000),
            account_cache: LRUCache::new(5000),
            transaction_cache: LRUCache::new(10000),
            contract_cache: LRUCache::new(500),
        }
    }

    pub fn get_block(&mut self, height: u64) -> Option<Vec<u8>> {
        self.block_cache.get(&height)
    }

    pub fn cache_block(&mut self, height: u64, data: Vec<u8>) {
        self.block_cache.insert(height, data);
    }

    pub fn get_account(&mut self, address: &str) -> Option<Vec<u8>> {
        self.account_cache.get(&address.to_string())
    }

    pub fn cache_account(&mut self, address: String, data: Vec<u8>) {
        self.account_cache.insert(address, data);
    }

    pub fn get_transaction(&mut self, tx_id: &str) -> Option<Vec<u8>> {
        self.transaction_cache.get(&tx_id.to_string())
    }

    pub fn cache_transaction(&mut self, tx_id: String, data: Vec<u8>) {
        self.transaction_cache.insert(tx_id, data);
    }

    pub fn get_contract(&mut self, contract_id: &str) -> Option<Vec<u8>> {
        self.contract_cache.get(&contract_id.to_string())
    }

    pub fn cache_contract(&mut self, contract_id: String, data: Vec<u8>) {
        self.contract_cache.insert(contract_id, data);
    }

    pub fn cleanup_all(&mut self) {
        self.block_cache.cleanup();
        self.account_cache.cleanup();
        self.transaction_cache.cleanup();
        self.contract_cache.cleanup();
    }

    pub fn clear_all(&mut self) {
        self.block_cache.clear();
        self.account_cache.clear();
        self.transaction_cache.clear();
        self.contract_cache.clear();
    }

    pub fn total_size(&self) -> usize {
        self.block_cache.len() + 
        self.account_cache.len() + 
        self.transaction_cache.len() + 
        self.contract_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LRUCache::new(3);
        
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3);
        
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
        assert_eq!(cache.len(), 3);
        
        // Should evict "a" when inserting "d"
        cache.insert("d", 4);
        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"d"), Some(&4));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_lru_cache_ttl() {
        let mut cache = LRUCache::with_ttl(3, Duration::from_millis(50));
        
        cache.insert("a", 1);
        assert_eq!(cache.get(&"a"), Some(&1));
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(60));
        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn test_multi_level_cache() {
        let mut cache = MultiLevelCache::new();
        
        cache.cache_block(1, vec![1, 2, 3]);
        cache.cache_account("addr1".to_string(), vec![4, 5, 6]);
        
        assert_eq!(cache.get_block(1), Some(vec![1, 2, 3]));
        assert_eq!(cache.get_account("addr1"), Some(vec![4, 5, 6]));
        assert_eq!(cache.total_size(), 2);
    }
}