---

## §0 目标（**唯一真源，冲突时以本节为准**）

> 2026-10-01 修订。**上一版的目标是错的** —— 错在把「与上游逐字一致」当成交付质量。

### 现在要什么

**一个自洽、可用、好看的桌面应用。** 判据三条，全部可量：

1. **能用** —— 每个可见控件都有真实后端支撑；点了会失败的按钮不许存在。
2. **自洽** —— 高度链通、控件等高、间距有节奏、深浅两套都达可读对比。
3. **可维护** —— 改动有一道门能接住；没有门就改的地方等于「凭感觉调」。

### 两条路径的定位（不要再混）

| 树 | 定位 | 门 |
|---|---|---|
| `apps/neobot-desktop/neobot-ui/` | **唯一交付物**。顶栏/侧栏/对话/桌宠/设置都在这里 | `nt_check_ship_ui` + `nt_check_visual` + `neobot-ui-smoke` |
| `apps/neobot-desktop/frontend/` | **参考树，不可交付**。商用已确认 ⇒ 上游「No Commercial Secondary Development」生效 | `nt_check_upstream_1to1`（只守「别漂移」） |

**vendored 树的一切数字都不是交付质量的证据。** `frontend/` 有 140 个文件
1:1、有 4 道门守着它 —— 而交付物当时连桌宠页都没有、macOS 菜单点了没反应。
**门全绿 ≠ 产品能用**：那些门查的是结构，而「界面是乱的」不是结构错。

### 上游该怎么用

**取布局判断，不取代码。** 该抄的是「44px 导航栏 + 控件等高」「侧栏行高与
选中态」「正文列宽上限」这类**决定**，不是那 731 行 `navbar.tsx`。
商用已确认 ⇒ 代码层面它是禁区，思路层面它是好教材。


## §1 已建成（可点可验）

### 1.1 前端 —— 16405 行

上游 `dsh-harness-desktop 0.19.1` 1:1（见 `frontend/VENDOR.md`）+ NeoBot 增量。
旧自研 UI（`tokens.css` / `MOCK_ITEMS` / `selftest.ts` 那套）的台账已删除 ——
文件都不存在了，台账留着就是谎言。增量清单：

| 文件 | 改动 | 依据 |
|---|---|---|
| `src/neobot-root.tsx`（新增） | 自持对话区：会话列表 + 消息流（调 `neobot_convo_list` / `neobot_send`） | 上游对话区是空 submodule 的 iframe，本仓无该运行时 |
| `src/api-panel.ts` + `src/api-panel.css` + `src/dom.ts`（新增） | API 契约可视化（设置 → API 页签） | 前端是后端的可视化交互 |
| `src/layout/components/webview.tsx` | `ready` 时 `selfHosted ? <NeoBotRoot/> : <Iframe>` | 空 service_url 时挂 iframe 会加载不存在的地址 |
| `src/store/modules/harness/store.ts` | `selfHosted` 状态；自持时跳过装依赖整段 | 逐条 stub 会让超时等待等到绝对超时 |
| `src/ui/dialog/config.tsx` | `ConfigTab` 加 `'api'` | 接口清单与配置面板同类信息 |
| `index.html` / `pet.html` | 标题 → NeoBot / NeoBot Pet | 顶层 chrome 去上游品牌 |
| `src/ui/dialog/about.tsx` | 兜底文案 → NeoBot | 同上（后端正常时显示后端值） |
| `src/i18n/locales/*.json` | 4 处 `DeepSeek Harness` → NeoBot | 同上；`app.wordmark: HARNESS` 不动（测试锁了它） |

⛔ 未动的：其余 130+ 文件与上游逐字节一致；`dsh://` 协议、`DSH_HOME`、
`dsh-tauri` 桥名一律不动 —— 那是标识符，不是品牌。

### 1.2 库侧（本仓唯一真源）

| 模块 | 行 | 测试 | 内容 |
|---|---|---:|---|
| `crates/neotrix-neobot/src/nt_evidence.rs` | 329 | 8 | 证据审计：断言有无出处 / 过度断言 / `sourced_ratio` 缺席≠0 |
| `crates/neotrix-neobot/src/nt_panel.rs` | 532 | 18 | 决策面板契约 + **过期作答检测** + **面板注册表** |
| `crates/neotrix-neobot/src/nt_pet.rs` | 791 | 8 | 宠物清单/导入/资产/种子（manifest/网格/压缩包三 cap，OpenGhost 口径） |
| `crates/neotrix-neobot/src/nt_cost.rs` | 356 | 6 | 计价 + 用量账本（按天按模型，OpenGhost 口径） |
| `crates/neotrix-neobot/src/nt_effect_key.rs` | 138 | 6 | 效果键（规范 JSON + sha256，rakazo 口径；补发去重在用） |

### 1.3 命令（22 个 neobot_，前后端两侧一致）

```
neobot_agent_run
neobot_convo_list
neobot_member_list
neobot_convo_group
neobot_convo_dm
neobot_send
neobot_convo_messages
neobot_evidence_summary
neobot_core_capabilities
neobot_panel_answer
neobot_panel_publish
neobot_panel_clear
neobot_panel_demo_publish
neobot_api_specs
neobot_api_call
neobot_usage_summary
neobot_memory_list
neobot_memory_add
neobot_memory_undo
neobot_member_add
neobot_skill_list
neobot_skill_install
```

