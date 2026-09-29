# D-3 裁决结论 — 融合为单一自进化验证底座

> **裁决问题**：「通用评测引擎」还是「自我修复专用 harness」？
> **结论**：**这个二选一本身是错的** —— 它们不是同层的两个候选，而是**三层**。
> 本文件记录诊断、实现、以及实现过程中被门顶回来的三处错误。

---

## 1. 诊断：实测推翻了两选一

### 1.1 三个候选的真实形态

| | `nt_mind_eval_harness`（活） | `nt_meta/eval_engine`（死） | `l5_cognition/traits.rs:142` |
|---|---|---|---|
| 体量 | 2,235 行 / 8 文件 | **650 行 / 4 文件** | 13 行（2 个 trait） |
| 状态 | ✅ 已接线 | ⛔ **`mod` 未声明，从不编译** | ⚠️ trait 已定义，**全仓无实现** |
| 消费者 | `nt_repair_facade.rs:27` | **零** | `seal_loop.rs:665` 收 `Option<&dyn EvalHarnessApi>` |
| 问的问题 | 模型能力曲线形状、合规平面 | 数据集/变体/判官**算术** | 改了一版，**怎么知道变好了** |

### 1.2 ⛔ 关键发现：`eval_engine` 的 `llm_judge` 从不调 LLM

```rust
// neotrix-core/src/l6_meta/nt_meta/eval_engine/llm_judge.rs:86
let normalized = 1.0;          // ← 每条 criterion 恒满分
// :90   score: config.max_score,
// :91   reasoning: "Full score".into(),
```

- 全文件**零** `async` / `reqwest` / `provider` / `http`
- 签名是 `pub fn evaluate_response(config, prompt, response) -> JudgeResult`，
  `prompt` 与 `response` 被 `let _ = (prompt, response);` **丢弃**
- ⇒ **它给任何输入都返回满分**，与它自己的文件名 `llm_judge` 不符

**⇒ 这是「永远满分」的假测量。接线它比不接更危险。**

### 1.3 `traits.rs` 明文禁止 L5 直接引 L6

```rust
// neotrix-core/src/l5_cognition/traits.rs:133-139
// L6 → L5 Trait Abstractions (跨层引用隔离)
// L5 认知层通过这些 trait 访问 L6 元认知层提供的能力，避免直接
// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
// 在 L6 模块中提供（impl L5Trait for L6Type）。
```

⇒ 架构早已规定：**L5 定义 trait，L6 提供 impl**。
缺的只是那个 impl —— **这正是本模块要补的洞**。

---

## 2. 结论：三层结构，本模块补最上面缺的一层

```text
┌─ L6 能力评测 ────────────────────────────────────────────────┐
│  nt_mind_eval_harness · 活 · 问「模型能力曲线是什么形状」      │
│  ⇒ 保留不动。它是「评测模型」，不是「验证进化」                │
└──────────────────────────────────────────────────────────────┘
┌─ L6 通用评测原语 ────────────────────────────────────────────┐
│  nt_meta/eval_engine · 650 行死代码 · 问「数据集/变体/判官」  │
│  ⇒ ⛔ 不接。llm_judge 恒满分 ⇒ 接了是假测量                    │
│     但 Dataset/Experiment 的**形状**值得在新模块里复用          │
└──────────────────────────────────────────────────────────────┘
┌─ L6 确定性验证器 ────────────────────────────────────────────┐
│  nt_verify_oracle · 405 行活 · 问「这条产出可判定吗」          │
│  ⇒ 保留。新模块的 judge_case 是它的「无 provider 依赖」简化版   │
└──────────────────────────────────────────────────────────────┘
┌─ L5 自进化接口 ──────────────────────────────────────────────┐
│  traits.rs:142 EvalHarnessApi · 问「改了一版是好是坏」         │
│  ⇒ 🔴 **全仓无实现** —— 本次补的就是这一层                      │
└──────────────────────────────────────────────────────────────┘
```

**⇒ 「通用引擎」与「修复 harness」不是竞品，是上下游。**
融合的最优解 = **保留活的两层 + 不接死的那层 + 补上缺的第四层**。

---

## 3. 实现：`nt_evolution_eval`（754 行 + 19 个 `#[test]`）

### 3.1 落点

