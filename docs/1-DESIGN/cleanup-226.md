# Cleanup #226 — 最终跨域引用清零检查

**日期**: 2026-09-11  
**检查范围**: 6层架构层间引用 (排除 facade/test)  
**工具**: `grep -rn "use crate::$to" --include="*.rs" | grep -v test | grep -v facade`

## 跨域引用残留

| 源层 → 目标层 | 引用数 |
|---|---|
| l2_perception → l1_action | 1 |
| l3_embodiment → l1_action | 6 |
| l5_cognition → l1_action | 2 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |
| l6_meta → l5_cognition | 1 |

**总计**: 13 处跨域引用残留

## Facade 模块清单 (10个)

| 文件 | 所在层 |
|---|---|
| `l2_perception/nt_world/l1_facade.rs` | l2_perception |
| `l6_meta/l1_facade.rs` | l6_meta |
| `l5_cognition/act_facade.rs` | l5_cognition |
| `l5_cognition/l6_facade.rs` | l5_cognition |
| `l5_cognition/io_skills_facade.rs` | l5_cognition |
| `l5_cognition/l2_facade.rs` | l5_cognition |
| `l5_cognition/kb_facade.rs` | l5_cognition |
| `l5_cognition/l3_facade.rs` | l5_cognition |
| `l5_cognition/io_facade.rs` | l5_cognition |
| `l3_embodiment/l1_facade.rs` | l3_embodiment |

## 分析

- **l5_cognition** 是 facade 集中层 (8个 facade)，承担认知层对其他层的适配职责
- **l3_embodiment → l1_action (6)** 是最大残留源，需检查是否可通过 facade 消化
- **l6_meta ↔ l5_cognition (2+1)** 双向引用，可能需要抽象接口解耦
- **l2_perception → l1_action (1)** 单点残留，定向清理可行性高

## 建议优先级

1. **P0**: l3_embodiment → l1_action (6处，最大残留)
2. **P1**: l5_cognition → l1_action (2处) + l5_cognition → l6_meta (2处)
3. **P2**: l6_meta → l5_cognition (1处) + l5_cognition → l2_perception (1处) + l2_perception → l1_action (1处)
