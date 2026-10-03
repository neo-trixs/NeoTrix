# 外部吸收：zeron + OpenDots（2026-10-03）

> ⛔ **取证边界（先声明，按本仓纪律）**
> - **zeron**：`git clone --depth 1` 成功，一手源码。`LICENSE` = **MIT**（Copyright (c) 2026 Wing）⇒ 吸收**无许可障碍**。
> - **OpenDots**：`git clone --depth 1` 成功，一手源码。`LICENSE` = **MIT**（Copyright (c) Atai Barkai）。
> - ⛔ **Claude Code mods**：clone **失败**（`Connection reset by peer`）⇒ 该节全部来自
>   官方博客 + `code.claude.com/docs` + 第三方社区逆向（2.1.286/2.1.287 实测），
>   **非一手源码**。故只采纳**设计思路**，⛔ 不采纳任何实现细节。

---

## 1. zeron —— 能力/工具可信度的「证明分级」

### 1.1 事实（F1，源码 `crates/harness/src/code_signature.rs`，350 行）

zeron 要装 6 个外部 agent（`claude`/`codex`/`cursor`/`opencode`/`pi`/`acp`）。
问题是「registry 上的版本可能比我在源码里 pin 的更新」。它的解法是**两级证明**：

| 级 | 证明 | 适用 |
|---|---|---|
| 1 | **pinned digest**（源码里钉死的 sha256） | 版本在 pin 内 ⇒ 最强 |
| 2 | **平台代码签名**（registry 版本新于 pin 时兜底） | macOS = Apple Developer ID team `EQHXZ8M8AV`；Windows = 发给 Google LLC 的 Authenticode 叶证书 |

⭐ 5 条可采纳的设计细节：

1. **每个**解压出的文件都要验签，**不只是主二进制** ⇒ 防「签名的主程序 + 被替换的旁文件」。
2. **「谁的签名」是判据**，不是「有没有签名」⇒ 身份被钉死（`GOOGLE_APPLE_TEAM` 常量）。
3. ⭐⭐ `pub const SUPPORTED: bool = cfg!(any(target_os="macos", windows));`
   并明写「linux 构建无签名 ⇒ pinned digest 是**唯一**可接受的证明」
   ⇒ **证明能力不可用时显式降级 + 说明替代证明，绝不静默 pass**。
4. 非普通文件（symlink 等）**直接拒绝**（`is_file()`/`is_dir()` 二分，无 else 放行）。
5. 文件 `sort()` 后逐个验 + 180s 超时 ⇒ 顺序确定 + 不被慢速签名服务拖死。

### 1.2 ⭐ 它**独立验证**了我们已有的设计

`nt-core-capability-tree/src/registry.rs:21-26` 的 `Epistemic` 轴注释：

> 「未解析」是一等状态：`supported` 这个数字究竟是 **真实上限**，还是
> **我们目前能证明的地板**？证据缺失时二者不可区分… 故显式携带此轴，
> 避免 CI 门禁把「没登记证据」印成「已查清不够格」。

⇒ 与 zeron 的 `SUPPORTED` 常量**是同一个洞察**：把「证伪了」与「没证」分开。
⭐ 这是**外部独立印证**，不是新增需求 ⇒ 该轴应当保留，不要在后续重构中被「简化」掉。

### 1.3 ⭐ 已落地：解掉了涌现门的冷启动死锁

`dc5b3e41` 的 `neobot-check-emergence.mjs` 直接采用第 3 条的三态语义：

| 态 | 含义 |
|---|---|
| `PASS` | 接线在 + 成熟度审计绿 |
| `COLD_START` | 接线在，静态门拿不到运行期节点数 ⇒ **证据不足，非缺陷** |
| `FAIL` | 接线不在，或审计红 |

⛔ `COLD_START` **不是**绿灯豁免：每次复跑重判；接线被删/审计转红立即翻 FAIL。

---

