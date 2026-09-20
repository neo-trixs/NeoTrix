# NeoTrix Development Rules (R-P161–R-P211)

> Absorbed from 56 projects (1000+ URLs) — 2026-09-20
> Prior rules: R-P1–R-P101 (coding baseline), R-P111–R-P115 (architecture governance), R-P116–R-P160 (prior absorption wave)

---

## Security Hardening (R-P161–R-P170)

### R-P161: Adversarial Input Pipeline Stages
- **Source**: garak (9.3K★) + OWASP ASVS (3.6K★)
- **Rule**: Every external input must pass a 7-stage defense pipeline before reaching core logic: InputValidation → OutputSanitization → BehaviorMonitoring → AnomalyDetection → ResponseFiltering → MemoryScrubbing → AuditLogging.
- **Rationale**: garak demonstrates that LLM-adjacent systems face prompt injection, jailbreaks, and data exfiltration. ASVS V5 requires input validation at trust boundaries. A single-stage check is insufficient — adversaries chain bypasses across stages.
- **Implementation**: Extend `nt_shield::adversarial_pipeline` stages. Each stage returns `DefenseError` or passes cleaned `String` forward. Stages must be independently testable and composable. Never skip stages for "trusted" inputs.

### R-P162: Secret Scanning Pre-Commit Gate
- **Source**: trufflehog (28K★)
- **Rule**: No commit may contain hardcoded secrets, API keys, tokens, or private keys. Run trufflehog-style detector (700+ patterns) as a pre-commit hook AND a CI gate. Use entropy analysis for unknown patterns.
- **Rationale**: trufflehog found secrets in 1 in every 10 public repos. NeoTrix handles sensitive LLM API keys and proxy credentials — a leak is catastrophic.
- **Implementation**: Integrate `nt_shield::secret_scanner` with regex detectors + entropy heuristic. Block git commits on detection. Scan `.env`, config files, and source code. Rotate any detected secrets immediately.

### R-P163: Container Image Hardening
- **Source**: trivy (38K★)
- **Rule**: All container images must pass CVE scanning (OS + app-level), SBOM generation, and IaC misconfiguration checks before deployment. No images from unverified registries. Pin base images by digest, not tag.
- **Rationale**: trivy identifies vulnerabilities in 70%+ of container images. Supply chain attacks target base images and dependencies.
- **Implementation**: Run `nt_shield::container_scan` (trivy-compatible) on every image build. Generate SBOM (SPDX/CycloneDX). Fail CI on critical CVEs. Use distroless or scratch base images.

### R-P164: LLM Red Team Rotation
- **Source**: garak (9.3K★) + Azure/PyRIT (4.5K★)
- **Rule**: Run automated LLM adversarial testing monthly. Rotate attack probes across: prompt injection, jailbreak, data extraction, hallucination forcing, and role-play abuse. Log all findings to `nt_shield::red_team` ledger.
- **Rationale**: garak's probe library evolves — static defenses decay. PyRIT demonstrates adaptive multi-turn attacks that bypass single-turn filters.
- **Implementation**: Schedule monthly garak/PyRIT scan against NeoTrix LLM endpoints. Classify findings by OWASP LLM Top 10. Track fix rate. Auto-disable models that fail critical probes.

### R-P165: Network Egress Rule Engine
- **Source**: SpiderFoot (22.4K★) + nt_shield_stealth_net
- **Rule**: All outbound network requests must pass through a rule engine with DomainSuffix/DomainExact/Cidr/SchemeMatch conditions. Default action is deny. Rules must support TTL-based temporary entries and external rule sources.
- **Rationale**: SpiderFoot demonstrates that compromised agents exfiltrate data via DNS, HTTPS, and ICMP. The existing `rules.rs` infrastructure must be enforced on all egress.
- **Implementation**: Wire `RuleEngine::matches()` into all HTTP clients, DNS resolvers, and socket operations. Never bypass for "internal" requests. Log all deny decisions.

### R-P166: Attack Class Registry Expansion
- **Source**: Cloudflare security-audit-skill + OWASP ASVS
- **Rule**: Maintain a living attack class registry covering: Injection, AccessControl, ResourceFileHandling, CryptographySecrets, BusinessLogic, FeatureAbuse, ChainedAttacks, Wildcard. Each class must have defined dangerous sinks, sub-categories, and hunting angles.
- **Rationale**: The `attack_class_registry` in nt_shield currently covers 8 classes. Cloudflare's framework adds hunting angles (SadPaths, Boundaries, ParserDisagreement) that catch logic bugs static analyzers miss.
- **Implementation**: Extend `AttackClassRegistry::register_defaults()` with Cloudflare hunting angles. Map each class to domain companions. Use during audit phases (recon → hunt → validate).