Rust 侧共注册 74 个（22 neobot_ + 41 上游同名 + 11 显式拒绝）；
契约 `src/api.rs` 共 118 条：实现 63 / 显式拒绝 11 / 决定不做 39 / 待做 5；
上游分母 96（`builder.rs` 的 `generate_handler!`），已逐条登记，无未展开项。
对账门：`node scripts/ops/nt_check_api.mjs`。

### 1.4 资产

- 图标主稿：`icons/icon.svg` 彩豚（以上游黑剪影为基改海豚：喙+笑线+横尾+镰状背鳍），
  mac squircle 只描边不填底故透明；UI 档 `mark-mono.svg` 海豚简化版。
- 栅格 **20 PNG**（`tauri icon` 桌面档）+ ico + icns。
  ⛔ 2026-10-01 删掉 `icons/android/`（18 mipmap）与 `icons/ios/`（8 AppIcon）：
  本项目是 macOS 桌面应用（bundle 只产 `.app`/`.dmg`），三者未被任何配置引用 ——
  留在库里只会让人以为要维护。本数字是 STATUS 自校验门算出来的，删完即红。
- 同步 `skills/assets/icons/neobot/`（16→1024 全档 + ico）。

### 1.5 门禁（10 个，2 个已删除）

`nt_check_api`（契约三方对账）· `nt_check_bytes`（U+FFFD）·
`nt_check_status`（本文件与实测一致）· `nt_shot`（界面截图）·
`nt_check_upstream_1to1`（**vendored 前端与上游的差集 = 白名单**，含 macOS 菜单/窗口 chrome 断言）·
`nt_check_ui_calls`（**界面可达模块里的每个 invoke 都有已注册命令**，守 `frontend/`）·
`nt_check_ship_ui`（**交付路径门**：tauri 指向 neobot-ui + pet.html 在产物里 + 原生菜单已接线）·
`nt_check_visual`（**视觉门**：高度链/控件等高/行高下限/列宽上限/无溢出 —— 治「全绿但界面不能用」）·
`nt_check_layout` v2（stub-boot 真渲染：分支/几何/暗色/a11y/零异常/零未登记）·
`nt_check_interact`（真点：发送链参数/切会话换历史/记忆面板三步/宠物渲染/两页零异常）。
~~以下 2 个守旧自研 UI~~ **已删除**（2026-10-01 蜕皮）：
`nt_check_ipc` 的职责由 **`nt_check_ui_calls`** 承接且更强（走真实 import 图，
含 `@/` 别名；旧门只 grep 目录）· `nt_check_tokens` 守的
`frontend/src/ui/tokens.css` **根本不存在**（上游用 Tailwind），留着是空气。
前端 0 组自测（旧自研 UI 的 `selftest.ts` 已随旧 UI 退役）。

> 复现：`node scripts/ops/nt_check_{api,bytes,ui_calls,ship_ui,upstream_1to1,layout,interact,status}.mjs` ·
> `cargo test -p neobot-desktop`（库测试97条：库 26 + app 71）

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
| **Stub 与 Planned 必须分开** | 都是「不能做」，但一个是决定（DSH 运行时专属），一个是欠账。混在一起路线图失真 |
| **空 service_url，不编假 URL** | 编 `http://127.0.0.1:3080` 会让 iframe 去加载不存在的东西，错误搬到离原因很远的地方 |
| **桌宠 `visible` 恒等于 `enabled`** | 与上游同：临时收起已移除，用户关就是关 |
| **pet size 越界拒绝不收敛** | 收敛会让用户误以为设上了；显示层收敛是前端的事 |
| **分母是上游 Rust 96，不是前端 invoke 数** | 前端没调≠缺口（如 pet 窗没打开）。按 invoke 数对账会逼着把「没调」当缺口 |
| **vendor 文件永不手改** | 改即分叉 —— md5 对不上时先问「我改的还是上游变的」。样式禁取：只取渲染结构，配色现写 |

---

## §3 已知缺口（诚实列出，按严重度）

### 🔴 阻断级（无）

曾列的骨架面板注册表、`MOCK_ITEMS` 列表、`data_dir` 对齐、
`neobot_send` 无会话上下文四项**已闭合**。

### 🟡 应当修

| # | 缺口 | 说明 |
|---|---|---|
| 1 | ~~`neobot_send` 无会话上下文~~ **已闭合** | 问落库 → 跑 → 答落库全进 `messages` 表；切会话读 `neobot_convo_messages` 换历史 |
| 2 | ~~`neobot_core_capabilities` 前端未消费~~ **已闭合** | 对话区顶栏渲染模型 · 工具数；失败不渲染，不设静态默认 |
| 3 | 桌宠内容剩 1 Planned | 清单/导入/资产/预设恒空/窗口/种子已接；1×1 静帧**目检通过**；剩鼠标流（OS 线程，不可验不动） |
| 4 | 更新通道 3 个 Planned | `check/download/open_desktop_installer` —— 更新源与签名策略待定 |
| 5 | 远端桥 + 侧栏 2 个 Planned | `remote_bridge_ping`（SSH 管理待接）、`toggle_sidebar`（自持模式是 React 状态，待确认是否仍需后端命令） |
| 6 | 无 a11y 门 | ~~只有局部的 `aria-live`/`role` 处理~~ **已闭合**（布局门 ⑦ 查自持区 img/button/textarea） |
| 7 | 界面对比度/暗色未实测 | ~~深色模式未渲染验证过~~ **自持区已闭合**（布局门 ⑥ 翻色断言；气泡固定色有文档） |

### ⚪ 已知可接受

