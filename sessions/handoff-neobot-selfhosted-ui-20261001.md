# Handoff — 商用自持前端（neobot-ui）重构收尾

## 1. 会话标识

- 日期：2026-10-01
- 分支：`feat/capability-absorb-20260828`
- 窗口角色：主开发代理（本会话）；同期有**另一窗口**推进桌宠/原生菜单（见 §7）
- 本会话提交（自 `005270ed` 起）：`005270ed` `ede6ea7f` `7940f13c` `7600bdbf`
  `5982ffc7` `6f9a8c21` `44c28264` `f79f53a3` `ccac66b4` `c160d781` `8d0821a1`
  `8e0cb2ec` `5b1031b8` `c94aef49` `72a2e62b` `cb1a2baa` `550fe19c` `45276943`
  `8cf6063c`

## 2. 目标（一句话）

把 NeoTrix/NeoBot 从「依赖受限 dsh-tauri vendored 前端」改为
**零 vendored 依赖的自持前端**，并为「下架哪些功能」建立可复算的判据。

## 3. 已完成

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M3 API 契约门 | ✅ | `nt_api_contract.py`：契约 118 / 注册 74 / 前端 94 三方一致，A=B=C=D=0 |
| R5 openghost 迁出受限树 | ✅ | 许可门**独立**审计两处 openghost，均判「无附加条款」 |
| R1 自持入口 | ✅ | `neobot-ui/src/main.tsx` + 入口已切 `tauri.conf.json` |
| R2 自研外壳 | ✅ | 顶栏/语言/日志/退出 + `open_external_url`，a11y 4 项实证 |
| R4 i18n | ✅ | 56 键自有词条（与上游 452 键**零重合**），聊天区实测随语言变化 |
| 功能存活性门 | ✅ | `nt_feature_viability.py`：无静默丢弃 |
| 运行时冒烟 | ✅ | headless Chrome：真渲染 + 4 个失败场景全诚实报错 |

**实测规模**：自持树 **14 文件 / 1,550 行**（vendored 为 134 文件 / 15,647 行）。

## 4. 正在改的文件（关键！逐个列）

### 4.1 曾在工作树、现已由另一窗口提交时一并带走（**订正 4.2 之前的记录**）

| 文件 | 现状（2026-10-01 实测） |
|---|---|
| `apps/neobot-desktop/tauri.conf.json` | **已入库**（`frontendDist: neobot-ui/dist` 等 3 项随之落库） |
| `apps/neobot-desktop/frontend/src/neobot-root.tsx` | **已入库**（含我的**剪贴板降级修复**，文件内 `write_clipboard_text` 2 处） |

⚠️ 我写这份交接时它们还是未提交状态，随后被另一窗口的提交一并带走。
**这是共享 index 的第三次互相吞并**（前两次见 §7）。**均未丢内容**，
只是提交归属与他窗混在一起。接手会话**不必再去找这两处未提交改动**。

⚠️ 另一处仍需留意：`tauri.conf.json` 里他窗的 `beforeBuildCommand`
若仍是裸 `pnpm run build`，那是**坏的** —— Tauri 在 `tauri.conf.json`
所在目录（`apps/neobot-desktop/`）执行该命令，而那里**没有 package.json**。
我写的正确形式是 `pnpm --prefix neobot-ui run build`。

## 5. 下一步（2026-09-30 晚更新；⛔ 上一版的多数条目已完成）

**已完成（原 §5 的条目）**：R1/R2/R4/R5 全部落地并经运行时验证；
入口已切 `tauri.conf.json`；浅色盘接活为可切换主题；
会话列表与消息列表**均已虚拟化**并各有常驻门。

**仍待做**：

1. ⛔ **删 `apps/neobot-desktop/frontend/src/vendor/`**（`check-license` 转绿的最后一步）。
   ⛔ **必须等另一窗口收工**：该树尚有他窗在途工作，删它=销毁别人的工作
   （AGENTS.md 记载的 2026-09-28 事故）。
2. **收敛 `nt_feature_viability.py` 的 `pet → DROPPED`** —— 该条已过期
   （另一窗口已提交自研桌宠 `efe75ded`），但按用户指示「由他窗自己改」，
   我只加了 🚩 标记。**不要由旁人改。**
3. **`get_dsh_theme` 的 DSH 命名债** —— 自持 UI 依赖一个 DSH 命名的 API。
   改名涉及 Rust + 契约表，**须两侧同时改**，前端单方面改会造成两套真源。
4. **补 trendshift 台账元数据**（375 条仍无 license）—— 工具
   `nt_absorption_enrich.py` 就绪，GitHub 未认证配额 60/h，需增量跑数小时。
5. `api-panel.ts` 的硬编码文案未迁 i18n。

## 6. 阻塞点

- **无授权阻塞**：用户已明确「无需取得 dsh-tauri 书面授权」，走路径 B。
- 唯一硬阻塞是**共享工作树**：`frontend/` 尚需保留到他窗收工；`tauri.conf.json`
  与 `neobot-root.tsx` 因混有他窗改动而无法由我提交。

## 7. 给接手会话的话

**⚠️ 共享 index 会让彼此的提交互相吞并——本会话双向都发生过：**

