use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct DnsCacheEntry {
    pub domain: String,
    pub ip: IpAddr,
    pub family: AddressFamily,
    pub created_at: Instant,
    pub ttl: Duration,
    pub hit_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressFamily {
    V4,
    V6,
}

#[derive(Debug, Clone, Default)]
pub struct DnsCacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub expired: u64,
    pub evictions: u64,
}

pub struct VsaDnsCache {
    entries: HashMap<String, DnsCacheEntry>,
    capacity: usize,
    default_ttl: Duration,
    stats: DnsCacheStats,
}

impl VsaDnsCache {
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
            default_ttl,
            stats: DnsCacheStats::default(),
        }
    }

    pub fn resolve(&mut self, domain: &str, family: AddressFamily) -> Option<IpAddr> {
        let now = Instant::now();
        if let Some(entry) = self.entries.get(domain) {
            if now.duration_since(entry.created_at) < entry.ttl && entry.family == family {
                let entry = self.entries.get_mut(domain).unwrap();
                entry.hit_count += 1;
                self.stats.hits += 1;
                return Some(entry.ip);
            }
            if now.duration_since(entry.created_at) >= entry.ttl {
                self.stats.expired += 1;
                self.entries.remove(domain);
                return None;
            }
        }
        self.stats.misses += 1;
        None
    }

    pub fn insert(&mut self, domain: &str, ip: IpAddr, family: AddressFamily) {
        self.evict_if_full();
        self.entries.insert(
            domain.to_string(),
            DnsCacheEntry {
                domain: domain.to_string(),
                ip,
                family,
                created_at: Instant::now(),
                ttl: self.default_ttl,
                hit_count: 0,
            },
        );
    }

    pub fn insert_with_ttl(
        &mut self,
        domain: &str,
        ip: IpAddr,
        family: AddressFamily,
        ttl: Duration,
    ) {
        self.evict_if_full();
        self.entries.insert(
            domain.to_string(),
            DnsCacheEntry {
                domain: domain.to_string(),
                ip,
                family,
                created_at: Instant::now(),
                ttl,
                hit_count: 0,
            },
        );
    }

    pub fn invalidate(&mut self, domain: &str) {
        self.entries.remove(domain);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let before = self.entries.len();
        self.entries
            .retain(|_, e| now.duration_since(e.created_at) < e.ttl);
        self.stats.expired += (before - self.entries.len()) as u64;
    }

    pub fn stats(&self) -> DnsCacheStats {
        let mut s = self.stats.clone();
        s.entries = self.entries.len();
        s
    }

    fn evict_if_full(&mut self) {
        if self.entries.len() >= self.capacity {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.created_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
                self.stats.evictions += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_resolve_miss_returns_none() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        assert!(cache.resolve("example.com", AddressFamily::V4).is_none());
    }

    #[test]
    fn test_insert_and_resolve() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        let ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));
        cache.insert("example.com", ip, AddressFamily::V4);
        assert_eq!(cache.resolve("example.com", AddressFamily::V4), Some(ip));
    }

    #[test]
    fn test_family_mismatch_returns_none() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        let ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));
        cache.insert("example.com", ip, AddressFamily::V4);
        assert!(cache.resolve("example.com", AddressFamily::V6).is_none());
    }

    #[test]
    fn test_invalidate_removes_entry() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        let ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));
        cache.insert("example.com", ip, AddressFamily::V4);
        cache.invalidate("example.com");
        assert!(cache.resolve("example.com", AddressFamily::V4).is_none());
    }

    #[test]
    fn test_clear_removes_all() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        cache.insert(
            "a.com",
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            AddressFamily::V4,
        );
        cache.insert(
            "b.com",
            IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8)),
            AddressFamily::V4,
        );
        cache.clear();
        assert_eq!(cache.stats().entries, 0);
    }

    #[test]
    fn test_stats_track_hits_and_misses() {
        let mut cache = VsaDnsCache::new(100, Duration::from_secs(300));
        cache.insert(
            "a.com",
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            AddressFamily::V4,
        );
        cache.resolve("a.com", AddressFamily::V4);
        cache.resolve("a.com", AddressFamily::V4);
        cache.resolve("b.com", AddressFamily::V4);
        let s = cache.stats();
        assert_eq!(s.hits, 2);
        assert_eq!(s.misses, 1);
    }

    #[test]
    fn test_eviction_when_full() {
        let mut cache = VsaDnsCache::new(2, Duration::from_secs(300));
        cache.insert(
            "a.com",
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            AddressFamily::V4,
        );
        cache.insert(
            "b.com",
            IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)),
            AddressFamily::V4,
        );
        cache.insert(
            "c.com",
            IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3)),
            AddressFamily::V4,
        );
        assert_eq!(cache.stats().evictions, 1);
        assert_eq!(cache.stats().entries, 2);
    }
}
