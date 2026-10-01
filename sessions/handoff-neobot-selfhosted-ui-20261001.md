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

**我本会话未提交、留在工作树的**（原因逐条）：

| 文件 | 原因 |
|---|---|
| `apps/neobot-desktop/tauri.conf.json` | **混有他窗改动**（`macOSPrivateApi`/窗口尺寸/label）。我只改了 `build` 段 3 项（`frontendDist` → `neobot-ui/dist`、两个命令加 `--prefix neobot-ui`）。⛔ 他窗那版裸 `pnpm run build` 是**坏的** —— Tauri 在本目录执行，而此处无 package.json |
| `apps/neobot-desktop/frontend/src/neobot-root.tsx` | md5 `5909ee44`，仅含我的**剪贴板降级修复**（`navigator.clipboard` 失败时回落 `write_clipboard_text`）。该文件有他窗 615 行在途改动，提交它=连带提交他的工作 |

## 5. 下一步（按优先级排序）

1. **删 `apps/neobot-desktop/frontend/src/vendor/`**（他窗已提交其桌宠工作后可做）⇒
   `check-license.sh` 由 rc=1 转绿。**删前必看** §7 的两条理由。
2. 收敛 `nt_feature_viability.py` 里 `pet → DROPPED`（**由桌宠作者本人做**，见 §7）。
3. `neobot-root.tsx` 的 `get_dsh_theme`：上游入口遗留的 DSH 概念泄漏，删树后成孤儿命令。
4. `api-panel.ts` 的硬编码文案未迁 i18n（术语多为英文命令名，优先级低）。

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
- **未提交改动的去向**：
  - `tauri.conf.json`、`frontend/src/neobot-root.tsx` —— **明确声明保留在工作树**，
    原因是与另一窗口改动混杂，提交会造成越权；内容已在本文件 §4 逐条列明，
    接手会话可直接取用。**非弃用。**
  - 其余本会话产出**全部已提交**。
- **门状态（2026-10-01 实测）**：
  `nt_api_contract` 0 · `nt_neobot_ui_wiring` 0 · `nt_feature_viability` 0 ·
  `runtime smoke` 0 · `check-license` **1（正确**：受限 vendored 树仍在仓库，
  删树后转绿）。
