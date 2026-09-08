# Iteration Batch 691 — Configuration, Feature Flags & Secret Management Audit

**Date**: 2026-09-06
**Focus**: Configuration management, compile-time config, environment variables, secrets lifecycle, AI-agent secret leakage
**Prior batch**: 690 (crypto primitives: SHA-256/BLAKE3, ML-KEM key reuse, Ed25519 deprecation, trait abstraction)

---

## 1. Defect: No Configuration Baseline or Drift Detection

**Severity**: HIGH | **Category**: Configuration Management

2026 CM best practices (Cloudaware, InfoQ, TheLinuxCode) universally mandate:
- A **configuration baseline** (approved, immutable state of all CIs at a point in time)
- **Continuous drift detection** (automated comparison of actual vs. desired state)
- **Baseline promotion gates** (build/test/security/operational integrity before promotion)

NeoTrix has **none of these**. Configuration is implicit in Cargo feature flags and ad-hoc `#[cfg]` attributes. There is:
- No declared baseline manifest per release
- No drift detection against declared config state
- No gate process for promoting config changes between dev/staging/production
- No audit trail linking config changes to commits, tickets, or approvals

**Impact**: Configuration drift compounds silently. A feature flag toggled for debugging leaks into production. A crypto backend silently downgraded. No way to answer "what was the approved config state at time T?"

**Sources**: Cloudaware 2026 CM guide; InfoQ "Configuration as a Control Plane" (2026-03); TheLinuxCode SCM guide (2026-02)

---

## 2. Defect: No Single Source of Truth for Configuration Items

**Severity**: HIGH | **Category**: CM Architecture

2026 practice mandates **one source of truth per attribute** — two tools fighting over the same field causes data drift (Cloudaware CMDB best practices, 2026-06).

NeoTrix fragments configuration across:
- `Cargo.toml` `[features]` table (compile-time flags)
- `build.rs` scripts (dynamic cfg generation)
- `#[cfg(...)]` attributes scattered across source files
- `.env` files (runtime config, if used)
- Hardcoded `const` values in source

There is no centralized configuration registry. The same setting (e.g., "which crypto backend") could be defined in 3+ places with no reconciliation.

**Impact**: Conflicting definitions. Feature `crypto-blake3` might be enabled in `Cargo.toml` but a `build.rs` script conditionally disables it based on target arch. No single authoritative source.

**Source**: Cloudaware CMDB best practices §2 (2026-06-26)

---

## 3. Defect: Feature Flags as Configuration Items Without Lifecycle Management

**Severity**: MEDIUM | **Category**: Feature Configuration

2026 best practice: treat feature flags as CIs with **owner, expiry date, and cleanup policy**. Stale flags must be audited every sprint (TheLinuxCode, 2026-02). Flags without lifecycle management create "the change that wasn't logged" — the most dangerous config drift.

NeoTrix has no:
- Expiry dates on feature flags
- Ownership assignment per flag
- Automated stale-flag detection
- Flag-to-requirement traceability

**Impact**: Dead feature flags accumulate. A `cfg(feature = "old-crypto")` remains in code long after the old crypto is removed, creating dead code paths and testing combinatorial explosion.

**Source**: TheLinuxCode SCM guide §"Common Failure Patterns" (2026-02)

---

## 4. CRITICAL Defect: `.env` Files Are an Active Secret Leakage Vector to AI Agents

**Severity**: CRITICAL | **Category**: Secret Management / AI Agent Security

Multiple 2026 sources confirm a **new attack surface**: AI coding agents (Claude Code, Cursor, Copilot) read `.env` files from the filesystem and inject their contents into LLM context windows. Secrets then propagate to provider logs, gateway logs, and chat history.

Key findings:
- **SonarSource (2026-08-17)**: AI agents "read everything" — source files, configs, env vars. Secrets travel to model providers and become scattered across systems you don't own.
- **CVE-2026-59261**: OpenClaw framework lets workspace `.env` files silently override trusted provider credentials (CVSS 7.1). Attack vector: drop a `.env` in a shared repo, agent picks up attacker-controlled credentials.
- **Keyway (2026-04)**: `.env` files leak through logs, crash dumps, Docker layers, `ps` output, and now AI agents. The fix is **zero-disk injection** — never write secrets to disk at all.
- **Bitwarden (2026-04)**: Agents run `cat .env`, `printenv`, `grep -r "API_KEY"`, `Read ~/.aws/credentials` without being prompted. The agent is "not malicious, it's autocompleting."
- **Abrahamberg (2026-06)**: "The new advice is: do not put real secrets where your coding agent can casually read them."