| # | 项 | 说明 |
|---|---|---|
| 8 | 16px 完整图标不可读 | 与 neotrix 自家图标同级。UI 侧用 `mark-mono.svg` 简化档 |
| 9 | 前端/Rust 两份 `DecisionPanel` 类型 | 跨语言无法共享，靠 `nt_check_api.mjs` ⑦守参数双向一致 |
| 10 | 49 个 Stub（DSH 运行时专属） | 插件/profile/预装/核心分发/档案/服务日志 —— 本仓不跑该运行时，决定不做 |

---

## §3b 这份清单被门守着

`scripts/ops/nt_check_status.mjs` 会核对本文的**关键数字**与实测是否一致：
前端行数 · 图标 PNG 数 · `neobot_` 命令清单（§1.3 逐个列全）· 库测试数 ·
门禁数 · 自测分组数。不一致即 FAIL。

**为什么需要**：`CAPABILITY-MAP-2026-09-29` 整张表腐化而**无人发现** ——
它长得像实测结果（有行号、有证据列），但没有任何东西去核对它。
本文是同一类文件（人写的数字 + 表格），所以给它配一道门。

门自己也腐化过两次（只扫一层目录；`endsWith(".ts")` 漏掉 35 个 `.tsx` 少算 7,163 行），
都是**门先报错**逼修门。门不可信时先修门，不改文档迎合错的门。

## §4 已知的假绿教训（本会话累计 39 次）

这一节比上面的功能清单更值钱 —— 每一条都是**「没看到失败」被当成「验证通过」**：