```
neotrix-core/src/l6_meta/nt_meta/nt_evolution_eval.rs              （新，754 行）
neotrix-core/src/l6_meta/nt_meta/tests/nt_evolution_eval_tests.rs （新，459 行 / 19 个 `#[test]`）
neotrix-core/src/l6_meta/nt_meta/mod.rs                            （接线 + re-export）
```

### 3.2 四条不可协商的约束

| # | 约束 | 抄自 | 为什么 |
|---|---|---|---|
| 1 | **臂中立**：评分器不知道跑的是哪个臂；只用 `Baseline`/`Candidate` 两个对称臂 | `Aegis` `score_agentic_benchmark_outcome.py:28-62` | 否则「装了 vs 没装」是自证。⛔ 允许 N 个命名臂 = 允许挑一个赢的报上去 |
| 2 | **噪声地板**：差值必须 > δ×σ 才算改进；`n<2` ⇒ `None` | `Soup/benchmarks/gate-836`（13 次同配置 2.43× 差异） | 没有地板的 A/B 是在测噪声 |
| 3 | **正负都进账**：`Ledger` 只 `append`，**无 `update`/`remove`** | `autoresearch`（MIT） | 只记通过的账本是日志，不是测量 |
| 4 | **可证伪**：`Preregistration.falsifier` 为空 ⇒ 判决必拒 | `EVOLUTION-ROADMAP-2026-09-28 §0.2 动作A` | 说不出的不是假设，是感觉 |

### 3.3 六种 veto（**非补偿**：任一命中即整体否决）

```
deterministic_regression   某 case 从过变不过（逐 case 定位，不用聚合）
safety_regressed           candidate 触发 baseline 未触发的 forbidden
within_noise               差值未超噪声地板
environment_mismatch       两臂 env digest 不同 ⇒ 结论不可比
insufficient_evidence      没有噪声地板 ⇒ 不许判「改进」
no_falsifier               预注册没写「什么结果会削弱它」
```

**⛔ 为什么否决而非加权求和**：`rrsi/selection.py` 的 non-compensatory guard ——
加权会让「别处得分高」买下「这里回归了」。

### 3.4 两个正交的诚实性字段

```rust
pub struct CaseOutcome {
    pub passed: bool,            // 确定性判定（真相）
    pub agent_claimed: Option<String>,  // 被测物自述（永不用于判定）
    ...
}
```

依据 `typesafe-computer-use` 的 `benchmarks/osworld/*.jsonl` 实测：
**10 行里 2 行 `outcome='done'` 但 `score=0.0`**，1 行 `low confidence` 但 1.0。
⇒ **只看 agent 自述会得到完全错误的结论。**
`ClaimFidelity::false_completion_rate()` 单独统计这个偏差。

---

## 4. 🔴 实现过程中被门顶回来的三处错误（这才是重点）

**三个都是测试/实现自己的错，门把它们抓出来了。**

### 4.1 判决逻辑对，测试写错了（2 次同类）

初版测试断言「baseline 与 candidate 都全过 ⇒ 应接受」。
实测返回 `REJECT: within_noise delta=+0.0000` —— **判决是对的**（delta=0 不是改进），
**测试是错的**。同类错误出现 **2 次**（`noise_floor_blocks_small_delta`、
`ledger_records_rejections_too`）⇒ 说明「delta=0 不是改进」极易被忽略，已在测试里显式留痕。

### 4.2 `CaseOutcome::pass()` 的名字有歧义 ⇒ 加了 `fail()`

根因：想造一个「失败的 case」时写了 `CaseOutcome::pass("a", Arm::Baseline)`，
结果得到 `passed: true`。实测表现是 `delta=+0.0000`（应为 +1.0），
**看起来像 `pass_rate` 算错了，其实是构造器名字骗人**。

修法（已落进代码注释）：
```rust
/// ⚠️ 之所以提供它：`pass()` 这个名字在测试里很容易被误当成「造一条 case」，
/// 于是想造失败时写了 `pass()` 却得到 `passed: true`。
/// 本轮实测踩过一次 ⇒ 失败的构造必须有一个**不含 "pass" 字样**的名字。
pub fn fail(case_id, arm, missing_required, hit_forbidden) -> Self
```

### 4.3 ⛔ 门本身有假阳性 —— 修了门

加了新文件后 `check-truth-surface` 报 `UNREACHABLE nt_evolution_eval_tests.rs`。
查因：**`scripts/check-truth-surface.sh:169` 的 `#[path]` 正则要求与 `mod` 同行**
（`[^\n]*?`），而我写成了两行：

```rust
#[path = "tests/nt_evolution_eval_tests.rs"]
mod nt_evolution_eval_tests;
```

⇒ **真实存在的文件被判为不可达。** 这与本轮 09-28 那次（把 `tests/` 扩到全部
module 目录）**是同一类缺陷的第三个实例**：门覆盖不全/解析不全 ⇒ 假阳性。

修法：`[^\n]*?` → `[\s\S]{0,200}?`（**限窗口**，避免跨到下一个 `mod`）。
**并做了非空门证明**：注入真孤儿 `zz_orphan_probe.rs` 后门仍报 ⇒ 修门没把门改瞎。

---

## 5. 验证

```
隔离工作树（干净 HEAD e42ec81a + 我的 3 文件）：
  cargo check --lib -p neotrix      →  Finished dev profile in 2m 07s（0 错误）
  cargo test  -p neotrix --lib nt_evolution_eval
      →  19 passed; 0 failed; 12210 filtered out
  cargo test  -p neotrix --lib（全量）
      →  12187 passed; 0 failed; 42 ignored      ← 无回归
```

> ⚠️ 主工作树当时**编不过**，但那是**另一窗口的在制品**：
> `crates/neotrix-neobot/src/nt_agent.rs`（`git diff` 显示 +62 行，
> 新增 `execute_pdf_ground_text`，`truncate_output` 类型不匹配）。
> ⇒ 按 `LESSONS-20260928-fresh-checkout`，用**隔离工作树**验证是唯一可信做法。

---

## 6. 落地后仍需的接线（2026-09-29 更新：B5b 已完成）

| 步骤 | 状态 |
|---|---|
| ① `impl EvalHarnessApi` | ✅ 早已存在（`nt_harness.rs:316`） |
| ② `SelfIteratingBrain.eval_harness` 注入点 | ✅ A9 已加（默认 `None`） |
| ③ **谁注入** | ✅ **B5b 已完成**：`l5_cognition::traits::EvalHarnessFactory`（L5 声明）+ `DefaultEvalHarnessFactory`（L6 实现）+ `entry/brain.rs::attach_eval_harness`（装配点） |
| ④ `CaseSpec` 来源 | ⬜ 本模块的 `judge_case` 是**独立**的纯函数，与 `nt_verify_oracle` 平行。两者尚未合并（合并需统一 `required`/`forbidden` 语义） |
| ⑤ 噪声地板的数据源 | ⬜ 需**同臂重复运行 ≥2 次** |

### ⛔ 接线完成后**仍不能说**「进化已被验证」

`DefaultEvalHarnessFactory` 返回的 harness **只覆盖「确定性回归」这一类判定**。

| 它能回答 | 它**不能**回答 |
|---|---|
| 这次候选变更是否引入**确定性回退** | 这次进化是否让**模型整体**变好了 |
| 候选是否触发了**禁止模式** | 臂中立 A/B 是否有**统计显著**的提升 |

后者需要本模块（`nt_evolution_eval`）+ 噪声地板 + ≥2 次重复运行。

> ⛔ **不要因为「闸门接上了」就认为「进化已被验证」。**
> 那正是本文件开头引的 `prime-agent` 教训：
> `RefinementEvent.outcome` 是**模型自己写的自由文本** ⇒ 日志闭环 ≠ 反馈闭环。
> 一个只有确定性回归的闸门，是**真闸门但很窄**，不是伪证据 —— 区别在于
> **别把它当成后者来引用**。

---

## 7. 复核命令

```bash
# 模块与测试
wc -l neotrix-core/src/l6_meta/nt_meta/nt_evolution_eval.rs
wc -l neotrix-core/src/l6_meta/nt_meta/tests/nt_evolution_eval_tests.rs
grep -n "nt_evolution_eval" neotrix-core/src/l6_meta/nt_meta/mod.rs

# 决策 1.2：eval_engine 的 judge 恒满分（这是不接它的理由）
grep -n "let normalized = 1.0" neotrix-core/src/l6_meta/nt_meta/eval_engine/llm_judge.rs

# 决策 1.3：跨层隔离约定
sed -n '133,145p' neotrix-core/src/l5_cognition/traits.rs

# 4.3 修的门
grep -n '\[\\s\\S\]{0,200}' scripts/check-truth-surface.sh

# 跑测试（需干净 HEAD；主树可能被他窗 WIP 阻断）
git worktree add --detach /tmp/ntx HEAD
cargo test -p neotrix --lib nt_evolution_eval
```
