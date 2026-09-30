# 裁决材料 · 第二棵树最后 2 个模块（2026-09-30）

> **本文只取证，不含任何改动。** 两个模块都已勘察完毕，等你选处置方式。
> B 方案进度：6/8 已完成，`second-tree` 130 → **56 文件**（减 57%）。

## 背景：为什么这两个不能照前 6 个的流程做

前 6 个（`proxy_daemon_wrapper` / `nt_capability_bridge` / `ffi` / `nt_jev` /
`nt_file_ability` / `nt_core_error`）都能「搬 + 改引用 + 跑门」一把过，
因为它们**搬迁后不产生新的门违规**。

**这两个不同**：搬迁会让 `check-layer-deps` **第一次看见一批既存违规**。
不是搬坏了，是**它们一直违规，只是路径写法恰好躲过了门的匹配式**（详见 §3）。

---

## A. `nt_crystal_core` · 52 文件 · 预计暴露 **22 处**

### A.1 引用面（实测）

| 消费者位置 | 文件数 |
|---|---:|
| **`l1_action/`（L1）** | **7** |
| `l5_cognition/` | 2 |
| `bin/` + `entry/` | 5 |
| `nt_crystal_core/` 内部 | 8 |
| 合计 | 22 个文件 |

### A.2 违规预测（**用门的真实正则 + 门的真实注释过滤器模拟，非手推**）

把 7 个 L1 文件里的 `crate::neotrix::nt_crystal_core::`
替换成 `crate::l5_cognition::nt_crystal_core::` 后重跑门逻辑：

| 文件 | L1→l5 违规 |
|---|---:|
| `l1_action/nt_tui_app.rs` | **13** |
| `l1_action/nt_core_task_dispatcher/nt_dispatcher_core.rs` | 3 |
| `l1_action/nt_dialogue_tui.rs` | 2 |
| `l1_action/nt_model_cli.rs` | 1 |
| `l1_action/nt_free_pool.rs` | 1 |
| `l1_action/nt_crystal_llm_bridge.rs` | 1 |
| `l1_action/nt_stdin_human.rs` | 1 |
| **合计** | **22** |

⇒ `--strict` 会从 `PASS: 0 new` 变成 `FAIL: 22 new`。

### A.3 三个选项

| 选项 | 做法 | 代价 | 后果 |
|---|---|---|---|
| **A-1** | 搬 + 把 22 处**记入 baseline** | 小 | 棘轮 12 → 34 条。债务被看见并锁住，但**架构问题不解** |
| **A-2** | 搬 + **修正 L1 的依赖**（把 TUI/CLI 需要的类型下沉，或改走 facade） | **大** | 真正修复，但要先设计下沉边界；可能触及 22 处 + 它们的调用链 |
| **A-3** | 搬 + **重新裁决 `nt_crystal_core` 的层归属**（它其实是不是 L5？） | 中 | 若它被判定为 L1，22 处违规**自动消失**。需读 52 个文件判断它真实语义 |

**我的倾向：A-1 先记账保进度 + A-3 并行做语义判定**。
理由：22 处集中在 4 个文件且形态高度同质（`NtLlmAsk` / `NtDemand` 这类
**数据结构 + trait**），A-2 很可能只是「把几个类型搬到 L1」的机械工作，
但**必须先有人确认它们该在 L1**，否则就是把债换个地方。

⚠️ 我**没有**读那 52 个文件来判断 `nt_crystal_core` 是否真是 L5 ——
**这需要你的领域判断，我不应自行假设**（R-SCAN-2）。

---

## B. `nt_core_event_bus` · 666 行 · 已知 **2 处**违规

### B.1 4 处逐行取证（`nt_core_event_bus.rs`）

| 行 | 违规对象 | 是真 API？ |
|---:|---|---|
| 6 | `use crate::l5_cognition::nt_core_dispatch::Dispatcher;` | ✅ 定义在 `entry/exec.rs:8` |
| 111 | `crate::l3_embodiment::nt_shield::shield_core::redaction::redact_json_line` | ✅ 定义在 `l3_embodiment/…/redaction.rs:170` |
| 337 | `crate::l5_cognition::…::CONSCIOUSNESS_THRESHOLDS` | ✅ 定义在 `l5_cognition/…/mod.rs:49` |
| 407 | 同上（第二处） | 同上 |

