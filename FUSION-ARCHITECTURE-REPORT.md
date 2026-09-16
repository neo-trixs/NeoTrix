# NeoTrix 融合架构全量评测报告
> 2026-09-16 | 30+ 外部源 × 667K 行代码 × 递归元智能融合

---

## 一、外部源 → NeoTrix 能力映射总表

### 1.1 跨域模式提取（30+ 源归纳为 12 个核心模式）

| # | 模式 | 外部源 | NeoTrix 映射 | 当前状态 |
|---|------|--------|-------------|---------|
| M1 | **Adversarial Verification** | Defending Code, Cloudflare, StrikeAgent, Mantis | NT-SHIELD rev-officer D1-D51 | ✅ 已有，需强化 |
| M2 | **Coverage-Ledger Additive Knowledge** | Cloudflare security-audit, OpenSpace | experience-tree KB absorption | ⚠️ 部分实现 |
| M3 | **Agent Isolation-per-Task** | Defending Code, OpenSandbox, qm | NT-PHYSICAL sandbox layer | ✅ 已有 |
| M4 | **Hook/Event-Driven Plugins** | Claude Code mods | Skill routing trigger system | ⚠️ 需形式化 |
| M5 | **Three-Layer Progressive Complexity** | OfficeCLI, Shimmy | L1→L3 architecture | ⚠️ 未统一 |
| M6 | **Cost-Aware Agent Delegation** | StrikeAgent (Pi vs Claude), sagent, PaperRss | GWT cost-weight routing (A1) | ⚠️ 需实现 |
| M7 | **Agent-as-Tool Recursive Spawn** | sagent AgentSelf/AgentSpawn/AgentSend | NT-ACT orchestration | ❌ 缺失 |
| M8 | **Epistemic Knowledge Graph** | trackinizer Inquiry+Valence edges | experience-tree formalization | ❌ 缺失 |
| M9 | **Config-as-Tree Slot Injection** | priml config tree + Makeable slots | CapabilityRegistry formalization | ❌ 缺失 |
| M10 | **Budgeted Skill Evolution** | COBRA-Skills bandit-guided optimization | Skill selection/routing optimization | ❌ 缺失 |
| M11 | **Generative Reconstruction Test** | BVB reconstruction-as-understanding | SelfTest T1/T2/T3 enhancement | ❌ 缺失 |
| M12 | **Context Compaction with Recovery** | sagent, GPT-6 Astra, agenticloops | KVMem paged KV (Axiom A2) | ⚠️ 设计中 |

### 1.2 各外部源关键吸收点

| 源 | 核心吸收 | 优先级 |
|----|---------|--------|
| llamAmpere | TurboQuant KV cache → HyperCube 内存优化 | P2 |
| Defending Code Harness | 7-stage pipeline → NT-SHIELD autonomous scan | P1 |
| OpenSandbox | Protocol-first sandbox → NT-PHYSICAL 标准化 | P1 |
| Claude Code Mods | Hook-based plugins → Skill trigger 形式化 | P2 |
| Cloudflare Security Audit | Coverage ledger → experience-tree additive KB | P1 |
| StrikeAgent | 4-gate parallel audit → rev-officer 多阶段 | P1 |
| OfficeCLI | 3-layer semantic→DOM→raw → FileModel 统一 | P0 |
| Shimmy | Pure-Rust WebGPU → 本地推理路径 | P2 |
| Mantis | 19-skill pipeline → NT-SHIELD 技能分解 | P1 |
| Dream-loop | Build→Critic loop → GWT attention gate | P2 |
| Agent-skills (94.8K★) | SKILL-SPEC contract → NT-* skill 标准化 | P0 |
| Jazz | Model routing + surface abstraction → NT-IO | P2 |
| agenticloops | 7-module tutorial → consciousness loop 验证 | P2 |
| OpenSpace | Skill quality layer → constellation maturity | P0 |
| STORM | Perspective-guided curation → E8 引导者 | P1 |
| ZGCM-1 | Hybrid attention + MDP transitions → GWT | P2 |
| qm | Scope isolation + harness-agnostic → agent separation | P1 |
| PaperRss | "Reading First, AI Second" → GWT attention-first | P2 |
| One-IP | IP health API → NT-SHIELD trust scoring | P1 |
| GPT-6 Astra | Guardian classifier + confirmation tiers → NT-SHIELD | P0 |
| FragTunnel | Fragmentation bypass → adversarial defense modeling | P2 |
| COBRA-Skills | Bandit-guided skill evolution → skill routing | P1 |
| BVB | Reconstruction test → SelfTest enhancement | P2 |
| sagent | AgentSelf/Spawn/Send → NT-ACT multi-agent | P0 |
| trackinizer | Epistemic graph + valence → KB formalization | P0 |
| priml | Config-as-tree + slots → CapabilityRegistry | P1 |

