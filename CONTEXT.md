# NeoTrix — Shared Language (Ubiquitous Language)

This document defines the precise meaning of domain terms used across NeoTrix. Every agent session loads this as a prefix, so terms are consistent across sessions.

## Core Domain

| Term | Definition | Avoid |
|------|-----------|-------|
| **NeoTrix** | AI-native developer toolkit with self-evolving reasoning, VSA HyperCube knowledge representation, and GWT attention routing. The project name. | "the system", "the framework" |
| **ConsciousnessTree** | The 11-branch meta-cognition module that runs a 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Tracks cross-domain health, module maturity, and self-evolution velocity. | "the tree", "consciousness module" |
| **E8 Hexagram** | 64-element hexagonal grid used as the core reasoning engine. Each hexagram is a 6-line yijing-style symbol representing an architectural or reasoning state. | "the hex engine", "E8" (when context is ambiguous) |
| **GWT** | Global Workspace Theory — the attention routing mechanism. Broadcasts salient information across specialist modules, with resonance-based routing. | "attention system", "the workspace" |
| **VSA HyperCube** | Vector Symbolic Architecture-based knowledge representation. Maps concepts to high-dimensional vectors, enabling associative recall and analogical reasoning. | "the cube", "hypercube" |
| **SEAL Pipeline** | Self-Evolving Architecture Loop — the pipeline that runs exploration, distillation, self-test, and absorption cycles. Stages defined by `make_stage!` macro. | "the pipeline", "evolution loop" |
| **KB** | Knowledge Base — SQLite-backed persistent store. Shared state layer for all 7 domains. Contains nodes (entities), edges (relations), embeddings, and BM25 index. | "the database", "storage" |

## Faction System (7 Domains)

| Term | Definition | Avoid |
|------|-----------|-------|
| **NT-CORE** | Foundation domain: E8, GWT, HyperCube, Self module. "E8引导者" — pure logic and consciousness. | "core module" |
| **NT-MIND** | Self-evolution domain: SEAL pipeline, distillation, skill crystallization. "进化工匠". | "mind module" |
| **NT-MEMORY** | Knowledge domain: SQLite KB, FTS5 search, embeddings, versioning, caching. "知识守护者". | "memory module" |
| **NT-WORLD** | Perception domain: UnifiedCrawler, fetchers, parsers, classifiers, content extraction. "虚空探索者". | "world module", "crawler" |
| **NT-ACT** | Action domain: MCP tools, social media, code, autonomy, orchestration. "行动执行者". | "act module" |
| **NT-IO** | Interface domain: LLM providers, CLI, web server, ACP, LSP. "界面使徒". | "io module" |
| **NT-SHIELD** | Security domain: stealth net, proxy pool, Tor client, fingerprint management, audit. "影卫". | "shield module" |

### ConsciousnessTree Branches (11 Branches)

NT-META (元吸收者), NT-REPAIR (自愈工程师), NT-GOVERNANCE (架构仲裁者), NT-NEXUS (枢纽) — plus the 7 core domains above.

### Skill Domain 收编 (UCN 命名统一)

外部 `skills` 域技能收编进 NT-* 域，映射为域内"星辰"。单一事实源在 KB `domain_nt_*` namespace（写入函数 `unify_domain_mapping`）。映射为 1:N（一个域可有多颗星辰）。

| skills 域源 | → NT-* 域 | 星辰 | KB namespace |
|---|---|---|---|
| `rev/officer` | NT-SHIELD | Rev-明 | `domain_nt_shield` |
| `dev/implementer` | NT-ACT | Dev-匠 | `domain_nt_act` |
| `des/architect` | NT-CORE | Des-观 | `domain_nt_core` |
| `res/scholar` + `methodology/researcher` | NT-MIND | Res-深 | `domain_nt_mind` |
| `experience-tree` | NT-MEMORY | Exp-藏 | `domain_nt_memory` |
| `nexus/weaver` | NT-MEMORY | Nexus-梭 | `domain_nt_memory` |
| `meta/coordinator` | NT-META | Meta-镜 | `domain_nt_meta` |
| `sg/diagnostician` | NT-META | SG-诊 | `domain_nt_meta` |
| `repair/healer` | NT-REPAIR | Repair-医 | `domain_nt_repair` |
| `gov/steward` | NT-GOVERNANCE | Gov-衡 | `domain_nt_governance` |
| `mil/officer` | NT-SCOUT | Search-觅 | `domain_nt_scout` |
| `ed/tutor` | NT-IO | Edu-灯 | `domain_nt_io` |

L3 厂商技能（36+）为只读能力分支，不进收编映射表。

## Architecture Patterns

| Term | Definition | Avoid |
|------|-----------|-------|
| **Skill Tree** | Per-domain capability progression with 3 node tiers: Small Passive (微节点), Notable Passive (显节点), Keystone (基石). POE-inspired. | "skill tree", "passive tree" |
| **Rune Socketing** | Per-module configuration with 5 rune colors: Crimson (data), Indigo (transform), Obsidian (cache), Golden (error), Alabaster (monitor). Runeword = emergent effect from rune combination. | "plugin system", "config slots" |
| **Constellations (C0-C6)** | Module maturity ladder: C0=compiles, C1=unit tests, C2=integration tests, C3=benchmarked, C4=integrated into pipeline, C5=self-healing. Genshin-inspired. | "maturity levels" |
| **Dual Specialization** | Weapon Set I/II switching per context. AttentionManager routes between CORE+WORLD (acquisition) and CORE+MIND (evolution) modes. POE-inspired. | "modes", "profiles" |
| **The Spice Must Flow** | Data pipeline axiom: every module must have clear input→transform→output with no disconnects. Dune-inspired. | "data flow" |
| **Dark Forest** | Module survival axiom: every module must compile + test + connect (have consumers) or be deleted. Three-Body-inspired. | "cleanup rule" |