| # | 形态 | 后果 |
|---|---|---|
| 1 | 失败信息被 `2>/dev/null` 丢掉 | 6 个变异全报「0 失败」 |
| 2 | 变异 harness 先还原文件再跑门 | 同上 |
| 3 | 只跑运行时自测、漏 tsc | 2 个纯类型变异全报「0 失败」 |
| 4 | 改源码不重建（**测的是产物**） | 2 处 `min-height` 移除全报 PASS，差点去改门 |
| 5 | shell 变量没 `export` | 5 个 Rust 变异一个都没写入却全报「8 passed」 |
| 6 | 变异 harness 把**编译失败**读成「测试通过」 | `找不到("test result", "")` 后 `"FAILED" in ""` 为假 ⇒ 变异没编过却判 PASS |
| 7 | 空洞的测试：断言「不该发生的事」却**没构造那件事** | `版本倒退被拒` 没发生倒退，测试却绿着 |
| 8 | 项内注释用 `//!`（应为 `///`） | 11 处 E0753 —— 从旧文件抄注释格式抄错了边 |
| 9 | `open_path` 传 `PathBuf`（要 `Into<String>`） | E0277 —— opener 的 path 参数不是泛型 Path |
| 10 | 分母按 95 算（真值 96） | `update_app_config` 的 `#[allow]` 行隔在属性与 fn 之间，grep `-A1` 漏计 —— **裸 grep 的计数不构成证据** |
| 11 | 门正则只认字符串 note，不认 `DSH_ONLY` 常量 | 48 条解析失败，门报「契约 62 条」而实际 110 —— 门少算了近一半还 PASS 了别的项 |
| 12 | `endsWith(".ts")` 漏 `.tsx` | 35 文件 7,163 行没数 —— 与 #10 同类：**数数方式本身要被验证** |
| 13 | `move_pet_window` 自造绝对定位，pet 窗调相对增量 | 与 `log_frontend` 同类错，前后端各绿、产品坏 —— 而这次是**我亲手造的**，不是继承的 |
| 14 | `transparent` 在 macOS 被门控到 `macos-private-api` 后面 | 特性开了 Cargo 侧，构建脚本报 allowlist 不对 —— 配置键是 `macOSPrivateApi` 布尔，不是 `features` 数组 |
| 15 | 种子图硬套动画网格 | 方形静帧过不了 8×N 整除 —— 网格是动画表的形状，静态单帧记 1×1 并标注待目检，不硬凑 |
| 16 | `dark:` 变体全仓零命中还指望它翻色 | tailwind `darkMode:'class'` 要 `.dark` 类，应用只写 `dataset.theme` —— chrome 必须走壳语义 token，`dark:` 在此永远是死码 |
| 17 | 布局探针查 min-height 指定值，浏览器用生效值 | aside 指定 `auto` 但因 `overflow-y:auto` 生效为 0，滚动正常 —— 探针冤枉好人；按纪律显式加 `min-h-0`，两边都对 |
| 18 | `listen` 桩只 mock 了 invoke | unlisten 路径要 `__TAURI_EVENT_PLUGIN_INTERNALS__` —— 缺它报 5 次 `unregisterListener` 未定义，全是桩的错 |
| 19 | 桩里泛 `plugin:` 分支排在 MAP 前面 | 具体窗几何桩永远走不到，回 null 炸了 `new PhysicalPosition` —— 分支顺序也是逻辑 |
| 20 | 探针找 canvas/img/video，组件画的是 div 背景图 | 查错标签冤枉正常渲染 —— 先读现场（outerHTML）再写断言 |
| 21 | 空清单 vs 未注册看起来一样，实现前没读消费方 | `usePetSource` 对空走 `PET_NOT_FOUND` 可见诊断 —— 先证无退化再落地，否则「恒空」可能是把诊断链掐了 |
| 22 | 门模板里的换行先求值，桩源码的单引号被截断 | addScript 对语法错静默丢弃 —— 注入前 node 侧验桩语法，失败直接报（连注释里的示范都不许写单写） |
| 23 | 去重键只含内容，会吞掉合法重复 | "谢谢" 发两次是两句话 —— 键必须含意图域（task），内容相同意图不同才是新行 |
| 24 | 前端加了新调用，桩没跟 | 布局门「未登记调用」先红 —— 严格未登记断言的意义：新接口不更新桩就过不了门 |
| 25 | zsh 重定向顺序当证据 | `2>&1 >/dev/null` 显示"stderr 有内容"，差点误判 `memory get` 双写；改 `>/tmp/o 2>/tmp/e` 量字节才定案（39/0） |
| 26 | 手推 append 的历史条数 = 实际 | 我以为"第一次 append 不留历史"，测试当场打脸：历史记的是**写前**整份，第一次的"写前"是空串。改断言不如改事实 |
| 27 | 改 `.rs` 后不重跑门 | 每轮 STATUS 门自己抓到过期数字（本轮：前端行数 15834→15858、命令 15→16、测试 87→89），三次全红 |
| 28 | 门的测试选择器写死下标 | 交互门原先用 `aside button[1]` 点会话；加记忆面板后按钮顺序变了，按下标会点错 —— 而门照样可能绿。改成**按文本找**（`includes('记忆')` / `includes('撤一版')`），下标漂移不再静默 |
| 29 | 只在终端接的能力 = 半个能力 | memory 补完库+CLI 就记完成 ⇒ 用户在 GUI 里既看不到也改不了。接线口径：GUI 是主面，CLI 是补充面 |
| 30 | 「配置读起来对」≠「跑得起来」 | `beforeBuildCommand` 写 `pnpm --dir ../frontend`，而 `tauri.conf.json` 就在 `apps/neobot-desktop/`、frontend 在其**子**目录 ⇒ `../frontend` 指向不存在的 `apps/frontend`。纸面核对（路径存在、命令存在）全过，真跑第一步就死 |
| 31 | 降级工具链来"解决"版本不匹配 = 制造新错 | tauri 2.12 ↔ @tauri-apps/api 2.11 不匹配时，我先把 Rust 侧降到 2.11.1 ⇒ **tauri 自己编译失败**（`UnexpectedMenuKind` 找不到）。正解是升 JS 侧到 2.12.0，不是把库拆散 |
| 32 | 拿 `git checkout` 撤销自己的实验 | 降级实验后想回退，先 `git stash` 看一眼更安全 —— `checkout` 直接覆盖，且工作区是共享的（AGENTS.md 并行公约） |
| 33 | **前端逐字一致 ≠ 壳层行为一致** | macOS 原生菜单从未移植：前端 141 个文件里 135 个逐字一致、32 个 testid 一个不缺、tsc 过、10 个门全绿，而用户在 macOS 上**点不到设置/关于/更新**。差异不在前端，在 Rust 侧 ⇒ 只能靠「跨语言对账」发现。⇒ 新门 `nt_check_upstream_1to1.mjs` 同时断言前端差集白名单 + Rust 侧菜单/chrome 存在 |
| 34 | 补 A 功能时顺手引入 B 的 bug | 补 macOS 菜单时若只补文件/帮助，⌘C/⌘V 会因缺编辑菜单而失效（上游 issue #85）—— 「原先没有菜单」是**侥幸**，不是正确。把这类连带项写进门断言 |
| 35 | 写下「上一轮刚犯过」的教训，下一轮原地再犯 | 交互门 `aside button[n]` 下标点击：我上一轮把它写进教训 28，本轮给侧栏加了个搜索框就又踩了 —— **教训写进文档不等于行为被改**，要改行为就得换一个不会被下标漂移打破的选择器 |
| 36 | 截图里「看不出差别」≠ 差别不存在 | 选中态与 hover 同色时截图完全正常（无报错、无溢出、a11y 过），只有逐像素比对才发现。判据改成**量 computed backgroundColor**，并固化成门 |
| 37 | 颜色变量存在 ≠ 工具类存在 | `--color-nav-active` 写在 `:root` 里，但 Tailwind 只为**注册过的**颜色生成类，`bg-nav-active` 静默不存在 ⇒ 改用已注册且取值相同的 `bg-btn-active`。证据是构建后 grep 产物 CSS |
| 38 | 「查不了」被写成「通过」 | 参考树被移走时，1:1 门打印 SKIP 并 exit 0 —— 门从「证明没漂移」退化成「什么都没做」，而 CI 眼里一样绿。缺前提必须 FAIL |
| 39 | 门把自己的注释当数据 | api 门从我写的「见 desktop.rs」里读出命令 `rs`；字节门从注释里读出损坏字符。**门犯的错和它要抓的错常常同一种** —— 判据要能防住自己 |

**共同根因**：把「没有观察到失败」当成了「验证通过」。

---

## §5 下一步（按依赖排序）

1. ~~`neobot_send` 带上 `convo_id`~~ **已闭合**（问答落库 + 历史命令 + 切会话换历史）。
2. ~~能力矩阵消费 `neobot_core_capabilities`~~ **已闭合**（顶栏直读，无静态默认）。
3. ~~桌宠内容~~ **基本闭合**（清单/导入/资产/预设恒空/窗口/种子/静帧目检；剩鼠标流）。
4. ~~a11y + 暗色~~ **已闭合**（自持区：布局门 ⑥⑦ 守翻色与可读名；气泡固定色有文档）。
5. ~~布局门复活~~ **已闭合**（v2 stub-boot 真渲染生产包）。
6. ~~交互链~~ **已闭合**（交互门真点：发送参数/切会话/宠物渲染/截图留档）。
7. **真机联调** —— 运行中 Tauri app 内（WKWebView）：目前 Chrome 侧全通，
   剩 WKWebView 容器差异（WebKit vs Blink）。
   ✅ **打包链已实跑**（2026-10-01 首次）：`tauri build` 成功出
   `target/release/bundle/macos/NeoBot.app` + dmg，`strings` 里查得到
   `neobot_memory_list/add/undo` 三条注册；**原生菜单栏已在真机 AX 层核验**
   （见 §8）。**跑之前先修的三处纸面错**（见 §7）。
