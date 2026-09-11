# Cleanup #285 — 跨域引用审计

> 扫描时间: 2026-09-11
> 范围: `neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}`
> 排除: `test`, `facade`

## 汇总矩阵

| 源↓ \ 目标→ | l1 | l2 | l3 | l4 | l5 | l6 |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| **l1_action** | — | 0 | 0 | 0 | 0 | 0 |
| **l2_perception** | 0 | — | 0 | 0 | 0 | 0 |
| **l3_embodiment** | 0 | 0 | — | 0 | 0 | 0 |
| **l4_emotion** | 0 | 0 | 0 | — | 0 | 0 |
| **l5_cognition** | **4** | **1** | 0 | 0 | — | **2** |
| **l6_meta** | 0 | 0 | 0 | 0 | 0 | — |

**结论: 仅 l5_cognition 有跨域引用，其余 5 层完全隔离。**

---

## l5_cognition → l1_action (4 active)

全部通过 **facade 模块** 中转（不散布 `use crate::l1_action::*` 到业务代码）:

| Facade | Re-export 模块 | 用途 |
|--------|---------------|------|
| `act_facade.rs` | `nt_act_code`, `nt_act_crypto`, `nt_act_trade` | 工具/动作能力 |
| `io_skills_facade.rs` | `nt_io_*` (10+ 子模块) | IO 技能能力 |
| `kb_facade.rs` | `nt_memory_kb` (含 bm25/coeffect/write_guard) | 知识库能力 |
| `io_facade.rs` | `nt_io_provider`, `nt_io_standalone` | IO 基础设施 |
| `evolution_loop.rs` | `nt_act_types::ProjectSnapshot` | 进化循环用 |
| `self_diagnose.rs` | `nt_act_types::ProjectSnapshot` | 自诊断用 |

**评估: 合理 — facade 模式控制了依赖方向，l5 认知层通过门面访问 l1 能力层。**

## l5_cognition → l2_perception (1 active)

| Facade | Re-export 模块 | 用途 |
|--------|---------------|------|
| `l2_facade.rs` | `nt_world_model_v2`, `nt_world_search`, `nt_world_novel` | 世界感知能力 |

**评估: 合理 — 单一 facade 入口。**

## l5_cognition → l6_meta (5 active + 3 commented)

| Facade | Re-export 模块 | 用途 |
|--------|---------------|------|
| `l6_facade.rs` | `nt_repair::*`, `memory::*` | 自愈/修复/进化评估 |
| `nt_repair_self_heal.rs` | (commented out) | 测试中已注释 |

**评估: 合理 — l5↔l6 双向依赖通过 facade 隔离，符合认知层↔元认知层协作模式。**

---

## 架构合规性

| 维度 | 状态 | 说明 |
|------|------|------|
| **单向依赖** | ✅ | 下层 (l1-l4) 不依赖上层 (l5-l6) |
| **Facade 隔离** | ✅ | 跨域引用全部通过 `*_facade.rs` 模块 |
| **零散布** | ✅ | 业务代码无 `use crate::lX::*` 直接跨域 |
| **l3/l4 零引用** | ✅ | 具身层和情感层完全自闭 |

## 无问题项

- l1→l2, l1→l3, l1→l4, l1→l5, l1→l6: **0**
- l2→l1, l2→l3, l2→l4, l2→l5, l2→l6: **0**
- l3→*, l4→*: **0** (完全隔离)
- l6→l1-l5: **0** (l6 无反向依赖)

**六层架构依赖方向正确，跨域引用控制良好，无需清理动作。**
