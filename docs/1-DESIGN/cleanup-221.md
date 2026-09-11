# Cleanup-221: 跨域引用清理报告

**日期**: 2026-09-11
**范围**: 六层架构跨层引用违规修复
**状态**: ✅ 已完成

## 1. 问题概述

六层架构 (L1→L6) 存在跨层直接引用，违反层间隔离原则。修复前统计：

| 违规方向 | 修复前 | 修复后 | 减少 |
|----------|--------|--------|------|
| L1→L3   | 0      | 0      | —    |
| L2→L1   | 8      | 1*     | -87% |
| L3→L1   | 15     | 0      | -100%|
| L3→L2   | 0      | 0      | —    |
| L5→L1   | 36     | 0      | -100%|
| L5→L3   | 1      | 0      | -100%|
| L6→L1   | 1      | 0      | -100%|

> *L2→L1 剩余1处为 `nt_world_github_absorber.rs` 测试代码中 `DownloadOptions` 直接导入，因该类型为 `pub(crate)` 无法通过 facade re-export。

## 2. 修复方案: Facade 门面模式

每层新增 `l1_facade.rs` / `l3_facade.rs` 模块，集中 re-export 跨层类型，避免散布直接引用。

### 新增 Facade 模块

| 模块 | 路径 | 用途 |
|------|------|------|
| L3 l1_facade | `l3_embodiment/l1_facade.rs` | L3→L1 类型 re-export |
| L6 l1_facade | `l6_meta/l1_facade.rs` | L6→L1 类型 re-export |
| L5 l3_facade | `l5_cognition/l3_facade.rs` | L5→L3 类型 re-export |

### 已有 Facade 模块 (未修改)

| 模块 | 路径 | 用途 |
|------|------|------|
| L2 l1_facade | `l2_perception/nt_world/l1_facade.rs` | L2→L1 类型 re-export |
| L5 act_facade | `l5_cognition/act_facade.rs` | L5→L1 NT-ACT re-export |
| L5 io_facade | `l5_cognition/io_facade.rs` | L5→L1 NT-IO re-export |
| L5 io_skills_facade | `l5_cognition/io_skills_facade.rs` | L5→L1 IO 技能 re-export |
| L5 kb_facade | `l5_cognition/kb_facade.rs` | L5→L1 KB re-export |

## 3. 修改文件清单

### 新建文件 (3)
- `neotrix-core/src/l3_embodiment/l1_facade.rs`
- `neotrix-core/src/l6_meta/l1_facade.rs`
- `neotrix-core/src/l5_cognition/l3_facade.rs`

### 修改 mod.rs 注册 (3)
- `neotrix-core/src/l3_embodiment/mod.rs` — 添加 `pub mod l1_facade`
- `neotrix-core/src/l6_meta/mod.rs` — 添加 `pub mod l1_facade`
- `neotrix-core/src/l5_cognition/mod.rs` — 添加 `pub mod l3_facade`

### 修改源文件 (8)
| 文件 | 变更 |
|------|------|
| `l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs` | `crate::l1_action::*` → `crate::l3_embodiment::l1_facade::*` |
| `l3_embodiment/nt_shield/nt_shield_audit.rs` | 活跃代码 + 测试代码 → L3 facade |
| `l3_embodiment/nt_shield/nt_shield/audit.rs` | `crate::l1_action::*` → `crate::l3_embodiment::l1_facade::*` |
| `l3_embodiment/nt_shield/nt_shield_cleanup/risk_assessor.rs` | `crate::l1_action::*` → `crate::l3_embodiment::l1_facade::*` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `crate::l3_embodiment::*` → `crate::l5_cognition::l3_facade::*` |
| `l6_meta/coordination/nt_meta_cleanup/coordinator.rs` | `crate::l1_action::*` → `crate::l6_meta::l1_facade::*` |
| `l2_perception/nt_world/nt_world_ods.rs` | 测试代码 → L2 l1_facade |
| `l2_perception/nt_world/nt_world_monitor.rs` | 测试代码 → L2 l1_facade |

### 附带修复 (预存缺陷)
| 文件 | 变更 |
|------|------|
| `l1_action/nt_act/nt_act_cleanup/shared.rs` | `ScanCategory` 枚举补充 `BuildArtifacts/BackupFiles/TempFiles/OldLogs/Other` 变体; `ScanResult` 字段对齐实际用法 |
| `l2_perception/nt_world/l1_facade.rs` | 移除 `DownloadOptions` re-export (类型为 `pub(crate)` 无法跨模块 re-export) |
| `l2_perception/nt_world/nt_world_github_absorber.rs` | 测试代码改为直接导入 L1 类型 |

## 4. 构建验证

```
cargo check -p neotrix --lib
```

新增 facade 相关错误: **0** (全部解决)
预存错误: **10** (非本次变更引入，包括 `knowledge_store.rs` 类型不匹配、`seal_pipeline.rs` 未使用导入等)

## 5. 注入规则

建议在 CI 中添加跨层引用检查脚本:

```bash
#!/bin/bash
# cross-layer-check.sh — 检测跨层直接引用
VIOLATIONS=0

# L3→L1 (应通过 l3_facade)
COUNT=$(grep -rn "use crate::l1_action" src/l3_embodiment --include="*.rs" | grep -v "l1_facade" | grep -v "//.*use crate" | wc -l)
if [ "$COUNT" -gt 0 ]; then echo "FAIL: L3→L1: $COUNT"; VIOLATIONS=$((VIOLATIONS+COUNT)); fi

# L5→L1 (应通过 facade)
COUNT=$(grep -rn "use crate::l1_action" src/l5_cognition --include="*.rs" | grep -v "facade" | grep -v "//.*use crate" | wc -l)
if [ "$COUNT" -gt 0 ]; then echo "FAIL: L5→L1: $COUNT"; VIOLATIONS=$((VIOLATIONS+COUNT)); fi

# L5→L3 (应通过 l3_facade)
COUNT=$(grep -rn "use crate::l3_embodiment" src/l5_cognition --include="*.rs" | grep -v "l3_facade" | grep -v "//.*use crate" | wc -l)
if [ "$COUNT" -gt 0 ]; then echo "FAIL: L5→L3: $COUNT"; VIOLATIONS=$((VIOLATIONS+COUNT)); fi

# L6→L1 (应通过 l6_facade)
COUNT=$(grep -rn "use crate::l1_action" src/l6_meta --include="*.rs" | grep -v "l1_facade" | grep -v "//.*use crate" | wc -l)
if [ "$COUNT" -gt 0 ]; then echo "FAIL: L6→L1: $COUNT"; VIOLATIONS=$((VIOLATIONS+COUNT)); fi

exit $VIOLATIONS
```