8. **5 Planned 收敛** —— ~~`toggle_sidebar`~~ **已裁决为 Stub**（上游本体即空操作，
   且无任何前端调用方）；更新 3 与 CLI 各有**出口条件**（见契约 note，
   无通道/无二进制不动）；`remote_bridge_ping`（SSH 后端）、
   `start_pet_mouse_stream`（OS 线程，不可验不动）。
9. **OpenGhost 吸收下半场** —— 本轮取了渲染三件套 + 账本口径；剩下：
   `diagram.js`（先读架构再定）、approval 效果分级（自持区无面板 UI，先欠着）、
   OS 密钥（等 keyring 插件）、agent-tools 风险表（本仓 policy 已有覆盖，增量再议）。
10. **rakazo 吸收下半场** —— 本轮取了 effect-key（含补发接线）+ 验证信条；剩下：
    连接器审批模式（无 MCP 消费者，先欠着）、ask 快照（自持区无面板 UI）、
    memory 修订/导出（nt_memory 对照后定）、voice（无音频管线，大件单立项）、
    computer 运行时（要 Docker，先读 `computer-runtime.md`）。
11. ~~用量读侧~~ **已闭合**（`neobot_usage_summary` 日/周/月 + 顶栏今日数；文件缺席=零，坏文件=Err）。
12. ~~memory 对照~~ **已裁决**（见 §5 对照表）：只补「修订/撤销 + 原子写」两件；
    检索/外部 provider 明确**不追**（无消费者 / 与零外部 I/O 冲突）。
13. **memory 已接到 GUI**（`neobot_memory_list/add/undo` + 侧栏面板）——
    上一轮只补了库与 CLI，等于「用户看得见的能力藏在终端里」；
    面板默认收起、容量 `bytes/cap` 同屏可见（撞上限时能看见为什么记不进）、
    拉不到就明说读不到不装作空；交互门新增「展开→记→撤」三步实测。

### §5 rakazo memory ↔ `nt_memory` 对照（2026-09-30 实读 `packages/memory/src/index.ts`）

| 能力 | rakazo | NeoBot 裁决 | 理由 |
|---|---|---|---|
| 存储形态 | Postgres 多文档 + `scope`/`botId` 维度 | 单文件 `MEMORY.md`，8KiB 上限 | 不追：只有一个用户一个 bot，多文档是它多租户的形状 |
| 写原子性 | `Serializable` 事务 + 重试 | ❌→✅ `tmp + rename` | **真缺陷**：`fs::write` 是截断直写，崩一次整份归零；记忆是用户手打、不可再生。同款修法见 `nt_cost::UsageLedger::save` |
| 修订/撤销 | `memoryRevision` 表逐版留档 | ❌→✅ `MEMORY.rev.jsonl` + `neobot memory undo` | **真缺口**：全文逐轮注入 prompt，改错了没法回头。撤前把当前存回历史 ⇒ 撤销可再撤销 |
| 历史上限 | 无限（DB 撑） | 20 版（`MEMORY_REV_CAP`） | 不设上限 = 每记一行永久留一份 8KiB，磁盘只增不减 |
| 坏历史行 | DB 不会坏 | **Err，拒猜** | 跳过会让 undo 拿回「合法但不是上一版」的内容；静默回退错版本比拒绝回退坏 |
| 检索 | 子串 + snippet | **不追** | 没有检索消费者（无记忆 UI），且全文已在 prompt 里，检索对 8KiB 无意义 |
| 导出/导入 | `exportMarkdown`/`importMarkdown` | **不新增** | 文件本身就是可导出的 markdown（已在 `<data_dir>/MEMORY.md`） |
| 外部 provider | supermemory 等托管记忆 | **不追** | 与本仓「零外部 I/O」基线冲突 |

实测（CLI 真跑，`NEOBOT_DATA_DIR=/tmp/nb-mem-smoke`）：
set/set/get/undo/undo 四步正确回退又前进，`MEMORY.rev.jsonl` 4 行。

⛔ 一次自证失败的读法：`2>&1 >/dev/null | head` 在 zsh 里显示"stderr 有内容"，
差点当成"get 同时写了两条流"。改用 `>/tmp/o 2>/tmp/e` 量字节数才定案：
stdout 39 字节 / stderr 0。**管道顺序下的重定向不可当证据。**

---

## §7 打包链实测（2026-10-01 首次真跑 `tauri build`）

命令：`apps/neobot-desktop$ ./frontend/node_modules/.bin/tauri build --no-bundle`
（CLI 从 `frontend/node_modules/.bin` 取，因为 `tauri.conf.json` 在 app 目录而非 `src-tauri/`。）

### 跑之前纸面全过、跑起来当场死的三处

