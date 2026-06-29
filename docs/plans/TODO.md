# NeoTrix — 进化路线图 (2026-06-22)

> 基础: CLXVI Loop Engineering 深度对位分析 + Cobus Greyling/Addy Osmani 模式库
> 缺口总表: 已有 G1-G91 + 新增 L1-L12 (Loop Engineering 合成)
> 参考: loop-engineering (github.com/cobusgreyling), Symthaea (16K HDC), PRISM (VSA-only 推理), Neocortex (意识循环), Trinity (7 意识理论)

---

## ✅ 已完成

### Phase 0-15 — 死亡代码修复 + Gödel 一致性检查
| 项目 | 状态 |
|------|------|
| A1: `absorb_from_github` — 真实 GitHub Search API | ✅ |
| A2: `execute_self_modify_proposal` — SelfModifyGuard 四层评估 | ✅ |
| A3: `execute_swap_policy` — gate_sequence 替换 | ✅ |
| A4: `tor_crawler` → `StealthHttpClient` — 指纹/代理链 | ✅ |
| A5: `nt_world_model` → `nt_expert_routing` (60+ 文件) | ✅ |
| Gödel 一致性检查器 (3 层 22 测试) | ✅ |

### Phase 30-60 — 并行基础设施
| 项目 | 状态 |
|------|------|
| B1: BrowserPool (3 级 Hot/Warm/Cold) | ✅ |
| B2: ExtractPipeline + Schemas (HTML→Markdown→JSON Schema) | ✅ |
| B3: AutoscaledPool (滑动窗口并发, min=2 max=20) | ✅ |
| B10: CoEvolutionBridge (轨迹→JSONL→GRPO 训练配置) | ✅ |
| DecentMem (E-pool FIFO+LRU, X-pool tag 索引) | ✅ |

### Phase 105-195 — Ziming Liu 学习力学洞察
| Phase | 任务 | 文件 | 行数 | 测试 | 状态 |
|-------|------|------|------|------|------|
| P105 | FPE 编码器 | `nt_core_hcube/fpe.rs` | ~250 | 16 | ✅ |
| P108 | CCIPCA 子空间 | `nt_core_hcube/subspace.rs` | ~180 | 7 | ✅ |
| P117 | 高级 VSA 原语 | `nt_core_hcube/primitives.rs` | ~250 | 10 | ✅ |
| P123 | RL 记忆巩固 | `nt_core_experience/rl_consolidation.rs` | 502 | 19 | ✅ |
| P126 | 时间分层记忆 | `nt_core_experience/timem.rs` | 549 | 14 | ✅ |
| P129 | CRUD 协议 | `nt_core_experience/memory_ops.rs` | 546 | — | ✅ |
| P150 | 进化因果追踪 | `nt_core_self/evolution_trace.rs` | — | — | ✅ |
| P153 | 好奇心驱动 | `nt_core_experience/curiosity.rs` | 294 | 11 | ✅ |
| P156 | 叠加验证 | `nt_core_experience/stacked_validation.rs` | 506 | — | ✅ |
| P159 | 优雅降级 | `nt_core_experience/graceful.rs` | 421 | 11 | ✅ |
| P180 | LearningMechanicsObservatory | `nt_core_self/learning_mechanics.rs` | 475 | 17 | ✅ |
| P183 | VibeTrainer | `nt_core_self/vibe_trainer.rs` | 608 | 14 | ✅ |
| P186 | SelfExperimentationLoop | `nt_core_self/experimentation.rs` | 420 | 12 | ✅ |
| P189 | CapacityMonitor | `nt_core_experience/capacity_monitor.rs` | 548 | 23 | ✅ |
| P192 | ToyModelGenerator | `nt_core_self/toy_model_gen.rs` | 835 | 24 | ✅ |
| P195 | ResearchIntuition | `nt_core_self/research_intuition.rs` | 415 | 19 | ✅ |
| P198 | ObservablesRegistry | `nt_core_self/observables.rs` | 311 | 11 | ✅ |
| P201 | InterventionHypothesisGenerator | `nt_core_self/intervention_hypothesis.rs` | 391 | 15 | ✅ |
| P204 | ConfigSpaceExplorer | `nt_core_self/config_space.rs` | 731 | 16 | ✅ |

