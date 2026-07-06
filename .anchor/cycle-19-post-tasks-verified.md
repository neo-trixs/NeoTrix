# Session Anchor — Cycle 19 Post-Tasks: Wikipedia Fallback + Proxy Env Var + Stability Test + Rust Strip

## Completed Tasks

### 1. Wikipedia 503 Fallback — MediaWiki API ✅
- Added `_fetch_mediawiki()` fallback strategy to `WikipediaTransport`
- Strategy chain: REST API → MediaWiki API → None (3-tier)
- MediaWiki uses `action=query&prop=extracts|info&exintro&explaintext&format=json`
- Both strategies return normalized `{title, extract, pageid, url}` (MediaWiki) or raw REST response
- Verified: REST `Artificial_intelligence` ✅ (492 chars), MediaWiki `Python_...` ✅ (1436 chars)

### 2. NEOTRIX_PROXY_URL Env Var ✅
- Added `NEOTRIX_PROXY_URL` env var support to `ProxyPool`
- Pre-populated on init with `valid=True` flag and `_env=True` marker
- `refresh()` preserves env proxy at index 0 (always first/priority)
- `get()` returns env proxy first when available (deterministic priority)
- Verified: `pool_size=1`, `get()` returns env URL

### 3. Multi-Endpoint Stability Test ✅
- 5 endpoints × 2 rounds = 10/10 ✅
  - httpbin.org/headers, example.com, google.com, github.com, wikipedia.org
- Persona rotation observed: chrome_win, chrome_mac, firefox_win, edge_win
- Zero NeoTrix leaks (checked body for neotrix/nt_ patterns)

### 4. Rust `strip_internal_patterns` in http_proxy.rs ✅
- Added `strip_internal_patterns()` function (5 regex patterns match Python INTERNAL_PATTERNS)
- Added `strip_header_values()` for header name+value cleaning
- Wired into `handle_client()`: strips all headers + URL before forwarding
- No cargo errors from http_proxy.rs — only pre-existing nt_mind legacy errors

## Build Status
- ✅ All 3 Python files py_compile pass
- ✅ CommRouter+DirectTransport: httpbin.org 200, zero leak
- ✅ Cargo: 0 new errors (http_proxy.rs compiles clean)
- ✅ Anchor preserved

## Files Changed
- `scripts/nt_api_client.py` — WikipediaTransport dual-strategy; ProxyPool env var + valid flag
- `nt_shield/http_proxy.rs` — strip_internal_patterns + strip_header_values + wired into handle_client
