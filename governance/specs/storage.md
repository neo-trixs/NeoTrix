# Storage Subsystem
> Version 1.0 — Status: ✅ active

## Purpose
Provides durable, append-only binary segment storage (NTSSEG format) for VSA vectors, knowledge entries, and consciousness state. Zero external dependencies beyond serde + std. Includes write-through caching and credit-based soft rate limiting.

## Exposed Operations

### Tools
- `put(collection: String, key: String, data: &[u8]) -> Result<(), StoreError>` — Store a record
- `get(collection: String, key: String) -> Result<Option<Vec<u8>>, StoreError>` — Retrieve a record (cache-fast)
- `delete(collection: String, key: String) -> Result<(), StoreError>` — Tombstone a record
- `put_vsa(vector: &QuantizedVSA, metadata: &[u8]) -> Result<(), StoreError>` — Store VSA vector + IVF index
- `search_vsa(query: &QuantizedVSA, top_k: usize) -> Vec<SearchResult>` — IVF-accelerated similarity search
- `compact(collection: String) -> Result<CompactionReport, StoreError>` — Merge segments, remove tombstones

### Resources
- `storage://stats` — Segment count, record count, disk usage, credit utilization
- `storage://collection/{name}` — Per-collection metadata and size
- `storage://index/vsa` — IVF centroid count and partition sizes

## Configuration Schema

```rust
pub struct StorageConfig {
    pub base_path: PathBuf,                           // default ~/.neotrix/store
    pub max_segment_size: u64,                        // bytes, default 64MB
    pub auto_compact_threshold: f64,                  // tombstone ratio, default 0.3
    pub credit_budget: Option<CreditBudget>,          // optional soft limit
}

pub struct CreditBudget {
    pub max_writes: u64,       // default 1_000_000
    pub max_vsa_vectors: u64,  // default 100_000
    pub max_disk_bytes: u64,   // default 1_073_741_824 (1GB)
}
```

## Dependencies
- **VSA Core** (nt_core_hcube) — VsaTag types, QuantizedVSA
- No external crates (serde only)

## Error States

| State | Trigger | Recovery |
|-------|---------|----------|
| `CreditExhausted` | Write exceeds budget | Return `io::ErrorKind::ResourceBusy`, set degrade flag |
| `SegmentCorrupt` | Magic/checksum mismatch | Skip segment, rebuild index from intact segments |
| `CompactionLocked` | Concurrent compaction in progress | Retry after backoff (100ms, 500ms, 2s) |
| `CacheStale` | Write-through cache vs segment mismatch | Flush cache, re-read from segment |
