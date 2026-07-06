# NeoTrix Hierarchical KV Cache — Design Document

**Blind Spot**: No KV cache tiering for LLM inference — every prefix is recomputed.
**Source**: LMCache (8.9k★, TensorMesh + UChicago), CacheGen compression, CacheBlend prefix sharing, Tutti (SSD-backed KV with GPU-direct I/O).
**Implementation Location**: `neotrix-core/src/nt_core_kvcache/`
**9-Layer Location**: L7 Capability (infrastructure layer) — sits above nt_io_provider, below nt_core_gwt.

---

## 1. Architecture Overview

Three-tier hierarchical KV cache with LRU eviction and automatic promotion/demotion:

```
┌─────────────────────────────────────────────────────────┐
│                    GatewayV2                              │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  Semantic Cache (embedding similarity, Phase 3)      │ │
│  └──────────────────────┬──────────────────────────────┘ │
│  ┌──────────────────────▼──────────────────────────────┐ │
│  │         HierarchicalKvCache                          │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │ │
│  │  │ GPU (HBM)    │→ │ DRAM (RAM)   │→ │ SSD (NVMe)│ │ │
│  │  │ LRU, 8GB max │  │ LRU, 64GB    │  │ ~1TB      │ │ │
│  │  │ cache lines  │  │ compressed   │  │ CacheGen  │ │ │
│  │  └──────────────┘  └──────────────┘  └───────────┘ │ │
│  └──────────────────────┬──────────────────────────────┘ │
│  ┌──────────────────────▼──────────────────────────────┐ │
│  │  CacheBlend Prefix Matcher                          │ │
│  │  (rolling hash → exact verify → blend)             │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Core Types

```rust
// neotrix-core/src/nt_core_kvcache/types.rs

use std::time::Instant;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Storage tier in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheTier {
    /// GPU HBM — fastest, smallest (e.g. 8 GB on H100 for KV)
    Gpu,
    /// System DRAM — medium speed, large (e.g. 64 GB)
    Dram,
    /// NVMe SSD — slowest, largest (e.g. 1 TB)
    Ssd,
}

impl CacheTier {
    pub fn priority(&self) -> u8 {
        match self {
            CacheTier::Gpu => 0,   // highest
            CacheTier::Dram => 1,
            CacheTier::Ssd => 2,   // lowest
        }
    }
}

/// Compression method applied to the KV tensor payload
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    /// Raw BF16, no compression
    None,
    /// 4x compression: sparse attention mask + uniform quantization to int8
    /// Reference: CacheGen (ICLR 2024, arXiv:2310.07240)
    CacheGen,
    /// Sparsification: keep only top-k attention heads (k=4 for 32-head models)
    /// Reference: H2O (NeurIPS 2023)
    Sparse,
    /// NVIDIA NVFP4: 4-bit floating point per KV element
    /// Reference: NVIDIA Blackwell FP4 KV Cache (2025)
    Nvfp4,
}

/// A single entry in the KV cache
#[derive(Debug, Clone)]
pub struct KvCacheEntry {
    /// Rolling hash (or exact prefix hash) identifying this token sequence
    pub prefix_hash: u64,
    /// The token IDs that produce this KV cache (for verification)
    pub tokens: Vec<u32>,
    /// Serialized K and V tensors.
    /// Shape: [num_layers, num_kv_heads, num_tokens, head_dim] flattened + compressed
    pub kv_tensors: Vec<u8>,
    /// Current storage tier
    pub tier: CacheTier,
    /// Number of times this entry has been looked up
    pub access_count: u64,
    /// Wall time of most recent access (for LRU ordering)
    pub last_access: Instant,
    /// Creation timestamp
    pub created_at: Instant,
    /// Compression method applied to kv_tensors
    pub compression: CompressionMethod,
    /// Model-specific metadata: number of layers, heads, head_dim
    /// Needed to deserialize kv_tensors correctly
    pub model_signature: ModelSignature,
    /// Token offset within the full context (for partial prefix matching)
    pub token_offset: usize,
    /// Total tokens in the full sequence this was extracted from
    pub total_sequence_length: usize,
}

/// Identifies which model produced this KV cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelSignature {
    pub model_name: String,
    pub num_hidden_layers: u32,
    pub num_kv_heads: u32,
    pub head_dim: u32,
    pub dtype: String, // "bf16", "fp8", "nvfp4"
}

impl KvCacheEntry {
    pub fn size_bytes(&self) -> usize {
        self.kv_tensors.len()
    }

    /// Estimated GPU memory footprint if loaded uncompressed
    pub fn uncompressed_size_bytes(&self) -> usize {
        self.total_sequence_length as usize
            * self.model_signature.num_hidden_layers as usize
            * self.model_signature.num_kv_heads as usize
            * self.model_signature.head_dim as usize
            * 2 // K + V
            * 2 // BF16 = 2 bytes per element
    }

