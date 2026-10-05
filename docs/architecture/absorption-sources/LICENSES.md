# 吸收源许可台账（2026-09-28）

> ⛔ **抄码前必查本表。** 三类：可抄 / 只读设计 / 不可用。
>
> **核实方式与可信度**：
> - ✅ `LICENSE` 直读 = 高可信（绕开 API 限流）
> - ✅ 研究期 GitHub API `license.spdx_id` = 高可信
> - ⚠️ README 自述 = 中可信（以仓库声明为准）
> - ❌ **未核实 = 不写**（本表不含任何未核实条目）

---

## ⛔ 不可用（禁止抄码）

| 仓 | 许可 | 依据 | 可信度 |
|---|---|---|---|
| `tdeverx/contained-app` | **PolyForm Noncommercial 1.0.0** | README: *"source-available and free for non-commercial use under the PolyForm Noncommercial License 1.0.0"* | ⚠️ README 自述 |
| `multica-ai/multica` | 自定义（禁托管服务，类 BSL） | 条款: *"you may not… provide a hosted service to third parties… **even when it is offered free of charge**"* | ⚠️ LICENSE 直读 |
| `volcengine/OpenViking` | **AGPL-3.0** | 研究期 API 核实 | ✅ |
| `vectorize-io/agent-memory-benchmark` | **无 LICENSE 文件** | 根目录无 LICENSE，`pyproject.toml` 无 `license` 字段 | ✅ 研究期逐项检查 |
| `digipulse-engineering/GAAI-framework` | **ELv2**（非 OSI，API 返回 `NOASSERTION`） | 禁作托管服务。仅读「双轨进程隔离」设计 | ✅ API `NOASSERTION` |

**⚠️ 补充**：`tdeverx/contained-app` 经研究期实测**无 XPC / MCP / URL scheme**（全仓 grep 零命中）—— 即使许可允许，它也**零 agent 面**，没有可吸收的东西。

## ⚠️ 只读设计（可引 idea，不可抄码/散文）

| 仓 | 许可 | 限制 | 可信度 |
|---|---|---|---|
| `lopopolo/harness-engineering` | **CC-BY-4.0** | **散文需署名**。只引 `evals/README.md` 的协议结构，勿抄进源码 | ✅ API |
| `microsoft/autogen` | **CC-BY-4.0**（API 核实，**非 MIT**） | 且已**维护模式**（README: *"New users should start with Microsoft Agent Framework"*）。勿指向 `autogen-agentchat` | ✅ API |
| `PrimeIntellect-ai/prime-agent` | MIT | ⚠️ LICENSE 头写 `Copyright (c) 2025 Mario Zechner`（继承自 `pi` 基座）—— vendor 前查清归属 | ✅ |
| `letta-ai/letta` | — | ⚠️ **`main` 只有 10 个文件、零代码**（README 自述真码已迁 `letta-code`）。24.9k 星衡量的是 MemGPT V1 历史 | ⚠️ README 自述 |
| `trailofbits/skills` | **CC-BY-SA-4.0** | **只取思想（rust-review/insecure-defaults/fp-check 等审计纪律），不抄代码不抄文本**；相同署名-相同方式共享，沾上即传染 | ⚠️ 页面徽标自述 |

## 🟡 Copyleft（**条件可取**，须先满足本档三条硬条件）

> **为什么需要这一档（2026-10-05 立）**
> 原表只有「⛔ 不可用」与「✅ 可抄」两档。
> 而 **GPL-3.0 落在两档之间的空白区**：它既不是宽松许可（不能进「可抄」），
> 也不是「完全禁止」（**只取设计是允许的**）。
> ⇒ 空白区的危害是**每个 agent 会各自猜**：
> 有的当宽松放行（违法），有的当禁止浪费可用的设计。
> （本次触发源：`rizinorg/cutter`，19,878★，实测 GPL-3.0。）

### 三条硬条件（**全部**满足才可取码）

1. **取的是「独立可执行体」而非链接库**
   GPL 文末原文：*"The GNU General Public License does not permit
   incorporating your program into proprietary programs."*
   ⇒ 取代码必须**整体独立**（独立进程/独立二进制），**不得**静态或动态链接进
   NeoTrix 主体，否则整个作品须 GPL。

