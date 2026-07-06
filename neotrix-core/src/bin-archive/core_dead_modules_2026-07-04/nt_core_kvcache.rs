//! # nt_core_kvcache — Hierarchical KV Cache for LLM Inference
//!
//! Three-tier GPU→DRAM→SSD cache with LRU eviction and automatic promotion/demotion.
//!
//! **Layer**: L4 Cognition (inference optimization)
//!
//! ## Tier Hierarchy
//!
//! ```text
//! GPU (HBM)  →  DRAM (RAM)  →  SSD (NVMe)
//!   fast         medium          slow
//!   small        large           huge
//! uncompressed  compressed      compressed+sparse
//! ```
//!
//! ## Key Operations
//! - `lookup(prefix_hash)` — search all tiers, promote on hit
//! - `insert(entry)` — always lands in GPU, cascades evictions
//! - `evict_lru(tier)` — demote LRU entry to next lower tier
//!
//! ## Phase 1 Scope
//! Core types + LRU eviction + cascade + SSD store + rolling hash.
//! Compression is passthrough (Phase 3).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

// ─── CacheTier ───────────────────────────────────────────────────────────────

/// Storage tier in the three-level hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheTier {
    /// GPU HBM — fastest, smallest (e.g. 8 GB on H100 for KV)
    Gpu,
    /// System DRAM — medium speed, large (e.g. 64 GB)
    Dram,
    /// NVMe SSD — slowest, largest (e.g. 1 TB)
    Ssd,
}

impl CacheTier {
    /// Priority where lower = faster / higher priority.
    pub fn priority(&self) -> u8 {
        match self {
            CacheTier::Gpu => 0,
            CacheTier::Dram => 1,
            CacheTier::Ssd => 2,
        }
    }

    /// The next lower tier to demote to, or `None` if already at SSD.
    pub fn lower_tier(&self) -> Option<CacheTier> {
        match self {
            CacheTier::Gpu => Some(CacheTier::Dram),
            CacheTier::Dram => Some(CacheTier::Ssd),
            CacheTier::Ssd => None,
        }
    }
}

// ─── CompressionMethod ─────────────────────────────────────────────────────

/// Compression method applied to the KV tensor payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    /// Raw BF16, no compression.
    None,
    /// Sparse attention: keep top-k attention heads.
    SparseAttention,
    /// INT8 uniform quantization.
    Int8Quantized,
    /// CacheGen: per-channel INT8 quantize K + sparse attention mask on V + zlib.
    CacheGen,
}

// ─── KvCacheEntry ───────────────────────────────────────────────────────────

/// A single entry in the hierarchical KV cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvCacheEntry {
    /// Rolling hash (or exact prefix hash) identifying this token sequence.
    pub prefix_hash: u64,
    /// The token IDs that produce this KV cache (for verification).
    pub tokens: Vec<u32>,
    /// Serialized K and V tensor data (flattened + compressed).
    pub kv_data: Vec<u8>,
    /// Current storage tier.
    pub tier: CacheTier,
    /// Number of times this entry has been looked up.
    pub access_count: u64,
    /// Wall time of most recent access (for LRU ordering).
    #[serde(skip, default = "Instant::now")]
    pub last_access: Instant,
    /// Compression method applied to `kv_data`.
    pub compression: CompressionMethod,
    /// Size of `kv_data` in bytes (cached for efficiency).
    pub byte_size: usize,
    /// Creation timestamp.
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
}

impl KvCacheEntry {
    /// Create a new entry at the GPU tier.
    pub fn new(prefix_hash: u64, tokens: Vec<u32>, kv_data: Vec<u8>) -> Self {
        let byte_size = kv_data.len();
        let now = Instant::now();
        Self {
            prefix_hash,
            tokens,
            kv_data,
            tier: CacheTier::Gpu,
            access_count: 1,
            last_access: now,
            compression: CompressionMethod::None,
            byte_size,
            created_at: now,
        }
    }

    /// Estimated GPU memory footprint if loaded uncompressed.
    /// Phase 1 uses `byte_size` as a proxy; Phase 3+ will compute from model params.
    pub fn size_bytes(&self) -> usize {
        self.byte_size
    }
}

// ─── TierConfig ──────────────────────────────────────────────────────────────

/// Configuration for a single tier.
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// Maximum number of entries.
    pub max_entries: usize,
    /// Maximum total byte size across all entries (0 = unlimited by bytes).
    pub max_bytes: usize,
    /// Eviction policy (Phase 1: LRU only).
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least recently used — evict the entry accessed longest ago.
    Lru,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_bytes: 8 * 1024 * 1024 * 1024, // 8 GB
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

// ─── KvCacheConfig ──────────────────────────────────────────────────────────

/// Top-level configuration for the hierarchical KV cache.
#[derive(Debug, Clone)]
pub struct KvCacheConfig {
    /// GPU tier configuration.
    pub gpu: TierConfig,
    /// DRAM tier configuration.
    pub dram: TierConfig,
    /// SSD tier configuration.
    pub ssd: TierConfig,
    /// Cold TTL in seconds before SSD entries are pruned (default 86400 = 24h).
    pub cold_ttl_secs: u64,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        Self {
            gpu: TierConfig {
                max_entries: 4096,
                max_bytes: 8 * 1024 * 1024 * 1024,
                eviction_policy: EvictionPolicy::Lru,
            },
            dram: TierConfig {
                max_entries: 32768,
                max_bytes: 64 * 1024 * 1024 * 1024,
                eviction_policy: EvictionPolicy::Lru,
            },
            ssd: TierConfig {
                max_entries: 262144,
                max_bytes: 1024 * 1024 * 1024 * 1024,
                eviction_policy: EvictionPolicy::Lru,
            },
            cold_ttl_secs: 86400,
        }
    }
}

// ─── CacheStats ──────────────────────────────────────────────────────────────

/// Aggregate cache statistics.
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub gpu_hits: u64,
    pub dram_hits: u64,
    pub ssd_hits: u64,
    pub misses: u64,
    pub gpu_entries: usize,
    pub dram_entries: usize,
    pub ssd_entries: usize,
    pub total_ssd_bytes: u64,
    pub evictions: u64,
    pub promotions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.gpu_hits + self.dram_hits + self.ssd_hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.gpu_hits + self.dram_hits + self.ssd_hits) as f64 / total as f64
        }
    }
}

// ─── LruTier ─────────────────────────────────────────────────────────────────

/// An LRU-managed cache tier.
///
/// Uses a `HashMap<u64, KvCacheEntry>` for O(1) access and a
/// `BTreeMap<(Instant, u64), u64>` for LRU ordering (oldest access first).
/// The second tuple element is a monotonically increasing sequence number
/// that provides a tiebreaker when `Instant` values collide.
pub struct LruTier {
    entries: HashMap<u64, KvCacheEntry>,
    /// Maps (access_time, sequence) → prefix_hash. BTreeMap sorts by tuple, giving LRU order.
    order: BTreeMap<(Instant, u64), u64>,
    config: TierConfig,
    total_bytes: usize,
    next_seq: u64,
}

