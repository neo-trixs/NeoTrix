# NeoTrix 冗余-缺陷-错位扫描报告 (215)

**日期**: 2026-09-10  
**范围**: neotrix-core/src 全量扫描  

## 摘要

| 类别 | 数量 | 严重度 |
|------|------|--------|
| 聚焦冗余 | 78 个 Registry 结构体, 35 个 Config, 15 个 Engine, 7 个 Manager | HIGH |
| 扁平缺陷 | 4923 unwrap/expect, 107 panic!, 330 unsafe | CRITICAL |
| 跨域错位 | 25 个非法层引用 | HIGH |
| 模块放置 | 15 个重复 mod 名, 30 个重复 types.rs | MEDIUM |

---

## 1. 聚焦冗余清单

### 1.1 跨域重复 Registry (同名不同域)

| 名称 | 出现次数 | 位置 |
|------|---------|------|
| `ToolRegistry` | 3 | `core/nt_core_gate`, `l2_perception/nt_world/crawl/agentic_browse`, `neotrix/nt_act/tools` |
| `RegistryStats` | 3 | `l2_perception/nt_world/asset_registry`, `l5_cognition/nt_core/skill_crystallizer`, `neotrix/nt_core_capability_tree/registry` |
| `CapabilityRegistry` | 3 | `core/l7_capability/registry`, `core/nt_core_capability/mod`, `neotrix/nt_core_capability_tree/registry` |
| `SkillRegistry` | 2 | `core/l7_capability/skill_acquire`, `l5_cognition/nt_core/skill_crystallizer` |
| `FeedRegistry` | 2 | `l2_perception/nt_world/source/text/feed/registry`, `unified_archive/layers/perception/nt_world/nt_world_media_source/text/feed/registry` |
| `ExperimentRegistry` | 2 | `l5_cognition/nt_mind/evolution/experiment`, `l5_cognition/nt_mind/experiment` |
| `AssetRegistry` | 2 | `l2_perception/nt_world/asset_registry`, `l2_perception/nt_world/media_asset_registry` |

**根因**: 多处 CapabilityRegistry 同时存在于 `core/l7_capability`、`core/nt_core_capability`、`neotrix/nt_core_capability_tree` 三处，是历史迁移遗留。

### 1.2 域内泛滥类型

| 类型 | 总数 | Top 域 |
|------|------|--------|
| `*Config` | 35 | L1 (nt_io), L5 (nt_mind), core |
| `*Engine` | 15 | L5 (nt_mind), L2 (nt_world), core |
| `*Manager` | 7 | L5 (nt_mind), L1 (nt_act) |
| `*Service` | 2 | core |
| `*Builder` | 1 | L5 |

### 1.3 重复文件名

| 文件名 | 出现次数 | 含义 |
|--------|---------|------|
| `types.rs` | 30 | 多数模块各自定义 types，无统一 |
| `traits.rs` | 9 | 多处 trait 定义，部分重复 |
| `engine.rs` | 9 | 多个 Engine 实现 |
| `registry.rs` | 8 | 多个 Registry 实现 |
| `config.rs` | 7 | 多个 Config 结构 |

### 1.4 重复 mod 名 (同名目录在不同层)

| mod名 | 出现次数 |
|-------|---------|
| `evolution` | 4 |
| `core` | 3 |
| `video`, `text`, `social` | 各 2 |
| `nt_mind`, `nt_shield`, `nt_feel`, `nt_act` | 各 2 |

---

## 2. 扁平缺陷清单

### 2.1 unwrap/expect 分布 (4923 处)

| 域 | 数量 | 占比 | 风险等级 |
|----|------|------|---------|
| **L1 nt_memory/nt_memory_kb** | 828 + 221(ntx) | 21.3% | CRITICAL |
| **core** | 350 | 7.1% | HIGH |
| **L5 nt_mind** | 187 + 173(evolution) | 7.1% | HIGH |
| **neotrix (顶层)** | 198 | 4.0% | MEDIUM |
| **L2 nt_world** | 158 | 3.2% | MEDIUM |
| **L1 nt_io** | 140 | 2.8% | MEDIUM |
| **L3 nt_shield** | 127 | 2.6% | MEDIUM |
| **bin** | 112 | 2.3% | LOW (CLI) |