2. **不触发「整个作品」传染**
   GPL §5(c)：*"You must license the entire work, as a whole, under this
   License to anyone who gets a copy."*
   ⇒ 只要与 NeoTrix 同一分发物，就必须整体 GPL ⇒ **实际不可行**。

3. **分发时附 Corresponding Source + 挂 Appropriate Legal Notices**
   ⇒ 我方无「分发」场景（自用），故本档现实结论通常是**只取设计**。

### 本档现实裁决（2026-10-05）

| 仓 | 许可 | 裁决 | 依据 |
|---|---|---|---|
| `rizinorg/cutter` | **GPL-3.0** | 🟡 **只取设计，不取码** | 条件 2 不满足（C++/Qt 逆向平台，与我方主体同分发物）。⭐ **取证曲折**：`dev/LICENSE` 返回 **404**（默认分支是 `dev` 非 `main`，且文件名不叫 `LICENSE`），实际文件是 **`COPYING`**（35,148 字节）。**只看 `LICENSE` 会误判成「无许可证」** ⇒ 文件名不是判据，内容才是 |

⚠️ **GPL vs AGPL 的实质区别（别混为一谈）**
· GPL：**分发**时传染（我方不分发 ⇒ 不触发）
· AGPL：**提供网络服务**时传染（我方是长期运营的服务 ⇒ **必然触发**）
⇒ 这就是 `AFK-surf/Comma`（AGPL）进「⛔ 不可用」而 cutter 进本档的原因。

⚠️ **无 LICENSE 与 NC 类的区别（2026-10-05 新增教训）**
· **无 LICENSE** = 默认全权保留 ⇒ 你什么都拿不到 ⇒ **安全地失败**
· **PolyForm NC / CC-BY-NC** = 「看起来能拿，拿到就违法」⇒ **更危险**
  实测：`GitNexus`（47,728★）= PolyForm Noncommercial 1.0.0；
  `htd-ai-augmented-education`（268★）= CC BY-NC 4.0。
  ⇒ API 返回 `NOASSERTION` 时**必须读原文**，不许按「无许可」处理。

## ✅ 可抄（已核实）

| 许可 | 仓 |
|---|---|
| **MIT** | `NousResearch/hermes-agent` · `kunchenguid/backpass` · `PrimeIntellect-ai/prime-agent` · `rlaope/oh-my-hermes` · `orwa-mahmoud/nightshift` · `lidge-jun/opencodex` · `FoundationAgents/MetaGPT` · `HKUDS/nanobot` · `bytedance/deer-flow` · `BAAI-Agents/Cradle` · `langchain-ai/langgraph` · `crewAIInc/crewAI` · `oil-oil/oil-ui`（LICENSE 文件直读 ✅） · `vitali87/code-graph-rag` · `CodeGraphContext/CodeGraphContext` · `vercel-labs/agent-skills`（三者页面徽标自述 ⚠️，抄码前需 LICENSE 直读复核） |
| **Apache-2.0** | `loopx-project/loopx` · `mvschwarz/openrig` · `simular-ai/Agent-S` · `nanobrowser/nanobrowser` · `xlang-ai/OSWorld` |

**⚠️ 上表只列已用 `LICENSE` 文件直读或研究期 API 核实过的。** 其余仓的许可**未逐一核实** ——
抄码前请自行跑：

```bash
curl -s "https://raw.githubusercontent.com/<owner>/<repo>/HEAD/LICENSE" | head -4
```

---

## 📌 数字陷阱（勿照抄）

