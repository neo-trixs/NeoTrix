# 进化核心建议落盘 — 2026-08-15

> 来源: 「Rust 多智能体自审计系统设计文档」深度分析 + 12 个外部技术/仓库调研。
> 目标: 把设计文档的确定性工具分层 + 外部最佳实践落地为 NeoTrix 真实演进,
> 消灭死代码 (R-P79), 统一工具层 (GAP-4), 强化自愈闭环。

## 决策摘要

| # | 类别 | 决策 | 状态 |
|---|------|------|------|
| P0-1 | 消灭死代码 | `neotrix_code_graph` MCP 工具经 `/code-graph` CLI 命令真实接线 `CodeGraphMCP` | ✅ 已落地 |
| P0-2 | 消灭死代码 | `MetaAuditor` 从"仅测试调用"升级为架构审计真实消费端 | ✅ 已落地 |
| P1-3 | 统一工具层 | `BuildRunner` 统一 cargo L1/L2/L3 分层 (超时+kill+证据+Denylist) | ✅ 已落地 |
| P1-4 | 自愈闭环 | `apply_auto_fixable` 从"只计数"升级为真实事务落地 | ✅ 已落地 |
| P2-7 | 错误自愈 | 错误码 `FixSuggestion` 增加 `suggestion` + `valid_range` (OfficeCLI 模式) | ✅ 已落地 |
| P2-5 | Skill 质量门 | 确定性 selftest 门 (shuohao-skills 模式) | 📋 待下周期 |
| P2-6 | 进化注入 | Verbalized Sampling 注入 SEAL 吸收 (arXiv 2510.01171) | 📋 待下周期 |

## 前置审计发现 (证据化)

盘点既有审计/自愈/进化基础设施后确认 4 个真实缺口:

- **GAP-1 — CodeGraphMCP 死代码**: `nt_core_retrieval.rs` 的 `CodeGraphMCP`
  提供 4 个确定性检索工具, 但 `neotrix_code_graph` MCP 定义 (nt_agent_mcp_tools.rs L29-37)
  声明 `McpTransport::Local { command: "neotrix", args: ["code-graph"] }`,
  而 CLI 无 `code-graph` 子命令 → 调用即回退注册表失败, 生产从未触达。
- **GAP-2 — MetaAuditor 空转**: `nt_core_meta_auditor.rs` 的 `record_finding`
  仅在测试中调用; 后台 `handle_architecture_audit` 的 converge_check/SelfTest
  结果只打日志, 未汇入任何审计器。
- **GAP-3 — apply_auto_fixable 只计数不落地**: `HealerRegistry::apply_auto_fixable`
  统计 `auto_fixable` 计数即返回, 从不执行真实修复。
- **GAP-4 — 无统一 build_runner**: cargo 调用散落 8+ 处
  (behavioral_verifier/self_audit d42/safe_applier/nt_shield::audit/self_model/
  AutoFixer/goal_contract/evolution_loop), 无统一超时/证据/门禁。

## P0-1: code-graph CLI 接线 (已完成)

**文件**: `neotrix-core/src/cli/commands/code_graph_cmds.rs` (+ mod.rs / registry.rs 注册)

- 新增 `CodeGraphCmd` (`/code-graph`, 别名 `/cg` `/graph`), 解析
  `search_symbols|file_stats|graph_topology|get_node` + `--query`/`--root`。
- 直接构造 `CodeGraphMCP::new()` → `build(root)` → 调确定性工具 → 返回 JSON 细节。
- 生产调用链: MCP `tools/call` → Local transport 子进程 `neotrix code-graph` →
  main.rs clap InvalidSubcommand 回退 → `/code-graph` 注册表命令 → CodeGraphMCP。
- **R-P79 满足**: CodeGraphMCP 公开方法被非测试代码调用, 输出可影响行为。
- **T3 满足**: MCP def 存在 (L29-37) + 命令注册 + 真实执行。
- 注意: `CodeGraphMCP::build` 对大目录 (如 neotrix-core/src) 全量索引较慢
  (>60s), 后续可在 MCP 层加缓存/增量 (Obsidian rune 预留)。

**验证**: 4 单测通过; CLI 实测 `neotrix code-graph search_symbols --query foo --root <tiny>` → `1 hits`。

## P0-2: MetaAuditor 消费端 (已完成)

**文件**: `run.rs` (持久字段) + `handlers_consciousness.rs` (消费接线)

- `BackgroundLoopHandle` 新增 `meta_auditor: MetaAuditor` 持久字段 (克隆注册模式,
  与 `tool_grounding` 一致)。
- `handle_architecture_audit` 内:
  - converge_check `findings` 全部汇入 `record_finding` (severity 映射 Error 0.9/
    Warning 0.6/Info 0.3);
  - SelfTest `run_all()` 失败项汇入 (category `selftest_failure`, severity 0.8);
  - 写回持久实例 + 注册副本进 registry → accuracy 随时间真实累积。
- **R-P79 满足**: record_finding 被生产代码 (3600s 架构审计 tick) 调用。

## P1-3: BuildRunner 统一 cargo 层 (已完成)

**文件**: `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind_build_runner.rs`

- `BuildLayer` (L1-fast / L2-audit / L3-heavy) 映射设计文档工具分层:
  - L1: `check` `clippy` `fmt` `test` `tree` `metadata` (默认 ≤180s)
  - L2: `audit` `deny` `outdated`
  - L3: `llvm-cov` `miri` `expand` (≥600s)
