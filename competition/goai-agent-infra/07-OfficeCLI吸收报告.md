# OfficeCLI 吸收报告 — 世界观构建方法论五阶段

> 吸收对象：[iOfficeAI/OfficeCLI](https://github.com/iOfficeAI/OfficeCLI) v1.0.144
> 吸收时间：2026-08-14 · 本 session 完成接线（R-P79）
> 证据：本机实测二进制 v1.0.144、`view stats/outline/screenshot/html`、`plugins`、`mcp` stdio 握手、PDF 19 页导出（Chrome headless 渲染链路）

---

## 一句话

**OfficeCLI 是给 AI agent 用的 Office 套件 CLI**：单二进制（C#/.NET 自包含 ~34MB）直接读改写 `.docx/.xlsx/.pptx`（OpenXML），内置高保真 HTML/PNG/PDF 渲染引擎 + 内置 MCP server + 向各 AI 客户端自动安装 SKILL.md。不需要 WPS/Office/LibreOffice。

---

## Phase 1 资料收集（证据清单）

| Source | Pattern (机制) | 证据 |
|---|---|---|
| README | 单二进制、Apache 2.0、28.3K stars、自包含渲染引擎 | api.github.com + 本机下载运行 v1.0.144 |
| 命令参考 | DOM 路径寻址（`/slide[1]/shape[@id=N]`）、XPath 1-based 索引 | `officecli help`、command-reference wiki |
| 命令清单 | create/open/close/view/get/query/set/add/remove/move/raw/validate/batch/dump/refresh/watch/mark/merge/mcp/skills/install | `officecli help` 实测 |
| 渲染引擎 | `view html/svg/screenshot/pdf`，per-page PNG，KaTeX 公式、Three.js 3D、morph 动画 | 本机实测 `screenshot --page N` 输出 1280x720 PNG |
| MCP | `officecli mcp` stdio server，注册目标 lms/claude/cursor/vscode | 本机实测 initialize/tools-list 握手 OK |
| Skills | `officecli skills install <target>`，pptx/docx/xlsx/morph-ppt/pitch-deck/academic-paper/data-dashboard/financial-model 8 项专精 | command-skills wiki |
| Plugin | exporter 插件支持 `.pdf`/`.doc`/`.hwpx`，`OFFICECLI_PLUGIN_<KIND>_<EXT>` 环境变量寻址 | plugins/plugin-protocol.md §2.2/§3/§8 |
| 命令 | `merge` `{{key}}` 模板合并、`dump`→`batch` round-trip、`batch` 原子多步 | command-reference wiki |

---

## Phase 2 世界观公理（先立公理，再推演）

| 公理 | 内容 | 对应 NeoTrix 公理 | 违反后果 |
|---|---|---|---|
| **世界运转** | Office 文档 = OpenXML 标准 DOM 树，一切操作以路径寻址（`set/add/remove/query`），不依赖 WPS/Office/LibreOffice | **Dark Forest**（无外部运行时依赖，二进制即插即用） | 若依赖特定 Office 安装，跨环境不可复现，违背"单二进制零依赖"承诺 |
| **核心稀缺** | AI agent 的上下文与"看得见"能力稀缺——文档必须输出结构化 JSON + HTML/PNG 视觉反馈，而非猜测 DOM | **指针守恒/惰性加载**（结构化、按需、可回放，不倾倒全文） | 若只给 agent 原始 XML，推理成本爆炸且无法自我校验版式 |
| **金手指** | 内置高保真渲染引擎 + 内置 MCP server + 向 11 种 AI 客户端自动装 SKILL.md，让 agent 直接"看渲染结果→改→再验证"闭环 | **R-P79 接线门**（能力必须被 agent 消费，非摆设） | 若无 MCP/skill 通道，CLI 只是又一个一次性脚本，无法被 AgentTeams Worker 调用 |

---

## Phase 3 力量体系（境界阶梯 + 代价）

| 境界 | 晋级条件 | 代价 | 能力表现 |
|---|---|---|---|
| C0 编译 | dotnet build 自包含二进制 | .NET 编译期依赖 | 单文件可直接运行（实测 v1.0.144） |
| C1 单测 | schema-crc 指纹 + dump→batch round-trip | 属性面需字节级稳定 | `dump` 可序列化再 `batch` 回放 |
| C2 集成 | `validate` OpenXML schema 校验 + 渲染引擎 | 渲染精度需持续校准 | `view html/screenshot` 1280x720 PNG 实测 |
| C3 benchmark | 高保真渲染引擎（公式/图表/3D/morph/水印） | 引擎复杂度高（C# + 前端） | agent 可"看"输出，无需真 Office |
| C4 流水线 | 内置 MCP server + skills install 到 11 客户端 | 版本演进需兼容多种客户端 | `officecli mcp` stdio 握手 OK |
| C5 自愈 | plugin 生态（exporter/format-handler/dump-reader） | 插件协议需版本化维护 | `.pdf/.doc/.hwpx` 由插件扩展 |

---

## Phase 4 一致性维护

| 设定 | 版本 | 前后矛盾? | 处置 |
|---|---|---|---|
| 命令面 | v1.0.144 schema-crc 指纹 | 否（升级字节级不变） | 记录指纹，升级前复检 |
| 插件协议 | plugins/plugin-protocol.md §3 发现顺序 first-match-wins | 否 | env 变量寻址，避免多插件冲突 |
| PDF 导出 | 需 exporter 插件；官方 registry `officecli.ai` 522 宕机 | 是（官方通道不可用） | 本次用 `view html + Chrome headless --print-to-pdf` 兜底，产出 19 页 PDF 已实测 |

---

## Phase 5 工业化生产（接线点 + 门禁 + 节奏）

| 阶段 | 存稿/缓存 | 大纲层级 | 门禁 | 接线点 |
|---|---|---|---|---|
| 本 session | 二进制落 `~/.local/bin/officecli` v1.0.144 | 大赛初赛交付 | PDF 19 页 / PNG 19 张已产出 | 初赛方案 PPTX+PDF 生成链路 |
| 复赛 Demo | officecli 以 MCP server 接入 AgentTeams Worker | A3 实施者文档能力 | 大赛 MCP 推荐项 + 第三方依赖披露 | `mcp-gateway` 工具注册 `officecli` |
| 远期 | 官方 exporter 插件（registry 恢复后 `plugins install officecli-pdf`） | 原生 PDF 导出 | 校验 SHA-256 | 替代 html+Chrome 兜底链路 |

---

## 输出契约

**一句话**: 它是 AI-agent 原生的 Office 读改写套件（单二进制 + 内置渲染 + MCP + skill 分发）。

**三条公理**: 世界运转=OpenXML DOM 路径寻址零依赖；核心稀缺=结构化 JSON + 视觉反馈；金手指=内置渲染引擎 + MCP/skill 通道。

**力量体系**: C0-C6（编译→round-trip→schema 校验→渲染→MCP 流水线→plugin 自愈）。

**一致性**: v1.0.144 schema-crc 指纹稳定；PDF 官方插件通道宕机已用 Chrome headless 兜底验证。

**生产**: 已接线——初赛 PPTX+PDF 交付完成；复赛以 `officecli mcp` 接入 mcp-gateway 供 A3 生成 .docx/.pptx 报告。

**行动**: **HIGH**。纳入复赛 MCP 工具清单（替代/并列 gawirable office-mcp，理由：单二进制零依赖 + 内置 MCP + agent 渲染可视化，均本机实测）。

---

## 与现有方案的关系（R-P42 强化现有节点）

| 现有文档 | 处置 |
|---|---|
| `06-办公套件读写方案.md` | 更新推荐落地方案：OfficeCLI 升为 P0 首选（本机实测），office-mcp 降为备选 |
| 附录 B Skill 清单 | mcp-gateway Skill 增加 `officecli` 工具注册项（复赛接线） |
| 方案 PPT | 初赛不涉及，无需改动 |