| # | 纸面看着对 | 实际 | 修法 |
|---|---|---|---|
| 1 | `tauri.conf.json` 有 `beforeBuildCommand`，路径存在 | 写成 `pnpm --dir ../frontend` ⇒ 指向 `apps/frontend`（不存在）。tauri 以**本目录**为 cwd 跑 hook | 改 `pnpm run build`（cwd 已是 frontend —— 由 `pwd` 探针实测得到，不是猜的） |
| 2 | `Cargo.toml` 写 `tauri = "2"`，能解析 | 解析出 2.12.0，而 `pnpm-workspace.yaml` 锁 `@tauri-apps/api ^2.11.1` ⇒ **tauri CLI 硬拒**（`Found version mismatched Tauri packages`） | 升 JS 侧：catalog `@tauri-apps/api`/`cli` → `^2.12.0`、`plugin-store` → `^2.5.0`，`pnpm install` |
| 3 | 降级 Rust 侧到 2.11.1 能"对齐" | ⛔ **tauri 2.11.1 自己编译失败**（`error::Error::UnexpectedMenuKind` 不存在，9 个错误全在 tauri 内部） | 撤销降级（`git checkout Cargo.lock` + `cargo update -p tauri`），走第 2 条 |

### 结论

- 产物：`target/release/neobot-desktop`，Mach-O 64-bit arm64，7.0MB。
- 注册核验：`strings` 命中 `neobot_memory_list` / `neobot_memory_add` / `neobot_memory_undo`
  —— 命令确实编进了二进制（不是「源码里有」）。
- 全量回归：lib 471 · app 65 · 五门 PASS。
- ⛔ **本节只证明「打包链通 + 命令编进去了」**。**不证明**运行时正确：
  WKWebView 容器差异、窗口级操作、真实 LLM 调用仍未验（§5 第 7 条）。
- ⚠️ `--no-bundle` 不产 `.app`/dmg；要真分发还需 `tauri build`（含 bundle targets）与签名。

## §8 上游 UI 1:1 审查（2026-10-01，用户要求逐组件复核）

方法：`diff -rq` 逐文件比 `apps/neobot-desktop/frontend/src` 与上游 `src/`，
再用真机（`open .app` + Accessibility 层）核验**行为**，最后把差集固化成门
`nt_check_upstream_1to1.mjs`。

### 结论：前端 1:1 早已成立，漏的是 Rust 侧的壳层 chrome

| 面 | 上游 | 本仓（审查前） | 判定 |
|---|---|---|---|
| `navbar.tsx` 731 行 | 逐字 | **逐字** | ✅ |
| 5 个配置面板（core/profile/plugin/debug/backup，共 2486 行） | 逐字 | **逐字** | ✅ |
| 9 个通用组件（modal/panel/logs/toast/item/empty/info/ellipsis/primitives） | 逐字 | **逐字** | ✅ |
| 4 个对话框（update/update-core/core-upgrade-profile/about） | 逐字 | about 仅换品牌兜底串 | ✅ |
| 桌宠页 `src/pet/`（app/constants/main.css/main/hooks/utils） | 逐字 | **逐字** | ✅ |
| 32 个 `data-testid` | 32 个 | 33 个（多的 1 个是本仓自持根） | ✅ 无缺失 |

### ⛔ 真正没做到的三处（全在 Rust 侧，前端门一个都抓不到）

| # | 缺口 | 后果 | 修法 |
|---|---|---|---|
| **1** | **macOS 原生菜单栏从未安装**。`navbar.tsx`（逐字未改）在 macOS 上把「文件/运行/帮助」整组用 `<If cond={!IS_MACOS}>` 隐藏，假定它们由 `builder.rs::install_macos_menu` 承载 | macOS 上**设置 / 关于 / 运行日志 / 检查更新 / 文档 / 新建窗口 / 新聊天 / 打开文件夹全部无入口**；且前端 `useListen('macos-menu-action')` 那一整段分发是**死代码** | 新增 `apps/neobot-desktop/src/menu.rs`（上游 `install_macos_menu` 移植），`main.rs` 的 `setup` 调 `menu::install` |
| **2** | **主窗 chrome 不一致**：`tauri.conf.json` 无 `titleBarStyle: Overlay` / `hiddenTitle` / `trafficLightPosition` | macOS 上多出一条**独立标题栏**；导航栏左侧 `pl-20` 让位给**不存在**的交通灯 ⇒ 「标题栏 + 空导航栏」两层 | 按上游值补齐三键，`y = SHELL_NAV_HEIGHT/2 - 2.5 = 19.5` |
| **3** | **窗口尺寸**：1280×820 / min 940×600 | 与上游 1280×840 / min 860×620 不一致 | 改为上游值；三个截图门的默认视口同步改 840 |

**为什么「编辑」菜单不是可选项**：macOS 一旦挂上主菜单，⌘X/⌘C/⌘V/⌘A 会先经菜单
key-equivalent 路由，不挂标准编辑项则 WebView 输入框**无法复制粘贴**（上游 issue #85）。
本仓原先「没有菜单」反而侥幸没触发这个 bug —— 补菜单时必须一起补，否则是**把 bug 装进来**。
门 `nt_check_upstream_1to1.mjs` 逐项断言 `copy/paste/select_all` 存在。

### ✅ 真机核验（Accessibility 层，非截图猜测）

`open NeoBot.app` 后读 `menu bar 1`：

- 菜单栏：`Apple, NeoBot, 应用, 文件, 编辑, 帮助`（5 个，与上游同序）
- 应用：`设置…, 进入全屏幕`
- 文件：`新建窗口, 新聊天, 打开文件夹, 关闭, 退出`
- 编辑：`撤销, 重做, 剪切, 复制, 粘贴, 全选`
- 帮助：`运行日志, 重启, 检查更新, 文档, 关于 Desktop`
- 窗口实测 `1280x840`

