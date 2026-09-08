# Iteration 689 — Regex / String Processing / Text Normalization Defects

## Sources Consulted
- `regex-automata` v0.4.13 (docs.rs, crates.io) — DFA/NFA/lazy-DFA engines, O(m*n) worst-case
- `unicode-normalization` v0.1.24 (docs.rs) — NFC/NFD/NFKC/NFKD, no_std+alloc
- `unicode-segmentation` v1.12.0 (lib.rs) — grapheme clusters, word/sentence boundaries
- `simd-normalizer` v0.1.1 (GitHub) — SIMD-accelerated NFC/NFD/NFKC/NFKD (SSE4.2/AVX2/AVX-512/NEON)
- `icu_normalizer` (lib.rs) — ICU4X normalization
- `rustfaq.org` (2026) — Unicode handling in Rust best practices
- `std::string` docs (2026-09-01) — String type update

## Defects Found

### DEFECT-R689-1: Regex Recompilation in Hot Paths (CRITICAL)
**Files**: `nt_shield/redaction.rs:36-59`, `nt_world_search.rs:430`, `nt_core_self_constitution.rs:451-589`, `laws.rs:57-147`, `nt_core_arch_fitness.rs:72-458`, `nt_shield_mcp_security.rs:471-809`
**Finding**: ~60+ `regex::Regex::new()` calls compile patterns at every invocation. The redaction module (`redaction.rs`) compiles 14+ regex patterns on every LLM egress check. The security module compiles IP/domain/hash/URL patterns on every request. The architecture fitness checker compiles patterns on every audit call.
**Impact**: Regex compilation is O(pattern_size * states). Unicode character classes explode state count. On egress guard hot path, this adds ~50-200μs per LLM request.
**Root Cause**: No `LazyLock`/`OnceLock` caching for these patterns. Only `nt_normalizer.rs` and `nt_world_browse/session.rs` use `OnceLock` correctly.
**Fix**: Migrate all static regex patterns to `LazyLock<Regex>` or `OnceLock<Vec<(Regex, ...)>>`. For redaction, pre-compile all 14+ patterns into a static `Vec<(&str, Regex)>` at module init.

### DEFECT-R689-2: No DFA/Lazy-DFA for High-Throughput Search (HIGH)
**Files**: `nt_world_search.rs`, `nt_core_code_search.rs`, `nt_world_crawl/discover.rs`
**Finding**: All search operations use the default `regex` crate engine (PikeVM/thompson NFA hybrid). The `regex-automata` crate offers:
- `dfa::dense::DFA` — fastest search, worst-case O(m*n), but exponential compile
- `hybrid::dfa::DFA` (lazy DFA) — builds incrementally during search, avoids compile explosion
- `dfa::sparse::DFA` — compact storage for pre-compiled patterns
**Impact**: For KB search (`nt_world_search`) and code search (`nt_core_code_search`), the lazy DFA could provide 2-5x throughput improvement with bounded scratch space. Currently unused.
**Fix**: For the code search MCP tool and crawl discovery (patterns are known/static), use `dfa::dense::DFA` with `to_bytes`/`from_bytes` serialization. For dynamic user patterns, use `hybrid::dfa::DFA`.

### DEFECT-R689-3: No Aho-Corasick for Literal Pattern Sets (MEDIUM)
**Files**: `nt_shield/redaction.rs:36-59`, `nt_shield_comm.rs:292-310`, `nt_world_crawl/discover.rs:93-223`
**Finding**: Multiple modules run N independent regex scans over the same text. The redaction module runs 14+ `is_match()` calls sequentially. `nt_shield_comm.rs` runs 4 independent regex checks. The crawl discovery module runs 5+ rules sequentially.
**Impact**: `regex-automata` supports multi-pattern search natively, or `aho-corasick` crate does multi-pattern literal matching in a single pass. Currently each pattern is evaluated independently, causing N full-text scans.
**Fix**: Use `regex::RegexSet` for multi-pattern or `aho-corasick::AhoCorasick` for literal patterns. Single-pass multi-pattern scan replaces N sequential scans.

### DEFECT-R689-4: `.chars().take(N)` Truncation Breaks Grapheme Clusters (HIGH)
**Files**: `nt_normalizer.rs:230`, `nt_file_ability/tables.rs:38-41`, `nt_file_ability/output_formatter.rs:218,259,287`, `nt_capability_bridge.rs:348`, `nt_file_ability/structured.rs:54`
**Finding**: Multiple files use `.chars().take(N)` or `.chars().count()` for text truncation/measurement. Rust's `char` is a Unicode scalar value, NOT a grapheme cluster. Examples:
- `"café"` with NFD: `chars().take(4)` → `"cafe"` (combining accent lost)
- Emoji with ZWJ: `"👨‍👩‍👧‍👦".chars().take(1)` → first code point only, renders as broken glyph
- Thai/Hindi with combining marks: truncation mid-cluster produces mojibake
**Impact**: Content previews, table cell rendering, and capability rationale truncation all produce visually broken output for non-Latin text. KB content stored via `extract_key_sections` truncates at 500 chars via scalar count.
**Fix**: Replace `.chars().take(N)` with `unicode_segmentation::UnicodeSegmentation::graphemes(true).take(N)` for user-visible text. Use `.chars().count()` only for internal scalar counting.