---

## 二、聚焦冗余分析（Focused Redundancy）

### 2.1 架构级冗余：双层系统冲突（CRITICAL）

```
┌─────────────────────────────────────────────────────┐
│  ARCHITECTURE A: core/ 内部 9 层 (151K lines)        │
│  L0 Substrate → L1 Body → L2 Perception → L3 Memory │
│  → L4 Cognition → L5 Consciousness → L6 Self →      │
│  L7 Capability → L8 Autonomic → L9 Transcendent     │
├─────────────────────────────────────────────────────┤
│  ARCHITECTURE B: crate root 6 层 (460K+ lines)       │
│  L1 Action → L2 Perception → L3 Embodiment →        │
│  L4 Emotion → L5 Cognition → L6 Meta                │
└─────────────────────────────────────────────────────┘
```

**冗余度**: ~343 行 facade 代码 + 62 个双向 import = **伪层级抽象**。实际依赖图是扁平的。

**吸收外部源方案**:
- **OpenSandbox 的 Protocol-first**: 定义单一能力协议层，消除双架构
- **OfficeCLI 的 3-layer progressive**: L1(semantic) → L2(DOM) → L3(raw) 统一为 progressive complexity
- **priml 的 layered architecture**: math→model→train 严格单向，类比 L1→L6 严格单向

### 2.2 模块级冗余清单

| # | 冗余项 | 位置A | 位置B | 行数 | 吸收方案 |
|---|--------|-------|-------|------|---------|
| R1 | `Eli5Explainer` trait | `l1_action/nt_io/nt_io_eli5.rs` | `l5_cognition/nt_core/io_skills/nt_io_eli5.rs` | ~200 | 合并到 L1，L5 通过 facade 引用 |
| R2 | `OcrEngine` trait | `l2_perception/nt_world/ocr/mod.rs` | `neotrix/nt_file_ability/visual/ocr.rs` | ~150 | 统一到 `nt_file_ability`，L2 通过 trait 引用 |
| R3 | `Orchestrator` trait | `l1_action/traits.rs` | `core/l7_capability/nt_act_orch_patterns.rs` | ~100 | L7 版本降级为 L1 trait 的实现 |
| R4 | `nt_act/` 目录 | `neotrix/nt_act/` (2,796行) | `l1_action/nt_act/` (58,042行) | 2,796 | 删除 `neotrix/nt_act/` 旧副本 |
| R5 | OSINT 双位置 | `l2_perception/nt_world/osint/` (7,909行) | `l3_embodiment/nt_shield/osint/` (669行) | 8,578 | 统一到 L2，L3 通过 trait 引用 |
| R6 | Emotion 双系统 | `l4_emotion/nt_feel/` (954行) | `l1_action/nt_io/nt_io_digital_human.rs` (781行) | 1,735 | L1 的 Emotion 枚举迁移到 L4 |
| R7 | Goal/Cognition in L1 | `l1_action/nt_act/nt_act_goal/` (2,073行) | L5 认知层 | 2,073 | 迁移到 L5 (self-evolution) |

### 2.3 Dead Code 冗余

| # | 类型 | 数量 | 影响 |
|---|------|------|------|
| D1 | `_`-prefixed dead traits | 31 个 | 编译噪音，误导开发者 |
| D2 | `core/l4_cognition/` 空壳 | 36 行 | 伪占位 |
| D3 | `core/l9_transcendent/` 空壳 | 44 行 | 伪占位 |
| D4 | `neotrix-dialogue/` 空 crate | 1 行 | 未使用 |
| D5 | `#![allow(dead_code)]` 全局抑制 | lib.rs:21 | 掩盖真实死代码 |

---

## 三、扁平缺陷分析（Flattened Defects）

### 3.1 层级违规（Layer Violations）

