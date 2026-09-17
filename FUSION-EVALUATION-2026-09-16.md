# NeoTrix 全量迭代评测 — Buzz + OpenResearch 吸收后

> **日期**: 2026-09-16 | **Sources**: block/buzz (32.9k★), alphaXiv/OpenResearch (3.5k★)
> **评估维度**: 架构完整度 × 功能覆盖率 × 冗余度 × 缺陷密度 × 跨域对齐

---

## 一、项目总览

### 1.1 已完成的核心吸收

| 模块 | 文件位置 | 外部来源 | 成熟度 | 状态 |
|------|----------|----------|--------|------|
| **Universal Provider (UMA)** | `nt_universal_provider/mod.rs` | Buzz ACP + OR Multi-Model | C2 | ✅ 新建 |
| **NIP-01 Event Bus** | `nt_nostr/mod.rs` | Buzz Nostr Relay | C2 | ✅ 新建 |
| **ACP Protocol** | `nt_acp/mod.rs` | Buzz ACP Harness | C2 | ✅ 新建 |
| **Cost-Aware Router** | `nt_universal_provider/mod.rs` | Axiom A1 | C2 | ✅ 新建 |
| **Event Store** | `nt_nostr/mod.rs` | OpenResearch Local-First | C2 | ✅ 新建 |
| **AcpMcpBridge** | `nt_acp/mod.rs` | Buzz ACP ↔ MCP | C2 | ✅ 新建 |
| **复杂度分类器** | `nt_io/model_routing.rs` | openfreerouter | C4 | ✅ 已有 |
| **投机解码** | `nt_io/model_routing.rs` | vLLM N-Gram | C4 | ✅ 已有 |
| **GWT Attention Routing** | `nt_core_gwt` | Spotify Shunt | C5 | ✅ 已有 |
| **E8 引导者** | `nt_core` | Neotrix | C5 | ✅ 已有 |
| **Egress Privacy Guard** | `nt_core_llm` | NT-SHIELD | C5 | ✅ 已有 |

### 1.2 架构映射完成度

```
L6 Meta-Cognition  →  nt_meta + nt_repair + nt_nexus    [85%]
L5 Cognition       →  nt_core + nt_mind                  [80%]
L4 Emotion         →  nt_feel (core emotion engine)     [70%]
L3 Embodiment      →  nt_physical + nt_shield + nt_feel [75%]
L2 Perception      →  nt_world + nt_sense                [65%]
L1 Action          →  nt_act + nt_io + nt_memory         [80%]
+ Protocol         →  nt_nostr (NIP-01)                  [NEW]
+ Adapter          →  nt_universal_provider + nt_acp     [NEW]
```

---

## 二、聚焦冗余分析 (已识别 10 项)

| # | 冗余项 | 位置 | 状态 | 优先级 |
|---|--------|------|------|--------|
| R1 | 模型路由 vs GWT salience | model_routing + nt_core_gwt | 待合并 | P0 |
| R2 | 文件解析双位置 | nt_file_ability + nt_memory | 待统一 | P1 |
| R3 | 浏览器自动化跨域 | nt_world + nt_shield | 待分界 | P1 |
| R4 | 技能加载多源 | nt_act + nt_io + l3_vendor | 待统一 | P1 |
| R5 | 知识库双存储 | nt_memory + codebase-memory | 待合并 | P1 |
| R6 | nt_act/ 旧副本 | neotrix/nt_act/ | 待删除 | P0 |
| R7 | Eli5Explainer 双位置 | l1 + l5 | 待合并 | P2 |
| R8 | Emotion 双系统 | l4 + l1 | 待迁移 | P2 |
| R9 | OSINT 双位置 | l2 + l3 | 待统一 | P2 |
| R10 | Goal/Cognition in L1 | l1 → l5 | 待迁移 | P2 |

---

## 三、扁平缺陷分析 (已识别 7 项)

