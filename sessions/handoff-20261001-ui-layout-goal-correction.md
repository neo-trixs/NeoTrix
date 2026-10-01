# 交接：NeoBot 交付路径 UI 重构（2026-10-01）

> 窗口主题：商用确认后把目标从「与上游逐字一致」修正为「一个自洽可用的桌面应用」，
> 并在交付树 `neobot-ui/` 上修掉三处「全绿但界面不能用」的缺陷 + 建视觉门。
> 本文件按 `sessions/HANDOFF-TEMPLATE.md` §8 填「收工自查」。

---

## §1 接手时先读这三样（别读整个 STATUS）

1. **`apps/neobot-desktop/STATUS.md` §0** —— 目标唯一真源。上一版的目标是错的，
   错在把「与上游 1:1」当成交付质量。§0 写死了新目标与两条路径的定位。
2. **`apps/neobot-desktop/STATUS.md` §8/§9/§10** —— 本轮三批改动的原因与实测数字。
3. **`docs/architecture/FRONTEND-REBUILD-2026-10-01.md`** —— 商用重构的裁决与依据。

## §2 两条路径（**不要再混**）

| 树 | 定位 | 守它的门 |
|---|---|---|
| `apps/neobot-desktop/neobot-ui/` | **唯一交付物** | `nt_check_ship_ui` · `nt_check_visual` · `neobot-ui-smoke` |
| `apps/neobot-desktop/frontend/` | **参考树，不可交付**（商用条款） | `nt_check_upstream_1to1`（只守「别漂移」） |

⛔ vendored 树的任何数字都不是交付质量的证据。它有 140 文件 1:1 + 4 道门，
而交付物当时连桌宠页都没有、macOS 菜单点了没反应。

## §3 本轮修掉的（都有实测数字）

| 项 | 之前 | 之后 | 怎么测的 |
|---|---|---|---|
| `.nb-main` 高度 | **213px**（`flex-grow: 0`） | **792px**（视口−顶栏） | 视觉门① + 手动探针 |
| 顶栏控件高度 | 参差（26 / 更高） | 全部 28px，极差 **0** | 视觉门②③ |
| 会话项行高 | 34px | **53px** | 视觉门④ |
| 对话列宽 | 无上限 | 760px | 视觉门⑤ |
| 桌宠窗 | **不存在**（dist 无 pet.html） | 220×220 真机出现 | `nt_check_ship_ui` + 真机 AX |
| macOS 菜单 | 装了但**没人 listen** | 五项齐 + 未接项给一句「尚未接入」 | 真机 AX 逐条读出 |
| 设置入口 | 无（`api-panel.ts` 是孤儿文件） | 顶栏按钮 + 原生菜单共用 | 目视 + 门 |
| 消息分组 | 每条等距 | 组内 18px / 组间 28px | 视觉门③b |

**根因值得记住**：`.nb-main` 少一条 `flex: 1 1 auto`。这不是「少写一个属性」——
**同一个缺陷被修过一次、修法只修了一半**（上次加了 `display:flex`，
把 `flex:1 1 auto` 删掉了）。两者分工不同，两个都在时构建/类型/渲染仍全绿。

## §4 门（新增 4 道，都是「量」出来的）

- `nt_check_visual` —— 专治「全绿但界面不能用」。高度链 / 控件等高 / 基线对齐 /
  会话行高 / 消息列宽 / 消息分组 / 溢出 / 输入框在视口。
- `nt_check_ship_ui` —— 交付路径门：tauri 指向 neobot-ui / dist 有 pet.html /
  产物含 `macos-menu-action` / hook 构建的是同一棵树 / 自研壳调用全部已注册。
- `nt_check_ui_calls` —— 界面可达模块里每个 `invoke` 都有已注册命令。
- `nt_check_upstream_1to1` —— 参考树差集 = 白名单；**参考树找不到时 FAIL**
  （此前是 SKIP + exit 0，「查不了」长得像「通过」）。

复现：`node scripts/ops/nt_check_{visual,ship_ui,ui_calls,upstream_1to1}.mjs`

## §5 剩下的队列（按价值排序，交接即可直接开工）

1. **侧栏按最近活跃分组**（今天/更早）+ **折叠**。会话多起来后这是最大缺口；
   ⚠️ 该区域已被另一窗口做过虚拟化（`perf(neobot-ui): 会话列表虚拟化`），
   改之前先 `stat -f "%Sm"` 看有没有人在写。
2. **暗色对比度分级**。现在深色下靠 token「恰好对」，没有分级依据，
   也没有门在守 —— 视觉门只量几何，不量对比度。
3. **真机重验**。最近几轮（目标修正 / 空状态 / 消息分组）改完后**没有**重新
   `tauri build` + 启动。上次真机确认（桌宠窗、原生菜单）在这些改动之前。
   命令：`cd apps/neobot-desktop && ./frontend/node_modules/.bin/tauri build`
   然后 `open -a .../NeoBot.app`，用 `osascript` 读 `menu bar 1` 与窗口列表。
4. 气泡悬停操作（复制）、代码块样式统一、动效、图标体系统一。

## §6 ⛔ 未验证 / 不能当已完成的