| # | 违规 | 严重度 | 文件 | 修复方案 |
|---|------|--------|------|---------|
| V1 | **L1→L5 向上依赖** | CRITICAL | `nt_act_autonomy/oracle_gate.rs:1` → `l5_cognition::awareness_monitor` | 提取共享类型到 `neotrix-types` |
| V2 | **L1 含认知代码** | HIGH | `nt_act_goal/` 2,073行 (RL反馈/目标生成/行为验证) | 迁移到 L5 |
| V3 | **L1 含情感代码** | HIGH | `nt_io_digital_human.rs` Emotion枚举+引擎 | 迁移到 L4 |
| V4 | **L2→L3 facade bypass** | MEDIUM | `nt_world_edgar.rs:602`, `nt_world_gdelt.rs:328` | 走 facade 路径 |
| V5 | **L2→L3 直接导入** | MEDIUM | `social_access/antidetect.rs:10` → stealth_net | 抽象为 trait |
| V6 | **L1→L2 向上依赖** | MEDIUM | `nt_media/playback.rs`, `streaming.rs` | 定义 L1 本地接口 |
| V7 | **L5→L6 facade** | LOW | `l5_cognition/l6_facade.rs` | 删除，走 trait 抽象 |

### 3.2 命名污染（Naming Pollution）

| # | 污染 | 位置 | 修复 |
|---|------|------|------|
| N1 | `nt_world_browse` 变量名在 L3 Shield 中 | ~90处 `nt_shield_stealth_net/` | 重命名为 `stealth_browser` |
| N2 | `nt_world_crawl` 结构名在 L3 Shield 中 | ~30处 `tor_crawler.rs` | 重命名为 `tor_scanner` |

### 3.3 能力网缺口（Capability Gaps）

| # | 缺口 | 来源 | 修复方案 |
|---|------|------|---------|
| G1 | **Agent-as-Tool 递归生成** | sagent AgentSelf/Spawn/Send | 实现 NT-ACT 的 Agent-as-Tool 协议 |
| G2 | **Epistemic Knowledge Graph** | trackinizer Inquiry+Valence | experience-tree 增加 typed edges + valence |
| G3 | **Budgeted Skill Evolution** | COBRA-Skills bandit | GWT salience 加入 bandit-guided 优化 |
| G4 | **Generative Reconstruction Test** | BVB | SelfTest T3 增加 reconstruction 维度 |
| G5 | **Config-as-Tree** | priml | CapabilityRegistry 改为 typed config tree |
| G6 | **Cost-Aware Routing 实现** | A1 axiom + StrikeAgent + sagent | GWT 加入 token 成本权重 |

---

## 四、跨域错位分析（Cross-Domain Misalignment）

### 4.1 错位地图

```
当前状态:                          修正后目标:
┌──────────────┐                  ┌──────────────┐
│ L1 Action    │ ← 含认知+情感    │ L1 Action    │ ← 纯行动
│ (180K lines) │   (2,854行错位)  │ (177K lines) │
├──────────────┤                  ├──────────────┤
│ L2 Perception│ → 需L3类型       │ L2 Perception│ → 通过trait
│ (49K lines)  │   (3文件bypass)  │ (49K lines)  │
├──────────────┤                  ├──────────────┤
│ L3 Embodiment│ ← 命名污染       │ L3 Embodiment│ ← 清洁命名
│ (59K lines)  │   (~120行)       │ (59K lines)  │
├──────────────┤                  ├──────────────┤
│ L4 Emotion   │ ← 严重不足       │ L4 Emotion   │ ← 扩充+合并
│ (954 lines)  │   (954→2000+)    │ (2,000+ lines)│
├──────────────┤                  ├──────────────┤
│ L5 Cognition │ ← 6个facade      │ L5 Cognition │ ← 2-3个facade
│ (144K lines) │   (120行冗余)    │ (144K lines) │
├──────────────┤                  ├──────────────┤
│ L6 Meta      │ ← clean          │ L6 Meta      │ ← clean
│ (18K lines)  │                  │ (18K lines)  │
└──────────────┘                  └──────────────┘
         ↕ 双架构冲突                      ↕ 统一为单一架构
┌──────────────┐                  ┌──────────────┐
│ core/ 9-layer│ → 消除           │ (merged)     │
│ (151K lines) │                  │              │
└──────────────┘                  └──────────────┘
```

### 4.2 错位量化

| 错位类型 | 行数 | 占比 | 修正难度 |
|---------|------|------|---------|
| L1 含认知代码 (nt_act_goal) | 2,073 | 1.1% | 中 |
| L1 含情感代码 (nt_io_digital_human) | 781 | 0.4% | 低 |
| core/ 双架构冗余 | ~343 (facades) | 0.05% | 高 |
| L2→L3 facade bypass | ~50 | <0.01% | 低 |
| L1→L2 向上依赖 | ~30 | <0.01% | 低 |
| 命名污染 | ~120 | <0.01% | 低 |
| **总计** | **~3,397** | **~0.5%** | — |