⛔ **诚实边界**：交通灯纵向对齐**只核到配置值**（`y=19.5`），未做像素核验 ——
本机无屏幕录制权限（`screencapture` 报 `could not create image from display`）。
AX 报按钮框中心约在窗口顶端下方 17px，而上游公式推得约 21.5px；差值可能来自
AX 命中框（16×16）与视觉圆心的差，也可能来自 tao 的 inset 行为。**不下结论**。

## §9 侧栏 / 对话区 UI 优化（2026-10-01）

判据不是「好看」，是**先找出功能缺陷再改**。逐条都带「原来会怎样」。

### 侧栏

| 问题 | 原来 | 现在 |
|---|---|---|
| **没有新建会话入口** | 空态把人赶去「设置 → 应用」建，而那里也没有 ⇒ 死路 | 「+」表单：标题 + 成员 id（可空），提交即建并选中。成员可**就地登记**（`neobot_member_add`，幂等）——因为 `create_conversation` 会校验成员已登记，而 GUI 此前只有 `neobot_member_list`：**能看不能建** |
| **选中态与 hover 态同色** | 都是 `bg-panel-hover` ⇒ 截图里根本看不出选中哪个 | 选中用 `bg-btn-active` + 左侧 `inset 2px` 强调条；hover 用 `bg-btn-hover`。布局门新增断言：**选中/未选中底色必须不同**（实测 `rgba(38,49,72,0.1)` vs `transparent`） |
| **记忆面板会被长列表滚走** | 整栏一个 `overflow-y-auto` + `mt-auto`（`mt-auto` 在滚动容器里本就无效） | 侧栏拆三段：筛选/新建（不滚动）· 列表（独立滚动，`nb-convo-list`）· 记忆（钉底） |
| 40 条会话翻找靠滚 | 只有滚动 | 搜索框（标题/成员/id 任一命中；**只筛不重排**，排序是后端 `last_active` 语义） |
| `muted` 字段拿到了却没用 | 静音会话与普通会话长得一样 | 标题右侧 🔇 + title 说明 |
| 错误框固定 `bg-red-50` | 暗色侧栏里一块浅粉 | `bg-btn-danger-hover`（明暗都有取值） |
| 空态文案指错路 | 「在设置 → 应用里建一个」 | 「点右上角 + 建一个」 |

### 对话区

| 问题 | 原来 | 现在 |
|---|---|---|
| **输入框不随内容增高** | 占位符承诺「Shift+Enter 换行」，框永远 `rows=1` ⇒ 用户看不见自己打的第二行 | 按 `scrollHeight` 自增，上限 160px 后转内部滚动 |
| **新消息不自动滚到底** | 长会话里发完看不到自己的气泡 | 自动跟随，但**仅当用户本来就在底部**（阈值 48px）；滚上去后不再硬拽，并出现「回到底部 ↓」 |
| 失败气泡与正常回复无法区分 | 只是一段 `发送失败：…` 文本 | 标 `failed` + 气泡下方「重发」（重发替换原气泡，不留残骸） |
| 无时间 | — | 每条气泡下方 `HH:MM`（跨天补 `M/D`） |
| 「正在想」只在按钮上是个 `…` | 看不到 bot 是不是还在跑 | 消息流末尾脉冲点「正在想…」 |

### 门在这一轮抓到的自己的错

- **交互门 `aside button[1]` 又踩了一次下标陷阱**（上一轮刚写进教训 28）：侧栏顶部加了搜索框与「+」后，
  下标位移，门点到的不是会话2 却仍可能绿 ⇒ 已改为**按标题文本找**，等列表用 `nb-convo-list` testid。
- **新交互必须进门跑**：新建会话的三个调用只存在于**默认收起的表单**里，不点开就永远不会发生 ——
  「没跑过」和「跑通了」在门眼里一模一样 ⇒ 交互门新增「展开→填→建」三步，实测后端收到 `neobot_member_add(id=neo)`。
- 布局门的滚动断言原本打 `aside`，侧栏分段后它去断言一个不滚动的壳（报 `overflow-y=visible` ——
  **报错对但指错对象**）⇒ 改用 `nb-convo-list` testid。

### 踩坑两则

- `bg-nav-active` / `focus:border-info` **在 Tailwind 里不存在**：颜色必须在
  `tailwind.config.js` 注册才会生成工具类，而这两个 CSS 变量只在 `main.css` 的 `:root` 里。
  该文件自己的注释已记过一次同款坑（加载页 token）。⇒ 改用已注册且**取值逐条相同**的
  `bg-btn-active`（0.14/0.1）、`bg-btn-hover`（0.08/0.06）。证据是构建后 grep 产物 CSS，不是读配置猜的。
- 在门的**模板字符串**里写注释用了反引号，直接截断字符串 ⇒ node 报 `SyntaxError: Unexpected identifier`。
  `bash -n` 与 tsc 都抓不到，只有真跑门才会暴露。

## §10 设置面板裁决：26 处「调了但没注册」（2026-10-01）

### 起因：一门没跑出来的数字

上游 5 个配置面板 + navbar/recovery/iframe/preinstall 共 **26 处 `invoke('X')`
在本仓从未注册**。它们的失效方式极其安静：react-query 拿到 reject 后回落到
默认值 `[]`，面板画出**空列表** —— 于是「没有插件/没有档案」看起来像**事实**，
而不是「这个功能在本仓不存在」；按钮点了才报 `command not found`，
而没人会为空白面板点按钮。

