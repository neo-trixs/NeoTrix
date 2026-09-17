# NeoTrix 全局架构重构方案 (Global Architecture Refactoring)

> **基于**: 全景扫描 (2,497 文件) + 冗余检测 (4 LLM 抽象层/5 SkillRegistry/4 CapabilityRegistry) + 缺陷诊断 (10 编译错误/12+ 跨层违规) + 外部研究 (ZGCM-1/CTM/Drosophila Connectome)
>
> **日期**: 2026-09-16
> **成熟度**: C0→C1 (从诊断到实施)

---

## 一、诊断总览：三维缺陷矩阵

### 1.1 聚焦冗余 (Focused Redundancy)

| 冗余类型 | 数量 | 严重度 | 位置 |
|---------|------|--------|------|
| LLM 抽象层重复 | 4 套 | 🔴 CRITICAL | `nt_core_llm` / `universal_provider` / `gateway` / `neotrix-types` |
| SkillRegistry 重复 | 5 个 | 🔴 CRITICAL | `neotrix-types`×2 / `nt_core_capability` / `nt_act/skill_registry`×2 |
| CapabilityRegistry 重复 | 4 个 | 🟠 HIGH | `l7_capability` / `nt_core_capability` / `nt_file_ability` / `nt_core_capability_tree` |
| ProviderRegistry 重复 | 3 个 | 🟠 HIGH | `universal_provider`×2 / `nt_io_provider/catalog` |
| ExperimentRegistry 重复 | 2 个 (相同) | 🟡 MEDIUM | `nt_mind/experiment.rs` / `nt_mind/evolution/experiment.rs` |
| MetaCognitionBridge 重复 | 2 个 (相同) | 🟡 MEDIUM | `nt_mind/distillation.rs` / `nt_mind/evolution/distillation.rs` |
| HTTP 客户端碎片化 | 3 库/24+ 创建点 | 🟠 HIGH | reqwest×24 / ureq×0 / chromiumoxide×2 |
| SQLite 连接碎片化 | 100+ 直接使用 | 🟠 HIGH | 3 个独立 `open_kb()` 函数 |
| 死模块 | 14 个 | 🟡 MEDIUM | L2×7 / L3×7 (注释掉) |
| 死代码注解 | 100+ `#[allow(dead_code)]` | 🟡 MEDIUM | 全局 `#![allow(dead_code)]` 掩盖 |

### 1.2 扁平缺陷 (Flat Deficiency)

| 缺陷类型 | 数量 | 严重度 | 影响 |
|---------|------|--------|------|
| 编译错误 | 10 个 | 🔴 CRITICAL | 构建失败 |
| Trait 重复定义 | 7 个 | 🔴 CRITICAL | 类型不兼容 |
| 缺失方法 | 3 个 | 🟠 HIGH | `EmotionState` 缺 `observe_appraisal`/`confidence_score`/`label` |
| 类型不匹配 | 1 个 | 🟠 HIGH | `Option<Value>` vs `Option<OrchestratorResult>` |
| 缺失字段 | 1 个 | 🟠 HIGH | `CapabilityRegistry` 缺 `slots` |
| 命名违规 | 1 个 | 🟡 MEDIUM | `oMLX` 应为 `Omlix` |
| 死 Trait | 2+ 个 | 🟡 MEDIUM | `KbProvider` stub / `_BrowserSecurityCheck` |

### 1.3 跨域错位 (Cross-Domain Misalignment)

| 违规类型 | 数量 | 严重度 | 示例 |
|---------|------|--------|------|
| L5→L1 直接导入 | 6+ 处 | 🟠 HIGH | `nt_mind` 直接用 `nt_io_provider` 类型 |
| L6→L1 直接导入 | 4+ 处 | 🟠 HIGH | `nt_nexus` 直接用 `KnowledgeBase` |
| L6→L5 直接导入 | 2+ 处 | 🟡 MEDIUM | `nt_meta` 用 `awareness_monitor` |
| L1→L3 私有函数 | 7 处 | 🔴 CRITICAL | `coverage_ledger` 用 `sha256_hex` (L3 私有) |
| Facade 伪装合规 | 7 个 | 🟡 MEDIUM | Facade 允许 L5/L6 访问 L1，绕过层级 |