- 我用 `git add -A apps/neobot-desktop/neobot-ui`，把他窗正在写的
  **自研桌宠 4 个文件**卷进了我的提交（已 `reset --soft` 回退重做，他的文件已
  回到未跟踪状态，工作树未丢东西）。
- 随后他窗的 `17fe0f05` 又把我未提交的**冒烟桩修复**合进了他的提交。

⇒ **一律显式列文件 + `git commit --only <我的文件>`，永不用 `-A`。**
（本仓已因该模式丢过 850 处未提交改动，见 `AGENTS.md` 收工义务。）

**⚠️ `pet → DROPPED` 已过期，但按用户指示「由他窗自己改」，我未擅改，只加了 🚩 标记。**
保留该条目的**唯一目的**是防止后来人读到「pet 已下架」而**删掉那段能跑的自研代码**
（`neobot-ui/src/pet/`，commit `efe75ded`，覆盖 5 个 pet 命令中的 4 个，
缺 `list_preset_pets`）。

**本会话反复验证的纪律**（每条都真的栽过）：

- 「构建绿」≠「产物可用」：vite 默认 `base:'/'` 在非根路径下渲染失败（子路径下
  `#root` 为空），而 build/tsc **全绿**。已用 `base:'./'` + 门第 3 项守住。
- 「渲染通过」≠「真的渲染」：`addInitScript(fn, arg)` 只接受**一个** arg；
  Node 侧词表闭包在浏览器里是 `undefined` ⇒ 每次 invoke 抛 ReferenceError，
  而 UI 把错误 `.catch` 掉**照样渲染** ⇒ 测的是**错误态**。
- 「有提示」≠「探针触发了故障」：失败路径若跳过动作，`set_language` 这类
  **只在用户动手时发生**的故障从未被触发。
- 门转红先归因「**谁不完整**」：他窗的 `listen('macos-menu-action')` 是对的，
  缺 `__TAURI_EVENT_PLUGIN_INTERNALS__` 的是**我的桩**。
- 「同名 ≠ 同一符号」：上游 `frontend/src/main.tsx` 与我方
  `neobot-ui/src/main.tsx` 同名但内容不同；按文件名判归属会掩盖
  上游入口依赖 `get_dsh_theme`（DSH 概念泄漏）。

## 8. 收工自查

- **worktree 去向**：本会话**未创建任何 worktree**（全程在主工作树，
  `git worktree list` 无我新增项）。
- **未提交改动的去向**：**本会话产出已全部入库**。
  其中 `tauri.conf.json` 与 `frontend/src/neobot-root.tsx` 曾因混有他窗改动
  而由我保留在工作树，后被另一窗口提交一并带走（见 §4.1）—— 非弃用，无遗失。
  接手会话**无遗留未提交改动需要认领**。
- **门状态（2026-09-30 晚 实测，七道）**：
  `nt_api_contract` 0 · `nt_neobot_ui_wiring` 0 · `nt_feature_viability` 0 ·
  `nt_absorption_audit` 0 · `runtime smoke` 0 · `neobot-msg-virtual` 0 ·
  `check-license` **1（正确**：受限 vendored 树仍在仓库，删树后转绿）。

## 9. 补记：本会话后半段的**元教训**（比功能清单更重要）

我在**同一个文件里反复栽在「用字面匹配改结构」**上，共 5 次：
作用域错误 ×2（把 `await page.evaluate` 写进报告循环，`page` 只在 `probe` 内有）、
括号失衡 ×2（在 600 行文件里按缩进切片删 JSX）、位置选择器 ×1
（加主题选择器后，语言探针静默改去操作它 ⇒ 报「语言切换坏了」而**产品没坏**）。

每一次都是被 `tsc --noEmit` 或**新写的门**抓到的。归纳出三条：

1. **判据与实现要同时设计。** 我曾先改代码再补判据，结果判据与场景打架
   （千条消息时应用正确地停在底部，我却拿「首条可见」去测），
   只能回退重做。⇒ **先定判据，再写实现。**
2. **一个门只管一件事。** 往 600 行的冒烟文件里塞断言导致反复失衡，
   且一度出现「报告层被 `if (mvz)` 跳过 ⇒ rc=0 其实是**没跑**」——
   **比红更坏**。⇒ 拆成独立小门（`neobot-msg-virtual.mjs`）。
3. **门必须先证明它会红。** 我栽过「绿色的谎言」：
   正则写成 `[\b]t\(`（字符类里的 `\b` 是**退格符**不是词边界）
   ⇒ 扫出 0 个键却判 PASS。已加最小数量断言兜底。
   同理，**负向测试抓到了我自己上一轮的错误结论**：
   我声称「窗口化后不能用 `scrollTop = scrollHeight`」，
   注入后门仍 PASS ⇒ 浏览器会把它钳到底部，该说法**夸大**，已订正。

⚠️ 最后一件事：用户问「吸收了多少项目」时，我能给出的**唯一诚实答案**是
「真读过并落地 2 条」——因为在此之前**吸收不可审计**。
现已建 `.neotrix/absorption-audited.json` + `nt_absorption_audit.py`
（每条须附 `read_evidence` 与落地物/未采纳原因，license 与 GitHub API 交叉核对）。
**下一个会话若要回答这个问题，请直接跑那个门，不要凭印象报数字。**