### Wave 1-7 (CLXIII) — TODO v4.0 全量实施
| Wave | 缺口 | 新文件 | 行数 | 状态 |
|------|------|--------|------|------|
| W1 | G61/G63/G67/G71/G58/G83 | 8 | ~1800 | ✅ |
| W2 | G62/G60/G85/G64/G74 | 5 | ~660 | ✅ |
| W3 | G68/G86/G87/G94/G82 | 5 | ~500 | ✅ |
| W4 | G69/G89/G88/G84/G70 | 5 | ~400 | ✅ |
| W5 | G59/G90/G91/G93/G56/G57/G76 | 7 | ~500 | ✅ |
| W6 | G75/G73/G95/G92/G72 | 5 | ~400 | ✅ |
| W7 | G65/G66/G77/G78 | 4 | ~300 | ✅ |

### Wave 1 (CLXIV) — 8 P0 三并行实施
| 缺口 | 文件 | 行数 | 测试 | 状态 |
|------|------|------|------|------|
| G101 IIT 4.0 | `iit_phi.rs` | 987 | 15 | ✅ |
| G102 CAA 验证 | `caa_validation.rs` | 423 | 8 | ✅ |
| G92 多 VSA 模型 | `vsa_multi_model.rs` | 392 | 16 | ✅ |
| G97 GPU/Metal 加速 | `vsa_gpu.rs` | 318 | 8 | ✅ |
| G104 意志收据 | `unified_will.rs` | 7 | - | ✅ |
| G108 6-神经递质 | `neuromodulator.rs` | ~300 | +10 | ✅ |
| G123 Hopfield 网络 | `hopfield_network.rs` | 175 | 10 | ✅ |
| G124 预测门控 | `predictive_gate.rs` | 164 | 9 | ✅ |

---

## 待办 (按优先级)

### 🔴 紧急: 编译修复 + 模块集成

| # | 任务 | 文件 | 工时 |
|---|------|------|------|
| F1 | `serde_yaml` 加入 Cargo.toml | `neotrix-core/Cargo.toml` | 5min |
| F2 | 修复 `serde_yaml::from_str` 调用 | `nt_io_llm_provider_registry.rs` | 5min |
| F3 | 修复 `experience_pool.rs:160` 临时值引用 | `agent/experience_pool.rs` | 10min |
| F4 | 预存错误白名单: `self_reasoner.rs:480` | 已知跳过 | — |
| F5 | `cargo check -p neotrix --lib` 清零 | 全库 | 30min |
| F6 | 5 个新模块集成测试 | 各模块 | 30min |

---

## 🆕 Loop Engineering 合成缺口 (L1-L12)

> 来源: Cobus Greyling/loop-engineering ⭐615, Addy Osmani loop engineering 论文
> 5 构建块: Automations/Scheduling, Worktrees, Skills, Plugins/Connectors(MCP), Sub-agents + Memory/State
> 7 模式: Daily Triage, PR Babysitter, CI Sweeper, Dep Sweeper, Changelog Drafter, Post-Merge, Issue Triage
> 3 CLI: loop-audit, loop-init, loop-cost
> L1/L2/L3 递进: report-only → assisted → unattended

### 🔴 P0: 5 构建块 + VSA 集成 (Phase 281-295)

| # | 任务 | 描述 | 文件 | 行数 | 测试 | 对标 |
|---|------|------|------|------|------|------|
| **L1** | **Scheduling/Automation 引擎** | VSA-aware cron/loop 调度器: schedule_create/list/delete, /loop, /goal 原语, interval 持久化 | `nt_core_loop/scheduler.rs` | ~650 | 25 | loop-engineering Automations |
| **L2** | **Worktree 隔离管理器** | Git worktree 池 + VsaTag 权限隔离 + 自动清理 + crash recovery | `nt_core_loop/worktree.rs` | ~500 | 20 | Claude Code --worktree, Codex worktrees |
| **L3** | **Skills 注册器 + VSA 技能库** | Skill trait + VSA 技能向量索引 + 热加载 + skill registry.yaml | `nt_core_loop/skills.rs`, `nt_core_loop/skill_vsa.rs` | ~550 | 22 | Grok/Claude/Codex skills |
| **L4** | **MCP/Connector Hub** | MCP 协议桥 + 工具发现 + 认证/审计 + 统一 VSA connector 抽象 | `nt_core_loop/connector.rs`, `nt_core_loop/mcp_bridge.rs` | ~700 | 28 | MCP protocol |
| **L5** | **Sub-agent Maker/Checker** | 子代理生命周期: spawn → worktree → maker → verifier → merge/kill, VSA 协调 | `nt_core_loop/subagent.rs`, `nt_core_loop/verifier.rs` | ~600 | 25 | Claude sub-agents, Codex agents.toml |