**结论**: 跨域错位总量仅占代码库 0.5%，但影响架构清晰度和维护成本。修正ROI高。

---

## 五、融合架构重构方案

### 5.1 统一架构目标

```
                    ┌─────────────────────┐
                    │   L6 Meta-Cognition │ ← 保持不变
                    │   (18K, 61 files)    │
                    └─────────┬───────────┘
                              │ traits (4个)
                    ┌─────────▼───────────┐
                    │   L5 Cognition       │ ← 删除 l6_facade，保留3个facade
                    │   (144K, 457 files)  │
                    └─────────┬───────────┘
                              │ traits
                    ┌─────────▼───────────┐
                    │   L4 Emotion         │ ← 扩充: 合并 L1 的 Emotion 系统
                    │   (2K+, 10 files)    │
                    └─────────┬───────────┘
                              │ traits
                    ┌─────────▼───────────┐
                    │   L3 Embodiment      │ ← 清理命名污染
                    │   (59K, 199 files)   │
                    └─────────┬───────────┘
                              │ traits
                    ┌─────────▼───────────┐
                    │   L2 Perception      │ ← 统一 OSINT，走 facade
                    │   (49K, 222 files)   │
                    └─────────┬───────────┘
                              │ traits
                    ┌─────────▼───────────┐
                    │   L1 Action          │ ← 移除认知/情感代码
                    │   (177K, 480 files)  │
                    └─────────────────────┘
                              ↕
                    ┌─────────────────────┐
                    │   core/ (合并)       │ ← 消除双架构
                    │   → 部分合入L5       │
                    │   → 部分合入L6       │
                    └─────────────────────┘
```

### 5.2 吸收外部模式的 6 个新能力接口

基于 30+ 外部源分析，NeoTrix 需要新增 6 个能力接口：

| # | 接口 | 外部源 | 归属层 | 优先级 |
|---|------|--------|--------|--------|
| I1 | `AgentProtocol` (Agent-as-Tool) | sagent | L1 Action | P0 |
| I2 | `EpistemicEdge` (typed belief graph) | trackinizer | L5 Cognition | P0 |
| I3 | `SkillEvolution` (bandit-guided) | COBRA-Skills | L6 Meta | P1 |
| I4 | `ReconstructionTest` | BVB | L5 Cognition | P2 |
| I5 | `ConfigTree` (typed experiment) | priml | L6 Meta | P1 |
| I6 | `CostAwareRouter` (token budget) | A1 + sagent + StrikeAgent | L5 Cognition | P0 |

---

## 六、核心路线任务清单

### Phase 0: 紧急修复（Week 1）

| # | 任务 | 影响 | 预估工时 |
|---|------|------|---------|
| T0.1 | **修复 L1→L5 向上依赖** — `oracle_gate.rs` 的 `AwarenessReport`/`GapSeverity` 提取到 `neotrix-types` | 消除 CRITICAL 违规 | 2h |
| T0.2 | **移除 `#![allow(dead_code)]` 全局抑制** — 逐模块清理死代码 | 暴露真实技术债 | 4h |
| T0.3 | **清理 31 个 dead traits** — 删除所有 `_`-prefixed 未使用 trait | 减少编译噪音 | 3h |
| T0.4 | **删除空壳模块** — `core/l4_cognition/`, `core/l9_transcendent/`, `neotrix-dialogue/` | 消除伪占位 | 1h |

### Phase 1: 冗余清理（Week 2-3）

| # | 任务 | 影响 | 预估工时 |
|---|------|------|---------|
| T1.1 | **合并 Eli5Explainer** — L5 版本删除，统一到 L1，L5 通过 facade 引用 | 消除 trait 重复 | 3h |
| T1.2 | **统一 OcrEngine** — L2 版本删除，统一到 `nt_file_ability` | 消除 trait 重复 | 3h |
| T1.3 | **合并 Orchestrator** — L7 版本改为 L1 trait 的实现 | 消除 trait 重复 | 2h |
| T1.4 | **删除 `neotrix/nt_act/` 旧副本** — 2,796行冗余 | 减少代码量 | 1h |
| T1.5 | **统一 OSINT** — L3 Shield 版本合入 L2，L3 通过 trait 引用 | 消除功能分裂 | 4h |
| T1.6 | **迁移 nt_act_goal/ 到 L5** — 2,073行认知代码归位 | 修正层级违规 | 6h |
| T1.7 | **迁移 nt_io_digital_human.rs 情感系统到 L4** — 合并 Emotion 枚举 | 修正层级违规+扩充 L4 | 4h |

