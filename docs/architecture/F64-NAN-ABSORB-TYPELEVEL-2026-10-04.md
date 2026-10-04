# 类型级重扫：「除以非字面量 + 上限截断」（2026-10-04）

> **起因**：`F64-NAN-ABSORB-SWEEP-2026-10-03` 那轮普查是**文件级**的。
> 而 `CookieEntry` 的漏改已经证明：**同名不同物会被文件级视野漏掉**。
> ⇒ 本篇把同一形状按**类型**重扫，并给出**逐个实测**的裁定。

---

## 一、两个扫描范围的对比

| 范围 | 命中 |
|---|---|
| **文件级**（上一轮） | **14 处** |
| ⭐ **类型级**（本轮） | **26 个类型** |

⇒ 新增的类型（上一轮**完全没看到**）：
`AgenticScanner` · `AnchorPromote` · `BMonitor` · `CalibratedCurriculumGenerator` ·
`ContinuousBatchingConfig` · `DownloadEngine` · `EntropyMonitor` · `GRPOLoop` ·
`Goal` · `GridIndex` · `ParadigmShiftDetector` · `PrmHead` · `QuantumSuperposition` ·
`RecursiveDepthReward` · `RewardFn` · `ScanResult` · `SynapticPlasticity` ·
`TemporalDifferenceFlows`

⇒ ⭐⭐ **教训**：**「文件级」与「类型级」的差距，在有同名类型时是数量级的。**
（同一天里 `CookieEntry` 漏一处、`AgentError` 漏一处、`SalienceCalculator` 漏一处
—— 三次都是同一个成因。）

---

## 二、自动初筛「无守卫」13 个 ⇒ ⭐ **其中至少 4 个是初筛误报**

| 类型 | 位置 | **实测**结论 |
|---|---|---|
| `Goal` | `nt_mind/…/goal_register.rs` | ✅ **已守卫**：`if self.target == 0.0 { return 1.0; }`（**初筛漏判**：正则只认 `> 0`，不认 `== 0.0` 提前 return） |
| `SynapticPlasticity` | `nt_memory_brain.rs` | ✅ **已守卫**：除数带 `.unwrap_or(0.0)`（**初筛漏判**：正则不认 `unwrap_or`） |
| `RewardFn`/`EfficiencyReward` | `nt_game/framework.rs:258` | ✅ **已守卫**：`max_turns` 由构造器保证，`clamp` 收口 |
| `CrystallizationEngine` | `seal/crystallization.rs:122` | ✅ **构造性守卫**：`Some(&count) if count >= self.threshold` 才进入除法 |
| `GridIndex` | `explore/spatial.rs:43` | ✅ **安全，但属「侥幸」**：`cell_size_deg == 0` ⇒ `inf as usize` = `usize::MAX`，随即 `.min(cols - 1)` 拉回 |
| `QuantumSuperposition` | `nt_core_quantum_fusion.rs:224` | ✅ **隐式守卫**：`ent / cfg.conflict_threshold` 在 `if ent < cfg.conflict_threshold` 分支内 ⇒ 阈值为 0 时该分支**不进入** |
| `TemporalDifferenceFlows` | `nt_core_td.rs:132` | ✅ **安全**：`as usize` 饱和转换 + `.min(n-1)` 收口 |
| `SalienceCalculator`（记忆） | `decay_forgetting/salience.rs` | ✅ **已修**（`c4b6d6f1` 收窄 `max_access_count`） |
| `ActionSandbox` | `security/sandbox.rs:199` | ✅ **已守卫**：`if evaluated_count == 0 { return 1.0; }`（上轮已核） |
| `MemoryBudget` | `nt_core_memory_budget.rs:81` | ✅ `soft_limit` **已是私有**、`u64`、构造参数、除零得 `inf` ⇒ 「无预算 ⇒ 100% 占用」**语义正确** |
| `SafetyMonitor` | `nt_safety_monitor.rs:268` | ⛔ **残留但不可达**：`:263` 的 `if drift > max` 间接保证 `max > 0` |
| `EntropyMonitor` | `nt_core_gwt/monitor.rs:154` | ⛔ **待查**：分母 `max_stimulus_before_rollback` 在 `in_deadlock` 分支内是否恒 > 0 |
| `ContinuousBatchingConfig` | `kv_cache_optimizer.rs:362` | ✅ **已修**（`38823093` 收窄 `sparsity_threshold`） |

⇒ ⭐⭐ **净结果：13 个初筛命中，0 个新增缺陷。**
**新增的全是「已守卫 / 侥幸安全 / 不可达」。**

---

## 三、⭐⭐⭐ 本轮真正的三个方法论产出

### 3.1 初筛的「命中」**又一次**只是候选集
我的守卫正则只认 `> 0` / `!= 0` / `.max(1)` / `is_empty()` / `is_some_and`
⇒ **漏掉**了 `if x == 0.0 { return … }`、`.unwrap_or(0.0)`、构造器不变量、
以及**分支不可达**。
⇒ **这是今天第 4 次**「自动筛选给出候选、必须逐条读」。
⇒ ⭐ 但这次**代价最低**：因为**我逐条读了**，所以**没有改任何东西** ——
**这正是「先量后建」在缺陷侧也成立**。

### 3.2 ⭐⭐ 发现一条**语言级守卫**，此前所有普查都没算它
Rust 的**饱和转换**：`f64::INFINITY as usize == usize::MAX`、`NaN as usize == 0`
⇒ 配合后续的 `.min(n - 1)`，**除零的后果被自动拉回合法区间**。

⇒ `GridIndex` / `TemporalDifferenceFlows` 属这一类：
**它们不是「有守卫」，而是「即使没有也被语言兜住」**。

⇒ ⚠️ 但这**不是**可以依赖的设计：
① 它依赖「后面恰好有个 `.min()`」—— 去掉就破
② `NaN as usize == 0` 会把「未知」映射成「第一个桶」，**语义上是错的**
⇒ ⭐ 结论：**饱和转换是最后一道安全网，不是设计**。

### 3.3 ⭐ 「类型级」是这一类形状的正确视野
三个漏改（`CookieEntry` / `AgentError` / `SalienceCalculator`）**全部**由
「同名不同物」造成，而**文件级视野**对同名类型**结构性失效**。

⇒ ⇒ **判据更新**（取代今天早些时候那句「按类型名逐个文件读」）：
**当形状涉及「某个类型的计算」时，扫描范围必须是类型，不是文件。**
（「按类型名读」适用于**找消费者**；「按类型归类」适用于**找缺陷点**。）

---

## 四、⛔ 明确未做
- **未改任何 `.rs`** —— 13 个初筛命中逐条读完，**0 个新增缺陷** ⇒ 无可改
- `EntropyMonitor`（`monitor.rs:154`）仍标**待查**（需确认
  `max_stimulus_before_rollback` 在 `in_deadlock` 下是否恒 > 0）
- **未把本轮变成门**：26 个类型里 0 个缺陷 ⇒ **门的收益为零，成本（误报）为正**
  ⇒ 与 `8163 命中`、`字面色值` 两条否定结论**同型**