**Top 5 最高密度目录**:
1. `l1_action/nt_memory/nt_memory_kb` — 828 处
2. `core` — 350 处
3. `l1_action/nt_memory/nt_memory_kb/ntx` — 221 处
4. `neotrix` (顶层) — 198 处
5. `l5_cognition/nt_mind/nt_mind` — 187 处

### 2.2 panic! 分布 (107 处)

| 文件 | 数量 | 场景 |
|------|------|------|
| `core/nt_core_self/session_log_antipattern.rs` | 5 | 模式检测 |
| `core/energy_core/vibration.rs` | 2 | 能量计算 |
| `core/nt_game/play/adaptive.rs` | 2 | 游戏自适应 |
| `core/nt_core_self_review/mod.rs` | 2 | 自审逻辑 |
| `core/nt_core_context/context_budget.rs` | 1 | 上下文预算 |
| `core/nt_core_dispatch.rs` | 1 | 分发 |
| `core/nt_core_guard_chain.rs` | 1 | 守卫链 |
| `core/nt_core_error_recovery.rs` | 1 | 错误恢复 |

**关键风险**: `session_log_antipattern.rs` 中 5 处 panic 在正常路径触发，应替换为 Result 传播。

### 2.3 unsafe 分布 (330 处)

| 域 | 数量 | 说明 |
|----|------|------|
| **L5 nt_mind/evolution** | 73 | 进化模块 FFI/FFI |
| **core/nt_core_meta** | 35 | 元认知 FFI |
| **L5 nt_consciousness_core** | 29 | 意识核心 FFI |
| **L1 nt_act_goal** | 21 | 目标系统 FFI |
| **L3 nt_shield** | 14 | 安全模块 FFI |
| **L2 nt_world** | 14 | 世界模块 FFI |

**R-P1 违规**: 项目 axioms 要求 `#![forbid(unsafe_code)]` 在 core，但 core 域有 35+ unsafe 在 `nt_core_meta`，且 L5 evolution 有 73 处。

---

## 3. 跨域错位清单

### 3.1 层引用违规

| 违规方向 | 数量 | 严重度 |
|----------|------|--------|
| **L2→L1** | 11 | HIGH (感知层引用行动层) |
| **L5→L3** | 10 | HIGH (认知层引用具身层) |
| **L1→L2** | 3 | MEDIUM (行动层引用感知层) |
| **L6→L5** | 1 | LOW (元认知引用认知层, 可接受) |

### 3.2 L2→L1 违规详情 (11 处)

L2 感知层不应直接依赖 L1 行动层 (nt_memory_kb)：

| 文件 | 引用目标 |
|------|---------|
| `l2_perception/nt_world/nt_world_github_absorber.rs:12` | `l1_action::nt_memory::nt_memory_kb::KnowledgeBase` |
| `l2_perception/nt_world/nt_world_github_absorber.rs:351` | `l1_action::nt_memory::nt_memory_kb::nt_http` |
| `l2_perception/nt_world/nt_world_exploration_engine.rs:15` | `l1_action::nt_memory::nt_memory_kb::KnowledgeBase` |
| `l2_perception/nt_world/osint/mod.rs:31-33` | `l1_action::nt_memory::nt_memory_kb` (3处) |
| `l2_perception/nt_world/osint/mod.rs:1034` | `l1_action::nt_memory::nt_memory_kb::nt_discovery_github_topics` |
| `l2_perception/nt_world/nt_world_ods.rs:9,169` | `l1_action::nt_memory::nt_memory_kb::KnowledgeBase` (2处) |
| `l2_perception/nt_world/nt_world_monitor.rs:9,195` | `l1_action::nt_memory::nt_memory_kb::KnowledgeBase` (2处) |

**根因**: `KnowledgeBase` 是 L1 的记忆基础设施，L2 感知层写入 KB 应通过 trait 抽象（如 `MemoryWriter` trait），不应直接引用 `l1_action::nt_memory`。

### 3.3 L5→L3 违规详情 (10 处)

L5 认知层不应直接依赖 L3 具身层 (nt_shield)：

