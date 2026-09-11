# Cleanup #233: L6↔L5 双向引用修复

## 问题

`l6_meta/coordination/cross_module_audit.rs` 直接导入 L5 层类型：

```rust
use crate::l5_cognition::nt_core::seal::rhythm_recalculator::SegmentData;
```

违反六层架构约束：L6 不应直接依赖 L5 实现。

## 修复方案

**将共享数据类型下沉到 core 层（L4 认知层）**，L5 和 L6 均从 core 引用。

### 变更清单

| 文件 | 变更 |
|------|------|
| `core/nt_core_narrative_types.rs` | **新建** — 定义 `SegmentType` + `SegmentData` |
| `core/mod.rs` | 注册 `pub mod nt_core_narrative_types` |
| `l5_cognition/nt_core/seal/rhythm_recalculator.rs` | 删除类型定义，改为 `pub use crate::core::nt_core_narrative_types::*` |
| `l6_meta/coordination/cross_module_audit.rs` | import 路径从 `l5_cognition::...` 改为 `crate::core::nt_core_narrative_types` |

### 依赖流向（修复后）

```
core (nt_core_narrative_types)  ← SegmentType, SegmentData
  ↑               ↑
  L5              L6
  rhythm_         cross_module_
  recalculator    audit
  (re-export)     (direct import)
```

### 验证

```bash
grep -rn "use crate::l5_cognition" neotrix-core/src/l6_meta --include="*.rs" | grep -v test | grep -v facade
# 输出为空 — 零 L6→L5 违规
```

### 向后兼容

L5 内部模块（如 `blank_space_checker.rs`）仍通过 `rhythm_recalculator::SegmentData` 引用，re-export 保证零改动。
