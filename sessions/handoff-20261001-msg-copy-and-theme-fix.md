# 交接 — neobot-ui 补缺陷窗口（2026-10-01 续）

> 前序：`handoff-20261001-ui-layout-goal-correction.md`（他窗 `f38acb0f`）·
> `handoff-20261001-unified-reconcile.md`（两窗冲突裁决）· 本窗长会话
> `handoff-neobot-selfhosted-ui-20261001.md`

## §8 收工自查（必填）

| 项 | 状态 |
|---|---|
| worktree 去向 | ⛔ **未开新 worktree**。`nt_worktree_gate.sh check` rc=0；现存 4 个（含主树）**均为他窗/历史**，我一个都没动。门提示「2 个 worktree 的未提交改动不在任何提交里」+「target 累计 4673M 可回收」—— **⛔ 都不是我的改动，我未执行 prune/clean** |
| 我的未提交改动 | **无**。本窗提交均已落盘：`7ce88e27`、`c3b99dc5`、`cc1d8886`(amend→`9255f294`)、`2b9b610a` |
| patch 兜底 | 不适用（无未提交改动） |
| 弃用声明 | ⛔ **已作废**：上一轮删掉的 `neobot-check-msg-copy.mjs` 已在 `2b9b610a` **重做并做绿**（A~F，3/3 稳定） |

## §1 本窗提交

| 提交 | 内容 |
|---|---|
| `7ce88e27` | 修**我自己引入的**主题不持久缺陷 + 清 DSH 命名债；新增 `neobot-check-theme-persist.mjs`（A/B/C/D + 负向测试） |
| `c3b99dc5` | 气泡悬停/键盘复制；抽出 `copyText()` 共用降级链 |
| `2b9b610a` | 气泡复制门 A~F 做绿（3/3 稳定）；全门 18 道 rc=0 |

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

ⓘ 元教训：**「我造的门跑红」先怀疑门，再怀疑产品** —— 但要**给出证据**（本窗用
「同一 dist 下他门 rc=0」+「产物含该字符串」两条把产品嫌疑排除掉），不能靠猜。

## §4 门状态（收工实测，18 道全绿）

本侧 10（含 `neobot-check-msg-copy`）：`api_contract` / `ui_wiring` / `feature_viability` / `absorption_audit` /
`ui-smoke` / `msg-virtual` / `check-contrast` / `check-convo-groups` /
`check-theme-persist` / `check-msg-copy` — 全 rc=0
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
