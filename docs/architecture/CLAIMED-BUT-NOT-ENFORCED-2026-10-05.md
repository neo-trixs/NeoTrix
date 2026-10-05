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
| 9 | `main.rs` 的 `--yolo` / `--full-auto` / `--auto-edit` | 写入全局 `ApprovalMode`，但唯一生产读者 `require_approval` 只被 **两条都不在生产链上的路径**消费（`ShieldEnforcer` 悬空 + `turn_stream_with_approval` 零调用方）⇒ **三 flag 在工具执行上零效果** | ⚠️ **已加诚实告警**（`main.rs` 在设 mode 后打印「该模式未被任何生产工具执行路径读取」并指向本表） |
| 10 | `nt_permission_profiles` 的继承合并单调性 | 继承合并是**子档无条件覆盖父档**（`tightened_with` 只在 `set_rule` 用） | ✅ **裁定为「刻意设计」**（见下节） |

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

---

## ⭐ 第 10 项的最终裁定：**刻意设计，不是缺陷**（2026-10-05 留痕）

### 我一度做了什么
把继承合并从「子档无条件覆盖父档」改成 `existing.tightened_with(v)`（单调收紧），
理由是 Codewhale 授权栈第 1 层「项目 overlay **只能收紧**，不能放松」。

### 为什么撤回
实测让 **4 条既有测试变红**，而那 4 条**并不过时**：
`general` 档断言 `write_file → Allow`，而其祖先 `nt_shield` 是 `Ask`。

⇒ 单调合并正确地把它收紧成 `Ask`/`Deny`，
**而这正是 `general` / `developer` 两个档失去存在意义的原因** ——
它们是有意的产品阶梯（`general` = 通用开发；`developer` 注释直写
`most permissive`）。

### ⭐⭐ 教训（本轮最值钱的一条）
**「规则违反了我从外部读来的原则」与「这条原则在这个系统里是错的」是两件事。**

我机械套用了 Codewhale 的「只能收紧」，**没有先验证它的前提**：
Codewhale 能这么写，是因为它**只有一个 profile** + per-project overlay，
用户**没有「切换到宽松档」这个需求**。
⇒ 本仓有「宽松档」这个产品概念 ⇒ 前提不成立 ⇒ 结论不能照搬。

**推广纪律**：外部原则落地前，先问「它的前提在我方成立吗？」——
这与「吸收前置门」同源，但更细一层。

### 现在留下的东西（不是「什么都没做」）
1. `resolve` 里留了**完整的撤回记录与理由**（防止下一个 agent 再「修」一遍）。
2. 新增 **2 条契约锁**：
   · `child_profile_may_explicitly_relax_parent_on_purpose`
     —— 断言 `general` 的放宽**必须真的生效**，且**未被显式覆盖的键
     必须继承祖先**（特别是 `read_secrets` / `git_force_push` 的 `Deny`）。
   · `explicit_deny_in_a_profile_is_never_relaxed_within_that_profile`
     —— 安全下界：任何档位里解析后为 `Deny` 的键必须仍是 `Deny`。

⇒ 「刻意允许放宽」现在是**契约**，不是**未被记录的巧合**。

---

## ⭐ 真正剩余的问题（第 10 项剥离后）

不是「合并无单调」，而是 **`switch_profile` 能改全局审批模式且无任何记录**：
它沿父链继承 `approval_mode_override` 后直接 `engine.set_mode(..)`
⇒ 从 `strict-nt_shield`（`write_file → Deny`）切到 `developer`
会**单次调用**把全局模式改写为 `AutoEdit`，无记录、无单调检查、无确认。

⚠️ **需产品裁决**（不是纯技术缺陷）：是否允许一个档位改变全局审批模式？
若允许 ⇒ 必须**落审计 + 要求显式确认**；若不允许 ⇒ 该字段应从 profile 移除。

---

## ⭐ 本轮的一条工作方法：`rg` 命中同名不同物时，先核「是不是同一个类型」

第 9 项的根因核实过程中，`rg -n '\.check_all\('` 返回了 8 处非测试命中，
看起来「`ShieldEnforcer` 有生产调用方」⇒ 结论本该是「三 flag 有效」。

逐个读那一行才发现**全是同名不同物**：
`monitor.health.check_all()`（健康检查）、`stack.check_all()`（工具栈）。
真正的 `ShieldEnforcer::check_all` 调用方**只有 1 处，且零生产调用方**。

⇒ **命中数不是证据，「那一行引用的是哪个类型」才是。**
（这与文档方法论节里那三处同名不同物是同一类，只是这次是我自己踩的。）

---

## 📌 本会话的收尾状态（2026-10-05 23:20）