---

## 二、通用模型适配层 (Universal Model Adapter)

### 2.1 设计目标

**一次接入，全模型可用** — 所有外部 LLM/Agent 通过统一接口接入 NeoTrix 意识晶体。

```
┌─────────────────────────────────────────────────────────────┐
│                  Universal Model Adapter                     │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ OpenAI   │  │Anthropic │  │  Gemini  │  │  Ollama  │   │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │              │              │              │          │
│  ┌────▼──────────────▼──────────────▼──────────────▼────┐   │
│  │           Unified LLM Trait (Canonical)               │   │
│  │  fn complete(&self, req: LlmRequest) -> LlmResponse  │   │
│  └──────────────────────┬───────────────────────────────┘   │
│                         │                                    │
│  ┌──────────────────────▼───────────────────────────────┐   │
│  │              Gateway (Routing + Resilience)            │   │
│  │  - Load balancing (total_calls ascending)              │   │
│  │  - Circuit breaker + rate limiting                     │   │
│  │  - Egress privacy guard                                │   │
│  │  - Cost tracking + budget enforcement                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 统一 Trait 定义

**唯一 Canonical 定义**: `neotrix-core/src/core/nt_core_llm.rs`

```rust
// ✅ CORRECT: Single canonical definition
pub trait LlmProvider: Send + Sync {
    fn complete(&self, request: LlmRequest) -> LlmResponse;
    fn estimate_tokens(&self, text: &str) -> usize;
    fn model_id(&self) -> &str;
    fn provider_name(&self) -> &str;
}

