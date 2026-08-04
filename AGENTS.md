# NeoTrix — AI-Native Developer Toolkit

NeoTrix is an AI-native developer toolkit with self-evolving reasoning, knowledge representation via VSA HyperCube, and Global Workspace Theory attention routing.

**Preamble**: This session loads `CONTEXT.md` (root) as the shared language prefix. All domain terms are defined there. Before using any domain term, refer to CONTEXT.md for its precise definition and avoid column.

**统一吸收协议 (MUST)**: 每次会话结束必须自动执行 `experience-tree` 五阶段吸收 (快照→蒸馏→分类→落盘→反馈)。经验统一写入 `~/.neotrix/knowledge.db` 的 `kv_store` `experience` 命名空间; 本文件仅保留 cycle 摘要指针。执行: 汇总会话经验 → `~/.neotrix/pending-absorb.json` → 插件 `session.idle` 自动调 `~/.agents/skills/experience-tree/scripts/absorb_session.py absorb` + `close --cycle NNN`。协议详见 `~/.agents/skills/experience-tree/SKILL.md`。

**指针守恒 (HARD RULE)**: AGENTS.md 是**纯指针文档**，永久禁止追加以下内容到本文件：cycle 完整正文、Session 明细表、Build Baseline 明细、元认知发现清单、吸收细节。所有经验正文统一经 `experience-tree` 流程落盘 KB，本文件仅允许：新增/更新 Experience Index 一行指针、修订操作规则 (Dev Rules/审查维度/共享语言) 本身。新内容超过 3 行 → 必须走 KB 吸收流程。违反即回滚。

**写入门禁 (MECHANISM)**: 经验指针与全文统一存 KB `experience` namespace hub，**禁止在 AGENTS.md 内联经验表或 cycle 正文**（手工追加会被门禁拒绝）。指针检索唯一路径：`python3 ~/.agents/skills/experience-tree/scripts/absorb_session.py hub` 查看 cycle 索引 / `query --kw` 检索全文。AGENTS.md 结构受双门禁保护：opencode 插件 `.opencode/plugins/agents-guard.js` 在 `session.idle` 校验本文件行数与节结构 + git pre-commit hook 拒绝超阈提交。规则由机制执行，不依赖 agent 自律。

**外部文件惰性加载 (LAZY LOAD)**: 本文件是 L1 常驻层，只含最高信号内容。以下文件**不要预加载**，遇到相关任务时用 Read 按需读取，加载后为强制规则：
- `@dev-rules.md` — 全量 R-P1-R-P80 (编码/构建/审查/吸收纪律)。处理编码、构建、审查、吸收任务时加载
- `~/.agents/skills/rev/officer/rev-officer-agent.md` — D1-D51+S1-S7 审查维度。触发 review 时加载
- `~/.agents/skills/experience-tree/SKILL.md` — 吸收协议。会话收尾时加载
- KB 检索: `absorb_session.py query` — 历史经验全文，需要时按关键词查询

## Experience Index

Cycle 指针与全文统一存 KB: `~/.neotrix/knowledge.db` `experience` 命名空间 hub。查看: `absorb_session.py hub`（索引） / `query --kw`（全文）。禁止内联经验表。