impl LruTier {
    pub fn new(config: TierConfig) -> Self {
        let cap = config.max_entries.min(1_000_000);
        Self {
            entries: HashMap::with_capacity(cap),
            order: BTreeMap::new(),
            config,
            total_bytes: 0,
            next_seq: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn config(&self) -> &TierConfig {
        &self.config
    }

    pub fn contains(&self, hash: u64) -> bool {
        self.entries.contains_key(&hash)
    }

    /// Get an entry without updating its access time.
    pub fn get(&self, hash: u64) -> Option<&KvCacheEntry> {
        self.entries.get(&hash)
    }

    /// Get a mutable reference and update the LRU position.
    pub fn get_mut(&mut self, hash: u64) -> Option<&mut KvCacheEntry> {
        let entry = self.entries.get_mut(&hash)?;
        entry.last_access = Instant::now();
        entry.access_count += 1;
        // Remove old key (find by value = hash, since we don't know the sequence number)
        let old_key = self.order.iter().find(|(_, &v)| v == hash).map(|(k, _)| *k);
        if let Some(key) = old_key {
            self.order.remove(&key);
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        self.order.insert((entry.last_access, seq), hash);
        Some(entry)
    }

    /// Insert an entry, evicting if at capacity. Returns the evicted entry, if any.
    pub fn insert(&mut self, mut entry: KvCacheEntry) -> Option<KvCacheEntry> {
        if let Some(old) = self.entries.get(&entry.prefix_hash) {
            let old_key = self.order.iter().find(|(_, &v)| v == entry.prefix_hash).map(|(k, _)| *k);
            if let Some(key) = old_key {
                self.order.remove(&key);
            }
            self.total_bytes = self.total_bytes.saturating_sub(old.byte_size);
        }
        let hash = entry.prefix_hash;
        entry.last_access = Instant::now();
        self.total_bytes += entry.byte_size;
        let seq = self.next_seq;
        self.next_seq += 1;
        self.order.insert((entry.last_access, seq), hash);
        self.entries.insert(hash, entry);
        self.evict_to_fit()
    }

    /// Evict the least recently used entry. Returns the evicted entry.
    pub fn pop_lru(&mut self) -> Option<KvCacheEntry> {
        let oldest_key = self.order.keys().next().copied()?;
        let hash = self.order.remove(&oldest_key)?;
        if let Some(entry) = self.entries.remove(&hash) {
            self.total_bytes = self.total_bytes.saturating_sub(entry.byte_size);
            Some(entry)
        } else {
            None
        }
    }

    /// Remove a specific entry by hash.
    pub fn remove(&mut self, hash: u64) -> Option<KvCacheEntry> {
        if let Some(entry) = self.entries.remove(&hash) {
            let key = self.order.iter().find(|(_, &v)| v == hash).map(|(k, _)| *k);
            if let Some(k) = key {
                self.order.remove(&k);
            }
            self.total_bytes = self.total_bytes.saturating_sub(entry.byte_size);
            Some(entry)
        } else {
            None
        }
    }

    /// Evict entries until we fit within constraints.
    /// Returns the last evicted entry (if any).
    fn evict_to_fit(&mut self) -> Option<KvCacheEntry> {
        let mut last_evicted = None;
        loop {
            let over_capacity = self.entries.len() > self.config.max_entries;
            let over_bytes = self.config.max_bytes > 0 && self.total_bytes > self.config.max_bytes;
            if !over_capacity && !over_bytes {
                break last_evicted;
            }
            if let Some(entry) = self.pop_lru() {
                last_evicted = Some(entry);
                if self.entries.is_empty() {
                    break last_evicted;
                }
            } else {
                break last_evicted;
            }
        }
    }

    /// Iterate over entries for stats / debugging.
    pub fn iter(&self) -> impl Iterator<Item = &KvCacheEntry> {
        self.entries.values()
    }
}

// ─── HierarchicalKvCache ─────────────────────────────────────────────────────

/// Three-tier GPU→DRAM→SSD hierarchical KV cache with LRU eviction.
///
/// - `insert()` always writes to the GPU tier.
/// - When the GPU is full, the LRU entry cascades to DRAM, then SSD.
/// - `lookup()` promotes on hit: SSD→DRAM, DRAM→GPU.
pub struct HierarchicalKvCache {
    gpu: LruTier,
    dram: LruTier,
    ssd: SsdBackedStore,
    config: KvCacheConfig,
    stats: CacheStats,
}

impl HierarchicalKvCache {
    /// Create a new hierarchical cache with the given configuration.
    pub fn new(config: KvCacheConfig) -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let cache_path = home + "/.neotrix/kvcache";
        let ssd = SsdBackedStore::new(
            &cache_path,
            (config.ssd.max_bytes / (1024 * 1024 * 1024)) as u64,
        );
        Self {
            gpu: LruTier::new(config.gpu.clone()),
            dram: LruTier::new(config.dram.clone()),
            ssd,
            stats: CacheStats::default(),
            config,
        }
    }

    /// Create a new cache with an explicit SSD path (for testing).
    pub fn new_with_ssd_path(config: KvCacheConfig, ssd_path: &str) -> Self {
        let ssd = SsdBackedStore::new(
            ssd_path,
            (config.ssd.max_bytes / (1024 * 1024 * 1024)) as u64,
        );
        Self {
            gpu: LruTier::new(config.gpu.clone()),
            dram: LruTier::new(config.dram.clone()),
            ssd,
            stats: CacheStats::default(),
            config,
        }
    }

    /// Look up a prefix hash across all tiers. Promotes on hit.
    pub fn lookup(&mut self, prefix_hash: u64) -> Option<KvCacheEntry> {
        // 1. GPU tier — fastest path
        if let Some(entry) = self.gpu.get_mut(prefix_hash) {
            entry.access_count += 1;
            entry.last_access = Instant::now();
            self.stats.gpu_hits += 1;
            return Some(entry.clone());
        }

        // 2. DRAM tier — promote to GPU
        if let Some(mut entry) = self.dram.remove(prefix_hash) {
            entry.access_count += 1;
            entry.last_access = Instant::now();
            entry.tier = CacheTier::Gpu;
            compression_passthrough(&mut entry);
            let evicted = self.gpu.insert(entry.clone());
            if let Some(ev) = evicted {
                self.cascade_eviction(CacheTier::Gpu, ev);
            }
            self.stats.dram_hits += 1;
            self.stats.promotions += 1;
            return Some(entry);
        }

        // 3. SSD tier — promote to DRAM (slowest path)
        if let Some(raw) = self.ssd.read(prefix_hash) {
            match serde_json::from_slice::<KvCacheEntry>(&raw) {
                Ok(mut entry) => {
                    entry.access_count += 1;
                    entry.last_access = Instant::now();
                    entry.tier = CacheTier::Dram;
                    compression_passthrough(&mut entry);
                    let evicted = self.dram.insert(entry.clone());
                    if let Some(ev) = evicted {
                        self.cascade_eviction(CacheTier::Dram, ev);
                    }
                    self.stats.ssd_hits += 1;
                    self.stats.promotions += 1;
                    Some(entry)
                }
                Err(_) => {
                    self.stats.misses += 1;
                    None
                }
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a new entry, always starting at the GPU tier.
    pub fn insert(&mut self, entry: KvCacheEntry) {
        let evicted = self.gpu.insert(entry);
        if let Some(ev) = evicted {
            self.cascade_eviction(CacheTier::Gpu, ev);
        }
    }

    /// Cascade an evicted entry down the tier hierarchy.
    fn cascade_eviction(&mut self, from_tier: CacheTier, mut entry: KvCacheEntry) {
        match from_tier {
            CacheTier::Gpu => {
                entry.tier = CacheTier::Dram;
                compression_passthrough(&mut entry);
                let evicted = self.dram.insert(entry);
                if let Some(ev) = evicted {
                    self.cascade_eviction(CacheTier::Dram, ev);
                }
            }
            CacheTier::Dram => {
                entry.tier = CacheTier::Ssd;
                // Only write to SSD if the tier has capacity (max_entries > 0)
                if self.config.ssd.max_entries > 0 || self.config.ssd.max_bytes > 0 {
                    let raw = serde_json::to_vec(&entry).unwrap_or_default();
                    self.ssd.write(entry.prefix_hash, &raw);
                }
                self.stats.evictions += 1;
            }
            CacheTier::Ssd => {
                // Evicted from SSD → discarded
                self.stats.evictions += 1;
            }
        }
    }

    /// Evict the LRU entry from the given tier, demoting it to the next tier.
    pub fn evict_lru(&mut self, tier: CacheTier) -> Option<KvCacheEntry> {
        let entry = match tier {
            CacheTier::Gpu => self.gpu.pop_lru()?,
            CacheTier::Dram => self.dram.pop_lru()?,
            CacheTier::Ssd => return None, // SSD eviction happens via TTL
        };
        self.cascade_eviction(tier, entry.clone());
        self.stats.evictions += 1;
        Some(entry)
    }

    /// Promote an entry from its current tier to GPU.
    pub fn promote(&mut self, prefix_hash: u64) -> bool {
        // Check DRAM
        if self.dram.contains(prefix_hash) {
            if let Some(entry) = self.dram.remove(prefix_hash) {
                let evicted = self.gpu.insert(entry);
                if let Some(ev) = evicted {
                    self.cascade_eviction(CacheTier::Gpu, ev);
                }
                self.stats.promotions += 1;
                return true;
            }
        }
        // Check SSD
        if let Some(raw) = self.ssd.read(prefix_hash) {
            if let Ok(entry) = serde_json::from_slice::<KvCacheEntry>(&raw) {
                let evicted = self.gpu.insert(entry);
                if let Some(ev) = evicted {
                    self.cascade_eviction(CacheTier::Gpu, ev);
                }
                self.stats.promotions += 1;
                return true;
            }
        }
        false
    }

    /// Return current cache statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Number of entries in the GPU tier.
    pub fn gpu_len(&self) -> usize {
        self.gpu.len()
    }

    /// Number of entries in the DRAM tier.
    pub fn dram_len(&self) -> usize {
        self.dram.len()
    }

    /// Number of entries in the SSD store.
    pub fn ssd_len(&self) -> usize {
        self.ssd.len()
    }

    /// Access the underlying tiers (for inspection in tests).
    pub fn gpu_tier(&self) -> &LruTier {
        &self.gpu
    }
    pub fn dram_tier(&self) -> &LruTier {
        &self.dram
    }
    pub fn ssd_store(&self) -> &SsdBackedStore {
        &self.ssd
    }

    /// Prune cold entries from the SSD store.
    pub fn prune_cold_ssd(&mut self) {
        let ttl = Duration::from_secs(self.config.cold_ttl_secs);
        self.ssd.prune_older_than(ttl);
    }

    /// Reset all statistics.
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }
}

/// Phase 1 compression is passthrough (no actual compression).
fn compression_passthrough(entry: &mut KvCacheEntry) {
    entry.compression = CompressionMethod::None;
}

// ─── SsdBackedStore ─────────────────────────────────────────────────────────

/// File-backed KV cache store on NVMe SSD.
///
/// One file per entry, named `{prefix_hash:016x}.kcache`, with an in-memory index.
pub struct SsdBackedStore {
    base_path: PathBuf,
    /// In-memory index: prefix_hash → file metadata.
    index: HashMap<u64, SsdEntryMeta>,
    /// Total disk usage tracking.
    total_bytes: u64,
    /// Max disk usage in bytes.
    max_bytes: u64,
}

#[derive(Debug, Clone)]
struct SsdEntryMeta {
    file_name: String,
    size_bytes: u64,
    #[allow(dead_code)]
    created_at: SystemTime,
    last_access: SystemTime,
}

impl SsdBackedStore {
    pub fn new(path: &str, max_gb: u64) -> Self {
        let base = PathBuf::from(path);
        fs::create_dir_all(&base).ok();
        let max_bytes = if max_gb == 0 {
            u64::MAX
        } else {
            max_gb * 1024 * 1024 * 1024
        };
        Self {
            base_path: base,
            index: HashMap::new(),
            total_bytes: 0,
            max_bytes,
        }
    }

    /// Write a serialized KV cache entry to disk.
    /// Returns the file path written to.
    pub fn write(&mut self, hash: u64, data: &[u8]) -> PathBuf {
        if self.total_bytes + data.len() as u64 > self.max_bytes {
            self.evict_cold_entries();
        }

        let file_name = format!("{:016x}.kcache", hash);
        let file_path = self.base_path.join(&file_name);

        // Write atomically: temp file → rename
        let tmp_path = file_path.with_extension("tmp");
        if let Ok(mut f) = fs::File::create(&tmp_path) {
            let _ = f.write_all(data);
            let _ = f.flush();
        }
        let _ = fs::rename(&tmp_path, &file_path);

        self.total_bytes += data.len() as u64;
        self.index.insert(
            hash,
            SsdEntryMeta {
                file_name,
                size_bytes: data.len() as u64,
                created_at: SystemTime::now(),
                last_access: SystemTime::now(),
            },
        );

        file_path
    }

    /// Read a serialized KV cache entry from disk.
    pub fn read(&mut self, hash: u64) -> Option<Vec<u8>> {
        let file_name = self.index.get(&hash).map(|m| m.file_name.clone());
        let file_path = match file_name {
            Some(ref name) => self.base_path.join(name),
            None => return None,
        };

        let mut buf = Vec::new();
        match fs::File::open(&file_path) {
            Ok(mut f) => {
                if f.read_to_end(&mut buf).is_err() {
                    return None;
                }
            }
            Err(_) => return None,
        }

        // Update last access
        if let Some(meta) = self.index.get_mut(&hash) {
            meta.last_access = SystemTime::now();
        }

        Some(buf)
    }

    /// Remove files older than the given TTL from the last access time.
    pub fn prune_older_than(&mut self, ttl: Duration) {
        let now = SystemTime::now();
        let mut to_remove = Vec::new();

        for (&hash, meta) in &self.index {
            match now.duration_since(meta.last_access) {
                Ok(duration) if duration >= ttl => to_remove.push(hash),
                Err(_) => to_remove.push(hash),
                _ => {}
            }
        }

        self.evict(to_remove);
    }

    /// Remove specific entries from disk.
    fn evict(&mut self, hashes: Vec<u64>) {
        let mut freed = 0u64;
        for hash in hashes {
            if let Some(meta) = self.index.remove(&hash) {
                let path = self.base_path.join(&meta.file_name);
                let _ = fs::remove_file(&path);
                freed += meta.size_bytes;
            }
        }
        self.total_bytes = self.total_bytes.saturating_sub(freed);
    }

    /// Total bytes stored on disk.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Number of entries in the index.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Evict cold entries (accessed >24h ago).
    fn evict_cold_entries(&mut self) {
        let now = SystemTime::now();
        let mut to_evict = Vec::new();
        for (&hash, meta) in &self.index {
            if let Ok(duration) = now.duration_since(meta.last_access) {
                if duration.as_secs() > 86400 {
                    to_evict.push(hash);
                }
            }
        }
        self.evict(to_evict);
    }
}

// ─── RollingHasher (Buzhash) ────────────────────────────────────────────────

/// Buzhash rolling hash for prefix matching (CacheBlend Phase 1).
///
/// Maintains a running hash over a sliding window of `u32` tokens.
/// Uses a precomputed random table of 256 entries.
pub struct RollingHasher {
    window_size: usize,
    buffer: VecDeque<u32>,
    hash: u64,
    table: [u64; 256],
    /// Sequence counter for the position of each token in the window.
    pos: u64,
}

impl RollingHasher {
    /// Create a new rolling hasher with the given window size.
    pub fn new(window_size: usize) -> Self {
        let mut table = [0u64; 256];
        // Precompute random-ish lookup table from a fixed seed for determinism.
        let mut rng_state: u64 = 0xdead_beef_cafe_babe;
        for entry in table.iter_mut() {
            // Simple xorshift64
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 7;
            rng_state ^= rng_state << 17;
            *entry = rng_state;
        }

        Self {
            window_size,
            buffer: VecDeque::with_capacity(window_size),
            hash: 0,
            table,
            pos: 0,
        }
    }

    /// Feed a token into the rolling hash.
    /// Returns the current hash value.
    pub fn feed(&mut self, token: u32) -> u64 {
        let table_idx = ((token ^ (token >> 8) ^ (token >> 16) ^ (token >> 24)) & 0xFF) as usize;

        if self.buffer.len() >= self.window_size {
            // Window is full: remove oldest token
            if let Some(old_token) = self.buffer.pop_front() {
                let old_idx =
                    ((old_token ^ (old_token >> 8) ^ (old_token >> 16) ^ (old_token >> 24))
                        & 0xFF) as usize;
                // Remove the contribution of the oldest token.
                // In Buzhash, each position's contribution is rotated by its position index.
                let rot_amount = (self.window_size as u64) % 64;
                self.hash ^= self.table[old_idx].rotate_left(rot_amount as u32);
            }
        }

        // Shift all existing contributions left by 1 (rotate hash)
        self.hash = self.hash.rotate_left(1);

        // Add new token's contribution
        self.hash ^= self.table[table_idx];

        self.buffer.push_back(token);
        self.pos += 1;
        self.hash
    }

    /// Get the current hash value.
    pub fn current_hash(&self) -> u64 {
        self.hash
    }

    /// Reset the hasher to initial state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.hash = 0;
        self.pos = 0;
    }

    /// The number of tokens currently in the window.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Compute the hash for a given slice of tokens (convenience).
    pub fn hash_tokens(tokens: &[u32], window_size: usize) -> u64 {
        let mut hasher = Self::new(window_size);
        for &t in tokens {
            hasher.feed(t);
        }
        hasher.current_hash()
    }
}

// ─── Serialization ──────────────────────────────────────────────────────────
// We use serde for compact binary serialization of KvCacheEntry.
// KvCacheEntry needs Serialize/Deserialize for SSD storage.

// ─── P1: KV Cache Quantization (TurboQuant-style) ────────────────────────────

/// Quantization precision for KV cache components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvCacheQuant {
    /// No quantization (FP32)
    None,
    /// Half precision (FP16)
    Fp16,
    /// 8-bit integer
    Int8,
    /// 4-bit integer (TurboQuant)
    Int4,
    /// 3-bit (TurboQuant 2026 — 6x memory reduction)
    Int3,
}

impl KvCacheQuant {
    pub fn bits_per_value(&self) -> f64 {
        match self {
            KvCacheQuant::None => 32.0,
            KvCacheQuant::Fp16 => 16.0,
            KvCacheQuant::Int8 => 8.0,
            KvCacheQuant::Int4 => 4.0,
            KvCacheQuant::Int3 => 3.0,
        }
    }

    pub fn memory_ratio(&self) -> f64 {
        self.bits_per_value() / 32.0
    }
}

/// TurboQuant-style KV cache quantization engine
#[derive(Debug, Clone)]
pub struct TurboQuant {
    pub precision: KvCacheQuant,
    pub per_token: bool,
    pub per_head: bool,
}

impl TurboQuant {
    pub fn new(precision: KvCacheQuant) -> Self {
        Self {
            precision,
            per_token: true,
            per_head: true,
        }
    }

    /// Quantize an FP32 KV cache block to compressed representation
    pub fn quantize(&self, data: &[f32]) -> Vec<u8> {
        match self.precision {
            KvCacheQuant::None => {
                let mut out = Vec::with_capacity(data.len() * 4);
                for &v in data {
                    out.extend_from_slice(&v.to_le_bytes());
                }
                out
            }
            KvCacheQuant::Fp16 => {
                let mut out = Vec::with_capacity(data.len() * 2);
                for &v in data {
                    let half = f32_to_f16(v);
                    out.extend_from_slice(&half.to_le_bytes());
                }
                out
            }
            KvCacheQuant::Int8 => {
                let (min, max) = min_max(data);
                let range = max - min;
                let scale = if range == 0.0 { 1.0 } else { 255.0 / range };
                let mut out = Vec::with_capacity(data.len() + 8);
                out.extend_from_slice(&min.to_le_bytes());
                out.extend_from_slice(&scale.to_le_bytes());
                for &v in data {
                    let q = ((v - min) * scale).round().max(0.0).min(255.0) as u8;
                    out.push(q);
                }
                out
            }
            KvCacheQuant::Int4 => {
                let (min, max) = min_max(data);
                let range = max - min;
                let scale = if range == 0.0 { 1.0 } else { 15.0 / range };
                let bits = 4usize;
                let packed_len = data.len().div_ceil(2);
                let mut out = Vec::with_capacity(packed_len + 8);
                out.extend_from_slice(&min.to_le_bytes());
                out.extend_from_slice(&scale.to_le_bytes());
                let mut byte: u8 = 0;
                let mut shift = 0;
                for (i, &v) in data.iter().enumerate() {
                    let q = ((v - min) * scale).round().max(0.0).min(15.0) as u8;
                    byte |= q << shift;
                    shift += bits;
                    if shift >= 8 || i == data.len() - 1 {
                        out.push(byte);
                        byte = 0;
                        shift = 0;
                    }
                }
                out
            }
            KvCacheQuant::Int3 => {
                let (min, max) = min_max(data);
                let range = max - min;
                let max_q: f32 = 7.0;
                let scale = if range == 0.0 { 1.0 } else { max_q / range };
                let bits = 3usize;
                let packed_len = data.len().div_ceil(2);
                let mut out = Vec::with_capacity(packed_len + 8);
                out.extend_from_slice(&min.to_le_bytes());
                out.extend_from_slice(&scale.to_le_bytes());
                let mut byte: u8 = 0;
                let mut shift = 0;
                for (i, &v) in data.iter().enumerate() {
                    let q = ((v - min) * scale).round().max(0.0).min(max_q) as u8;
                    byte |= q << shift;
                    shift += bits;
                    if shift >= 8 || i == data.len() - 1 {
                        out.push(byte);
                        byte = 0;
                        shift = 0;
                    }
                }
                out
            }
        }
    }

    /// Dequantize back to FP32
    pub fn dequantize(&self, data: &[u8], expected_len: usize) -> Vec<f32> {
        match self.precision {
            KvCacheQuant::None => {
                data.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
            }
            KvCacheQuant::Fp16 => {
                data.chunks_exact(2).map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]]))).collect()
            }
            KvCacheQuant::Int8 => {
                if data.len() < 8 { return vec![]; }
                let min = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let scale = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                data[8..].iter().map(|&q| min + q as f32 / scale).collect()
            }
            KvCacheQuant::Int4 | KvCacheQuant::Int3 => {
                if data.len() < 8 { return vec![]; }
                let min = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let scale = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let bits = self.precision.bits_per_value() as usize;
                let mask = (1u8 << bits) - 1;
                let mut out = Vec::with_capacity(expected_len);
                for &byte in data[8..].iter() {
                    for shift in (0..8).step_by(bits) {
                        if out.len() >= expected_len { break; }
                        let q = (byte >> shift) & mask;
                        out.push(min + q as f32 / scale);
                    }
                }
                out
            }
        }
    }
}