    /// Compression ratio achieved
    pub fn compression_ratio(&self) -> f64 {
        self.uncompressed_size_bytes() as f64 / self.size_bytes().max(1) as f64
    }
}
```

---

## 3. Hierarchical Cache Engine

```rust
// neotrix-core/src/nt_core_kvcache/engine.rs

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HierarchicalKvCache {
    /// GPU-tier: small, fast, uncompressed or NVFP4
    gpu_cache: RwLock<LruCache<u64, KvCacheEntry>>,
    /// DRAM-tier: compressed (CacheGen), evicted from GPU
    dram_cache: RwLock<LruCache<u64, KvCacheEntry>>,
    /// SSD-tier: compressed (CacheGen + sparsification), evicted from DRAM
    ssd_store: Arc<SsdBackedStore>,
    /// Configuration
    config: KvCacheConfig,
    /// Total cache statistics
    stats: RwLock<CacheStats>,
}

#[derive(Debug, Clone)]
pub struct KvCacheConfig {
    /// Max GPU cache entries (default 4096, ~8GB with 2MB entries)
    pub gpu_capacity: NonZeroUsize,
    /// Max DRAM cache entries (default 32768, ~64GB with 2MB compressed entries)
    pub dram_capacity: NonZeroUsize,
    /// SSD storage path
    pub ssd_path: String,
    /// Max SSD storage in GB (default 1024)
    pub ssd_max_gb: u64,
    /// Chunk size in tokens for CacheBlend (default 256)
    pub chunk_size: usize,
    /// Cold TTL in seconds before SSD eviction (default 86400 = 24h)
    pub ssd_cold_ttl_secs: u64,
    /// Compression method for DRAM tier (default CacheGen)
    pub dram_compression: CompressionMethod,
    /// Compression method for SSD tier (default CacheGen + Sparse)
    pub ssd_compression: CompressionMethod,
    /// Enable CacheBlend prefix sharing (default true)
    pub cache_blend_enabled: bool,
    /// Rolling hash window size for fuzzy prefix matching (default 8)
    pub rolling_hash_window: usize,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        Self {
            gpu_capacity: NonZeroUsize::new(4096).unwrap(),
            dram_capacity: NonZeroUsize::new(32768).unwrap(),
            ssd_path: std::env::var("HOME").unwrap_or_default() + "/.neotrix/kvcache",
            ssd_max_gb: 1024,
            chunk_size: 256,
            ssd_cold_ttl_secs: 86400,
            dram_compression: CompressionMethod::CacheGen,
            ssd_compression: CompressionMethod::CacheGen,
            cache_blend_enabled: true,
            rolling_hash_window: 8,
        }
    }
}

