# Iteration Batch 588 — Configuration, Env Vars, Secrets Management

**Date**: 2026-09-06
**Upstream**: Batch 587 (write skew+GWT cascade, MVCC×VSA explosion, missing transactional outbox, 2PC SPOF, Saga domain-awareness)
**Research Scope**: Configuration management, environment variables, secrets management

---

## What's NEW vs Batch 587

Batch 587 focused on **transactional consistency** (write skew, outbox, 2PC, Saga). Batch 588 shifts to **configuration and secrets surface area** — how NeoTrix loads config, manages secrets at runtime, and handles feature-flag-driven code paths. Five new defects identified.

---

## Defect 1: Feature Flag State Explosion × Combinatorial Test Gap

**Source**: Zylos Research 2026, GrowthBook 2026, Imperialis Tech 2026

**Finding**: 2026 feature flag systems now support multivariate flags (A/B/C/D), percentage rollouts, targeted user segments, and kill switches. With N flags, combinatorial state space is 2^N. The industry reports 10 flags = 1,024 states; practical testing covers <1%. NeoTrix modules already use internal feature toggles (SEAL stages, GWT attention modes, VSA projection modes) but have **zero matrix testing infrastructure**.

**Defect**: NeoTrix has no feature-flag matrix testing framework. When flags interact (e.g., GWT resonance mode ON + VSA HyperCube projection ON + SEAL distillation ON), write skew + GWT silent corruption (batch 587) could compound across flag combinations. The corruption is non-deterministic per flag state.

**Impact**: Silent data corruption cascade only manifests under specific flag combinations, undetectable in unit tests. Batch 587's write skew finding is amplified by flag-state-dependent execution paths.

**Fix**: Implement flag-state-aware test injection. Each SelfTest must declare which flags it's sensitive to. CI runs matrix tests across critical flag combinations. Flag lifecycle requires expiry dates and owner assignment (anti-permanent-flag pattern from industry).

**Sources**:
- https://zylos.ai/research/2026-02-12-feature-flags/
- https://imperialis.tech/en/blog/feature-flags-feature-management-production-2026
- https://www.growthbook.io/insights/ab-testing-and-feature-flags

---

## Defect 2: No Transactional Outbox for Feature Flag State Changes

**Source**: Imperialis Tech 2026, Zylos Research 2026

**Finding**: 2026 best practice mandates that feature flag state changes must be audited (who changed what, when, why) and that flag changes are **configuration transactions**, not just API calls. Production flag changes require two-person approval workflows.

**Defect**: NeoTrix EventBus lacks a transactional outbox for feature-flag-triggered side effects (batch 587 identified missing transactional outbox generally). Specifically: when a flag state change triggers a module reconfiguration (e.g., kill-switch disables an LLM provider), the flag change + provider deregistration are not atomic. If the flag persists but deregistration fails, the system is in an inconsistent state — modules think the provider is disabled but it's still receiving traffic.

**Impact**: Partially-disabled provider continues receiving requests after kill-switch activation. Users experience degraded quality without knowing the provider was supposed to be offline.

