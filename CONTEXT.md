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
| **total_calls ascending** | In gateway provider selection, lower total_calls takes priority for rotation, ensuring even distribution across available providers. Sorts ascending (least-used provider first). Related to NT-ACT load balancing. | "total_calls descending" |
| **Egress Privacy Guard** | Outbound LLM request filter preventing NeoTrix's own source code / KB / conversation / local paths from leaking to external models. Single logic source in `nt_core_llm::egress_privacy_guard` (neotrix layer delegates to it). Trust tiers: `Trusted` (local/Ollama — passthrough after secret scrub), `Contracted` (paid cloud — always redact internal fingerprints + absolute paths + secrets across all fields: messages/tools/structured_output/provider_params/model), `Untrusted` (free/proxy — fail-closed block on internal fingerprint). Secrets + absolute paths always scrubbed regardless of trust. | "privacy filter", "the guard" |

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
| **NT-PHYSICAL** | Physical embodiment domain: sensors, motors, safety kernel, power management, body schema. "具身骨架". | "physical module", "hardware module" |
| **NT-FEEL** | Emotional domain: EmotionEngine (unified), regulation, expression, social emotion. "情感中枢". | "emotion module", "feel module" |

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
| **Six-Layer Architecture** | 6-layer consciousness-embodiment architecture: L1 Action (行动层: nt_act+nt_io+nt_memory), L2 Perception (感知层: nt_world+nt_sense), L3 Embodiment (具身层: nt_physical+nt_shield+nt_feel), L4 Emotion (情感层: nt_feel core), L5 Cognition (认知层: nt_core+nt_mind), L6 Meta-Cognition (元认知层: nt_meta+nt_repair+nt_nexus). Each layer has `traits.rs` interface contracts. | "3-tier", "three layer", "Consciousness-Embodiment-Capability" |
| **Consciousness-Embodiment-Capability** | Legacy 3-layer architecture (now expanded to 6 layers). L5 Consciousness (涌现觉知: E8+GWT+Emotion+SEAL), L3 Embodiment (具身骨架: sensors+motors+safety+power), L1 Capability Network (能力网: tools+perception+memory+action). | "3-tier", "three layer" (use Six-Layer Architecture instead) |
| **Skill Tree** | Per-domain capability progression with 3 node tiers: Small Passive (微节点), Notable Passive (显节点), Keystone (基石). POE-inspired. | "skill tree", "passive tree" |
| **Rune Socketing** | Per-module configuration with 5 rune colors: Crimson (data), Indigo (transform), Obsidian (cache), Golden (error), Alabaster (monitor). Runeword = emergent effect from rune combination. | "plugin system", "config slots" |
| **Constellations (C0-C6)** | Module maturity ladder: C0=compiles, C1=unit tests, C2=integration tests, C3=benchmarked, C4=integrated into pipeline, C5=self-healing. Genshin-inspired. | "maturity levels" |
| **Dual Specialization** | Weapon Set I/II switching per context. AttentionManager routes between CORE+WORLD (acquisition) and CORE+MIND (evolution) modes. POE-inspired. | "modes", "profiles" |
| **The Spice Must Flow** | Data pipeline axiom: every module must have clear input→transform→output with no disconnects. Dune-inspired. | "data flow" |
| **Dark Forest** | Module survival axiom: every module must compile + test + connect (have consumers) or be deleted. Three-Body-inspired. | "cleanup rule" |
| **Heartbeat Aggregator** | Unified system health signal collector. Aggregates compilation/test/KB/eventbus/module health into single `SystemHealthSnapshot` with time-decay. Single fact source for GWT attention modulation. | "health check", "health monitor" |
| **EmotionLabel** | Unified emotion enum (11 variants: Neutral/Joy/Sadness/Anger/Fear/Trust/Disgust/Surprise/Anticipation/Confused/Thinking). Single fact source for all emotion expression. Replaces per-module Emotion enums. | "Emotion enum", "emotion type" |
| **PerceptionBridge** | Attention-gated bridge connecting SensoryIntegrationHub (L2 perception) with SelectiveState (L5 consciousness). Uses `awareness_score()` to filter sensory events based on consciousness level. Single fact source for attention-gated perception flow. | "sensory filter", "attention gate" |
| **CapabilityBridge** | Bridge connecting evolution view (CapabilityTree) with runtime view (CapabilityRegistry). Maps tree node IDs to runtime capability IDs, enabling cross-view queries and auto-discovery of relationships. | "registry bridge", "capability mapper" |

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

## Absorbed Terminology (2026-09-02, 动态漫技能吸收)