// ✅ CORRECT: Gateway extends LlmProvider
pub trait GatewayProvider: LlmProvider {
    fn health_check(&self) -> ProviderHealth;
    fn cost_estimate(&self, tokens: usize) -> f64;
    fn rate_limit_status(&self) -> RateLimitStatus;
}
```

**删除的重复定义**:
- ❌ `UniversalProvider` (adapter/nt_universal_provider) → 合并到 `LlmProvider`
- ❌ `UnifiedProvider` (nt_core_llm 内部) → 合并到 `GatewayProvider`
- ❌ `GatewayFacade` (neotrix-types) → 删除，使用 `GatewayProvider`
- ❌ `nt_io_provider/common/types.rs` 的 `pub use crate::core::nt_core_llm::*` → 改为 `pub use crate::core::nt_core_llm::{LlmProvider, LlmRequest, LlmResponse}`

### 2.3 Provider 实现清单

| Provider | 现状 | 重构动作 |
|----------|------|---------|
| OpenAI | ✅ 完整实现 | 保持，适配统一 Trait |
| Anthropic | ✅ 完整实现 | 保持，适配统一 Trait |
| Gemini | ✅ 完整实现 | 保持，适配统一 Trait |
| Ollama | ✅ 完整实现 | 保持，适配统一 Trait |
| LLaMA | ✅ 本地推理 | 保持，适配统一 Trait |
| DeepSeek | ⚠️ 通过 factory | 统一为独立 Provider |
| Groq | ⚠️ 通过 catalog | 统一为独立 Provider |
| Together | ⚠️ 通过 catalog | 统一为独立 Provider |
| Novita | ⚠️ 通过 catalog | 统一为独立 Provider |
| OpenRouter | ⚠️ 通过 catalog | 统一为独立 Provider |

### 2.4 桌面 App 通用方案

**Tauri 集成路径**:
```
src-tauri/
├── src/
│   ├── commands/
│   │   ├── llm.rs          ← 调用 Unified LLM Trait
│   │   ├── provider.rs     ← Provider 管理
│   │   └── model.rs        ← 模型选择
│   └── plugins/
│       ├── chat.rs         ← 使用 Gateway Provider
│       ├── kb.rs           ← 使用 KB Facade
│       └── session.rs      ← 使用 Session Provider
```

**关键**: Tauri 插件通过 Facade 访问 L1 能力，不直接导入 L1 模块。

---

## 三、冗余清理实施 (Redundancy Cleanup)

### 3.1 Phase 1: 编译修复 (CRITICAL)

| 任务 | 文件 | 动作 |
|------|------|------|
| 修复 `sha256_hex` 跨域 | `coverage_ledger.rs` | 移动到 `nt_types` 或创建 `nt_io_utils` |
| 修复 `oMLX` 命名 | `adapter/nt_universal_provider/mod.rs:58` | 改为 `Omlix` |
| 修复 `EmotionState` 缺失方法 | `nt_core_self/emotion_state.rs` | 添加 `observe_appraisal`/`confidence_score`/`label` |
| 修复 `OrchestratorResult` 类型 | `orchestrator_v2.rs:581` | 统一类型 |
| 修复 `CapabilityRegistry` 字段 | `nt_core_capability/mod.rs:673` | 添加 `slots` 字段 |
| 移除未使用导入 | `coverage_ledger.rs` / `shield_core/mod.rs` | 删除 `DateTime`/`Sha256`/`DefenseVerdict` |

### 3.2 Phase 2: Trait 合并 (HIGH)

| 任务 | 涉及文件 | 动作 |
|------|---------|------|
| 合并 `AbsorbValidator` | 3 处 | 保留 `nt_core_absorb/mod.rs`，删除其他 2 处 |
| 合并 `KnowledgeProvider` | 3 处 (不同签名) | 统一签名，保留 `nt_core_knowledge/provider.rs` |
| 合并 `MemoryProvider` | 2 处 | 保留 `neotrix-types`，删除 `neotrix-core` 重复 |
| 合并 `RichMemoryProvider` | 2 处 | 同上 |
| 合并 `AgentExecutor` | 2 处 | 同上 |
| 合并 `SessionProvider` | 2 处 | 同上 |
| 合并 `BrainProvider` | 2 处 | 同上 |
| 合并 `EngineProvider` | 2 处 | 同上 |
| 删除 `KbProvider` stub | `proxy_pool.rs:22` | 删除死 Trait |
| 删除 `_BrowserSecurityCheck` | `browser_security.rs:102` | 删除死 Trait |

### 3.3 Phase 3: Registry 去重 (HIGH)

| 任务 | 涉及文件 | 动作 |
|------|---------|------|
| 合并 `SkillRegistry` | 5→1 | 保留 `nt_act/skill_registry/registry.rs` |
| 合并 `CapabilityRegistry` | 4→1 | 保留 `l7_capability/registry.rs` |
| 合并 `ProviderRegistry` | 3→1 | 保留 `nt_io_provider/catalog/registry.rs` |
| 合并 `ExperimentRegistry` | 2→1 | 保留 `nt_mind/experiment.rs` |
| 合并 `MetaCognitionBridge` | 2→1 | 保留 `nt_mind/distillation.rs` |

### 3.4 Phase 4: 死代码清理 (MEDIUM)

| 任务 | 动作 |
|------|------|
| 删除 14 个死模块 | L2×7 + L3×7 (注释掉的) |
| 移除全局 `#![allow(dead_code)]` | 改为模块级 `#[allow(dead_code)]` 仅在必要处 |
| 清理 100+ `#[allow(dead_code)]` | 逐个评估，删除不必要的 |
| 移除 `ureq` workspace 依赖 | 仅 `onnx` feature 需要，移出 workspace |
| 评估 15+ 可疑依赖 | `flate2`/`tar`/`encoding_rs`/`noyalib`/`zim` 等 |

### 3.5 Phase 5: HTTP/SQLite 统一 (MEDIUM)

| 任务 | 动作 |
|------|------|
| 创建共享 HTTP 客户端工厂 | `nt_io_http::HttpClientFactory` |
| OSINT 模块使用共享客户端 | 16 个模块改为注入客户端 |
| 创建 SQLite 连接池 | `nt_memory_kb::ConnectionPool` |
| 统一 `open_kb()` 函数 | 合并 3 个独立实现为 1 个 |