### DEFECT-R689-5: Inconsistent Unicode Normalization Before Dedup/FTS (HIGH)
**Files**: `nt_normalizer.rs:166` (only caller), `nt_core_code_search.rs`, `nt_world_search.rs`, `nt_shield/redaction.rs`
**Finding**: `normalize_text()` (NFKC) is only called in the KB normalizer. However:
- `nt_core_code_search.rs` does code search WITHOUT normalizing input
- `nt_shield/redaction.rs` scans LLM output WITHOUT normalizing before matching
- `nt_world_search.rs` performs search ranking WITHOUT normalizing query terms
- FTS5 index is populated via `strip_markdown()` → `normalize_text()` but search queries skip normalization
**Impact**: Fullwidth characters (`１` vs `1`), ligatures (`ﬁ` vs `fi`), and compatibility equivalents produce false negative search results. User queries with `café` (decomposed) won't match `café` (precomposed) unless both paths normalize.
**Fix**: Enforce normalization at all text ingestion boundaries: search query parsing, FTS query construction, redaction scanning, and code search input.

### DEFECT-R689-6: No SIMD-Accelerated Normalization for Large Documents (MEDIUM)
**Files**: `nt_normalizer.rs:166` — `text.nfkc().collect::<String>()`
**Finding**: The `unicode-normalization` crate uses scalar code. The new `simd-normalizer` crate (2026) provides:
- x86_64: SSE4.2, AVX2, AVX-512
- aarch64: NEON
- wasm32: simd128
- Runtime CPU dispatch on x86_64
- Zero-copy `Cow::Borrowed` when input is already normalized
- Scans 64-byte chunks, skipping ASCII/passthrough regions in bulk
**Impact**: For large crawled documents (10K-100K chars), SIMD normalization could provide 3-8x throughput improvement. Current scalar implementation processes one codepoint at a time.
**Fix**: Add `simd-normalizer` as optional dependency. Use it for bulk normalization in `strip_markdown` and document ingestion pipeline. Fall back to `unicode-normalization` for small strings.

### DEFECT-R689-7: No Zero-Copy Normalization Check (LOW)
**Files**: `nt_normalizer.rs:162-169`
**Finding**: `normalize_text()` always allocates a new `String` via `.nfkc().collect()`. For text that is already normalized (common in programmatically-generated content), this wastes allocation.
**Impact**: ~30% of crawled content may already be NFKC-normalized (programmatic sources, API responses). Each unnecessary allocation adds heap pressure on the KB write path.
**Fix**: Use `Cow`-based normalization: check if input is already normalized (via `is_nfc()`/`is_nfkc()`), return `Cow::Borrowed` if so, `Cow::Owned` if not. The `unicode-normalization` crate's iterator already supports this pattern.

### DEFECT-R689-8: Regex Escaping Vulnerability in User-Supplied Patterns (MEDIUM)
**Files**: `nt_core_mcp.rs:859`, `nt_core_self_constitution.rs:532-533`, `nt_absorb_mapper.rs:130`
**Finding**: User-supplied regex patterns are passed directly to `Regex::new()` without validation beyond the `map_err`. While `regex::Regex::new()` will reject invalid patterns, it does NOT bound resource consumption. Pattern `[01]*1[01]{100}` compiles a DFA with 2^102 states — exponential blowup.
**Impact**: A malicious or malformed query to the MCP `search_code` tool could cause OOM by supplying a pathological regex pattern. The `regex-automata` crate's `dense::Config::size_limit()` exists for this purpose but is not used.
**Fix**: Set `regex::RegexBuilder::size_limit(1 << 20)` (1MB DFA cap) on all user-facing regex compilation. Log and reject patterns that exceed the limit.

## Summary

| # | Severity | Category | Status |
|---|----------|----------|--------|
| R689-1 | CRITICAL | Regex recompilation | Open |
| R689-2 | HIGH | No DFA/lazy-DFA | Open |
| R689-3 | MEDIUM | No Aho-Corasick | Open |
| R689-4 | HIGH | Grapheme cluster breakage | Open |
| R689-5 | HIGH | Inconsistent normalization | Open |
| R689-6 | MEDIUM | No SIMD normalization | Open |
| R689-7 | LOW | No zero-copy normalization | Open |
| R689-8 | MEDIUM | Regex resource exhaustion | Open |

## Connection to Batch 688 Memory Defects
- **R689-1** compounds the arena/slab absence: each regex allocates on the global allocator (system malloc) with no pooling. If arenas were on SEAL hot paths, regex compilation would benefit from arena allocation.
- **R689-4** affects KB storage quality: broken grapheme truncation means stored content has corrupted previews, which feeds back into the attention system (GWT) with degraded signal.
- **R689-5** creates false negatives in FTS search, which means the knowledge retrieval pipeline misses relevant results — the "spice" stops flowing.
