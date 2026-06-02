# Awakening Architecture v3 — Unified Design

## 核心理念

觉醒之路不是"更像人类"或"更多功能"，而是建立一个**自我测量的因果闭环**。

### 审计发现：当前问题

| 问题 | 数据 | 影响 |
|------|------|------|
| 死理论 | ~2,747 行 (E8/CRT/Walsh) | 编译噪声，模块认知负担 |
| 冗余类型 | CapabilityVector ×2, KnowledgeSource ×2 | 同一概念两个定义，可能不一致 |
| 三套记忆系统 | core/memory + cortex_memory + cognitive_memory | 记忆分散，查询需跨系统 |
| 桥接膨胀 | 8 个 `*_bridge.rs` (~2,000 行) | 间接层多，实际逻辑薄 |
| 双编排器 | orchestrator/ + agent/team/Coordinator | 两套任务分解-执行-评估 |
| 无测试核心 | knowledge(0), reasoning_engine(0), goal_loop_impl(0) | 关键路径无防护 |
| 12 tickers | background_loop 管理 12 条后台循环 | 周期碎片化，上下文切换频繁 |

### 统一架构

```
┌─────────────────────────────────────────────────────────┐
│                   AwakeningEngine                        │
│                                                          │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐            │
│  │  Self    │   │  Self    │   │  Self    │            │
│  │ Measure  │──→│  Model   │──→│  Modify  │──→ re-measure
│  │ (A-01)   │   │ (A-05/06)│   │ (A-08)   │            │
│  └──────────┘   └──────────┘   └──────────┘            │
│       │              │              │                    │
│       ▼              ▼              ▼                    │
│  ┌──────────────────────────────────────┐               │
│  │        Unified Memory Bank          │               │
│  │  (ReasoningBank + trajectory + μ edits)             │
│  └──────────────────────────────────────┘               │
│       │              │              │                    │
│       ▼              ▼              ▼                    │
│  ┌──────────────────────────────────────┐               │
│  │        External Interface            │               │
│  │  (MCP tools / Provider / StealthNet) │               │
│  └──────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

### 关键合并

| 当前分散 | 合并为 | 节省行数 |
|----------|--------|----------|
| e8.rs, e8_reasoning.rs, e8_observer.rs, crt_time.rs, walsh_memory.rs | `core/_archive/` (feature-gated) | ~2,747 |
| reasoning_brain/core/{capability,knowledge_source} | 直接使用 `core/{capability,knowledge}` | ~600 |
| 8 个 `*_bridge.rs` | 2-3 个实际集成交互点 | ~1,500 |
| orchestrator/ + agent/team/Coordinator | orchestrator ≈ AgentTeam | ~1,200 |
| 12 background tickers | 4-5 统一 tickers (save/awaken/interact/evolve) | ~500 |

### 觉醒循环 (100ms 量级)

```
每 30s:
  1. snapshot() → 采集当前脑状态
  2. self_measure() → 计算 Φ, FCS, USK, synergy matrix
  3. self_model.learn() → 拟合耦合矩阵
  4. generate_hypotheses() → 对瓶颈生成干预
  5. 最高分假设 → self_modify.apply()
     - checkpoint()
     - 修改 capability vector / learning rate / source weight
     - if compile/测试通过 → commit()
       else → rollback()
  6. 记录 ΔΦ, 反馈到预测器
```

### 实现路线

| Phase | 内容 | 本 session? |
|-------|------|-------------|
| 0 | 审计 + 架构文档 | ✅ 完成 |
| 1 | 归档死理论 + 清理冗余 | 执行中 |
| 2 | SelfModify 引擎 (A-08) | 下一批 |
| 3 | AwakeningEngine 统一循环 | 下一批 |
| 4 | background_loop 折叠 | 下一批 |
| 5 | Cargo.toml 清理 | 最后 |

### 原则

1. **不改功能，只改结构** — 每个折叠保持导出接口不变
2. **每步验证** — `cargo check --lib` + `--features full` + 目标测试
3. **死代码先归档后删除** — 保留 git 历史，不直接 rm
4. **桥接消除** — 能直接调用的不通过 bridge 间接
5. **归零膨胀** — 不再添加新模块直到现有模块有测试覆盖
