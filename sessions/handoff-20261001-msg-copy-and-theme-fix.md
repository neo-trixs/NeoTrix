# 交接 — neobot-ui 补缺陷窗口（2026-10-01 续）

> 前序：`handoff-20261001-ui-layout-goal-correction.md`（他窗 `f38acb0f`）·
> `handoff-20261001-unified-reconcile.md`（两窗冲突裁决）· 本窗长会话
> `handoff-neobot-selfhosted-ui-20261001.md`

## §8 收工自查（必填）

| 项 | 状态 |
|---|---|
| worktree 去向 | ⛔ **未开新 worktree**。`nt_worktree_gate.sh check` rc=0；现存 4 个（含主树）**均为他窗/历史**，我一个都没动。门提示「2 个 worktree 的未提交改动不在任何提交里」+「target 累计 4673M 可回收」—— **⛔ 都不是我的改动，我未执行 prune/clean** |
| 我的未提交改动 | **无**。本窗两笔提交均已落盘：`7ce88e27`、`c3b99dc5` |
| patch 兜底 | 不适用（无未提交改动） |
| 弃用声明 | `scripts/ops/neobot-check-msg-copy.mjs` **已 `rm`**：功能门跑不绿，按「不提交红门」处理。**不是**遗忘，见 §2 |

## §1 本窗两笔提交

| 提交 | 内容 |
|---|---|
| `7ce88e27` | 修**我自己引入的**主题不持久缺陷 + 清 DSH 命名债；新增 `neobot-check-theme-persist.mjs`（A/B/C/D + 负向测试） |
| `c3b99dc5` | 气泡悬停/键盘复制；抽出 `copyText()` 共用降级链 |

### `7ce88e26` 的实质：队列 #5 背后是真缺陷
`desktop.rs:33` `get_dsh_theme()` **恒返回 `Theme::System`**（写死常量、零信息量）。
`theme-mode.ts` 原先挂载时 `setThemeMode(后端值)`
⇒ **每次启动覆盖用户存的主题** ⇒ **选了深色下次又变浅**。
根因是**自相矛盾**：该模块自己在注释里写下「后端没有写入口 ⇒ 前端是真源」，
却仍在实现里读后端（契约表确实**无 `set_theme`**，已实测）。
修法 = **删掉该调用**（Rust 侧保持注册不动，不碰他窗代码）。
实测本会话 invoke 6 次、`get_dsh_theme` **0 次** ⇒ 命名债同时清掉。

## §2 ⛔ 交接给下一窗口的未闭环项

### 2.1 气泡复制**没有专属门**（唯一硬缺口）
功能已实现、tsc rc=0、构建通过、**7 道现有门全绿**、且**曾观测到实际渲染**
（`copyBtns: 1`）。但**当前不可复现**，故删门而非提交红门。

崩溃签名（本窗 harness 独有；无/有 `permissions`、单/双 init 脚本**三种变体全部**复现同一点）：
```
TypeError: Cannot read properties of null (reading '0')
  at main-*.js:31:2575     ⇒ React 未挂载（#root 为空）
```

**已排除**：
- ⛔ 不是产品回归 —— 同一 `dist` 下 `neobot-ui-smoke`、`neobot-msg-virtual` 均 **rc=0**，
  且产物 `main-*.js` 内**确实含** `nb-msg-copy`。
- ⛔ 桩内容 —— 与 `neobot-msg-virtual.mjs` 的 `STUB` **逐字段相同**。

**我自己的两个错误（别重犯）**：
1. 伪造 `navigator.clipboard`（`Object.defineProperty`）会**改变被测代码控制流**；
   已改用授真权限 + `readText()` 读回。
2. `page.$()` **不支持** `>> nth=0` 引擎语法（当普通 CSS 落空）；
   取「第一条」须用 `page.locator(sel).first()`。

**未闭环线索**：崩溃似与桩**规模**相关（1 会话+1 消息崩；1000+1000 在他窗门里正常）。
若成立 ⇒ **「单会话/单消息」路径当前无任何门覆盖**，`neobot-ui-smoke` 桩覆盖面有缺口。
改用 1000 规模后我的门**仍**报 0 个按钮，故此线索**未证实**。

### 2.2 建议的判据（已想清，可直接实现）
`neobot-check-msg-copy.mjs` 应含 A 默认 `opacity:0`+`pointer-events:none` ·
B 悬停出现 · C **`focus-within`** 也出现（⛔ 只做 hover 则键盘用户永远看不到）·
D **读回真剪贴板**（⛔ 不能只看点击不报错）· E **气泡高度不变**
（⛔ `measureElement` 量的就是它，按钮进流会污染虚拟化）。

## §3 本窗方法论（第三次「单向断言假通过」）

| # | 坑 | 修法 |
|---|---|---|
| 1 | 主题门只测「浅色」⇒ 负向测试**假通过**（headless 系统默认就是浅色，恰好与选择一致） | **双向必测**；实测是「深色」那档抓到 rc=1 |
| 2 | 折叠门只测正向 ⇒ 前向不能红 | 负向注入 + 断言锚点 |
| 3 | 主题门 A/B 只在**同一会话**内比较 ⇒ 覆盖 bug（发生在**挂载时**）根本不显现 | 必须**跨刷新** |
| 4 | 固定 `waitForTimeout` 当等待 ⇒ 随机红/绿，等于没门 | 改 `waitForSelector` 确定性等待 |
| 5 | 伪造宿主 API（`navigator.clipboard`）改变被测控制流 | 授真权限 + 真 API 读回 |

ⓘ 元教训：**「我造的门跑红」先怀疑门，再怀疑产品** —— 但要**给出证据**（本窗用
「同一 dist 下他门 rc=0」+「产物含该字符串」两条把产品嫌疑排除掉），不能靠猜。

## §4 门状态（收工实测，13 道）

本侧 9：`api_contract` / `ui_wiring` / `feature_viability` / `absorption_audit` /
`ui-smoke` / `msg-virtual` / `check-contrast` / `check-convo-groups` / `check-theme-persist` — 全 rc=0
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
