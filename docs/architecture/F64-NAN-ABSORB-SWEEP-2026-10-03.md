# 普查：「维度被压平成常量」—— 收敛到**一个**机制（2026-10-03）

> 只读普查，**未落任何代码**（共享树当前被别窗重构打断，无法验证）。
> 本篇的价值在于**机制**：不是「有 14 处要改」，而是「一切都是同一个语言语义」。

## ⭐⭐⭐ 唯一机制

> **`f64::min` / `f64::max` 会静默吸收 `NaN` —— 返回那个非 `NaN` 的操作数。**

独立程序实测（不依赖仓库，故结论不受共享树状态影响）：
```
NaN.min(1.0) = 1.0     NaN.max(0.0) = 0.0
```

⇒ **一个 `.min`/`.max` 就足以把「无定义」变成「看起来正常的极值」**，
而调用处**没有任何信号**表明值已被替换。
⇒ 这与本会话已修的 `effective_salience` 饱和缺陷、
   与 `salience.rs` 的维度失效，**是同一件事的三次显形**。

## 三处已确认实例（同一机制）
| 站点 | 表达式 | NaN 被吸收后 |
|---|---|---|
| `nt_core_gwt/resonance.rs` `effective_salience`（**已修**） | `.min(1.0)` | 增益把**所有**模块顶到 1.0 ⇒ winner 恒为最后一个（我按邻居数归一修掉） |
| `decay_forgetting/salience.rs:76` | `access_count / self.max_access_count` 后 `.min(1.0)` | 分母为 0 ⇒ `inf`或 `NaN` ⇒ **恒 1.0** ⇒ 该维度对所有条目失去区分能力 |
| `nt_act/actions/security/sandbox.rs:199` | `(1.0 - denied/evaluated).max(0.0).min(1.0)` | `evaluated_count == 0` ⇒ `0/0`=NaN ⇒ `.max(0.0)` ⇒ **恒 0.0** ⇒ 未评估时静默「0% 安全」 |

## ⭐ 仓里**已有**正确写法（所以这不是「不知道」，是「没一致用」）
· `nt_core_consciousness_tree/lifecycle.rs:668` — `self.branches.len().max(1)`（**护分母**）
· 同文件 `:577` / `:129` — `if self.roots.total_fetched > 0 { … }`（**护分母**）
⇒ 正确模式**近在同几行内**，只是没用在其余 14 处。

## 普查清单（14 处同形状：除以**非字面量**后立刻截断）
```
nt_core_observer.rs:627              distinct / self.mode_buffer.len()      ← Vec::len() 可为 0
nt_safety_monitor.rs:268             drift / config.max_personality_drift  ← 配置项可为 0
decay_forgetting/salience.rs:76      access_count / self.max_access_count ← pub 可绕过守卫
nt_io_inference/kv_cache_optimizer.rs:353  variance / self.sparsity_threshold
nt_core_hcube/cube.rs:59             total / self.entries.len()
nt_core_memory_budget.rs:81          rss / self.soft_limit
nt_act/actions/security/sandbox.rs:199     denied / evaluated             ← 初值 0（已实测）
nt_mind/seal/crystallization.rs:122 count / self.threshold
nt_core_gwt/monitor.rs:154           attempts / self.max_stimulus_before_rollback
nt_core_consciousness_tree/lifecycle.rs:559  kb_edge_count / kb_node_count
nt_core_consciousness_tree/lifecycle.rs:578  total_failed / total_fetched  ← ⛔ 已有守卫
nt_core_consciousness_tree/lifecycle.rs:586  total_absorbed / total_fetched ← ⛔ 疑似同守卫
nt_core_consciousness_tree/lifecycle.rs:668  fog / branches.len().max(1)   ← ✅ 正确写法
```

## ⛔ 我在本普查里**又错了两次**（如实记录）
| 我一度判断 | 实测 |
|---|---|
| `lifecycle.rs:578` 启动期为 0 ⇒ 维度死 | ⛔ **错**，`:577` 已有 `if total_fetched > 0` |
| `sandbox.rs:199` 会被压成 1.0 | ⛔ **错**，是 `.max(0.0)` ⇒ 压成 **0.0** |

⇒ 第三次同型失误（又是**未读守卫就下结论**）。
⇒ 再次印证 `ABSORPTION-PRECONDITION-GATE`：**逐点读现场，不靠模式匹配判缺陷。**

## 建议的修法形状（**本轮不落**）
分母侧护住，**与本仓既有正确写法一致**，而不是在结果侧猜：
- 计数/长度类：`self.branches.len().max(1)`（照 `:668`）
- 比率类：`if denom > 0 { … } else { <明确的缺省语义> }`（照 `:577`）
⛔ 关键：**缺省语义要显式写出来**。`sandbox.rs:199` 现在缺的正是这个 ——
「未评估」到底该报 0.0 还是别的，**是语义决定，不是修 bug**。

## ⛔ 未落代码的理由
1. 共享树当前被别窗重构打断（`nt_dialogue_tui` 缺失、`cli_free_source.rs` 编译错）⇒ **无法验证**，
   而我的事故记录已写死「未验证改动不可接受」。
2. 部分缺省语义**需先裁决**（见上），不是机械替换。
3. 应**分批**：单文件 → 跑绿 → 提交（事故记录的约束 2）。
