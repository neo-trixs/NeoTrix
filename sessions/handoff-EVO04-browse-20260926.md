# EVO-04 高速浏览器环 Spec（抄 browser-use / jev-ultrafast）

- 日期：2026-09-26 / 分支：`feat/capability-absorb-20260828`
- 身份：NeoTrix 规格代理（EVO-04 spec only）
- 现状备注：任务指定的 `neotrix-core/src/l1_action/nt_io/nt_io_browser_engine.rs` 已不存在（现为目录 `nt_io_browser_engine/`：`mod.rs`(727行，内核门面+UA池/超时/资源上限）+ `engine.rs`(51行，BrowserEngine门面）+ `engine/nt_*` / `session.rs` / `types.rs` 等）。本 spec 只读前 40 行了解结构，不做任何引擎改动，不代修 browser 窗领地的 3 处缺参。

## 1. 目标：高速环（默认环）

抄 browser-use 原子快照 + jev-ultrafast 单 RTT 决策：一次快照 → 一次 operation+target 决策 → 一次执行。默认无截图环，截图只作异常取证。

### 1.1 原子 DOM 快照表

- 格式：`[id] type name/value` 单行一张表，可交互元素才有 id。
  - 例：`[12] button 'Search' / [13] combobox 'Country' value='CN' / [14] textbox 'Email' value=''`
- 原子性：快照带 `snapshot_id + url + seq`，决策必须引用该 `snapshot_id`，过期即废弃重拍。
- 裁剪：只留可交互 + 文本锚点；link 200 / 正文 20k / 表单 20 / 字段 50 上限沿用现有内核声明（见 `mod.rs:25`），不另加配额。

### 1.2 单请求 operation+target 决策

- 决策体单 JSON：`{ snapshot_id, operation, target_id, args, expected }`
- `operation ∈ { click, fill, select, press, scroll, goto }`，`target_id` 为快照表 `[id]`，禁止 CSS/XPath 直写（只允许快照 id 解析后引擎内映射）。
- 单 RTT：LLM 一次只发一个决策；需多步则由上层环循环，不在一次请求里打包多 op。

### 1.3 执行前遮挡 / 新鲜度校验

- 执行前（引擎内部，不加 LLM 往返）必查两项：
  1. 新鲜度：`snapshot_id == latest` 且 url 未跳变，否则返回 `StaleSnapshot{latest_id}`，上层重拍重试。
  2. 遮挡 / 可操作性：target bounding box 零面积 / `display:none` / 被遮挡 / disabled → 返回 `Occluded{reason}`，不上报成功。
- 校验失败不计入 action history 成功项，只记 audit 元数据（无正文）。

### 1.4 等待按需

- 默认零等待；只在两类情形加短睡：
  - `combobox/select` 展开：200ms
  - 其他（click 后下拉 / fill 后联想）：50ms
- 禁止固定 `sleep(1s+)`、禁止截图轮询等待。动态内容用一次按需重拍快照确认，不用忙等。

### 1.5 无截图默认环

- 默认环：快照表（文本）→ 决策 → 执行 → 新快照 diff。截图默认关闭。
- 截图仅在 `Occluded` 连续 2 次 / shield 风控触发 / 人工取证时按需开一次，不进主循环。

## 2. 三步落地（归属均为 browser 窗）

### 步骤 1 — 快照表（Snapshot Table）
- 做什么：在现有 Http/Cdp 后端之上加 `snapshot(snapshot_id, url, seq, elements[][id,type,name,value])` 文本序列化器，复用现有 DOM/scarper 输出，不动抓取路径。
- 完成定义：`Mock + Http` 下单测可断言 `[id] type name/value` 行级稳定输出；行数受既有资源上限约束。
- 归属：browser 窗。
- 禁事项：不改 `BrowserEngine::execute` / `BrowserAction` / `BrowserResult` 签名；不碰会话/Cookie/UA/超时逻辑；3 处缺参由 browser 窗自修，本 spec 不代修。

### 步骤 2 — 单 RTT（Single operation+target）
- 做什么：新增上层决策 DTO `{snapshot_id, operation, target_id, args}` → 引擎内 `target_id→locator` 一处映射；LLM 侧单请求单 op。
- 完成定义：一次决策往返完成 click/fill 各一例 e2e（含 Http 回放 + Cdp 真提交各一）；多 op 打包被拒并返回明确错误。
- 归属：browser 窗。
- 禁事项：不新增 operation 种类之外的协议字段；不改引擎 struct 字段 / 公开函数签名；XPath/CSS 直写入口不新增。

### 步骤 3 — 校验（Occlusion + Freshness + 按需等待）
- 做什么：执行前加 `freshness + occluded/disabled/零面积` 双检 + 按需等待（combobox 200ms / 其他 50ms）+ 失败码 `StaleSnapshot / Occluded`。
- 完成定义：构造过期快照与被遮挡 target 的单测各一，皆返回对应错误码且不记成功历史；默认环全程无截图跑通。
- 归属：browser 窗。
- 禁事项：不改全局超时双保险与资源上限（`mod.rs:24-25`）；截图开关默认保持关，不将截图加入主环；不碰 `nt_io` 外模块。

## 3. 非目标 / 红线（本 spec 代理亦遵守）
- 本次只新建本 spec 文件 + 只读引擎文件；未改任何代码文件，未跑 cargo，未做 git 写操作。
- 引擎签名冻结：`BrowserEngine` 字段、`execute`、`BrowserAction/Result`、`BrowserConfig/Session` 均不改。
- browser 窗领地：`nt_io_browser_engine/` 下任何文件不碰；3 处缺参不代修。