**NeoTrix Impact**: If NeoTrix uses `.env` files for configuration (or any workspace-level secret files), the entire agent context pipeline leaks secrets to LLM providers. This is **architecturally incompatible** with NeoTrix's Egress Privacy Guard — which redacts outbound data but cannot retroactively scrub secrets already embedded in prompt context.

**Sources**:
- SonarSource blog (2026-08-17): "Your secrets are leaking to AI coding agents"
- CVE-2026-59261 (2026-07-08): OpenClaw dotenv override leak
- Keyway (2026-04-06): "Are .env Files Still Safe for Secrets in 2026?"
- Bitwarden (2026-04-02): "Your coding agent can read your .env file"
- Abrahamberg (2026-06-01): "Your Coding Agent Can Read Your .env"
- AgenticControlPlane (2026-04-26): "Stop your AI agent from leaking secrets"
- Sonar (2026-08-26): "Secrets Management in the Age of AI"

---

## 5. Defect: No Ephemeral Credential / Workload Identity Architecture

**Severity**: HIGH | **Category**: Secret Lifecycle

2026 consensus (SPIFFE/SPIRE, Vault 1.16, Gartner): the vault-as-safe-deposit-box model is **architecturally obsolete** for agentic systems. The new paradigm:

| Old Model (Vault) | New Model (Ephemeral + Workload Identity) |
|---|---|
| Credential lifetime: weeks–quarters | Minutes, often single-use |
| Trust basis: possession of secret | Cryptographic proof of workload identity |
| Exposure if stolen: full access until manual rotation | Capped at credential's short lifetime |
| Scaling limit: central vault throughput | Distributed issuance, no single choke point |

Key requirements NeoTrix lacks:
1. **SPIFFE-based workload identity** for agent processes (each agent gets a cryptographically attested identity, not a shared API key)
2. **RFC 8693 token exchange** for user-context delegation
3. **Per-task scoped credentials** (not per-agent standing keys)
4. **Credential lifetime = task lifetime** (minutes, not months)

**NeoTrix Impact**: Agent processes holding long-lived API keys for LLM providers, MCP tools, and external services. A leaked key gives an attacker 90+ days of access. With ephemeral credentials, exposure is capped at minutes.

**Sources**:
- CitadelCloudManagement (2026-07-27): "Workload Identity for AI Agents: SPIFFE, Token Exchange, and Ephemeral Credentials"
- UberEther (2026-05-22): "From Long-Lived API Keys to Short-Lived SVIDs"
- Johal (2026-05-03): "We Cut Secret Sprawl 80% by Migrating from Env Vars to Vault 1.16"

---

## 6. Defect: No Configuration-as-Control-Plane Safety Patterns

**Severity**: MEDIUM | **Category**: Config Safety

InfoQ (2026-03) documents hyperscaler convergence on config safety patterns that NeoTrix should adopt:

1. **Staged rollout with blast-radius containment**: Config changes apply to a small subset before global promotion. NeoTrix applies feature flags globally.
2. **Schema-validated config**: Strongly typed, schema-validated configuration that rejects invalid updates at the control plane level. NeoTrix has no config schema validation.
3. **Dependency-aware validation**: Understanding which modules/services are affected by a config change. NeoTrix has no dependency graph for feature flags.
4. **Automated rollback tied to SLOs**: Config changes auto-revert when health degrades. NeoTrix has no config-driven rollback.
5. **Versioned, immutable config**: Every config version is an immutable artifact with full auditability. NeoTrix config is mutable state.

**Source**: InfoQ "Configuration as a Control Plane" (2026-03-20)

---

## 7. Improvement: Compile-Time Configuration Validation

**Severity**: MEDIUM | **Category**: Build-Time Config

Several 2026 Rust crates enable compile-time config validation that NeoTrix should consider:

- **`concrete-config`**: Proc-macro that bakes TOML config into `const` values at build time. Validates TOML↔struct mapping at compile time. Works in `no_std` environments. (MSRV: Rust 1.88)
- **`inline-config`**: Compile-time config with path existence and type compatibility checked at compile time. Supports JSON/YAML/TOML.
- **`static_assertions`**: For validating compile-time config constraints (buffer sizes, connection limits, etc.)