| 陷阱 | 实测真相 |
|---|---|
| `mem0ai/mem0` 的 92.5 | README 自述: *"Scores reflect Mem0's **managed platform, which includes proprietary optimizations not available in the open-source SDK**"*。规模自降 LoCoMo 92.5 → BEAM(10M) **48.6**。**不可作为 OSS 可达数字引用** |
| `Hindsight` 的 `access_count` | 迁移文件自承 **从初版 schema 起就是死的，每行都是 0** ⇒ 按访问次数衰减**从未上线** |
| `hev/auv` 系 ruflo 73k★ | README 主打改名 + 付费层 + 工具数。**星数不是机制证据** |
| trendshift yearly 的 `gained` | ≈ 总星数（20/25 仓 r≈1.0）⇒ **不是周期增长，比例无意义**。仅 monthly 板有相对增长，且只有 `r ∈ [0.83, 0.99]` 段可信 |
| Jev/Laya 星群 | 17 仓跨 6 榜同品牌播种，**14/17 是 astroturf** ⇒ 取架构，**忽略生态** |
| arXiv API | 2026-09-28 实测 **API 被封**（对照测试 `1706.03762` 也返回空）⇒ 只能读 abs 页面。这是环境问题，**不是论文不存在** |

---

## 📌 本表的一处自纠

初版此表**含 3 个编造条目**（`neobot-absorption` 相关仓 / `mokia21/agent-browser` /
`ByteDance-Seed/UI-TARS-desktop`）—— 那些是研究结论里的**名字片段**，我当成了真实 `owner/repo`。
已删除，并把 `microsoft/autogen` 从「MIT」更正为 **CC-BY-4.0**（API 核实），
`huggingface/smolagents` 从「MIT」更正为 **Apache-2.0**（API 核实）。

**⇒ 教训**：写许可台账时，**每个 `owner/repo` 都必须来自已抓取的数据文件**，
不能凭印象补全 —— 与「导出 ≠ 调用」同源。

## 2026-10-01 新增（用户提交，吸收判定见 [ABSORPTION-2026-10-01-CHUNUI-GROWTH-PI.md](../ABSORPTION-2026-10-01-CHUNUI-GROWTH-PI.md)）

| 仓库 | SPDX | 可吸收性 |
|---|---|---|
| `liseami/ChunUI` | **MIT** | ⛔ 代码不可（SwiftUI/iOS）；✅ 硬约束定档写法 → 已落地为 `neobot-check-typography.mjs` |
| `GetBrew/growth-engineer` | **MIT** | ⛔ 域内容不可（GTM/销售）；✅ 1 条构建基础设施模式 → 已落地为 `nt_build_lock.sh` |
| `earendil-works/pi` | **MIT** | ⛔ 代码不可（agent harness）；✅ 供应链预防手段 → 部分落地（仓库根 `.npmrc`） |

三者均为 MIT ⇒ 可自由取用思路与代码片段（**实际未复制任何代码**）。

## 2026-10-04 第四批

| 仓库 | SPDX | 备注 |
|---|---|---|
| `rehan-remade/universal-modder` | **MIT** | ✅ 宽松 |
| `allenai/olmo-core` | **Apache-2.0** | ⛔ 有**专利授权**条款 |
| `Niko1221/Strata` | **MIT** | ✅ 已于 `fc5a1107` 吸收，不重复登记 |

统一索引：`ABSORPTION-2026-10-04-OLMOCORE-MODDER.md`

## 2026-10-03 第三批：用户提交 12 源中需单列的两条

| 仓库 | SPDX | 备注 |
|---|---|---|
| `moguzbulbul/blueprint-animation` | **CC-BY-NC-4.0** | ⛔ **非商用**。常被误当 CC BY而漏掉 NC 条款；我方是否商用未确认 ⇒ 确认前不可取用 |
| `markfulton/agent-cookie-sync` | **MIT** | ✅ 宽松 |
| `Untrivial-ai/agent-orchestrator` | **Apache-2.0** | ⛔ 有**专利授权**条款，判据不同于 MIT |
| `awesomedata/awesome-public-datasets` | ⛔ **未核实** | 未抓取（链接目录，且前置门第①问不通过）⇒ 不填未经核实的值 |

统一索引：`ABSORPTION-INDEX-user-urls-2026-10-03.md`

## 2026-10-03 新增（第二批）

| 仓库 | SPDX | 可吸收性 |
|---|---|---|
| `Niko1221/Strata` | **MIT** | ✅ 可吸收（MoE 跨层级卸载 / 投机解码 / MCP 自管理） |
| `yetone/magpie` | **MIT** | ✅ 可吸收（一网关四协议 / 凭据单一收口 / 历史不按今天配置回填） |
| `yetone/cumora` | **MIT** | ✅ 可吸收（seen-cursor 新鲜度闸 / 默认 fail-closed 沙箱） |