| Cycle | Domain | Summary |
|-------|--------|---------|
| 163 | NT-CORE | Frontier model research: Kimi K3, Fable 5, DeepSeek V4-Pro, Qwen3.7/3.8, Gemini 3.6 Flash, Grok 4.5 + HuggingFace datasets/models mapped to NeoTrix consciousness core evolution plan |
| 185 | NT-CORE | Iter44 audit: VSA bundle encoding-agnostic x!=0 voting + Kanerva count*2>=n (all-zero collapse), FHRR FPE scalar + seed split from GHRR, GWT workspace index-space, IIT phi off-diagonal, gold-standard coherence bounds, entropy normalization, per-tick activation reset |
| 186 | NT-CORE | 周天星系大阵断链修复: ConsciousnessRuntime 接入 KB (attach_kb/query_kb/tick 注入, 带 provenance) + 背景循环 surface 到 GWT + 好奇心 HyperCubeBridge 空桥改真实皮层数据 + MemoryProvider/RichMemoryProvider 死抽象实现到 KnowledgeBase/ReasoningBank |
| 186 | NT-IO | 竞品差距批: 空状态 quick-action chips + 单代码块复制 (decorateCodeBlocks) + 滚动吸底按钮 + slash 命令集 8→12 (/btw /new /model /init); Playwright workers=2+retries=1 治 dev-server 竞争超时; CSS module 块截断陷阱; 真实环境测试栈 (tauri-driver 不支持 Playwright/WKWebView) |
| 187 | NT-IO | Evolve 批: 能力网健康面板 (7域意识树健康网格, 消费 health_report) + 主题 accent 系统 (6色 swatch) + 竞品12产品 TOP-10 差距研究 + Evolve B1-B7 阻塞项 (独立验证器/受限自治/激活预算/中途持久化/域熔断/自愈复核/公证); 修 absorb_session query 排序崩溃 + /tmp 碰撞 + spec 花括号陷阱 |
| 187 | NT-CORE | Frontier model fusion wired to E8: nt_core_synthesis.rs (AttnRes/compressed_attention, V4 fused_distribution, Gemini 3.6 StepRouteCache, effort tiers) → engine_core.reason() + telemetry; defects fixed: attention tail-mass, birkhoff Sinkhorn double-count, matrix_distribution cap 0.5→0.8, engine_core tm_vec u64→f64, negentropy sensor_attention_focus caller; gold_standard 8-test regression reconciled to Iter45 phi semantics (±1 cluster fixture, >0.4); KB test order-pollution confirmed |
| 188 | NT-CORE | Muon/Safety 修复 (Newton-Schulz Frobenius 归一化防 NaN, risk 归一化 /1.2) + 前沿模型知识落库 (Kimi K3/DeepSeek V4/Fable 5/Qwen3.8/Gemini 3.6/Grok 4.5 → KB seed → hypercube SystemDesign 轴); seal_loop.init_attention_router 触发 seed_foundational; frontier seed 全链路测试通过 |
| 189 | NT-CORE | Iter45 周天星系大阵运转链路审计轮 (13 HIGH/12 MEDIUM/10 LOW): H3 hypercube_bridge 多标签坐标塌缩聚合; M-8 geometry_sync min_info_partition 丢 current_state; HIGH-2 iit_phi 均衡饱和 → centered + intensity×rho (23维均衡0/差异化0.58); FHRR permute % → rem_euclid(TAU); M-9 negentropy 硬编码假传感器 → 真实 state+0.5降级; L-4 graph_memory 重复 id 误驱逐 LRU; 提交 89d58dd (6 文件); H3 文件因并发 ingest_from_kb WIP 留待并发合并; HIGH-1 Track2 死代码删除: cognitive_tick+cognitive_orchestrator 1124行平行孤儿 (R-P42/R-P79) → 5c24375; 共享 mod.rs 用 staged 纯删除+working 保留并发行隔离 |
| 190 | NT-CORE | Phase 7.2 InnerSpeech (MIRROR §3.3): inner_speech.rs 自我对话 (广播→独白→context 写回, 有界 ring=32, 自我问答 cadence); GlobalWorkspace 接线 + engine_core GWT tick telemetry; 11 测试 + GWT 全量 316 通过 |
| 191 | NT-CORE | Phase 7.5+7.3+7.4: ModalityRouter (5-modal softmax q·k router + REINFORCE, Step 4c) / CLS_Buffer (ring100 episodic + hybrid 检索, Step 4d) / CTM_Verifier (5 公理 finite-state/finite-action/globality/δ/bounded-tape, Step 4e) 接线 GlobalWorkspace; GWT 337 通过, 全 lib 6810 (仅 2 预存 DNS 失败); 教训: REINFORCE 勿 unit-normalize, verify 用 in-flight report 非 last_resonance, ReasoningHexagram pub 构造测 invalid state |
| 192 | NT-CORE | Phase 8.1 CognitiveType (MiCRo): 4 认知类型 (Linguistic/Logical/Knowledge/Social) classify + group_activation 聚合 + CognitiveProfile softmax/dominant/entropy, 接线 resonant_broadcast Step 4f |
| 193 | NT-CORE | C5-P0-05 McpRegistry::call_tool stub→真转发 (transport 映射 + mcp_call_tool, 5 测试); 并发审计法: grep/Read 矛盾=并发实时改写, ground truth=git status; TODO.yml 多数已落地 |
| 193 | NT-CORE | Phase 10 统一潜在推理 (iter193): 10.1 UnifiedLatentSpace (JL 式 SeededProjection 跨域 cosine, E8/GWT/VSA→256-d) / 10.2 LatentReasoningPipeline (E8→latent→NN→GWT 直发无文本, episodic 256 + outcome 加权注意力) / 10.3 MultimodalEncoder (text n-gram hash kernel + image/audio → unified, ModalityRouter 融合→E8 mode); 全接线 engine_core + telemetry; 23 新测试, GWT 370 |
| 194 | NT-CORE | EA3-P0-01 MCP 吸收管线闭环 (ToolOrchestrator 真实现 + as_native_tools + entry 双接线 + /mcp list absorbed) + rev-officer 审计 15 项修 3 (/review 路由真引擎删 stub, Span gen_ai setter 真记录, 删 3 假模块); 6888 测试过 (2 预存 DNS); 遗留: evolution_daemon 硬编码 stub, HookRegistry 空壳, DelegateEngine 假成功 |
| 196 | NT-CORE | 推理控制面 F1-F6 落地 + 接线: F1 budget 注入闭环 / F2 去 debug / F3 Vllm+Sglang provider / F4c learn_from_trace 分层校准 / F5 R2-Bench eval_harness 接线 (/bench eval 子命令 + factory provider) / F6 MERA control_distillation 接线 (learn_from_trace→distill_trace→distilled_sequences 状态接地); 新增 R-P86 (Option min/max 非幂等,map_or 显式) + R-P87 (测试夹具勿撞检测器 marker); F5/F6 同 session 接线消除 R-P79 门, 测试 228 全过 |
| 197 | NT-CORE | 20 外部仓库吸收批 (source=absorption, 18 条目): cybers襄rike/CyberStrike (3-gate 确认协议+7656 Ed25519 技能, NT-SHIELD) · reverse-skill (路由+授权+证据链纪律) · PentAGI (监督三层/refSeeder → NT 的 Evovle B3-B6 阻塞项) · open-code-review 确定性×agent 混合评审 (token 1/9, 升级 rev-officer) · crm 证据定价+FOR UPDATE SKIP LOCKED 队列 (NT-ACT) · loopx quota/gate/evidence 控制面 (opencode 兼容) · qm scope 多租户隔离 · grok-build Rust crate 划分 · spec-kit SDD 工作流 · autoresearch 薄路由 95% token 降 · jakub/oklch/interfaces make-it (同心圆角+光学对齐, NT-IO) · ai-knowledge-graph SPO → NT-MEMORY 摄入 · 其余 MEDIUM/LOW; 警示: absorb 误用已占用 cycle 号会污染 hub, 需按 session_id 甄别真源 |
| 198 | NT-CORE | 第一性原理评测 CLI 冗余: 自主动循环(周天大阵)需观测面不需指挥面, 干预cmd=伪需求冗余; 删除 brain_cmds.rs 未注册 EvolveCmd 死壳 (R-P42/R-P79) + registry 5测试过; 立规则: 命令三分类(指挥→daemon自驱冗余 / 观测→CLI必要 / 配置→CLI交互), 能力网单一主权原则(daemon调度/CLI观测/KB存态, 无主权方特性不应存在); holon 可选git依赖是预存环境问题非代码缺陷 |
| 199 | NT-CORE | F6 蒸馏训练闭环接地: successor closed-loop (分布蒸馏→SFT+CSPO→E8策略更新→影响后续reason), sing strategy authority R-P42(temp trainer from prm.policy→write-back), 修复 ControlTrainer apply_* 空壳真调 learn_from_scores(步骤按 step_idx 稠密对齐供 steps.get 位置索引) + reason() 主流程接线 learn_from_trace(R-P79死链修复); 清理孤儿 nt_core_delegate_engine.rs + xueshu_login.js(从未运行/仓库走Rust CDP), install.sh Homebrew 公式路径断裂修复 git mv → deploy/homebrew; 语言评测: 41万行Rust主核+平台强约束脚本层(.rb DSL/.js, Playwright/.py 数理)无需迁移, 真正遗留是路径断裂非语言; 并行 HookRegistry(agent.hooks vs nt_mind_hook) 真技术债留独立 rev-officer session |
| 199 | NT-CORE | (本期 hub 已有并发会话 cycle=199 条目: 证据门控自治 DelegateEngine DomainGate/证据强度反哺 crm-loopx-PentAGI; 增量构建缓存幻影错误) |
| 200 | NT-CORE | KB 全域审查 (rev-officer): 修 D52-1 hub 幽灵索引 (absorb 增量漂移→_refresh_hub_metrics 从实际分支全量重建, 幂等自愈) + D52-2 cycle int/str 混用 (统一 str) + D53-1 中文长短语神经元不可达 (_neural_associative 模糊联想 + 滑窗切词); 验证 0 幽灵突触/0 孤儿; 神经记忆层落地 (概念神经元去重 + 突触 + 联想检索, backfill 幂等回填) |
| 201 | NT-CORE | 神经记忆层 v2 — Hebb 共现网络 (星型→立体): 概念 co 字段 (同分支两两累计 fire together), query 一阶+二阶扩散 ([关联] 排后); cmd_prune 清理 16 停用词噪声神经元 (幂等); 全库 155 万概念对内存批量+单事务 2s (原逐对 commit >120s); 3 并发 opencode 写 KB 下 hub 仍自洽 (分支=唯一事实源); 停用词收敛纪律 (勿误杀 code/test 等有效词) |
| 202 | NT-CORE | rev-officer Phase 3 并行批: theater gate 8 模块 (research feature, 挖出 memory_budget 函数级 global() 真用回退) + evolution_daemon 挂起修复 (mutation_enabled 默认 true → 测试只读不触发嵌套 cargo 死锁) + Phase 3a hub 拆分 (engine_core reason() 663→14行+4 helper; run.rs 1896→700 拆 handlers_core/consciousness/maintenance 3 worker 经 #[path] mod) + Phase 3b unwrap 审计 (6 热点全在测试块, 生产区已 unwrap_or*) |
| 203 | NT-CORE | 大阵全量审计+修复批: 定性 GWT 共振环为"彩排剧场"(真闭环在背景环 ConsciousnessRuntime+daemon); F1 entry:291 with_panorama 接线 GWT 单例(注释宣称统一实为双实例); F2+F3 删 GeometrySync+ResonatorNetwork 1928 行恒 None 死件(34+16 测试假绿灯); F6 删 L8 Awakening 孪生死影; F7 FEPIIT score→record_metric 真落库+删死方法; F8 CLS 写多读零收敛; F4/F5 seal_loop+background_loop 补真测试; 389 测试过; 第一性原理 3 问审计法(生产什么/谁消费/删了变吗) |

## Skill Routing (何时加载什么)

| 任务 | 加载 |
|------|------|
| `review`/审计/审查/盘点 | `rev-officer-agent.md` → NeoTrix Max 全量审查 (D1-D51+S1-S7) |
| 吸收外部仓库/技术 | `skills/external-absorption/SKILL.md` (C1-C6 契约) + `dev-rules.md` (R-P42/R-P79/R-P80) |
| 会话收尾 | `experience-tree/SKILL.md` → 五阶段吸收 |
| 探索代码库 | `skills/codebase-exploration` (语义搜索/依赖图) |
| 实现功能/修复 | `dev-rules.md` (编码/构建/持久化纪律) + `skills/dev/implementer` |
| 架构设计 | `skills/des/architect` |

## Architecture — 7 Domains as Faction Skill Trees

```
NT-CORE  (E8引导者)  | NT-MIND  (进化工匠)  | NT-MEMORY (知识守护者)
NT-WORLD (虚空探索者) | NT-ACT   (行动执行者) | NT-SHIELD (影卫)
NT-IO    (界面使徒)
```

- **技能节点 3 层**: Small Passive (微节点自愈) / Notable Passive (域级突破) / Keystone (跨域变革)
- **Ascendancy 双专精**: 每 session 两个 Weapon Set，经 `nt_core_self::AttentionManager` 按任务类型路由
- **Rune Socketing 5 槽**: Crimson(数据摄取) / Indigo(变换) / Obsidian(缓存) / Golden(错误恢复) / Alabaster(监控)；组合产生 Runeword (如 Scry = 完整 ETL)
- **Constellation 成熟度**: C0 编译 → C1 单测 → C2 集成测试 → C3 benchmark → C4 主流水线 → C5 自愈/自适应

## Always-On Core Rules (常驻纪律)

- **R-P1**: `#![forbid(unsafe_code)]` — zero unsafe in core
- **构建缓存不可信**: 结构变更后强制 `cargo clean` 或连续 build 两次获取真实错误计数 (R-P9/R-P17/R-P29/R-P35/R-P51/R-P54)
- **R-P16**: 每次编辑后 re-read 文件验证持久化 — 不信工具成功消息
- **R-P79**: 外部技术吸收必须同 session 接线到生产路径，禁止延期死代码
- **R-P42**: 吸收强化现有节点，禁止平行适配器模块
- **全量规则**: 见 `@dev-rules.md` (处理编码/审查任务时加载)

## Shared Language

**Preamble**: Every session loads `CONTEXT.md` at the root as the shared language prefix. All domain terms used in this project are defined there. Before using any domain term, refer to CONTEXT.md for its precise definition and avoid column.

New terms are proposed via `domain-modeling` skill (`skills/engineering/domain-modeling/SKILL.md`) — scan for inconsistencies, stress-test against code, then update CONTEXT.md.

Key shared language decisions:
- Always use the `nt_` prefix for module names (e.g., `nt_core_self`, `nt_world_crawl`)
- Always use the full domain name when referring to a domain (e.g., "NT-WORLD" not "crawler", "NT-CORE" not "core")
- Distinguish "KB embedding" (vector storage) from "VSA embedding" (symbolic representation)
- Distinguish "ConsciousnessTree" (meta-cognition loop) from "GWT" (attention routing)
- Use C0-C6 constellation notation for module maturity (not "levels" or "stages")
- Use "T1/T2/T3" for SelfTest wiring tiers (not "partial/full")

## Build

```sh
cargo build -p neotrix              # CLI
cargo build -p neotrix-tauri        # Desktop
cargo check --all-targets -p neotrix
cargo check --features full --lib -p neotrix
```

## Test

```sh
cargo test -p neotrix --lib         # Unit tests
cargo test -p neotrix --lib -- <test_name>
npm test                            # Frontend tests
```

## Key Locations

| Path | Purpose |
|------|---------|
| `neotrix-core/src/` | Main crate |
| `neotrix-core/src/core/` | Foundation: E8, HyperCube, GWT |
| `neotrix-core/src/neotrix/` | All subsystem modules |
| `neotrix-core/src/cli/` | CLI command definitions |
| `crates/` | Shared libraries |
| `src-tauri/` | Desktop app |
