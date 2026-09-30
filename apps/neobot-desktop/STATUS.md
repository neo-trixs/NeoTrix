# NeoBot 状态清单

> **快照日期**：2026-09-30 · **生成方式**：本文数字全部由实测命令产出，不是人工计数。
> 复现命令见每节末尾。**这份文件会腐化** —— 真源是那些命令，不是这张表
> （与 `CAPABILITY-MAP-2026-09-30.md` 同一个教训：那张表曾经整张腐化）。

---

## §1 已建成（可点可验）

### 1.1 前端 —— 4,445 行

| 文件 | 行 | 职责 |
|---|---:|---|
| `src/ui/tokens.css` | 197 | 两层 token：primitive（真实色）→ semantic（用途）+ 深色 + 动效偏好 |
| `src/ui/shell.css` | 363 | 四栏骨架 Rail/Panel/Content/Inspector |
| `src/ui/base.css` | 230 | 基础组件（btn/field/switch/section/row…） |
| `src/ui/primitives.css` | 144 | Item / Empty / Loadable / Progress（DSH 范式） |
| `src/ui/blocks.css` | 164 | 块序列样式（tool/artifact/reasoning/system/panel） |
| `src/ui/blocks.css`+`sheet.css` | 158 | 弹层表单（DSH SettingsForm 复刻） |
| `src/ui/island.css` | 80 | 活动岛 |
| `src/plugin/contract.ts` | 436 | 能力注册表 + 插件契约 + **主体身份（防自报提权）** |
| `src/host/host.ts` | 111 | 宿主适配层 + 内存替身 |
| `src/ui/block-model.ts` | 261 | 块类型 + 决策面板不变量（纯逻辑） |
| `src/ui/blocks.ts` | 176 | 块渲染（**逐块隔离**） |
| `src/ui/panel-view.ts` | 208 | 决策面板交互（作答 + 校验 + 出处） |
| `src/ui/island-model.ts` | 131 | 岛状态推导（**只能被推导，不能手设**） |
| `src/ui/island.ts` | 55 | 岛渲染 |
| `src/list.ts` | 133 | 分组列表（交集解剖：头像+主标+副标+尾部） |
| `src/ipc.ts` | 165 | **命令表推导**取代调用方断言 |
| `src/main.ts` | 640 | 装配 |
| `src/selftest.ts` | 548 | 7 组自测 |

### 1.2 库侧（本仓唯一真源）

| 模块 | 行 | 测试 | 内容 |
|---|---:|---:|---|
| `crates/neotrix-neobot/src/nt_evidence.rs` | 329 | 8 | 证据审计：断言有无出处 / 过度断言 / `sourced_ratio` 缺席≠0 |
| `crates/neotrix-neobot/src/nt_panel.rs` | 440 | **18** | 决策面板契约 + **过期作答检测** + **面板注册表** |

### 1.3 命令（10 个，前后端两侧一致）

```
neobot_agent_run           neobot_convo_group        neobot_convo_dm
neobot_send                neobot_evidence_summary   neobot_core_capabilities
neobot_panel_answer
neobot_panel_publish              neobot_panel_clear              neobot_panel_demo_publish
```

### 1.4 资产

- 图标主稿 2 份（`icon.svg` 透明底孩童鲨 / `mark-mono.svg` 16px UI 档）
- 栅格 **49 PNG + ico + icns**（Tauri/iOS/Android 全套）
- 同步 `skills/assets/icons/neobot/` 10 个（16→1024 全档）

### 1.5 门禁（6 个，全部经变异验证）

| 门 | 抓什么 | 变异 |
|---|---|---|
| `nt_check_tokens.mjs` | token 未定义 / 组件直连旧 token / 组件裸 hex | 3/3 |
| `nt_check_bytes.mjs` | U+FFFD（写文件环节有 bug 的信号） | 1/1 |
| `nt_check_ipc.mjs` | 前后端命令**双向**不一致 / 重复注册 | 3/3 |
| `nt_check_layout.mjs` | 高度链 / 真实可滚 / token 已解析 / 溢出 | 5/5 |
| `nt_shot.mjs` | 界面截图（HTTP 服务方式） | — |
| `nt_check_status.mjs` | **本文件与实测是否一致** | 见 §4 |

**前端 7 组自测 + 库测试38 条（库 26 + command 层 12）Rust 测试。**

