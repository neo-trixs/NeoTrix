use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::obfuscation::rand_range;

pub(crate) struct DnsCache {
    entries: Mutex<HashMap<String, (Vec<SocketAddr>, Instant)>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl DnsCache {
    pub(crate) fn new() -> Self {
        DnsCache {
            entries: Mutex::new(HashMap::new()),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    pub(crate) fn resolve(&self, host: &str, port: u16) -> Result<SocketAddr, String> {
        let key = format!("{host}:{port}");
        {
            let cache = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            if let Some((addrs, expiry)) = cache.get(&key) {
                if Instant::now() < *expiry {
                    if let Some(addr) = addrs.first() {
                        self.hits.fetch_add(1, Ordering::Relaxed);
                        return Ok(*addr);
                    }
                }
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let sock_addr = (host, port)
            .to_socket_addrs()
            .map_err(|e| format!("resolve {host}:{port}: {e}"))?
            .next()
            .ok_or_else(|| format!("no address for {host}:{port}"))?;
        let ttl = Duration::from_secs(30 + rand_range(60));
        {
            let mut cache = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            cache.insert(key, (vec![sock_addr], Instant::now() + ttl));
        }
        Ok(sock_addr)
    }

    pub(crate) fn stats(&self) -> (u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }
}

pub(crate) fn dns_cache() -> &'static DnsCache {
    static CACHE: std::sync::OnceLock<DnsCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(DnsCache::new)
}
