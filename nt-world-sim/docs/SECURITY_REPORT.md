# Security Report — NeoTrix Project

**Generated**: 2026-09-14
**Scope**: Full codebase security scan

---

## Executive Summary

The codebase has several security concerns: SQL injection risks, extensive `unwrap()` usage (901 instances), and 147 files with `unsafe` code. The NT-SHIELD security domain exists but doesn't cover all attack surfaces.

| Metric | Value | Status |
|--------|-------|--------|
| Files with `unsafe` | 147 | 🔴 Critical |
| `unwrap()` in prod | ~901 | 🔴 High Risk |
| SQL injection risks | 3+ | 🔴 Critical |
| Hardcoded secrets | Not found | ✅ Good |
| XSS vulnerabilities | Low (CLI app) | ✅ Low Risk |
| Path traversal risks | Moderate | ⚠️ Review Needed |

---

## 1. SQL Injection Risks

Found SQL queries using string formatting (potential injection):

| File | Line | Pattern | Risk |
|------|------|---------|------|
| `l1_action/nt_memory/nt_memory_kb/nt_memory_embed.rs` | 574 | `format!("SELECT node_id, vector FROM embeddings LIMIT {l}")` | 🔴 High |
| `l1_action/nt_memory/nt_memory_kb/nt_memory_crawl.rs` | 1517 | `format!("SELECT node_id FROM geo_index WHERE source='{}'", source)` | 🔴 High |
| `l1_action/nt_memory/nt_memory_kb/nt_memory_unify.rs` | 1184 | `format!("SELECT COUNT(*) FROM {}", table)` | 🔴 High |

**Fix**: Use parameterized queries (`rusqlite::params![]`) instead of string formatting.

---

## 2. Unsafe Code (R-P1 Violation)

**147 files** contain `unsafe` blocks. Top security-critical files:

| File | unsafe Count | Risk |
|------|-------------|------|
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 69 | 🔴 High |
| `core/nt_core_meta/scanner.rs` | 26 | 🔴 High |
| `core/nt_core_meta/weakness.rs` | 17 | 🔴 High |
| `l1_action/nt_act/nt_act_code/code_writer.rs` | 11 | 🔴 High |
| `l1_action/nt_act/nt_act_goal/goal_generator.rs` | 10 | 🔴 High |

**Severity**: 🔴 CRITICAL — Unsafe code bypasses Rust's memory safety guarantees.

---

## 3. Panic Risk (unwrap())

**901 `unwrap()` calls** found in production code (excluding tests/examples). These can cause:
- Service crashes
- Denial of service
- Data corruption

**Top offenders** (non-test code):
- `core/nt_core_rule_memory.rs`: Template processing with unwrap
- Various memory/embedding modules: DB operations with unwrap

**Fix**: Replace with `.unwrap_or_else()`, `.ok_or()`, or `?` operator.

---

## 4. Path Traversal Risks

| Pattern | Found | Risk |
|---------|-------|------|
| File operations without path validation | Some | ⚠️ Medium |
| User-provided paths in file I/O | Present | ⚠️ Medium |

**Recommendation**: Validate all file paths against a whitelist of allowed directories.

---

## 5. Secret Exposure

| Check | Result |
|-------|--------|
| Hardcoded API keys | ✅ None found |
| Hardcoded passwords | ✅ None found |
| Hardcoded tokens | ✅ None found |
| `.env.example` present | ✅ Yes |
| `.gitleaks.toml` present | ✅ Yes |

**Assessment**: ✅ Good — Secret management appears properly configured.

---

## 6. Egress Privacy Guard

The NT-SHIELD domain includes an Egress Privacy Guard that filters outbound LLM requests. Assessment:

| Feature | Status |
|---------|--------|
| Internal fingerprint blocking | ✅ Implemented |
| Trust tiers (Trusted/Contracted/Untrusted) | ✅ Implemented |
| Secret scrubbing | ✅ Implemented |
| Absolute path redaction | ✅ Implemented |

---

## 7. Recommendations

1. **CRITICAL**: Fix SQL injection vulnerabilities (3 instances) — use parameterized queries
2. **CRITICAL**: Audit and remove `unsafe` code (147 files, R-P1 violation)
3. **HIGH**: Replace `unwrap()` in production code with proper error handling
4. **MEDIUM**: Add path validation for all file operations
5. **LOW**: Run `cargo audit` to check dependency vulnerabilities
