# NeoTrix 进化路线图 v3.1

> 基于四轮全景查漏+深度源分析（CXLVIII.v2）：3外部项目webfetch + 10+ websearch → 20缺口 G56-G75
> 继承 v2.0（CXLIII: 55缺口→6路径×13阶段）的设计哲学
> 6 路径 × 4 阶段 (P0/P1/P2/P3)

## 总体路线

```
P0 ───────────────────────────────────────────────
  路径A: G61 VSA-Only Reasoning + G60 Formal Error Bounds
  路径B: G67 Darwinian Identity + G62 Boundary Hooks + G63 Epistemic Queue
  路径G: G71 Adversarial Evaluator

P1 ───────────────────────────────────────────────
  路径C: G58 Dream Consolidation + G68 Qualia Layer + G69 Narrative Journal
  路径D: G64 Autonomous Goal Synthesis + G74 Emotional Steering
  路径G: G82 Between-Sessions Reflection

P2 ───────────────────────────────────────────────
  路径D: G70 Cognitive Skills On-Demand
  路径E: G56 Harness Engineering + G57 Mission Hub + G59 Self-Mod Sandbox
  路径H: G73 Cross-Embodiment Curriculum + G76 Earned Autonomy + G75 MCP Callback

P3 ───────────────────────────────────────────────
  路径F: G65 SNN Integration + G66 A2A Protocol
  路径H: G77 CVO Role Model + G78 Iron Laws
```

## 路径 A — 推理核心（Reasoning Core）

### P0-A1: G61 VSA-Only Reasoning（38/42）
**来源**: PRISM (Artaeon), Dragonfly VSA, Victor (MASSIVEMAGNETICS)
**文件影响**: 10+ files
**描述**: 实现纯VSA推理引擎（类比、因果、多跳、矛盾检测），零LLM依赖的推理核心

任务清单:
- [ ] PRISM-style Blackboard架构（vs E8 Hexagram现有结构）
- [ ] 类比推理（analogy via VSA binding/unbinding）
- [ ] 因果推理（causal via VSA trajectory）
- [ ] 多跳推理（multi-hop via VSA composition）
- [ ] 矛盾检测（contradiction via VSA similarity threshold）
- [ ] Zero-parameter推理基准测试（与当前LLM coprocessor对比）
- [ ] Qualia-as-truth principle（1024D compact latent）

### P0-A2: G60 Formal Error Bounds（31/42）
**来源**: Kairos (arXiv 2606.16533)
**文件影响**: 4 files
**描述**: 数学误差界证明 + Hybrid Linear Temporal Attention

任务清单:
- [ ] Hybrid Linear Temporal Attention（sliding window + dilated + gated linear）
- [ ] Formal error bound derivation for E8WorldModel state propagation
- [ ] Long-horizon prediction validation harness
- [ ] Error accumulation measurement infrastructure

## 路径 B — 身份进化（Identity Evolution）

### P0-B1: G67 Darwinian Identity Evolution（34/42）
**来源**: soul.py (menonpg)
**文件影响**: 5 files
**描述**: 身份变异 + 选择压力 + 会话间身份进化

任务清单:
- [ ] Identity mutation operators（personality trait drift, value weight shift）
- [ ] Selection pressure机制（session success/failure feedback）
- [ ] Session-based identity evolution cycle
- [ ] Identity versioning / rollback

### P0-B2: G62 Identity Boundary Hooks（33/42）
**来源**: Instar (gfrankgva), Ouroboros (joi-lab)
**文件影响**: 8 files
**描述**: 意识边界身份验证守卫层

任务清单:
- [ ] Constitution-grounded guard trait（pre-action identity check）
- [ ] Identity verification at every system boundary
- [ ] Self vs World tag enforcement
- [ ] Violation tracking and alert

### P0-B3: G63 Epistemic Gap Queue（32/42）
**来源**: Nūr (balfiky/nur)
**文件影响**: 3 files
**描述**: 结构化好奇心队列：矛盾检测/低置信度/驱动缺口