| 文件 | 引用目标 |
|------|---------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:3-5` | `l3_embodiment::nt_shield::nt_shield::browser_security::*`, `check_registry::CheckRegistry` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1427` | `l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1500` | `l3_embodiment::nt_shield::nt_shield_audit` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2083-2085` | 同上重复引用 |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:65-66` | `l3_embodiment::nt_shield::nt_shield::browser_security::*`, `check_registry::CheckRegistry` |

**根因**: `BrowserSecurityScanner` 和 `CheckRegistry` 是 L3 安全扫描器，L5 认知层调用安全检查应通过 trait 抽象（如 `SecurityCheck` trait），不应直接引用 L3 实现。

### 3.4 L1→L2 违规详情 (3 处)

| 文件 | 引用目标 |
|------|---------|
| `l1_action/nt_act/nt_act_orchestrator/critic.rs:33` | `l2_perception::nt_world::nt_world_model::TaskType` |
| `l1_action/mod.rs:10` | `l2_perception::nt_sense::nt_infra_semantic_router` |
| `l1_action/nt_memory/mod.rs:15` | `l2_perception::nt_sense::nt_memory_spatial` |

**根因**: `mod.rs` 中的 `pub use` 是 re-export，属于接口适配，严重度较低。`critic.rs` 中的 `TaskType` 引用应提升到共享 types crate。

---

## 4. 模块放置问题

### 4.1 重复 mod 目录 (同名在不同层)

| mod名 | 出现位置 |
|-------|---------|
| `evolution` (4处) | `l5_cognition/nt_mind/evolution`, `l5_cognition/nt_mind/nt_mind/evolution`, `l5_cognition/nt_mind/nt_mind/seal_core`, `l5_cognition/nt_mind/nt_mind/skill_tree` |
| `core` (3处) | `core/`, `l5_cognition/nt_core/`, `l1_action/nt_act/nt_act_crypto/` |
| `video` (2处) | `l2_perception/nt_world/source/video/`, `l2_perception/nt_world/nt_world_video_pipeline` |
| `text` (2处) | `l2_perception/nt_world/source/text/`, `l2_perception/nt_world/nt_world_intel_selftest` |
| `social` (2处) | `l1_action/nt_io/nt_io_hermes_community`, `l1_action/nt_act/actions/media` |

### 4.2 重复 types.rs (30 处)

`types.rs` 是最泛滥的文件名，多数模块各自定义本地类型。建议：
- 提取公共类型到 `neotrix-core/src/shared/types.rs`
- 保留各域特有类型在本地 `types.rs`

### 4.3 L4 情感层放置

L4 目录下 `nt_feel/` 同时存在于 `l3_embodiment/nt_feel/` 和 `l4_emotion/nt_feel/`，形成双层引用。`l4_emotion/nt_feel/nt_feel/emotion_engine.rs` 和 `l3_embodiment/nt_feel/` 共享同名模块，增加认知负担。

### 4.6 unified_archive 中的旧代码

`neotrix-core/src/unified_archive/layers/perception/nt_world/nt_world_media_source/text/feed/registry.rs` 仍保留旧路径的 `FeedRegistry`，与 `l2_perception` 中的新版重复。

---

## 5. 建议修复优先级

| 优先级 | 修复项 | 预估工作量 |
|--------|--------|-----------|
| **P0** | L2→L1 KnowledgeBase 引用改为 trait 抽象 | 2-3 天 |
| **P0** | L5→L3 BrowserSecurity 引用改为 trait 抽象 | 1-2 天 |
| **P1** | 合并 3 处 CapabilityRegistry 为单一实现 | 2-3 天 |
| **P1** | `session_log_antipattern.rs` panic! → Result | 0.5 天 |
| **P2** | `nt_memory_kb` 828 处 unwrap 逐步替换 | 5-7 天 |
| **P2** | 合并 ExperimentRegistry 2 处为 1 处 | 0.5 天 |
| **P3** | 提取公共 types 到 shared crate | 2-3 天 |
| **P3** | 清理 unified_archive 旧路径代码 | 1 天 |
| **P4** | unsafe 审计 (330 处) | 5-10 天 |

---

*报告由 neotrix scan 脚本自动生成*
