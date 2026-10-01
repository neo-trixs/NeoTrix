# 交接 — neobot-ui 补缺陷窗口（2026-10-01 续）

> 前序：`handoff-20261001-ui-layout-goal-correction.md`（他窗 `f38acb0f`）·
> `handoff-20261001-unified-reconcile.md`（两窗冲突裁决）· 本窗长会话
> `handoff-neobot-selfhosted-ui-20261001.md`

## §8 收工自查（必填）

| 项 | 状态 |
|---|---|
| worktree 去向 | ⛔ **未开新 worktree**。`nt_worktree_gate.sh check` rc=0；现存 4 个（含主树）**均为他窗/历史**，我一个都没动。门提示「2 个 worktree 的未提交改动不在任何提交里」+「target 累计 4673M 可回收」—— **⛔ 都不是我的改动，我未执行 prune/clean** |
| 我的未提交改动 | **无**。本窗提交均已落盘：`7ce88e27`、`c3b99dc5`、`9255f294`、`287d9614`、`2b9b610a`、`abf0bd08`、`b788da8f` |
| 我开的 worktree | 本轮为验证「干净检出可构建」建了 `/tmp/opencode/clean`（`--detach`）⇒ **已 `git worktree remove --force` 移除**。现存 4 个 worktree **均非我所开**，门提示的「2 个未提交改动」「4673M target」**都不是我的**，我未执行 prune/clean |
| patch 兜底 | 不适用（无未提交改动） |
| 弃用声明 | ⛔ **已作废**：上一轮删掉的 `neobot-check-msg-copy.mjs` 已在 `2b9b610a` **重做并做绿**（A~F，3/3 稳定） |

## §0b ⭐ 代码块：「先测量」救了这一轮（`b788da8f`）

原任务是「代码块视觉统一」。**实测**后才发现样式基本健全
（浅色 5.55:1 / 深色 11.43:1、不撑破 554 vs 578、长行可滚 1150/552），
**缺的是门** + 一个**真缺陷**：垫片 `shim.ts` 的 `lang()` 回退链会
**覆盖宿主显式设置** ⇒ **应用选英文 + 系统中文 ⇒ 复制按钮永远中文**
（实测 `documentElement.lang=en-US` 而 aria-label 未变）。
⇒ 修法：宿主显式设置权威，`navigator.language` 只在宿主未设值时兜底。

ⓘ ⓘ **两处错都是「门/判据自己错」，不是代码错**：
1. A 判据被**自己的夹具**推翻 —— 我把 markdown 放进 6 条消息，
   其中 3 条是 `user`，而 **user 消息按设计显示原文**。
   ⇒ **判据错 ≠ 代码有缺陷，夹具错同理。**
2. ⛔ `selectOption(...).catch(() => {})` 吞掉失败（select 带 `disabled={busy}`）
   ⇒ 伪装成「文案没变」的假象。**验证动作一律不吞错。**

ⓘ 改了漂移门（把 `shim.ts` 移出两树逐字对照，按该文件既有先例），
故**证明了它仍有牙齿**：改 1 字节进 `markdown.js` ⇒ rc=1，还原 ⇒ rc=0，
md5 回到 `0b7576b6…`。⇒ 三个**上游**文件仍在对照内。

## §0 ⭐ 本窗最重发现：自持 UI **干净克隆构建不出来**（`abf0bd08`）

`neobot-root.tsx` import 的 4 个 vendored 文件被根 `.gitignore` 的 blanket
`vendor/` **排除在版本控制之外**，`build` 无 prebuild 拷贝步骤
⇒ `git ls-files …/neobot-ui/src/vendor/` = **0** ⇒ **换机器/换克隆即炸**。

ⓘ **18 道既有门全都测不出来** —— 它们和产物读的是**同一份未跟踪文件**，
本机照样全绿。⇒ 这类缺陷只能靠**质疑前提**的门抓住
（新增 `neobot-check-selfcontained.mjs`，判据 A/B/C，负向测试已证）。

**实证修复**：HEAD 的干净 worktree 里 `pnpm run build` rc=0，
`window.Markdown` 确实进包（修复前会在 import 解析处失败）。
ⓘ `pnpm install` 返回 1 是 `ERR_PNPM_IGNORED_BUILDS`（**构建脚本策略门**，
非依赖解析失败），`node_modules` 已就位 —— 不是本缺陷的一部分。