## Audit Dimensions (D1-D50)

| Range | Name | Purpose |
|-------|------|---------|
| D1-D12 | Standard Audit | Build, modules, layers, safety, architecture, config, tests, errors, supply chain, security, deps, docs |
| D13-D16 | Meta-Cognition | Consciousness architecture, topology evolution, health chain, self-deception |
| D17-D20 | Architecture Base | SelfTest 3D coverage, production wiring, visibility chain, absorption progress |
| D21-D25 | Meta-Cognition II | External observation, self-healing loop, visibility chain, build poisoning, decoupling |
| D26-D30 | Production Readiness | Self-healing maturity, retry cap, ratio trend, throwaway instances, EventBus grounding |
| D31-D36 | Structural | Two-layer EventBus, reentrant lock, persistent fields, phase deps, threshold gating, inline SelfTest |
| D37-D40 | Meta-Evolution | Constitution compliance, meta-pattern absorption, cross-dimension synthesis, evolution velocity |
| D41-D50 | Meta-Review | Pipeline continuity, tool grounding, behavior production gate, architecture weight, monotonicity gate, review discipline, architecture memory, cross-domain energy flow, dependency dead weight, meta-audit |

## SelfTest Tiers

| Term | Definition |
|------|-----------|
| **T1 Existence** | `impl SelfTest for TypeName` exists in the file |
| **T2 Registration** | Registered in run.rs + pipeline.rs SelfTest registries |
| **T3 Production Wiring** | The actual detection function (`evaluate()`, `check()`, `audit()`, `scan()`) is called by non-test code, and its output can influence behavior |

## Review Methodology

| Term | Definition |
|------|-----------|
| **Fractal Review Loops** | 5-level review chain: Artifact → Task → Session → Epic → PR. Each level inspects the level below. |
| **Convergence Check** | The `converge_check()` function that audits ghost modules, orphan files, and persistence verification. Runs as SEAL Phase-0. |
| **Evidence-First** | Every finding must be traced to specific file:line, command output, or build result. No hallucinated findings. |
| **Dual Verification** | `cargo check` + `cargo test` must both pass. Each has independent caches. Clean build after structural changes. |

## Flagged Ambiguities

| Term | Resolution |
|------|-----------|
| "crawler" | Use **NT-WORLD** (domain) or **UnifiedCrawler** (specific module). "Crawler" alone is ambiguous. |
| "pipeline" | Use **SEAL pipeline** (evolution), **KB pipeline** (search/retrieval), or **crawl pipeline** (data acquisition). |
| "embedding" | Use **KB embedding** (vector storage) or **VSA embedding** (symbolic representation). Different systems. |
| "consciousness" | Use **ConsciousnessTree** (6-stage meta-cognition loop), **GWT** (attention routing), or **Phi** (IIT integration score). |
| "self-test" | Use **SelfTest** (trait + registry for detection modules) or **converge_check** (architecture self-audit). |
| "audit" | Use **D1-D50** (specific dimension) or **rev-officer** (full health check) or **self_audit** (module-level scan). |
| "module" | Use **Rust module** (`.rs` file + `mod` declaration) or **domain module** (`nt_*` subsystem) or **detection module** (implements SelfTest). |

## Absorbed Terminology (2026-08-16, 22-source batch)

| Term | Definition | Avoid |
|------|-----------|-------|
| **PTC** | Programmatic Tool Calling — typed-stub tool invocation: JSON tool schema exposed as Python signature stubs, chained + parallel calls in a single agent turn. Lives in `nt_agent_mcp_gateway` (P1). | "tool stubs", "code tool calling" |
| **Egress Policy** | Per-sandbox outbound network trust boundary: allow/deny host/port rules, deny-wins, `default_allow` fallback. `*.suffix` matches subdomains only, never bare apex. Lives in `nt_shield_sandbox` (P2). | "network policy", "firewall rules" |
| **VoI** | Value-of-Information — expected KL (prior‖posterior) used to select the next experiment in Bayesian experiment design. Lives in `nt_core_hcube::bayesian_experiment` (P3). | "information gain" (when referring to VoI specifically) |
| **M-open check** | Predictive adequacy check in Bayesian experiment design: when posterior concentrates on < threshold of hypotheses, the hypothesis space is expanded. (P3) | "expansion trigger" |
| **Disclosure Ladder** | Anchor-then-promote: first request anchors on a Minimal tool budget, promotes to Standard once the session is durable. Implemented as `AnchorPromote` in `nt_mind_skill_engine` (P4). | "tool budget", "context budget" |
| **Ordered Backend Router** | Single search interface routing across backends (DDG→Wikipedia) with ordered fallback, zero external API cost. Lives in `nt_world_search` (P16). | "search router", "fallback chain" |