### R-P167: Proxy Chain Rotation Discipline
- **Source**: SpiderFoot + nt_shield_stealth_net proxy_pool
- **Rule**: OSINT crawling must rotate proxies per-request or per-domain. Never reuse the same exit IP for more than 10 requests. Monitor pool health continuously. Rotate on detection, not on schedule.
- **Rationale**: Target sites fingerprint and block static IPs. SpiderFoot's proxy rotation avoids rate limiting and IP bans during reconnaissance.
- **Implementation**: Use `proxy_pool::rotate_on_demand()` not `rotate_on_timer()`. Monitor `pool_health` metrics. Eject unhealthy proxies immediately. Maintain minimum 5 viable proxies for OSINT tasks.

### R-P168: Audit Logging Immutable Ledger
- **Source**: OWASP ASVS V7 + garak
- **Rule**: All security-relevant actions (auth, data access, config changes, defense verdicts) must write to an append-only audit log. Logs must be tamper-evident (hash chain or Merkle tree). Retain for 90 days minimum.
- **Rationale**: ASVS V7 mandates audit logging for compliance. garak's defense pipeline already generates audit events — these must persist immutably.
- **Implementation**: Extend `nt_shield_audit` with append-only storage. Each entry includes: timestamp, actor, action, verdict, input hash. Chain hashes for tamper detection. Export to `nt_memory::evidence_store`.

### R-P169: Sandbox Escalation Detection
- **Source**: garak sandbox + nt_shield_sandbox
- **Rule**: Every sandboxed execution must monitor for escape attempts: filesystem access outside boundary, network connections to non-allowlisted hosts, process spawning, and environment variable leakage. Kill on first violation.
- **Rationale**: garak demonstrates that LLM agents can be tricked into sandbox escape via crafted prompts. nt_shield_sandbox provides the infrastructure but needs continuous monitoring.
- **Implementation**: Use `nt_shield_sandbox::judge` for continuous evaluation. Set up seccomp/AppArmor profiles. Monitor syscalls via `nt_shield_sandbox::device`. Fail-closed: terminate on any boundary violation.

### R-P170: Firmware & Binary Analysis Gate
- **Source**: nt_shield_audit firmware_analyzer + binary_analyzer
- **Rule**: Before integrating any third-party binary or firmware component, run: entropy analysis (detect packing/obfuscation), string extraction (find hidden commands), signature verification, and YARA rule matching.
- **Rationale**: Supply chain attacks increasingly target build artifacts and firmware. Binary analysis catches malware that source review misses.
- **Implementation**: Add `binary_analyzer::analyze()` to the dependency review pipeline. Flag high-entropy sections (>7.5 bits/byte). Verify signatures against known-good databases. Match against YARA rules in `nt_shield_audit::yara_scanner`.

---

## OSINT Best Practices (R-P171–R-P178)

### R-P171: Social Username Enumeration Protocol
- **Source**: sherlock (92.1K★)
- **Rule**: When enumerating social accounts, respect rate limits per-platform. Use HTTP status code analysis (200 = exists, 302 = redirect = exists, 404 = not found, 429 = rate limited). Never attempt credential stuffing. Log findings to `nt_world::osint_social` with source attribution.
- **Rationale**: sherlock covers 400+ platforms but each has different anti-abuse. Uncontrolled enumeration triggers bans and legal risk.
- **Implementation**: Implement per-platform rate limiters (1-5 req/sec default). Cache results for 24h. Deduplicate by normalization (case, underscores). Output structured JSON with platform, username, url, status, timestamp.

### R-P172: OSINT Data Provenance Tracking
- **Source**: SpiderFoot (22.4K★)
- **Rule**: Every OSINT finding must record: source URL, retrieval timestamp, data hash, confidence score, and correlation ID. Raw data must be stored separately from derived intelligence.
- **Rationale**: SpiderFoot's correlation engine links data across modules. Without provenance, findings cannot be verified or audited.
- **Implementation**: Use `nt_memory::evidence_store` with `DataSovereigntyProof`. Raw responses stored in `nt_world::osint_raw`. Derived facts in `nt_world::osint_derived`. Link via correlation_id.

### R-P173: Email Harvesting Ethical Boundary
- **Source**: theHarvester (17.5K★)
- **Rule**: Email/subdomain enumeration must only use publicly available sources (search engines, certificate transparency logs, DNS records). Never scrape contact forms, never send unsolicited emails. Respect robots.txt.
- **Rationale**: theHarvester aggregates from public sources but boundary violations (scraping private databases, sending probes) create legal liability.
- **Implementation**: Limit to: CT logs (crt.sh), DNS MX/NS records, search engine dorking, GitHub/GitLab leaks. Block private API access. Log all queries for audit.

### R-P174: OSINT Correlation Engine Rules
- **Source**: SpiderFoot + BloodHound.py
- **Rule**: When correlating OSINT data across modules, apply confidence weighting: first-party observation (1.0), second-party report (0.8), third-party inference (0.5). Never treat correlation as confirmation. Require ≥2 independent sources for high-confidence claims.
- **Rationale**: SpiderFoot's correlation engine and BloodHound's graph analysis demonstrate that single-source data is unreliable. Correlation ≠ causation.
- **Implementation**: Implement `nt_world::correlation::confidence_score()` with source provenance weights. Flag single-source findings as "unverified". Require manual review for high-stakes conclusions.