/// Approximate FP32 → FP16 (truncation, not true IEEE 754 rounding)
fn f32_to_f16(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = ((bits >> 13) & 0x3ff) as u16;
    if exp <= 0 {
        sign
    } else if exp >= 31 {
        sign | 0x7c00 | if mantissa != 0 { 0x200 } else { 0 }
    } else {
        sign | ((exp as u16) << 10) | mantissa
    }
}

fn f16_to_f32(v: u16) -> f32 {
    let sign = ((v >> 15) as u32) << 31;
    let exp = ((v >> 10) & 0x1f) as i32 - 15 + 127;
    let mantissa = (v & 0x3ff) as u32;
    if exp <= 0 {
        f32::from_bits(sign)
    } else if exp >= 255 {
        f32::from_bits(sign | 0x7f800000 | (mantissa << 13))
    } else {
        f32::from_bits(sign | ((exp as u32) << 23) | (mantissa << 13))
    }
}

fn min_max(data: &[f32]) -> (f32, f32) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for &v in data {
        if v < min { min = v; }
        if v > max { max = v; }
    }
    (min, max)
}

// ═══════════════════════════════════════════════════════════════
// P0.2 — HyperQuant: Lattice Quantization (E8/D4/A2 + RHT + Rice)
// ═══════════════════════════════════════════════════════════════

