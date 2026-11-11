//! A bounded owner of decoded bundle payloads. Caller-held Arcs have separate lifetimes.

use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CacheStats {
    pub entries: usize,
    pub payload_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

pub struct BundleCache<T> {
    entries: BTreeMap<String, (Arc<T>, u64)>,
    lru: VecDeque<String>,
    max_entries: usize,
    max_bytes: u64,
    stats: CacheStats,
}

impl<T> BundleCache<T> {
    pub fn new(max_entries: usize, max_bytes: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            lru: VecDeque::new(),
            max_entries,
            max_bytes,
            stats: CacheStats::default(),
        }
    }

    pub fn get(&mut self, name: &str) -> Option<Arc<T>> {
        let Some((value, _)) = self.entries.get(name) else {
            self.stats.misses += 1;
            return None;
        };
        let value = value.clone();
        self.lru.retain(|key| key != name);
        self.lru.push_back(name.into());
        self.stats.hits += 1;
        Some(value)
    }

    pub fn insert(&mut self, name: String, value: Arc<T>, bytes: u64) {
        if self.max_entries == 0 || bytes > self.max_bytes {
            return;
        }
        if let Some((_, old_bytes)) = self.entries.remove(&name) {
            self.stats.payload_bytes -= old_bytes;
            self.lru.retain(|key| key != &name);
        }
        while self.entries.len() >= self.max_entries
            || self.stats.payload_bytes > self.max_bytes - bytes
        {
            let Some(oldest) = self.lru.pop_front() else {
                break;
            };
            if let Some((_, old_bytes)) = self.entries.remove(&oldest) {
                self.stats.payload_bytes -= old_bytes;
                self.stats.evictions += 1;
            }
        }
        self.stats.payload_bytes += bytes;
        self.entries.insert(name.clone(), (value, bytes));
        self.lru.push_back(name);
        self.stats.entries = self.entries.len();
    }

    pub fn stats(&self) -> CacheStats {
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_dependencies_survive_eviction_and_caller_handles_remain_valid() {
        let mut cache = BundleCache::new(2, 10);
        let a = Arc::new("a");
        cache.insert("a".into(), a.clone(), 4);
        cache.insert("b".into(), Arc::new("b"), 4);
        assert_eq!(*cache.get("a").unwrap(), "a");
        cache.insert("c".into(), Arc::new("c"), 4);
        assert!(cache.get("b").is_none());
        assert!(cache.get("a").is_some());
        assert_eq!(*a, "a");
        assert_eq!(cache.stats().payload_bytes, 8);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn large_bundle_is_returnable_but_never_retained_above_cache_budget() {
        let mut cache = BundleCache::new(3, 10);
        cache.insert("a".into(), Arc::new(1), 6);
        cache.insert("b".into(), Arc::new(2), 6);
        assert!(cache.get("a").is_none());
        cache.insert("large".into(), Arc::new(3), 11);
        assert!(cache.get("large").is_none());
        assert_eq!(cache.stats().entries, 1);
        assert_eq!(cache.stats().payload_bytes, 6);
    }
}
