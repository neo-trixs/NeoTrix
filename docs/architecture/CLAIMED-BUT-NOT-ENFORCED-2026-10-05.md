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
| 9 | `main.rs` 的 `--yolo` / `--full-auto` / `--auto-edit` | 写入全局 `ApprovalMode`，但唯一生产读者 `require_approval` 只被 **两条都不在生产链上的路径**消费（`ShieldEnforcer` 悬空 + `turn_stream_with_approval` 零调用方）⇒ **三 flag 在工具执行上零效果** | ⚠️ **2026-10-06 已收敛**（`2169b633`）：三个 flag 降为 `--approval-mode <suggest\|auto-edit\|full-auto>` 的**别名**，解码抽成 `resolve_approval_mode` 并加 7 条**「启用即断言接线」**测试。⚠️ **本项仍未闭环** —— `require_approval` 那两个消费者依旧不在生产链上 ⇒ **「落地 ⇒ 被消费」那一段仍断** |
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

### ⭐⭐⭐ 第 9 项：已从「未闭环的 TODO」升级为**已实测的可达性结论**（2026-10-06）

此前本项只写「两个消费者不在生产链上」。本轮**逐层实测**后，结论具体到「哪一段闸
在哪个档位下可达」—— 因为**笼统的「未闭环」无法指导下一步**。

#### 实测链路
生产工具执行路径是 **`crates/neotrix-neobot/src/nt_agent.rs::execute_tool`**，
它由 `neobot` 二进制与 `nt_channel_dispatch.rs` 调用。
它的闸是 `nt_policy::evaluate_policy` —— **`PolicyDecision` 只有
`Allow` / `Deny` 两态，没有 `Ask`** ⇒ `--approval-mode` 在这条链上**无绑定点**。

#### 三条实测的硬约束（不是「还没做」，是「这样做会出事」）
1. ⛔ **`Ask` 在无 UI 路径上无法执行**。
   `nt_io_neocodex/agent/nt_agent_exec.rs` 已实测记录：
   *「实测它在默认 `Suggest` 模式下对**每一条**命令（含 `echo hello`）都返回
   `RequireApproval`，而这条路径**没有审批 UI** ⇒ 等于把 shell 功能 100% 关掉」*，
   并注明 `nt_sandboxed_shell::execute_guarded` **零消费者，很可能正因此被搁置**。
   ⇒ **本轮特意没有**把 `Ask` 接进 bot 路径：那会**复现同一事故**。
2. ⛔ **依赖方向禁止**：`neotrix-neobot` **不依赖** `neotrix-core`，
   而 `neotrix-core` **依赖** `neotrix-neobot`
   ⇒ bot 进程**根本看不见** `l6_meta::global_approval()`。加依赖会成环。
3. ⛔ **粗闸先行**：`ShieldEnforcer::check_all` 的**第 1 段** `SecurityGuard`
   在 `Suggest`/`AutoEdit` 档对**一切**动作返回 `RequireApproval` 并**提前 return**
   ⇒ sandbox 段（第 4 段）**只在 `FullAuto` 档可达**（已加测试钉住）。
   ⇒ 想让 sandbox/审批链在默认档生效，**必须先处理这道粗闸**，
   而那是一次**朝宽松方向**的改动 —— 不该由「接线」任务顺手做掉。

#### 本轮确实闭环的部分
· ✅ `action_verdict` 曾**丢 AutoEdit 的文件类白名单**（已修 + 穷举一致性锁）
· ✅ `--sandbox` 曾写进**无人读的单例**（已修：`init_sandbox` 同时推进
  `global_shield()` 的活对象，两侧默认都是 `Disabled` ⇒ 不传 flag 时零行为变化）

#### ⛔ 仍未闭环（**不要因为上面两条✅就把本项划掉**）
「`--approval-mode` 的 `Ask` 档」在**生产工具执行链上**仍无绑定点。
**唯一诚实的收口路径**（都不是「接线」，需要独立立项）：
· 给 bot/channel 路径一个**审批通道**（聊天本身就是通道），或
· 把 profile 档位下移到一个 bot 也能依赖的 crate（`neotrix-types`），或
· 先把第 3 条那道粗闸改成有判别力的形态（**朝宽松方向，需裁决**）

### ⭐ 第 10 项已从「库函数」变成「命令行可达」（`145c33d7`）
⚠️ 上一笔实现完 `switch_profile_with_audit` 后我核对发现：它**零生产调用方**
⇒ 库函数写好、测试全绿，而命令行**根本够不着** ⇒ 等于没有。
本笔新增 `neotrix profile list|show|use|current`，其中：

· ⭐⭐⭐ 安全边界做成**纯函数** `authorize_profile_use`
  | 改审批模式 | TTY | `--yes` | 裁决 |
  |---|---|---|---|
  | 否 | 任意 | 任意 | `NoConfirmationNeeded` |
  | 是 | 是 | 任意 | `NeedsInteractiveConfirm` |
  | 是 | 否 | 是 | `AllowNonInteractive` |
  | 是 | 否 | 否 | **`Refuse`** |

  ⛔ 若该判据内联在 I/O 里就**测不到** ⇒ 边界改松了没人知道
  （与第 9 项同一种病：flag 存在、能编译，行为却不受约束）。
· ⭐ **最后一格比 `claude-code` 更严**：`claude-code` 无对话框时**直接放行**，
  我们**拒绝**。理由：它的默认是放行、我们是拒绝
  ⇒ **静默降级只允许朝严格方向**；否则 CI 里 `profile use developer`
  会悄悄把审批降到 auto-edit 且日志零痕迹。