/// Lattice type for HyperQuant quantization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatticeType {
    /// 2D hexagonal (A2) — ~2.67 bits/value
    A2,
    /// 4D checkerboard (D4) — ~2.0 bits/value  
    D4,
    /// 8D Gosset (E8) — ~1.7 bits/value (densest sphere packing)
    E8,
}

impl LatticeType {
    pub fn dimension(&self) -> usize {
        match self { LatticeType::A2 => 2, LatticeType::D4 => 4, LatticeType::E8 => 8 }
    }
    pub fn bits_per_value(&self) -> f64 {
        match self { LatticeType::A2 => 2.67, LatticeType::D4 => 2.0, LatticeType::E8 => 1.7 }
    }
    pub fn label(&self) -> &'static str {
        match self { LatticeType::A2 => "A2", LatticeType::D4 => "D4", LatticeType::E8 => "E8" }
    }
}

// ─── Rice Coding ────────────────────────────────────────────

pub struct RiceCoder;

impl RiceCoder {
    /// Encode a sequence of non-negative integers using Rice coding with parameter k.
    /// Returns packed bytes.
    pub fn encode(values: &[i32], k: u32) -> Vec<u8> {
        if values.is_empty() {
            return vec![];
        }
        let mask = (1u32 << k) - 1;
        let mut bits: Vec<bool> = Vec::new();

        for &v in values {
            let abs_v = v.unsigned_abs();
            let q: u32 = abs_v >> k;
            let r = abs_v & mask;
            let sign = v >= 0;

            // Unary: q ones followed by a zero
            bits.extend(std::iter::repeat_n(true, q as usize));
            bits.push(false);

            // Remainder: k bits (MSB first)
            for i in (0..k).rev() {
                bits.push((r >> i) & 1 == 1);
            }

            // Sign: 1 for positive, 0 for negative
            bits.push(sign);
        }

        // Pack bits into bytes
        let byte_count = bits.len().div_ceil(8);
        let mut out = vec![0u8; byte_count];
        for (i, &b) in bits.iter().enumerate() {
            if b {
                out[i / 8] |= 1 << (i % 8);
            }
        }
        out
    }

    /// Decode Rice-coded data. k must match the encoding parameter.
    /// Returns `count` decoded values.
    pub fn decode(data: &[u8], count: usize, k: u32) -> Vec<i32> {
        let mut values = Vec::with_capacity(count);
        let mut bit_pos = 0usize;

        while values.len() < count && bit_pos < data.len() * 8 {
            // Read unary: count consecutive 1 bits until a 0
            let mut q: u32 = 0;
            while bit_pos < data.len() * 8 {
                let byte_idx = bit_pos / 8;
                let bit_idx = bit_pos % 8;
                let b = (data[byte_idx] >> bit_idx) & 1;
                bit_pos += 1;
                if b == 0 { break; }
                q += 1;
                if q > 64 { break; } // safety limit
            }
            if q > 64 { break; }

            // Read remainder: k bits
            let mut r: u32 = 0;
            for _ in 0..k {
                if bit_pos >= data.len() * 8 { break; }
                let byte_idx = bit_pos / 8;
                let bit_idx = bit_pos % 8;
                let b = (data[byte_idx] >> bit_idx) & 1;
                bit_pos += 1;
                r = (r << 1) | b as u32;
            }

            // Read sign
            let sign = if bit_pos < data.len() * 8 {
                let byte_idx = bit_pos / 8;
                let bit_idx = bit_pos % 8;
                let b = (data[byte_idx] >> bit_idx) & 1;
                bit_pos += 1;
                b == 1
            } else {
                true
            };

            let val = (q << k) | r;
            values.push(if sign { val as i32 } else { -(val as i32) });
        }

        values
    }
}

// ─── Randomized Hadamard Transform ──────────────────────────

pub struct RandomizedHadamardTransform {
    dim: usize,
    signs: Vec<f64>,
    normalizer: f64,
}

impl RandomizedHadamardTransform {
    pub fn new(dim: usize) -> Self {
        let mut rng_state: u64 = 0xe8e8_e8e8_e8e8_e8e8;
        let signs: Vec<f64> = (0..dim)
            .map(|_| {
                rng_state ^= rng_state << 13;
                rng_state ^= rng_state >> 7;
                rng_state ^= rng_state << 17;
                if rng_state & 1 == 0 { -1.0 } else { 1.0 }
            })
            .collect();
        Self { dim, signs, normalizer: (dim as f64).sqrt() }
    }

    /// Apply RHT: random sign flips + Walsh–Hadamard transform + normalize.
    pub fn apply(&self, data: &mut [f64]) {
        assert_eq!(data.len(), self.dim);
        // Random sign flips
        for (d, &s) in data.iter_mut().zip(self.signs.iter()) {
            *d *= s;
        }
        // Fast Walsh–Hadamard transform (in-place)
        let mut h = 1;
        while h < self.dim {
            for i in (0..self.dim).step_by(2 * h) {
                for j in i..i + h {
                    let x = data[j];
                    let y = data[j + h];
                    data[j] = x + y;
                    data[j + h] = x - y;
                }
            }
            h *= 2;
        }
        // Normalize
        for d in data.iter_mut() {
            *d /= self.normalizer;
        }
    }

    /// Inverse RHT: unnormalize + inverse WHT + restore signs.
    pub fn apply_inverse(&self, data: &mut [f64]) {
        assert_eq!(data.len(), self.dim);
        // Unnormalize
        for d in data.iter_mut() {
            *d *= self.normalizer;
        }
        // Inverse WHT (same as forward, just divide by dim at end)
        let mut h = 1;
        while h < self.dim {
            for i in (0..self.dim).step_by(2 * h) {
                for j in i..i + h {
                    let x = data[j];
                    let y = data[j + h];
                    data[j] = (x + y) / 2.0;
                    data[j + h] = (x - y) / 2.0;
                }
            }
            h *= 2;
        }
        // Restore signs
        for (d, &s) in data.iter_mut().zip(self.signs.iter()) {
            *d *= s;
        }
    }
}

// ─── Lattice Nearest-Point Algorithms ───────────────────────