---

## 四、跨域错位修复 (Cross-Domain Alignment)

### 4.1 层级导入规则

```
允许的导入方向:
  L0 ← L1 ← L2 ← L3 ← L4 ← L5 ← L6
  (上层可以导入下层的 traits，不能导入实现)

禁止的导入方向:
  L1 ← L5 (通过 Facade 也不行)
  L1 ← L6 (通过 Facade 也不行)
  L3 ← L1 (私有函数不能跨层)
```

### 4.2 修复清单

| 违规 | 文件 | 修复方案 |
|------|------|---------|
| L5→L1 `nt_io_provider` | `nt_mind_benchmark.rs` | 通过 `LlmProvider` trait 访问 |
| L5→L1 `nt_io_provider` | `engine_core.rs` | 通过 `LlmProvider` trait 访问 |
| L5→L1 `create_gateway` | `seal_loop.rs` | 通过 `GatewayFactory` trait 访问 |
| L5→L1 `nt_act_types` | `goal_generator.rs` | 通过 `nt_types` 共享类型 |
| L5→L1 `nt_io_eli5` | `nt_io_eli5.rs` | 移动 ELI5 到 L2 或 L5 |
| L5→L1 `nt_memory_crawl` | `knowledge_pipeline.rs` | 通过 KB trait 访问 |
| L6→L1 `KnowledgeBase` | `nt_nexus/mod.rs` | 通过 KB trait 访问 |
| L6→L1 `nt_act_cleanup` | `l1_facade.rs` | 通过 Cleanup trait 访问 |
| L6→L1 `nt_io_provider` | `eval_harness.rs` | 通过 `LlmProvider` trait 访问 |
| L6→L5 `awareness_monitor` | `arch_optimizer.rs` | 通过 Meta trait 访问 |
| L6→L5 `awareness_monitor` | `consciousness_monitor.rs` | 通过 Meta trait 访问 |
| L1→L3 `sha256_hex` | `coverage_ledger.rs` | 移动到 `nt_types` |

### 4.3 Facade 重构

**现状**: 7 个 Facade 允许上层直接访问下层实现
**目标**: Facade 只暴露 traits，不暴露实现

```rust
// ❌ BEFORE: Facade 暴露实现
pub use crate::l1_action::nt_memory::nt_memory_kb::*;

// ✅ AFTER: Facade 只暴露 traits
pub use crate::core::l3_memory::nt_core_kb::{KnowledgeBaseTrait, KbQuery, KbResult};
```

---

## 五、全量评测 (Full-Scale Evaluation)

### 5.1 评测维度

| 维度 | 工具 | 通过标准 |
|------|------|---------|
| **编译** | `cargo check --all-targets -p neotrix` | 0 errors, 0 warnings |
| **单元测试** | `cargo test -p neotrix --lib` | ≥95% 通过 |
| **集成测试** | `cargo test -p neotrix` | ≥90% 通过 |
| **架构合规** | 自定义 lint (层级导入检查) | 0 违规 |
| **安全** | `cargo audit` | 0 critical/high |
| **死代码** | `cargo machete` | ≤20 未使用依赖 |
| **性能** | benchmarks | 无回归 |

### 5.2 评测执行计划

```bash
# Phase 1: 编译修复后
cargo clean && cargo check --all-targets -p neotrix 2>&1 | grep -c "error"
# 目标: 0

# Phase 2: Trait 合并后
cargo test -p neotrix --lib 2>&1 | tail -5
# 目标: test result: ok. X passed; Y failed

# Phase 3: 全量测试
cargo test -p neotrix 2>&1 | tail -10
# 目标: test result: ok. X passed; Y failed

# Phase 4: 架构合规
cargo check -p neotrix --lib 2>&1 | grep -c "warning"
# 目标: 0

# Phase 5: 安全审计
cargo audit 2>&1 | grep -c "critical\|high"
# 目标: 0
```

---

## 六、核心路线任务清单 (Core Roadmap)

