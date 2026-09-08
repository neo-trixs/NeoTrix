# Iteration Batch 745 — Configuration Format & Serialization Ecosystem Audit

**Date**: 2026-09-07
**Sources**: 12 web sources, 3 CVE advisories, 1 config-lang deep dive

---

## 1. TOML — `toml` crate 1.1.4 (spec-1.1.0)

### What's NEW

| Finding | Source |
|---------|--------|
| `toml` crate reached **v1.1.4+spec-1.1.0** (Jul 28, 2026) — TOML v1.1 spec landed Mar 2026 | crates.io |
| TOML 1.1 adds: multi-line inline tables, trailing commas, `\e` escape, `\xHH` escape, optional seconds in times | toml-rs/toml CHANGELOG |
| `toml_edit` v0.25.13 fixed **stack overflow** with many `=` in a row and many `+`/`-` in a row (Mar 2026) | CHANGELOG |
| **CVE-2026-16634** (CVSS 9.8): TOML::XS (Perl, tomlc99) stack overflow from deeply nested documents — unmaintained `tomlc99` library | NVD/GitHub Advisory |
| Rust `toml-rs` issue #206: recursion limits not enforced by default — deeply nested TOML (>100) can blow the stack on drop | GitHub issue |

### Defects Found

| # | Defect | Severity | NeoTrix Impact |
|---|--------|----------|----------------|
| **T1** | `toml` crate has **no default recursion depth limit** — maliciously nested TOML can stack-overflow on deserialization AND on recursive Drop | CRITICAL | NeoTrix config loader (`nt_io`) must enforce nesting depth limit (recommend max 128). CVE-2026-16634 proves real-world exploitability |
| **T2** | `toml_edit` panic on unclosed arrays/inline tables (fixed v0.25.11) — but `toml` crate still wraps `toml_edit` and inherits this path | HIGH | Defensive: wrap all TOML parsing in `catch_unwind` or pre-validate nesting |
| **T3** | TOML 1.1 adds `\e` (escape) and `\xHH` — potential injection vector if config values are passed to shell/exec | MEDIUM | Egress Privacy Guard must scrub TOML-escaped sequences in provider_params |
| **T4** | No `toml` crate feature flag for strict TOML 1.0-only parsing — NeoTrix may silently accept 1.1 features that break downstream parsers | LOW | Pin `toml` to `~1.0` or document TOML 1.1 requirement |

---

## 2. YAML — serde_yaml Deprecated, yaml_serde Fork

### What's NEW

| Finding | Source |
|---------|--------|
| `serde_yaml` by dtolnay is **archived and deprecated** — last release 0.9.33, no further versions planned | dtolnay/serde-yaml releases |
| Official replacement: **`yaml_serde`** crate (by yaml org, v0.10) — full API compat, rename `serde_yaml` → `yaml_serde` | yaml/yaml-serde GitHub |
| CVE on serde_yaml 0.6.0: XXE (XML External Entity) injection vulnerability published 2026 | CVEDetails |
| YAML security guides in 2026 emphasize: disable dangerous constructors, validate schemas, use `yamllint` | devsecopsschool.com, tecnoyfoto.com |
| `yaml_serde` adds CLAUDE.md — indicating active AI-assisted maintenance | yaml/yaml-serde |

### Defects Found

| # | Defect | Severity | NeoTrix Impact |
|---|--------|----------|----------------|
| **Y1** | **serde_yaml is unmaintained** — NeoTrix must migrate to `yaml_serde` (or remove YAML support entirely). CVE history + no patches = supply chain liability | CRITICAL | `Cargo.toml` dependency audit: if `serde_yaml` present, replace with `yaml_serde` or drop YAML |
| **Y2** | YAML 0.6.0 XXE vulnerability — even if NeoTrix doesn't parse XML-in-YAML, the C library behind serde_yaml (libyaml) processes YAML which can contain anchors that expand recursively | HIGH | YAML parser must enforce recursion/alias expansion limits. NeoTrix already uses R-P1 (no unsafe) but libyaml is C |
| **Y3** | YAML type coercion is implicit and ambiguous (`"10"` → int vs string, `yes` → bool) — config drift between environments | MEDIUM | Define explicit schema for any YAML config; reject implicit type coercion in production paths |
| **Y4** | YAML security posture: no safe-loading mode by default — `yaml.load()` vs `yaml.safe_load()` pattern doesn't translate well to serde | MEDIUM | Enforce `yaml_serde` with tagged deserialization only; never accept arbitrary `Value` from untrusted YAML |

---

## 3. Configuration Format Landscape 2026

### What's NEW