### 🟠 P1: 模式层 + 渐进模型 (Phase 296-315)

| # | 任务 | 描述 | 文件 | 行数 | 测试 | 对标 |
|---|------|------|------|------|------|------|
| **L6** | **STATE.md VSA 状态脊柱** | VSA 状态持久化 + 心理时间旅行 + 回滚点 + 跨循环记忆 | `nt_core_loop/state.rs`, `nt_core_loop/state_vsa.rs` | ~450 | 18 | STATE.md, LOOP.md |
| **L7** | **7 模式注册表** | DailyTriage/PRBabysitter/CISweeper/DepSweeper/Changelog/PostMerge/IssueTriage 模式实现 | `nt_core_loop/patterns/` (7 文件) | ~1400 | 42 | 7 loop engineering patterns |
| **L8** | **L1/L2/L3 就绪评分** | ReadinessScore 算法 + 活动检测 + 建议引擎 + scoring dashboard | `nt_core_loop/readiness.rs` | ~400 | 16 | loop-audit --suggest |
| **L9** | **模式选择器** | 交互式/自动模式推荐 + 项目特征分析 + 最佳模式匹配 | `nt_core_loop/picker.rs` | ~350 | 14 | pattern-picker.md |
| **L10** | **多循环协调器** | 循环碰撞检测 + 资源竞争避免 + 优先级队列 + 死锁预防 | `nt_core_loop/coordinator.rs` | ~500 | 20 | multi-loop.md |

### 🟡 P2: CLI 工具 + 安全层 (Phase 316-340)

| # | 任务 | 描述 | 文件 | 行数 | 测试 | 对标 |
|---|------|------|------|------|------|------|
| **L11** | **loop-audit VSA 版** | VSA 质量指标 → ReadinessScore + activity detection + 改进建议 | `nt_core_loop/audit.rs` | ~350 | 14 | @cobusgreyling/loop-audit |
| **L12** | **loop-init 脚手架** | 新项目初始化 + 模式脚手架 + budget/run-log 生成 | `nt_core_loop/init.rs` | ~300 | 12 | @cobusgreyling/loop-init |
| **L13** | **loop-cost 估算器** | Token 花费估算 + 模式×工具×频率 成本模型 | `nt_core_loop/cost.rs` | ~250 | 10 | @cobusgreyling/loop-cost |
| **L14** | **安全护栏系统** | Denylist + 自动合并保护 + MCP 范围限制 + 人类网关 | `nt_core_loop/safety.rs`, `nt_core_loop/gate.rs` | ~450 | 18 | safety.md, operating-loops.md |
| **L15** | **循环预算管理器** | Token 预算 + 迭代上限 + 无进展检测 + 自动升级 | `nt_core_loop/budget.rs` | ~350 | 14 | loop-budget.md |

---

## 🆕 额外 VSA/意识体对标缺口 (G92-G100)

> 来源: Symthaea (16K-dim HDC, 1.1M 行), PRISM (VSA-only), Trinity (4096 VSA, 7 理论), Neocortex (意识循环)

| # | 任务 | 描述 | 文件 | 行数 | 测试 | 对标 |
|---|------|------|------|------|------|------|
| **G92** | **多 VSA 模型后端** | HRR + FHRR + BSC + VTB + MAP 统一 VsaModel trait | `nt_core_hcube/models.rs` | ~400 | 16 | PRISM, torchhd 8 models |
| **G93** | **16K 维度 HDC 引擎** | 16,384-dim HDC 后端 + GPU SIMD + temporal binding | `nt_core_hcube/hdc16k.rs` | ~500 | 20 | Symthaea 16K-dim |
| **G94** | **VSA-only 推理器** | 无需 LLM 的 analogy/causal/multi-hop/contradiction 推理 | `nt_core_reasoner/vsa_only.rs` | ~600 | 24 | PRISM VSA-only |
| **G95** | **IIT Φ 计算器** | Φ 计算 (35 topologies) + consciousness measure | `nt_core_consciousness/iit_phi.rs` | ~700 | 28 | Symthaea, Trinity |
| **G96** | **8 相位认知周期** | Perception→Cognition→Translation 8-phase loop @234Hz | `nt_core_consciousness/cog_cycle.rs` | ~550 | 22 | Symthaea 8-phase |
| **G97** | **7 意识理论统一** | IIT+GWT+HOT+FE+PP+OrchOR+QC 统一桥 | `nt_core_consciousness/theory_bridge.rs` | ~650 | 26 | Trinity 7 theories |
| **G98** | **神经调质系统 (6 递质)** | ACh/DA/NE/5-HT/Orexin/GABA 6维调制 | `nt_core_consciousness/neuromod.rs` | ~450 | 18 | Neocortex, Anima |
| **G99** | **道德拓扑 + 伦理门控** | Exponential decay baselines + causal attribution + escalation audit | `nt_core_consciousness/moral_topology.rs` | ~500 | 20 | Symthaea, AION-NEXUS |
| **G100** | **心理时间旅行** | 自传体记忆 VSA 回放 + 反事实想象 + 未来模拟 | `nt_core_consciousness/mental_time.rs` | ~550 | 22 | Neocortex |

