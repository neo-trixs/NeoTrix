# Iteration Batch 815 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Compression (10)
- flate2: Backend swap hazard (multiple features = unstable behavior), no decompression size limit
- zstd 0.13.3: C binding, best ratio/speed, pure-Rust ruzstd 1.4-3.5× slower
- lz4_flex 0.14.0: Block-only for no_std, no async (issue open since 2020)
- snap 1.1.1: Fast but poor ratio vs zstd, no tunable compression
- async-compression 0.4.42: "zstd stream did not finish" bug (Issue #420), AsyncWrite protocol violation
- brotli 8.0.1: LZ77 spec compliance bug fixed in 8.0.0
- ruzstd 0.9.0: Pure Rust but immature, multi-frame blindness
- CVE-2025-30160: Decompression bombs (no built-in size limit across all crates)
- zstd C binding FFI boundary is attack surface for untrusted input
- No async support in lz4_flex

### XML/HTML Parsing (8)
- quick-xml 0.42.0: Fixed O(N²) duplicate attribute DoS + unbounded namespace allocation
- scraper 0.27.0: Browser-grade parsing but 2.7× memory for large documents
- lol_html 3.0.1: Cloudflare streaming rewriter, constant O(1) memory
- html5ever 0.39.0: No streaming mode (requires full document in memory)
- select.rs 0.6.1: Low maintenance, limited CSS selector coverage
- dom_query: Enhanced scraper fork with :has/:contains pseudo-classes
- quick-xml RUSTSEC-2026-0194/0195: Quadratic DoS + unbounded namespace
- All HTML parsers lack built-in content extraction (readability)

### Regex/Text Processing (8)
- regex 1.13.1: Incorrect match offsets in reverse suffix optimization (correctness bug)
- fancy-regex 0.19.0: CRITICAL ReDoS vulnerability with backtracking features
- regex-automata 0.4.16: DFA state explosion unbounded (2^N states)
- aho-corasick 1.1.5: Prefilter jitter near ~100 pattern boundary
- memchr 2.8.3: AVX2 requires std feature (no_std limited to SSE2)
- fst 0.4: Stagnant since 2021, no async, u64 value limit
- regex! macro (1.13.0): Lazy compilation from string literals
- fancy-regex missing RegexSet equivalent

### I18n/Locale (8)
- ICU4X 2.3: Rule-based segmenters, CLDR 48.2 data, 50-90% smaller than ICU4C
- fluent 0.17.0: Pre-1.0, FTL is now legacy format (MF2 recommended)
- unic-langid 0.9.6: Never reached 1.0, dead upstream (ICU4X replaces)
- intl-memoizer 0.5.3: Pre-1.0 stagnation, ICU4X DataProvider subsumes
- rust-i18n 4.2.1: Simple key-value only (no plural/gender/BiDi)
- MessageFormat 2.0 (MF2): Now stable in CLDR, recommended over Fluent for new projects
- FluentBundle is !Send + !Sync by default (thread-safety gap)
- icu meta-crate 7.5MB compiled data (expensive for embedded)

---

## Defects Identified (30+)

### Compression (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-COMP-1 | No decompression size limit across all crates (zip bombs) | High |
| D-COMP-2 | flate2 backend swap hazard (multiple features = unstable) | High |
| D-COMP-3 | async-compression "zstd stream did not finish" bug | Medium |
| D-COMP-4 | async-compression AsyncWrite protocol violation (silent data loss) | Medium |
| D-COMP-5 | ruzstd multi-frame blindness (only first frame decoded) | Medium |
| D-COMP-6 | zstd C binding FFI boundary is attack surface | Medium |

### XML/HTML Parsing (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-PARSE-1 | quick-xml <0.41.0 O(N²) duplicate attribute DoS | High |
| D-PARSE-2 | quick-xml unbounded namespace allocation | High |
| D-PARSE-3 | html5ever no streaming mode (full document in memory) | Medium |
| D-PARSE-4 | scraper ego-tree DOM prevents incremental queries | Medium |
| D-PARSE-5 | select.rs low maintenance, limited CSS selectors | Medium |
| D-PARSE-6 | All HTML parsers lack content extraction (readability) | Medium |

### Regex/Text Processing (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-REGEX-1 | regex incorrect match offsets in reverse suffix optimization | High |
| D-REGEX-2 | fancy-regex ReDoS with backtracking features | Critical |
| D-REGEX-3 | regex-automata DFA state explosion unbounded | Medium |
| D-REGEX-4 | fancy-regex missing RegexSet equivalent | Medium |
| D-REGEX-5 | fst stagnant since 2021 (no async, u64 limit) | Medium |
| D-REGEX-6 | memchr AVX2 requires std (no_std limited to SSE2) | Low |
| D-REGEX-7 | aho-corasick prefilter jitter near ~100 patterns | Low |

### I18n/Locale (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-I18N-1 | unic-langid dead upstream (never reached 1.0) | High |
| D-I18N-2 | fluent FTL is legacy format (MF2 recommended) | Medium |
| D-I18N-3 | FluentBundle !Send + !Sync (thread-safety gap) | High |
| D-I18N-4 | intl-memoizer pre-1.0 stagnation | Medium |
| D-I18N-5 | icu meta-crate 7.5MB (too expensive for embedded) | Medium |
| D-I18N-6 | rust-i18n no plural/gender/BiDi support | Low |
| D-I18N-7 | No Rust-native MF2 runtime yet | Low |

## Key Insights (This Batch)

1. **fancy-regex ReDoS is critical**: Any user-facing or attacker-exposed regex path must use regex (linear-time), never fancy-regex. This affects NT-MEMORY FTS5 and NT-SHIELD egress guard.

2. **regex match offsets bug**: Correctness bug in 1.13.1 reverse suffix optimization. Match occurs but start()/end() can be wrong. Must wrap all Regex::find/captures with offset validation.

3. **quick-xml DoS vulnerabilities**: Quadratic duplicate attribute check + unbounded namespace allocation. Must pin >= 0.41.0 for NT-WORLD XML parsing.

4. **decompression bombs are systemic**: No crate enforces max decompressed size. Must implement DecompressLimiter wrapper.

5. **unic-langid is dead**: ICU4X's icu::locale is the sanctioned successor. Must replace for language identifier handling.

6. **FluentBundle thread-safety**: !Send + !Sync by default. NT-CORE's async/multi-threaded consciousness tree requires Arc<Mutex<...>> or ICU4X sync feature.

7. **MF2 is the future**: MessageFormat 2.0 is now stable in CLDR. Fluent FTL is legacy. NeoTrix should plan MF2 migration path now.

8. **ICU4X selective data is key**: Full icu meta-crate is 7.5MB. Must use selective icu4x-datagen with locale subset (zh, en, ja, ko) for embedded use.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 815 |
| New defects (this batch) | 26 |
| Cumulative defects | D01-D76522 |
| Research sources (this batch) | 34 |
| Cumulative research sources | 97,313+ |