### R-P175: AD Security Graph Audit Trail
- **Source**: BloodHound.py (2.4K★)
- **Rule**: When querying Active Directory data, log all LDAP queries, enumerate permissions used, and record data access scope. Never modify AD state during reconnaissance. Use read-only LDAP binds.
- **Rationale**: BloodHound demonstrates that AD graph data is highly sensitive. Unauthorized access or modification violates security policies.
- **Implementation**: Use `nt_world::ad_graph` with read-only LDAP binds only. Log all queries to audit trail. Enforce principle of least privilege. Export findings via `nt_memory::evidence_store` with sovereignty proof.

### R-P176: OSINT Rate Limiting Per-Target
- **Source**: sherlock + SpiderFoot
- **Rule**: Implement per-target rate limiting: max 10 requests/minute per domain for OSINT crawling. Use exponential backoff on 429 responses. Rotate user agents and proxies.
- **Rationale**: Aggressive crawling triggers IP bans and legal action. Rate limiting preserves access and reduces detection.
- **Implementation**: Use `nt_shield_stealth_net::proxy_pool` for rotation. Implement per-domain request counters. Backoff: 1s → 2s → 4s → 8s → 30s max. Reset on successful response.

### R-P177: OSINT Report Sanitization
- **Source**: SpiderFoot + theHarvester
- **Rule**: OSINT reports must strip: credentials, tokens, personal identifiers (unless target), internal infrastructure details. Reports are classified by sensitivity level (Public/Confidential/Secret).
- **Rationale**: Raw OSINT output often contains incidental sensitive data. Unsanitized reports create exposure risk.
- **Implementation**: Apply `nt_memory::privacy::DataSovereigntyProof` to all OSINT reports. Auto-classify by content patterns (emails → Confidential, passwords → Secret). Strip/encrypt before storage.

### R-P178: Multi-Source Intelligence Fusion
- **Source**: SpiderFoot (22.4K★) + sherlock (92.1K★)
- **Rule**: Fuse intelligence from ≥3 independent sources before acting. Each source contributes evidence weight. Contradictions must be flagged and resolved before recommendations.
- **Rationale**: Single-source intelligence is unreliable. SpiderFoot's strength is fusing 200+ modules — NeoTrix must match this rigor.
- **Implementation**: Implement `nt_world::fusion::multi_source_evidence()`. Require ≥3 sources for automated action. Flag contradictions for manual review. Weight by source reliability (historical accuracy).

---

## Memory Optimization (R-P179–R-P185)

### R-P179: Three-Tier Memory Architecture
- **Source**: mem0 (65.6K★) + Letta (24.8K★)
- **Rule**: Implement strict 3-tier memory: L1 (working/VRAM — hot, <1MB), L2 (episodic/RAM — warm, <1GB), L3 (semantic/NVMe — cold, unlimited). Auto-promote on access frequency, auto-demote on staleness.
- **Rationale**: mem0's unified memory layer and Letta's tiered architecture demonstrate that flat memory doesn't scale. Context windows are scarce resources.
- **Implementation**: Use `nt_memory::tiered_pipeline` with access frequency tracking. L1: last-access <5min, L2: last-access <24h, L3: everything else. Promote/demote automatically. Track hit rates.

### R-P180: Memory Consolidation During Idle
- **Source**: Letta (24.8K★)
- **Rule**: During idle periods (no user interaction for >5min), consolidate working memory into episodic memory. Merge duplicates, resolve conflicts, update relevance scores. Never consolidate during active conversations.
- **Rationale**: Letta's self-editing memory demonstrates that consolidation during idle preserves context without impacting response latency.
- **Implementation**: Use `nt_memory::consolidation::consolidator` with idle detection. Trigger consolidation only when `idle_duration > 300s`. Merge rules: prefer newer timestamps, keep highest confidence, deduplicate by content hash.

### R-P181: KV Cache Compression for Long Sessions
- **Source**: KVSharer (arXiv:2506.17533) + KIVI
- **Rule**: For sessions >16K tokens, compress KV cache using per-channel INT8 quantization for keys and per-token INT2 quantization for values. Preserve 32 high-attention heads in FP16.
- **Rationale**: KVSharer achieves 6x memory reduction with 0.05% quality loss. Long sessions without compression exhaust GPU memory.
- **Implementation**: Integrate with `nt_memory::lmcache_hotstore`. Monitor cache size. Trigger compression at 16K tokens. Track quality delta. Alert if compression causes >1% quality drop.

### R-P182: Memory Deduplication by Content Hash
- **Source**: mem0 (65.6K★)
- **Rule**: Before writing to any memory tier, compute content hash (SHA-256 of normalized text). If hash exists, update timestamp and relevance score instead of creating duplicate entry.
- **Rationale**: mem0 deduplicates at ingestion, preventing memory bloat. Duplicates waste storage and confuse retrieval.
- **Implementation**: Add hash check to `nt_memory::write_guard`. Normalize: lowercase, collapse whitespace, strip punctuation. Store hash alongside entry. Batch deduplication during consolidation.