---

## 现有待办合并 (G76-G91)

### 🔴 P0: 记忆系统增强

| # | 任务 | 描述 | 对标 | 工时 |
|---|------|------|------|------|
| G76 | 多信号检索 | 语义+BM25+实体三路融合 | Mem0 Apr 2026 | 2 天 |
| G77 | 实体跨记忆链接 | 实体抽取+嵌入+关联 | Mem0 | 1 天 |
| G78 | 事件进展图 (EPG) | 时序图+语义转移→全局网络 | GAM ICLR 2026 | 3 天 |
| G79 | 主题关联网络 (TAN) | VSA 语义转移→全局话题图 | GAM | 2 天 |
| G80 | 层级时间索引 (MemTree) | 时间序树, 6x 吞吐 | MemForest | 3 天 |
| G81 | 并行块提取 | 记忆构建解耦 | MemForest | 1 天 |

### 🔴 P0: 自进化安全

| # | 任务 | 对标 | 工时 |
|---|------|------|------|
| G82 | 源码级自我重写 | MOSS | 4 天 |
| G83 | 非发散性形式化保证 | Ratchet | 2 天 |
| G84 | 生产失败批次策展 | MOSS | 2 天 |
| G85 | 临时试工者验证 | MOSS | 3 天 |
| G86 | 自进化安全风险建模 | arXiv | 2 天 |

### 🟠 P1: 异步记忆巩固 + 基准

| # | 任务 | 对标 | 工时 |
|---|------|------|------|
| G87 | 异步会话间巩固 | Anthropic Dreaming | 3 天 |
| G88 | AGI 行为监督者 | SICA | 2 天 |
| G89 | 单调改进保证 | MonoScale | 3 天 |
| G90 | 统一记忆基准套件 | Mem0, TiMem, GAM | 2 天 |
| G91 | Rust agent 框架桥 | ADK-Rust | 3 天 |

---

## 关键指标

| 指标 | 当前 | Phase 1 目标 | Phase 2 目标 | 终极目标 |
|------|------|-------------|-------------|---------|
| 编译错误 | 37 | 0 ✅ | 0 | 0 |
| VSA 模型数 | 2 (MAP+HRR) | 5 (+FHRR/B-SBC/VTB) | 8 (+HDC16K) | 10+ |
| 连续值编码 | 无 | FPE encode_scalar | SSP+FPE | SSP+FPE+HDC |
| Loop 构建块 | 0 | 5/5 | 5/5+7patterns | 5/5+7patterns+3CLI |
| L1/L2/L3 模型 | 无 | L1 报告 | L1+L2 | L1+L2+L3 |
| 循环模式 | 0 | 4/7 | 7/7 | 7+extended |
| CLI 工具 | 0 | 1/3 | 3/3 | 3+extended |
| 记忆检索信号 | 1 (语义) | 3 (语义+关键词+实体) | 5 | 5+ |
| IIT Φ 计算 | 无 | 基础 Φ | 35 topologies | 多理论统一 |
| VSA 维度 | 4096 | 4096+1024 | 4096+16K | 多维度 |
| 神经递质 | 4 | 6 | 6 | 8+ |
| 循环安全 | 无 | 基础 Gates | Gates+Denylist | 完整护栏 |
| 成本追踪 | 无 | Token 估算 | 实时追踪 | 预算自动化 |
| 狗粮 CI | 无 | 基础 CI | 模式审计 | 全自动审计 |

