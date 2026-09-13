# Agent-Reach → NeoTrix Fusion Plan

**Source**: Panniantong/Agent-Reach (Python)  
**Target**: NeoTrix (Rust)  
**Compliance**: R-P42 — strengthen existing nodes, no new parallel modules  
**Date**: 2026-09-13

---

## 1. Channel + Ordered Backend → NT-WORLD Perception Routing

### Agent-Reach Pattern
`Channel` ABC with ordered `backends: List[str]`, `can_handle(url)`, `check(config)` → sets `active_backend`. `ordered_backends()` applies user override (env `<CHANNEL>_BACKEND`) to reorder candidates.

### Existing NeoTrix Node
`OrderedBackendRouter` in `l2_perception/nt_world/crawl/ordered_backend_router/mod.rs:69` — ordered backends with health cache, priority-based selection, `BackendType` enum (BrowserAct/Camoufox/Playwright/HttpOnly). `SearchBackend` trait in `nt_world_search.rs:282` — `name()`, `search()`.

### Fusion: Strengthen Existing Nodes
1. **Extend `BackendType` enum** — add platform-categorical variants (`YouTube`, `Twitter`, `GitHub`, `Reddit`, `RSS`, etc.) alongside existing browser backends. The enum becomes the single registry for ALL backend types.
2. **Add `Channel`-like trait to `OrderedBackendRouter`** — new trait `PlatformProbe` with `can_handle(url: &str) -> bool`, `tier: u8`, `description: &str`. Implement per-platform. This goes in `ordered_backend_router/mod.rs` as a sub-trait, NOT a new module.
3. **User override via config** — Agent-Reach's `<channel>_backend` env override maps to existing `BackendConfig.config` HashMap. Add `ordered_backends()` method that reorders by config/env override (same semantics as Agent-Reach).
4. **URL-routing table** — add `fn route_url(url: &str) -> Option<&BackendType>` to `OrderedBackendRouter` that tries `can_handle()` on each registered platform in priority order.

### Code Locations
| What | Where |
|------|-------|
| Extend `BackendType` | `ordered_backend_router/mod.rs:13` |
| Add `PlatformProbe` trait | `ordered_backend_router/mod.rs` (new trait) |
| URL routing | `ordered_backend_router/mod.rs` (new method) |
| Config override | `BackendConfig.config` HashMap (existing) |

---

## 2. Doctor Command → NT-REPAIR Health Check

### Agent-Reach Pattern
`doctor.py` — iterates ALL channels, calls `ch.check(config)` per-channel, catches exceptions (degrades to `status="error"`), scrubs URLs from output, formats tiered report (Tier 0/1/2). Config file permission check at end.

### Existing NeoTrix Node
`HeartbeatAggregator` in `core/nt_core_heartbeat.rs:143` — aggregates `ComponentHealth` per-subsystem, generates `SystemHealthSnapshot` for GWT. `ConsciousnessTree` SelfTest chain (D1-D51 dimensions).

### Fusion: Strengthen Existing Nodes
1. **Channel health → ComponentHealth** — map each platform probe result to `ComponentHealth { name: "nt_world_{platform}", status: HealthStatus::{Healthy|Degraded|Unhealthy}, message, last_updated }`. Feed into `HeartbeatAggregator` as a new source.
2. **Tiered reporting** — add `DoctorReport` struct to `nt_core_heartbeat.rs` with tier-grouped output (zero-config / needs-key / complex-setup). This reuses Agent-Reach's tier semantics but renders via NeoTrix's existing health infrastructure.
3. **Exception isolation** — Agent-Reach's per-channel try/catch pattern → Rust's `catch_unwind` or just `Result` propagation per-platform probe. Single platform failure never corrupts the aggregate report.
4. **Config permission check** — port `doctor.py`'s `stat.S_IRGRP | stat.S_IROTH` check to Rust using `std::os::unix::fs::MetadataExt::mode()`. Add as a SelfTest dimension (T1) in `nt_core_heartbeat.rs`.

### Code Locations
| What | Where |
|------|-------|
| Channel health mapping | `nt_core_heartbeat.rs` (add source) |
| DoctorReport struct | `nt_core_heartbeat.rs` (new struct) |
| Permission check SelfTest | `nt_core_heartbeat.rs` (new fn) |

---

## 3. Probe System → NT-WORLD Real Probing

### Agent-Reach Pattern
`probe.py` — `probe_command()` executes `cmd --version` with timeout, retries, and 3-way failure classification: `missing` (not on PATH), `broken` (stale venv shim — `which()` passes but exec fails), `timeout/error`. `_BROKEN_EXIT_CODES = (126, 127)`. `reinstall_hint()` generates fix instructions.

### Existing NeoTrix Node
`OrderedBackendRouter::check_backend()` in `ordered_backend_router/mod.rs:236` — runs `sh -c {cmd}` with timeout, returns `(bool, capabilities, Option<String>)`. `nt_shield_sandbox` has a `ProbeResult` for network probes.

### Fusion: Strengthen Existing Nodes
1. **Replace boolean health with `ProbeResult` enum** — change `check_backend()` return from `(bool, BackendCapabilities, Option<String>)` to a structured `ProbeResult { status: ProbeStatus, output: String, hint: String }` where `ProbeStatus` = `Ok | Missing | Broken | Timeout | Error`. This is a strict superset of the existing boolean.
2. **Three-way failure in `perform_health_checks()`** — Agent-Reach's `FileNotFoundError` (stale shebang) / exit 126-127 / timeout classification maps directly to `ProbeStatus::Broken` / `ProbeStatus::Timeout`. Port the `_BROKEN_EXIT_CODES` check.
3. **Retry logic** — add `retries: u8` field to `BackendConfig`. Only retry on `Timeout`/`Error`, never on `Missing`/`Broken` (same as Agent-Reach).
4. **`reinstall_hint()`** — add `fn reinstall_hint(package: &str) -> String` to `ordered_backend_router/mod.rs`. Generates `uv tool install --force {package}` / `pipx reinstall {package}` fix instructions.