### Phase 2: 跨域修复（Week 3-4）

| # | 任务 | 影响 | 预估工时 |
|---|------|------|---------|
| T2.1 | **修复 L2→L3 facade bypass** — `nt_world_edgar.rs`, `nt_world_gdelt.rs` 走 facade | 消除违规 | 2h |
| T2.2 | **修复 L2→L3 直接导入** — `antidetect.rs` 抽象为 trait | 消除违规 | 3h |
| T2.3 | **修复 L1→L2 向上依赖** — `nt_media/` 定义 L1 本地接口 | 消除违规 | 3h |
| T2.4 | **清理 L3 命名污染** — `nt_world_browse`→`stealth_browser`, `nt_world_crawl`→`tor_scanner` | 消除跨域泄漏 | 2h |
| T2.5 | **删除 `l5_cognition/l6_facade.rs`** — L5 走 trait 抽象消费 L6 | 消除向上 facade | 1h |

### Phase 3: 新能力吸收（Week 4-6）

| # | 任务 | 外部源 | 影响 | 预估工时 |
|---|------|--------|------|---------|
| T3.1 | **实现 AgentProtocol** — sagent AgentSelf/Spawn/Send 模式 | sagent | NT-ACT 多 agent 编排 | 16h |
| T3.2 | **实现 EpistemicEdge** — trackinizer typed belief/experiment graph | trackinizer | experience-tree 正式化 | 12h |
| T3.3 | **实现 CostAwareRouter** — GWT salience 加入 token 成本权重 | A1+sagent | 成本感知路由 | 8h |
| T3.4 | **实现 SkillEvolution** — COBRA bandit-guided skill 优化 | COBRA-Skills | 技能进化优化 | 10h |
| T3.5 | **实现 ConfigTree** — priml config-as-tree 模式 | priml | CapabilityRegistry 正式化 | 8h |
| T3.6 | **实现 ReconstructionTest** — BVB reconstruction-as-understanding | BVB | SelfTest T3 增强 | 6h |

### Phase 4: 架构统一（Week 6-8）

| # | 任务 | 影响 | 预估工时 |
|---|------|------|---------|
| T4.1 | **core/ 双架构合并** — 识别 core/ 中可保留的模块，合入 L5/L6 | 消除双架构 | 40h |
| T4.2 | **Facade 瘦身** — 12个facade→4个，删除不必要的 re-export | 减少耦合 | 8h |
| T4.3 | **L4 Emotion 扩充** — 从 954行扩充到 2K+，成为完整情感层 | 补全能力缺口 | 12h |
| T4.4 | **CapabilityNetwork 正式化** — 8个category trait 统一为 I1-I6 接口 | 统一能力网 | 10h |

### Phase 5: 测试与验证（Week 8-10）

| # | 任务 | 影响 | 预估工时 |
|---|------|------|---------|
| T5.1 | **全量 `cargo check` 通过** — 0 errors, 0 warnings | 编译健康 | 4h |
| T5.2 | **单元测试覆盖率 >80%** — 新增模块必须有测试 | 质量保证 | 20h |
| T5.3 | **架构约束测试** — 编译时检查层级依赖方向 | 防止回退 | 8h |
| T5.4 | **性能回归测试** — 改构前后 benchmark 对比 | 确保无退化 | 4h |

---

## 七、多 Agent 自动巡检修复方案

### 7.1 巡检 Agent 分工

| Agent | 职责 | 工具 | 巡检频率 |
|-------|------|------|---------|
| **Agent-Lint** | 死代码/命名规范/层级违规检测 | `cargo clippy`, `cargo check`, grep | 每次提交 |
| **Agent-Security** | 安全漏洞扫描/依赖审计 | `cargo audit`, NT-SHIELD scan | 每日 |
| **Agent-Arch** | 架构约束验证/层级依赖图 | 自定义架构 lint rules | 每次 PR |
| **Agent-Perf** | 性能回归检测/编译时间监控 | `cargo build --timings`, criterion | 每周 |
| **Agent-Test** | 测试覆盖率/回归检测 | `cargo tarpaulin`, `cargo test` | 每次 PR |

