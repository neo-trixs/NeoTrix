# NeoTrix 意识核心 — 后续任务清单

## 📋 当前状态总结

| 项目 | 状态 | 详情 |
|------|------|------|
| 吸收报告 | ✅ 完成 | galaxy-tree + 70+ URL |
| 任务设计 | ✅ 完成 | 12周6阶段计划 |
| 代码实现 | ✅ 完成 | 46文件, 19,328行 |
| 新模块编译 | ✅ 通过 | 0 errors |
| 全量编译 | ❌ 阻塞 | ~66 pre-existing errors |
| 单元测试 | ⏳ 待运行 | 需先修复全量编译 |

---

## 🔴 Phase A: 修复预存编译错误 (阻塞项)

**目标**: 使 `cargo check -p neotrix --lib` 通过
**优先级**: P0 (阻塞后续所有任务)
**预计工时**: 2-3天

### A1: nt_act_trade 模块错误 (主要来源)

| # | 文件 | 错误类型 | 描述 | 修复方案 |
|---|------|----------|------|----------|
| A1.1 | `nt_mind/mod.rs:202` | E0432 | 未解析导入 `LogisticsDocSet` 等7个类型 | 改为从子模块导入: `full_cycle::LogisticsDocSet` |
| A1.2 | `nt_mind/mod.rs:245` | E0432 | 未解析导入 `TradePhase`, `TradeContext` 等 | 改为从 `nt_act_trade::TradePhase` 导入 |
| A1.3 | `full_cycle.rs:335` | E0425 | 找不到类型 `PaymentProof` | 添加 `use finance_compliance::PaymentProof` |
| A1.4 | `orchestrator.rs:37` | unused import | `CostBreakdown` 未使用 | 删除未使用导入 |
| A1.5 | `finance_compliance.rs:427` | E0609 | `Discrepancy` 无 `description` 字段 | 改为正确的字段名 |
| A1.6 | `full_cycle.rs:413` | E0277 | `StateMachine<TradePhase>` 未实现 `Default` | 为 `StateMachine` 实现 `Default` |
| A1.7 | `trade_core.rs:116` | E0277 | `dyn CostComponent` 未实现 `Debug`/`Clone` | 添加 `#[derive(Debug)]` 手动实现或使用 `dyn ... + Send + Sync` |
| A1.8 | `trade_core.rs:418` | E0277 | `dyn RiskRule` 未实现 `Debug`/`Clone` | 同上 |
| A1.9 | `production_logistics.rs:1116` | E0609 | `ProductionSchedule` 无 `milestones` 字段 | 改为正确的字段名或添加字段 |

### A2: code_writer 模块错误

| # | 文件 | 错误类型 | 描述 | 修复方案 |
|---|------|----------|------|----------|
| A2.1 | `code_writer.rs:483` | E0559 | `ActionPlan::HumanDecision` 无 `file` 字段 | 删除 `file: None` 或添加字段 |

### A3: performance 模块错误

| # | 文件 | 错误类型 | 描述 | 修复方案 |
|---|------|----------|------|----------|
| A3.1 | `performance.rs:67` | E0277 | `T` 未实现 `Clone` | 添加 `T: Clone` 约束 |
| A3.2 | `performance.rs:73` | E0599 | `T` 无 `clone` 方法 | 同上 |

### A4: 其他模块错误

| # | 文件 | 错误类型 | 描述 | 修复方案 |
|---|------|----------|------|----------|
| A4.1 | `asset_map_capability.rs` | unused import | 未使用的导入 | 删除 |
| A4.2 | 其他文件 | 各类 | 待全量编译后统计 | 逐个修复 |

---

## 🟡 Phase B: 运行并修复单元测试

**目标**: 所有新增模块测试通过
**优先级**: P0
**前置**: Phase A 完成
**预计工时**: 1-2天

### B1: 意识核心模块测试

| # | 模块 | 测试数 | 状态 | 修复方案 |
|---|------|--------|------|----------|
| B1.1 | memory_kernel | ~8 | 待运行 | — |
| B1.2 | cas_store | ~6 | 待运行 | — |
| B1.3 | bitemporal_graph | ~5 | 待运行 | — |
| B1.4 | belief_anchor | ~6 | 待运行 | — |
| B1.5 | constitution | ~8 | 待运行 | — |
| B1.6 | skill_crystallizer | ~6 | 待运行 | — |
| B1.7 | evolution_genome | ~8 | 待运行 | — |
| B1.8 | learnable_skill | ~7 | 待运行 | — |
| B1.9 | guardrail_pipeline | ~6 | 待运行 | — |
| B1.10 | credential_sentinel | ~5 | 待运行 | — |
| B1.11 | approval_gate | ~6 | 待运行 | — |
| B1.12 | sandbox_isolator | ~5 | 待运行 | — |
| B1.13 | mscf_scorer | ~8 | 待运行 | — |
| B1.14 | consciousness_metrics | ~6 | 待运行 | — |
| B1.15 | pivot_tournament | ~8 | 待运行 | — |
| B1.16 | graph_community | ~8 | 待运行 | — |
| B1.17 | persistent_kv | ~6 | 待运行 | — |
| B1.18 | paged_attention | ~5 | 待运行 | — |
| B1.19 | mla_attention | ~5 | 待运行 | — |
| B1.20 | cost_router | ~6 | 待运行 | — |

### B2: 原有模块测试

