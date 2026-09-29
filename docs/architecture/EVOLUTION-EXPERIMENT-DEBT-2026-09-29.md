# 进化实验设施 — 备件清单（2026-09-29 实测 + 当日已启动）

> ⚠️ **状态已更新：实验流程已启动**（`scripts/ops/nt_evolution_exp.py`）。
> 本文的核心结论仍然成立 —— **3 套设施仍是零生产消费者**，
> 但现在**有了一个真的在跑的流程**（虽然它是 Python CLI，不是 Rust 路径）。
> 见 §已启动 的实测记录。

## 实测（三次 grep，全部零命中）

| 设施 | 位置 | 体量 | 生产调用者 | 状态 |
|---|---|---|---|---|
| `ExperimentRegistry` / `Hypothesis` / `ABTestDesign` | `l5_cognition/nt_mind/nt_mind/evolution/experiment.rs` | — | **0** | ⛔ 备件 |
| `judge_ab` / `Preregistration` / `Ledger` | `l6_meta/nt_meta/nt_evolution_eval.rs` | 754 行 + 19 测试 | **0**（本轮 B6 前） | ⛔ 备件 |
| `ExperimentRunner` | `l6_meta/nt_meta/nt_evolution_runner.rs` | 238 行 + 8 测试 | **0** | ⛔ 备件（本轮 B6） |

`seal_loop` 的闭环钩子**只做单点确定性回归**
（`generate_regression_test` + `run_regression_test`），
⛔ **不做 A/B、不估噪声地板、不要求预注册**。

## 为什么不继续加调用点

本轮已连续**四次**为同一个病花力气：

| # | 症状 | 解法 | 结果 |
|---|---|---|---|
| ① | `check-doc-drift` 恒红 | 改账本棘轮 | ✅ 门修好了 |
| ② | `seal_loop` 硬传 `None` | 加 factory + entry 装配 | ✅ 闸门接上了 |
| ③ | `nt_evolution_eval` 零消费 | 加 `ExperimentRunner` | ⚠️ runner 也是零消费 |
| ④ | `ExperimentRunner` 零消费 | 加 `prereg_from_hypothesis` 桥接 L5 | ⚠️ **L5 那套也是零消费** |

⇒ **根因不是「缺调用点」，是「没有任何进化实验流程在跑」。**
⑤ 若继续加调用点，就是给一个不存在的流程再加一层 —— 第五次重复犯错。

## ⛔ 三件**不能**宣称的事

1. **不能**说「进化已被验证」—— 没有任何 A/B 跑过。
2. **不能**说「闸门接上了所以进化受保护」——
   接上的只是**单点确定性回归**，它能答「有无确定性回退」，
   不能答「整体是否变好」。
3. **不能**把 `coverage-gaps` 门对这 3 套设施的报警当成噪声屏蔽掉——
   **它报的是真的**。详见下。

## 唯一诚实的处置

`scripts/ops/nt_coverage_gaps.py` 报 `☠` 时，
**本文件就是「已知备件」的登记处**。与 `truth-surface-baseline.txt`
（265 条未编译文件）同性质：**记录，而非忽略**。

⇒ 若将来真的启动进化实验流程（哪怕是人工对照实验），
第一件事是**删掉本文件**并把三套设施收敛成一套。
⛔ 在那之前，「三套并存」是**已知债**，不是**已解决的问题**。

## ✅ 已启动（2026-09-29 晚）：`scripts/ops/nt_evolution_exp.py`

### 为什么第一版方案（「模型输出两臂」）被否

实测**三个阻塞**（不是猜的）：

| # | 阻塞 | 证据 |
|---|---|---|
| 1 | `RegressionCase.id` 是 `candidate` 的哈希 | `nt_regression.rs:40-42` ⇒ 两臂 case id **必然不同** ⇒ `case_level_regressions` 永远匹配不上 |
| 2 | `required_categories` 来自 `self.datasets`（工厂里是空的） | 该判据**永不触发** |
| 3 | `run_regression_test` 是纯函数 | 零随机/时间源 ⇒ 同臂重复 N 次**方差恒为 0** |

⇒ 那条路**不能诚实启动**。改用一组**真实存在**的素材：

- **两臂** = 两个真实 git commit（临时 worktree 检出，退出即清理）
- **case 集** = 4 道**纯文件型**门（不 spawn cargo，秒级）
- **oracle** = 门对该 commit 是否报错 —— 确定性但真实
- **噪声地板** = 同臂重复 N 次；确定性 ⇒ σ=0，判决如实报

