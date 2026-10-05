# 「声称存在但实际不生效」的防线清单 — 2026-10-05

> **判据**：文档/注释声称某个保护在生产生效，而代码实测**不求值**或**无法命中**。
> 本仓最贵的一类缺陷 —— 它不是「缺功能」，而是**让人以为防护在位**。
> **核实时间**：2026-10-05（全部经编译/测试/引用搜索实证）。
> 📌 **按符号名引用，刻意不写行号** —— 行号锚点必腐化（AGENTS.md §0）。

---

## 为什么要单独列一张表

本轮修的三处硬拒缺陷（`nt_approval` 的 `Deny`、`ActionSandbox` 的 tie-break、
`nt_policy` 的 early-return），**根因都是同一件事**：
规则被声明、被测试、被文档引用，但**从未在生产路径上被求值**。

⇒ 「有测试」不等于「在跑」；「被文档引用」不等于「有效」。

---

## 清单

| # | 声称 | 实测 | 状态 |
|---|---|---|---|
| 1 | `nt_approval`：`read_secrets` / `git_force_push` 是 `Deny` | 键永不相交（`action_type_to_key` 只产 `git_push`）⇒ **从无 `ActionType` 能命中**；且 `Deny` 被 `bool` 压成「需审批」 | ✅ **本轮已修**（`0b5358cb`） |
| 2 | `ActionSandbox::new()`：预置硬拒前缀 `rm:`/`wipe:`/`destroy:` | 同长度时**后注册的 allow 胜** ⇒ 一次 `add_rule` 即可覆盖 | ✅ **本轮已修** |
| 3 | `nt_policy`：路径检查后的 `return Allow` 是安全的 | 它**跳过**了 bash 逃逸检查 ⇒ 多余一个 `path` 参数即绕过 | ✅ **本轮已修**（`bad544f9`） |
| 4 | `agent_guardrails/mod.rs`：`R-P132: Guardrails are non-optional in production.` | 已挂载、34 测试全绿，但 **`PolicyEngine` 在目录外零引用** ⇒ 不求值 | ⛔ **未接生产**（见下） |
| 5 | `ShieldEnforcer::check_all`：8 段授权链 | 唯一非测试调用方 `nt_sandboxed_shell` **零生产调用方** | ⛔ 悬空 |
| 6 | `shield_core::SecurityManager::inspect_tool`：5 层 + 22 条规则 | **零调用方**（连测试都没有）；`attach_safety_kernel` 零调用方 ⇒ `execution_guard` 恒 `None` | ⛔ 悬空 |
| 7 | `ProjectLaws::check_laws`（仓库法） | 注释自承 **"non-blocking by default"** | ⛔ 非阻断 |
| 8 | `PolicyConfig::default()`：`blocked_input_patterns` 为 `vec![]` | 空 —— 凭据/注入/外泄三组正则**默认全不生效**，只靠 `injection_patterns`/`exfil_patterns` 那些内置项 | ⚠️ 待核 |
| 9 | `main.rs` 的 `--yolo` / `--full-auto` / `--auto-edit` | 写入全局 `ApprovalMode`，但 **`run_one_shot` / `run_interactive_with_ephemeral` / `run_headless_mode` 都不走带审批的那条路** ⇒ **三个 flag 在工具执行上零效果** | ⛔ 语义落空 |
| 10 | `nt_permission_profiles` 的继承合并单调性 | `tightened_with` 只在 `set_rule` 用；继承合并是**子档无条件覆盖父档** ⇒ `developer` 档可抹掉 `nt_shield` 的收紧 | ⛔ 单调性空转 |

---

## ⭐ 方法论：如何验「声称 vs 实际」

**别只 grep 方法名** —— 本轮实测到三处**同名不同物**（L15）：

| grep 命中 | 实际属于 | 结论 |
|---|---|---|
| `self.unified_defense.validate_input(input)`（`nt_shield_enforcer.rs` 的 `UnifiedDefenseLayer`） | `UnifiedDefenseLayer` | **不是** `agent_guardrails::InputValidator` |
| `p.run_pipeline(&mut el)`（`pipeline_autofixer.rs` 的 `PipelineAutoFixer`） | `PipelineAutoFixer` | **不是** `PolicyEngine` |
| `f(act).unwrap_or_else(\|e\| e.into_inner())` | `std::sync::PoisonError` | 与 `ApprovalDecision` 无关 |

**正确判据（三步，缺一不可）**：
1. **排除自身目录与 `tests.rs`**，按**类型名**搜（不按方法名）；
2. 逐个读命中那一行，确认它引用的是**哪个类型**；
3. 排除 re-export（`pub use` 只是让符号可见，**不代表被调用**）。

### ⭐ 反例：编译器比搜索更可靠
`agent_guardrails/mod.rs` 里
`GuardrailCategory` 原本 derive 了 `Copy`，而它含 `Custom(String)`
⇒ **E0277**。这说明该目录**从未被编译过**
⇒ 「有没有编译」这件事，**一次 `cargo check` 就有答案**，
比任何搜索都快、都可靠。

⇒ 推广：**怀疑某段代码没在跑时，先 `cargo check` 确认它在编译树里，
再查消费者。** 顺序反了会浪费大量时间在「它到底有没有被调用」上。

---

## 下一步（待裁决，非本轮已做）

| 优先 | 动作 | 判据 |
|---|---|---|
| P0 | 把 `PolicyEngine` 接成 `ShieldEnforcer::check_all` 的**第 9 段** | 必须在同处接，**不另建平行入口**；且 `GuardrailVerdict::Block` 要真正阻断（别学 `check_laws` 的 non-blocking） |
| P1 | `--yolo` / `--full-auto` 要么接到实际路径，要么**明确报「该 flag 当前无效」** | 静默无效比报错更糟 |
| P1 | 核实 #8：`blocked_*_patterns` 空默认值是否有意 | 空 ⇒ 三组正则不生效，需在文档写明 |
| P2 | `developer` 档的 `most permissive` 语义与「overlay 只能收紧」冲突 | 需裁决：保留 developer 档，还是删掉它的覆盖能力 |
