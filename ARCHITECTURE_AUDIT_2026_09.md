# NeoTrix 架构审计报告 2026-09-16

> **审计范围**: neotrix-core/src/ 全量 1993 个 Rust 文件
> **审计日期**: 2026-09-16 | **审计方法**: 自动化扫描 + 人工审查

---

## 一、架构总览

### 1.1 当前目录结构

```
neotrix-core/src/
├── l1_action/          # L1 行动层 (新架构)
│   ├── nt_act/         # 决策执行
│   ├── nt_io/          # 模型通信
│   ├── nt_memory/      # 记忆
│   ├── nt_media/       # 媒体
│   └── traits.rs
├── l2_perception/      # L2 感知层 (新架构)
│   └── nt_world/
├── l3_embodiment/      # L3 具身层 (新架构)
│   ├── nt_shield/
│   └── l1_facade.rs
├── l4_emotion/         # L4 情感层 (新架构)
│   ├── nt_feel/
│   └── nt_feel_facade.rs
├── l5_cognition/       # L5 认知层 (新架构)
│   ├── nt_core/
│   ├── nt_mind/
│   ├── nt_goal/
│   └── facade.rs
├── l6_meta/            # L6 元认知层 (新架构)
│   ├── nt_meta/
│   ├── nt_repair/
│   ├── nt_nexus/
│   └── coordination/
├── core/               # ⚠️ 旧 9 层架构 (需清理)
│   ├── l0_substrate/   # 旧 L0
│   ├── l1_body/        # 旧 L1
│   ├── l2_perception/  # 旧 L2
│   ├── l3_memory/      # 旧 L3
│   ├── l5_consciousness/ # 旧 L5
│   ├── l6_self/        # 旧 L6
│   ├── l7_capability/  # 旧 L7
│   ├── l8_autonomic/   # 旧 L8
│   └── (100+ 个 nt_core_* 模块)
├── protocol/           # 新增协议层
│   └── nt_nostr/
├── adapter/            # 新增适配层
│   └── nt_universal_provider/
├── neotrix/            # 新模块
│   ├── nt_acp/
│   ├── nt_nostr/
│   └── nt_universal_provider/
└── (其他: cli/, server/, agent.rs, lib.rs, main.rs)
```

### 1.2 双架构冲突 (CRITICAL)

| 架构 | 位置 | 层数 | 模块数 | 状态 |
|------|------|------|--------|------|
| **新架构** | l1_action/ ~ l6_meta/ + protocol/ + adapter/ | 6+2 | ~40 | ✅ 活跃 |
| **旧架构** | core/ (l0_substrate ~ l8_autonomic) | 9 | ~100+ | ⚠️ 仍存在 |

**冗余度**: 旧架构 core/ 中的模块与新架构 l1-l6 中的模块存在大量功能重叠。

### 1.3 已识别冗余 (10 项)

| # | 冗余项 | 位置A | 位置B | 影响 |
|---|--------|-------|-------|------|
| R1 | 模型路由策略 | nt_io/model_routing.rs | nt_core_gwt | 双重路由决策 |
| R2 | 文件解析 | nt_file_ability/ | nt_memory/ | 数据双写 |
| R3 | 浏览器自动化 | nt_world/ | nt_shield/ | 职责不清 |
| R4 | 技能加载 | nt_act/ + nt_io/ | l3_vendor/ | 多源重复 |
| R5 | 知识库存储 | nt_memory/ SQLite | codebase-memory | 存储冗余 |
| R6 | nt_act/ 旧副本 | neotrix/nt_act/ | l1_action/nt_act/ | 重复代码 |
| R7 | Eli5Explainer | l1_action/ | l5_cognition/ | 双位置 |
| R8 | Emotion 系统 | l4_emotion/ | l1_action/nt_io/ | 双系统 |
| R9 | OSINT | l2_perception/ | l3_embodiment/ | 双位置 |
| R10 | Goal/Cognition | l1_action/nt_act/ | l5_cognition/ | 错位 |

### 1.4 已识别缺陷 (7 项)

| # | 缺陷 | 严重度 | 位置 | 状态 |
|---|------|--------|------|------|
| D1 | GWT 缺复杂度维度 | 高 | nt_core_gwt | 待修 |
| D2 | 投机解码未集成 SelfModel | 高 | nt_core_self | 待修 |
| D3 | PolicyDrivenForgetting 未激活 | 中 | typed_memory/forgetting.rs | 待修 |
| D4 | LMCache 热存储未接入 | 中 | nt_memory | 待修 |
| D5 | WHALE phase 切换未接入 | 高 | nt_mind | 待修 |
| D6 | SkillSpector 验证未实现 | 中 | nt_mind | 待修 |
| D7 | Codebase Memory MCP 未接入 | 低 | nt_memory | 待修 |

### 1.5 跨域错位 (5 项)