· ⭐ actor 区分人/机：`cli-confirmed` / `non-interactive` / `cli`
· ⭐ 反向锁 6 条，含**穷举 (TTY × --yes × 是否改模式) 全 8 格**
  ⇒ 逐格测试容易漏的那格，恰好就是「CI 里静默放宽审批」那条路径
· ⚠️ **夹具坑（R-SCAN-1 实例）**：macOS `script -q /dev/null cmd`
  **不把管道输入送进子进程 pty**（`^D` 早于输入到达）
  ⇒ 首轮测出「输入 `y` 也被取消」的**假阳性**。
  按「先读现场证实/证伪」改用 python `pty` 精确驱动后确认**代码无缺陷**。
  **不修正它就会去「修」正确的代码。**

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

---

# 追加：2026-10-06 系统性裁决 —— 用 `nt_pub_dead.py` 扫「导出≠调用」

用户授权按实际代码裁决。本轮用**本仓自有门**做系统性取证
（`nt_lock_audit` 0 ✓ · `check-layer-deps --strict` PASS 0 new · `nt_pub_dead`）。

## 一、修掉的最后一处长期红项（**裁决：测试陈旧，不是代码有 bug**）

`mod_orphan::tests::scan_tree_sorted_by_lines_desc` 自 2026-10-05 起一直红。
读现场后裁决**扫描器是对的、测试是陈旧的**，三条证据：
1. `OrphanKind::DirModule` 注释明写「**2026-10-05 新增**」⇒ 有意的能力；
2. 模块文档说明**非有它不可**：`dual_track/mod.rs` 这类**目录模块本身**就是那个
   未被声明的文件，旧实现扫出来是空 ⇒ **完全看不见**；
3. 实测输出 `big`(5行)→`small`(1行)→`s`(0行) **正是降序**，与
   `sort_by(|a,b| b.lines.cmp(&a.lines).then(a.stem.cmp(&b.stem)))` 一致。

⚠️ **为什么必须裁决而不是「改到绿」**：把 `DirModule` 报成 bug 删掉也能让断言变绿，
但那是**删掉刚加的正确能力**。陈旧红项的真正危害就在这里 ——
它训练所有人忽略红色，并诱导下一个人去「修」**正确的**代码。
⇒ 改测试，并把它变成**同时锁住新能力**的测试。

**结果：全量 `cargo test -p neotrix --lib` ⇒ 13395 passed / 0 failed**
（本次会话首次全绿）。

## 二、⭐ 新发现：`nt_shield_ztnet` 整个子系统**零外部消费者**

| 项 | 实测值 |
|---|---|
| 规模 | **26 个 `.rs` / 5077 行** |
| 挂载 | ✅ `nt_shield/mod.rs:41 pub mod nt_shield_ztnet;` |
| 仓内外部消费者 | **0** —— 只被**自己内部文件**与 `error_conversions.rs` 引用 |
| 测试 | 19 处 `#[test]` ⇒ **被认真测过** |
| orphan 门能否抓到 | ❌ **不能** —— 它已挂载，故不在 orphan 基线里 |
| 内容 | WireGuard 密钥轮换（`REKEY_AFTER_MESSAGES = 2^64−2^16−1` / 120s、REJECT 180s、COOKIE 轮换）+ Noise-IK 握手，引 **NDSS 2024** |

`_decide_rekey` / `_RekeyState` / `_RekeyAction` 三者**全仓零引用**，
且用 `_` 前缀（Rust 约定 = 故意未用，故不触发 `dead_code` 警告）。

### 裁决：**不删**（这是本仓已错过 3 次的坑）
「导出 ≠ 调用」在本仓已被记为**反复误判来源**（`DIR-REMEDY` §2.5 明确
`nt_jev` + `nt_crystal_core` 是**活路径**，勿当死代码删）。
⇒ 这 5077 行是一套**被认真测试过的**完整实现，属「**已就绪、待接线**」，
不是死代码。**接线它是独立立项**（需要决定 ZTNet 是否进入生产网络路径）。

## 三、⭐⭐ 顺带暴露的**扫描器盲区**（比上面那条更值钱）

`nt_pub_dead.py` 是**逐项**统计引用的 ⇒ 一个**自引用的孤岛**
（岛内 A 引用 B、B 引用 A）在它眼里「全部有引用 ⇒ 活着」。
ztnet 正是这种形状：全岛 5077 行互相引用，**逐项零引用只有 3 条**，
但整岛**外部消费者为 0**。

⇒ **缺的检查是「模块级可达性」**：从生产入口出发，这个子系统**可达吗**？
⚠️ 逐项零引用**无法回答**这个问题。
⇒ 这是本轮最有复用价值的产出：下一门应当是**可达性门**，而不是再加一个逐项扫描器。

## 四、其余取证结论（避免重复评估）
· `nt_lock_audit.py neotrix-core/src` ⇒ **可疑 0 处**（rc=0）
· `check-layer-deps.sh --strict` ⇒ **PASS: 0 new**（13 条 known/recorded）
· 全仓 `pub` 零引用候选 **1866 条**（const 109 / fn 1753 / static 4）。
  抽样核实 3 条（`PHI_MIN_THRESHOLD` / `DEFAULT_SIZE` / `COL_U32`）
  ⇒ **全仓只有定义行、确属真零引用**，扫描器**无误报**。
  ⚠️ 但 **1866 条不可逐个改**（多为文件内配置常量）⇒ 正确做法是
  **棘轮基线**（只拦新增），不是一次性清理。
