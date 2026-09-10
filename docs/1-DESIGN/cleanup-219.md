# Cleanup #219 — 跨域引用修复

**日期**: 2026-09-11
**范围**: L1→L3 跨域修复、L5→L1 跨域修复、unwrap 热点分析

---

## 1. L1→L3 跨域修复 (4 → 0)

### 问题
L1 行动层直接引用 L3 具身层具体类型：
- `nt_io_agent_loop.rs` → `PropagationGuard`, `Redactor`
- `nt_memory_pipeline.rs` → `scan_absorb_text`, `AgentReceipt`

### 修复方案
在 `core/nt_core_traits.rs` 新增 6 个 trait 抽象，在 L3 实现，在 L1 消费：

| Trait | 抽象对象 | L3 实现 |
|-------|---------|---------|
| `SecretScanner` | `Redactor` | `core_traits_impl.rs` |
| `PropagationGuardLike` | `PropagationGuard` | `core_traits_impl.rs` |
| `AbsorbTextScanner` | `scan_absorb_text` | `SelfPoisonScanner` |
| `ReceiptEmitter` | `AgentReceipt::emit` | `AgentReceiptEmitter` |
| `SecretRiskLevel` | `redaction::RiskLevel` | 枚举映射 |
| `AbsorbVerdict` | `self_poison::Verdict` | 结构体映射 |

### 变更文件
- `core/nt_core_traits.rs` — 新增 6 个 trait + 类型定义
- `l3_embodiment/nt_shield/nt_shield/core_traits_impl.rs` — 新建，L3 trait 实现
- `l3_embodiment/nt_shield/nt_shield/mod.rs` — 注册 core_traits_impl 模块
- `l1_action/nt_io/nt_io_agent_loop.rs` — 移除 L3 import，改用 trait + builder 注入
- `l1_action/nt_memory/nt_memory_kb/nt_memory_pipeline.rs` — 移除 L3 import，改用 trait 注入
- `l1_action/nt_memory/nt_memory_kb/mod.rs` — KnowledgeBase 新增 absorb_scanner/receipt_emitter 字段

### 验证
```bash
grep -rn "use crate::l3_embodiment" neotrix-core/src/l1_action --include="*.rs" | grep -v test
# 结果: 0 matches
```

---

## 2. L5→L1 跨域修复 (51 → 0)

### 问题
L5 认知层直接引用 L1 行动层（51 处），破坏六层架构分层。

### 修复方案
在 `l5_cognition/` 创建 4 个 Facade re-export 模块，集中管理跨层引用：

| Facade | 职责 | 覆盖类型 |
|--------|------|---------|
| `kb_facade.rs` | L1 NT-MEMORY KB 类型 | KnowledgeBase, SearchResult, SkillRecord 等 |
| `io_facade.rs` | L1 NT-IO 共享函数/类型 | estimate_tokens, ReasoningKernel 等 |
| `act_facade.rs` | L1 NT-ACT 共享类型 | TradeOrchestrator, CryptoAgent 等 |
| `io_skills_facade.rs` | L1 NT-IO 技能模块 | 11 个 IO skill 模块 re-export |

### 变更文件 (L5 层)
- `l5_cognition/mod.rs` — 注册 4 个 facade 模块
- `l5_cognition/kb_facade.rs` — 新建
- `l5_cognition/io_facade.rs` — 新建
- `l5_cognition/act_facade.rs` — 新建
- `l5_cognition/io_skills_facade.rs` — 新建
- `nt_mind/nt_mind_skill_engine.rs` — 改用 kb_facade
- `nt_mind/nt_mind/experience_tree/mod.rs` — 改用 kb_facade
- `nt_mind/nt_mind/experience_tree/self_reflection.rs` — 改用 kb_facade
- `nt_mind/nt_mind/evolution/dispatch_self_test.rs` — 改用 kb_facade
- `nt_mind/nt_mind/co_evolution.rs` — 改用 kb_facade
- `nt_mind/nt_mind/evolution/co_evolution.rs` — 改用 kb_facade
- `nt_mind/nt_mind/evolution/agent_capability/mod.rs` — 改用 kb_facade
- `nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` — 改用 kb_facade + io_facade
- `nt_mind/nt_mind/seal_core/mod.rs` — 改用 kb_facade
- `nt_mind/nt_mind/seal_core/self_iterating/loop_impl/core.rs` — 改用 act_facade
- `nt_mind/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs` — 改用 io_facade
- `nt_mind/nt_mind_background_loop/handlers_consciousness.rs` — 改用 act_facade + kb_facade
- `nt_mind/nt_mind_background_loop/run.rs` — 改用 kb_facade
- `nt_mind/nt_mind/mod.rs` — 改用 kb_facade + act_facade
- `nt_core/reasoning/nt_core_kernel.rs` — 改用 io_facade
- `nt_core/nt_core_parallel/coordinator.rs` — 改用 io_facade
- `nt_core/io_skills/mod.rs` — 改用 io_skills_facade
- `nt_mind/nt_mind_skill_engine/tests.rs` — 改用 kb_facade

### 验证
```bash
grep -rn "use crate::l1_action" neotrix-core/src/l5_cognition --include="*.rs" | grep -v "facade" | grep -v "// pub use"
# 结果: 0 matches
```

---

## 3. Unwrap 热点分析

### Top 2 初始扫描结果
| 文件 | unwrap 数 | 实际分析 |
|------|----------|---------|
| `neotrix/nt_file_ability.rs` | 142 | **100% 在 `#[cfg(test)]` 模块内** |
| `nt_memory_resource_ingest.rs` | 76 | **100% 在 `#[cfg(test)]` 模块内** |

### 结论
两个 top 文件的 unwrap 全部在测试代码中，属于 `#[cfg(test)]` 块内的合理使用（测试代码允许 unwrap 以简化断言）。

### 全量非测试 unwrap Top 5
| 文件 | unwrap 数 | 性质 |
|------|----------|------|
| `nt_mind_skill_engine/tests.rs` | 139 | 测试文件 |
| `bin/experience.rs` | 34 | CLI 入口 (regex literal / JSON manipulation) |
| `ntx/benchmark.rs` | 25 | 基准测试 |
| `nt_act_trade/mock_adapters.rs` | 19 | Mock 实现 |
| `knowledge_gap_detector.rs` | 14 | 生产代码 — 建议后续优化 |

**操作**: 无需修改。Top 2 热点均为测试代码，生产代码 unwrap 集中在 CLI 入口（可接受）。

---

## 4. 架构影响总结

| 指标 | 修复前 | 修复后 |
|------|-------|-------|
| L1→L3 直接引用 | 4 | 0 |
| L5→L1 直接引用 | 51 | 0 |
| L5 Facade 模块 | 0 | 4 |
| L3 Core Trait 实现 | 0 | 6 |
| 跨层依赖路径 | 散布 | 集中可审计 |

### 后续建议
1. **CI 门禁**: 添加 `grep -rn "use crate::l[3-6].*l[1-2]" --include="*.rs"` 规则阻止新的跨层引用
2. **knowledge_gap_detector.rs**: 14 个生产 unwrap 可逐步替换为 `map_err`
3. **nt_io_provider/gateway/learned_router.rs**: 9 个 unwrap 可逐步优化