impl HierarchicalKvCache {
    pub fn new(config: KvCacheConfig) -> Self {
        std::fs::create_dir_all(&config.ssd_path).ok();
        Self {
            gpu_cache: RwLock::new(LruCache::new(config.gpu_capacity)),
            dram_cache: RwLock::new(LruCache::new(config.dram_capacity)),
            ssd_store: Arc::new(SsdBackedStore::new(&config.ssd_path, config.ssd_max_gb)),
            config,
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Look up a prefix hash across all tiers.
    /// Promotes on hit: SSD → DRAM, DRAM → GPU.
    pub async fn lookup(&self, prefix_hash: u64, model: &ModelSignature) -> Option<KvCacheEntry> {
        // 1. Check GPU tier (fast path, no deserialization needed)
        {
            let mut gpu = self.gpu_cache.write().await;
            if let Some(entry) = gpu.get_mut(&prefix_hash) {
                entry.access_count += 1;
                entry.last_access = Instant::now();
                self.stats.write().await.gpu_hits += 1;
                return Some(entry.clone());
            }
        }

        // 2. Check DRAM tier (needs GPU decompression on load)
        {
            let mut dram = self.dram_cache.write().await;
            if let Some(entry) = dram.get_mut(&prefix_hash) {
                entry.access_count += 1;
                entry.last_access = Instant::now();
                // Promote: decompress and move to GPU
                let mut gpu_entry = entry.clone();
                gpu_entry.tier = CacheTier::Gpu;
                // Decompress if compressed (cachegen/decompress → bf16)
                if gpu_entry.compression != CompressionMethod::None {
                    gpu_entry.kv_tensors = cachegen_decompress(&gpu_entry.kv_tensors, &gpu_entry.model_signature);
                    gpu_entry.compression = CompressionMethod::None;
                }
                let hash = prefix_hash;
                self.gpu_cache.write().await.put(hash, gpu_entry);
                self.stats.write().await.dram_hits += 1;
                return Some(entry.clone()); // return DRAM copy (caller loads from GPU)
            }
        }

        // 3. Check SSD tier (slowest path, needs I/O + decompression)
        {
            if let Some(raw) = self.ssd_store.read(prefix_hash).await {
                let entry: KvCacheEntry = bincode::deserialize(&raw).ok()?;
                // Promote to DRAM
                let mut dram_entry = entry.clone();
                dram_entry.tier = CacheTier::Dram;
                let hash = prefix_hash;
                self.dram_cache.write().await.put(hash, dram_entry);
                self.stats.write().await.ssd_hits += 1;
                return Some(entry);
            }
        }

        self.stats.write().await.misses += 1;
        None
    }

    /// Insert a new entry, always starting at GPU tier.
    /// If GPU is full, evict oldest GPU entry → DRAM (compressed).
    /// If DRAM is full, evict oldest DRAM → SSD (compressed + sparsified).
    pub async fn insert(&self, mut entry: KvCacheEntry) {
        entry.tier = CacheTier::Gpu;
        entry.last_access = Instant::now();
        entry.created_at = Instant::now();

        let hash = entry.prefix_hash;
        let mut gpu = self.gpu_cache.write().await;

        if gpu.len() >= gpu.capacity().get() {
            // Evict oldest from GPU → DRAM
            if let Some((evicted_hash, evicted_entry)) = gpu.pop_lru() {
                self.demote_to_dram(evicted_hash, evicted_entry).await;
            }
        }

        gpu.put(hash, entry);
    }

    /// Demote a GPU entry to DRAM (compress with CacheGen)
    async fn demote_to_dram(&self, hash: u64, mut entry: KvCacheEntry) {
        entry.tier = CacheTier::Dram;
        if self.config.dram_compression != CompressionMethod::None {
            entry.kv_tensors = cachegen_compress(&entry.kv_tensors, &entry.model_signature);
            entry.compression = self.config.dram_compression;
        }

        let mut dram = self.dram_cache.write().await;
        if dram.len() >= dram.capacity().get() {
            if let Some((evicted_hash, evicted_entry)) = dram.pop_lru() {
                self.demote_to_ssd(evicted_hash, evicted_entry).await;
            }
        }
        dram.put(hash, entry);
    }

    /// Demote a DRAM entry to SSD (compress with CacheGen + sparsification)
    async fn demote_to_ssd(&self, hash: u64, mut entry: KvCacheEntry) {
        entry.tier = CacheTier::Ssd;
        // Apply sparsification (keep only top-k attention heads)
        if self.config.ssd_compression == CompressionMethod::CacheGen {
            entry.kv_tensors = cachegen_compress(&entry.kv_tensors, &entry.model_signature);
            // Then sparsify: remove low-magnitude KV positions
            entry.kv_tensors = sparsify_kv(&entry.kv_tensors, 0.7); // keep 70% of positions
            entry.compression = CompressionMethod::CacheGen;
        }

        let serialized = bincode::serialize(&entry).unwrap();
        self.ssd_store.write(hash, &serialized).await;
    }

    /// CacheBlend: given a cached entry (with KV for prefix tokens) and new suffix tokens,
    /// compute blended KV: reuse cached K,V for prefix + compute new K,V for suffix.
    /// Returns the full KV cache for the combined sequence.
    pub fn blend(&self, cached: &KvCacheEntry, new_tokens: &[u32]) -> Vec<u8> {
        if !self.config.cache_blend_enabled {
            return cached.kv_tensors.clone(); // fallback to full cached KV
        }

        let cached_len = cached.tokens.len();
        let new_len = new_tokens.len();

        // Deserialize cached KV tensors
        let (cached_k, cached_v) = deserialize_kv(&cached.kv_tensors, &cached.model_signature);

        // Allocate output tensors: [cached_len + new_len, num_heads, head_dim]
        let mut blended_k = cached_k.clone();
        let mut blended_v = cached_v.clone();

        // Extend K,V tensors with zeros for new tokens (will be filled by inference)
        // In practice, the model runner only computes attention for new token positions,
        // while the cached prefix positions are masked out in the causal attention.
        blended_k.extend(/* zeros for new_tokens */);
        blended_v.extend(/* zeros for new_tokens */);

        // Return re-serialized full KV cache
        serialize_kv(&blended_k, &blended_v, &cached.model_signature)
    }

    /// Return current cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}
```

---

## 4. CacheGen Compression Algorithm

Reference: [CacheGen: KV Cache Compression and Streaming for Fast Large Language Model Serving](https://arxiv.org/abs/2310.07240) (SIGCOMM 2024).

### Key Insight

Attention patterns are highly sparse: many positions contribute near-zero weight. CacheGen achieves **3.0-4.3x compression** with <1% quality degradation by:

1. **Uniform quantization of K to int8** per channel (per-head, per-layer): 2x compression
2. **Sparse attention mask on V**: identify and zero out V positions with low attention scores, then compress the sparse residual with zlib: 1.5-2.15x additional

### Implementation

```rust
// neotrix-core/src/nt_core_kvcache/compression.rs

/// CacheGen compression pipeline
///
/// Input: raw KV tensor [2, num_layers, num_kv_heads, num_tokens, head_dim] (BF16)
///    → 2 * L * H * S * D * 2 bytes
///
/// Step 1: Quantize K from BF16 → INT8 per-channel (min-max asymmetric)
///   scale = (max - min) / 255, zero_point = -min / scale
///   K_int8 = clamp(round((K_fp32 - min) / scale), 0, 255)
///   Store: K_int8 (S * D bytes) + scale + zero_point (2 * H * 4 bytes each)
///
/// Step 2: Compute attention scores with a lightweight proxy (single head, cached Q)
///   scores = softmax(Q @ K_int8^T / sqrt(d))
///   Keep top-70% positions by cumulative score mass
///
/// Step 3: For dropped V positions, store a 1-bit mask + run-length encode
///   Residual: encode only kept positions' V values (INT8 quantized)
///
/// Step 4: zlib compress the combined bitstream
pub fn cachegen_compress(raw: &[u8], sig: &ModelSignature) -> Vec<u8> {
    let (k_tensor, v_tensor) = deserialize_raw_kv(raw, sig);
    let num_tokens = k_tensor.shape()[2]; // S

    // Step 1: INT8 per-channel quantization of K
    let (k_quant, k_scale, k_zp) = quantize_int8_per_channel(&k_tensor);

    // Step 2: Lightweight attention scoring
    // Use a proxy Q (learned or random projection) to estimate attention scores
    let proxy_q = get_proxy_query(sig); // [1, num_kv_heads, 1, head_dim]
    let scores = compute_attention_scores(&proxy_q, &k_quant, k_scale, k_zp);

    // Step 3: Select top tokens by cumulative attention mass
    // Target: keep 70% of tokens by default
    let (mask, kept_indices) = select_top_tokens(&scores, 0.70);

    // Step 4: INT8 quantize kept V positions
    let v_kept = select_indices(&v_tensor, &kept_indices);
    let (v_quant, v_scale, v_zp) = quantize_int8_per_channel(&v_kept);

    // Step 5: Pack everything into a bitstream
    // [header: K_scale(32bit * H * L) + K_zp + V_scale + V_zp + mask(rle)
    //  + K_int8(H * L * S * D) + V_int8(H * L * kept * D)]
    // Metadata overhead: ~2 * 2 * H * 4 bytes ≈ small
    let bitstream = pack_bitstream(
        &k_quant, k_scale, k_zp, &v_quant, v_scale, v_zp, &mask,
        sig, num_tokens, kept_indices.len(),
    );

    // Step 6: zlib compression (adds 1.5-2x on top of quantization)
    let mut compressed = Vec::new();
    let mut encoder = zlib::Encoder::new(&mut compressed, zlib::Compression::best());
    encoder.write_all(&bitstream).unwrap();
    encoder.finish().unwrap();

    compressed
}

/// Decompress: reverse of the above
pub fn cachegen_decompress(compressed: &[u8], sig: &ModelSignature) -> Vec<u8> {
    // zlib decompress
    let mut bitstream = Vec::new();
    let mut decoder = zlib::Decoder::new(&bitstream);
    decoder.read_to_end(&mut compressed).unwrap();

    // Unpack header + tensors
    let (k_quant, k_scale, k_zp, v_quant, v_scale, v_zp, mask, num_tokens) =
        unpack_bitstream(&bitstream, sig);

    // Dequantize K
    let k_deq = dequantize_int8_per_channel(&k_quant, &k_scale, &k_zp);

    // Zero-initialize V, fill kept positions
    let mut v_deq = Tensor::zeros(&[sig.num_hidden_layers as _, sig.num_kv_heads as _, num_tokens, sig.head_dim as _]);
    let kept_indices = decode_mask(&mask, num_tokens, 0.70);
    let v_kept = dequantize_int8_per_channel(&v_quant, &v_scale, &v_zp);
    scatter(&mut v_deq, &kept_indices, &v_kept);

    serialize_kv(&k_deq, &v_deq, sig)
}

/// Compression test
#[test]
fn test_cachegen_compress_decompress_roundtrip() {
    let sig = ModelSignature {
        model_name: "test".into(),
        num_hidden_layers: 1,
        num_kv_heads: 4,
        head_dim: 128,
        dtype: "bf16".into(),
    };
    // Create random KV tensor: 1 layer, 4 heads, 512 tokens, 128 dim
    let raw = generate_random_kv(1, 4, 512, 128);

    let compressed = cachegen_compress(&raw, &sig);
    let decompressed = cachegen_decompress(&compressed, &sig);

    // Quality: cosine similarity > 0.99
    let sim = cosine_similarity(&raw, &decompressed);
    assert!(sim > 0.99, "cosine similarity = {}", sim);

    let ratio = raw.len() as f64 / compressed.len() as f64;
    println!("Compression ratio: {:.2}x", ratio);
    assert!(ratio > 3.0, "ratio {:.2} < 3.0x", ratio);
}
```

---

## 5. CacheBlend Prefix Matching

Reference: [CacheBlend: Fast Large Language Model Serving with Cached Prompt and Blended Prefix](https://arxiv.org/abs/2405.16444) (NSDI 2025).

### Algorithm

Two-phase: **Rapid fuzzy matching** (rolling hash) then **exact token verification**.

```rust
// neotrix-core/src/nt_core_kvcache/cacheblend.rs

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Rolling hash-based prefix matching.
/// Phase 1: compute rolling hash window of M tokens → compare to indexed hashes.
/// Phase 2: on match, verify exact token sequence.
pub struct PrefixMatcher {
    /// For each stored KV entry, store its rolling hash signatures
    /// hash_value → Vec<(prefix_hash, token_offset, window_start, window_end)>
    index: HashMap<u64, Vec<HashIndexEntry>>,
    /// Window size in tokens (default 8)
    window_size: usize,
}

#[derive(Debug, Clone)]
struct HashIndexEntry {
    pub entry_hash: u64,       // KvCacheEntry.prefix_hash
    pub token_offset: usize,    // where this window starts in the full sequence
    pub window_content: Vec<u32>, // actual tokens in this window (for verification)
}

impl PrefixMatcher {
    pub fn new(window_size: usize) -> Self {
        Self {
            index: HashMap::new(),
            window_size,
        }
    }

    /// Index a new KvCacheEntry's tokens for matching
    pub fn index_entry(&mut self, entry: &KvCacheEntry) {
        let tokens = &entry.tokens;
        if tokens.len() < self.window_size {
            return;
        }

        for window in tokens.windows(self.window_size) {
            let hash = rolling_hash(window);
            self.index.entry(hash).or_default().push(HashIndexEntry {
                entry_hash: entry.prefix_hash,
                token_offset: 0, // TODO: track offset for partial matches
                window_content: window.to_vec(),
            });
        }
    }

    /// Find the longest prefix match for a query token sequence.
    /// Returns the matching entry and the number of prefix tokens that match.
    pub fn find_longest_prefix(&self, query: &[u32]) -> Option<(u64, usize)> {
        if query.len() < self.window_size {
            return None;
        }

        // Compute rolling hashes for query
        let query_hashes: Vec<u64> = query.windows(self.window_size)
            .map(rolling_hash)
            .collect();

        // Find candidate entries with matching first window
        let first_hash = query_hashes[0];
        let candidates = self.index.get(&first_hash)?;

        let mut best_match: Option<(u64, usize)> = None;

        for candidate in candidates {
            // Phase 2: exact token verification
            if candidate.window_content != query[..self.window_size] {
                continue;
            }

            // Extend match: count how many consecutive tokens match
            let match_len = self.extend_match(candidate, query);
            if match_len >= self.window_size {
                let is_better = best_match
                    .as_ref()
                    .map_or(true, |(_, best_len)| match_len > *best_len);
                if is_better {
                    best_match = Some((candidate.entry_hash, match_len));
                }
            }
        }

        best_match
    }

    /// Extend a candidate match by checking subsequent tokens
    fn extend_match(&self, candidate: &HashIndexEntry, query: &[u32]) -> usize {
        // Verify the rest of the query against the stored entry
        // In practice, load the full entry tokens and find the longest common prefix
        // For now, just return the window size as a conservative estimate
        self.window_size
    }
}

/// Rolling hash function (polynomial rolling hash with base 131, mod 2^64)
fn rolling_hash(window: &[u32]) -> u64 {
    const BASE: u64 = 131;
    window.iter().fold(0u64, |hash, &tok| {
        hash.wrapping_mul(BASE).wrapping_add(tok as u64)
    })
}

/// Compute the number of common prefix tokens between two sequences
pub fn common_prefix_len(a: &[u32], b: &[u32]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}
```

---

## 6. SSD-Backed Store

```rust
// neotrix-core/src/nt_core_kvcache/ssd_store.rs

use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Semaphore;

/// File-backed KV cache store on NVMe SSD.
/// Uses one file per entry, with an index file for fast lookup.
pub struct SsdBackedStore {
    base_path: PathBuf,
    /// Per-entry index: prefix_hash → (file_path, size_bytes, created_at)
    index: RwLock<HashMap<u64, SsdEntryMeta>>,
    /// Total disk usage tracking
    total_bytes: AtomicU64,
    /// Max disk usage in bytes
    max_bytes: u64,
    /// Concurrency limiter (max 4 concurrent reads/writes)
    io_semaphore: Semaphore,
}

#[derive(Debug, Clone)]
struct SsdEntryMeta {
    file_name: String,
    size_bytes: u64,
    created_at: std::time::SystemTime,
    last_access: std::time::SystemTime,
}

impl SsdBackedStore {
    pub fn new(path: &str, max_gb: u64) -> Self {
        let base = PathBuf::from(path);
        std::fs::create_dir_all(&base).ok();
        Self {
            base_path: base,
            index: RwLock::new(HashMap::new()),
            total_bytes: AtomicU64::new(0),
            max_bytes: max_gb * 1024 * 1024 * 1024,
            io_semaphore: Semaphore::new(4), // 4 concurrent I/Os
        }
    }

    /// Write a serialized KV cache entry to disk
    pub async fn write(&self, hash: u64, data: &[u8]) {
        let _permit = self.io_semaphore.acquire().await.unwrap();

        // Check disk quota
        if self.total_bytes.load(Ordering::Relaxed) + data.len() as u64 > self.max_bytes {
            self.evict_cold_entries().await;
        }

        let file_name = format!("{:016x}.kcache", hash);
        let file_path = self.base_path.join(&file_name);

        // Write atomically: write to temp, then rename
        let tmp_path = file_path.with_extension("tmp");
        tokio::fs::write(&tmp_path, data).await.unwrap();
        tokio::fs::rename(&tmp_path, &file_path).await.unwrap();

        self.total_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.index.write().await.insert(hash, SsdEntryMeta {
            file_name,
            size_bytes: data.len() as u64,
            created_at: std::time::SystemTime::now(),
            last_access: std::time::SystemTime::now(),
        });
    }

    /// Read a serialized KV cache entry from disk
    pub async fn read(&self, hash: u64) -> Option<Vec<u8>> {
        let file_path = {
            let index = self.index.read().await;
            let meta = index.get(&hash)?;
            if let Ok(elapsed) = meta.last_access.elapsed() {
                if elapsed.as_secs() > 86400 {
                    drop(index);
                    self.evict(vec![hash]).await;
                    return None;
                }
            }
            self.base_path.join(&meta.file_name)
        };

        let _permit = self.io_semaphore.acquire().await.unwrap();
        let data = tokio::fs::read(&file_path).await.ok()?;

        // Update last access time
        if let Some(meta) = self.index.write().await.get_mut(&hash) {
            meta.last_access = std::time::SystemTime::now();
        }

        Some(data)
    }

    /// Evict cold entries (oldest by last_access, not accessed in 24h)
    async fn evict_cold_entries(&self) {
        let now = std::time::SystemTime::now();
        let mut to_evict = Vec::new();

        let index = self.index.read().await;
        for (&hash, meta) in index.iter() {
            if let Ok(duration) = now.duration_since(meta.last_access) {
                if duration.as_secs() > 86400 { // 24h
                    to_evict.push(hash);
                }
            }
        }
        drop(index);
        self.evict(to_evict).await;
    }

    /// Remove specific entries from disk
    async fn evict(&self, hashes: Vec<u64>) {
        let mut freed: u64 = 0;
        let mut index = self.index.write().await;

        for hash in hashes {
            if let Some(meta) = index.remove(&hash) {
                let path = self.base_path.join(&meta.file_name);
                tokio::fs::remove_file(path).await.ok();
                freed += meta.size_bytes;
            }
        }

        self.total_bytes.fetch_sub(freed, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ssd_write_read_roundtrip() {
        let dir = std::env::temp_dir().join("kvcache_test");
        let store = SsdBackedStore::new(dir.to_str().unwrap(), 1);

        let hash: u64 = 0xdeadbeef;
        let data = vec![1, 2, 3, 4, 5];

        store.write(hash, &data).await;
        let loaded = store.read(hash).await.unwrap();
        assert_eq!(loaded, data);

        // Cleanup
        tokio::fs::remove_dir_all(dir).await.ok();
    }
}
```

---

## 7. Sparsification Algorithm (H2O-style)

Reference: [H2O: Heavy-Hitter Oracle for Efficient Generative Inference of Large Language Models](https://arxiv.org/abs/2306.14048) (NeurIPS 2023).

```rust
/// Sparsify KV tensor: keep only top-k fraction of positions by attention score magnitude.
/// Keeps the "heavy hitters" — positions with consistently high attention scores.
///
/// Algorithm:
/// 1. Compute cumulative attention score for each position across all heads
/// 2. Sort positions by cumulative score
/// 3. Keep the top `keep_fraction` of positions
/// 4. Return a mask + compacted tensor
pub fn sparsify_kv(kv_bytes: &[u8], keep_fraction: f64) -> Vec<u8> {
    let (k, v) = deserialize_raw_kv(kv_bytes);
    let num_tokens = k.shape()[2];     // S
    let num_layers = k.shape()[0] as usize;
    let num_heads = k.shape()[1] as usize;
    let head_dim = k.shape()[3] as usize;

    // Step 1: Compute per-position attention scores
    let mut scores: Vec<(usize, f64)> = (0..num_tokens)
        .map(|pos| {
            let score: f64 = (0..num_layers)
                .flat_map(|l| (0..num_heads).map(move |h| (l, h)))
                .map(|(l, h)| {
                    // L2 norm of K[pos] across all heads and layers
                    let k_slice = get_k_slice(&k, l, h, pos);
                    l2_norm(k_slice)
                })
                .sum();
            (pos, score)
        })
        .collect();

    // Step 2: Sort by score descending, keep top fraction
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let keep_count = (num_tokens as f64 * keep_fraction).ceil() as usize;
    let keep_count = keep_count.max(1).min(num_tokens);

    let kept_positions: std::collections::BTreeSet<usize> = scores
        .iter()
        .take(keep_count)
        .map(|(pos, _)| *pos)
        .collect();

    // Step 3: Build compacted K,V (only kept positions)
    let compact_k = compact_tensor(&k, &kept_positions);
    let compact_v = compact_tensor(&v, &kept_positions);

    // Step 4: Serialize + store mask
    let mut output = Vec::new();
    output.extend(serialize_mask(&kept_positions, num_tokens));
    output.extend(serialize_compact_kv(&compact_k, &compact_v));

    output
}
```

---

## 8. Integration Points

### 8.1 GatewayV2 — KV Cache Injection

In `gateway.rs`, before making an LLM call:

```rust
pub async fn complete_with_selection(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
    // 1. Compute rolling hash of prompt prefix
    let prompt_hash = rolling_hash(&request.messages.iter()
        .flat_map(|m| m.content.as_bytes())
        .map(|b| *b as u32)
        .collect::<Vec<u32>>());

    // 2. Check hierarchical KV cache
    if let Some(cached) = self.kv_cache.lookup(prompt_hash, &self.model_sig()).await {
        // 2a. CacheBlend: compute only new tokens
        debug!("KV cache HIT: blending {} cached tokens + {} new tokens",
            cached.tokens.len(), request.max_tokens);
        let blended_kv = self.kv_cache.blend(&cached, &new_tokens());
        // 2b. Inject blended KV into the inference call
        // (actual mechanism depends on LLM backend — vLLM prefix caching API, etc.)
        let response = self.complete_with_kv_cache(request, &blended_kv).await?;

        // 2c. Update cache access stats
        self.kv_cache.stats().await;
        return Ok(response);
    }

    // 3. Cache miss: normal LLM call
    let response = self.complete_inner(request).await?;

    // 4. Cache the new KV from this response
    if let Some(kv_data) = response.extract_kv_cache() {
        let entry = KvCacheEntry {
            prefix_hash: prompt_hash,
            tokens: prompt_tokens(),
            kv_tensors: kv_data,
            tier: CacheTier::Gpu,
            access_count: 1,
            last_access: Instant::now(),
            created_at: Instant::now(),
            compression: CompressionMethod::None,
            model_signature: self.model_sig(),
            token_offset: 0,
            total_sequence_length: prompt_tokens().len() + response.completion_tokens,
        };
        self.kv_cache.insert(entry).await;
    }

    Ok(response)
}
```

### 8.2 KB Search — CacheBlend for RAG Prefix Sharing

When multiple RAG queries share the same document prefix, compute KV once and blend:

```rust
impl KnowledgeBase {
    pub async fn search_with_kv_cache(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        // Query embedding
        let query_embed = self.embed(query).await?;

        // Check if documents have KV cache entries
        let doc_hash = rolling_hash(query.as_bytes().chunks(64).next().unwrap_or(b""));
        if let Some(cached) = self.kv_cache.lookup(doc_hash, &self.model_sig).await {
            // The KB itself has a KV cache from a previous RAG query
            // Only need to compute attention for new query tokens
            debug!("RAG prefix KV cache HIT, blend with {} query tokens", query.len());
        }

        // Normal search
        self.search_fused(query, top_k).await
    }
}
```

### 8.3 nt_core_cache — Triple Cache Hierarchy

The KV cache sits as the middle layer in a three-tier cache stack:

```
┌─────────────────────────────────────────────┐
│  Layer 1: Semantic Cache (nt_core_cache)    │
│  Exact + embedding-similarity cache hits    │
│  Key: (prompt_hash, model) → response       │
│  Hit: 100% LLM call saved                   │
├─────────────────────────────────────────────┤
│  Layer 2: KV Cache (nt_core_kvcache) ◄──   │
│  Hierarchical prefix KV reuse               │
│  Hit: prefill computation saved (70-99%)    │
├─────────────────────────────────────────────┤
│  Layer 3: LLM Inference (GatewayV2)         │
│  Full compute path                          │
└─────────────────────────────────────────────┘
```

---

## 9. File Layout

```
neotrix-core/src/nt_core_kvcache/
├── mod.rs              # re-exports, cache factory
├── types.rs            # CacheTier, KvCacheEntry, CompressionMethod, ModelSignature
├── config.rs           # KvCacheConfig
├── engine.rs           # HierarchicalKvCache (LRU per tier, eviction, promotion)
├── compression.rs      # CacheGen compress/decompress (+ roundtrip tests)
├── cacheblend.rs       # PrefixMatcher (rolling hash + exact verify + blend)
├── ssd_store.rs        # SsdBackedStore (file-backed NVMe storage)
├── sparsify.rs         # H2O-style sparsification
├── stats.rs            # CacheStats (hit rates per tier, miss ratio)
└── serialization.rs    # KV tensor serializer/deserializer helpers
```

---

## 10. CacheStats

```rust
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
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.gpu_hits + self.dram_hits + self.ssd_hits + self.misses;
        if total == 0 { 0.0 } else {
            (self.gpu_hits + self.dram_hits + self.ssd_hits) as f64 / total as f64
        }
    }

    pub fn avg_promotion_latency(&self) -> f64 {
        // DRAM→GPU decompression: ~50μs per MB
        // SSD→DRAM I/O: ~500μs per MB (NVMe)
        // Combined: ~550μs per MB
        550.0
    }
}
```

---

## 11. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_cache_lru_eviction() {
        let config = KvCacheConfig {
            gpu_capacity: NonZeroUsize::new(2).unwrap(),
            ..Default::default()
        };
        let cache = HierarchicalKvCache::new(config);

        let entry1 = make_entry(0x1, 16);
        let entry2 = make_entry(0x2, 16);
        let entry3 = make_entry(0x3, 16);

        cache.insert(entry1).await;
        cache.insert(entry2).await;
        cache.insert(entry3).await; // evicts entry1 → DRAM

        assert!(cache.lookup(0x1, &MODEL_SIG).await.is_some()); // DRAM hit
        assert!(cache.lookup(0x3, &MODEL_SIG).await.is_some()); // GPU hit
        assert_eq!(cache.gpu_cache.read().await.len(), 2);
    }

    #[tokio::test]
    async fn test_cacheblend_prefix_matching() {
        let query = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let entry = make_entry(rolling_hash(&query[..8]), 8);

        let mut matcher = PrefixMatcher::new(8);
        matcher.index_entry(&entry);

        let (hash, match_len) = matcher.find_longest_prefix(&query).unwrap();
        assert_eq!(hash, entry.prefix_hash);
        assert_eq!(match_len, 8);
    }

    #[test]
    fn test_compression_ratio() {
        let raw = generate_random_kv(32, 8, 4096, 128);
        let sig = ModelSignature { /* ... */ };
        let compressed = cachegen_compress(&raw, &sig);
        let ratio = raw.len() as f64 / compressed.len() as f64;
        assert!(ratio > 3.0, "CacheGen ratio {:.2} < 3.0", ratio);
    }

    #[test]
    fn test_compression_quality() {
        let raw = generate_random_kv(32, 8, 4096, 128);
        let sig = ModelSignature { /* ... */ };
        let compressed = cachegen_compress(&raw, &sig);
        let decompressed = cachegen_decompress(&compressed, &sig);
        let cos_sim = cosine_similarity(&raw, &decompressed);
        assert!(cos_sim > 0.99, "cosine similarity = {}", cos_sim);
    }
}
```

---

## 12. Implementation Plan

| Phase | Description | Files | Effort |
|-------|-------------|-------|--------|
| 1 | Core types + LRU per tier + eviction pipeline | types.rs, config.rs, engine.rs | 2 days |
| 2 | CacheBlend prefix matching + merge algorithm | cacheblend.rs, serialization.rs | 3 days |
| 3 | CacheGen compression + decompression | compression.rs, sparsify.rs | 3 days |
| 4 | SSD backend (file store + eviction) | ssd_store.rs | 2 days |
| 5 | GatewayV2 integration + cache injection | gateway.rs (modify) | 2 days |
| 6 | KB RAG prefix sharing integration | search.rs (modify) | 1 day |

**Total: ~13 days**

Checkpoint at Phase 2: end-to-end CacheBlend prefix matching working with GPU-only cache.
Checkpoint at Phase 5: full tiered cache integrated into GatewayV2 with `--kvcache` CLI flag.

---

## 13. References

1. [LMCache: An Efficient KV Cache Layer for Enterprise-Scale LLM Inference](https://arxiv.org/abs/2505.19164) — primary architecture reference
2. [CacheGen: KV Cache Compression and Streaming](https://arxiv.org/abs/2310.07240) — compression algorithm
3. [CacheBlend: Fast LLM Serving with Cached Prefix](https://arxiv.org/abs/2405.16444) — prefix sharing
4. [H2O: Heavy-Hitter Oracle for KV Cache Eviction](https://arxiv.org/abs/2306.14048) — sparsification
5. [Tutti: Making SSD-Backed KV Cache Practical](https://arxiv.org/abs/2605.03375) — GPU-direct NVMe I/O
6. [KIVI: 2-bit KV Cache Quantization](https://arxiv.org/abs/2402.02750) — alternative compression
7. [KVQuant: KV Cache Quantization with Per-Channel + Per-Token](https://arxiv.org/abs/2401.12068) — quantization reference