| Finding | Source |
|---------|--------|
| **CUE, Pkl (Apple), KCL form the "big three"** of 2026 config languages — all data+constraints unified | youngju.dev deep dive |
| **KDL 2.0 spec** published Jan 2026 — node-oriented, XML-like, explicit types, adopted by Zellij, niri | kdl.dev |
| **Pkl** (Apple, Feb 2024) gaining traction — strongly typed, IDE support, generated types | youngju.dev |
| **Nix** remains niche but fanatical — used by Jane Street, Anduril, Determinate Systems | youngju.dev |
| **JSON5** vs JSONC: JSON has RFC, JSON5 has spec, JSONC has only "defaults" — dialect confusion | zeroutil.com |
| JSON 2026 reality: JSON is "boring" (good), but 3 dialects (JSON/JSON5/JSONC) create parsing ambiguity | zeroutil.com |

### Defects Found

| # | Defect | Severity | NeoTrix Impact |
|---|--------|----------|----------------|
| **C1** | **No CUE/Pkl/KDL parser in NeoTrix** — the "big three" config languages have no Rust crates integrated. Config format support is limited to TOML/JSON (YAML deprecated) | MEDIUM | Consider `kdl` crate (Rust, KDL 2.0) for NT-WORLD/NT-MEMORY config. KDL's explicit types avoid YAML's coercion bugs |
| **C2** | **JSON dialect confusion** — NeoTrix may parse JSONC or JSON5 but advertise "JSON" — tsconfig allows trailing commas, standard JSON doesn't | MEDIUM | Document which JSON dialect NeoTrix accepts. If using `serde_json`, it's strict RFC 7159 only |
| **C3** | **No config schema validation layer** — all three formats (TOML/YAML/JSON) lack a unified schema validation step in NeoTrix config pipeline | HIGH | Add `JsonSchema` derive + validation gate to all config structs. `schemars` crate integration |
| **C4** | **Config format selection is implicit** — file extension based detection (.toml/.yaml/.json) but no MIME sniffing or magic-byte validation | LOW | Add file magic validation before parsing; reject ambiguous files |
| **C5** | **TOML trailing commas (1.1) may break strict parsers** — downstream tools expecting TOML 1.0 will reject 1.1 configs | MEDIUM | Document TOML version requirement; or pin to 1.0-compatible subset |

---

## 4. Cross-Cutting Defects (All Formats)

| # | Defect | Severity | Fix |
|---|--------|----------|-----|
| **X1** | **No recursion depth limit on ANY parser** — TOML, YAML, JSON all vulnerable to stack overflow from deeply nested input | CRITICAL | Add `recursion_limit` config parameter; default to 128 for TOML/YAML, 256 for JSON |
| **X2** | **No parsing timeout** — malicious config can hang the parser with deeply nested structures or huge documents | HIGH | Add `Duration` timeout to all `from_str`/`from_reader` calls |
| **X3** | **Error messages leak internal paths** — parser errors may include file system paths in stack traces | MEDIUM | Sanitize parser error output in production (Egress Privacy Guard scope expansion) |
| **X4** | **No config signing/verification** — untrusted config files loaded without integrity check | MEDIUM | Consider Ed25519 signing for critical configs (LLM provider keys, KB paths) |

---

## 5. Actionable Recommendations (Priority Order)

1. **[CRITICAL]** Add recursion depth limit (128) to all TOML/YAML parsers — CVE-2026-16634 proves exploitability
2. **[CRITICAL]** Audit `serde_yaml` dependency — migrate to `yaml_serde` or remove YAML support
3. **[HIGH]** Add `schemars` JSON Schema validation to all config structs
4. **[HIGH]** Add parsing timeout (5s default) to all `from_str` calls
5. **[MEDIUM]** Evaluate KDL 2.0 crate (`kdl`) for NT-WORLD/NT-MEMORY config — explicit types, no coercion
6. **[MEDIUM]** Document TOML version requirement (1.0 vs 1.1) — pin Cargo.toml accordingly
7. **[LOW]** Add file magic validation before config parsing

---

## Sources Cited

1. https://releasealert.dev/cratesio/toml — toml crate release history
2. https://github.com/toml-rs/toml/blob/main/crates/toml_edit/CHANGELOG.md — toml_edit changelog
3. https://github.com/toml-rs/toml/issues/206 — stack overflow recursion issue
4. https://github.com/yaml/yaml-serde — yaml_serde official fork
5. https://github.com/dtolnay/serde-yaml/releases — serde_yaml deprecation notice
6. https://www.cvedetails.com/vulnerability-list/vendor_id-20180/product_id-57307/version_id-1433069/ — serde_yaml XXE CVE
7. https://github.com/advisories/GHSA-hr7v-gv37-xpx2 — CVE-2026-16634 TOML::XS stack overflow
8. https://cvefeed.io/vuln/detail/CVE-2026-16634 — TOML::XS vulnerability details
9. https://www.youngju.dev/blog/culture/2026-05-16-configuration-languages-2026 — Config languages deep dive
10. https://kdl.dev/spec — KDL 2.0 specification
11. https://hashnode.com/posts/json-vs-json5-vs-yaml — JSON dialect comparison
12. https://zeroutil.com/blog/developer-json-guide/ — JSON 2026 usage guide