### 7.2 自动巡检流程

```
┌─────────────────────────────────────────────────┐
│                   Git Push                       │
└─────────┬───────────────────────────────────────┘
          │
          ▼
┌─────────────────┐    ┌─────────────────┐
│ Agent-Lint      │    │ Agent-Arch      │
│ • dead code     │    │ • layer deps    │
│ • naming        │    │ • facade usage  │
│ • unused import │    │ • trait impls   │
└────────┬────────┘    └────────┬────────┘
         │                      │
         ▼                      ▼
┌─────────────────┐    ┌─────────────────┐
│ Agent-Security  │    │ Agent-Test      │
│ • cargo audit   │    │ • coverage      │
│ • dep scan      │    │ • regression    │
│ • secret leak   │    │ • unit tests    │
└────────┬────────┘    └────────┬────────┘
         │                      │
         └──────────┬───────────┘
                    ▼
         ┌─────────────────┐
         │ Agent-Perf      │
         │ • compile time  │
         │ • binary size   │
         │ • benchmark     │
         └────────┬────────┘
                  ▼
         ┌─────────────────┐
         │ 综合报告         │
         │ • Pass/Fail     │
         │ • 风险评分       │
         │ • 修复建议       │
         └─────────────────┘
```

### 7.3 巡检规则（可执行）

| 规则ID | 检查项 | 严重度 | 自动修复 |
|--------|--------|--------|---------|
| AR-01 | 低层模块向上依赖高层 | CRITICAL | ❌ 需人工 |
| AR-02 | facade 包含 trait+impl (应仅 re-export) | HIGH | ✅ 可自动提取 |
| AR-03 | `_`-prefixed 未使用 trait | MEDIUM | ✅ 可自动删除 |
| AR-04 | `#![allow(dead_code)]` 全局使用 | HIGH | ✅ 可自动移除 |
| AR-05 | 命名跨域泄漏 (L3含L2命名) | MEDIUM | ⚠️ 需确认后重命名 |
| AR-06 | 空壳模块 (<50行有效代码) | LOW | ✅ 可自动标记 |
| AR-07 | 重复 trait 定义 | HIGH | ⚠️ 需确认合并策略 |
| AR-08 | Cargo.toml 未使用 feature flag | MEDIUM | ✅ 可自动清理 |
| AR-09 | 编译时间 >10% 增长 | MEDIUM | ❌ 需人工分析 |
| AR-10 | 测试覆盖率 <60% | HIGH | ❌ 需人工补充 |

---

## 八、融合架构成熟度评估

### 8.1 当前状态 (Before)

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构一致性 | 55/100 | 双架构冲突，层级违规 |
| 代码卫生 | 40/100 | 31 dead traits, 全局 dead_code 抑制 |
| 能力完整性 | 70/100 | 8/8 L1 categories 有实现，但缺 Agent-as-Tool |
| 外部模式吸收 | 30/100 | 12个核心模式仅吸收3个 |
| 测试覆盖 | 50/100 | 部分模块有测试，无架构约束测试 |
| **综合** | **49/100** | — |

### 8.2 目标状态 (After Phase 1-5)

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构一致性 | 90/100 | 单一架构，层级清晰 |
| 代码卫生 | 85/100 | 无 dead code，命名规范 |
| 能力完整性 | 90/100 | 6个新接口实现 |
| 外部模式吸收 | 80/100 | 12个模式吸收10个 |
| 测试覆盖 | 85/100 | 架构约束测试+单元测试 |
| **综合** | **86/100** | +37 提升 |

---

## 九、Stigmergic 技术生态对齐

基于递归元智能理论，NeoTrix 的进化应遵循：

1. **Proposal–Consequence 分离** → NT-SHIELD 的 sandbox 执行 + NT-CORE 的意识判断
2. **物理 Stigmergy** → KB 作为共享工件，agent 通过读写 KB 协调（非直接对话）
3. **持久技术生态** → experience-tree 的累积知识，跨 session 存活
4. **递归元智能** → NT-META 的 ConsciousnessTree 驱动自我改进
5. **工件即消息** → 每个 NT-* 模块的输出是下一个模块的输入环境

**NeoTrix 的独特优势**: 已有 E8 reasoning + HyperCube + GWT + SEAL 理论基础，外部源验证了这些方向的正确性。需要的是**工程化吸收**（把理论变成可执行代码）而非**理论创新**。