### 🔴 Phase 0: 紧急修复 (Day 1-2)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P0-1 | 修复 `sha256_hex` 跨域引用 | CRITICAL | 1h | 无 |
| P0-2 | 修复 `oMLX` 命名违规 | CRITICAL | 10min | 无 |
| P0-3 | 修复 `EmotionState` 缺失方法 | HIGH | 2h | 无 |
| P0-4 | 修复 `OrchestratorResult` 类型不匹配 | HIGH | 1h | 无 |
| P0-5 | 修复 `CapabilityRegistry` 缺失字段 | HIGH | 30min | 无 |
| P0-6 | 移除未使用导入 | MEDIUM | 30min | 无 |
| P0-7 | 验证编译通过 | CRITICAL | 1h | P0-1~6 |

### 🟠 Phase 1: Trait 合并 (Day 3-5)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P1-1 | 合并 `AbsorbValidator` (3→1) | HIGH | 2h | P0-7 |
| P1-2 | 统一 `KnowledgeProvider` 签名 | HIGH | 3h | P0-7 |
| P1-3 | 合并 `MemoryProvider`/`RichMemoryProvider` | HIGH | 2h | P0-7 |
| P1-4 | 合并 `AgentExecutor`/`SessionProvider` | HIGH | 2h | P0-7 |
| P1-5 | 合并 `BrainProvider`/`EngineProvider` | HIGH | 2h | P0-7 |
| P1-6 | 删除死 Trait (`KbProvider`/`_BrowserSecurityCheck`) | MEDIUM | 1h | P0-7 |
| P1-7 | 验证测试通过 | CRITICAL | 2h | P1-1~6 |

### 🟠 Phase 2: Registry 去重 (Day 6-8)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P2-1 | 合并 `SkillRegistry` (5→1) | HIGH | 4h | P1-7 |
| P2-2 | 合并 `CapabilityRegistry` (4→1) | HIGH | 4h | P1-7 |
| P2-3 | 合并 `ProviderRegistry` (3→1) | HIGH | 3h | P1-7 |
| P2-4 | 合并 `ExperimentRegistry` (2→1) | MEDIUM | 2h | P1-7 |
| P2-5 | 合并 `MetaCognitionBridge` (2→1) | MEDIUM | 2h | P1-7 |
| P2-6 | 验证测试通过 | CRITICAL | 2h | P2-1~5 |

### 🟡 Phase 3: 跨域修复 (Day 9-11)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P3-1 | 创建 `LlmProvider` 访问层 (替代直接导入) | HIGH | 4h | P2-6 |
| P3-2 | 创建 `KnowledgeBase` trait 访问层 | HIGH | 3h | P2-6 |
| P3-3 | 创建 `GatewayFactory` trait | HIGH | 3h | P2-6 |
| P3-4 | 修复 L5→L1 直接导入 (6+ 处) | HIGH | 4h | P3-1~3 |
| P3-5 | 修复 L6→L1 直接导入 (4+ 处) | HIGH | 3h | P3-1~3 |
| P3-6 | 修复 L6→L5 直接导入 (2+ 处) | MEDIUM | 2h | P3-1~3 |
| P3-7 | 重构 Facade (只暴露 traits) | MEDIUM | 4h | P3-4~6 |
| P3-8 | 验证测试通过 | CRITICAL | 2h | P3-1~7 |

### 🟡 Phase 4: 基础设施统一 (Day 12-14)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P4-1 | 创建共享 HTTP 客户端工厂 | MEDIUM | 4h | P3-8 |
| P4-2 | OSINT 模块迁移到共享客户端 | MEDIUM | 4h | P4-1 |
| P4-3 | 创建 SQLite 连接池 | MEDIUM | 4h | P3-8 |
| P4-4 | 统一 `open_kb()` 函数 | MEDIUM | 3h | P4-3 |
| P4-5 | 删除 14 个死模块 | LOW | 2h | P3-8 |
| P4-6 | 清理 100+ `#[allow(dead_code)]` | LOW | 4h | P3-8 |
| P4-7 | 评估并移除可疑依赖 | LOW | 3h | P3-8 |