NeoTrix should adopt **at minimum**: `const` assertions validating crypto config constraints at compile time (e.g., key sizes, hash output lengths, ML-KEM parameter sets).

**Sources**:
- concrete-config crate (crates.io)
- inline-config crate (crates.io, created 2026-02)
- OneUptime "How to Create Compile-Time Constants in Rust" (2026-01-30)

---

## 8. Defect: No CI/CD Integration for Config Compliance

**Severity**: MEDIUM | **Category**: Config Governance

2026 practice mandates **policy-as-code in CI pipelines** — automated checks that block unsafe config changes before deployment:

- Secret scanning in CI (TruffleHog, GitGuardian, SonarQube)
- Feature flag lifecycle validation (no unowned flags, no expired flags)
- Config schema validation on every commit
- Compliance mapping to CIS/NIST controls

NeoTrix has no CI integration for configuration compliance. Feature flag changes are not gated. Config drift is not detected in CI.

**Source**: DevSecOpsSchool CM guide (2026-02); Cloudaware CMDB best practices (2026-06)

---

## 9. Improvement: Runtime Config Injection Without Disk Persistence

**Severity**: HIGH | **Category**: Secret Architecture

2026 tools that solve the AI-agent secret leakage problem:

| Tool | Approach |
|---|---|
| **SecretEnv** | Multi-backend alias registry, no secrets on disk, one-command migration between backends |
| **dotvault** | Provider-agnostic, environment-aware, secrets resolved at runtime only |
| **sstart** | Zero-persistence CLI, MCP proxy for AI agents, template providers |
| **doppler/infisical** | `doppler run -- npm start` pattern — inject into process memory, gone when process exits |

NeoTrix should implement a **zero-disk secret injection pattern** for its agent processes:
1. Secrets stored in a dedicated backend (not `.env` files)
2. Injected into process memory at runtime via `secretenv run` / `dv run` / similar
3. Never written to disk, never readable by AI agent filesystem operations
4. Scoped per-agent, per-task, with automatic expiry

**Sources**:
- secretenv (github.com/TechAlchemistX/secretenv)
- dotvault (github.com/jondot/dotvault)
- sstart (github.com/dirathea/sstart)

---

## 10. Defect: No Config Versioning or Immutable Artifacts

**Severity**: MEDIUM | **Category**: Config Auditability

2026 baseline: every production deploy must map to **immutable config artifacts** with:
- Artifact digests (not mutable tags)
- Baseline manifest linking code version → config version → deployment state
- Rollback path verified before promotion

NeoTrix config is mutable state — feature flags can be changed at any point without version tracking. There is no config manifest, no config digest, no rollback verification.

**Source**: TheLinuxCode SCM guide §"Baseline Promotion Gates" (2026-02)

---

## Summary: New Defects Found

| # | Defect | Severity | Category |
|---|--------|----------|----------|
| 1 | No configuration baseline or drift detection | HIGH | CM |
| 2 | No single source of truth for config items | HIGH | CM |
| 3 | Feature flags without lifecycle management | MEDIUM | Feature Config |
| 4 | `.env` files leak secrets to AI agents | **CRITICAL** | Secret Mgmt |
| 5 | No ephemeral credential / workload identity | HIGH | Secret Lifecycle |
| 6 | No config-as-control-plane safety patterns | MEDIUM | Config Safety |
| 7 | No compile-time config validation | MEDIUM | Build-Time |
| 8 | No CI/CD config compliance integration | MEDIUM | Governance |
| 9 | No zero-disk secret injection | HIGH | Secret Architecture |
| 10 | No config versioning or immutable artifacts | MEDIUM | Auditability |

## Cross-Reference with Batch 690

| Batch 690 Finding | Batch 691 Connection |
|---|---|
| No trait abstraction for pluggable crypto | Config system should be pluggable too — crypto backend selection via config, not hard-coded `#[cfg]` |
| ML-KEM key reuse destroys forward secrecy | Ephemeral credentials (Defect #5) fix this pattern for key management |
| Ed25519 deprecated by NIST 2030 | Config baseline should track crypto algorithm deprecation timelines |
| BLAKE3 2-5× faster than SHA-256 on ARM | Compile-time config validation should enforce hash algorithm selection per target arch |