| # | 错位 | 域A | 域B | 解决方案 |
|---|------|-----|-----|----------|
| X1 | NT-ACT vs NT-IO | nt_act | nt_io | ACT=决策, IO=通信 |
| X2 | NT-CORE vs NT-MIND | nt_core | nt_mind | CORE=推理, MIND=进化 |
| X3 | NT-WORLD vs NT-SHIELD | nt_world | nt_shield | WORLD=感知, SHIELD=隐身 |
| X4 | NT-MEMORY vs NT-NEXUS | nt_memory | nt_nexus | MEMORY=KB, NEXUS=编织 |
| X5 | nt_file_ability vs nt_world | nt_file_ability | nt_world | file=Office, world=Web |

---

## 二、已实现的融合模块

| 模块 | 位置 | 来源 | 成熟度 |
|------|------|------|--------|
| Universal Provider (UMA) | adapter/nt_universal_provider/ | Buzz+OR | C2 |
| NIP-01 Event Bus | protocol/nt_nostr/ | Buzz | C2 |
| ACP Protocol | neotrix/nt_acp/ | Buzz | C2 |
| Cost-Aware Router | adapter/nt_universal_provider/ | Axiom A1 | C2 |
| Event Store | protocol/nt_nostr/ | OR Local-First | C2 |
| AcpMcpBridge | neotrix/nt_acp/ | Buzz ACP↔MCP | C2 |

---

## 三、核心修复任务清单

### P0 — 立即执行 (阻塞性)

| # | 任务 | 模块 | 工时 | 验收标准 |
|---|------|------|------|----------|
| P0-1 | UMA 层创建 | adapter/nt_universal_provider | **已完成** | 所有外部模型统一接口 ✅ |
| P0-2 | NIP-01 事件总线 | protocol/nt_nostr | **已完成** | 所有操作编码为 signed events ✅ |
| P0-3 | GWT+Complexity 融合 | nt_core_gwt | 2h | GWT salience 含 ComplexityProfile |
| P0-4 | WHALE 循环接入 | nt_mind | 3h | consciousness_tick 含 autoresearch |
| P0-5 | SelfModel 投机解码 | nt_core_self | 2h | SelfModel 感知 spec_decode |
| P0-6 | 旧 core/ 架构清理 | core/ | 3h | 删除旧 l0-l8 子目录 |
| P0-7 | ACP 协议抽象 | neotrix/nt_acp | **已完成** | Goose/Codex/ClaudeCode 统一 ✅ |
| P0-8 | Autoresearch Loop | nt_mind | 3h | propose→experiment→decide 闭环 |

### P1 — 近期 (增强性)

| # | 任务 | 模块 | 工时 |
|---|------|------|------|
| P1-1 | Experiment Tree | nt_nexus | 2h |
| P1-2 | LMCache HotStore | nt_memory | 2h |
| P1-3 | SkillSpector 验证 | nt_mind | 2h |
| P1-4 | Parallel Worktrees | nt_physical | 2h |
| P1-5 | YAML Workflow Engine | nt_workflow (new) | 2h |
| P1-6 | Persona → SelfModel | nt_core_self | 1h |
| P1-7 | 桌面 Tauri Commands | src-tauri | 3h |
| P1-8 | Local SQLite Store | nt_memory | 1h |
| P1-9 | CLI 增强 | nt_cli (new) | 1h |
| P1-10 | Multi-Model Download | ModelManager | 2h |

### P2 — 后续 (优化性)

| # | 任务 | 模块 | 工时 |
|---|------|------|------|
| P2-1 | Cost-Aware Routing | nt_core_gwt | 2h |
| P2-2 | Agent-as-Tool Recursive | nt_act | 2h |
| P2-3 | Epistemic Knowledge Graph | nt_nexus | 2h |
| P2-4 | Config-as-Tree Slots | nt_core | 1h |
| P2-5 | Budgeted Skill Evolution | nt_act | 2h |
| P2-6 | Context Compaction | nt_memory | 2h |
| P2-7 | WebGPU Inference | nt_physical | 3h |
| P2-8 | Adversarial Defense | nt_shield | 2h |
| P2-9 | Coverage Ledger | nt_memory | 1h |
| P2-10 | NIP-34 Git Events | nt_nostr | 2h |

---

## 四、执行计划

### Phase 1: 核心缺陷修复 (P0-3, P0-4, P0-5)
- GWT+Complexity 融合
- WHALE 循环接入
- SelfModel 投机解码

### Phase 2: 旧架构清理 (P0-6)
- 清理 core/ 中的旧 l0-l8 子目录
- 统一为单一 6+2 层架构

### Phase 3: 增强功能 (P1 系列)
- YAML Workflow + CLI + Tauri Commands
- Multi-Model Download + Local Storage

### Phase 4: 优化完善 (P2 系列)
- Cost-Aware + Agent-as-Tool + Adversarial Defense

---

*审计完成。准备执行修复。*