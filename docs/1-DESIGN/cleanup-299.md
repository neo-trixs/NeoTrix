# Cleanup-299: 跨层引用扫描

**扫描时间**: 2026-09-11
**扫描范围**: `neotrix-core/src/` 下 6 层 (l1_action → l6_meta)
**排除**: test 文件、facade 文件

---

## 合规判定: ❌ FAIL

发现 **5 处**跨层 `use crate::` 引用违规 (非 facade、非 test)。

---

## 违规清单

### L5 → L1 (向上跳层，4处)

| # | 文件 | 行号 | 引用内容 |
|---|------|------|----------|
| 1 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 679 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch}` |
| 2 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 680 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{...}` |
| 3 | `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |
| 4 | `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |

### L5 → L6 (向上引用更高层，1处)

| # | 文件 | 行号 | 引用内容 |
|---|------|------|----------|
| 5 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `use crate::l6_meta::coordination::self_improvement::SystemMetrics` |

---

## 依赖方向分析

```
L6 Meta ─────────────────────────────────────────┐
   ↑ (向上引用)                                    │
L5 Cognition ──→ L2 Perception (通过 facade)       │
   │              L3 Embodiment (通过 facade)       │
   │              L6 Meta (通过 facade)             │
   │              L1 Action (通过 facade)           │
   │                                                │
   └── ❌ 直接 use crate::l1_action::* (绕过 facade) │
   └── ❌ 直接 use crate::l6_meta::* (绕过 facade)   │
                                                    │
L4 Emotion ──────────────────────────────────────── │
L3 Embodiment ──→ L1 Action (通过 l1_facade)       │
L2 Perception ──→ L1 Action (通过 l1_facade)       │
L1 Action ──────────────────────────────────────── ┘
```

### Facade 机制 (已合规)

各层通过 facade 文件合规地引用下层类型:

| Facade 文件 | 方向 | 用途 |
|-------------|------|------|
| `l2_perception/nt_world/l1_facade.rs` | L2→L1 | KnowledgeBase, CrawlCycleReport, DownloadOptions |
| `l3_embodiment/l1_facade.rs` | L3→L1 | GatewayV2, L1Error, KnowledgeBase |
| `l5_cognition/kb_facade.rs` | L5→L1 | KnowledgeBase, NodeType, kv_set/get |
| `l5_cognition/io_facade.rs` | L5→L1 | context_budget, standalone tools |
| `l5_cognition/io_skills_facade.rs` | L5→L1 | IO skill modules |
| `l5_cognition/act_facade.rs` | L5→L1 | ACT types, trade, crypto |
| `l5_cognition/l2_facade.rs` | L5→L2 | WorldModelV2, SearchResult |
| `l5_cognition/l3_facade.rs` | L5→L3 | Shield audit types |
| `l5_cognition/l6_facade.rs` | L5→L6 | ConsciousnessMonitor, EvalHarness |
| `l6_meta/l1_facade.rs` | L6→L1 | Cleanup shared types |

---

## 修复建议

### 违规 1-2: handlers_maintenance.rs (L5→L1)

**问题**: 直接引用 `nt_memory_kb::nt_memory_community` 和 `nt_memory_store`
**修复**: 在 `l5_cognition/kb_facade.rs` 中增加 re-export

```rust
// l5_cognition/kb_facade.rs 中追加
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{...}; // 具体需要的类型
```

然后 handlers_maintenance.rs 改为:
```rust
use crate::l5_cognition::kb_facade::{CommunityDetector, CommunityAwareSearch};
```

### 违规 3-4: self_diagnose.rs / evolution_loop.rs (L5→L1)

**问题**: 直接引用 `nt_act::nt_act_types::ProjectSnapshot`
**修复**: 在 `l5_cognition/act_facade.rs` 中增加 re-export

```rust
// l5_cognition/act_facade.rs 中追加
pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

然后改为:
```rust
use crate::l5_cognition::act_facade::ProjectSnapshot;
```

### 违规 5: handlers_maintenance.rs (L5→L6)

**问题**: 直接引用 `l6_meta::coordination::self_improvement::SystemMetrics`
**修复**: 在 `l5_cognition/l6_facade.rs` 中增加 re-export

```rust
// l5_cognition/l6_facade.rs 中追加
pub use crate::l6_meta::coordination::self_improvement::SystemMetrics;
```

然后改为:
```rust
use crate::l5_cognition::l6_facade::SystemMetrics;
```

---

## 扫描方法

```bash
# 扫描 crate:: 跨层 use (排除 facade + test + 注释)
find neotrix-core/src/l{1..6}_*/ -name '*.rs' \
    ! -name '*facade*' ! -name '*test*' ! -path '*/test*' \
    -exec grep -Hn 'use crate::l[1-6]_' {} + | grep -v '^\s*//'

# 层级判定: 文件路径中的 l{1-6}_* 为源层, use 中的 crate::l{1-6}_* 为目标层
# 违规条件: 目标层 != 源层 (跨层引用)
# 合规条件: 通过 facade 文件间接引用
```