### 4 次实测（`results.tsv`，append-only）

| baseline | candidate | delta | accept | vetoes |
|---|---|---|---|---|
| `9c303b07` | `ea74eddd` | +0.000 | ❌ | `no_falsifier` |
| `9c303b07` | `ea74eddd` | +0.000 | ❌ | `no_falsifier` |
| `9c303b07` | `ea74eddd` | +0.000 | ❌ | —（预注册齐但无改进） |
| `abe95ff3` | `f0120b02` | **+0.250** | ✅ **ACCEPT** | — |

**第 4 行是真的 ACCEPT**：`f0120b02` 把 `check-doc-drift` 从恒红改成账本棘轮，
candidate 臂的 `doc-drift` 由 exit=1 转 exit=0，pass_rate 0.500 → 0.750。
⇒ **判决机制双向验证过**（既会拒也会接受，不是永远拒绝的假门）。

### ⛔ 仍然不能宣称的事

1. **不能**说「进化已被验证」—— 这验证的是**判决机制**，不是模型能力。
2. **不能**说「Rust 侧 `ExperimentRunner` 已被使用」—— 它仍是零消费者。
   本 CLI 是**独立 Python 实现**，刻意不 import Rust：
   ⛔ 直接调 Rust 就变成「用被测物测被测物」。
3. **三套 Rust 设施仍零消费者**（本文的结论未变）。
   ⇒ 若要让 Rust 侧也用上，需要把本 CLI 的 case 集与判决规则搬进 Rust ——
   **那是产品判断（要不要维护两套实现）**，不是技术障碍。

## 若要让 Rust 侧也用上，需要什么（缺的不是代码）

| 缺什么 | 为什么代码解决不了 |
|---|---|
| **两臂的真实产出** | 噪声地板要求同臂重复 ≥2 次 ⇒ 需要能重复执行同一变更的场景 |
| **一个谁来跑的流程** | 现在没有任何进程会调 `run_experiment` |
| **可比较的 case 集** | `CaseSpec{required, forbidden}` 需要真实回归用例，不是占位 |

⚠️ 这三样**都不是本仓能单方面决定的** —— 需要「要不要做自进化验证」
这个产品判断。⇒ 列入 `DECISIONS-REQUIRED-2026-09-29.md` 的后续裁决项。

## 复核命令

⚠️ **简单 grep 会把「自身定义」「自身测试」「`mod.rs` 的 `pub use` 重导出」
都算成命中**，必须三层过滤。这是本轮实测踩过的坑：

```bash
# 三层过滤：① 排除 tests 目录 ② 排除自身三个文件 ③ 排除 mod.rs（重导出非调用）
for f in design_ab_test judge_ab run_experiment; do
  printf "%-20s " "$f"
  grep -rn "$f" --include="*.rs" neotrix-core/src crates 2>/dev/null \
    | grep -v "/tests/" \
    | grep -vE "(experiment|nt_evolution_eval|nt_evolution_runner)\.rs" \
    | grep -v "/mod\.rs:" \
    | wc -l
done
```

实测（本轮，两轮踩坑后的正确版）：
```
design_ab_test   0
judge_ab         0
run_experiment   0
```

### ⛔ 两次踩坑记录（比结论更值钱）

| 我写的过滤 | 得到 | 真相 |
|---|---|---|
| ① 只排 `tests.rs` | 2 / 4 / 1 | **非零假象** —— 命中全在 `nt_evolution_eval_tests.rs`（该文件名不以 `tests` 结尾） |
| ② 加排三文件，仍留 `/mod.rs:` | 0 / **1** / 0 | **仍有假象** —— 那 1 处是 `mod.rs:76` 的 `pub use judge_ab` 重导出，**不是调用** |
| ③ 三层过滤 | 0 / 0 / 0 | ✅ 与 `coverage-gaps` 门一致 |

> **「导出 ≠ 调用」这条我已在本项目文档里写过多次，但自己写复核命令时
> 还是连踩两次。** ⇒ 这不是知识问题，是**必须用门而不是人肉 grep** 的理由。
> `nt_coverage_gaps.py` 一次就做对了这三层过滤。

## 门怎么说

```bash
python3 scripts/ops/nt_coverage_gaps.py neotrix-core/src 2>/dev/null | grep -A4 nt_evolution
```
⇒ 报 `run_experiment` / `with_repeats` / `prereg_from_hypothesis` /
`outcome_from_regression` 四个 `☠`（零真实调用者）。
**⛔ 不要屏蔽这条告警** —— 它是本文件存在的理由。