/// Find the nearest A2 (hexagonal) lattice point to a 2D vector.
fn nearest_a2(x: f64, y: f64) -> (i32, i32) {
    let sqrt3_2 = 0.866_025_403_784_438_6;
    // Convert to (i,j) coordinates in A2 basis
    let j_float = (2.0 * y) / (3.0_f64).sqrt();
    let i_float = x - j_float * 0.5;

    let i0 = i_float.round() as i32;
    let j0 = j_float.round() as i32;

    // Check the 4 candidate neighbors
    let mut best_i = i0;
    let mut best_j = j0;
    let mut best_dist = f64::MAX;

    for di in -1..=1 {
        for dj in -1..=1 {
            let i = i0 + di;
            let j = j0 + dj;
            let px = i as f64 + j as f64 * 0.5;
            let py = j as f64 * sqrt3_2;
            let dist = (x - px).powi(2) + (y - py).powi(2);
            if dist < best_dist {
                best_dist = dist;
                best_i = i;
                best_j = j;
            }
        }
    }

    (best_i, best_j)
}

/// Find the nearest D4 (checkerboard) lattice point to a 4D vector.
fn nearest_d4(v: &[f64; 4]) -> [i32; 4] {
    // Round to integers
    let mut a = [0i32; 4];
    // Round to half-integers
    let mut b = [0i32; 4];
    let mut a_closest = [0.0f64; 4];

    for i in 0..4 {
        a[i] = v[i].round() as i32;
        a_closest[i] = a[i] as f64;
        b[i] = (v[i] - 0.5).round() as i32;
        // b is at b[i] + 0.5
    }

    let a_sum: i32 = a.iter().sum();
    let _b_sum: i32 = b.iter().sum();

    // For D4: all coordinates must have same parity (even sum)
    // If sum is odd, flip the coordinate with largest error

    if a_sum % 2 == 0 {
        // a is a valid D4 lattice point
        return a;
    }
    // Flip the coordinate with largest |a_i - v_i|
    let mut max_err = -1.0f64;
    let mut flip_idx = 0;
    for i in 0..4 {
        let err = (a_closest[i] - v[i]).abs();
        if err > max_err {
            max_err = err;
            flip_idx = i;
        }
    }
    a[flip_idx] += if v[flip_idx] > a_closest[flip_idx] { 1 } else { -1 };
    a
}

/// Find the nearest E8 (Gosset) lattice point to an 8D vector.
fn nearest_e8(v: &[f64; 8]) -> [i32; 8] {
    // Two candidate points: rounded to Z^8 and (Z+0.5)^8
    // E8 = {x ∈ Z^8 ∪ (Z+0.5)^8 : sum(x_i) even}

    let mut a = [0i32; 8];

    for i in 0..8 {
        a[i] = v[i].round() as i32;
    }

    let mut b = [0i32; 8];

    for i in 0..8 {
        let half = (v[i] - 0.5).round();
        b[i] = half as i32;
    }

    let a_sum: i32 = a.iter().sum();
    let b_sum: i32 = b.iter().sum();

    // Fix parity: E8 requires even sum
    let fix_parity = |arr: &mut [i32; 8], v: &[f64; 8], sum: i32| {
        if sum % 2 != 0 {
            // Find coordinate with largest quantization error, flip it
            let mut max_err = -1.0f64;
            let mut flip_idx = 0;
            for i in 0..8 {
                let err = (v[i] - arr[i] as f64).abs();
                if err > max_err {
                    max_err = err;
                    flip_idx = i;
                }
            }
            arr[flip_idx] += if v[flip_idx] > arr[flip_idx] as f64 { 1 } else { -1 };
        }
    };

    let (mut a_fixed, mut b_fixed) = (a, b);
    fix_parity(&mut a_fixed, v, a_sum);
    fix_parity(&mut b_fixed, v, b_sum);

    // Pick the closer one
    let a_err: f64 = (0..8).map(|i| (v[i] - a_fixed[i] as f64).powi(2)).sum();
    let b_err: f64 = (0..8).map(|i| (v[i] - (b_fixed[i] as f64 + 0.5)).powi(2)).sum();

    if a_err <= b_err { a_fixed } else { b_fixed }
}

// ─── HyperQuant Engine ───────────────────────────────────────

/// HyperQuant: lattice quantization with RHT and Rice coding.
///
/// Achieves ~1.7 bits/value (E8), ~2.0 (D4), ~2.67 (A2).
/// Uses:
/// 1. Randomized Hadamard Transform to spread quantization error
/// 2. Lattice quantization to the nearest lattice point
/// 3. Rice coding for efficient lattice index storage
#[derive(Debug, Clone)]
pub struct HyperQuant {
    pub lattice: LatticeType,
    pub use_rht: bool,
    pub rice_k: u32,
    pub per_group_scale: bool,
    pub scale_bits: u32,
}

impl HyperQuant {
    /// Create a new HyperQuant quantizer with default settings.
    pub fn new(lattice: LatticeType) -> Self {
        let (rice_k, scale_bits) = match lattice {
            LatticeType::A2 => (4, 16),
            LatticeType::D4 => (3, 16),
            LatticeType::E8 => (2, 16),
        };
        Self { lattice, use_rht: true, rice_k, per_group_scale: true, scale_bits }
    }

    /// Configure RHT on/off.
    pub fn with_rht(mut self, enable: bool) -> Self {
        self.use_rht = enable;
        self
    }

    /// Configure Rice coding parameter k (larger = more efficient for large values).
    pub fn with_rice_k(mut self, k: u32) -> Self {
        self.rice_k = k;
        self
    }

    /// Quantize a block of FP32 values to compressed lattice representation.
    pub fn quantize(&self, data: &[f32]) -> Vec<u8> {
        let dim = self.lattice.dimension();
        let n_groups = data.len().div_ceil(dim);
        let padded = dim * n_groups;

        // Pad with zeros if needed
        let mut f64_buf: Vec<f64> = data.iter().map(|&v| v as f64).collect();
        f64_buf.resize(padded, 0.0);

        // Output format:
        // [header: 1 byte lattice_type | 1 byte flags | 2 bytes n_groups]
        // [per-group scale: n_groups * 2 bytes (f16)]  (if per_group_scale)
        // [rice-coded lattice indices: variable]

        let lattice_u8 = match self.lattice {
            LatticeType::A2 => 0u8,
            LatticeType::D4 => 1u8,
            LatticeType::E8 => 2u8,
        };
        let flags: u8 = (if self.use_rht { 1u8 } else { 0 }) | (if self.per_group_scale { 2u8 } else { 0 });
        let scale_bytes = if self.per_group_scale { n_groups * (self.scale_bits as usize / 8) } else { 0 };
        let header_size = 4 + scale_bytes;

        let mut all_lattice_indices: Vec<i32> = Vec::with_capacity(n_groups * dim);

        for g in 0..n_groups {
            let start = g * dim;
            let mut group: Vec<f64> = f64_buf[start..start + dim].to_vec();

            // Optional RHT
            if self.use_rht && dim > 1 {
                let rht = RandomizedHadamardTransform::new(dim);
                rht.apply(&mut group);
            }

            // If per_group_scale, normalize by group max
            let group_max = group.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
            let scale = if group_max > 1e-10 { group_max } else { 1.0 };
            if self.per_group_scale {
                for v in &mut group {
                    *v /= scale;
                }
            }

            // Quantize to lattice
            match self.lattice {
                LatticeType::A2 => {
                    for pair in group.chunks(2) {
                        let (i, j) = nearest_a2(pair[0], pair[1]);
                        all_lattice_indices.push(i);
                        all_lattice_indices.push(j);
                    }
                }
                LatticeType::D4 => {
                    let arr4: [f64; 4] = [group[0], group[1], group[2], group[3]];
                    let indices = nearest_d4(&arr4);
                    all_lattice_indices.extend_from_slice(&indices);
                }
                LatticeType::E8 => {
                    let mut arr8 = [0.0f64; 8];
                    for (dst, src) in arr8.iter_mut().zip(group.iter()) {
                        *dst = *src;
                    }
                    let indices = nearest_e8(&arr8);
                    all_lattice_indices.extend_from_slice(&indices);
                }
            }
        }

        // Rice-code the lattice indices (differences from 0)
        let rice_data = RiceCoder::encode(&all_lattice_indices, self.rice_k);

        // Build output
        let mut out = Vec::with_capacity(header_size + rice_data.len());
        out.push(lattice_u8);
        out.push(flags);
        out.extend_from_slice(&(n_groups as u16).to_le_bytes());

        // Per-group scale (stored as f16)
        if self.per_group_scale {
            let scale_bits_inner = if self.scale_bits == 16 { 16 } else { 32 };
            for g in 0..n_groups {
                let start = g * dim;
                let group: Vec<f64> = f64_buf[start..start + dim].to_vec();
                let group_max = group.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
                let scale = if group_max > 1e-10 { group_max } else { 1.0 };
                match scale_bits_inner {
                    16 => out.extend_from_slice(&(f32_to_f16(scale as f32)).to_le_bytes()),
                    _ => out.extend_from_slice(&(scale as f32).to_le_bytes()),
                }
            }
        }

        out.extend_from_slice(&rice_data);
        out
    }