## 2. OpenDots —— SSRF 防御的**单点化**（本节结论最可执行）

### 2.1 事实（F2，源码 `src/browser/security.ts`，59 行 + `src/shared/computer-types.ts`，106 行）

OpenDots 的每个 agent 有「自己的 computer」（browser），因此要挡 SSRF。
`validateUrl()` 是**六层纵深**，每层都可独立判定：

1. 协议白名单：仅 `http:` / `https:`
2. ⭐ **URL 内禁带凭证**（`url.username || url.password`）⇒ 挡 `http://user:pass@host`
3. ⭐ **端口白名单**：仅 `80` / `443` ⇒ 挡内网服务在非常规端口
4. 主机名字符串检查：`localhost` / `*.localhost` / `*.local`
5. ⭐⭐ `dns.lookup(hostname, {all:true})` ⇒ **每一个**解析出的地址都必须是公网
   （`addresses.some(a => !isPublicAddress(a.address))` 即拒）
6. ⭐⭐ `isPublicAddress()` 的网段表**完整**：
   - IPv4：`0/8`、`10/8`、`127/8`、`≥224`、`100.64/10`(CGNAT)、`169.254/16`(link-local)、
     `172.16/12`、`192.168/16`、`192.0/24`、`192.0.2/24`(TEST-NET-1)、`198.18/15`、
     `198.51/24`、`203.0/24`
   - IPv6：`2001:0::/32`(Teredo)、`2001:db8::/32`(文档)、`2001:10::/28`(ORCHID)、
     `2001:20::/28`(ORCHIDv2)、`2002::/16`(6to4)，且只接受 `/^[23][0-9a-f]{3}:/`（2000::/3 全局单播）
   - ⭐⭐ **非 IP 输入 ⇒ `return false`** ⇒ **fail-closed**（未知形态默认不可信）

⭐⭐ 最值得抄的一条：**fail-closed 默认值**。安全判定函数对「不认识的东西」必须返回「不可信」，
而不是「大概是公网吧」。

### 2.2 ⭐⭐⭐ 本仓的**真实缺口**（实测，非推测）

⛔ **不是**「NeoTrix 没有 SSRF 防护」。实测：**有，而且相当好** ——
`nt_memory_crawl/mod.rs` 有 **5 条专用 SSRF 测试**（loopback / private+reserved /
ipv4-mapped-IPv6 / bad scheme+unparseable / allows-public），
且注释里记录了「`::ffff:127.0.0.1` 曾绕过旧守卫（`is_loopback` 只匹配 `::1`）」这类**实战教训**。

✅ **真正的缺口是「同一份防护被复制在多处」** —— `169.254.169.254` 出现在 **9 处 / 8 个文件**：

| 文件 | 命中数 |
|---|---|
| `l4_emotion/nt_memory/nt_memory_kb/nt_memory_crawl/mod.rs` | 7 |
| `l4_emotion/nt_memory/nt_memory_kb/nt_memory_crawl/nt_http.rs` | 5 |
| `l3_embodiment/nt_shield/nt_shield_audit/mod.rs` | 3 |
| `l1_action/nt_io/nt_io_browser_engine/mod.rs` | 3 |
| `l3_embodiment/nt_shield/shield_core/http_proxy.rs` | 2 |
| `l3_embodiment/nt_shield/nt_shield_stealth_net/network_monitor.rs` | 2 |
| `l3_embodiment/nt_shield/nt_shield_agentic_scan.rs` | 1 |
| `l1_action/nt_io/nt_io_browser_engine/policy.rs` | 1 |

⇒ **漂移风险**：新 fetch 点忘了加，或旧点漏了一个网段，**没有任何门会发现**。
⇒ ⭐ OpenDots 的对策就是 `security.ts`：**一个 59 行的单点，全层共用**。

### 2.3 建议动作（未实施，按优先级）