### R-P183: Memory Decay with Salience Weighting
- **Source**: Letta + nt_memory::decay_forgetting
- **Rule**: Apply exponential forgetting curves to memory entries: salience_score × e^(-λt). High-salience entries decay slower. Salience increases with access frequency and decreases with age.
- **Rationale**: Human memory decays predictably. Letta's forgetting curves and NeoTrix's `decay_forgetting::curves` already implement this — enforce it universally.
- **Implementation**: Use `nt_memory::decay_forgetting::curves::exponential_decay()`. λ = 0.1 (default), adjusted by salience. Prune entries below threshold during consolidation. Never decay entries marked as "permanent".

### R-P184: Memory Conflict Resolution Protocol
- **Source**: mem0 + nt_memory::typed_memory::conflict
- **Rule**: When memory entries conflict (same entity, different facts), apply resolution: recency (newer wins), source authority (primary > secondary), confidence (higher wins). Log all conflicts and resolutions.
- **Rationale**: Conflicting memories cause agent confusion. mem0 handles this via entity linking, NeoTrix via conflict module — enforce the protocol.
- **Implementation**: Use `nt_memory::typed_memory::conflict::resolve()`. Log conflicts to `nt_memory::conflict_log`. Resolution must be deterministic for same inputs. Allow manual override.

### R-P185: Context Window Budget Allocation
- **Source**: NeoTrix axiom A2 (Context as Scarce Resource)
- **Rule**: Every context window insertion must declare its budget allocation. System prompt ≤20%, retrieved memory ≤30%, conversation history ≤50%. Hard cap at 100% — never overflow.
- **Rationale**: Context window exhaustion causes truncation of critical information. Budget enforcement prevents one component from monopolizing space.
- **Implementation**: Add `budget_tokens: usize` to all context insertion points. Track cumulative usage per request. Reject or compress when approaching limits. Log budget violations.

---

## Agent Coordination (R-P186–R-P190)

### R-P186: Agent Handoff Protocol
- **Source**: OpenAI Agents (29.6K★) + CrewAI (58.8K★)
- **Rule**: Agent-to-agent handoff must include: task context, state snapshot, completion criteria, and timeout. Handoff receiver must acknowledge within 30s or task returns to sender.
- **Rationale**: OpenAI's handoff pattern and CrewAI's delegation model show that unstructured handoffs lose context and cause deadlocks.
- **Implementation**: Use `nt_core::multi_agent::handoff()` with structured payload. Include `HandoffContext { task, state, criteria, timeout_secs }`. Implement heartbeat check. Auto-return on timeout.

### R-P187: Multi-Agent Conflict Resolution
- **Source**: CrewAI (58.8K★) + AGno (42.2K★)
- **Rule**: When multiple agents claim the same resource or produce conflicting outputs, apply priority-based arbitration: explicit lock ownership wins, then highest-priority agent, then oldest claim. Never allow simultaneous writes to shared state.
- **Rationale**: CrewAI's role-based conflicts and AGno's concurrent execution demonstrate that uncontrolled parallelism causes data corruption.
- **Implementation**: Use `nt_core::multi_agent::arbitrate()` with priority queue. Implement advisory locks on shared resources. Use `nt_core::state_graph::checkpoint` for conflict rollback. Log all arbitrations.

### R-P188: Agent Capability Registration
- **Source**: superpowers (288.7K★) + AGno (42.2K★)
- **Rationale**: superpowers' skill composition and AGno's agent registry show that unregistered capabilities cause routing failures. NeoTrix's `nt_core::capability` tree must be the single source of truth.
- **Implementation**: Every agent must call `nt_core::capability::register()` at startup. Registration includes: capability name, version, resource requirements, trust level. Deregister on shutdown. Health-check registered capabilities periodically.

### R-P189: Agent Task Decomposition Standards
- **Source**: CrewAI (58.8K★) + LangGraph (41.9K★)
- **Rule**: Complex tasks must be decomposed into atomic subtasks with: clear input/output schema, dependency graph, timeout per subtask, and failure strategy (retry/fallback/abort).
- **Rationale**: CrewAI's task decomposition and LangGraph's state graphs show that monolithic tasks are fragile. Atomic subtasks enable parallel execution and partial failure recovery.
- **Implementation**: Use `nt_core::task_decomposer::decompose()` with DAG output. Each node has `InputSchema`, `OutputSchema`, `timeout_secs`, `failure_strategy`. Validate DAG is acyclic before execution.