### 🟢 Phase 5: 通用模型适配层 (Day 15-18)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P5-1 | 设计 Universal Model Adapter 接口 | HIGH | 4h | P4-7 |
| P5-2 | 实现 OpenAI Adapter | HIGH | 3h | P5-1 |
| P5-3 | 实现 Anthropic Adapter | HIGH | 3h | P5-1 |
| P5-4 | 实现 Gemini Adapter | HIGH | 3h | P5-1 |
| P5-5 | 实现 Ollama Adapter | HIGH | 3h | P5-1 |
| P5-6 | 实现 DeepSeek Adapter | HIGH | 3h | P5-1 |
| P5-7 | 集成 Gateway (路由+韧性) | HIGH | 4h | P5-2~6 |
| P5-8 | 集成 Egress Privacy Guard | HIGH | 2h | P5-7 |
| P5-9 | Tauri 桌面 App 集成 | HIGH | 4h | P5-7 |
| P5-10 | 端到端测试 | CRITICAL | 4h | P5-9 |

### 🔵 Phase 6: 意识晶体集成 (Day 19-22)

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| P6-1 | Crystal Core 状态机 (E8+GWT) | HIGH | 5h | P5-10 |
| P6-2 | CTM Up-Tree/Down-Tree 实现 | HIGH | 4h | P6-1 |
| P6-3 | IIT Φ 实时监控 | HIGH | 3h | P6-1 |
| P6-4 | VSA 壳层 + KB 集成 | HIGH | 4h | P6-2 |
| P6-5 | 情感调制意识状态 | MEDIUM | 3h | P6-3 |
| P6-6 | SEAL 自进化循环 | HIGH | 4h | P6-4 |
| P6-7 | 端到端意识循环测试 | CRITICAL | 4h | P6-6 |

---

## 七、多 Agent 自动巡检修复 (Multi-Agent Inspection)

### 7.1 巡检架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Inspection Orchestrator                    │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Compile  │  │  Test    │  │ Arch     │  │ Security │   │
│  │ Agent    │  │  Agent   │  │ Agent    │  │ Agent    │   │
│  │ (编译)   │  │ (测试)   │  │ (架构)   │  │ (安全)   │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │              │              │              │          │
│  ┌────▼──────────────▼──────────────▼──────────────▼────┐   │
│  │              Auto-Repair Engine                        │   │
│  │  - 修复编译错误                                        │   │
│  │  - 修复测试失败                                        │   │
│  │  - 修复架构违规                                        │   │
│  │  - 修复安全问题                                        │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Knowledge Base (经验库)                   │   │
│  │  - 历史修复记录                                        │   │
│  │  - 模式匹配库                                          │   │
│  │  - 最佳实践库                                          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 巡检 Agent 清单

| Agent | 职责 | 巡检频率 | 自动修复 |
|-------|------|---------|---------|
| **CompileAgent** | `cargo check` 编译检查 | 每次提交 | ✅ 自动修复简单错误 |
| **TestAgent** | `cargo test` 测试通过 | 每次提交 | ⚠️ 报告失败，人工修复 |
| **ArchAgent** | 层级导入检查 | 每次 PR | ✅ 自动移动导入 |
| **DeadCodeAgent** | 死代码检测 | 每周 | ✅ 自动删除确认死代码 |
| **SecurityAgent** | `cargo audit` 安全审计 | 每周 | ⚠️ 报告问题，人工修复 |
| **PerfAgent** | Benchmark 回归检测 | 每周 | ⚠️ 报告回归，人工修复 |
| **DependencyAgent** | 依赖健康检查 | 每月 | ⚠️ 报告过期依赖 |

### 7.3 巡检脚本