1. ⭐⭐ 把 SSRF 判定**抽成 L0 substrate 的单一 canonical validator**
   （`nt_core_net_guard` 之类），语义照 F2 六层 + **fail-closed**。
2. ⭐ 加门 `check-net-guard.sh`：断言 `ureq::get|post` / `reqwest` / `TcpStream::connect`
   的调用点**全部**经该 validator ⇒ 防止第 9 处副本出现。
3. ⛔ **不**把 8 处副本一次性替换（结构改动，须先 `cargo clean && cargo build` 跑两遍）。
   先加门（可红可绿地暴露真实缺口），再逐处收敛。

---

## 3. Claude Code mods（⛔ 非一手，仅设计思路）

### 3.1 ⭐ 纠正一条流传说法

新闻/博客普遍说 mods「**不设沙箱**、只装可信来源」。这个说法**不完整**。
据社区逆向 loader 实测，mods 受到三重限制：

- ⛔ **拒绝** `node:fs`、`node:child_process`、一切 `node_modules` 包 ⇒ 直接拒绝整个模块
- ⛔ **`$`（宿主能力句柄）不能被存起来或传给 helper 藏起来**；加载前的静态扫描会
  穿透 helper 追 `$`（`claude plugin validate` 输出 `hooks:` / `calls:` 两行清单）
- ⭐ matcher 类型写错（`{command: 5}`）或空 matcher（`{tool: []}`）
  ⇒ **加载期就拒绝整个模块**，而非静默不生效

⇒ 准确表述：**无进程隔离（OS 级同权限），但语言级能力面窄且被静态扫描成可复核清单。**

### 3.2 最值得抄的 4 条

| # | 做法 | 对 NeoTrix 的意义 |
|---|---|---|
| 1 | **加载期产出「能力清单」**（`calls:` 行） | ⭐ 我们的 `CapabilityNode.provides/requires`（实测 17 处）**已是**结构化词汇表，缺的正是「这个扩展**实际调了什么**」这份可复核产物 |
| 2 | ⭐⭐ **内建功能自己就是 mod**（`/diff`、AGENTS.md loader、telemetry） | **自举防腐**：自家内建能力都走扩展点 ⇒ 扩展点无法腐烂。NeoTrix 的 `skills/` 只是**给模型看的指令**，**没有任何能力走扩展点** |
| 3 | `sec-default` 先加载、**压住用户 mod**（不许覆盖 permission deny） | 扩展点之上要有一层**策略钳制**，不指望每个扩展自守 |
| 4 | `$.state`（会话级、扛热重载）vs `$.store`（插件私有 JSON、跨会话）分离 | 生命周期分层，可映射到我们 KB 的 `experience` namespace |

### 3.3 ⭐ 与 NeoTrix 的对比结论（诚实版）

| 维度 | NeoTrix | Claude Code mods |
|---|---|---|
| 扩展形态 | `skills/`（markdown 指令） | TS/JS 代码 |
| 代码级事件拦截 | ⛔ 未找到 hook/event 总线 | ✅ before/after/instead/wrap |
| UI 扩展 | ⛔ 无 | ✅ |
| **能力形式化词汇表** | ✅ **有**（`provides`/`requires`） | ⛔ 无结构化词汇表 |
| **能力清单产物** | ⛔ 无 | ✅ `calls:` |
| **反虚标** | ✅ **`maturity_audit()`** | ⛔ 无 |

⭐ 我们在「能力的形式化与反虚标」上更强。
⭐⭐ 而「运行期可被扩展」这一侧 —— ⛔ **本节原写「完全空白」，是错的**，见 §3.4 订正。

### 3.4 ⛔⛔ 订正：「完全空白」是错的 —— 扩展点**已建成且从未接线**

2026-10-03 晚，同一 session 内自查发现（实测，非推测）：

`neotrix-core/src/l5_cognition/nt_core_dispatch.rs`（**354 行**）是一个**完整的
事件拦截扩展点**，其文件头自述：