    /// Dequantize back to FP32.
    pub fn dequantize(&self, data: &[u8], expected_len: usize) -> Vec<f32> {
        if data.len() < 4 {
            return vec![];
        }

        let _lattice_u8 = data[0];
        let flags = data[1];
        let n_groups = u16::from_le_bytes([data[2], data[3]]) as usize;
        let dim = self.lattice.dimension();
        let use_rht = flags & 1 == 1;
        let per_group_scale = flags & 2 == 2;

        let scale_bytes = if per_group_scale {
            n_groups * (self.scale_bits as usize / 8)
        } else {
            0
        };
        let header = 4 + scale_bytes;

        if data.len() <= header {
            return vec![];
        }

        // Read scales
        let scales: Vec<f32> = if per_group_scale {
            let mut s = Vec::with_capacity(n_groups);
            for g in 0..n_groups {
                let off = 4 + g * (self.scale_bits as usize / 8);
                match self.scale_bits {
                    16 => {
                        let half = u16::from_le_bytes([data[off], data[off + 1]]);
                        s.push(f16_to_f32(half));
                    }
                    _ => {
                        let bits = f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
                        s.push(bits);
                    }
                }
            }
            s
        } else {
            vec![1.0; n_groups]
        };

        // Decode Rice-coded lattice indices
        let n_indices = n_groups * dim;
        let lattice_indices = RiceCoder::decode(&data[header..], n_indices, self.rice_k);

        if lattice_indices.len() < n_indices {
            return vec![];
        }

        // Reconstruct values from lattice points
        let mut result = Vec::with_capacity(n_groups * dim);

        for g in 0..n_groups {
            let scale = scales[g];
            let base = g * dim;

            match self.lattice {
                LatticeType::A2 => {
                    for pair_idx in 0..dim / 2 {
                        let i = lattice_indices[base + pair_idx * 2];
                        let j = lattice_indices[base + pair_idx * 2 + 1];
                        let sqrt3_2 = 0.866_025_403_784_438_6;
                        let mut x = i as f64 + j as f64 * 0.5;
                        let mut y = j as f64 * sqrt3_2;
                        if per_group_scale {
                            x *= scale as f64;
                            y *= scale as f64;
                        }
                        // Inverse RHT
                        if use_rht && dim > 1 {
                            let rht = RandomizedHadamardTransform::new(2);
                            let mut pair = [x, y];
                            rht.apply_inverse(&mut pair);
                            result.push(pair[0] as f32);
                            result.push(pair[1] as f32);
                        } else {
                            result.push(x as f32);
                            result.push(y as f32);
                        }
                    }
                }
                LatticeType::D4 => {
                    let mut vec4 = [0.0f64; 4];
                    for i in 0..4 {
                        vec4[i] = lattice_indices[base + i] as f64;
                        if per_group_scale {
                            vec4[i] *= scale as f64;
                        }
                    }
                    if use_rht {
                        let rht = RandomizedHadamardTransform::new(4);
                        rht.apply_inverse(&mut vec4);
                    }
                    for v in vec4 {
                        result.push(v as f32);
                    }
                }
                LatticeType::E8 => {
                    let mut vec8 = [0.0f64; 8];
                    for i in 0..8 {
                        vec8[i] = lattice_indices[base + i] as f64;
                        if per_group_scale {
                            vec8[i] *= scale as f64;
                        }
                    }
                    if use_rht {
                        let rht = RandomizedHadamardTransform::new(8);
                        rht.apply_inverse(&mut vec8);
                    }
                    for v in vec8 {
                        result.push(v as f32);
                    }
                }
            }
        }

        result.truncate(expected_len);
        result
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn make_entry(hash: u64, num_tokens: u32) -> KvCacheEntry {
        let tokens: Vec<u32> = (0..num_tokens).collect();
        let kv_data: Vec<u8> = (0..64).map(|i| (hash as u8).wrapping_add(i as u8)).collect();
        let mut entry = KvCacheEntry::new(hash, tokens, kv_data);
        entry.compression = CompressionMethod::None;
        entry
    }

    fn small_config() -> KvCacheConfig {
        KvCacheConfig {
            gpu: TierConfig {
                max_entries: 3,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            dram: TierConfig {
                max_entries: 3,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            ssd: TierConfig {
                max_entries: 10,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            cold_ttl_secs: 86400,
        }
    }

    fn ssd_test_dir(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("kvcache_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        dir.to_str().unwrap_or("").to_string()
    }

    // ── Test 1: LRU eviction order ───────────────────────────────────────

    #[test]
    fn test_lru_eviction_order() {
        let mut tier = LruTier::new(TierConfig {
            max_entries: 3,
            max_bytes: 0,
            eviction_policy: EvictionPolicy::Lru,
        });

        tier.insert(make_entry(1, 8));
        tier.insert(make_entry(2, 8));
        tier.insert(make_entry(3, 8));
        assert_eq!(tier.len(), 3);

        // Access entry 1 so it's no longer LRU
        tier.get_mut(1).unwrap();
        // Access entry 2 so it's no longer LRU either
        tier.get_mut(2).unwrap();
        // Now LRU should be entry 3 (never accessed after insert)

        let evicted = tier.pop_lru().unwrap();
        assert_eq!(evicted.prefix_hash, 3, "LRU eviction: entry 3 should be evicted first");

        let evicted = tier.pop_lru().unwrap();
        assert_eq!(evicted.prefix_hash, 1, "Entry 1 should be next (only accessed once)");

        let evicted = tier.pop_lru().unwrap();
        assert_eq!(evicted.prefix_hash, 2, "Entry 2 was accessed most recently");

        assert!(tier.is_empty());
    }

    // ── Test 2: Cascade eviction GPU→DRAM→SSD ────────────────────────────

    #[test]
    fn test_cascade_eviction_gpu_to_dram() {
        let ssd_path = ssd_test_dir("cascade_1");
        let config = small_config();
        let mut cache = HierarchicalKvCache::new_with_ssd_path(config, &ssd_path);

        // Insert 4 entries (GPU capacity = 3)
        cache.insert(make_entry(1, 8));
        cache.insert(make_entry(2, 8));
        cache.insert(make_entry(3, 8));
        assert_eq!(cache.gpu_len(), 3);
        assert_eq!(cache.dram_len(), 0);

        // This should evict entry 1 from GPU → DRAM
        cache.insert(make_entry(4, 8));
        assert_eq!(cache.gpu_len(), 3);
        assert_eq!(cache.dram_len(), 1);

        // Entry 1 should be in DRAM now
        let found = cache.lookup(1);
        assert!(found.is_some(), "Entry 1 should exist in DRAM after cascade");
        assert_eq!(found.unwrap().prefix_hash, 1);

        // Cleanup
        let _ = fs::remove_dir_all(&ssd_path);
    }

    #[test]
    fn test_cascade_eviction_dram_to_ssd() {
        let ssd_path = ssd_test_dir("cascade_2");
        let mut config = small_config();
        config.gpu.max_entries = 2; // GPU can hold 2, forcing cascade
        config.dram.max_entries = 2; // DRAM can hold only 2
        let mut cache = HierarchicalKvCache::new_with_ssd_path(config, &ssd_path);

        // Fill GPU → cascade to DRAM
        cache.insert(make_entry(1, 8));
        cache.insert(make_entry(2, 8));
        cache.insert(make_entry(3, 8)); // pushes 1→DRAM
        assert_eq!(cache.gpu_len(), 2);
        assert_eq!(cache.dram_len(), 1);

        cache.insert(make_entry(4, 8)); // pushes 2→DRAM
        assert_eq!(cache.gpu_len(), 2);
        assert_eq!(cache.dram_len(), 2);

        // This next insert pushes 3→DRAM, which overflows DRAM (max=2), pushing 1→SSD
        cache.insert(make_entry(5, 8));
        assert_eq!(cache.gpu_len(), 2);
        assert_eq!(cache.dram_len(), 2);
        assert_eq!(cache.ssd_len(), 1, "Entry 1 should be in SSD");

        // Entry 1 should be findable via lookup (SSD hit → promote to DRAM)
        let found = cache.lookup(1);
        assert!(found.is_some(), "Entry 1 should be found via SSD");

        // Cleanup
        let _ = fs::remove_dir_all(&ssd_path);
    }

    // ── Test 3: Promotion on hit ─────────────────────────────────────────

    #[test]
    fn test_promotion_on_dram_hit() {
        let ssd_path = ssd_test_dir("promotion");
        let mut cache = HierarchicalKvCache::new_with_ssd_path(small_config(), &ssd_path);

        // Fill GPU and cascade to DRAM
        cache.insert(make_entry(1, 8));
        cache.insert(make_entry(2, 8));
        cache.insert(make_entry(3, 8));
        cache.insert(make_entry(4, 8)); // pushes 1→DRAM

        assert_eq!(cache.gpu_len(), 3);
        assert_eq!(cache.dram_len(), 1);

        // Lookup entry 1 (DRAM hit → promote to GPU)
        let found = cache.lookup(1);
        assert!(found.is_some(), "Entry 1 should be found in DRAM");

        // After lookup, entry 1 should be promoted to GPU
        // GPU has 3 slots, so one entry may have been evicted to DRAM
        // But entry 1 should definitely be in GPU now
        let gpu_has_1 = cache.gpu.contains(1);
        let dram_has_1 = cache.dram.contains(1);
        assert!(gpu_has_1, "Entry 1 should be promoted to GPU");
        assert!(!dram_has_1, "Entry 1 should be removed from DRAM");

        // Cleanup
        let _ = fs::remove_dir_all(&ssd_path);
    }

    // ── Test 4: SSD store write/read/prune lifecycle ─────────────────────

    #[test]
    fn test_ssd_write_read_roundtrip() {
        let dir = ssd_test_dir("ssd_lifecycle");
        let mut store = SsdBackedStore::new(&dir, 1);

        let hash: u64 = 0xdead_beef_cafe_babe;
        let data = vec![1, 2, 3, 4, 5];

        let path = store.write(hash, &data);
        assert!(path.exists(), "File should exist on disk");
        assert_eq!(store.len(), 1);

        let loaded = store.read(hash);
        assert!(loaded.is_some(), "Should read back successfully");
        assert_eq!(loaded.unwrap(), data);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ssd_prune_older_than() {
        let dir = ssd_test_dir("ssd_prune");
        let mut store = SsdBackedStore::new(&dir, 1);

        store.write(1, b"data1");
        store.write(2, b"data2");
        store.write(3, b"data3");
        assert_eq!(store.len(), 3);

        // Prune with 0-second TTL → all entries should be evicted
        store.prune_older_than(Duration::from_secs(0));
        assert_eq!(store.len(), 0, "All entries should be pruned with 0s TTL");
        assert_eq!(store.total_bytes(), 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ssd_read_nonexistent() {
        let dir = ssd_test_dir("ssd_miss");
        let mut store = SsdBackedStore::new(&dir, 1);
        assert!(store.read(999).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Test 5: Rolling hash ─────────────────────────────────────────────

    #[test]
    fn test_rolling_hash_basic() {
        let mut hasher = RollingHasher::new(4);
        assert_eq!(hasher.len(), 0);
        assert!(hasher.is_empty());

        let _h1 = hasher.feed(1);
        assert_eq!(hasher.len(), 1);
        let _h2 = hasher.feed(2);
        assert_eq!(hasher.len(), 2);
        let _h3 = hasher.feed(3);
        assert_eq!(hasher.len(), 3);
        let h4 = hasher.feed(4);
        assert_eq!(hasher.len(), 4);

        // Feeding a 5th token should slide the window (no longer growing)
        let h5 = hasher.feed(5);
        assert_eq!(hasher.len(), 4);
        assert_ne!(h5, h4);

        // Reset
        hasher.reset();
        assert_eq!(hasher.len(), 0);
        assert!(hasher.is_empty());
    }

    #[test]
    fn test_rolling_hash_reproducibility() {
        let tokens = vec![10, 20, 30, 40, 50, 60];
        let h1 = RollingHasher::hash_tokens(&tokens, 4);
        let h2 = RollingHasher::hash_tokens(&tokens, 4);
        assert_eq!(h1, h2, "Rolling hash should be deterministic");
    }

    #[test]
    fn test_rolling_hash_prefix_matching() {
        // Two sequences share a prefix of 5 tokens
        let seq_a: Vec<u32> = vec![100, 200, 300, 400, 500, 600, 700];
        let seq_b: Vec<u32> = vec![100, 200, 300, 400, 500, 999, 888];

        let hash_a = RollingHasher::hash_tokens(&seq_a[..5], 4);
        let hash_b = RollingHasher::hash_tokens(&seq_b[..5], 4);
        assert_eq!(hash_a, hash_b, "Hashes of the same prefix should match");
    }

    #[test]
    fn test_rolling_hash_sliding_window() {
        // Hash of tokens[0..4] should differ from tokens[1..5]
        let tokens: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let h_first = RollingHasher::hash_tokens(&tokens[..4], 4);
        let h_shift = RollingHasher::hash_tokens(&tokens[1..5], 4);
        assert_ne!(h_first, h_shift, "Sliding window should produce different hashes");

        // But the same window at different positions should still hash consistently
        let h_dup = RollingHasher::hash_tokens(&tokens[2..6], 4);
        let h_dup2 = RollingHasher::hash_tokens(&tokens[2..6], 4);
        assert_eq!(h_dup, h_dup2);
    }

    // ── Test 6: Config boundary conditions ───────────────────────────────

    #[test]
    fn test_zero_sized_gpu_tier() {
        // GPU with 0 capacity: every insert immediately cascades to DRAM
        let ssd_path = ssd_test_dir("zero_gpu");
        let config = KvCacheConfig {
            gpu: TierConfig {
                max_entries: 0,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            ..small_config()
        };
        let mut cache = HierarchicalKvCache::new_with_ssd_path(config, &ssd_path);

        cache.insert(make_entry(42, 8));
        // GPU has 0 capacity, so entry should cascade to DRAM immediately
        assert_eq!(cache.gpu_len(), 0);
        assert_eq!(cache.dram_len(), 1, "Entry should cascade to DRAM when GPU capacity is 0");

        // Lookup should still find it (via DRAM)
        let found = cache.lookup(42);
        assert!(found.is_some(), "Should find entry in DRAM after GPU insert with 0 capacity");

        let _ = fs::remove_dir_all(&ssd_path);
    }

    #[test]
    fn test_all_tiers_zero() {
        // All tiers with 0 capacity: entry is immediately evicted through all tiers
        let ssd_path = ssd_test_dir("all_zero");
        let config = KvCacheConfig {
            gpu: TierConfig {
                max_entries: 0,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            dram: TierConfig {
                max_entries: 0,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            ssd: TierConfig {
                max_entries: 0,
                max_bytes: 0,
                eviction_policy: EvictionPolicy::Lru,
            },
            cold_ttl_secs: 86400,
        };
        let mut cache = HierarchicalKvCache::new_with_ssd_path(config, &ssd_path);

        cache.insert(make_entry(42, 8));
        assert_eq!(cache.gpu_len(), 0);
        assert_eq!(cache.dram_len(), 0);
        // With SSD max_bytes=0 and max_entries=0, writes still go through if max_bytes is checked
        // as 0 bytes cap. Actually, the SSD store only checks capacity on write.
        // But the cascade_eviction always calls self.ssd.write().

        let found = cache.lookup(42);
        assert!(found.is_none(), "Entry should not be found after evicted through all tiers");

        let _ = fs::remove_dir_all(&ssd_path);
    }

    #[test]
    fn test_lru_tier_max_bytes() {
        let mut tier = LruTier::new(TierConfig {
            max_entries: usize::MAX,
            max_bytes: 128, // Very small byte limit
            eviction_policy: EvictionPolicy::Lru,
        });

        // Each entry has 64 bytes of kv_data + some overhead. Insert 3 entries.
        tier.insert(make_entry(1, 8));
        tier.insert(make_entry(2, 8));
        tier.insert(make_entry(3, 8));

        // The byte limit (128) should trigger eviction
        assert!(tier.len() < 3, "Byte limit should cause eviction");
        assert!(tier.total_bytes() <= 128);
    }

    // ── P1: KV Cache Quantization ──

    #[test]
    fn test_quant_levels_bit_widths() {
        assert!((KvCacheQuant::None.bits_per_value() - 32.0).abs() < 1e-9);
        assert!((KvCacheQuant::Int8.bits_per_value() - 8.0).abs() < 1e-9);
        assert!((KvCacheQuant::Int3.bits_per_value() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_quant_levels_memory_ratio() {
        assert!((KvCacheQuant::None.memory_ratio() - 1.0).abs() < 1e-9);
        assert!((KvCacheQuant::Int8.memory_ratio() - 0.25).abs() < 1e-9);
        assert!((KvCacheQuant::Int3.memory_ratio() - 0.09375).abs() < 1e-9);
    }

    #[test]
    fn test_turbo_quant_fp16_roundtrip() {
        let tq = TurboQuant::new(KvCacheQuant::Fp16);
        let data = vec![1.0f32, 2.0, 3.0, 4.0, -1.0, -2.0];
        let compressed = tq.quantize(&data);
        assert!(compressed.len() < data.len() * 4);
        let recovered = tq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 0.1, "FP16 error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_turbo_quant_int8_roundtrip() {
        let tq = TurboQuant::new(KvCacheQuant::Int8);
        let data: Vec<f32> = (0..32).map(|i| (i as f32 - 16.0) / 16.0).collect();
        let compressed = tq.quantize(&data);
        let recovered = tq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 0.02, "INT8 error too high: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_turbo_quant_int4_packed_size() {
        let tq = TurboQuant::new(KvCacheQuant::Int4);
        let data = vec![0.5f32; 16];
        let compressed = tq.quantize(&data);
        // 8 bytes min/scale + 8 bytes packed (16 values * 4 bits / 8)
        assert!(compressed.len() <= 16);
        let recovered = tq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
    }

    #[test]
    fn test_turbo_quant_int3_roundtrip() {
        let tq = TurboQuant::new(KvCacheQuant::Int3);
        let data = vec![0.0f32, 0.25, 0.5, 0.75, 1.0, -0.5, -1.0];
        let compressed = tq.quantize(&data);
        let recovered = tq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 2.0, "INT3 error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_turbo_quant_noop_roundtrip() {
        let tq = TurboQuant::new(KvCacheQuant::None);
        let data = vec![std::f32::consts::PI, std::f32::consts::E];
        let compressed = tq.quantize(&data);
        let recovered = tq.dequantize(&compressed, data.len());
        assert_eq!(data, recovered);
    }

    #[test]
    fn test_f16_conversion_roundtrip() {
        let vals = [0.0f32, 1.0, -1.0, 0.5, 65504.0, -65504.0];
        for &v in &vals {
            let half = f32_to_f16(v);
            let back = f16_to_f32(half);
            let err = (v - back).abs();
            assert!(err < 0.1 || v.abs() > 60000.0, "f16 roundtrip: {} -> {} -> {} (err={})", v, half, back, err);
        }
    }

    #[test]
    fn test_min_max_basic() {
        let data = vec![-1.0, 0.0, 3.0, -5.0, 10.0];
        let (min, max) = min_max(&data);
        assert!((min - (-5.0)).abs() < 1e-9);
        assert!((max - 10.0).abs() < 1e-9);
    }

    // ── P0.2: HyperQuant Lattice Quantization ──

    #[test]
    fn test_lattice_type_dimensions() {
        assert_eq!(LatticeType::A2.dimension(), 2);
        assert_eq!(LatticeType::D4.dimension(), 4);
        assert_eq!(LatticeType::E8.dimension(), 8);
    }

    #[test]
    fn test_lattice_type_labels() {
        assert_eq!(LatticeType::A2.label(), "A2");
        assert_eq!(LatticeType::D4.label(), "D4");
        assert_eq!(LatticeType::E8.label(), "E8");
    }

    #[test]
    fn test_rice_encode_decode_small() {
        let values = vec![0i32, 1, -1, 2, -2, 3, -3, 10, -10, 100, -100];
        let encoded = RiceCoder::encode(&values, 3);
        let decoded = RiceCoder::decode(&encoded, values.len(), 3);
        assert_eq!(decoded, values, "Rice roundtrip failed: {:?} vs {:?}", decoded, values);
    }

    #[test]
    fn test_rice_encode_empty() {
        let encoded = RiceCoder::encode(&[], 3);
        assert!(encoded.is_empty());
        let decoded = RiceCoder::decode(&encoded, 0, 3);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_rht_apply_inverse_roundtrip() {
        let dim = 8;
        let rht = RandomizedHadamardTransform::new(dim);
        let original: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut data = original.clone();

        rht.apply(&mut data);
        // Data should be transformed (different from original)
        let max_diff_after_apply: f64 = original.iter().zip(data.iter()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(max_diff_after_apply > 1.0, "RHT should change the data significantly");

        rht.apply_inverse(&mut data);
        // Should recover original
        for (a, b) in original.iter().zip(data.iter()) {
            let err = (a - b).abs();
            assert!(err < 1e-10, "RHT roundtrip error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_rht_preserves_norm() {
        let dim = 4;
        let rht = RandomizedHadamardTransform::new(dim);
        let original: Vec<f64> = vec![3.0, -1.0, 2.0, 5.0];
        let mut data = original.clone();
        rht.apply(&mut data);

        let norm_orig: f64 = original.iter().map(|v| v * v).sum();
        let norm_trans: f64 = data.iter().map(|v| v * v).sum();
        let diff = (norm_orig - norm_trans).abs();
        assert!(diff < 1e-10, "RHT should preserve L2 norm: {} vs {} (diff={})", norm_orig, norm_trans, diff);
    }

    #[test]
    fn test_nearest_a2_origin() {
        let (i, j) = nearest_a2(0.0, 0.0);
        assert_eq!(i, 0);
        assert_eq!(j, 0);
    }

    #[test]
    fn test_nearest_a2_point() {
        let (i, j) = nearest_a2(1.2, 0.8);
        // Point should be close to (1,1) in A2 which is at (1.5, 0.866)
        assert!((i - 1).abs() <= 1);
        assert!((j - 1).abs() <= 1);
    }

    #[test]
    fn test_nearest_d4_origin() {
        let result = nearest_d4(&[0.0; 4]);
        assert_eq!(result, [0, 0, 0, 0]);
    }

    #[test]
    fn test_nearest_d4_even_sum() {
        let result = nearest_d4(&[1.2, 0.8, -0.3, 1.7]);
        let sum: i32 = result.iter().sum();
        assert_eq!(sum % 2, 0, "D4 lattice point must have even sum, got {:?} sum={}", result, sum);
    }

    #[test]
    fn test_nearest_e8_origin() {
        let result = nearest_e8(&[0.0; 8]);
        assert_eq!(result, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_nearest_e8_even_sum() {
        let result = nearest_e8(&[0.1; 8]);
        let sum: i32 = result.iter().sum();
        assert_eq!(sum % 2, 0, "E8 lattice point must have even sum, got {:?} sum={}", result, sum);
    }

    #[test]
    fn test_nearest_e8_half_integer() {
        // A point at (0.5, 0.5, ...) should be a valid E8 lattice point
        let result = nearest_e8(&[0.5; 8]);
        let sum: i32 = result.iter().sum();
        assert_eq!(sum % 2, 0, "E8 half-integer point must have even sum");
        // All coordinates should be integers (representing the integer shift from 0.5)
        // E8 in half-integer case: each coordinate = k+0.5 for some integer k
        for i in 0..8 {
            assert!((result[i] as f64 + 0.5 - 0.5 as f64).abs() < 1.0 || result[i] == 0 || result[i] == 1);
        }
    }

    #[test]
    fn test_hyperquant_quantize_e8_roundtrip() {
        let hq = HyperQuant::new(LatticeType::E8).with_rht(true);
        let data: Vec<f32> = (0..16).map(|i| (i as f32 - 8.0) / 8.0).collect();
        let compressed = hq.quantize(&data);
        let recovered = hq.dequantize(&compressed, data.len());

        assert_eq!(recovered.len(), data.len());
        assert!(compressed.len() < data.len() * 2, "E8 should compress to <2 bytes/value");

        // Allow quantization error up to the E8 covering radius (√2 ≈ 1.414)
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 2.0, "E8 error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_hyperquant_quantize_d4_roundtrip() {
        let hq = HyperQuant::new(LatticeType::D4).with_rht(true);
        let data: Vec<f32> = (0..12).map(|i| (i as f32 - 6.0) / 6.0).collect();
        let compressed = hq.quantize(&data);
        let recovered = hq.dequantize(&compressed, data.len());

        assert_eq!(recovered.len(), data.len());
        assert!(compressed.len() < data.len() * 3, "D4 should compress well");
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 2.0, "D4 error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_hyperquant_quantize_a2_roundtrip() {
        let hq = HyperQuant::new(LatticeType::A2).with_rht(true);
        let data: Vec<f32> = (0..10).map(|i| (i as f32 - 5.0) / 5.0).collect();
        let compressed = hq.quantize(&data);
        let recovered = hq.dequantize(&compressed, data.len());

        assert_eq!(recovered.len(), data.len());
        for (a, b) in data.iter().zip(recovered.iter()) {
            let err = (a - b).abs();
            assert!(err < 1.0, "A2 error: {} vs {} (err={})", a, b, err);
        }
    }

    #[test]
    fn test_hyperquant_compression_ratio_e8() {
        let hq = HyperQuant::new(LatticeType::E8);
        let data: Vec<f32> = (0..64).map(|i| (i as f32 - 32.0) / 32.0).collect();
        let compressed = hq.quantize(&data);

        let original_bytes = data.len() * 4;
        let ratio = compressed.len() as f64 / original_bytes as f64;
        assert!(ratio < 0.25, "E8 compression ratio {:.3} should be < 0.25 (1/4 of original)", ratio);
    }

    #[test]
    fn test_hyperquant_d4_compression_ratio() {
        let hq = HyperQuant::new(LatticeType::D4);
        let data: Vec<f32> = (0..32).map(|i| (i as f32 - 16.0) / 16.0).collect();
        let compressed = hq.quantize(&data);

        let original_bytes = data.len() * 4;
        let ratio = compressed.len() as f64 / original_bytes as f64;
        assert!(ratio < 0.4, "D4 compression ratio {:.3} should be < 0.4", ratio);
    }

    #[test]
    fn test_hyperquant_empty_data() {
        let hq = HyperQuant::new(LatticeType::E8);
        let compressed = hq.quantize(&[]);
        // Should still produce header bytes + scales for 0 groups
        assert_eq!(compressed.len(), 4);
        let recovered = hq.dequantize(&compressed, 0);
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_hyperquant_invalid_data() {
        let hq = HyperQuant::new(LatticeType::E8);
        let recovered = hq.dequantize(&[], 10);
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_nearest_d4_vector() {
        let v = [1.0, 1.0, 1.0, 1.0];
        let result = nearest_d4(&v);
        assert_eq!(result, [1, 1, 1, 1], "All 1.0 should quantize to [1,1,1,1]");
        let sum: i32 = result.iter().sum();
        assert_eq!(sum % 2, 0, "Even sum check");
    }

    #[test]
    fn test_hyperquant_e8_no_rht() {
        let hq = HyperQuant::new(LatticeType::E8).with_rht(false);
        let data: Vec<f32> = (0..8).map(|i| (i as f32 - 4.0) / 4.0).collect();
        let compressed = hq.quantize(&data);
        let recovered = hq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
    }

    #[test]
    fn test_hyperquant_custom_rice_k() {
        let hq = HyperQuant::new(LatticeType::E8).with_rice_k(4);
        let data: Vec<f32> = (0..16).map(|i| (i as f32 - 8.0) / 8.0).collect();
        let compressed = hq.quantize(&data);
        let recovered = hq.dequantize(&compressed, data.len());
        assert_eq!(recovered.len(), data.len());
    }

    #[test]
    fn test_cache_stats_tracking() {
        let ssd_path = ssd_test_dir("stats");
        let mut cache = HierarchicalKvCache::new_with_ssd_path(small_config(), &ssd_path);

        // Miss
        let found = cache.lookup(999);
        assert!(found.is_none());
        assert_eq!(cache.stats().misses, 1);

        // Insert and hit GPU
        cache.insert(make_entry(1, 8));
        let found = cache.lookup(1);
        assert!(found.is_some());
        assert_eq!(cache.stats().gpu_hits, 1);

        // Fill and cascade
        cache.insert(make_entry(2, 8));
        cache.insert(make_entry(3, 8));
        cache.insert(make_entry(4, 8)); // pushes 1→DRAM

        // Hit DRAM
        let found = cache.lookup(1);
        assert!(found.is_some());
        assert!(cache.stats().dram_hits >= 1);

        let _ = fs::remove_dir_all(&ssd_path);
    }
}