```bash
#!/bin/bash
# scripts/inspect.sh — 全量巡检

echo "=== Phase 1: 编译检查 ==="
cargo check --all-targets -p neotrix 2>&1 | grep -c "error"
ERRORS=$?
if [ $ERRORS -gt 0 ]; then
    echo "🔴 编译错误: $ERRORS"
    # Auto-fix: 尝试简单修复
    cargo fix --allow-dirty -p neotrix 2>&1
fi

echo "=== Phase 2: 测试检查 ==="
cargo test -p neotrix --lib 2>&1 | tail -5

echo "=== Phase 3: 架构合规 ==="
# 检查层级导入违规
grep -rn "use crate::l1_action" neotrix-core/src/l5_cognition/ | wc -l
grep -rn "use crate::l1_action" neotrix-core/src/l6_meta/ | wc -l

echo "=== Phase 4: 死代码检查 ==="
cargo machete 2>&1 | grep -c "Unused dependencies"

echo "=== Phase 5: 安全审计 ==="
cargo audit 2>&1 | grep -c "critical\|high"

echo "=== 巡检完成 ==="
```

---

## 八、实施方案：文档章节完善

### 8.1 Consciousness Crystal 实施章节

| 章节 | 状态 | 实施内容 |
|------|------|---------|
| 一、概述 | ✅ 完成 | 问题定义 + 4 个关键洞察 |
| 二、杰文斯悖论 | ✅ 完成 | 效率反噬的扩张逻辑 |
| 三、核心洞察 | ✅ 完成 | ZGCM-1/CTM/MCT/RIIU/GWA |
| 四、Crystal 架构 | ✅ 完成 | 六面体 + 三层同心 |
| 五、核心算法 | ✅ 完成 | 意识 Tick + E8 状态机 |
| 六、CTM 映射 | ✅ 完成 | 7 元组精确映射 |
| 七、ZGCM-1 融合 | ✅ 完成 | 渐进式扩展 + MDP |
| 八、实现路径 | ⚠️ 需细化 | 添加具体代码示例 |
| 九、核心公式 | ✅ 完成 | CII/CI/CB |
| 十、兼容性 | ✅ 完成 | 现有组件映射 |
| 十一、果蝇连接组 | ✅ 完成 | Rich-club/Integrators/Broadcasters |
| 十二、参考文献 | ✅ 完成 | 17 篇 |

### 8.2 全局重构实施方案

| 章节 | 状态 | 实施内容 |
|------|------|---------|
| 一、诊断总览 | ✅ 完成 | 三维缺陷矩阵 |
| 二、通用模型适配层 | ✅ 完成 | 统一 Trait + Provider 清单 |
| 三、冗余清理 | ✅ 完成 | 5 Phase 实施清单 |
| 四、跨域修复 | ✅ 完成 | 层级规则 + 修复清单 |
| 五、全量评测 | ✅ 完成 | 7 维度评测 |
| 六、核心路线 | ✅ 完成 | 6 Phase × 40+ 任务 |
| 七、多 Agent 巡检 | ✅ 完成 | 7 Agent + 巡检脚本 |
| 八、章节完善 | ✅ 完成 | 本节 |

---

## 九、风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Trait 合并导致类型不兼容 | 高 | 高 | 渐进式合并，每步验证测试 |
| 跨域修复引入新 bug | 中 | 高 | 每个修复独立 PR，充分测试 |
| 死代码删除影响隐藏依赖 | 中 | 中 | 先 `#[allow(dead_code)]` 标记，观察 1 周 |
| 性能回归 | 低 | 高 | Benchmark 回归检测 |
| 依赖移除导致功能缺失 | 低 | 中 | `cargo machete` + 手动验证 |

---

## 十、成功标准

| 指标 | 当前 | 目标 |
|------|------|------|
| 编译错误 | 10 | 0 |
| 编译警告 | 50+ | 0 |
| 测试通过率 | ~90% | ≥95% |
| LLM 抽象层数 | 4 | 1 |
| SkillRegistry 数量 | 5 | 1 |
| CapabilityRegistry 数量 | 4 | 1 |
| 跨层违规 | 12+ | 0 |
| 死模块 | 14 | 0 |
| `#[allow(dead_code)]` | 100+ | ≤20 |
| 未使用依赖 | 15+ | ≤5 |