tsc 不报、布局门不报、交互门不报（它没点那些按钮）—— **没有任何既有门会红**。

### 裁决与做法

| 面板 | 裁决 | 做法 |
|---|---|---|
| 档案 | **不提供**（库侧 `nt_config` 无 profile API） | 页签不渲染；3 条 profile 命令注册为显式拒绝 |
| 插件 | **不提供**（DSH 插件体系整条不存在） | 页签不渲染；9 条插件命令注册为显式拒绝 |
| **技能** | **本仓真正的扩展件** | **新增页签**，数据源 `nt_skills::scan_skills`（`skills/` 目录）。技能与插件在用户视角是同一件事（让 agent 多会一件事），所以顶替那个位置而不是新开第三个入口 |
| 核心 | 保留 | `get_cores` 早已映射到本地 provider；`download_core`/`update_local_core` 注册为显式拒绝 |
| 应用 | 保留 | `get_cli_link_status`/`copy_service_url` 注册为显式拒绝 |

### 契约新增一档：`Status::Refused`

⛔ `Stub` 与 `Refused` 的区别不是「做没做」，是**注册了没有**：

- 未注册 ⇒ 调用方拿到 `command not found`，那读起来像 **bug**；
- `Refused` ⇒ 拿到「本仓不提供 X」，那读起来像 **决定**。

契约门同步加**双向**断言：`Stub`/`Planned` 必须未注册，`Refused` **必须**已注册
（标了却没注册 = 承诺没兑现，界面又静默炸回去）。

### 新门 `nt_check_ui_calls.mjs`

走**真实 import 图**（含 `@/` 别名），断言「界面可达模块里的每个 `invoke` 都有已注册命令」。

- ⛔ 只扫目录会把「已下线但文件还在」的 vendored 文件也算进来 ⇒ 要么误报，
  要么逼人删上游文件（更大的 1:1 声明）。只查**可达**模块。
- ⛔ 第一版只认相对路径，漏掉全仓都在用的 `@/…` ⇒ 报出「只有 15 种调用」的
  干净结论而真凶一个没看见（84 个模块 / 61 种调用才对）。
- 结果：26 → 3（剩 3 条来自上游 `layout/index.tsx` 静态可达的 profile hook，
  运行期走不到但「静态可达」正是「哪天接了更新流程就会真调它」的准确预言）→ 0。

### 本轮门抓到的自身错误

- **字节门抓到我的写入损坏多字节字符**：`config.tsx` 注释里的「的按钮」三字被切坏成替换字符。
  同一处还有一处更早的损坏（`neobot-ui/src/i18n/index.ts`）也一并修了。
  这正是那道门存在的理由 —— 它在注释里，但同一个 bug 落在字符串字面量上就会静默改变行为。
- **api 门把注释里的字当命令名**：我在 `main.rs` 注册块里写「见 desktop.rs 段头注释」，
  正则把 `desktop` 与 `rs` 当成命令 ⇒ 门报「Rust 注册了 rs」。已让门先剥行注释。
  **门犯的错和它要抓的错是同一种。**
- `upstream_total=96 vs 97`：我批量改状态时正则跨行匹配，重复插入了 `copy_service_url`。
  由 `api::tests::名字不许重复` 抓住 —— 该测试的价值在这次得到兑现。

### ⛔ 参考树被移动，1:1 门当时是 SKIP + exit 0

`Downloads/deepseek-harness-desktop-0.19.1`（及 OpenGhost / rakazo）被移到
`Downloads/Neo/GitHub/` 下（zip 也在回收站，**未丢失**）。而 1:1 门当时
打印 `SKIP` 并 **exit 0** —— 「查不了漂移」长得和「没有漂移」一模一样。

已改为：`NB_UPSTREAM` 环境变量 → 两个已知落点；**找不到就 FAIL 并打印找过的路径**。

## §6 许可边界（不可越过）

| 仓库 | 许可 | 边界 |
|---|---|---|
| deepseek-ai/deepseek-harness | MIT | ✅ 代码可用；**商标不可用**，项目内只用 `DSH` 缩写 |
| dsh-tauri/deepseek-harness-desktop | MIT | ✅（前端 1:1 + favicon 派生，NOTICE 见 `frontend/VENDOR.md`） |
| ANDRETRIPOL/OpenGhost | **MIT（源码）+ 非商用保留（品牌/动画/视觉设计）** | ✅ 仅取渲染引擎三文件（逐字，md5 锁）；禁取样式/动画/品牌。NOTICE 见 `frontend/src/vendor/openghost/VENDOR-OPENGHOST.md` |
| elie222/rakazo | **Apache-2.0** | ✅ 取思想与移植模块（effect-key、二三次确认）；须保留版权声明、标注改动（含专利授权）。移植头注写明出处与偏离 |
| omdsh-dev/DSH-better-sidebar | MIT | ✅ |
| huiliyi37/dsh-tianshu-tui | **Apache-2.0** | ⚠️ 须保留 NOTICE、标注改动、含专利授权 |
| thinkany-ai/douchat | **AGPL** | ⛔ **不可取码**，只读结构与思路 |
| nightly-labs/openbot | **PolyForm Noncommercial** | ⛔ 不可取码（仅限非商业） |
| hikariming/dshfind | **无 LICENSE** | ⛔ 无许可证 = 保留一切权利，只读 README |

图标为原创构造（海豚），不临摹任何现有图标；favicon 黑豚剪影派生自上游（MIT，已注来源）。