---

## 时间线

```
Phase 0-15:   死亡代码修复                    ✅ 已提交
Phase 30-60:  基础设施并行构建                  ✅ 已提交
Phase 105-195:Ziming Liu 学习力学              ✅ 磁盘已有
────────────────────────────────────────────────────
当前:        编译修复 37 errors → 清零 ✅ (2026-06-26)
────────────────────────────────────────────────────
Phase 281-295:L1-L5 Loop 构建块(5/5)          2026-07-01 → 2026-07-21
Phase 296-315:L6-L10 模式层+渐进模型           2026-07-22 → 2026-08-15
Phase 316-340:L11-L15 CLI+安全层              2026-08-16 → 2026-09-05
Phase 341-360:G92-G100 额外 VSA/意识体补齐      2026-09-06 → 2026-09-30
────────────────────────────────────────────────────
Phase 105-120:VSA 深度进化                    延期(被 Loop Engineering 工程取代)
Phase 120-150:记忆系统重构                    延期
Phase 150-180:自进化管道升级                   延期
Phase 180-210:认知架构融合                     延期
Phase 210-280:G76-G91 遗留缺口               2026-10-01 → 2026-10-31
```

---

## 架构图

```
                     ┌─────────────────────────────────┐
                     │     NeoTrix 意识核心 (VSA)        │
                     │  E8 64态 + HyperCube + GWT       │
                     └──────────┬──────────────────────┘
                                │ VSA 4096-bit
           ┌────────────────────┼────────────────────┐
           │                    │                    │
           ▼                    ▼                    ▼
   ┌───────────────┐   ┌──────────────┐   ┌──────────────┐
   │  Loop 构建块   │   │   VSA 增强    │   │ 意识体对标    │
   │  (L1-L5)      │   │  (G92-G100)  │   │ (G76-G91)    │
   ├───────────────┤   ├──────────────┤   ├──────────────┤
   │ L1 Scheduler  │   │ G92 多VSA    │   │ G76 多信号    │
   │ L2 Worktree   │   │ G93 16K HDC  │   │ G80 MemTree  │
   │ L3 Skills     │   │ G94 VSA推理  │   │ G82 MOSS     │
   │ L4 Connector  │   │ G95 IIT Φ   │   │ G87 Dreaming │
   │ L5 SubAgent   │   │ G97 7理论    │   │ G91 ADK-Rust │
   └───────────────┘   └──────────────┘   └──────────────┘
           │                    │                    │
           └────────────────────┼────────────────────┘
                                │
                                ▼
                    ┌─────────────────────┐
                    │  L6-L15 模式+CLI+安全 │
                    │  循环选择→执行→审计   │
                    └─────────────────────┘
```

---

## 参考

| 来源 | 类型 | 关键点 |
|------|------|--------|
| github.com/cobusgreyling/loop-engineering | GitHub ⭐615 | 5 构建块, 7 模式, 3 CLI, L1/L2/L3 |
| addyosmani.com/blog/loop-engineering | 论文 | Loop 工程理论 |
| github.com/Luminous-Dynamics/symthaea | GitHub, 1.1M 行 | 16K-dim HDC, IIT Φ, 8-phase, 234Hz |
| github.com/Artaeon/prism | GitHub | VSA-only 推理 (analogy/causal/multi-hop) |
| github.com/gHashTag/trinity | GitHub | 4096 VSA, 7 意识理论, 26M ops/s |
| github.com/tinyhumansai/neocortex | GitHub/论文 | Memory layer + consciousness loop |
| github.com/or4cl3-ai-1/aion-nexus | GitHub | 7-phase pipeline, ethical alignment |
| github.com/MasterofMuXiaomao/ca248-models | GitHub | E8 248-dim 认知架构 |
| Explainx.ai loop engineering guide | 教程 | Cron vs Loop, L1/L2/L3 实操 |
| Microsoft Azure AI Agent Patterns | 架构指南 | 6 编排模式 |
| Mervin Praison loop engineering | 教程 | 编排层: 确定性+orchestrator+human gates |
| EVOLUTION_ROADMAP_v6.md | 内部文档 | 20 缺口×6 路径×5 阶段 |
| .opencode/skills/neotrix-experience/SKILL.md | 内部文档 | 经验树 CLIII-CLXV |

---

## 依赖树