### R-P190: Agent Observability and Cost Tracking
- **Source**: Langfuse (34.8K★) + AgentOps (5.8K★)
- **Rule**: Every agent execution must emit: trace ID, parent span, model used, tokens consumed, latency, cost estimate, and outcome. Aggregate into per-agent cost dashboards. Alert on cost anomalies.
- **Rationale**: Langfuse and AgentOps demonstrate that unmonitored agents cause cost overruns. NeoTrix's `nt_meta::otel_bridge` must enforce this.
- **Implementation**: Instrument all agent entry points with `nt_meta::otel_bridge::start_span()`. Record model, tokens, latency, cost. Export to OTel collector. Alert if single execution >10K tokens or >$0.50.

---

## Testing Discipline (R-P191–R-P195)

### R-P191: Inline Test Module Requirement
- **Source**: Rust stdlib + neotrix-core crate conventions
- **Rule**: Every new module must include a `#[cfg(test)] mod tests` block at the bottom of the file. A module without tests is considered incomplete. Test modules must contain at least one `#[test]` function that exercises the module's primary entry point.
- **Rationale**: Inline tests co-located with implementation are the first line of defense. Modules without tests accumulate regressions silently. The `neotrix-core/src/` tree demonstrates this pattern consistently.
- **Implementation**: Add `#[cfg(test)] mod tests { use super::*; #[test] fn it_works() { ... } }` to every new `.rs` file. CI must fail on modules with zero test coverage (use `cargo-tarpaulin` or `cargo-llvm-cov`). Exclude generated code and macros via `#[cfg(not(tarpaulin_include))]`.

### R-P192: Cross-Module Integration Tests
- **Source**: neotrix-core/tests/ directory
- **Rule**: Cross-module interactions must be tested in `neotrix-core/tests/` (integration test directory), not in inline `#[cfg(test)]` modules. Each integration test file covers a specific cross-module workflow. Integration tests must exercise the public API surface only.
- **Rationale**: Inline tests cannot test inter-module contracts (trait implementations across crates, message passing between agents, memory tier boundaries). Integration tests verify the system works as composed, not just in isolation.
- **Implementation**: Create `neotrix-core/tests/test_<workflow>.rs` for each cross-module flow (e.g., `test_agent_handoff.rs`, `test_memory_tier_promotion.rs`, `test_shield_pipeline.rs`). Use `#[tokio::test]` for async flows. Each test must be runnable independently with `cargo test --test test_<name>`.

### R-P193: Test Naming Convention
- **Source**: Rust convention + neotrix-core test_suites/ patterns
- **Rule**: All test functions must follow `test_<function_name>_<scenario>_<expected>` naming. Underscores separate the three components. Scenario and expected must be descriptive — avoid `test_foo_1`, `test_bar_ok`.
- **Rationale**: Consistent naming enables test discovery, filtering (`cargo test --filter`), and failure diagnosis. When a test fails, the name alone should communicate what broke.
- **Implementation**: Examples: `test_parse_config_valid_json_returns_config`, `test_parse_config_empty_string_returns_error`, `test_parse_config_duplicate_keys_uses_last`. Use `#[should_panic(expected = "...")]` for panic tests with matching message.

### R-P194: Edge Case Coverage Mandate
- **Source**: Testing best practices + neotrix-core test patterns
- **Rule**: Every test suite must cover these edge case categories: empty input, boundary values (zero, max, overflow), error paths (invalid input, missing fields), concurrent access (if applicable), and Unicode/special characters (for string processing).
- **Rationale**: Edge cases are where 80% of production bugs originate. Boundary values expose off-by-one errors. Error paths reveal missing validation. The `test_suites/` directory in neotrix-core must include explicit edge case tests.
- **Implementation**: Add a `// Edge cases` section to each test module. Cover: `""` (empty string), `0`/`MAX`/`usize::MAX` (boundaries), `Err(...)` paths, `tokio::spawn` for concurrency, `"⌘🚀"` for Unicode. Track edge case coverage percentage in CI.

### R-P195: No Untracked #[ignore] Tests
- **Source**: Rust testing best practices
- **Rule**: Never use `#[ignore]` without an accompanying GitHub issue link in the ignore attribute. Format: `#[ignore = "blocked by #<issue_number>: <brief description>"]`. CI must report count of ignored tests and fail if any `#[ignore]` lacks a tracking issue.
- **Rationale**: `#[ignore]` without tracking becomes a permanent test debt black hole. Every ignored test must have a path to being un-ignored. Without issue links, there's no accountability or timeline.
- **Implementation**: Add a CI lint step: `cargo test -- --ignored --list` parsed for missing `#<number>` patterns. Add to `.github/workflows/test.yml`. Auto-comment on linked issues when tests fail. Monthly review of all `#[ignore]` annotations.

---

## Documentation (R-P196–R-P200)

### R-P196: Module-Level Documentation
- **Source**: Rust doc conventions + neotrix-core crate standards
- **Rule**: Every `pub mod` declaration must have a `//!` doc comment explaining the module's purpose, key types, and relationship to the broader NeoTrix architecture. Private modules may use `//` comments but must still state purpose.
- **Rationale**: Module-level docs are the first thing developers see when navigating with `cargo doc` or IDE go-to-definition. Without them, the module tree becomes an opaque namespace maze.
- **Implementation**: Add `//! <purpose>` as the first line of every `.rs` file containing `pub mod`. Include relationship to parent/child modules. For complex modules, add a `//! # Architecture` section. Verify with `cargo doc --no-deps` warnings.

