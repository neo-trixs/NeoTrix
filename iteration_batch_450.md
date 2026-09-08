# Iteration Batch 450 — Configuration, Feature Flags, Secrets Management

**Date**: 2026-09-06
**Research Areas**: Configuration management, Feature flags, Secrets management
**Sources**: 20+ web sources (2025-2026 vintage)

---

## 1. Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| 1 | [12-Factor App Revisited 2026](https://bubble.ro/2026/07/16/the-twelve-factor-app-revisited-what-still-holds-in-2026/) | 2026-07 | Factor III env var gotchas |
| 2 | [12-Factor at 15](https://itnext.io/the-12-factor-app-15-years-later-does-it-still-hold-up-in-2026-c8af494e8465) | 2026-02 | Config not just env vars |
| 3 | [ConfigShield 12-Factor Checklist](https://configshield.app/blog/12-factor-app-config-environment-variables) | 2026-03 | Config classification + rotation |
| 4 | [Agentic Developer Cookbook 12-Factor](https://agenticdevelopercookbook.com/guidelines/implementing/infrastructure/twelve-factor-config) | 2026 | Typed config object at boot |
| 5 | [GrowthBook Flag Governance 2026](https://www.growthbook.io/blog/feature-flag-governance-framework) | 2026-09 | 6-pillar governance framework |
| 6 | [Flaggy.io Complete Playbook 2026](https://flaggy.io/blog/feature-flag-management/) | 2026-05 | Flag lifecycle + cleanup |
| 7 | [Enterprise Feature Flags Aug 2026](https://enterprise-software-review.contentwave.net/article/enterprise-feature-flags-updated-best-practices-for-aug-2026) | 2026-07 | WASM evaluation, GitOps flags |
| 8 | [ABTesting 14 Rules](https://abtesting.cc/blog/feature-flag-best-practices/) | 2026-08 | Per-flag death conditions |
| 9 | [GrowthBook Best Practices](https://www.growthbook.io/insights/feature-flag-best-practices) | 2026-07 | Fallback semantics, type taxonomy |
| 10 | [Vault + SOPS Guide 2026](https://core.cz/en/blog/2026/secrets-management-vault-sops-2026/) | 2025-12 | Dynamic secrets + SOPS for GitOps |
| 11 | [DapriPro Container Secrets](https://dapripro.com/secrets-management-in-containers-vault-sops-and-sealed-secrets/) | 2026-07 | Vault vs SOPS vs Sealed Secrets matrix |
| 12 | [Big Iron SOPS+age](https://www.bigiron.cc/guides/gitops-secrets-the-sops-and-age-pattern) | 2026-05 | SOPS+age sweet spot for small teams |
| 13 | [Codewithkarani SOPS+age](https://www.codewithkarani.com/blog/secrets-management-small-teams-sops-age) | 2026-06 | 2-person team workflow |
| 14 | [Peter Lee SOPS+GitOps](https://blog.peterlee.app/cloud/gitops-sops-encrypted-secrets/) | 2026-06 | Flux + SOPS age key rotation |
| 15 | [OneUptime SOPS+Vault+ESO](https://zeonedge.com/blog/secrets-management-2026-vault-sops-external-secrets-operator) | 2026-02 | SOPS for small, Vault for enterprise |
| 16 | [Sachith Monitoring SOPS+Vault](https://www.sachith.co.uk/secret-management-sops-vault-and-kms-monitoring-observability-practical-guide-jun-18-2026/) | 2026-06 | Observability of secrets pipelines |
| 17 | [OneUptime Config Drift GitOps](https://oneuptime.com/blog/post/2026-02-26-configuration-drift-detection-gitops/view) | 2026-02 | ArgoCD drift detection patterns |
| 18 | [ConfigTrace](https://configtrace.org/) | 2026 | Config drift monitoring tool |
| 19 | [SOPS+Vault Transit+Flux](https://oneuptime.com/blog/post/2026-03-05-encrypt-secrets-sops-vault-transit-flux-cd/view) | 2026-03 | Vault Transit as KMS for SOPS |
| 20 | [ITNotes SOPS+age](https://itnotes.dev/managing-git-secrets-safely-with-mozilla-sops-and-age-a-lightweight-hashicorp-vault-alternative/) | 2026-04 | Hybrid encryption model explained |

---

## 2. Defects Found in NeoTrix Design

### D1: No Startup Config Validation (CRITICAL)
**Source**: [1] [3] [4]
**Gap**: `NeoTrixConfig::load()` silently returns `Self::default()` on parse errors or missing file (config.rs:59-71). No validation of required fields (e.g., `provider`, `api_key`). Missing `DB_HOST` silently connects to localhost; missing `api_key` fails at LLM call time, not boot.
**Defect**: Config errors surface deep in execution as cryptic LLM failures instead of failing fast at startup.

### D2: Config Stored as Single TOML — No Environment Separation
**Source**: [2] [4]
**Gap**: One `~/.config/neotrix/config.toml` file serves all contexts. No staging/production/dev separation. No typed config object — fields accessed as raw strings via `save_field()`.
**Defect**: Violates Factor III — config varies between deploys but NeoTrix has no mechanism for this. A `config --env staging` path or per-workspace config overlay is missing.

### D3: Secrets Conflated with Non-Secret Config
**Source**: [1] [3] [20]
**Gap**: `api_key` (a secret) lives in the same TOML as `log_level` (non-secret). The `key_encryption.rs` module encrypts API keys in-place but the encrypted value is stored in the same file as plaintext config. `/proc/self/environ` and crash dumps would leak API keys.
**Defect**: No separation between sensitive and non-sensitive config. No `.env.example` pattern. No startup validation that distinguishes required-secret vs optional-non-secret fields.

### D4: `config encrypt-keys` / `decrypt-keys` Are TODO Stubs
**Source**: [10] [12] [13]
**Gap**: `entry.rs:174-179` — `run_config_encrypt_keys()` and `run_config_decrypt_keys()` are `eprintln!("TODO")` stubs. The CLI advertises `neotrix config encrypt-keys` but it does nothing.
**Defect**: Users trust the CLI to manage secrets but the encrypt/decrypt workflow is non-functional. No pre-commit hook protection exists.

### D5: Feature Flags Are Stub Commands — No Runtime Toggle
**Source**: [5] [6] [7] [8]
**Gap**: `FeaturesCommands::Enable` and `FeaturesCommands::List` (main.rs:264-271) both call `eprintln!("TODO")`. No feature flag registry, no environment-variable-based toggle, no kill-switch mechanism.
**Defect**: No way to roll back features without redeploy. No progressive rollout. No per-module enable/disable for the 8 domains.

### D6: No Feature Flag Lifecycle Management
**Source**: [5] [6] [8] [9]
**Gap**: No flag taxonomy (release/experiment/ops/permission), no owner assignment, no expiry tracking, no stale flag detection, no audit log of flag changes.
**Defect**: When flags eventually exist, there is no governance framework to prevent flag sprawl.

### D7: No Secrets Rotation Strategy
**Source**: [10] [11] [16]
**Gap**: API keys in `config.toml` have no rotation mechanism. `save_field("api_key", new_key)` is the only update path — manual, no audit trail, no automatic rotation.
**Defect**: Leaked credentials have indefinite blast radius. No TTL on API keys.

### D8: No Configuration Drift Detection
**Source**: [17] [18]
**Gap**: No mechanism to detect when running config diverges from expected config. No baseline snapshots. No diff between environments.
**Defect**: Silent config drift across environments (e.g., `prefer_free` changed in one place but not another) with no alerting.

### D9: No `.env.example` or Config Documentation
**Source**: [3] [4]
**Gap**: No `.env.example` file. No `config.example.toml`. New developers cannot discover required config fields without reading source code.
**Defect**: Onboarding friction. Unknown config fields lead to silent failures.

### D10: No Pre-Commit Secret Leak Prevention
**Source**: [12] [13] [15]
**Gap**: No `git-secrets` or `gitleaks` integration. No pre-commit hook to detect plaintext API keys. The `privacy_guard` module handles runtime egress but not repository-level leak prevention.
**Defect**: API keys can be committed to git history before any runtime guard activates.

---

## 3. Suggestions

### S1: Typed Config with Startup Validation
```rust
// Replace NeoTrixConfig with a validated typed struct
#[derive(Deserialize)]
pub struct AppConfig {
    pub provider: String,           // Required — fail fast if missing
    pub api_key: Option<SecretString>, // Optional, encrypted at rest
    pub default_model: String,      // Required
    pub log_level: Option<String>,  // Defaults to "info"
    // ... other fields
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let raw: RawConfig = load_raw()?;
        // Validate required fields
        // Classify secrets vs non-secrets
        Ok(Self { ... })
    }
}
```

### S2: Separate Secrets from Config (SOPS + age for Small Team)
- Use `~/.config/neotrix/config.toml` for non-secret config (log_level, color_mode, prefer_free)
- Use `~/.config/neotrix/secrets.enc.yaml` encrypted with SOPS + age for API keys
- `sops exec-env` injects secrets into the process at runtime — never touches disk as plaintext
- `.sops.yaml` lists per-developer age public keys as recipients

### S3: Implement Feature Flag Registry
```rust
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub owner: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub flag_type: FlagType, // release | experiment | ops | permission
}

pub struct FeatureRegistry {
    flags: HashMap<String, FeatureFlag>,
}

impl FeatureRegistry {
    pub fn load() -> Self { /* from env vars + config */ }
    pub fn is_enabled(&self, name: &str) -> bool { /* with fallback */ }
}
```

### S4: Config Drift Baseline + Diff
- Add `neotrix config baseline save` to snapshot current config as a baseline
- Add `neotrix config drift` to diff current vs baseline
- Alert on fields that differ from expected values

### S5: Pre-Commit Hook for Secret Leaks
- Add `gitleaks` or `detect-secrets` as a pre-commit hook
- Scan `config.toml` and any `.env` files before commit
- Block commits containing API key patterns (`sk-*`, `key-*`)

### S6: Config Change Audit Trail
- Log all `save_field()` calls to `~/.config/neotrix/audit.log` with timestamp, field, old_value (redacted), new_value (redacted)
- Feature flag changes also logged to the same audit trail

---

## 4. Priority Matrix

| Defect | Severity | Effort | Priority |
|--------|----------|--------|----------|
| D1: No startup validation | Critical | Low | P0 |
| D3: Secrets conflated | High | Medium | P0 |
| D4: Encrypt/decrypt stubs | High | Medium | P0 |
| D10: No pre-commit hooks | High | Low | P1 |
| D2: No env separation | Medium | High | P1 |
| D5: Feature flags stubs | Medium | Medium | P1 |
| D7: No rotation strategy | Medium | Medium | P2 |
| D9: No config documentation | Low | Low | P1 |
| D8: No drift detection | Low | High | P2 |
| D6: No flag lifecycle | Low | High | P3 |
