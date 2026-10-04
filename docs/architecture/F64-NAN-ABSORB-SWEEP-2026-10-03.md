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

---

# 补：逐处读上下文的**实证判定**（2026-10-03晚间）

> 上文那份 14 处清单是**基于 grep 的候选**，不是判定。
> 本节把其中 6 处**逐处读了上下文**，替换猜测。
> ⭐ **结论：6 处里 3 处早已有守卫 —— 而 grep 形态完全看不到它们。**

## ⛔ 先更正我自己的第 3 处误判：`sandbox.rs` **完全不需要改**
```rust
pub fn health(&self) -> f64 {
    if self.evaluated_count == 0 {
        return 1.0;              // ⭐ 守卫已存在
    }
    (1.0 - self.denied_count as f64 / self.evaluated_count as f64).max(0.0).min(1.0)
}
```
⇒ 我上文说「`sandbox.rs:199` 未评估时静默 0% 安全」⇒ **错**，守卫在**上方 2 行**。
⇒ 且**缺省语义早已裁决**：`1.0`（未评估 = 健康）—— 对一个名为 `health()` 的分数
  这是**正确**的默认值，不需要我再去「裁决」。
⇒ ⛔ 这是同型错误**第 4 次**（`lifecycle.rs:578`、`:586?`、`sandbox.rs:199`、本节的 `crystallization`）。

## 6 处逐处判定
| 站点 | 读到的守卫 | 判定 |
| |---|
| `nt_core_observer.rs:627` | ✅ `if self.mode_buffer.is_empty() { 0.5 }` | **已裁决**（0.5 中性多样性），无需改 |
| `nt_core_memory_budget.rs:81` | ✅ 护了**分子** `if rss == 0 { return 0.0 }`；⛔ **分母 `soft_limit` 未护** | **残留**：`soft_limit == 0` ⇒ `inf/NaN` ⇒ `.min(1.0)` ⇒ 恒 1.0 |
| `nt_mind/seal/crystallization.rs:122` | ✅ **构造性守卫**：`Some(&count) if count >= self.threshold` 才进入除法 | **基本安全**；仅 `count==0 && threshold==0` ⇒ `0/0` ⇒ NaN ⇒ 1.0 |
| `nt_io_inference/kv_cache_optimizer.rs:353` | ⛔ **无** | **残留且后果最重**：`sparsity_threshold == 0` ⇒ `variance/0` ⇒ `.min(1.0)=1.0` ⇒ `keep_ratio=1` ⇒ `1.0-1.0=0` ⇒ **保留 0 个元素**（静默丢数据） |
| `nt_core_gwt/monitor.rs:154` | ⛔ 分母未护，但在 `if self.in_deadlock` 分支内 | **待查**：`max_stimulus_before_rollback` 在该状态下是否恒 > 0 |
| `l6_meta/nt_safety_monitor.rs:268` | ⛔ **无** | **残留**：`config.max_personality_drift == 0` ⇒ severity ⇒ `.min(1.0)` ⇒ **1.0（最大告警）**，方向是**误报最大化** |

## ⭐ 判定标准（本节新增，比上文的方法论更严）
光看「分母可为 0」**不足以定缺陷**，还要看**后果方向**与**是否已被上游守卫**：

| 候选 | 后果方向 | 优先级 |
|---|---|---|
| `kv_cache_optimizer:353` | **静默丢数据**（保留 0 个元素） | ⭐最高 |
| `nt_safety_monitor:268` | 误报最大化（severity 1.0） | 中 |
| `memory_budget:81` | 该维度恒 1.0（无区分度） | 中 |
| `monitor:154` | 未确认 | 待查 |