| # | 缺陷 | 严重度 | 修复方案 | 状态 |
|---|------|--------|----------|------|
| D1 | GWT 缺复杂度维度 | 高 | 接入 ComplexityProfile → GWT salience | 待修 |
| D2 | 投机解码未集成 SelfModel | 高 | SelfModel 感知 speculative_decoding | 待修 |
| D3 | PolicyDrivenForgetting 未激活 | 中 | 实现衰减策略 + experience-tree | 待修 |
| D4 | LMCache 热存储未接入 | 中 | LMCache adapter 作为 HotStore | 待修 |
| D5 | WHALE phase 切换未接入 | 高 | consciousness_tick 含 phase 闭环 | 待修 |
| D6 | SkillSpector 验证未实现 | 中 | SKILL.md 合约验证 | 待修 |
| D7 | Codebase Memory MCP 未接入 | 低 | 集成到 NT-MEMORY | 待修 |

---

## 四、跨域错位分析 (已识别 5 项)

| # | 错位 | 域A | 域B | 解决方案 | 状态 |
|---|------|-----|-----|----------|------|
| X1 | NT-ACT vs NT-IO 执行边界 | nt_act | nt_io | ACT=决策, IO=通信 | 已分界 |
| X2 | NT-CORE vs NT-MIND | nt_core | nt_mind | CORE=推理, MIND=进化 | 已分界 |
| X3 | NT-WORLD vs NT-SHIELD | nt_world | nt_shield | WORLD=感知, SHIELD=隐身 | 已分界 |
| X4 | NT-MEMORY vs NT-NEXUS | nt_memory | nt_nexus | MEMORY=KB, NEXUS=编织 | 已分界 |
| X5 | nt_file_ability vs nt_world | nt_file_ability | nt_world | file=Office, world=Web | 已分界 |

---

## 五、核心路线任务清单

### P0 — 立即执行 (8 任务)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P0-1 | UMA 层创建 | nt_universal_provider | Buzz+OR | 所有外部模型统一接口 | **已完成** | nt-act |
| P0-2 | NIP-01 事件总线 | nt_nostr | Buzz | 所有操作编码为 signed events | **已完成** | nt-act |
| P0-3 | GWT+Complexity 融合 | nt_core_gwt | openfreerouter | GWT salience 含 ComplexityProfile | 2d | nt-repair |
| P0-4 | WHALE 循环接入 | nt_mind | KRAFTON | consciousness_tick 含 autoresearch | 3d | nt-repair |
| P0-5 | SelfModel 投机解码 | nt_core_self | vLLM | SelfModel 感知 spec_decode | 2d | nt-repair |
| P0-6 | 双架构合并 | core/ + crate root | 架构审查 | 单一 6+1 层架构 | 2d | nt-core |
| P0-7 | ACP 协议抽象 | nt_acp | Buzz | Goose/Codex/ClaudeCode 统一 | **已完成** | nt-act |
| P0-8 | Autoresearch Loop | nt_mind | OR | propose→experiment→decide 闭环 | 3d | nt-act |

### P1 — 近期待完成 (10 任务)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P1-1 | Experiment Tree | nt_nexus | OR | Git-native experiment lineage | 2d | nt-act |
| P1-2 | LMCache HotStore | nt_memory | LMCache | MemoryMultitier 含 LMCache | 2d | nt-repair |
| P1-3 | SkillSpector 验证 | nt_mind | NVIDIA | SKILL.md 合约验证 | 2d | nt-repair |
| P1-4 | Parallel Worktrees | nt_physical | OR | 独立 agent sessions | 2d | nt-act |
| P1-5 | YAML Workflow Engine | nt_mind | Buzz | 事件触发器 → agent action | 2d | nt-act |
| P1-6 | Persona → SelfModel | nt_core_self | Buzz | Persona packs 作为 SelfModel ext | 1d | nt-act |
| P1-7 | 桌面 Tauri + React | src-tauri | Buzz | React 前端 + Tauri backend | 3d | nt-io |
| P1-8 | Local SQLite Store | nt_memory | OR | 所有数据本地优先 | 1d | nt-repair |
| P1-9 | orx CLI 增强 | neotrix-cli | OR | 统一 CLI 命令空间 | 1d | nt-act |
| P1-10 | Multi-Model Download | ModelManager | OR | LM Studio/oMLX/Ollama 支持 | 2d | nt-io |