判据见 [ABSORPTION-2026-10-03-STRATA-MAGPIE-CUMORA.md](../ABSORPTION-2026-10-03-STRATA-MAGPIE-CUMORA.md)。

## 2026-10-03 新增（第一批）

| 仓库 | SPDX | 可吸收性 |
|---|---|---|
| `lightpanda-io/browser` | **AGPL-3.0** | ⛔ **代码不可吸收**（最强传染性 copyleft，链接即需整体开源）；✅ 4 条设计可移植（CDP 契约 / PandaScript 零 token 执行 / MCP 会话隔离 / robots 开关） |
| `open-slide/open-slide` | **MIT** | ✅ 可吸收；✅ 2 条设计（skill 携带硬规则 / 约束画布而非内容） |

判据见 [ABSORPTION-2026-10-01-CHUNUI-GROWTH-PI.md](../ABSORPTION-2026-10-01-CHUNUI-GROWTH-PI.md) 末节。

### ⚠️ 2026-10-03 复核：上表「4 条设计可移植」的落地率

复核方法：`rg` **按代码概念**查（不是按台账里的措辞查），结论如下。

| 声称可移植的设计 | 是否真落地 | 证据 |
|---|---|---|
| robots 开关 | ✅ 已落地 | `nt_io_browser_engine/engine/nt_politeness.rs:42-45` 真实解析并阻断 |
| CDP 契约（能力声明） | ✅ **早已落地** | `nt_io_browser_engine/types.rs:87-127` `BackendCaps{javascript,screenshot,form_submit,cookie_persist}` + `BackendKind::caps()`。⚠️ **上一轮我记为「未落地」是搜错了字符串**（搜 `CDP契约` 而非 `BackendCaps`），已更正 |
| MCP 会话隔离 | ❌ 未落地 | 无对应实现 |
| PandaScript 零 token 执行 | ❌ 未落地 | `rg -i PandaScript` 命中 0 |

⛔ **上一轮「1/4」的结论本身是错的**（因搜索词照抄台账措辞）。
教训：**核实落地率必须搜代码里的概念名，不能搜台账里的标签名** ——
台账用自己的话命名，代码用另一套话命名，按标签搜必然 0 命中，
然后就会得出「文档说 ✅ 但代码没有」的错误结论。**这本身又是一次
「台账/标签不可信，代码才是真相」。**

### ✅ 2026-10-03 新增落地：`nt_selector_contract.rs`

lightpanda 的「声明能力面」补到了**它没有的一层**：
`BackendCaps` 声明的是「后端能不能执行 JS」，而 X 抽取真正会坏的是
「JS 里的选择器还能不能命中」—— 后者没有任何机制守护。

新增 `social_access/nt_selector_contract.rs`：
- `X_SELECTOR_CONTRACT` 声明每个选择器的**页面形态**与**是否 essential**；
- `verify()` / `check()` 在运行时核对，漂移即报 `Parse` 错误；
- ⭐ 顺序语义：**先判形态再验契约**（未登录页天然 0 推文，
  反序会把「请登录」误报成「X 改版了」）；
- 已接进 `nt_x_browser::parse()`，修掉了该函数原先**无条件返回空列表**的假成功。

⇒ 顺带吸收 open-slide（MIT）「约束画布而非内容」：
不给「抽取结果」加内容校验，而是约束**抽取面**，
用探针让越界在结构上不可能。

### ⚠️ 2026-10-03 实测：lightpanda 的能力边界（不可替代 chromiumoxide）

本机已装 `lightpanda`（`~/.local/bin/lightpanda`），实测结论：