⇒ **4 处全是真引用**，且 3 个目标符号**都定义在更高层**。
（我一度判 337/407 是误报，实际那两行确实含 `l5_cognition::…::CONSCIOUSNESS_THRESHOLDS`。）

### B.2 关键观察：违规的性质是**「全局事件总线引用高层」**

它被登记为 `l0_substrate`（全局基础设施），却引用 L3（脱敏）与 L5（阈值/调度）。
这**在语义上是可疑的**：一个 L0 的事件总线为什么要知道「意识质量阈值」？

### B.3 三个选项

| 选项 | 做法 | 代价 |
|---|---|---|
| **B-1** | 搬到 `l0_substrate/` + 4 处**记入 baseline** | 小。诚实记账，但「L0 依赖 L5」这个**架构异味留存** |
| **B-2** | 搬 + 把 `CONSCIOUSNESS_THRESHOLDS` **下沉到 L0** | 中。阈值常量本无层属性，下沉合理；但要查它有多少消费者 |
| **B-3** | 搬 + 把 `nt_core_event_bus` **重新归到 L5** | 中。但它是「总线」语义，归 L5 可能让 L0 的消费者反向依赖 L5 ⇒ 需先查它的消费者 |
| **B-4** | 搬 + 用 **facade 反转依赖**（L0 定义 trait，L5 实现注入） | 大。最干净，但要设计 trait 边界 |

**我的倾向：B-2**。理由：
- `CONSCIOUSNESS_THRESHOLDS` 是**纯常量**，无层属性 —— 下沉到 L0 无副作用；
- `:111` 的 `redact_json_line` 与 `:6` 的 `Dispatcher` 更麻烦，但**只有 2 处**，
  可以逐个处理（一个改走注入，一个改走 facade）。

### B.4 前置数据已补齐（2026-09-30 实测）

`CONSCIOUSNESS_THRESHOLDS` 的真身：
```rust
// l5_cognition/nt_mind/nt_mind_background_loop/run.rs:70
pub static CONSCIOUSNESS_THRESHOLDS: LazyLock<_ConsciousnessThresholds> =
    LazyLock::new(_ConsciousnessThresholds::default);
```
经 `mod.rs:49` 的 `pub use` 转出。

**消费者 5 个文件 / 7 处**：
| 文件 | 处数 |
|---|---:|
| `neotrix/nt_core_event_bus.rs` | 2 |
| `…/nt_mind_background_loop/run.rs`（真身） | 1 |
| `…/nt_mind_background_loop/mod.rs`（`pub use`） | 1 |
| `…/handlers_consciousness/nt_event_bus.rs` | 1 |
| `…/handlers_consciousness/nt_consciousness_tick.rs` | 2 |

⇒ **下沉只需改这 5 个文件的 import 路径**，不动任何逻辑。
但**它是一个 `static LazyLock` 全局单例**，不是纯 `const` ——
下沉到 L0 时要确认 L0 不引入新的依赖面（`LazyLock` 来自 `std`，无问题）。

**B-2 的可行性因此提高**：改动面 = 5 文件 7 处 import + 1 处 `mod.rs` 声明。

---

## 需要你给的决策

| # | 决策点 | 我的倾向 |
|---|---|---|
| 1 | `nt_crystal_core` 走 A-1 / A-2 / A-3？ | **A-1 + 并行 A-3** |
| 2 | `nt_crystal_core` 真实层归属是 L5 吗？（**只有你能判**） | 待你判 |
| 3 | `nt_core_event_bus` 走 B-1 / B-2 / B-3 / B-4？ | **B-2** |
| 4 | 是否先补查 `CONSCIOUSNESS_THRESHOLDS` 的消费者数（B-2 前置）？ | 建议查 |

## 无论选哪个，两条纪律都适用

**① 搬迁后必须逐侧验三态**（本会话已栽 4 次）：
```sh
git ls-tree -r HEAD <旧目录> | grep -c <模块名>   # 必须 0
```
**不能只看 `cargo check` 通过** —— 无 `mod` 声明的孤儿不参与编译，编译永远绿。

**② 禁止 `git add -A` / `git add .`**：本会话已因它误删他窗 5 个图标。
即使写成 `git add -A <path>`，**`-A` 的作用域仍是全树**。
只能 `git add <目录>`（无 `-A`）或 `git add <逐个文件>`。