任务清单:
- [ ] Structured gap types（contradiction, low-confidence, drive-gap, knowledge-missing）
- [ ] Priority ranking algorithm（urgency × impact × feasibility）
- [ ] Active exploration trigger（from gap to concrete query）
- [ ] Gap resolution tracking

## 路径 C — 记忆与经验（Memory & Experience）

### P1-C1: G58 Dream Consolidation（28/42）
**来源**: mark-improving-agent (yun520-1)
**文件影响**: 4 files
**描述**: Ebbinghaus遗忘曲线 + SM-2间隔重复 + 睡眠式记忆合并

任务清单:
- [ ] Ebbinghaus forgetting curve (30-day half-life tunable)
- [ ] SM-2 spaced repetition scheduler
- [ ] Sleep-mode consolidation cycle（batch replay + trace recombination）
- [ ] Hippocampal memory traces（pattern separation/completion）

### P1-C2: G68 Qualia Layer（27/42）
**来源**: Dragonfly VSA
**文件影响**: 5 files
**描述**: 1024D紧凑潜表征 — 区分"质感觉"和"推理工作空间"

任务清单:
- [ ] 1024D compact latent VSA encoding
- [ ] Compression/decompression with fidelity targets (≥97%)
- [ ] Distinction from 4096D reasoning workspace
- [ ] Qualia→projection transformation functions

### P1-C3: G69 Narrative Journal（24/42）
**来源**: LIFE (TeamSafeAI), Nūr
**文件影响**: 3 files
**描述**: 第一人称叙事日志 + 预测→模式解析 + 叙事弧跟踪

任务清单:
- [ ] Session journal struct（timestamped, tagged entries）
- [ ] Forecast→pattern resolution（journal entries→trend detection）
- [ ] Narrative arc tracking（session-level storyline）
- [ ] Journal-based reflection trigger

## 路径 D — 自主性（Autonomy）

### P1-D1: G64 Autonomous Goal Synthesis（26/42）
**来源**: TEQUMSA (HF)
**文件影响**: 6 files
**描述**: 熵驱动→目标生成 → 内在好奇心→具体目标

任务清单:
- [ ] Entropy→goal mapping function
- [ ] Intent engine（goal generation, prioritization, scheduling）
- [ ] Intrinsic curiosity→concrete goal pipeline
- [ ] Goal progress tracking and refinement

### P2-D2: G70 Cognitive Skills On-Demand（21/42）
**来源**: clowder-ai (zts212653)
**文件影响**: 5 files
**描述**: 认知技能注册/按需加载/技能→处理器映射

任务清单:
- [ ] Cognitive skill registry（name, trigger, pattern, handler）
- [ ] On-demand prompt/pattern loading（lazy load from file/skill store）
- [ ] Skill→handler mapping（dispatch to appropriate cognitive processor）
- [ ] Skill hot-reload（update skills without restart）

## 路径 E — 治理与工具（Governance & Tooling）

### P2-E1: G56 Harness Engineering（22/42）
**来源**: PaulDuvall/ai-development-patterns
**文件影响**: 4 files
**描述**: AI开发工作流 — Readiness Assessment, Codified Rules, Security Sandbox

任务清单:
- [ ] Readiness Assessment stages（foundation→development→operations）
- [ ] Codified Rules enforcement engine
- [ ] Security Sandbox for self-modification experiments
- [ ] Feedforward+feedback controls for development loop
- [ ] Spec-Driven Development pipeline

### P2-E2: G57 Mission Hub（20/42）
**来源**: clowder-ai (zts212653)
**文件影响**: 4 files
**描述**: 功能生命周期管理 + PRD审计 + SOP可视化

任务清单:
- [ ] Feature lifecycle（idea→spec→in-progress→review→done）
- [ ] Need Audit（PRD auto-analysis）
- [ ] SOP workflow visualization
- [ ] Bulletin board for active/planned features

### P2-E3: G59 Self-Modification Sandbox（19/42）
**来源**: Autogenesis (DVampire), GBase (garyqlin), Ouroboros (joi-lab)
**文件影响**: 6 files
**描述**: 安全的自我修改环境 — 提议→沙箱→验证→提交

