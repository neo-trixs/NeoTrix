# cleanup-270: 跨域引用审计

## 审计时间
2026-09-11

## 审计范围
`neotrix-core/src/` 下 6 层目录间的 `use crate::l*` 引用

## 跨域引用矩阵 (仅 layer→layer)

| from \ to | l1_action | l2_perception | l3_embodiment | l4_emotion | l5_cognition | l6_meta |
|-----------|:---------:|:-------------:|:-------------:|:----------:|:------------:|:-------:|
| l1_action | — | 0 | 0 | 0 | 0 | 0 |
| l2_perception | 0 | — | 0 | 0 | 0 | 0 |
| l3_embodiment | 0 | 0 | — | 0 | 0 | 0 |
| l4_emotion | 0 | 0 | 0 | — | 0 | 0 |
| l5_cognition | **1** | 0 | 0 | 0 | — | 0 |
| l6_meta | 0 | 0 | 0 | 0 | 0 | — |

## 外部→layer 引用

| 来源 | 目标 | 数量 |
|------|------|------|
| core/ | l5_cognition | 40 |
| core/ | l1_action | 29 |
| core/ | l3_embodiment | 9 |
| cli/ | l5_cognition | 多处 (via SelfIteratingBrain) |
| core/ | l2_perception | 2 |
| core/ | l6_meta | 1 |

## 发现

### F1: l5_cognition → l1_action (1 处)

**位置**: `l5_cognition/nt_mind/evolution/self_diagnose.rs:10`
```rust
use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

**性质**: 认知层直接引用行动层类型，违反分层依赖规则 (L5 不应依赖 L1)。

**修复建议**:
1. 将 `ProjectSnapshot` 提升到 `core/` 共享类型
2. 或通过 `l5_cognition/act_facade` 门面访问
3. 或重构 `self_diagnose` 使 L1 类型通过 trait 抽象

### F2: l5_cognition → l6_meta (通过 facade，非直接引用)

**位置**: `l5_cognition/mod.rs:20-24` 定义 `l6_facade` 门面

**实际使用**: `handlers_consciousness.rs` / `pipeline.rs` 中有多处 `crate::l6_meta::*` 引用 (约 10+ 处)

**性质**: 已通过 facade 集中化，但 facade 本身是 L5→L6 向上依赖。代码注释已记录 (`⚠️ 向上依赖`)。

**状态**: 已治理，facade 模式可接受。

### F3: l5_cognition → l2_perception (通过 facade)

**位置**: `l5_cognition/mod.rs:14-18` 定义 `l2_facade` 门面

**性质**: 已通过 facade 集中化，单一事实源在 L2。

**状态**: 已治理。

### F4: facade 模式汇总

| Facade | 方向 | 文件 |
|--------|------|------|
| `kb_facade` | L5 → L1 (NT-MEMORY) | `l5_cognition/kb_facade.rs` |
| `io_facade` | L5 → L1 (NT-IO) | `l5_cognition/io_facade.rs` |
| `io_skills_facade` | L5 → L1 (NT-IO skills) | `l5_cognition/io_skills_facade.rs` |
| `act_facade` | L5 → L1 (NT-ACT) | `l5_cognition/act_facade.rs` |
| `l3_facade` | L5 → L3 | `l5_cognition/l3_facade.rs` |
| `l2_facade` | L5 → L2 | `l5_cognition/l2_facade.rs` |
| `l6_facade` | L5 → L6 | `l5_cognition/l6_facade.rs` |

**评价**: facade 集中化策略执行良好，跨层引用已收敛到可审计的门面点。唯一遗漏是 `self_diagnose.rs` 的直接引用。

## 行动项

| # | 优先级 | 行动 | 状态 |
|---|--------|------|------|
| A1 | P1 | 修复 `self_diagnose.rs:10` 直接引用 l1_action → 改走 act_facade 或提升类型到 core | TODO |
| A2 | P2 | 审计 core/→layer 引用是否合理 (core 作为共享层向下引用是预期行为) | DONE |
| A3 | P3 | 监控 facade 数量增长，超过 10 个时考虑重构为 trait 抽象层 | MONITOR |

## 结论

跨域引用治理状态 **良好**:
- 6 层间仅 1 处直接违规引用 (l5→l1)
- facade 集中化模式执行到位，7 个门面覆盖所有跨层需求
- core/ 作为共享层向下引用符合架构预期
- 无循环依赖 (layer 间无双向引用)
