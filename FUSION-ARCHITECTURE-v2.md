# NeoTrix Fusion Architecture v2 — Buzz + OpenResearch Absorption

> **日期**: 2026-09-16 | **Sources**: block/buzz (32.9k★), alphaXiv/OpenResearch (3.5k★)
> **目标**: 熔炼为 NeoTrix 桌面 app 通用方案，适用所有外部模型

---

## 一、外部源核心抽象提取

### 1.1 Buzz 提取的核心模式

| 模式 | Buzz 实现 | NeoTrix 映射 | 融合策略 |
|------|-----------|-------------|----------|
| **Nostr Relay Protocol** | NIP-01 events, NIP-42 auth, signed log | NT-SHIELD audit trail + NT-CORE event bus | 引入 `nt_nostr` 模块，将所有操作编码为 NIP-01 事件 |
| **Agent-as-Member** | Agents have own keys, channel membership, audit trail | NT-ACT agent identity | 每个 agent 获得独立 keypair + 域成员身份 |
| **ACP Harness** | buzz-acp (Goose/Codex/Claude Code) | NT-IO model adapter | 抽象 ACP 协议为 `nt_acp` trait，统一所有 agent harness |
| **Channel/Room** | Channels as persistent collaboration rooms | NT-WORLD workspace channels | 每个工作区 = 一个 channel，有自己的 event log |
| **YAML Workflows** | buzz-workflow (message/reaction/schedule/webhook triggers) | NT-MIND automation engine | YAML → 事件触发器 → agent action |
| **Persona System** | buzz-persona (agent persona packs) | NT-CORE self-model | Persona → SelfModel personality extension |
| **Desktop (Tauri + React)** | Buzz desktop app | NT-IO tauri_impl | Tauri 框架复用，React 前端替换 |

### 1.2 OpenResearch 提取的核心模式

| 模式 | OpenResearch 实现 | NeoTrix 映射 | 融合策略 |
|------|-------------------|-------------|----------|
| **Local-First** | SQLite store, 127.0.0.1 dashboard | NT-MEMORY local store | 所有数据本地优先，SQLite → 统一存储 |
| **Autoresearch Loop** | propose → change → experiment → inspect → decide | nt_mind_background_loop | `consciousness_tick` 增强 autoresearch 阶段 |
| **Experiment Tree** | Git-native experiment tree, immutable archive | NT-NEXUS knowledge graph | 每个 experiment = graph node，可追溯 lineage |
| **Multi-Model Adapter** | LM Studio, oMLX, Ollama, custom endpoints | ModelManager enhancement | 统一 ModelSource 枚举，增加 OpenResearch 源 |
| **Agent Skills** | `orx install-skills` for Claude Code/Codex/OpenCode/Cursor | SKILL-SPEC contract | OpenResearch skill → NeoTrix SKILL-SPEC 适配 |
| **Parallel Exploration** | Independent agent sessions + isolated git worktrees | NT-PHYSICAL sandbox | 每个 research direction = 独立 worktree + agent |
| **CLI (`orx`)** | orx projects/runs/logs/exp/discover/paper | nt-neotrix-cli enhancement | 统一 CLI 命令空间 |

---

## 二、通用模型适配层架构

### 2.1 Universal Model Adapter (UMA)

```
┌──────────────────────────────────────────────────────────────┐
│                    Universal Model Adapter                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Anthropic  │  │   OpenAI    │  │   Ollama    │         │
│  │  Provider   │  │  Provider   │  │  Provider   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│  ┌──────▼────────────────▼────────────────▼──────┐         │
│  │          Unified Message Protocol             │         │
│  │  (NIP-01 event format / ACP protocol)         │         │
│  └──────┬───────────────────────────────────────┘         │
│         │                                                │
│  ┌──────▼───────────────────────────────────────┐         │
│  │        Model Router (GWT + Cost)             │         │
│  │  - A1: Cost-Aware Routing                    │         │
│  │  - GWT salience + token cost weight          │         │
│  └──────┬───────────────────────────────────────┘         │
│         │                                                │
│  ┌──────▼───────────────────────────────────────┐         │
│  │        NT-CORE Inference Engine               │         │
│  │  - HyperCube (VSA embeddings)                │         │
│  │  - GWT Attention Routing                     │         │
│  │  - SelfModel (capability/uncertainty)        │         │
│  └──────┬───────────────────────────────────────┘         │
│         │                                                │
│  ┌──────▼───────────────────────────────────────┐         │
│  │        Local-First Storage                    │         │
│  │  - SQLite (events, sessions, experiments)    │         │
│  │  - Postgres (if relay mode)                  │         │
│  │  - S3/MinIO (media, models)                  │         │
│  └──────────────────────────────────────────────┘         │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Provider Abstraction Interface

```rust
/// 所有外部模型的统一接口
pub trait UniversalProvider: Send + Sync {
    /// 模型标识
    fn model_id(&self) -> &str;
    fn provider_type(&self) -> ProviderType;
    fn capabilities(&self) -> ModelCapabilities;
    