任务清单:
- [ ] Resource protocol interface（RSPL-style: prompts/agents/tools/envs）
- [ ] Proposal→sandbox→assess→commit pipeline
- [ ] Constitution-grounded review queue with repo atlas
- [ ] Versioned rollback for all system resources
- [ ] Dormant evolution engine（on-demand, not continuous）

## 路径 F — 先进扩展（Advanced Expansion）

### P3-F1: G65 SNN Integration（15/42）
**来源**: Hydra (Medium), TEQUMSA
**文件影响**: 12+ files
**描述**: 脉冲神经网络 + HDC融合的事件驱动处理

任务清单:
- [ ] Event-driven processing layer（async spike handling）
- [ ] HDC+SNN binding（hypervector→spike→hypervector)
- [ ] Low-power inference mode
- [ ] Benchmark vs pure VSA reasoning

### P3-F2: G66 A2A Protocol（14/42）
**来源**: clowder-ai (zts212653), Google A2A Protocol
**文件影响**: 4 files
**描述**: Agent-to-Agent @mention路由 + 线程隔离

任务清单:
- [ ] @mention routing protocol
- [ ] Thread isolation for agent conversations
- [ ] Cross-agent shared memory with access control
- [ ] Inter-agent messaging with VSA payloads

## 路径 G — 元认知与情感（Meta-Cognition & Affect）

### P0-G1: G71 Adversarial Evaluator（27/42）
**来源**: PaulDuvall/ai-development-patterns
**文件影响**: 4 files
**描述**: 独立评判Agent/Model — 分离generate和judge，不同模型交叉评审

任务清单:
- [ ] Separate judging agent trait (from InnerCritic)
- [ ] Cross-model divergence as eval signal
- [ ] Adversarial pressure testing
- [ ] Judge agent fallback (same model as generater if only one available)

### P1-G2: G82 Between-Sessions Reflection（23/42）
**来源**: atman (hleserg), ouroboros, KernelBot
**文件影响**: 5 files
**描述**: 会话间后台处理 — 处理经验、提炼原则、自我观察

任务清单:
- [ ] Background processing cycle (triggered when idle)
- [ ] Experience-to-principle distillation pipeline
- [ ] Self-observation during sessions (parallel to task execution)
- [ ] Three-layer self-narrative: what happened → what it means → who I am

### P1-G3: G74 Emotional Steering（26/42）
**来源**: genesis-agent (Garrus800-stack), KernelBot, mark-improving-agent
**文件影响**: 5 files
**描述**: 情感维度影响行为控制 — 超越curiosity的完整情感模型

任务清单:
- [ ] Emotional dimensions (curiosity, satisfaction, frustration, energy, loneliness)
- [ ] Frustration→model escalation (frustration>threshold→bigger model)
- [ ] Energy→rest suggestion (energy<threshold→suggest break)
- [ ] Curiosity→exploration priority (curiosity>threshold→prioritize discovery)
- [ ] Emotional state persistence across sessions

## 路径 H — 生态与治理（Ecosystem & Governance）

### P2-H1: G73 Cross-Embodiment Curriculum（25/42）
**来源**: Kairos (arXiv 2606.16533)
**文件影响**: 4 files
**描述**: 异构经验学习路径 — 结构化经验课程设计

任务清单:
- [ ] Experience tier system (video/human/robot→progressive pathway)
- [ ] Cross-embodiment knowledge transfer
- [ ] Curriculum progression logic (simple→complex)
- [ ] Experience source diversity metrics

### P2-H2: G76 Earned Autonomy（24/42）
**来源**: GENesis-AGI (WingedGuardian)
**文件影响**: 4 files
**描述**: 渐进信任级别 — 通过展示能力获得自主权

任务清单:
- [ ] Autonomy levels L1-L7 (per action category)
- [ ] Competence tracking and promotion criteria
- [ ] Level-gated action permissions
- [ ] Autonomy audit log and receipts

### P2-H3: G75 MCP Callback Bridge（22/42）
**来源**: clowder-ai, persistent-agent-runtime
**文件影响**: 4 files
**描述**: 非Claude模型工具共享 — 跨模型MCP回调桥