> 事件派发调度器 —— 吸收自 **deepseek-harness `vendor/cordis/src/events.ts`**
> （4+1 dispatch modes: emit / waterfall / parallel / serial）
> 机制: `Waterfall` 链式中间件: 每个 handler 可调 `next()` 委托给下一环
> (around middleware), 或返回 `true` 短路
> ⭐ **NeoTrix 消费方 (R-P79)**: McpServer 工具调用 pre/post 钩子 (Waterfall 中间件链)

⇒ ⭐⭐ **这就是 Claude Code mods 的同一种东西**（事件拦截 + around 中间件 + 可短路），
**我们早就有了，而且是从 cordis 吸收来的**。我此前说「未找到 hook/event 总线」
⇒ **那句是错的**（我当时只 grep 了 `l1_action` 与 `handlers_consciousness` 两个目录）。

**核实「R-P79 声明」**（`AGENTS.md` §4.4：「导出 ≠ 接入」；且历史上已错过 3 次）：

| 核实项 | 结果 |
|---|---|
| McpServer 存在？ | ✅ 真实（`l1_action/nt_io/nt_io_mcp_bridge.rs` 等 5+ 文件） |
| 其中有谁用 `Dispatcher` / `Waterfall`？ | ⛔ **零**（唯一同时命中两者的文件是 `nt_core_dispatch.rs` 自己的文档注释） |
| `Dispatcher::new()` 构造点 | **仅 4 处，全部在** `l0_substrate/nt_core_event_bus.rs` |

⇒ ⛔ **那 354 行 waterfall 扩展机制的真实消费者为零**，
而文件头**声称**已接入 McpServer 并标注 R-P79。
⭐ 这正是 `AGENTS.md` §4.4 点名的失效形态：
**「门记录声称已做而实现从未入库」** —— ⭐ 而这次连门都没有，是注释里的声明。

### 3.5 ⭐⭐ 本日最重要的模式发现：**瓶颈不是「没建」，是「没接」**

今天在两个子系统上撞见**同一个形状**：

| 子系统 | 规模 | 状态 |
|---|---|---|
| `nt-core-capability-tree`（能力树） | 节点模型 + registry + 成熟度 + 反虚标 + CLI + CI 门 | **零运行期消费者** → 今天 `ee2cc276` + `ae3b3644` 接线 |
| `nt_core_dispatch`（waterfall 扩展点） | **354 行**，cordis 吸收，4 种派发模式 | ⛔ **零真实消费者**（R-P79 声明为假） |

### 3.6 ⛔ 订正订正：`54a2fa64` 的 commit message **又高估了一档**

我写「McpBridge 兑现了 R-P79，354 行资产从零消费者到有」。⭐ **只对了一半**，
严格核实（本日第 3 次实测同一形态，**且这次是我自己的提交**）：

| 核实项 | 结果 |
|---|---|
| `Dispatcher` 在 L0 的用法 | ⭐ **一直是活的**：`EventBus::register_hook`（`nt_core_event_bus.rs:101`）+ `emit_from:113` 真的 `dispatch_waterfall`。生产调用方 = `l5_cognition/nt_mind/nt_mind_background_loop/run.rs:398` 的 `register_hook(flood_guard(500ms))` |
| ⛔ `McpBridge::on_tool_call` 的调用方 | **只有我刚写的 3 条测试，零生产注册** |

⇒ ⭐⭐ 两处订正：
1. ⛔ 「`nt_core_dispatch` 真实消费者为零」**本来就错** —— L0 的 `register_hook`
   + `run.rs:398` 的 flood_guard **是真实生产接线**。
   我之前只数了「文件级引用」，⭐ **没区分「持有/分发」与「注册了真钩子」**。
2. ⛔ 我说「McpBridge 兑现 R-P79」也高估：`tool_hooks` 现在**被持有且被分发**
   （真实生产代码路径），但**没有任何生产代码注册工具钩子**
   ⇒ **拦截能力未被使用**。