### R-P197: Public Item Documentation
- **Source**: Rust doc conventions
- **Rule**: All public items (`pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, `pub const`) must have `///` doc comments. Each doc comment must include a one-line summary, followed by a blank line, followed by details if needed.
- **Rationale**: Public items are the API contract. Without documentation, consumers must read source code to understand usage. This is the Rust community standard and is enforced by `#[warn(missing_docs)]`.
- **Implementation**: Enable `#![warn(missing_docs)]` at crate root. Add `///` comments to all `pub` items. Summary must be a complete sentence ending with `.`. Complex items get `/// # Errors`, `/// # Panics`, `/// # Examples` sections.

### R-P198: Doc Comment Examples for Complex APIs
- **Source**: Rust stdlib conventions
- **Rule**: Any public function or method with non-trivial parameters, error handling, or side effects must include `/// # Examples` with compilable code in the doc comment. Examples must use `assert_eq!` or `assert!` for verification.
- **Rationale**: Doc examples serve dual purpose: documentation and compile-tested regression tests. `cargo test --doc` runs them. Complex APIs without examples force users to guess usage patterns.
- **Implementation**: Add `/// ```rust` blocks to complex public APIs. Each example must: (1) construct inputs, (2) call the function, (3) assert expected output. Include `use` statements. Keep examples under 15 lines. Run `cargo test --doc` in CI.

### R-P199: Architecture Document Maintenance
- **Source**: NeoTrix governance + ARCHITECTURE-MAP-ROADMAP-V2.md
- **Rule**: When adding a new L1-L6 module (consciousness, memory, shield, world, gateway, reasoning), update `ARCHITECTURE-MAP-ROADMAP-V2.md` with: module name, layer, responsibility, key types, and dependency arrows. Changes to L1-L2 require governance review per R-P111.
- **Rationale**: Architecture documents decay without maintenance. New modules that aren't documented create blind spots for onboarding developers and governance reviewers.
- **Implementation**: Add module to the appropriate layer section. Update dependency diagram. Add `<!-- LAST_UPDATED: YYYY-MM-DD -->` comment. Submit architecture update as part of the module PR. Governance review required for L1-L2 changes.

### R-P200: Changelog Entry Requirement
- **Source**: Keep a Changelog standard + NeoTrix release process
- **Rule**: Every release must have a corresponding `CHANGELOG.md` entry following Keep a Changelog format: Added, Changed, Deprecated, Removed, Fixed, Security. Each entry must reference the PR or issue number. Entries must be written in past tense.
- **Rationale**: CHANGELOG is the user-facing release history. Missing entries cause users to miss breaking changes, new features, and security patches. It's the primary communication channel for releases.
- **Implementation**: Add entry to `CHANGELOG.md` under `## [Unreleased]`. Move to version section on release. Format: `- Description of change (#<PR_NUMBER>)`. Use Added for new features, Fixed for bugs, Security for patches. CI must verify CHANGELOG is modified when version bumps.

---

## Performance (R-P201–R-P205)

### R-P201: Hot Path Allocation Discipline
- **Source**: Rust performance best practices + neotrix-core hot paths
- **Rule**: Code paths that execute in latency-sensitive contexts (agent loops, memory tier transitions, shield pipeline stages) must not perform heap allocations. Use stack buffers, `SmallVec`, or pre-allocated arenas. Profile to identify hot paths before enforcing.
- **Rationale**: Heap allocations in hot paths cause unpredictable latency spikes. NeoTrix's agent coordination and memory tier promotion are latency-sensitive — a single `Vec::push` in a tight loop can add 100μs of GC-pause-equivalent overhead.
- **Implementation**: Mark hot paths with `#[inline]` or `#[inline(always)]`. Use `SmallVec<[T; N]>` for bounded collections. Pre-allocate with `Vec::with_capacity()`. Run `cargo bench` before and after changes. Alert on >5% regression in critical benchmarks.

### R-P202: Function Signature Prefer &str
- **Source**: Rust API guidelines (C-STR)
- **Rule**: Function parameters that accept string data must use `&str` instead of `&String` whenever the function only reads the string. `String` parameters are allowed only when the function takes ownership (e.g., builder patterns, `Into<String>`).
- **Rationale**: `&str` accepts both `&String` and `&str` via deref coercion. `&String` forces callers to dereference. The Rust API guidelines (C-STR) mandate this for ergonomic and performance reasons.
- **Implementation**: Lint with `clippy::ptr_arg`. Change `fn foo(s: &String)` to `fn foo(s: &str)`. For owned strings, use `impl Into<String>` or `String` directly. Apply to all public APIs first, then internal functions.

### R-P203: Iterator Preference Over Manual Loops
- **Source**: Rust idioms + clippy::manual_flatten
- **Rule**: Prefer iterator chains (`map`, `filter`, `fold`, `collect`) over manual `for` loops when the intent is transformation or aggregation. Manual loops are acceptable for early-exit, complex mutation, or when iterator chains would be less readable.
- **Rationale**: Iterator chains are zero-cost abstractions that enable compiler optimizations (inlining, vectorization). They express intent more clearly than manual loops. Clippy lint `clippy::manual_flatten` catches unnecessary nesting.
- **Implementation**: Use `clippy::manual_flatten`, `clippy::needless_range_loop`, `clippy::iter_next_slice` lints. Convert `for i in 0..vec.len() { ... }` to `for item in &vec { ... }`. Keep manual loops for `break`/`continue` with complex conditions. Verify no performance regression.

### R-P204: Profile Before Optimizing
- **Source**: Software engineering principles + NeoTrix governance
- **Rule**: Never optimize code without first measuring its current performance. Profiling data (flamegraph, benchmark numbers) must accompany any PR that claims performance improvement. Premature optimization without measurement is prohibited.
- **Rationale**: Unmeasured optimization wastes developer time and can introduce bugs. The "97% of the time" rule applies: profile first, find the actual bottleneck, then optimize that specific code path.
- **Implementation**: Run `cargo flamegraph` or `perf record` on the target workload. Capture baseline metrics. Make the change. Capture post-change metrics. Include before/after in PR description. Use `criterion` for microbenchmarks. No optimization PR without data.

### R-P205: Critical Path Benchmarking
- **Source**: Criterion.rs + neotrix-core/benches/
- **Rule**: All critical code paths (agent task execution, memory tier promotion, shield pipeline, gateway routing) must have criterion benchmarks in `benches/`. Benchmarks must run in CI on every PR that touches the benchmarked module. Regression threshold: 5%.
- **Rationale**: Benchmarks catch performance regressions that unit tests miss. Criterion provides statistical rigor (outlier detection, regression analysis). Without CI integration, benchmarks rot into documentation.
- **Implementation**: Create `benches/bench_<module>.rs` for each critical path. Use `criterion_group!` and `Criterion::benchmark_group`. Add to CI with `cargo bench --bench bench_<module> -- --baseline`. Fail CI on >5% regression. Update baselines monthly.

---

## Security & Operations (R-P206–R-P211)

### R-P206: Secrets Management
- **Source**: OWASP Top 10 A07 + NeoTrix shield
- **Rule**: No secrets (API keys, tokens, passwords, certificates) may appear in source code, configuration files committed to git, or log output. All secrets must be injected via environment variables or a secrets manager (e.g., `nt_shield::secrets`). `.env` files must be in `.gitignore`.
- **Rationale**: Source code repositories are frequently leaked or shared. Hardcoded secrets in committed code are the #1 cause of credential exposure. Environment variable injection is the industry standard.
- **Implementation**: Add `nt_shield::secrets::validate_no_secrets()` to pre-commit hooks. Scan with trufflehog (R-P162). Use `std::env::var()` or `nt_shield::secrets::get()` at runtime. Rotate any detected secrets immediately. Add `.env` to `.gitignore`.

### R-P207: Public API Input Validation
- **Source**: OWASP ASVS V5 + nt_shield
- **Rule**: Every public API endpoint, function, or CLI command must validate all inputs before processing. Validation must check: type correctness, range bounds, length limits, format patterns, and semantic validity. Invalid inputs must return structured error responses, not panic.
- **Rationale**: Input validation is the first defense line. ASVS V5 mandates validation at trust boundaries. Unvalidated input is the root cause of injection attacks, buffer overflows, and logic errors.
- **Implementation**: Add `nt_shield::input_validation::validate()` calls at public API entry points. Use `thiserror` for structured error types. Validate before any processing. Return `Result<T, ValidationError>` with descriptive error messages. Never use `unwrap()` on user input.

### R-P208: External API Rate Limiting
- **Source**: NeoTrix stealth net + OSINT best practices
- **Rule**: All external API calls (LLM providers, OSINT targets, third-party services) must be rate-limited. Implement per-service rate limiters with configurable requests/second, burst size, and queue depth. Use token bucket or sliding window algorithms.
- **Rationale**: Uncontrolled external calls exhaust quotas, trigger bans, and cause cascading failures. Rate limiting preserves service access and prevents cost overruns. NeoTrix's OSINT crawling (R-P176) already demonstrates this pattern.
- **Implementation**: Use `nt_shield_stealth_net::rate_limiter` or `governor` crate. Configure per-service limits: LLM (10 req/s), OSINT (1 req/s per target), general (5 req/s). Queue overflow returns `RateLimitError`. Log rate limit hits for capacity planning.

### R-P209: Graceful Degradation
- **Source**: Distributed systems principles + NeoTrix resilience
- **Rule**: When external services fail, NeoTrix must degrade gracefully rather than crash. Implement fallback strategies: cached responses, reduced functionality, or user notification. Circuit breakers must open after 3 consecutive failures and half-open after 30s.
- **Rationale**: External services (LLM APIs, OSINT sources, proxy pools) are unreliable. Hard failures cascade. Graceful degradation preserves partial functionality and user trust.
- **Implementation**: Add circuit breaker to all external HTTP clients via `nt_shield::circuit_breaker`. Implement fallback: LLM → cached response, OSINT → last-known-good, proxy → direct connection. Log degradation events. Alert on >5min sustained degradation. Auto-recover when service returns healthy.

### R-P210: Structured Logging with Tracing
- **Source**: tracing crate + NeoTrix observability (R-P190)
- **Rule**: All operations must emit structured logs using the `tracing` crate with spans for operation context. Log levels: ERROR for failures, WARN for degraded state, INFO for milestones, DEBUG for diagnostics, TRACE for granular flow. Never use `println!` or `eprintln!` in library code.
- **Rationale**: Structured logs enable filtering, aggregation, and correlation. `tracing` spans provide hierarchical context (which agent → which task → which subtask). Unstructured `println!` output is unsearchable and unfilterable.
- **Implementation**: Wrap operations in `tracing::info_span!("operation_name")`. Use `tracing::instrument` for automatic span creation. Include structured fields (not string interpolation). Configure `tracing_subscriber` with JSON output. Route to `nt_meta::otel_bridge` for export (R-P190).

### R-P211: Health Check Endpoints
- **Source**: SRE practices + NeoTrix services
- **Rule**: Every long-running service (gateway, multi-agent coordinator, consciousness core) must expose a health check endpoint at `GET /healthz` (liveness) and `GET /readyz` (readiness). Liveness checks verify the process is alive. Readiness checks verify dependencies are connected.
- **Rationale**: Health checks enable orchestration systems (Docker, Kubernetes) to manage service lifecycle. Without them, failed services continue receiving traffic. Liveness vs. readiness distinction prevents restart loops during dependency outages.
- **Implementation**: Add `GET /healthz` returning `200 OK` if process is alive. Add `GET /readyz` returning `200 OK` if all dependencies (database, LLM API, proxy pool) are connected. Use `nt_gateway::health` module. Expose on management port (default 9090). Include in Docker healthcheck config.

---

## 七、文档管理规则 (R-P212~R-P220)

### R-P212: 文档标准地图
- **Rule**: 所有文档操作必须遵循 `DOCUMENTATION-MAP.md` 标准
- **Rationale**: 防止文档散落，保持项目结构清晰
- **Implementation**: 新会话开始时读取 `DOCUMENTATION-MAP.md`，按标准选择存放位置

### R-P213: 禁止私自创建文件
- **Rule**: 禁止在标准位置之外创建 md/yml/toml 文件
- **Rationale**: 无序创建导致文档碎片化，难以维护
- **Implementation**: 创建文件前检查是否在允许位置，不在则拒绝或移动到正确位置

### R-P214: 任务文件统一
- **Rule**: 所有任务必须记录在根目录 `TODO.md` + `TODO.yml`
- **Rationale**: 多会话协作需要单一任务源
- **Implementation**: 禁止创建 `TODO-*.md`、`TASKS-*.md` 等变体文件

### R-P215: 方案文件日期前缀
- **Rule**: `docs/plans/` 下的方案文件必须用 `YYYY-MM-DD-{topic}.md` 格式
- **Rationale**: 按时间排序，便于追溯
- **Implementation**: 无日期前缀的方案文件拒绝合并

### R-P216: 技能文件规范
- **Rule**: `skills/` 下每个技能目录必须有 `SKILL.md`，不超过 200 行
- **Rationale**: 统一技能定义格式，便于 agent 加载
- **Implementation**: 超过 200 行的 SKILL.md 必须拆分

### R-P217: API 文档自动生成
- **Rule**: `docs/api/` 文档从代码注释自动生成，不手写
- **Rationale**: 保证文档与代码同步
- **Implementation**: 使用 `cargo doc` 或 mdbook 生成

### R-P218: 研究笔记归档
- **Rule**: 研究笔记、分析报告完成后删除或合并到 `docs/plans/`
- **Rationale**: 避免 `neotrix-core/docs/` 等目录堆积临时文件
- **Implementation**: 超过 30 天的研究笔记自动标记为过期

### R-P219: 文档长度限制
- **Rule**: 单个 md 文件不超过 500 行
- **Rationale**: 过长文档难以阅读和维护
- **Implementation**: 超过 500 行必须拆分为多个文件

### R-P220: 运行时数据隔离
- **Rule**: `.neotrix/` 目录由运行时自动管理，禁止手动创建文件
- **Rationale**: 防止人工操作破坏运行时状态
- **Implementation**: 手动创建的文件在下次运行时可能被覆盖

---

> **Governance**: These rules are enforced by `nt_governance::policy_engine`. Violations block execution and log to `nt_shield::audit_log`. Rule updates require R-P111 architecture review.
> **Version**: 1.2.0 (2026-09-20)
> **Next review**: When R-P221+ rules are absorbed.