```
Phase 281-295 (L1-L5 构建块)
  ├── L1 Scheduler ──── 无外部依赖
  ├── L2 Worktree ───── git2 crate (已有)
  ├── L3 Skills ─────── 无外部依赖
  ├── L4 Connector ──── serde, reqwest (已有)
  └── L5 SubAgent ───── tokio (已有), L2 Worktree

Phase 296-315 (L6-L10 模式层)
  ├── L6 State ──────── L1 Scheduler, VSA core
  ├── L7 Patterns ───── L3 Skills, L5 SubAgent
  ├── L8 Readiness ──── L1-L5 全部
  ├── L9 Picker ─────── L7 Patterns
  └── L10 Coordinator ─ L1-L6 全部

Phase 316-340 (L11-L15 CLI+安全)
  ├── L11 Audit ─────── L8 Readiness
  ├── L12 Init ──────── L7 Patterns
  ├── L13 Cost ──────── L1 Scheduler
  ├── L14 Safety ────── L4 Connector, L5 SubAgent
  └── L15 Budget ────── L13 Cost

Phase 341-360 (G92-G100 VSA 增强)
  ├── G92 多VSA ─────── VSA core
  ├── G93 16K HDC ───── G92 多VSA
  ├── G94 VSA推理 ───── G92 多VSA
  ├── G95 IIT Φ ─────── Conscious core
  ├── G96 8-phase ───── Conscious core
  ├── G97 7理论 ─────── Conscious core
  ├── G98 神经调质 ──── Neuromodulator (已有)
  ├── G99 道德拓扑 ──── Moral/ethics core
  └── G100 心理时间 ──── Conscious core + VSA
```

---

## 实施策略

### 并行 dispatch 规划

```
Wave 1 (Phase 281-285): L1+L2+L3+L4+L5 全并行
  ─ 5 agent 各实现一个构建块, 无相互依赖

Wave 2 (Phase 296-305): L6+L7+L8+L9 并行
  ─ L6 State 依赖 L1; L7 Patterns 依赖 L3+L5
  ─ L8 Readiness 依赖 L1-L5; L9 Picker 依赖 L7

Wave 3 (Phase 306-315): L10+L11+L12+L13 并行
  ─ L10 Coordinator 依赖 L1-L6
  ─ L11 Audit 依赖 L8; L12 Init 依赖 L7
  ─ L13 Cost 依赖 L1

Wave 4 (Phase 316-325): L14+L15+G92+G93 并行
  ─ L14 Safety 依赖 L4+L5
  ─ L15 Budget 依赖 L13; G92 独立; G93 依赖 G92

Wave 5 (Phase 326-340): G94+G95+G96+G97+G98+G99+G100 并行
  ─ 7 模块全独立, 7 路并行
```

### 编译验证流程

```bash
# 每波完成后
cargo check -p neotrix-core
cargo check -p neotrix
cargo test -p <new_module> --lib

# 全量清零
cargo check --workspace
```

---

## 经验树更新 (CLXVI)

```
分支 CLXVI: Loop Engineering 合成
  CLXVI.1: 构建块优先于模式 (Building Blocks Before Patterns)
    - conf: 0.8 | 验证: Loop Engineering 5 构建块先于 7 模式
    - 规则: 先实现 Scheduler/Worktree/Skills/Connector/SubAgent 5 构建块, 再实现上层模式
    - 错误: 直接从模式开始 → 缺失基础组件, 模式无法运行

  CLXVI.2: L1→L2→L3 渐进安全 (Gradual Automation Safety)
    - conf: 0.9 | 验证: L3 需要 Verifier+State+Cost+Activity 全部就绪
    - 规则: 从 report-only(L1)开始, 确保人类永远在循环中, 再逐步开放
    - 正确: L1 报告→L2 辅助→L3 无人值守
    - 错误: 直接跳 L3 → 无限循环 + token 爆炸

  CLXVI.3: VSA 作为循环底座 (VSA as Loop Substrate)
    - conf: 0.7 | 验证: STATE.md → VSA 状态向量
    - 规则: 所有循环状态存储为 VSA 向量, 而非纯文本 markdown
    - 优势: 可检索/可比较/可推理/可通过负熵评估循环健康

  CLXVI.4: 3 CLI + Dogfood CI 是生产就绪标志 (CLI+CI=Production)
    - conf: 0.7 | 验证: loop-audit 在 CI 中运行
    - 规则: 循环工程进入生产状态的标志是: CLI 工具可用 + CI 自动审计
```