### 3.7 ⭐⭐⭐ 于是模式升级：**「建成未用」在每一层都成立，包括我自己的提交**

| 层次 | 资产 | 状态 |
|---|---|---|
| 模块级 | `nt-core-capability-tree` | 今天已接（`ee2cc276`+`ae3b3644`） |
| 文件级 | `nt_core_dispatch`（354 行） | ⛔ 我误判为零消费者；实为 L0 + `run.rs:398` **已在用** |
| **API 级** | ⭐ `McpBridge::on_tool_call`（我今天新增） | ⛔ **零生产注册，只有测试** |

⇒ ⭐⭐⭐ **最强的证据：连我自己的贡献也立刻复现了同一个失效形态。**
⇒ 由此得到一条**比「接线」更准**的判据，必须分三级看，缺一级就会自欺：

| 判据级 | 问的问题 | `on_tool_call` 现状 |
|---|---|---|
| L1 声明级 | 文件/模块有引用吗？ | ✅ 有 |
| L2 分发级 | 生产代码路径会**跑**它吗？ | ✅ 有（`call_local_tool` 开头必过 waterfall） |
| ⭐ **L3 使用级** | ⭐ **有生产代码真的注册了它吗？** | ⛔ **没有 ⇒ 能力未被使用** |

⭐ **今天我两次用 L1 级证据就宣称「已接线」** ⇒ 这个三级判据是本日最该沉淀的一条。

⇒ ⭐⭐ **结论：NeoTrix 的涌现瓶颈不在「缺引擎」，在「接线」。**
⇒ 这比任何单点设计都更该指导优先级：
**不要再去建第三套机制；先把已建的两套接进生产，并给「接线」加门。**
⇒ ⚠️ 可直接复用的判据（今天已两次实证）：
**凡看到「吸收自 X」+「消费方 (R-P79)」的注释，一律先核实消费者计数，再谈它已落地。**

---

## 4. 本轮共同教训（⭐ 三次同款命名陷阱）

1. ⛔ **Cargo 包名分隔符**：`[[package]] name` 用**连字符**，依赖键用**下划线**。
   ⛔ 一天内踩了**两次**（`ee2cc276` 的 grep、`neobot-check-emergence.mjs` 的 `cargo run`）
   ⇒ 已两次实测确认这是**常驻风险**。查依赖必须两种分隔符都查。
2. ⛔ **变异标记不能包含被测子串**：负向测试里 `unwired_register_node(` 仍含
   `register_node` ⇒ 变异「看着生效、实际没变」⇒ 假通过。
3. ⭐ **负向测试必须跑同一份逻辑**：自测重写一遍正则只能证明「第二个正则也能删」，
   **证明不了门本身会红** ⇒ 必须抽纯函数供正负两路共用。

⇒ 三条共同指向 `AGENTS.md` §5 **R-SCAN-1b**：裸 grep 的**命中与零命中都不构成证据**，
必须回到**文件本身**读那一行。
---

## 5. 论文：Decoding Looped Transformers Better for (Almost) Free

> **一手取证**：`webfetch https://arxiv.org/abs/2610.02185` 成功。
> arXiv:2610.02185（cs.LG），2026-10-01 提交，8 位作者（Weihao Liu, Huangjie Zheng,
> Tianrong Chen, Rohit Dilip, Richard He Bai, Yizhu Jiao, Yuyang Wang, Ruixiang Zhang），
> 32 页 19 图。⛔ 本节仅读 **abstract + 索引页**，⛔ **未读正文/图表**
> ⇒ 下面只写 abstract **明确陈述**的内容，以及**我标注为类比**的部分。

### 5.1 论文实际说了什么（仅 abstract 可证的）