- **WKWebView 层**：所有前端门跑在 headless Chrome。真机只验过「窗口起没起、
  菜单有没有」，没验过 WebKit 与 Blink 的渲染差异。
- **真实后端联调**：截图里是桩数据。真机跑起来时 `neobot_send` 走的是真
  neotrix-core 吗？未验。
- **`neobot-ui-smoke.mjs` 的 2 项 FAIL**：不是本轮引入，但接手时要知道它红着。

## §7 经验（这一段是本轮最值钱的东西）

### 7.1 目标错了，勤奋会把它放大

我把「与上游逐字一致」当成交付质量，投入四轮 + 4 道门，**而那棵树商用确认后
不可交付**。勤奋不能修正目标错误，只会让错误方向跑得更远。
**判据**：投入之前先问「这东西交付时被谁打开」。

### 7.2 「门全绿」与「门量对了对象」是两件事

同一道视觉门，我连续四次量错对象：

1. 根/侧栏在**顶栏之下**，判据该是「视口−顶栏」，我写成「视口」；
2. 列宽要量**居中那列**（`mx-auto`），我量了滚动容器（1024px 全宽）；
3. `data-index` 是**两个虚拟化列表共用**的属性 → 两列 top 混排，算出「负间距
   −53」，**门对着垃圾数据判 PASS**；
4. 绝对定位下 item 外框**首尾相接**，量外框间距恒为 0 → 必须量**气泡**。

第 3 条最险：**断言不是没跑，是对着错误数据通过了。**
所以每个量都要先问一句「我量的这个元素，是用户看到的东西吗」。

### 7.3 截图正常 ≠ 判据正确

我一度认为「选中态和 hover 态看起来一样」只是审美问题。改完后截图正常，
但那是因为**我加的强调条让差异变得可见**；原来是否有差异、差多少，
应该量 computed backgroundColor。**判据要选可量的那个。**

### 7.4 我制造的问题，也长成了门

- `status` 门的「前端行数」取全文首个 `N 行` ⇒ 我在 §0 写句「不是那 731 行」
  就把门顶歪（731 vs 16405）。**判据不锚定 ⇒ 文档任何改动都能挪动它。**
- 交互门的 `aside button[n]` 按下标点击 ⇒ 我给侧栏加个搜索框就点错按钮，
  而门照样可能绿。上一轮刚把这条写进教训 28，**下一轮原地再犯**。
  ⇒ **教训写进文档 ≠ 行为被改**，行为要改需要一个不会被下标漂移打破的选择器。
- 字节门抓到我的写入切坏多字节字符；api 门把我注释里的 `desktop.rs` 读成命令。
  **门犯的错和它要抓的错常常同一种。**

### 7.5 共享工作区的三条实操

- `git commit -- <paths>` **只暂存已跟踪文件** ⇒ 新文件根本没进库，而 `lib.rs`
  已经 `pub mod menu;`，**提交态编译不过**，pre-commit 的 cargo 门跑的是工作区，
  没抓到。必须 `git add -A <paths>`，并**验证提交态自洽**（逐个 `mod` 查文件）。
- `git reset --hard HEAD~5` 我算错基准，**退过头 4 个提交**（把另一窗口的工作
  退成未提交）。复位用 `git reset --mixed <正确的 sha>`，工作区内容不动。
- 链式命令 `a && b && c` 里 `a` 失败 ⇒ `b`/`c` 从未执行。
  我因此**以为删了移动端图标**，实际 `rm -rf` 没跑，图标被 `git add -A` 扫进
  提交，只好再补一个删除提交。**「我以为删了」与「删了」之间隔着一条会中途失败的链。**

## §8 收工自查

- **worktree**：`git worktree list` = **4 个**，`nt_worktree_gate.sh check` 报
  **2 个带未提交改动**、合计 4840M（target 占 4673M）。
  ⚠️ **这 2 个不是本窗口创建的**（本窗口未开 worktree），它们带在途改动，
  **不要 prune**，除非先 `sh scripts/ops/nt_worktree_gate.sh prune`（脚本会先做
  patch 兜底）。`target` 4673M 可回收：`sh scripts/ops/nt_worktree_gate.sh clean`
  （脚本判定零风险）。
- **未提交改动去向**：
  - 本窗口的**全部**改动都已入库（`00797d72` 消息分组、`31eb71a9` 目标修正、
    `62828a09` 布局三处+视觉门、以及更早的 6 个提交）。
  - 工作区剩余 `M api-panel.ts` / `M locales` / `M nt_check_ship_ui.mjs`
    **属另一窗口**（`api.*` / `chat.groupEarlier` 等 key 是他们加的），
    **我没提交、也不该提交**。
  - `.neotrix/*`、`neotrix-core/*`、`results.tsv`、`.github/*` 同样是工具态与他窗在途。
- **验证**：`tsc --noEmit` 0 · `neobot-ui` build 通过 · `nt_check_visual` PASS ·
  `nt_check_ship_ui` / `nt_check_ui_calls` / `nt_check_api` / `nt_check_status` PASS ·
  `cargo test -p neobot-desktop --lib` 71 passed。
- **打包/真机**：本窗口**没有**在最后几轮改动后重新 `tauri build` + 启动（见 §5.3）。