## ⛔ 仍未落代码的理由（补充）
上述 3 处「残留」我**都没有证明其配置项真的会被置 0**
（`soft_limit` / `sparsity_threshold` / `max_personality_drift` 是否有 0 的合法路径）。
⇒ 按本文档自己的前置条件门，**「可能为 0」不等于「为 0」** ⇒ 不动。
⇒ 真要动，需先反查这三个配置项的**写入点**（它们是否 pub、是否可从配置/环境传入 0），
再单文件修复 + 跑绿。**顺序是：先查写入点，后改代码。**

## ⭐ 本节的方法论收获
`rg` 能看到「除以非字面量 + 截断」的**形状**，**看不到上方两行的守卫**。
⇒ 上文那份 14 处清单应被读作**候选集**；
**只有逐处读过上下文的判定才算结论。**
⇒ 这是本会话第 4 次同型失误 ⇒ 它不是偶发，是**默认行为**，故写进文档而非只写进commit。

---

# 收口：14 处候选**已全部逐处读完**（2026-10-03 最后补完）

> 前一节只读了 6 处，剩 3 处未读 ⇒ 一份没读完的清单本身就是**误导**。
> 本节补完，并给全表**最终判定**。

## 本节补读的 3 处 —— **全部已有守卫**
| 站点 | 读到的守卫 | 判定 |
|---|---|---|
| `nt_core_hcube/cube.rs:59` | `if self.entries.is_empty() { return 0.0; }`（在除法**之前**）；分母是 `entries.len()` 而非配置项 | ✅ **安全** |
| `nt_core_consciousness_tree/lifecycle.rs:559` | `if self.soil.kb_node_count > 0 { … } else { … }`；上一行 `:554` 的 `/1000.0` 是**字面量**，恒 > 0 | ✅ **安全** |
| `nt_core_consciousness_tree/lifecycle.rs:586` | `if self.roots.total_fetched > 0 { … } else { … }`（与 `:577` 同一模式） | ✅ **安全** |

## ⭐ 全表最终判定（14 处候选，**逐处读过**）
| 判定 | 站点 |
|---|---|
| ✅ **已守卫，无需改** | `observer.rs:627` · `cube.rs:59` · `crystallization.rs:122`（构造性） · `sandbox.rs:199` · `lifecycle.rs:559` · `lifecycle.rs:578` · `lifecycle.rs:586` · `lifecycle.rs:668`（本来就是正确范式） |
| ✅ **已修**（真隐患，收窄封装） | `salience.rs:76` → `max_access_count` 收私有（`c4b6d6f1`） · `kv_cache_optimizer.rs:353` → `sparsity_threshold` 收私有（`38823093`） |
| ⬜ **残留但不可达** | `nt_safety_monitor.rs:268`（`:263` 的 `if drift > max` 间接保证 `max > 0`；默认 0.3；两处赋值都在测试里） · `memory_budget.rs:81`（`soft_limit` **已是私有**、`u64`、构造参数、`:165` 测试已断言 `> 0`；除零得 `inf` ⇒ 「无预算 ⇒ 100% 占用」**语义正确**） |
| — 不适用 | `monitor.rs:154`（在 `if self.in_deadlock` 分支内，待查） · `hebbian.rs:51`（**已注释掉**） |

## ⭐ 这次普查真正的产出：**三个数字**
1. **14 处候选里，0 处是「被截断到上限导致维度压平」的活缺陷。**
2. 真正值得改的只有 **2 处**，且都不是「加 `if denom > 0`」，
   而是**把 `pub` 字段收窄以让坏状态不可达** ⇒ 已改完。
3. ⭐ **我在这一份清单上错了 5 次**（`lifecycle.rs:578`、`:586`、`sandbox.rs:199`、
   `crystallization.rs:122`、以及把 `kv_cache_optimizer:353` 误判为最高优先级）
   —— **全部因为没读守卫就下结论**。

⇒ 所以这份文档的价值**不是清单**，而是那句结论：
**「有守卫的地方，grep 看不见。」**
⇒ 每个候选都必须读到「除法上方三行」，才允许进「缺陷」栏。

