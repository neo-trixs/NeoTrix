# NeoTrix Absorption Round 12: 余项扫尾

> **Absorption Date**: 2026-09-22 | **SIM**: SIM-46 | **Status**: Landed
> **Upstream**: R11 deferred（Exegol＋A–G 余项），6 读封顶（5 实读＋Exegol 双失败折叠）。
> **This round: 0 新条款**（NT-STD 停 1.0.8，连续两轮决议）。

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-77b | Exegol 3.1k★ | — | 两次传输失败（R11＋R12），永久折叠入库，不判内容 |
| S-80 | recurse 170★ Tauri 2＋Rust (Engine trait：后端即选择非依赖；agent 工具＋UI 后端无关＋canonical 结果；grounded 引用防幻觉；verification＞generation；懒分析；headless core 独立构建＋eval 独立二进制——cargo test 永不花钱；SQLite＋FTS5 记忆；malware-safe local-first；无反编译器优雅降级) | RE IDE | D-2/EQ-17/F03/D-3 注（金） |
| S-81 | awesome-ai-security-tools 1.5k★ (类型图例 公开/研究/商业/限制；skill 扫描器矩阵 SkillSpector 14.7k/Ramparts/aguara/skilltotal 离线确定性；MCP pinning；memory guard 防投毒；flight recorder 哈希链 JSONL＋Merkle；凭据占位 onecli/Earl；honeyslop 反幻觉报告；AISVS/asamm/ATR 基准族) | Panorama | SEAL 采购单 |
| S-82 | Open-LLM-VTuber 13.9k★ (离线可跑；模块配置切换无代码；Agent 接口抽象；内心独白透明；proactive speaking；chat 持久化；mic 安全上下文注；v2 重写纪律——v1 不接新功能) | Companion | L3/L4 注 |
| S-83 | Scrapling 82.9k★ (adaptive 重定位；MCP  stripping  prompt-injection 后再给 AI；无 LLM 的 RAG Markdown；spider checkpoint 续爬＋AutoThrottle 自适应退避＋robots 遵守＋dev 缓存重放；AI_POLICY.md；MCP＋skill 双带) | Scraper | V/E13/R8 注 |
| S-84 | multica 51k★ Multica License (issue→agent→run→review；intent/run/decision/diff 钉 issue；执行回放＋token 成本；review gate 落 review 非 main；技能即 playbook；runtime 即本机代码不出境；26 CLI 驱动不自带；make dev worktree 感知) | Workspace | SDB-C/工作区注＋license 警告 |

Rejected: Exegol 内容（未读不判）；云 judge；新 SaaS；H 类（维持折叠）。

---

## 2. Pattern → NeoTrix Mapping（全 bake note）

- Engine trait（S-80）→ SIM-43 D-2 金佐证：后端即选择，agent/UI 只吃 canonical 类型；
  r2 备选"自带不捆绑"即 NTS 依赖纪律形态。
- Eval 独立二进制（S-80）→ EQ-17 成本纪律：eval-run 是 binary 非 test，
  cargo test 永不花钱——fuzz/toxics/AgentHarm 首跑照此形（防 CI 账单爆炸）。
- Grounded 引用＋verification＞generation（S-80）→ E12/Confidence 注：
  地址是可点击对象非粘贴文本；人一键确认/否决即 grader。
- 懒分析＋优雅降级（S-80）→ F03/D-3 注：按需解码；缺能力报 graceful error 其余照常。
- SEAL 采购单（S-81）：skill 扫描 SkillSpector/aguara/skilltotal（offline 确定性优先）；
  MCP pinning（mcp-guardian）；memory guard（OWASP）；flight recorder（哈希链 JSONL→SDB 审计）；
  凭据占位（onecli/Earl→codex-router 卫生）；honeyslop（反幻觉 triage）。
  类型图例（公开/研究/商业/限制）收为外部引入分级法。
- 具身注（S-82）：模块配置切换（ledger 精神）；内心独白透明（F08）；v2 重写纪律
  （v1 只修 bug——与"结构项递延非跳过"同构）。
- 抓取注（S-83）：MCP 先 strip 注入再给 AI（V 前置过滤形态）；AutoThrottle 自适应退避
  （E13）；checkpoint 续爬（R8 durability）；adaptive 重定位须有验证（G10 门）。
- 工作区注（S-84）：issue 钉 intent/run/diff（证据绑定工作，OpenResearch 共振）；
  review gate（SDB C）；26 CLI 驱动不自带（ledger 精神）；worktree 感知（并行公约共振）。
  **License 警告**：Multica License 附加条件≠纯 Apache-2.0，借鉴模式不引代码。

---

## 3. Landings

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R12-1 | SIM-46 record | row + §45 present |
| L-R12-2 | BLUEPRINT v1.6.11 changelog | line present |
| L-R12-3 | EQ-17 成本纪律注（§2 eval 段） | rows referenced |

NT-STD 停 1.0.8（三轮连续决议：R10 用 1／R11 用 0／R12 用 0）。

Deferred: H 类（维持折叠）；R13（A–G 残余＋新输入；边际已薄，开轮需新 P0 出现）。

---

*End of Absorption Round 12*