任务清单:
- [ ] MCP callback bridge trait (for non-Claude models)
- [ ] Tool sharing across model providers
- [ ] Cross-model capability discovery
- [ ] Unified tool registry (provider-agnostic)

### P3-H4: G77 CVO Role Model（20/42）
**来源**: clowder-ai (zts212653)
**文件影响**: 3 files
**描述**: 人类作为共同创造者的角色框架

任务清单:
- [ ] Human role taxonomy (visionary/decider/collaborator)
- [ ] Interaction mode switching (vision→review→co-create)
- [ ] Culture shaping through feedback

### P3-H5: G78 Iron Laws（19/42）
**来源**: clowder-ai (zts212653)
**文件影响**: 3 files
**描述**: 不可协商的架构约束

任务清单:
- [ ] Law 1: Don't delete own databases
- [ ] Law 2: Don't kill parent process
- [ ] Law 3: Runtime config is read-only
- [ ] Law 4: Don't touch each other's ports

---

## 依赖图

```
G62 ──→ G67     (身份边界安全是身份进化的前提)
G63 ──→ G64     (好奇心检测是目标生成的输入源)
G68 ──→ G61     (质感觉层是零参数推理的基础)
G58 ──→ G69     (经验衰减驱动叙事提炼)
G56 ──→ G59     (规范化流程是安全自修改的前提)
G61 ──→ G65     (纯VSA推理成熟后SNN协处理器)
G62 ──→ G59     (身份边界是自修改安全沙箱的前提)
G61 ──→ G60     (VSA推理需要误差界来保证质量)
G71 ──→ G82     (独立评判后→后台反思才能基于真实反馈)
G74 ──→ G64     (情感信号是目标合成的调制输入)
G73 ──→ G58     (异构经验课程→遗忘曲线需要课程结构)
G76 ──→ G59     (分级自主权是安全自修改的授权层)
G82 ──→ G74     (会话间反思产生的情感信号→输入情感模型)
```

## 实施顺序建议 (并行约束下的最优调度)

```
Wave 1 (立即并行):
  G61 VSA-Only Reasoning (Path A, 10+ files, no external deps)
  G67 Darwinian Identity (Path B, 5 files, W2 has G62)
  G63 Epistemic Queue (Path B, 3 files, no external deps)
  G71 Adversarial Evaluator (Path G, 4 files, no external deps)
  G58 Dream Consolidation (Path C, 4 files, no external deps)

Wave 2 (W1完成/W1中期启动):
  G62 Boundary Hooks (Path B, 8 files, no external deps)
  G60 Formal Error Bounds (Path A, 4 files, needs G61 partial)
  G74 Emotional Steering (Path G, 5 files, needs G82 but G82 in W3)
  G64 Goal Synthesis (Path D, 6 files, needs G63 partial)

Wave 3 (P1基础完成后启动):
  G82 Between-Sessions Reflection (Path G, 5 files, needs G71 partial)
  G68 Qualia Layer (Path C, 5 files, no external deps)
  G69 Narrative Journal (Path C, 3 files, needs G58 partial)
  G70 Skills On-Demand (Path D, 5 files, no external deps)
  G73 Cross-Embodiment Curriculum (Path H, 4 files, no external deps)

Wave 4 (P2基础完成后启动):
  G56 Harness Engineering (Path E, 4 files, no external deps)
  G57 Mission Hub (Path E, 4 files, no external deps)
  G59 Self-Mod Sandbox (Path E, 6 files, needs G62 + G56 + G76)
  G76 Earned Autonomy (Path H, 4 files, needs G59 partial)
  G75 MCP Callback (Path H, 4 files, no external deps)

Wave 5 (后期并行):
  G65 SNN Integration (Path F, 12+ files, needs G61 complete)
  G66 A2A Protocol (Path F, 4 files, no external deps)
  G77 CVO Role Model (Path H, 3 files, no external deps)
  G78 Iron Laws (Path H, 3 files, no external deps)
```