顺带修正：溯源文档路径原写 `frontend/src/vendor/openghost/`，
实际在 `neobot-ui/src/vendor/openghost/` ⇒ **会误导下一窗口**。
md5 三个全部与文档吻合 ⇒ vendored 文件未被改动过。

## §1 本窗提交

| 提交 | 内容 |
|---|---|
| `7ce88e27` | 修**我自己引入的**主题不持久缺陷 + 清 DSH 命名债；新增 `neobot-check-theme-persist.mjs`（A/B/C/D + 负向测试） |
| `c3b99dc5` | 气泡悬停/键盘复制；抽出 `copyText()` 共用降级链 |
| `2b9b610a` | 气泡复制门 A~F 做绿（3/3 稳定）；全门 18 道 rc=0 |
| `abf0bd08` | ⭐ 修「干净克隆构建不出自持 UI」+ 新增自持性门；全门 19 道 rc=0 |
| `b788da8f` | 代码块复制按钮语言脱钩（真缺陷）+ 新增 markdown 门；全门 20 道 rc=0 |

### `7ce88e26` 的实质：队列 #5 背后是真缺陷
`desktop.rs:33` `get_dsh_theme()` **恒返回 `Theme::System`**（写死常量、零信息量）。
`theme-mode.ts` 原先挂载时 `setThemeMode(后端值)`
⇒ **每次启动覆盖用户存的主题** ⇒ **选了深色下次又变浅**。
根因是**自相矛盾**：该模块自己在注释里写下「后端没有写入口 ⇒ 前端是真源」，
却仍在实现里读后端（契约表确实**无 `set_theme`**，已实测）。
修法 = **删掉该调用**（Rust 侧保持注册不动，不碰他窗代码）。
实测本会话 invoke 6 次、`get_dsh_theme` **0 次** ⇒ 命名债同时清掉。

## §2 ✅ 已闭环（`2b9b610a`）：气泡复制门 A~F 落地

**上一轮那个「应用崩溃」的真凶是门自己的桩，不是产品。**
`invoke` 桩漏了 Tauri 事件插件：`plugin:event|listen` 必须返回**数字**事件 id，
我的 `Promise.resolve(t[c] ?? null)` 返回了 `null` ⇒ 应用挂载期解引用即崩
（`reading '0'`）。`neobot-msg-virtual.mjs` 注释写着**这坑作者栽过第 4 次**，
我照搬时丢了。⇒ 由此加判据 **F 零未捕获异常**（桩错会先在这里报，
而不是伪装成「功能没渲染」）；且**不再从零造 harness**，片段直接提取复用。

写门踩的三个坑（已写进 `task-index` 的 spec）：
1. 桩漏 listen（上面）。
2. ⛔ `.first()` 会选到**已滚出视口**那条（列表自动滚到底，DOM 第一个在 `y=-312`）
   ⇒ 鼠标落空 ⇒ 看起来像「悬停坏了」。须挑「**中心点最顶层**就是它」那条，
   并排除被吸顶栏盖住的。
3. ⛔ 期望值不能取气泡 `innerText`（含 `⧉` 字形），必须取**消息正文**，
   否则门明明复制成功却报「未写入」。

判据：A 默认隐藏/不吃点击/绝对定位 · B 悬停浮现 · C `focus-within` 浮现 ·
D 读回**真剪贴板** · D2 成功有 `is-copied` · E 气泡高度不变 · F 零异常。
实测 **3/3 稳定**（非 flaky）。全门 **18 道 rc=0**。

### ⭐⭐ 负向测试**证伪了我自己的「缺陷」判断**（R-SCAN-2 当场复发）
我曾断言「按钮浮在 `.group` 盒外 ⇒ 鼠标移向它就离开 `.group` ⇒ 用户永远点不到」，
据此把 `top` 从 `-9px` 改成 `4px`。
**注入 `-9px` 后门全过（rc=0）** ⇒ 断言是错的：按钮中心落在 `groupTop+2`，
**仍在 `.group` 内**。仍保留 `4px`，但**如实标注为防御性选择、非已证实的修 bug**
（盒内 ⇒ 结构上不可能被滚动容器裁掉；「浮在盒外会被裁」我**测不出来** ——
从 sizer 上溯找不到 `overflow-y:auto/scroll` 祖先 ⇒ 不下结论）。
CSS 注释里写清了这全过程。⏳ **未验证项**：浮在盒外是否真会被裁。