    /// 统一消息格式（NIP-01 event 或 ACP message）
    async fn complete(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError>;
    async fn stream(&self, request: UnifiedRequest) -> Result<tokio::sync::mpsc::Receiver<UnifiedResponse>, ProviderError>;
    
    /// 模型管理
    fn metadata(&self) -> ModelMetadata;
    async fn download(&self, url: &str) -> Result<PathBuf, ProviderError>;
}

/// 提供商类型 — 覆盖所有外部模型
pub enum ProviderType {
    // 商业 API
    Anthropic,
    OpenAI,
    GoogleGemini,
    OpenRouter,
    DeepSeek,
    Mistral,
    Groq,
    TogetherAI,
    // 本地/自托管
    Ollama,
    LlamaCpp,
    LMStudio,
    oMLX,
    LocalGGUF,
    VLLM,
    TGI,
    // Agent 工具
    Goose,
    Codex,
    ClaudeCode,
    Cursor,
    OpenCode,
    // Buzz/OpenResearch 原生
    BuzzRelay,
    OpenResearch,
    NostrRelay,
}
```

### 2.3 NIP-01 Event Integration

```rust
/// 所有桌面操作编码为 NIP-01 事件
pub struct NeoTrixEvent {
    /// 事件类型 (NIP-01 kind)
    pub kind: EventKind,
    /// 发布者公钥 (agent 或 user)
    pub pubkey: String,
    /// 事件内容 (JSON payload)
    pub content: String,
    /// 创建时间
    pub created_at: u64,
    /// 标签 (channel, worktree, etc.)
    pub tags: Vec<Vec<String>>,
    /// 签名 (agent key)
    pub sig: Option<String>,
}

/// NIP-01 kind 映射
pub enum EventKind {
    // 文本笔记
    TextNote = 1,
    // 加密消息
    Encrypted = 4,
    // 频道创建
    ChannelCreate = 40280,
    // 频道消息
    ChannelMessage = 40281,
    // Git 事件 (NIP-34)
    GitPatch = 30023,
    // 工作流步骤
    WorkflowStep = 30078,
    // 审批门
    ApprovalGate = 31337,
    // 研究实验
    Experiment = 31989,
    // Agent 心跳
    AgentHeartbeat = 31000,
}
```

---

## 三、聚焦冗余清理方案

### 3.1 架构级冗余：双系统冲突

**问题**: `core/` 内部 9 层 vs `crate root` 6 层，伪层级抽象，343 行 facade + 62 双向 import。

**修复**: 采用 OpenResearch 的 local-first 单层架构 + Buzz 的 protocol-first 统一层。

```
合并后单架构:
  L1 Action  →  nt_act, nt_io, nt_memory       (所有行动)
  L2 Perception → nt_world, nt_sense             (所有感知)
  L3 Embodiment  → nt_physical, nt_shield, nt_feel (具身)
  L4 Emotion     → nt_feel (单一情感引擎)         (情感)
  L5 Cognition   → nt_core, nt_mind               (推理)
  L6 Meta        → nt_meta, nt_repair, nt_nexus   (元认知)
  + Protocol     → nt_nostr (NIP-01 event bus)   (新增协议层)
  + Adapter      → nt_universal_provider          (新增通用适配层)
```

### 3.2 冗余清理清单

| # | 冗余项 | 位置A | 位置B | 修复 |
|---|--------|-------|-------|------|
| R1 | 模型路由 vs GWT | model_routing.rs + nt_core_gwt | 合并为 GWT+cost 统一信号 | **agent: refactor** |
| R2 | 文件解析双位置 | nt_file_ability + nt_memory | 统一到 nt_file_ability | **agent: refactor** |
| R3 | 浏览器自动化双域 | nt_world + nt_shield | WORLD=感知, SHIELD=隐身 | **agent: refactor** |
| R4 | 技能加载多源 | nt_act + nt_io + l3_vendor | 统一 SKILL-SPEC | **agent: refactor** |
| R5 | Knowledge 双存储 | nt_memory SQLite + codebase-memory | 合并为统一 KB | **agent: refactor** |
| R6 | nt_act/ 旧副本 | neotrix/nt_act/ (2796行) | 删除旧副本 | **agent: delete** |
| R7 | Eli5Explainer 双位置 | l1_action + l5_cognition | 合并到 L1 | **agent: refactor** |
| R8 | Emotion 双系统 | l4_emotion + l1_action | 迁移到 L4 | **agent: refactor** |
| R9 | OSINT 双位置 | l2_perception + l3_shield | 统一到 L2 | **agent: refactor** |
| R10 | Goal/Cognition in L1 | l1_action/nt_act_goal | 迁移到 L5 | **agent: refactor** |

### 3.3 扁平缺陷清单

| # | 缺陷 | 严重度 | 修复方案 |
|---|------|--------|----------|
| D1 | GWT 缺复杂度维度 | 高 | 接入 ComplexityProfile → GWT salience |
| D2 | 投机解码未集成 SelfModel | 高 | SelfModel 感知 speculative_decoding |
| D3 | PolicyDrivenForgetting 未激活 | 中 | 实现衰减 + experience-tree 联动 |
| D4 | LMCache 热存储未接入 | 中 | 添加 LMCache HotStore adapter |
| D5 | WHALE phase 切换未接入 | 高 | consciousness_tick 含 phase 闭环 |
| D6 | SkillSpector 验证未实现 | 中 | SKILL.md 合约验证 + content hashing |
| D7 | Codebase Memory MCP 未接入 | 低 | 集成到 NT-MEMORY |

### 3.4 跨域错位清单

| # | 错位 | 域A | 域B | 解决方案 |
|---|------|-----|-----|----------|
| X1 | NT-ACT vs NT-IO 执行边界 | nt_act | nt_io | ACT=决策执行，IO=模型通信+循环 |
| X2 | NT-CORE vs NT-MIND | nt_core | nt_mind | CORE=推理引擎，MIND=进化循环 |
| X3 | NT-WORLD vs NT-SHIELD 浏览器 | nt_world | nt_shield | WORLD=内容感知，SHIELD=隐身安全 |
| X4 | NT-MEMORY vs NT-NEXUS | nt_memory | nt_nexus | MEMORY=当前KB，NEXUS=跨会话编织 |
| X5 | nt_file_ability vs nt_world | nt_file_ability | nt_world | file_ability=Office格式，world=Web/爬取 |

---

## 四、迭代任务全量评测

### P0 — 立即执行 (阻塞性)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P0-1 | **UMA 层创建** | nt_universal_provider | Buzz+OR | 所有外部模型统一接口 | 3d | nt-act |
| P0-2 | **NIP-01 事件总线** | nt_nostr | Buzz | 所有操作编码为 signed events | 3d | nt-act |
| P0-3 | **GWT+Complexity 融合** | nt_core_gwt | openfreerouter | GWT salience 含 ComplexityProfile | 2d | nt-repair |
| P0-4 | **WHALE 循环接入** | nt_mind | KRAFTON | consciousness_tick 含 autoresearch | 3d | nt-repair |
| P0-5 | **SelfModel 投机解码** | nt_core_self | vLLM | SelfModel 感知 spec_decode | 2d | nt-repair |
| P0-6 | **双架构合并** | core/ + crate root | 架构审查 | 单一 6+1 层架构 | 2d | nt-repair |
| P0-7 | **ACP 协议抽象** | nt_acp | Buzz | Goose/Codex/ClaudeCode 统一 | 2d | nt-act |
| P0-8 | **Autoresearch Loop** | nt_mind | OR | propose→experiment→decide 闭环 | 3d | nt-act |

### P1 — 近期待完成 (增强性)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P1-1 | **Experiment Tree** | nt_nexus | OR | Git-native experiment lineage | 2d | nt-act |
| P1-2 | **LMCache HotStore** | nt_memory | LMCache | MemoryMultitier 含 LMCache | 2d | nt-repair |
| P1-3 | **SkillSpector 验证** | nt_mind | NVIDIA | SKILL.md 合约验证 | 2d | nt-repair |
| P1-4 | **Parallel Worktrees** | nt_physical | OR | 独立 agent sessions | 2d | nt-act |
| P1-5 | **YAML Workflow Engine** | nt_mind | Buzz | 事件触发器 → agent action | 2d | nt-act |
| P1-6 | **Persona → SelfModel** | nt_core_self | Buzz | Persona packs 作为 SelfModel ext | 1d | nt-act |
| P1-7 | **桌面 Tauri + React** | src-tauri | Buzz | React 前端 + Tauri backend | 3d | nt-io |
| P1-8 | **Local SQLite Store** | nt_memory | OR | 所有数据本地优先 | 1d | nt-repair |
| P1-9 | **orx CLI 增强** | neotrix-cli | OR | 统一 CLI 命令空间 | 1d | nt-act |
| P1-10 | **Multi-Model Download** | ModelManager | OR | LM Studio/oMLX/Ollama 支持 | 2d | nt-io |

### P2 — 后续完善 (优化性)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P2-1 | **Cost-Aware Routing** | nt_core_gwt | StrikeAgent | Token cost weight in GWT | 2d | nt-act |
| P2-2 | **Agent-as-Tool Recursive** | nt_act | sagent | AgentSelf/Spawn/Send | 2d | nt-act |
| P2-3 | **Epistemic Knowledge Graph** | nt_nexus | trackinizer | Inquiry+Valence edges | 2d | nt-repair |
| P2-4 | **Config-as-Tree Slots** | nt_core | priml | CapabilityRegistry formalization | 1d | nt-repair |
| P2-5 | **Budgeted Skill Evolution** | nt_act | COBRA-Skills | Bandit-guided optimization | 2d | nt-act |
| P2-6 | **Context Compaction** | nt_memory | sagent/GPT-6 | KVMem paged KV | 2d | nt-repair |
| P2-7 | **WebGPU Inference** | nt_physical | Shimmy | Pure-Rust WebGPU path | 3d | nt-physical |
| P2-8 | **Adversarial Defense** | nt_shield | Defending Code | 7-stage pipeline | 2d | nt-shield |
| P2-9 | **Coverage Ledger** | nt_memory | Cloudflare | Additive knowledge KB | 1d | nt-repair |
| P2-10 | **NIP-34 Git Events** | nt_nostr | Buzz | Patches, repo announcements | 2d | nt-nostr |

---

## 五、多 Agent 自动巡检修复调度

### 5.1 Agent 编排

```
┌─────────────────────────────────────────────────────────────┐
│                    NT-CORE (指挥官)                          │
│  统一下发任务 → 监控进度 → 汇总质量门 → 反馈循环             │
└──────┬──────────┬──────────┬──────────┬──────────┬─────────┘
       │          │          │          │          │
  ┌────▼───┐ ┌────▼───┐ ┌────▼───┐ ┌────▼───┐ ┌────▼───┐
  │ nt-act │ │nt-shield│ │nt-core │ │nt-io  │ │nt-repair│
  │  实现者│ │  安全官 │ │ 架构师  │ │  接口师 │ │  修复者 │
  │        │ │        │ │        │ │        │ │         │
  │ P0-1,P0-7│ │ P0-6  │ │ P0-3  │ │ P1-7  │ │ P0-3    │
  │ P0-2,P0-8│ │ P2-8  │ │ P0-6  │ │ P1-10 │ │ P0-4    │
  │ P1-4,P1-5│ │ P2-9  │ │ P2-4  │ │       │ │ P0-5    │
  │ P1-6,P2-1│ │       │ │ P2-3  │ │       │ │ P1-2    │
  │ P2-2,P2-5│ │       │ │ P2-10 │ │       │ │ P1-3    │
  │ P1-9    │ │       │ │       │ │       │ │ P2-6    │
  │         │ │       │ │       │ │       │ │ P2-9    │
  │         │ │       │ │       │ │       │ │ P0-6    │
  │         │ │       │ │       │ │       │ │ P1-8    │
  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

### 5.2 Agent 规格

| Agent | 角色 | 维度 | 负责任务 | 质量门 |
|-------|------|------|----------|--------|
| **nt-act** | 实现者 | P0-1,2,7,8, P1-4,5,6,9, P2-1,2,5 | 通用适配层、ACP、Autoresearch、CLI、YAML | cargo check + unit tests |
| **nt-shield** | 安全官 | P0-6, P2-8,9 | 架构合并、对抗防御、覆盖率 | rev-officer D1-D51 |
| **nt-core** | 架构师 | P0-3,6, P2-4,10 | GWT+Complexity、双架构合并、Config-as-Tree、NIP-34 | clippy + architecture review |
| **nt-io** | 接口师 | P1-7,10 | 桌面 Tauri+React、模型下载 | UI spec + model test |
| **nt-repair** | 修复者 | P0-3,4,5, P1-2,3,8, P2-6,9 | 全部缺陷修复、存储集成、循环接入 | cargo test --lib + D1-D51 |

### 5.3 巡检-修复闭环

```
Phase 1: Snapshot (快照)
  → neotrix-core_consciousness_status() 获取当前状态
  → 读取 NEOTRIX_EVALUATION.md 缺陷清单
  → 读取 FUSION-ARCHITECTURE-v2.md 架构定义

Phase 2: Dispatch (分发)
  → 按 agent 类型分发任务
  → 每个 agent 独立工作区 (git worktree)
  → 并行执行 P0 任务

Phase 3: Verify (验证)
  → cargo check --all-targets
  → cargo test -p neotrix --lib
  → clippy lint 检查
  → rev-officer 5 维度审查

Phase 4: Feedback (反馈)
  → 经验写入 KB (experience-tree)
  → 缺陷状态更新
  → 下一轮迭代规划
```

---

## 六、融合后架构全景

```
┌─────────────────────────────────────────────────────────────────┐
│                        NT-NEXUS  (跨会话记忆)                     │
│              Experiment Tree + Knowledge Graph                   │
├─────────────────────────────────────────────────────────────────┤
│                     NT-META  (元认知协调)                         │
│        consciousness_tick + MARS + SelfModel                     │
├─────────────────────────────────────────────────────────────────┤
│                     NT-MIND  (进化工匠)                           │
│  Autoresearch Loop + WHALE + SkillSpector + GWT Cost            │
├─────────────────────────────────────────────────────────────────┤
│                     NT-CORE  (推理引擎)                           │
│  HyperCube + E8 + ComplexityProfile + UniversalProvider         │
├─────────────────────────────────────────────────────────────────┤
│                     NT-SHIELD  (影卫)                             │
│  NIP-01 Audit + Adversarial Defense + NOSTR Auth + Privacy      │
├─────────────────────────────────────────────────────────────────┤
│                     NT-WORLD  (虚空探索者)                        │
│  Channels + Workspace + OSINT + Browser-感知                     │
├─────────────────────────────────────────────────────────────────┤
│                     NT-PHYSICAL (具身骨架)                        │
│  Sandbox + Worktrees + GPU KV Cache + WebGPU                   │
├─────────────────────────────────────────────────────────────────┤
│                     NT-FEEL  (情感中枢)                           │
│     Emotion Engine + Persona → SelfModel Extension               │
├─────────────────────────────────────────────────────────────────┤
│                     NT-ACT  (行动执行者)                          │
│  ACP Harness + YAML Workflow + Agent-as-Tool + CLI             │
├─────────────────────────────────────────────────────────────────┤
│                     NT-IO   (界面使徒)                            │
│  Universal Provider + Tauri+React + ModelManager + Desktop      │
├─────────────────────────────────────────────────────────────────┤
│                     NT-MEMORY (知识守护者)                        │
│  SQLite Local-First + LMCache + Coverage Ledger + KB           │
└─────────────────────────────────────────────────────────────────┘
```

---

## 七、核心路线图总结

### 总计: 28 个核心任务
- **P0**: 8 个任务 (立即执行，阻塞性)
- **P1**: 10 个任务 (增强性，近期完成)
- **P2**: 10 个任务 (优化性，后续完善)

### 多 Agent 调度: 5 个并行 agent
- **nt-act**: 9 个任务 (实现者)
- **nt-shield**: 3 个任务 (安全官)
- **nt-core**: 5 个任务 (架构师)
- **nt-io**: 2 个任务 (接口师)
- **nt-repair**: 10 个任务 (修复者)

### 关键里程碑
1. **Day 1-3**: UMA + NIP-01 + GWT 融合 + WHALE + SelfModel + 双架构合并 + ACP + Autoresearch
2. **Day 4-7**: Experiment Tree + 存储集成 + 缺陷修复 + 桌面适配
3. **Day 8-14**: 优化性任务 + 全量测试 + 验证
4. **Day 15+**: 持续迭代 + 经验吸收

---

*融合方案完成。准备启动多 Agent 自动巡检修复。*