### 待落地 2 笔（已验证、已双重兜底，门红在他窗）
| 内容 | 兜底 |
|---|---|
| 撤回错误的单调合并 + 2 条契约锁（`nt_permission_profiles.rs`） | `.neotrix/patches/2026-10-05-contract-locks-monotonic-revert.patch` + 分支 `backup/nt-v5-5b058ce8` |
| 三个无效 CLI flag 的诚实告警（`main.rs`） | `.neotrix/patches/2026-10-05-honest-flag-warning.patch` + 同分支 |

落地命令见 `sessions/handoff-2026-10-05-authorization-audit-and-inert-defenses.md` §8.2。
⚠️ **不要用 `git cherry-pick` 整体应用** —— 共享工作树暂存区里有他窗的改动与删除。

### ⭐⭐ 本会话最该被继承的三条经验

**1. 「规则违反了我从外部读来的原则」与「这条原则在本系统里是错的」是两件事。**
我曾把 Codewhale 的「overlay 只能收紧」套到权限档继承上，
结果 4 条**并不过时**的测试变红 —— 因为本仓**刻意**提供宽松档
（`general` / `developer (most permissive)`），而 Codewhale 只有一个 profile
+ per-project overlay，**前提不成立**。
⇒ **外部原则落地前，先问「它的隐含前提在我方成立吗？」**
这比吸收前置门更细一层：前置门问「危害是否存在」，这条问「前提是否成立」。

**2. 「有测试」不等于「在跑」，「被文档引用」不等于「有效」。**
本会话修的 3 处硬拒缺陷，根因全是同一件事：规则被声明、被测试、被文档引用，
但**从未在生产路径上被求值**。
⇒ 而**验「有没有在编译树里」，一次 `cargo check` 就够了** —— 比任何搜索都快、都可靠
（`GuardrailCategory` 那个 E0277 就直接证明了它从未被编译）。

**3. `rg` 的命中数不是证据，「那一行引用的是哪个类型」才是。**
本会话实测到**五处同名不同物**：`health.check_all()` / `stack.check_all()` /
`unified_defense.validate_input()` / `p.run_pipeline()` / `e.into_inner()`。
其中 `check_all` 那次让我差点得出「`ShieldEnforcer` 有生产调用方」的反向错误结论。

---

## 📌 补记（2026-10-06）：第 10 项的**裁决落地**与第三类并发并发病

### 第 10 项已由用户裁决并实现（`96056d65`）
**方案 A**：允许档位改变全局审批模式，但必须**显式确认 + 留痕**。
落地要点（详见 `sessions/handoff-2026-10-05-authorization-audit-and-inert-defenses.md`）：
· `plan_profile_switch`（**纯查询**）⇒ 调用方先看副作用再决定要不要问用户
· `switch_profile_with_audit(name, actor)`，**空 actor 被拒**
· ⭐ **匿名 `switch_profile` 在会改模式的档位上直接 `Err`**
  ⇒「能在无 actor 情况下放宽审批的路径」必须不存在
  （与「Deny 不可被 Ask 覆盖」同属**不可逆性保护**）

---

## ⭐⭐⭐ 第三类并发并发病：**测试之间**抢全局状态

前两类我已记录：
1. 共享 index 的「暂存区与提交不原子」（2026-09-29 实测事故）
2. 他窗 WIP 把门挡住（本会话反复遇到）

第三类是 **`cargo test` 默认多线程 + 模块级全局单例**：

`nt_permission_profiles` 有 13+ 处测试共用
`global_profile_manager` / `global_approval` 两个单例
⇒ 同模块测试互相改状态 ⇒ **间歇性失败，且失败行号漂移**。

### ⭐ 正确取证顺序（我这次差点搞反）
新写的锁④「`plan_profile_switch` 不得改动全局审批模式」首跑就红。
**第一反应**会是「`plan` 有副作用，去修它」—— 那是**错的**。

**先做单线程探针**：
```
PROBE after setup mode=Suggest active=nt_shield
PROBE after plan  mode=Suggest      ← plan 确实是纯的
```
⇒ **真因是测试抢全局态。**

依据同AGENTS.md §5 的 R-SCAN-1：扫描/测试告警先读现场证实或证伪再动代码。
**测试红了不等于被测代码错** —— 在共享单例的模块里，它更可能是测试自己抢了。

### 修法（本仓现状）
无 `serial_test` dev-dependency ⇒ 用标准库 `Mutex` 手写串行化
（`TEST_GLOBAL_STATE`，**中毒取内值** —— 守卫本身不含状态，
中毒只意味着另一个测试 panic 过，不该连带阻断本测试）。

**实证必要性**：去掉互斥锁后连跑两次 ⇒ 第一次 `FAILED(18/1)`、第二次 `ok(19/0)`。

⚠️ 若将来引入 `#[serial]`，**删掉这个手写锁**（双重串行化无害但会让人困惑）。

### 推广
任何「模块内有全局单例」的测试集都适用这条：
**先问「是不是测试之间在抢」，再问「是不是代码有 bug」。**
判别手段就一个：**单线程跑一遍**。