| Term | Definition | Avoid |
|------|-----------|-------|
| **DynamicParams** | 动态参数元数据结构：统一速度 (speed)、幅度 (amplitude)、频率 (frequency) 三要素的物理单位和约束规范。实现于 `nt_core_self::dynamic_params`。 | "动态参数" |
| **ScalingRating** | 爽点动态等级对应情感强度：Micro(微)/Medium(中)/Macro(强) 三级，映射到 EmotionLabel 11 variants。用于动态漫动态等级分类。 | "动态等级", "强度等级" |
| **RhythmRecalculator** | 节奏重算模块：基于功率律 (ratio^0.8) 的节段时长重算，支持默认/快节奏/10集系列布局。实现于 `seal::rhythm_recalculator`。 | "节奏重算", "时长分配" |
| **SegmentType** | 分段类型枚举：Setup(铺垫)/Conflict(冲突)/Climax(爽点)/Transition(转折)。用于动态漫剧情节奏设计。 | "节段类型" |
| **CharacterInteractionGraph** | 角色互动谱系图：管理角色、互动关系、成长弧线的结构化网络。实现于 `nt_core_self::character_interaction`。 | "角色关系图" |
| **BlankSpaceChecker** | 留白量化检查器：检查爽点后留白是否符合节奏呼吸感要求（3-5秒无台词+微动态）。实现于 `seal::blank_space_checker`。 | "留白检查" |
| **TransitionGradientAdvisor** | 转场梯度建议器：根据节奏快慢自动推荐转场类型（慢→渐变，快→硬切）。实现于 `seal::transition_gradient`。 | "转场建议" |
| **CrossModuleAudit** | 跨模块一致性检查器：检查动态等级与情绪拐点、节奏节拍的三线一致。实现于 `nt_meta::cross_module_audit`。 | "一致性检查" |
| **TemplateTagRegistry** | 模板复用标签系统：管理动态漫技能模板的标签、分类、复用关系，支持跨模块一致性检查和模板检索。实现于 `nt_meta::template_tag_registry`。 | "模板标签" |
| **AudioSyncPattern** | 动态-音效同步模式：定义动态效果和音效的同步模式，支持动态漫制作中的音画同步。实现于 `nt_physical::audio_sync_library`。 | "音效同步" |
| **QuickStartGuide** | 快速入门向导：提供动态漫技能的快速入门指引，帮助用户快速理解和使用 NeoTrix 动态漫能力。实现于 `nt_io::quick_start_guide`。 | "入门指南" |
| **AssetRegistry** | 主体库管理：管理角色、场景、道具、特效、配音等标准化资产，支持资产沉淀、复用、版本控制。实现于 `nt_world::asset_registry`。 | "资产库" |
| **BatchProductionManager** | 批量生产工作流：管理多任务并行、进度追踪、断点续传，支持工业化生产流水线。实现于 `nt_act::production_pipeline`。 | "批量生产" |
| **QualityGate** | 质量控制审核：实现 AI 初检 → 人工复审 → 平台终审的三级审核流程，支持自动化质量评估。实现于 `nt_meta::quality_gate`。 | "质量审核" |
| **ConsistencyAdapter** | 一致性控制接口：定义角色一致性控制的统一接口，支持 LoRA、IP-Adapter 等技术对接。实现于 `nt_io::consistency_adapter`。 | "一致性接口" |
| **ReferenceVideoMode** | 参考生视频模式：核心生产模式，实现"角色资产→场景→参考生+主体库→选片配音剪辑"四步闭环。实现于 `nt_io::reference_video_mode`。 | "参考生模式" |
| **FaceConsistencyManager** | 角色一致性增强：实现 ADetailer/FaceDetailer 自动补脸、Regional Prompting 多角色分区，增强角色跨镜头一致性。实现于 `nt_core::face_consistency`。 | "一致性增强" |
| **VideoTemporalStabilizer** | 视频时序稳定性：实现帧间色彩对齐、时序防抖、二次元超分修复，提升视频生成的时序稳定性。实现于 `nt_physical::video_temporal_stabilizer`。 | "时序稳定性" |
| **StoryboardExtractor** | 分镜智能拆解：实现 LLM 剧本→分镜自动拆解、镜头运动规划，将剧本转换为结构化分镜脚本。实现于 `nt_core::storyboard_extractor`。 | "分镜拆解" |
| **CostManager** | 成本控制：实现 Token 估算、预算管理、成本优化策略，支持 AI 漫剧生产的成本控制。实现于 `nt_act::cost_manager`。 | "成本控制" |
| **ShotContinuityChecker** | 镜头衔接：实现首尾帧链接、场景转场连续性，确保镜头之间的视觉连贯性。实现于 `nt_act::shot_continuity`。 | "镜头衔接" |
| **TaskScheduler** | 任务调度优化：实现 GPU 显存管理、批量调度优化、指数退避重试，提升 AI 漫剧生产任务的调度效率。实现于 `nt_act::task_scheduler`。 | "任务调度" |