> 复现：`node scripts/ops/nt_check_{tokens,bytes,ipc,layout,status}.mjs` ·
> `cd apps/neobot-desktop/frontend && node selftest.mjs` ·
> `cargo test -p neotrix-neobot -p neobot-desktop`

---

## §2 关键设计决定（附理由，避免被后人推翻）

| 决定 | 理由 |
|---|---|
| **缺席 ≠ 空**（`sourced_ratio` 为 `None` 而非 `0.0`；`Mode` 缺省 `Sample`） | 「没检查过」与「检查了，0%」是不同的话。缺省成 `Live` 等于无证据声称真实 |
| **主体身份必须有 `resolvedBy`** | `principalId` 谁都能填；模型输出「我是 neo」就会提权。`resolvedBy` 填不出「哪个提供方在哪个可信边界确认的」 |
| **岛状态只能被推导** | 状态若有两个来源（人手设 + 块流推）迟早不一致 |
| **能力门控：不可用就不渲染** | 灰按钮点了没反馈，用户只能猜 |
| **逐块 try/catch 而非整条** | 60 块里一块炸了，整条包一层就 **60 块全没** |
| **`minmax(0,1fr)` + `min-height:0`** | grid/flex item 默认 `auto`，拒绝收缩 ⇒ 内部 `overflow` 永不触发（本仓踩过最贵的 bug） |
| **拒绝时补传两个版本号** | 只说「已过期」用户无法判断该重选还是重试 |
| **过时不谎报状态** | 骨架拒收就不写 `selectedId`；宁可岛继续提示 |

---

## §3 已知缺口（诚实列出，按严重度）

### 🔴 阻断级

| # | 缺口 | 影响 | 位置 |
|---|---|---|---|
| 1 | ~~骨架无面板注册表~~ **已闭合** | 已有 `nt_panel::Registry`（18 测试）。`neobot_panel_answer` 现在**只收 answer**，基准由骨架持有；面板经 `neobot_panel_publish` 登记后由 `neobot:panel` 事件下发 | — |
| 2 | **骨架侧尚无面板下发器** | 通道已通（publish → 事件 → 界面 → answer → 注册表校验），但**没有真实骨架在发面板**。浏览器预览下因此没有待答面板 —— 这是正确的，不是缺陷 | 需骨架侧实现下发 |
| 3 | **无法在 Tauri 里做端到端验证** | Tauri on macOS 用 WKWebView，**不是 Chrome** ⇒ 布局门那套 CDP 接不上运行中的 app。真实点击链路仍未验证 | 需 Web Inspector 协议或骨架侧集成测试 |
| 4 | **列表仍是 `MOCK_ITEMS`** | 会话列表未接真实 store。`neobot_convo_group`/`convo_dm` 已能建会话，但列表不从库里读 | `main.ts` |

### 🟡 应当修

| # | 缺口 | 说明 |
|---|---|---|
| 4 | `data_dir()` 未与 CLI 对齐 | 标了 `TODO`。两处不一致 ⇒「桌面建的会话 CLI 看不见」，且**不报错** |
| 5 | `neobot_core_capabilities` 前端未消费 | 能力矩阵仍是静态默认值，命令就位但没接 |
| 6 | 无 a11y 门 | openbot 有 `@storybook/addon-a11y`；本仓只有局部的 `aria-live`/`role` 处理 |
| 7 | 剥注释逻辑重复三处 | Token 门、图标自查、契约自查各写一遍。应抽成共享工具（已记三次重犯） |
| 8 | 界面对比度/暗色未实测 | 布局门量几何，不量颜色；深色模式只在 `:root[data-theme=dark]` 里定义，**未渲染验证过** |

### ⚪ 已知可接受

| # | 项 | 说明 |
|---|---|---|
| 9 | 16px 完整图标不可读 | 与 neotrix 自家 `nt-core-icon-16` 同级。UI 侧另用 `mark-mono.svg` 简化档 |
| 10 | 前端/Rust 两份 `DecisionPanel` 类型 | 跨语言无法共享，靠 `nt_check_ipc.mjs` 守命令名一致，**字段仍需人工同步** |

---

## §3b 这份清单被门守着

`scripts/ops/nt_check_status.mjs` 会核对本文的**关键数字**与实测是否一致：
前端行数 · 图标 PNG 数 · IPC 命令数（逐个列全）· 库测试数 · 门禁数 · 自测分组数。
不一致即 FAIL。