### P2 — 后续完善 (10 任务)

| # | 任务 | 模块 | 来源 | 验收标准 | 工时 | agent |
|---|------|------|------|----------|------|-------|
| P2-1 | Cost-Aware Routing | nt_core_gwt | StrikeAgent | Token cost weight in GWT | 2d | nt-act |
| P2-2 | Agent-as-Tool Recursive | nt_act | sagent | AgentSelf/Spawn/Send | 2d | nt-act |
| P2-3 | Epistemic Knowledge Graph | nt_nexus | trackinizer | Inquiry+Valence edges | 2d | nt-repair |
| P2-4 | Config-as-Tree Slots | nt_core | priml | CapabilityRegistry formalization | 1d | nt-repair |
| P2-5 | Budgeted Skill Evolution | nt_act | COBRA-Skills | Bandit-guided optimization | 2d | nt-act |
| P2-6 | Context Compaction | nt_memory | sagent/GPT-6 | KVMem paged KV | 2d | nt-repair |
| P2-7 | WebGPU Inference | nt_physical | Shimmy | Pure-Rust WebGPU path | 3d | nt-physical |
| P2-8 | Adversarial Defense | nt_shield | Defending Code | 7-stage pipeline | 2d | nt-shield |
| P2-9 | Coverage Ledger | nt_memory | Cloudflare | Additive knowledge KB | 1d | nt-repair |
| P2-10 | NIP-34 Git Events | nt_nostr | Buzz | Patches, repo announcements | 2d | nt-nostr |

---

## 六、多 Agent 调度规划

### 5 Agent 并行编排

```
NT-CORE (指挥官)
  │
  ├── nt-act (实现者)     → 9 tasks: P0-1,2,7,8, P1-1,4,5,6,9, P2-1,2,5
  ├── nt-shield (安全官)  → 3 tasks: P0-6, P2-8,9
  ├── nt-core (架构师)    → 5 tasks: P0-3,6, P2-3,4,10
  ├── nt-io (接口师)      → 2 tasks: P1-7,10
  └── nt-repair (修复者)  → 10 tasks: P0-3,4,5, P1-2,3,8, P2-4,6,9, P0-6
```

### 关键里程碑

| 日期 | 里程碑 | 状态 |
|------|--------|------|
| Day 1 | UMA + NIP-01 + ACP 创建 | ✅ 完成 |
| Day 1-3 | P0-3, P0-4, P0-5, P0-6 修复 | 待执行 |
| Day 3-5 | P0-8, P1-1, P1-4, P1-5 实现 | 待执行 |
| Day 5-7 | P1-7, P1-10 桌面增强 | 待执行 |
| Day 7-14 | P1 全部 + P2 部分完成 | 待执行 |
| Day 14+ | P2 全部完成 + 全量测试 | 待执行 |

---

## 七、质量门汇总

| 检查项 | 标准 | 当前状态 |
|--------|------|----------|
| cargo check --all-targets | 通过 | ⚠️ 需要验证 |
| cargo test --lib | 无回归 | ⚠️ 需要验证 |
| clippy | 无新警告 | ⚠️ 需要验证 |
| #![forbid(unsafe_code)] | 零 unsafe | ✅ 所有新模块已遵守 |
| nt_ 前缀 | 所有模块 | ✅ 已遵守 |
| Rev-officer D1-D51 | 无 critical | ⚠️ 待审查 |
| 架构合并 | 无循环依赖 | ⚠️ 待执行 |

---

## 八、下一步行动

### 立即执行
1. 运行 `cargo check --all-targets` 验证新模块编译
2. 启动 P0-3 (GWT+Complexity) 修复
3. 启动 P0-6 (双架构合并) 审计

### 短期 (本周)
1. 完成 P0 所有 8 个任务
2. 开始 P1 任务
3. 全量测试通过

### 中期 (两周)
1. 完成 P1 全部 10 个任务
2. 开始 P2 任务
3. 桌面 app 可用

### 长期 (一个月)
1. 完成 P2 全部 10 个任务
2. 全量安全审查
3. 发布版本

---

*评测完成。准备执行多 Agent 自动巡检修复。*