- **Looped Transformer**：把**同一个 block** 在 recurrent loop 里**反复执行**以省参数。
- 每个 loop 都产出**可解码为同一 next token** 的中间表示，⛔ 但**标准解码把早期状态全丢掉**。
- ⭐ **关键洞察**：**更早的 loop 蕴含更少的计算** ⇒ 递归**天然**提供
  **对齐的「弱-强」预测对**（aligned weak-and-strong prediction pairs），
  **不需要辅助模型、不需要外部训练**。
- **LoopCD**（**training-free** 对比解码）：
  | 变体 | 空间 | 额外开销 |
  |---|---|---|
  | `LoopCD-Logits` | logit | **一次额外 output pass** |
  | `LoopCD-Hidden` | hidden-state | **零 output 开销** |
- 结果（4 个 looped Transformer 家族）：
  - Ouro-2.6B-Thinking AIME 2024 pass@1 **61.88% → 73.33%**（Logits）
  - Huginn HumanEval pass@1 **22.56% → 31.71%**（Hidden）
- ⭐⭐ **最重要的结果不是精度，而是**：这些增益**允许把 recurrent loop 数减半**
  同时**持平或超过全深度无引导基线** ⇒ **forward FLOPs 降 22.5%–48.2%**。

### 5.2 ⭐⭐ 与 NeoTrix 能力树的结构同构（**这是我的类比，不是论文的主张**）

| LoopCD | NeoTrix `nt-core-capability-tree` |
|---|---|
| 更早的 loop（**计算更少**）⇒ 弱预测 | 更低的 `constellation` 档（C0/C1 vs C4/C5） |
| 最终 loop（计算更多）⇒ 强预测 | `maturity_audit()` 依据证据给出的 `supported` |
| **对比弱 vs 强**来选 token | **对比 `claimed` vs `supported`** 来判虚标 |
| training-free、**不需辅助模型** | ⭐ 审计同样是纯规则、**无外部模型** |
| 「aligned」= 同一目标 token | ⭐ 同一 `node.id`、同一 `provides` 标签 |
| ⛔ 早期状态被**丢弃** | ⛔ `ConsciousnessRuntime` 的中间 `AwakeningReport`/`EmotionReport` 被丢弃 |

⭐⭐⭐ **同一个可迁移的方法论内核**：
> **你本来就要丢弃的中间计算，可以零成本地当作「弱监督」去校准强信号。**

### 5.3 ⭐ 可执行的研究方向（**假设，未验证**）

论文最值得抄的**不是精度数字，而是那句「减半 loop 仍持平」** ——
即**用便宜的中间证据避免昂贵的完整验证**。映射到 NeoTrix：

> **能否用已登记的低档证据（C0/C1，廉价）作为对比信号，
> 避免每次都跑昂贵的 C3Benchmark / C4MainPipeline 审计？**

⚠️ **必须先证伪的三点**（否则这就是又一次「手推当实证」）：
1. ⛔ 我们的 C0 证据与 C4 证据**是否真的「对齐」**（同一 node、同一 `provides`）？
   若不对齐，对比无意义（这正是 LoopCD 里 "aligned" 一词的实质约束）。
2. ⛔ 弱-强对比**是否会引入假虚标**（把「证据尚未登记」印成「已查清不够格」）
   ⇒ 这正是 `Epistemic` 轴存在的理由 ⇒ ⭐ **新方向必须复用 `Epistemic`，不得另造语义**。
3. ⛔ 是否存在「弱证据一致但强证据不一致」的节点？
   若普遍存在 ⇒ 弱监督只能**筛可疑**，不能**定罪**。

⛔ **本节不产出任何代码改动。** 按 `LESSONS-20260929-checked-is-not-verified.md`
的教训：qybaihe/mu 的回测显示其 admission/chunk 准入**成本最高（54% token）、
收益为零**（2412 块 drop 0 个）⇒ ⭐ **「听起来对味的机制」在本仓已被验证过一次是伪命题**。
⇒ 上表 5.3 必须先做「三点证伪」，再谈实现。