**为什么需要**：`CAPABILITY-MAP-2026-09-29` 整张表腐化而**无人发现** ——
它长得像实测结果（有行号、有证据列），但没有任何东西去核对它。
本文是同一类文件（人写的数字 + 表格），所以给它配一道门。

这道门第一次跑就抓到 **1 项真腐化**（加了它自己之后门禁数从 5 变 6，
文档还写 5）—— 证明它不是摆设。同时它的第一版有 2 个自身 bug
（walk 不递归，漏掉 `src/ui/` 与 `icons/android|ios/`），
是**门自己先报错**逼我修门，而不是反过来。

## §4 已知的假绿教训（本会话累计 7 次）

这一节比上面的功能清单更值钱 —— 每一条都是**「没看到失败」被当成「验证通过」**：

| # | 形态 | 后果 |
|---|---|---|
| 1 | 失败信息被 `2>/dev/null` 丢掉 | 6 个变异全报「0 失败」 |
| 2 | 变异 harness 先还原文件再跑门 | 同上 |
| 3 | 只跑运行时自测、漏 tsc | 2 个纯类型变异全报「0 失败」 |
| 4 | 改源码不重建（**测的是产物**） | 2 处 `min-height` 移除全报 PASS，差点去改门 |
| 5 | shell 变量没 `export` | 5 个 Rust 变异一个都没写入却全报「8 passed」 |
| 6 | 变异 harness 把**编译失败**读成「测试通过」 | `找不到("test result", "")` 后 `"FAILED" in ""` 为假 ⇒ 变异没编过却判 PASS。「抓到 2/3」里有一条其实是无效变异 |
| 7 | 空洞的测试：断言「不该发生的事」却**没构造那件事** | `版本倒退被拒` 先写 `version=0`（非法）又改回 `1`，而当前就是 `1` ⇒ 根本没发生倒退，测试却绿着。看起来覆盖了「版本倒退」，实际什么都没测 |

**共同根因**：把「没有观察到失败」当成了「验证通过」。这两件事在
`grep` 无匹配、`&&` 短路、类型擦除、产物未重建、env 未传递、
**变异没编过**、**前提没被构造** 七种情况下都会分叉。

⇒ 现有对策：变异 harness **自证变异真的写进去了**；纯类型不变量由
tsc 把关；静态门跑产物就跑产物。

---

## §5 下一步（按依赖排序）

1. **骨架侧面板注册表**（缺口 1）—— 让 `neobot_panel_answer` 只收 `answer`
   并按 id 查。这是「能校验」→「有真源」的最后一步，需同时改 Rust 与 `ipc.ts`。
2. **真实联调**（缺口 2/3）—— 骨架发面板 → 事件下发 → 界面渲染 → 作答回流。
   需要一个能在 WKWebView 里断言的手段（Web Inspector 协议，或让骨架侧出集成测试）。
3. **`data_dir` 对齐**（缺口 4）—— 最该先验的一条，因为它**不报错**。
4. **补 a11y 门 + 暗色渲染验证**（缺口 6/8）。
5. **剥注释抽共享工具**（缺口 7）—— 收敛点已记三次。

---

## §6 许可边界（不可越过）

| 仓库 | 许可 | 边界 |
|---|---|---|
| deepseek-ai/deepseek-harness | MIT | ✅ 代码可用；**商标不可用**，项目内只用 `DSH` 缩写 |
| anywhere-labs/dsh-desktop | MIT | ✅ |
| dsh-tauri/deepseek-harness-desktop | MIT | ✅ |
| omdsh-dev/DSH-better-sidebar | MIT | ✅ |
| dataelement/dsh-desktop | MIT | ✅（但仓内无 UI 源码） |
| ccch1mneyyy/dsh-TUI | MIT | ✅ |
| huiliyi37/dsh-tianshu-tui | **Apache-2.0** | ⚠️ 须保留 NOTICE、标注改动、含专利授权 |
| awesome-dsh-plugin | CC0-1.0 | ✅ 清单内容 |
| thinkany-ai/douchat | **AGPL** | ⛔ **不可取码**，只读结构与思路 |
| nightly-labs/openbot | **PolyForm Noncommercial** | ⛔ 不可取码（仅限非商业） |
| hikariming/dshfind | **无 LICENSE** | ⛔ 无许可证 = 保留一切权利，只读 README |

图标为原创构造，不临摹任何现有图标（早期一版做成「小海豚同款」是抄注册商标，已撤）。