**Fix**: Feature flag state changes that trigger side effects must go through the transactional outbox pattern (batch 587's fix). Implement `FlagStateChangeEvent` with outbox persistence before side-effect execution. Two-phase: (1) persist flag state + outbox entry, (2) execute side effects, (3) mark outbox consumed.

**Sources**:
- https://imperialis.tech/en/blog/feature-flags-feature-management-production-2026
- https://zylos.ai/research/2026-02-12-feature-flags/

---

## Defect 3: .env File ≠ Secrets — NeoTrix Configuration Layer Confusion

**Source**: DEV Community 2026, CodeTidy 2026, OneUptime 2026

**Finding**: 2026 industry consensus is stark: `.env` files are for **non-sensitive configuration** (port, log level, feature flags). Secrets (API keys, DB URLs, tokens) must never live in `.env` files — they go to dedicated secrets managers (Vault, AWS Secrets Manager, SOPS+age). The Twelve-Factor App methodology explicitly separates "config that varies between deploys" from "secrets that grant access."

**Defect**: NeoTrix uses a single configuration loading path (`.env` / `config.toml`) for both non-sensitive config AND secrets (LLM API keys, KB database credentials, Shield proxy credentials). There's no architectural separation between config-layer and secrets-layer. If `.env` is committed (accidental git-add), all secrets are exposed. No validation at startup catches missing or malformed secrets.

**Impact**: Credential leakage via `.env` in git history. No startup validation means missing API key = runtime crash instead of fail-fast. Staging `.env` with production secrets causes data corruption (staging writes to production DB).

**Fix**: Implement two-layer config architecture:
1. **Config Layer** (`.env` / `config.toml`): non-sensitive runtime config, committable via `.env.example`
2. **Secrets Layer**: SOPS+age encrypted files (for GitOps) or environment injection (for production)
3. **Startup Validator**: fail-fast on missing required secrets, type-coerce all config values
4. **Environment Isolation**: staging secrets never load in production context

**Sources**:
- https://dev.to/_d7eb1c1703182e3ce1782/environment-variables-best-practices-the-complete-developer-guide-2026-3bic
- https://codetidy.dev/blog/dotenv-best-practices
- https://oneuptime.com/blog/post/2026-01-25-dotenv-configuration-nodejs/view

---

## Defect 4: SOPS+age Key Rotation Not Wired to NeoTrix Key Lifecycle

**Source**: bigiron.cc 2026, khimananda.com 2026, mylinux.work 2026

**Finding**: 2026 SOPS+age has become the de facto standard for GitOps secret encryption (1-10 person teams, CNCF-graduated SOPS). Key rotation via `sops updatekeys` is mandatory quarterly hygiene. But key rotation ≠ secret rotation — they're separate steps. SOPS adds new recipients without touching encrypted payload; the actual secrets (passwords, tokens) must be rotated separately.

**Defect**: NeoTrix has no SOPS+age integration. Secrets for local development are either hardcoded or in plaintext `.env`. There's no encrypted-at-rest mechanism for GitOps workflows. NeoTrix's KB stores LLM provider API keys in plaintext SQLite — no encryption at rest, no rotation protocol, no key derivation separation.

**Impact**: If the KB SQLite file is exfiltrated, all LLM API keys are compromised in plaintext. No quarterly rotation means compromised keys remain valid indefinitely. NeoTrix's Shield module (anti-detection, proxy pool) manages its own proxy credentials separately — no unified rotation schedule.

**Fix**:
1. Adopt SOPS+age for local dev secret encryption (`.enc.yaml` convention)
2. KB SQLite must encrypt sensitive fields (API keys, tokens) with age-derived keys
3. Implement quarterly key rotation schedule with `sops updatekeys` + secret rotation
4. Shield proxy credentials must rotate on same schedule as LLM keys

**Sources**:
- https://www.bigiron.cc/guides/gitops-secrets-the-sops-and-age-pattern
- https://khimananda.com/blog/manage-secrets-with-sops-and-age
- https://mylinux.work/guides/managing-secrets-with-sops-and-age/

---

## Defect 5: Vault Dynamic Secrets vs NeoTrix Static Credentials — 2PC Coordinator SPOF Extended

**Source**: youngju.dev 2026, unixy.io 2026, mylinux.work 2026

**Finding**: 2026 Vault's core value over SOPS is **dynamic secrets** — per-request database credentials with TTL expiry, auto-revocation, and audit trail. NeoTrix batch 587 identified 2PC coordinator as SPOF. The deeper issue: NeoTrix's KB uses **static credentials** for all database connections, LLM API keys, and inter-service auth. These never expire, never rotate, and have no revocation mechanism.

**Defect**: When a 2PC coordinator fails (batch 587 SPOF), in-flight transactions hold locks with static credentials that can't be revoked without service restart. Vault dynamic secrets would generate per-transaction credentials with automatic TTL expiry — a failed coordinator's in-flight transactions would naturally expire their credentials, releasing locks. NeoTrix has no dynamic secret mechanism; failed coordinators leave orphaned locks indefinitely.

**Impact**: 2PC coordinator failure (batch 587) now has two compounding effects: (1) SPOF blocks new transactions, (2) static credentials prevent lock cleanup without full restart. Combined with write skew (batch 587), this creates a deadlock cascade.

**Fix**:
1. Short-term: Implement credential TTL with auto-revocation for KB connections (even without Vault)
2. Medium-term: Add Vault Transit Engine for encryption-as-service (eliminates key exposure)
3. Long-term: Migrate to dynamic database credentials via Vault PostgreSQL/MySQL engine
4. 2PC coordinator failure should trigger automatic credential revocation + lock cleanup

**Sources**:
- https://www.youngju.dev/blog/culture/2026-05-16-devops-secrets-management-2026-doppler-infisical-hashicorp-vault-aws-secrets-manager-1password-sops-age-deep-dive.en
- https://unixy.io/blog/secrets-management-2026/
- https://mylinux.work/guides/secrets-management-comparison/

---

## Summary of Compounding Defects (Batch 587 → 588)

| Batch 587 Defect | Batch 588 Amplification |
|---|---|
| Write skew + GWT = silent corruption | Feature flag state explosion creates non-deterministic corruption paths (Defect 1) |
| MVCC × VSA = storage explosion | .env plaintext secrets in KB SQLite = no encryption at rest (Defect 3) |
| Missing transactional outbox | Flag state changes lack outbox persistence (Defect 2) |
| 2PC coordinator SPOF | Static credentials prevent lock cleanup on coordinator failure (Defect 5) |
| Saga needs domain-awareness | SOPS+age adoption required for domain-secret isolation (Defect 4) |

---

## Sources Cited

1. Zylos Research — Feature Flags Architecture 2026: https://zylos.ai/research/2026-02-12-feature-flags/
2. Imperialis Tech — Feature Management in Production 2026: https://imperialis.tech/en/blog/feature-flags-feature-management-production-2026
3. GrowthBook — A/B Testing and Feature Flags 2026: https://www.growthbook.io/insights/ab-testing-and-feature-flags
4. DEV Community — Environment Variables Best Practices 2026: https://dev.to/_d7eb1c1703182e3ce1782/environment-variables-best-practices-the-complete-developer-guide-2026-3bic
5. CodeTidy — .env File Best Practices 2026: https://codetidy.dev/blog/dotenv-best-practices
6. OneUptime — dotenv Configuration 2026: https://oneuptime.com/blog/post/2026-01-25-dotenv-configuration-nodejs/view
7. BigIron — GitOps SOPS+age 2026: https://www.bigiron.cc/guides/gitops-secrets-the-sops-and-age-pattern
8. Khimananda — SOPS and age Guide 2026: https://khimananda.com/blog/manage-secrets-with-sops-and-age
9. mylinux.work — Secrets Management Comparison 2026: https://mylinux.work/guides/secrets-management-comparison/
10. youngju.dev — DevOps Secrets Management Deep Dive 2026: https://www.youngju.dev/blog/culture/2026-05-16-devops-secrets-management-2026-doppler-infisical-hashicorp-vault-aws-secrets-manager-1password-sops-age-deep-dive.en
11. unixy.io — Secrets Management 2026: https://unixy.io/blog/secrets-management-2026/
12. TheLinuxCode — Feature Flags vs A/B Testing 2026: https://thelinuxcode.com/feature-flags-vs-ab-testing-a-practical-2026-era-guide-from-the-trenches/
13. RBMSoft — Feature Flag Management Enterprise Guide 2026: https://rbmsoft.com/blogs/feature-flag-management
14. npm Compare — dotenv ecosystem comparison: https://npm-compare.com/@dotenvx/dotenvx,config,dotenv,dotenv-flow,dotenv-safe