| 项 | 实测结果 |
|---|---|
| CDP 服务 | ✅ 起得来，`/json/version` 正常，`Target.*` 可用 |
| `Runtime.evaluate` | ✅ 可用（`example.com` 取到 `document.title`） |
| `fetch --dump markdown` | ✅ 可用，输出干净正文 |
| **`Page.navigate` 实测** | ⛔ `BrowserContextNotLoaded`（-31998）—— 直接 navigate 失败 |
| **抓 x.com（无 cookie）** | ⛔ **只拿到登录页**（实测输出全是 Log in / Sign up） |
| `--load-resources` 默认 | ⛔ **不加载** image/iframe/worker/stylesheet |
| `Emulation.setAutomationDisabled` | ⛔ 源码零引用（研究已证） |
| 指纹一致性 | ⛔ `navigator.product="Gecko"` 与 Chrome UA 不自洽（反检测反噬） |
| License | **AGPL-3.0** vs 本仓 **MIT** ⇒ 代码层不可引入 |

⇒ 结论：**它是高吞吐正文抽取器，不是 stealth 浏览器**。
`nt_io/universal_browser.rs` 依赖的 `enable_stealth_mode`（= 设置
`Emulation.setAutomationDisabled` 一类）它没有 ⇒ **不能作为
`chromiumoxide` 的后端替换**。AGPL 单独即可否决代码引入。

### ⚠️ 2026-10-03 澄清：`open-slide/open-slide` ≠ `openslide/openslide`

若按「openslide」去找数字病理库，会**找错仓库**：

| | `open-slide/open-slide` | `openslide/openslide` |
|---|---|---|
| 是什么 | React 幻灯片框架 | C 库，读虚拟切片图像（数字病理） |
| 语言 | TypeScript | C |
| ★ | **8,718** | 518 |
| License | MIT | **LGPL-2.1**（不是 MIT） |

前者 star 数是后者 17 倍，**任何「搜 openslide」或模型回忆都会先命中它**。
非恶意抢注，只是无关项目在搜索可见性上胜出。⚠️ 注意真实 openslide 是
**LGPL-2.1**，此前若按 MIT 记录需更正。

⛔ **未核实故未吸收**：`KKKKhazix/AIHOT`（许可证未核）。
⛔ **未逐仓核验**：`trendshift.io` 榜单 —— 榜单条目本身**不等于**已核实的许可/可移植性。

⚠️ **本仓 vendored 第三方代码的许可另记**：`apps/neobot-desktop/frontend/`
是 dsh-harness-desktop 0.19.1（**MIT**，vendored，含 2 处本地改动，详见其 `VENDOR.md`）。
⛔ 它**不是**冗余副本 —— 不得按「重复文件」删除，且其字号等规范**不适用**本仓排版门。

## 2026-10-05 第五批：游戏引擎侧两源（吸收判定见 [ABSORPTION-MIU2D-RA2-2026-10-05.md](../ABSORPTION-MIU2D-RA2-2026-10-05.md)）

| 仓库 | SPDX | 备注 |
|---|---|---|
| `luckyyyyy/miu2d` | **MIT** | ✅ 宽松。⚠️ **游戏资源/IP 不在许可内**（属西山居）⇒ 只取引擎源码思路 |
| `rust-alert/ra2.exe` | **Apache-2.0** | ⛔ 含**专利授权**条款。⚠️ 同上，《红色警戒》IP 属 Westwood ⇒ 只取源码 |

⛔ **同组织另 7 仓本轮一律「只读设计」，不得抄码**：

| 仓库 | SPDX | 处置 |
|---|---|---|
| `rust-alert/ra2-remixer` / `ra2-tools` / `ra3.exe` / `rs-ddraw` / `YurisHook` / `factorio.exe` | **MPL-2.0** | ⚠️ **文件级 copyleft** —— 抄进 MIT/Apache 文件会污染本仓许可 ⇒ 只取设计 |
| `rust-alert/homm3.exe` | **CC0-1.0** | ✅ 公共领域贡献，本可抄；本轮未取 |
| `rust-alert/rgss.exe` / `terraria.exe` / `hl.exe` | **无 LICENSE** | ⛔ 只取设计（沿用 `ABSORPTION-AGENT-ARCH2-2026-09-29.md`「无 LICENSE ⇒ 只取设计」先例） |
| `rust-alert/vxl-renderer` / `FontsForRedAlert2` / `RA2YR-reMIXer` / `relert.js-browser` | — | ⛔ **皆为 fork**，且多数无 LICENSE ⇒ 只取设计 |