### Code Locations
| What | Where |
|------|-------|
| `ProbeResult` enum | `ordered_backend_router/mod.rs` (new enum) |
| Update `check_backend()` | `ordered_backend_router/mod.rs:236` |
| Retry in `perform_health_checks()` | `ordered_backend_router/mod.rs:207` |
| `reinstall_hint()` | `ordered_backend_router/mod.rs` (new fn) |

---

## 4. SKILL.md Format → nt_mind_skill_engine

### Agent-Reach Pattern
Agent-Reach doesn't define SKILL.md itself, but its `Channel` interface (name/description/backends/tier/can_handle/check) is a de facto skill template: declarative metadata + behavioral methods. Each channel is a self-contained capability descriptor.

### Existing NeoTrix Node
`SkillEntry` in `l5_cognition/nt_mind/nt_mind_skill_engine.rs:20` — `from_file()` parses SKILL.md frontmatter (name/category/tools/triggers/description). `SkillEngine` with `load_all()`, `find_matching()`, `skill_tree()`.

### Fusion: Strengthen Existing Nodes
1. **Platform channels as SkillEntry sources** — extend `SkillEntry::from_file()` or add `from_platform_probe()` constructor that builds a `SkillEntry` from a `PlatformProbe` implementation. Fields map: `name` → `name`, `description` → `description`, `backends` → `tools`, `tier` → `metadata.tier`.
2. **`can_handle` as trigger** — Agent-Reach's `can_handle(url)` maps to NeoTrix's trigger matching. Add `url_pattern: Option<String>` field to `SkillEntry` for URL-based skill routing.
3. **`check` as live validation** — Agent-Reach's `check()` → skill health check. Add `fn validate_live(&self) -> Option<HealthStatus>` to `SkillEntry` that calls the platform probe's check method. This gives skill engine runtime awareness of backend availability.
4. **Tier metadata** — Agent-Reach's tier (0=zero-config, 1=needs-key, 2=complex) → add `tier: u8` to `SkillEntry`'s metadata. Skill engine can filter/sort by tier for user-facing skill selection.

### Code Locations
| What | Where |
|------|-------|
| `from_platform_probe()` | `nt_mind_skill_engine.rs` (new constructor) |
| `url_pattern` field | `SkillEntry` struct |
| `validate_live()` | `SkillEntry` impl |
| `tier` metadata | `SkillEntry.metadata` |

---

## 5. Config Management → NT-CORE Config Pattern

### Agent-Reach Pattern
`config.py` — YAML config at `~/.agent-reach/config.yaml`. Atomic write (tempfile + `os.replace`), symlink rejection, permission lockdown (0600), `read_only` mode, sensitive key masking, env var fallback (`key.upper()`), `FEATURE_REQUIREMENTS` dict for feature→required-keys mapping, rollback on save failure.

### Existing NeoTrix Node
`ConfigManager` in `cli/tui/config.rs:171` — TUI-specific, minimal. No atomic writes, no security hardening.

### Fusion: Strengthen Existing Nodes
1. **Atomic write with symlink rejection** — port `_atomic_write_yaml()` to Rust: `tempfile::NamedTempFile` + `std::fs::rename` (atomic on same FS). Add symlink check via `std::fs::metadata` + `is_symlink()`. Store in `nt_core` or a shared `crates/` config module.
2. **Permission lockdown** — `std::os::unix::fs::OpenOptionsExt::mode(0o600)` on Unix. Skip on Windows. This is a direct port of Agent-Reach's `fchmod` pattern.
3. **Sensitive key masking** — `to_dict()` method with regex-based redaction for keys containing `key|token|password|secret|auth|cred|cookie|session`. Output `[REDACTED]` for values.
4. **Feature requirements** — `FEATURE_REQUIREMENTS: HashMap<&str, Vec<&str>>` → define per-domain required config keys. `is_configured(feature) -> bool` checks all required keys present.
5. **Read-only mode** — `read_only: bool` flag. `set()`/`delete()` raise error when read-only. Useful for production/locked configs.
6. **Env var fallback** — `get(key)` checks YAML first, then `std::env::var(key.to_uppercase())`. Standard pattern.

### Code Locations
| What | Where |
|------|-------|
| Atomic write | New fn in shared config crate or `nt_core_self` |
| Symlink rejection | Same location |
| Permission lockdown | Same location |
| Sensitive masking | Same location |
| Feature requirements | Same location |

---

## Summary: R-P42 Compliance Check

| Mapping | Target Node | New Module? | Strengthening |
|---------|------------|-------------|---------------|
| Channel + Ordered Backend | `OrderedBackendRouter` + `SearchBackend` | No | Extend `BackendType`, add `PlatformProbe` trait |
| Doctor → Health | `HeartbeatAggregator` + `SystemHealthSnapshot` | No | Add channel health source, `DoctorReport` |
| Probe → Real Probing | `OrderedBackendRouter::check_backend()` | No | Replace boolean with `ProbeResult` enum |
| SKILL.md → Skill Engine | `SkillEntry` + `SkillEngine` | No | Add `from_platform_probe()`, `validate_live()` |
| Config → Config Pattern | Shared config crate | No | Atomic write, security hardening |

All 5 mappings target existing nodes. Zero new modules created.
