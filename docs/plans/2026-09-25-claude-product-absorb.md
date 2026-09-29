# muse.ai / Claude 产品吸收 → neobot 补完（2026-09-25）

> 来源：muse.ai 直连被 bot 墙挡回（400），转攻 claude.com 全站（可读）+
> platform.claude.com 模型文档 + 独立计价器交叉。 livestock 引用全部来自本次抓取。
> MIT 许可的是代码；此处只吸收产品思想与公开价目数字。

## 1. 抄到的产品地图（2026-09 实况）

- 产品：Claude（chat）/ Claude Code（+ artifacts，并行 agents 桌面）/
  Claude Cowork（与 chat 合一、内置浏览器、插件）/ @Claude
- 功能：Claude in Chrome（默认 Haiku 可切）/ M365（含 Excel）/ Skills /
  Design·Science·Security 三垂直 App / Marketplace·Connectors·Plugins /
  Managed Agents（内置记忆、沙箱、MCP 隧道、多智能体编排）/
  Claude Tag（CI 急救）/ Cowork 企业部署（ spend 管控）
- 模型：Mythos / Fable / Opus / Sonnet / Haiku（1M 上下文、128K 输出、
  adaptive thinking）；计划：Free $0 / Pro $17–20 / Max $100–200

## 2. 本次落地（neobot）

| # | Claude 侧 | neobot 落点 | 说明 |
|---|---|---|---|
| 1 | 官方价目（Sonnet5 $3/$15 标准价 9-01 起；Opus 5/4.8/4.7/4.6 $5/$25；Haiku4.5 $1/$5；Fable/Mythos 5 $10/$50） | `nt_cost::CLAUDE_PRICES` + `price_for` | 优先级：运维配置 > 官方表 > 引擎自带；未知仍归零（永不猜测律保留） |
| 2 | 内置记忆（跨会话） | `nt_memory`（MEMORY.md 8K 上限，去重，密钥行拒写）+ `memory set/get/clear` + HTTP system 注入 + IPC | OpenMuse personal context 同构 |
| 3 | Skills（SKILL.md frontmatter） | `nt_skills` 解析 frontmatter（无 yaml 依赖手写）+ 回落旧 `#标题` | Claude 技能包可直接下沉安装 |
| 4 | in-Chrome 按任务切模型 | 已有模型切换器对应，不动 | 口径一致 |
| 5 | Cowork 插件/连接器/MCP | 不做（neobot 无 MCP 客户端，违本地独立保证即引入外部依赖） | 记录在案 |
| 6 | Artifacts 侧栏 | 不做（md-lite 代码块已覆盖行内；独立产物面板远期） | 记录在案 |
| 7 | Projects 分组 | convos 已是 project-lite，不做新实体 | 记录在案 |

来源：
- https://platform.claude.com/docs/en/models/sonnet-5/overview（$2/$10）
- https://platform.claude.com/docs/en/models/fable-5-1/overview.md（$10/$50）
- https://platform.claude.com/docs/en/models/mythos-5-1/overview（$10/$50，Glasswing 邀约制）
- https://pricepertoken.com/compare/provider/anthropic-vs-openai（全系列表）
- https://kunavo.com/guides/claude-api-pricing-2026（9-01 标准价口径）
- https://support.claude.com/en/articles/12138966（Fable/Sonnet 发布线 + Excel/Chrome 动态）
- https://claude.com/product/cowork（Cowork 定义与定价）
- http://claude.com/（Free/Pro/Max 三档）

## 3. 验证

- `cargo test -p neotrix-neobot --lib`：56 passed（含价目/记忆/frontmatter 新单测）
- CLI：`memory set/get/clear` 去重/拒密钥/二次确认全过
- 直连 api.anthropic.com 未收录为预设：其原生接口非 OpenAI 兼容，
  本引擎只说 OpenAI 协议（文档注记，不撒谎）