- `BuildEvidence` 结构化证据: exit/timed_out/error_count/warning_count/stdout/stderr/
  duration_ms → `success()` 供 R-P9/R-P16 双验证。
- **Denylist gate (fail-closed)**: `publish` `install` `vendor` `clean` `uninstall` 阻断;
  未知工具拒绝 (Deterministic Tools First)。
- 超时 kill 复用 behavioral_verifier::run_bounded 模式 (reader 线程 + mpsc + kill)。
- 生产接线: `self_audit_cmds.rs` d42 裸 `Command::new("cargo")` 替换为 `BuildRunner`。

**验证**: 5 单测通过 (含真实 cargo check 证据收集)。

## P1-4: apply_auto_fixable 真实落地 (已完成)

**文件**: `nt_mind_autofixer.rs` + `handlers_maintenance.rs`

- `scan_todos` 检测"纯占位 TODO 行" (// TODO / // FIXME 无正文) → 标记 `auto_fixable: true`。
- `apply_auto_fixable` 重写: 对 auto_fixable 建议执行 `AutoFixer::cleanup_todos_tx`
  (∂Γ 事务: 快照 + recover 回滚), 成功者从 `last_report` 移除 (防重复),
  遥测 `auto_fixes_applied` 只计真实落地。
- `handle_healer_scan` 扫描后立即 `apply_auto_fixable()` → 巡检从"报告"升级为"自愈闭环"。
- 安全: 只删纯占位注释行, 不动含正文 TODO (保留人工待办语义)。

## P2-7: 错误码自愈模板 (已完成)

**文件**: `nt_core_error_parse.rs`

- `FixSuggestion` 增加 `suggestion: Option<String>` + `valid_range: Option<String>`
  (OfficeCLI 模式: 错误码 = 可自愈的操作单元)。
- 8 类错误码全部补齐可执行指令 + 适用范围声明:
  E0425/E0433/E0412 (use 补全), E0308 (as/into 对齐), E0382/E0505 (clone),
  E0004 (match 分支), E0063 (补字段), E0599 (方法/impl), E0428 (去重),
  dead_code/unused_* (cargo fix)。

**验证**: 17 单测通过 (新增 officecli_suggestion_and_range / cleanup_cmd 两测)。

## P2-5 (待下周期): Skill 确定性 selftest 门

shuohao-skills 模式: 每个 skill 强制 `SKILL.md` + `scripts/selftest.mjs`
(10-16 道确定性质量门, 零 LLM, 可离线复现)。映射到 NeoTrix:

- 每个 L2 skill 目录要求 `scripts/selftest.sh` 或 `selftest.js`, 产出结构化 JSON
  (gates[]: name/pass/criteria), 失败即拒收。
- `SkillsEngine::init` 校验存在性, 缺 selftest 的 skill 标记 `unverified` 而非静默加载。
- 目标: 技能吸收从"信任声明"变"可验证证据", 与 R-P16 同构。

## P2-6 (待下周期): Verbalized Sampling 注入 SEAL 吸收

arXiv 2510.01171 (Verbalized Sampling, mode collapse 缓解):

- **发现**: 采样多样性不足源于典型性偏差 (生成模式坍缩); VS 提示模型
  "言化其概率分布" (对候选响应估计概率并输出) 而非贪婪解码。
- **收益**: diversity +1.6–2.1x, training-free, 越强模型收益越大。
- **映射**: SEAL 蒸馏阶段 (Distiller) 吸收候选时, 要求模型对候选输出
  言化分布 (理由+置信) 后再蒸馏 → 缓解蒸馏同质化, 提升进化多样性。
- 参考 OpenRouter 2026 announcements 的 Response Sampling 方向, 与
  "The Spice Must Flow" 数据管道公理一致。

## 设计文档 → 代码映射表

| 设计文档概念 | 落地 | 状态 |
|---|---|---|
| Deterministic Tools First + LLM Second | BuildRunner (确定性) + agent 编排 | ✅ BuildRunner |
| 工具分层 L1 check/clippy/fmt/test/tree/metadata | BuildLayer::Fast | ✅ |
| 工具分层 L2 audit/deny/outdated | BuildLayer::Audit | ✅ |
| 工具分层 L3 llvm-cov/miri/expand | BuildLayer::Heavy | ✅ |
| 自动修复白名单 + 沙箱回滚 | AutoFixer 白名单原语 + RepairBatch 事务 | ✅ (既有) |
| Critical/High 仅报告 | SelfTest registry 只记录不落地 | 既有 |
| RustOrchestrator 3 轮上限 | goal_loop / dispatch 已有 | 既有 |

## 遗留 / 下周期

1. **P2-5** skill selftest 门 (SkillEngine 校验 + CLI 报告)。
2. **P2-6** Verbalized Sampling 接入 Distiller 蒸馏候选生成。
3. CodeGraphMCP 增量索引 (缓存 large-tree build, Obsidian rune)。
4. 吸收流程: 本周期 5 项落地 + 2 项设计 → `experience-tree` 五阶段吸收。

---

*生成: 2026-08-15 · 决策已获用户批准 (「所有」) · 待执行体验吸收 (五阶段)*
