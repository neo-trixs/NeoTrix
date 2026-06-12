# 进化迭代 TODO List — 5 启发点深度融合

> 构建于: 2026-06-12
> 基础: blueprint (AI Agent 工程化核心技术架构) + 5 启发点 + 20+ 论文搜索 + 全量代码库审计

---

## 审计摘要 (当前状态)

| 启发点 | 代码库状态 | 缺口严重度 |
|--------|-----------|:----------:|
| ① Prediction-before-execution | 基础设施存在 (`predict_success`, `record_prediction`, 3个校准器) 但**未接线为 pre/post 比较循环** | 🔴 |
| ② Failure clustering | `extract_anti_patterns()` 做 string 分组； `RecurrenceDetector` 做 memory VSA 聚类但**不做 failure 聚类** | 🟡 |
| ③ Event log schema | `VsaTagged` 有 confidence/tag/salience，**无 prediction/outcome/belief_delta 字段** | 🔴 |
| ④ Transcript analysis | `DreamConsolidator` 存在但**被喂空 patterns** (`feed("bg", &[])`)；模块有但未接线 | 🔴 |
| ⑤ Meta-reflection | `MetaCognitiveLoop` 做代码级外部分析，**不消费 batch 自身指标** (ECE/meta-d'/M-ratio) | 🟡 |

---

## 论文驱动的启发点完善

### 启发点 ① → 完整定义: Prediction-Before-Execution 元认知校准环

**核心机制**: 每次决策前强制写 predict(outcome, confidence)，执行后自动比对 → 更新 calibration curve → 校准 taste。

**论文依据**:
- **Agentic Confidence Calibration (HTC, arXiv:2601.15778)** — 轨迹级过程特征提取，跨步骤动力学 + 步内稳定性 + 位置指标。GAC (通用校准器) 零样本最佳 ECE=0.118。
- **Mirror Benchmark (arXiv:2604.19809)** — 组合自预测普遍失败 (CCE 0.434-0.943); 但外部元认知脚手架将 CFR 从 0.600 降至 0.143 (76% 降幅)。**外部约束 > 自知识改善** → 我们不需要模型"更好的自我认知"，我们需要 **Harness 层的强制预测-比对环**。
- **Metacognitive Probe (arXiv:2605.09844)** — 5 维度行为校准诊断：T1-CC (置信度校准)、T2-EV (认知警觉)、T3-KB (知识边界)、T4-CR (跨任务范围)、T5-RCV (推理链验证)。扩展校准器至 5 维。
- **MetaCogAgent (arXiv:2605.17292)** — 自评估模块 ECE=0.087；元认知冲突检测 (verbal vs performance 信号分歧) → delegatio n 阈值动态收紧。**二阶不确定性信号**。
- **Metacognitive Harness for TTS (arXiv:2605.14186)** — FOK+JOL 双信号 SVM 控制器，轻量诊断锚定集 → 决策规则。**轻量、可训练的元认知控制层**。

**关键设计决策**:
- **外部约束 > 内部修正**: Mirror 证明提供校准分数给模型无显著改善 (p>0.05)，只有架构约束有效。所以不是"让模型更诚实"，而是"Harness 层强制执行预测日志 + 比对 + 校准更新"。
- **三校准器合一**: 现有 EpistemicSelfModel / ConfidenceCalibrator / EpistemicHonesty 三个独立校准器，合并为统一 `CalibrationEngine`，输出单一校准的 confidence。
- **HTC 特征无需全实现**: 我们不需要 GAIA 级跨域校准器。只需要 **process-centric**: 在 `handle_decision_compress` 前后各记录一次 VSA 向量，提取关键特征 (最后一跳 salience, internal coherence, 跨步变动)。

---

### 启发点 ② → 完整定义: VSA Failure Clustering + Cluster-Aware 修复管线

**核心机制**: 在 consciousness_batch 中加入 VSA-based failure clustering 阶段，自动聚类失败案例 → 识别最大失败堆 → 喂入 PolicyRepair / CurriculumGenerator。

**论文依据**:
- **AutoTriage (OpenReview 2026)** — 沙箱化 agentic judge 自动归类失败原因。关键发现：弱模型系统性过度归因到任务缺陷。**自我修复循环缺失的根本原因：失败归因是闭环的前置条件**。
- **TRACE (arXiv:2604.05336)** — 对比成功/失败轨迹 → 自动识别缺失能力 → 合成针对性训练环境 → RL 训练 LoRA adapter。**Capability-targeted 训练**：不是修复单个失败，而是修复缺失能力簇。
- **NeoSigma Self-Improving Agentic Systems** — 生产环境中自动发现 29+ 失败簇，按 root-cause 机制分组，高 `total_failures` + 低 `resolution_rate` 优先修复。
- **ErrorProbe (arXiv:2604.17658)** — 三步归因：MAST 分类法结构分解 → 症状驱动回溯(剪枝无关上下文) → 多 Agent 诊断 + 可验证记忆(仅证据确认的模式才持久化)。**Verified episodic memory**: 不是所有模式都存，仅工具确认的。
- **Pioneer Agent (arXiv:2604.09791)** — 失败分类法构建 + live confirmation(探测确认弱点系统化) + parent model awareness。

**关键设计决策**:
- **VSA 相似度聚类(非 string 分组)**: 现有 `extract_anti_patterns()` 用 string match，无法发现语义相似但表述不同的失败。改用 VSA Hamming 相似度 (复用 RecurrenceDetector 机制)。
- **二分门控**: 失败簇按 min_samples=3 + coherence≥0.78 自动识别。大簇 (>10) 拆分子簇。
- **Cluster → PolicyRepair 短路**: 当同一簇失败率 > 50% 且 resolution_rate < 0.3 时，跳过 normal handler，直接触发 `handle_policy_repair(cluster_id)`。
- **轻量实现**: 不需要 AutoTriage 的沙箱 LLM judge。只需要 VSA 相似度 + 频率统计 + 现有 PolicyRepairEngine。

---

### 启发点 ③ → 完整定义: Prediction-Outcome-Belief 事件日志结构

**核心机制**: 每个 batch 的 VsaTagged entry 扩展为包含 pre-prediction / post-outcome / belief-delta 三段的统一结构，实现"写下来"的可追溯性。

**论文依据**:
- **AgentTrace (arXiv:2602.10133)** — 三表面结构化 logging: cognitive (推理链/置信度) + operational (方法/状态/耗时) + contextual (工具调用/数据访问)。所有表面共享统一 envelope。**这是第一个开源 Agent 日志标准**。
- **The Log is the Agent — ActiveGraph (arXiv:2605.21997, Yohei Nakajima)** — 追加事件日志作为唯一真相源，working graph 是日志的确定性投影。**可确定性重放：每个运行可从日志完整重建**。支持 cheap forking。
- **AGENTOBS (RFC-0001)** — 开放事件 schema 标准: event envelope + agent span 层次 + token/cost 模型 + HMAC 审计链 + PII 脱敏 + OTLP 导出。已发布 v2.0 schema, 3000+ 测试。
- **Self-Archaeology Pattern (Agent Patterns Catalog)** — 周期 compaction pass: 按主题分组近期 thoughts，提取各时期立场，写 trajectory note。**明确禁止无轨迹支持的立场声称**。
- **chitta-research (belief graph)** — 8 节点类型 + 5 边缘类型的有向信念图。每条假设带 prior/posterior confidence。**失败了不能丢, 必须降低该领域 recall score**。
- **agentic-experiments / aexp** — Hypothesis → Experiment → Finding 强制链，git commit 绑定。signac-backed 运行追踪。

**关键设计决策**:
- **不是替换 VsaTagged，而是扩展**: `VsaTagged` 添加 `prediction: Option<PredictionRecord>` 和 `outcome: Option<OutcomeRecord>` 字段。
- **PredictionRecord**: `{predicted_vector, confidence, timestamp_before}` — 决策前记录。
- **OutcomeRecord**: `{actual_vector, success, error_category, timestamp_after}` — 决策后记录。
- **BeliefDelta**: 自动从 batch 前后 EpistemicSelfModel 的状态导出，不需要单独存储，作为派生字段。
- **可重放**: 每 500 cycles 自动输出 checkpoint (当前所有 PredictionRecord)，支持离线分析。

---

### 启发点 ④ → 完整定义: 自我转录分析 — 从自己输出中提取模式

**核心机制**: consciousness_batch 末尾加阶段：读取近期 batch 日志 → VSA pattern 提取 → 识别 recurring reasoning primitives → 注入 SkillAccumulator/CurriculumGenerator。

**论文依据**:
- **Inducing Reasoning Primitives from Agent Traces (arXiv:2606.02994, CMU)** — 从 Agent 自身 ReAct traces 中自动发现可复用推理子程序。**诱导库击败原 Agent** (最多 +44pp)。核心: filter(成功轨迹) → extract(thoughts) → categorize+merge → synthesize pseudo-tools。
- **Trajectory Intelligence Extraction (arXiv:2603.10600)** — 超越原始 logging 到语义理解：分析型思考、规划模式、验证行为、反思模式、自纠序列。**轨迹智能提取器 + 决策归因分析器 + 上下文学习生成器**。
- **claude-patterns / Instinct Engine** — 扫描最近 sessions/decisions/journal → 识别 recurring themes/decision logic/metaphors → 转为 CLAUDE.md rules / concept notes / skill 改进。**自动化模式识别**。
- **MARS (arXiv:2601.11974)** — principle-based reflection (抽象规范规则避免错误) + procedural reflection (推导分步策略复制成功)。**单循环高效的自我进化**。

**关键设计决策**:
- **复用 DreamConsolidator 而非新建**: 现有 DreamConsolidator 已经被喂空 patterns。修复调用点，将 batch 决策结果作为 pattern feed。
- **两步诱导 (CMU 风格)**: 不是直接 feed 原始 traces。先 categorize+merge (VSA 聚类同类 thinking pattern)，再 synthesize (从聚类中提取规范形式)。只存规范形式，不存原始 traces。
- **Primitives 作为 pseudo-skills**: 诱导出的推理模式注入 SkillAccumulator 作为 VSA skill。每 50 cycles 运行一次 induction pass。
- **轻量实现**: 不需要 LLM judge。DistillationBridge + VSA similarity + pattern extraction 已在代码库中存在。

---

### 启发点 ⑤ → 完整定义: 双层元反思 — "我变好了吗？"

**核心机制**: consciousness_batch 中加入 self-assessment handler，读取 batch 积累的校准指标 (ECE, meta-d', M-ratio, calibration error trend)，判断改进方向，写入 Event Log 作为元条目。

**论文依据**:
- **Bilevel Autoresearch (arXiv:2603.23420)** — 外环 meta-optimizes 内环的搜索机制，生成 Python 代码注入运行时。**Level 2 (机制替换) 比 Level 1.5 (参数调整) 好 5×**。关键: 不是"调整参数"，是"生成新机制替换旧逻辑"。
- **HyperAgents / DGM-H (Meta, arXiv:2603.19461)** — 自指自我改进: meta agent 可以修改自身改进机制。元层改进跨域转移 and 跨运行积累。**元认知自我修改: 不仅改行为，还改"改行为"的机制**。
- **Arbor — Hypothesis Tree Refinement (arXiv:2606.11926)** — 持久化假设树 + coordinator/executor 分离。每一轮证据写回树节点，向上抽象局部发现，决定扩展/剪枝/合并。**假设树作为操作研究状态**。
- **Sibyl — Self-Evolving Research System** — 每次迭代后自动提取 8 类别 lessons, time-decay weighted, 注入 agent prompts。**系统本身在变得更擅长运行研究**。
- **Self-Archaeology Pattern** — 周期 compaction: 按主题分组 thoughts → 提取各时期立场 → 写 trajectory note。明确禁止无轨迹支持的立场声称。

**关键设计决策**:
- **内环: per-batch 指标追踪**: 每 cycle 读取 ConfidenceCalibrator.ECE() + EpistemicHonesty.meta_d() + M_ratio() + EpistemicSelfModel.calibration_error()。追踪 100-cycle rolling window。
- **外环: 每 200 cycles 机制级比较**: 不是"参数调优"，而是比较两个阶段 {"before: no prediction hook" vs "after: with prediction hook"} 的 ECE/CFR 差异。如果差异显著，写 AGENTS.md 风格报告。
- **不自动改代码**(Level 2 太激进): 先做 Level 1.5 (报告 + 生成改进建议，人类确认后执行)。与 DGM-H writeback 的安全门一致。
- **轨迹报告 (Self-Archaeology)**: 每 100 cycles 生成 `belief_trajectory.json`: "在 cycle 1-100 期间，我的 calibration error 从 0.12 降至 0.08；meta-d' 从 1.2 升至 1.5。主要改进来自固定预测钩子接线。"

---

## 完整进化 TODO List

### P0 🔴 — 核心循环 (当前 batch 可插拔, 高 ROI)

| # | 任务 | 涉及文件 | 预估行数 | 论文驱动 |
|---|------|---------|:--------:|---------|
| 0.1 | **PredictionRecord 结构体**: `{predicted_vector, confidence, timestamp, context_hash}` | `core/nt_core_consciousness/vsa_tag.rs` | +30 | HTC (arXiv:2601.15778) |
| 0.2 | **OutcomeRecord 结构体**: `{actual_vector, success, error_category, latency}` | `core/nt_core_consciousness/vsa_tag.rs` | +25 | TRACE (arXiv:2604.05336) |
| 0.3 | **VsaTagged 扩展**: 添加 `prediction: Option<PredictionRecord>`, `outcome: Option<OutcomeRecord>` | `core/nt_core_consciousness/vsa_tag.rs` | +15 | AgentTrace (arXiv:2602.10133) |
| 0.4 | **Prediction hook in batch**: 在 `handle_decision_compress` 前调用 `predict_success()` → 写入 PredictionRecord | `consciousness.rs` | +40 | Mirror (arXiv:2604.19809) |
| 0.5 | **Outcome comparison in batch**: 在 batch 末尾比对 Prediction vs Outcome → 更新 CalibrationEngine | `consciousness.rs` | +30 | Metacognitive Harness (arXiv:2605.14186) |
| 0.6 | **CalibrationEngine 统一**: 合并 EpistemicSelfModel/ConfidenceCalibrator/EpistemicHonesty 调用为单入口 | `core/nt_core_experience/epistemic.rs` + `confidence_calibrator.rs` + `epistemic_honesty.rs` | +100 | Metacognitive Probe (arXiv:2605.09844) |
| 0.7 | **VSA FailureClustering step**: 基于 Hamming 相似度的失败聚类 (复用 RecurrenceDetector 机制) | `core/nt_core_experience/failure_trace.rs` | +120 | NeoSigma + ErrorProbe (arXiv:2604.17658) |
| 0.8 | **Cluster → PolicyRepair 短路**: 当簇失败率 > 50% + resolution_rate < 0.3 直接触发 policy_repair | `consciousness.rs` | +35 | AutoTriage |

### P1 🟡 — 自我分析层 (模式提取 + 日志重放)

| # | 任务 | 涉及文件 | 预估行数 | 论文驱动 |
|---|------|---------|:--------:|---------|
| 1.1 | **DreamConsolidator 真实喂入**: 将 batch 决策结果作为 patterns 喂入 (停止传 `&[]`) | `run.rs`:273 | +20 | CMU Reasoning Primitives (arXiv:2606.02994) |
| 1.2 | **两步诱导 pass**: categorize+merge (VSA 聚类同类 thinking)，再 synthesize (规范形式) → 注入 SkillAccumulator | `core/nt_core_experience/dream.rs` + `skill_acc.rs` | +150 | CMU + MARS (arXiv:2601.11974) |
| 1.3 | **Transcript analysis handler**: 每 50 cycles 从 Event Log 提取 recurring 模式 | `consciousness.rs` | +80 | Trajectory Intelligence (arXiv:2603.10600) |
| 1.4 | **PredictionOutcomeStore**: 可重放 prediction/outcome 对，每 500 cycles 生成分析报告 | `core/nt_core_consciousness/stream_buffer.rs` | +90 | ActiveGraph (arXiv:2605.21997) |
| 1.5 | **Checkpoint 自动输出**: 每 500 cycles 输出 prediction/outcome 数据集到磁盘 | `self_iterating/checkpoint.rs` | +40 | agentic-experiments / aexp |

### P2 🟢 — 元层反思 (双层元认知)

| # | 任务 | 涉及文件 | 预估行数 | 论文驱动 |
|---|------|---------|:--------:|---------|
| 2.1 | **内环指标收集器**: 每 cycle 读取 ECE/meta-d'/M-ratio/calibration_error，维护 100-cycle rolling | `consciousness.rs` | +60 | Bilevel Autoresearch (arXiv:2603.23420) |
| 2.2 | **外环报告生成器**: 每 200 cycles 对比前后 100 cycles 指标变化，输出趋势报告 | `core/nt_core_meta/metacognition_loop.rs` | +120 | HyperAgents (arXiv:2603.19461) |
| 2.3 | **Belief Trajectory 输出**: 每 100 cycles 生成 trajectory note (Self-Archaeology 模式) | `consciousness.rs` | +70 | Self-Archaeology (Agent Patterns) |
| 2.4 | **DGM-H 元层链路**: MetaCognitiveLoop 的改进建议输出 → DGM-H writeback buffer | `consciousness.rs` + `brain_dgm.rs` | +50 | DGM-H (Meta, 2026) |
| 2.5 | **技能级趋势暴露**: ExperienceStats 增加 `calibration_trend` / `meta_d_prime` / `m_ratio` / `failure_cluster_count` 字段 | `consciousness.rs` | +40 | — |

---

### Done criteria (编译 + 审计)

```
cargo check -p neotrix --lib = 0 errors ✅
cargo check -p neotrix --lib --tests = 0 errors ✅

四维审计:
  - 编译断裂: 0 ❌
  - panic 风险: 0 unwrap/expect in production code
  - 死代码: 0 #[allow(dead_code)] / 0 装饰 handler
  - 装饰缺口: 每个新字段在 new()/handler/stats/run.rs 四处接线
```

---

## 路线图 (分阶段)

```
Phase 1 (P0.1-P0.6): Prediction-Before-Execution 环     ← 最高 ROI, ~240 行
  ├── 新增 PredictionRecord/OutcomeRecord 结构体
  ├── 扩展 VsaTagged schema
  ├── 统一 CalibrationEngine
  ├── 接线 pre-prediction hook + post-outcome comparison
  └── Done: "每次决策前自动预测, 执行后自动校准"

Phase 2 (P0.7-P0.8): VSA Failure Clustering               ← ~155 行
  ├── 复用 RecurrenceDetector 做缺陷聚类
  ├── string match → VSA Hamming similarity
  └── Cluster → PolicyRepair 短路

Phase 3 (P1.1-P1.3): 自我转录分析                          ← ~250 行
  ├── DreamConsolidator 真实喂入
  ├── 两步诱导 pass (CMU + MARS)
  └── Transcript analysis handler

Phase 4 (P1.4-P1.5 + P2.1-P2.5): 元层反思 + 持久化        ← ~420 行
  ├── PredictionOutcomeStore + Checkpoint
  ├── 内环指标 + 外环报告
  ├── Belief Trajectory
  └── DGM-H 元层链路
```

---

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| Prediction hook 增加 batch 延迟 | Prediction 是 O(1) VSA 相似度查表，无新增 LLM 调用 |
| Failure clustering O(n²/2) 随失败数增长 | 设置 max_items=200，超出后随机采样；复用已存在的 RecurrenceDetector guard |
| CalibrationEngine 合并可能破坏现有 stats 接口 | 保持旧方法委托到新入口，后向兼容 |
| Belief Trajectory 文件膨胀 | 每 100 cycles 只写平均指标，检查点覆盖旧轨迹 |
| DGM-H 元层链路默认关闭 | 安全门保持默认 false，人工确认后开启 |

---

## 2026-06-13 更新: 互联网深度研究对进化计划的影响

### 全局优先级重排 (基于 Anthropic + DGM 实证)

```
新优先级: Stage 0 种子 > Prediction-Before-Execution > KROP > 其他
理由: RSI 已从理论变为实证, 自举加速是关键路径
```

### 受影响的任务

| 原计划 | 新优先级 | 理由 |
|--------|:--------:|------|
| Stage 0 种子 (Ne→Rust 编译器) | 🔴 **P0** | RSI 实证使自举加速成为唯一关键路径 |
| P0.1-P0.6 预测环 | 🔴 **P0** | 不变, 仍然是最高 ROI 的意识改进 |
| KROP cleanup | 🟡 降级 | 性能优化可以等, 先解锁自举能力 |
| A2A v1.2 升级 | 🟡 **P2→P1** | 标准已锁定, 桥接模式不可持续 |

### 新增任务 (源自 v3 差距分析)

| # | 任务 | 优先级 | 论文依据 |
|---|------|:------:|---------|
| 3.1 | **RSI 指标追踪**: code_auto_rate, engineer_multiplier, task_autonomy_hours | P0 | Anthropic RSI (June 2026) |
| 3.2 | **Gödel Agent 原型**: arena 中添加自引用 self-modify agent | P0 | Gödel Agent (arXiv 2410.04444) + DGM (ICLR 2026) |
| 3.3 | **A2A v1.2 升级**: gRPC + signed Agent Cards | P1 | A2A v1.2 spec (Google I/O 2026) |
| 3.4 | **PCC proof-of-concept**: safety_gate 调用 Dafny/Z3 验证 | P1 | PC^3 (UC Davis ASE 2024) |
| 3.5 | **Sutra VSA 类型**: Ne Phase 2 IR 添加 VSA-native 类型 | P1 | Sutra (clawrxiv 2604.01542) |
| 3.6 | **Metagraph 运行时**: hypergraph.rs 扩展为原子重写引擎 | P1 | MeTTa/Hyperon (SingularityNET) |
| 3.7 | **Linear Code VSA**: 布尔域随机线性码编码 | P2 | MIT Neural Computation 36(6) 2024 |
| 3.8 | **GC-VSA**: 栅格细胞空间位置编码 | P2 | Krausse et al. 2025 |

### 更新后的 6 条进化轨道

```
Track A: 语言自举 (Ne)           ← 提升为最高优先级
  Stage 0 种子 (P0) → Gödel Agent 自引用 → PCC 证明

Track B: 意识工程 (5 启发点)      ← P0 不变
  P0.1-P0.6 预测环 → P0.7-P0.8 失败聚类 → 元反思

Track C: VSA 理论极限             ← 次级优先级
  KROP → 多头部谐振器 → 稀疏 VSA → 线性码 VSA

Track D: 自改进引擎               ← 提升为 P0（受RSI实证驱动）
  Gödel Agent → DGM 存档进化 → RSI 指标追踪

Track E: 信任与验证               ← 新增紧迫条目
  PC^3 PCC → 三源验证自动化 → 自动回滚

Track F: 生态互操作               ← 升级至 P1
  A2A v1.2 原生 gRPC → A2A 版本协商 → signed Agent Cards
```