| # | 模块 | 测试数 | 状态 | 修复方案 |
|---|------|--------|------|----------|
| B2.1 | agent | ~5 | 待运行 | — |
| B2.2 | probes | ~4 | 待运行 | — |
| B2.3 | patches | ~4 | 待运行 | — |
| B2.4 | convergence | ~3 | 待运行 | — |
| B2.5 | self_observer | ~4 | 待运行 | — |
| B2.6 | self_evolver | ~4 | 待运行 | — |
| B2.7 | emergence_engine | ~3 | 待运行 | — |

---

## 🟢 Phase C: 系统集成

**目标**: 意识核心与现有 NeoTrix 系统集成
**优先级**: P1
**前置**: Phase A + B 完成
**预计工时**: 3-5天

### C1: KB 集成

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| C1.1 | MemoryKernel → KB | 将 MemoryKernel 接入现有 SQLite KB | 1天 |
| C1.2 | BitemporalGraph → KB | 双时态事实存入 KB | 1天 |
| C1.3 | CAS Store → KB | 内容寻址存储与 KB 同步 | 0.5天 |
| C1.4 | SkillRegistry → KB | 技能注册表持久化到 KB | 0.5天 |

### C2: EventBus 集成

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| C2.1 | 意识指标事件 | MSCF 评分变化发布到 EventBus | 0.5天 |
| C2.2 | 进化事件 | 进化变异/拒绝事件发布 | 0.5天 |
| C2.3 | 安全事件 | 护栏拦截/审批事件发布 | 0.5天 |
| C2.4 | 技能事件 | 技能结晶/注册事件发布 | 0.5天 |

### C3: CLI 集成

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| C3.1 | `nt consciousness status` | 显示意识核心状态 | 0.5天 |
| C3.2 | `nt consciousness evolve` | 手动触发进化周期 | 0.5天 |
| C3.3 | `nt consciousness metrics` | 显示意识指标历史 | 0.5天 |
| C3.4 | `nt skills list` | 列出已结晶技能 | 0.5天 |

### C4: SelfTest 集成

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| C4.1 | T1 存在性测试 | 每个模块实现 `SelfTest` trait | 1天 |
| C4.2 | T2 注册测试 | 在 run.rs + pipeline.rs 注册 | 0.5天 |
| C4.3 | T3 生产接线测试 | 验证检测函数被非测试代码调用 | 1天 |

---

## 🔵 Phase D: 性能优化

**目标**: 意识核心性能达标
**优先级**: P1
**前置**: Phase C 完成
**预计工时**: 2-3天

### D1: 基准测试

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| D1.1 | MemoryKernel 基准 | 读写延迟、吞吐量 | 0.5天 |
| D1.2 | CAS Store 基准 | 内容寻址查找速度 | 0.5天 |
| D1.3 | BitemporalGraph 基准 | 时序查询延迟 | 0.5天 |
| D1.4 | PersistentKV 基准 | 三级缓存命中率 | 0.5天 |

### D2: 优化实施

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| D2.1 | 热点路径优化 | 基准测试后的热点优化 | 1天 |
| D2.2 | 内存优化 | 减少不必要的 allocation | 0.5天 |
| D2.3 | 并发优化 | 关键路径并行化 | 0.5天 |

---

## 🟣 Phase E: 文档与部署

**目标**: 完善文档并准备部署
**优先级**: P2
**前置**: Phase D 完成
**预计工时**: 2-3天

### E1: API 文档

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| E1.1 | rustdoc 注释 | 为所有公共 API 添加文档 | 1天 |
| E1.2 | 架构图更新 | 更新 `docs/` 架构图 | 0.5天 |
| E1.3 | 使用示例 | 为关键模块添加 examples | 0.5天 |

### E2: 部署准备

| # | 任务 | 描述 | 预计工时 |
|---|------|------|----------|
| E2.1 | Cargo.toml 更新 | 确认所有依赖正确 | 0.5天 |
| E2.2 | Feature flags | 为可选模块添加 feature flags | 0.5天 |
| E2.3 | Release 构建 | `cargo build --release` 验证 | 0.5天 |

---

## 📊 任务统计

| Phase | 任务数 | 预计工时 | 优先级 | 状态 |
|-------|--------|----------|--------|------|
| A: 修复预存错误 | ~12 | 2-3天 | P0 | 🔴 待开始 |
| B: 单元测试 | ~27 | 1-2天 | P0 | 🔴 待开始 |
| C: 系统集成 | ~13 | 3-5天 | P1 | 🟡 待开始 |
| D: 性能优化 | ~7 | 2-3天 | P1 | 🟡 待开始 |
| E: 文档部署 | ~6 | 2-3天 | P2 | 🟢 待开始 |
| **总计** | **~65** | **10-16天** | | |

---

## 🎯 执行顺序

```
Phase A (修复编译) ──→ Phase B (测试) ──→ Phase C (集成) ──→ Phase D (优化) ──→ Phase E (文档)
     │                    │                    │                    │                    │
     └─ 2-3天             └─ 1-2天             └─ 3-5天             └─ 2-3天             └─ 2-3天
```

---

## ⚡ 快速启动命令

```bash
# Phase A: 修复编译
cargo check -p neotrix --lib 2>&1 | grep "^error" | wc -l  # 统计错误数
cargo check -p neotrix --lib 2>&1 | head -100               # 查看前100个错误

# Phase B: 运行测试
cargo test -p neotrix --lib -- nt_consciousness_core

# Phase C: 集成验证
cargo check -p neotrix --all-targets

# Phase D: 基准测试
cargo test -p neotrix --lib -- nt_consciousness_core --bench

# Phase E: 文档构建
cargo doc -p neotrix --open
```

---

**文档版本**: v1.0
**创建时间**: 2026-09-09
**状态**: 可执行
