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

⭐ **我们在「能力的形式化与反虚标」上更强，在「运行期可被扩展」上完全空白。**
而后者正是「涌现」缺的另一半 —— ⭐ **能力树能长节点，但长出来的能力没法被就地接入系统。**
⇒ 这是 §3.2 第 2 条（自举防腐）成为下一步首选的原因：**先让一个内建能力走扩展点**，
扩展点才第一次被真实使用，也才第一次可能腐烂。

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