## §3 本窗方法论（第三次「单向断言假通过」）

| # | 坑 | 修法 |
|---|---|---|
| 1 | 主题门只测「浅色」⇒ 负向测试**假通过**（headless 系统默认就是浅色，恰好与选择一致） | **双向必测**；实测是「深色」那档抓到 rc=1 |
| 2 | 折叠门只测正向 ⇒ 前向不能红 | 负向注入 + 断言锚点 |
| 3 | 主题门 A/B 只在**同一会话**内比较 ⇒ 覆盖 bug（发生在**挂载时**）根本不显现 | 必须**跨刷新** |
| 4 | 固定 `waitForTimeout` 当等待 ⇒ 随机红/绿，等于没门 | 改确定性等待 |
| 5 | 伪造宿主 API（`navigator.clipboard`）改变被测控制流 | 授真权限 + 真 API 读回 |
| 6 | **手推「鼠标会离开 .group」⇒ 当成缺陷去「修」** ⇒ 负向测试直接推翻 | ⭐ 改前先做负向测试；被推翻就如实写「防御性选择」而非「修 bug」 |
| 7 | 门挑「DOM 第一个」元素 ⇒ 选中已滚出视口那条 | 挑「中心点最顶层」的那条 |

⭐ 两条**我自己的**方法论错误（比门更重要）：
1. **「零命中」的新变体**：`rg` 默认尊重 ignore 规则，`vendor/` 在
   `.gitignore` 里 ⇒ 我搜 `src/` 得出「`window.Markdown` **全项目零定义**」，
   并据此断言「markdown 完全没接、那 100 行 CSS 是死 CSS」——**全是假的**。
   ⇒ 涉及被 ignore 的目录，**必须 `--no-ignore` 再下结论**。
2. ⛔ `tsc --noEmit | head -3; echo rc=$?` 取的是 **`head` 的退出码**。
   我据此说过好几次「tsc rc=0」，**全是无效证据**。
   ⇒ `$?` 取管道**最后**一个命令的码；验证门/工具一律**不带管道**。
   （本轮 `neobot-check-selfcontained` 的 rc 也踩过，已改写法。）

ⓘ 元教训：**「我造的门跑红」先怀疑门，再怀疑产品** —— 但要**给出证据**（本窗用
「同一 dist 下他门 rc=0」+「产物含该字符串」两条把产品嫌疑排除掉），不能靠猜。

## §4 门状态（收工实测，20 道全绿 + license 预期红）

本侧 10（含 `neobot-check-msg-copy`）：`api_contract` / `ui_wiring` / `feature_viability` / `absorption_audit` /
`ui-smoke` / `msg-virtual` / `check-contrast` / `check-convo-groups` /
`check-theme-persist` / `check-msg-copy` / `check-selfcontained` /
`check-markdown` — 全 rc=0
他侧 4：`check_visual` / `check_ship_ui` / `check_ui_calls` / `check_upstream_1to1` — 上轮实测 rc=0
`check-license` rc=1 **正确**（受限 vendored 树未删）

## §5 唯一真阻塞（沿用）

删 `apps/neobot-desktop/frontend/src/vendor/`：他窗有在途改动。**须先重查 status/mtime**，
⛔ 禁直接删。删前先 `git worktree add --detach HEAD` 干净检出测门。

## §6 沿用纪律

- 共享树：新文件先 `git add <显式路径>` → `git commit --only <同一批>`；⛔ 禁 `git add -A`。
- 桌宠（`src/pet/`）归他窗；`pet → DROPPED` 标记按用户指示**由他窗改**。
- `nt_feature_viability` 的 4b 仍报 `src/pet/pet.tsx` 直连 `@tauri-apps/api/core`（他窗文件，不判失败）。
- ⛔ 搜不到未授权 `OPENAI_API_KEY`；GitHub 配额 0，台账 license 停在 40/493。
