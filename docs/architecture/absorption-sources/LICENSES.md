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

## ✅ 可抄（已核实）

| 许可 | 仓 |
|---|---|
| **MIT** | `NousResearch/hermes-agent` · `kunchenguid/backpass` · `PrimeIntellect-ai/prime-agent` · `rlaope/oh-my-hermes` · `orwa-mahmoud/nightshift` · `lidge-jun/opencodex` · `FoundationAgents/MetaGPT` · `HKUDS/nanobot` · `bytedance/deer-flow` · `BAAI-Agents/Cradle` · `langchain-ai/langgraph` · `crewAIInc/crewAI` |
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
