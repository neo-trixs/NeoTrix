# 进化迭代实施计划 — 完整分阶段实现说明

> 基于: EVOLUTION_PLAN.md (5 启发点 + 20+ 论文) + AI Agent 技术架构线路图 (2026 共识)

---

## Phase 0: Prediction-Before-Execution 环

**目标**: 闭合"决策前预测 → 决策后比对 → 校准更新"元认知循环
**ROI**: 🔴 最高 — 基础设施已存在，仅接线
**估量**: ~240 行新增，跨 5 个文件
**论文**: HTC arXiv:2601.15778, Mirror arXiv:2604.19809, Metacognitive Harness arXiv:2605.14186

### 任务 0.1 — PredictionRecord 结构体

**文件**: `core/nt_core_consciousness/vsa_tag.rs`

```rust
/// 决策前的预测记录
#[derive(Clone, Debug)]
pub struct PredictionRecord {
    pub predicted_vector: QuantizedVSA,    // 预测的结果 VSA
    pub confidence: f64,                   // 0.0-1.0 预测置信度
    pub timestamp_before: Instant,         // 预测时间
    pub context_hash: u64,                 // 当前上下文的确定性哈希
    pub domain: String,                    // 领域标签 (code/reasoning/planning...)
}
```

**引用**: HTC 论文的 "process-centric confidence": 每个预测绑定到上下文哈希，支持事后归因。

### 任务 0.2 — OutcomeRecord 结构体

**文件**: `core/nt_core_consciousness/vsa_tag.rs`

```rust
/// 决策后的结果记录
#[derive(Clone, Debug)]
pub struct OutcomeRecord {
    pub actual_vector: QuantizedVSA,          // 实际结果 VSA
    pub success: bool,                         // 二元成功/失败
    pub error_category: Option<String>,        // 失败类别 (空 = 成功)
    pub latency: Duration,                     // 决策执行耗时
    pub timestamp_after: Instant,              // 结果时间
}
```

### 任务 0.3 — VsaTagged 扩展

**文件**: `core/nt_core_consciousness/vsa_tag.rs`, 在 `VsaTagged` 结构体新增字段:

```rust
pub struct VsaTagged {
    // ... 现有字段不变 (vector, tag, confidence, timestamp, salience, provenance)
    pub prediction: Option<PredictionRecord>,  // 决策前的预测 (新增)
    pub outcome: Option<OutcomeRecord>,        // 决策后的结果 (新增)
}
```

**后向兼容**: 两个字段都是 `Option`，现有代码无需修改。

### 任务 0.4 — Prediction hook 注入

**文件**: `consciousness.rs` — 在 `handle_decision_compress` 前注入

```rust
/// 在决策压缩前执行预测 → 写入 PredictionRecord
pub fn handle_prediction_before_exec(&mut self, context: &[u8]) {
    if let Some(epistemic) = &self.epistemic_self_model {
        let domain = infer_domain_from_context(context);
        let predicted = epistemic.predict_success(context);
        let (best_guess, best_conf) = match predicted {
            (Some(v), c) => (v, c),
            _ => return,  // 无足够数据，跳过
        };
        let ctx_hash = simple_hash(context);
        let record = PredictionRecord {
            predicted_vector: best_guess,
            confidence: best_conf,
            timestamp_before: Instant::now(),
            context_hash: ctx_hash,
            domain,
        };
        self.pending_prediction = Some(record);  // 暂存，等 outcome
    }
}
```

**设计决策**: 不直接用 `predict_success` 的输出写日志，而是暂存到 `pending_prediction` 字段，等待 outcome 回来后组合写入。

### 任务 0.5 — Outcome comparison hook

**文件**: `consciousness.rs` — 每次 batch 末尾，所有 handler 执行后注入

```rust
/// 比对 prediction vs outcome → 更新校准器
pub fn handle_outcome_comparison(&mut self) {
    if let Some(pred) = self.pending_prediction.take() {
        // 从最近一次 decision 结果获取 outcome
        let outcome = OutcomeRecord {
            actual_vector: self.last_decision_outcome.clone(),
            success: self.last_decision_success,
            error_category: self.last_decision_error.clone(),
            latency: self.last_decision_latency,
            timestamp_after: Instant::now(),
        };
        // 装填到 stream_buffer
        if let Some(entry) = self.stream_buffer.last_mut() {
            entry.prediction = Some(pred.clone());
            entry.outcome = Some(outcome.clone());
        }
        // 更新三校准器
        if let Some(cal) = &mut self.confidence_calibrator {
            cal.record_prediction(pred.confidence, outcome.success);
        }
        if let Some(epi) = &mut self.epistemic_self_model {
            epi.calibrate(&pred.domain, pred.confidence, outcome.success as u32);
        }
        if let Some(honest) = &mut self.epistemic_honesty {
            honest.calibrate(pred.confidence, outcome.success);
        }
    }
}
```

### 任务 0.6 — CalibrationEngine 统一

**文件**: `core/nt_core_experience/calibration_engine.rs` (新建)

```rust
/// 统一三校准器: EpistemicSelfModel + ConfidenceCalibrator + EpistemicHonesty
pub struct CalibrationEngine {
    pub epistemic: EpistemicSelfModel,
    pub confidence: ConfidenceCalibrator,
    pub honesty: EpistemicHonesty,
    // 聚合指标缓存 (避免每 cycle 重复计算)
    pub last_ece: f64,
    pub last_meta_d: f64,
    pub last_m_ratio: f64,
    pub last_calibration_error: f64,
}
impl CalibrationEngine {
    pub fn record_prediction_outcome(&mut self, predicted: f64, correct: bool, domain: &str) { ... }
    pub fn aggregated_confidence(&self, raw: f64) -> f64 { ... }
    pub fn stats(&self) -> CalibrationStats { ... }
}
```

**后向兼容**: 所有旧方法保持对外委托。`consciousness.rs` 的 CI 字段改为 `CalibrationEngine`，通过 `.epistemic` / `.confidence` / `.honesty` 访问旧方法。

---

## Phase 1: VSA Failure Clustering

**目标**: 用 VSA 语义相似度替代 string match 分组失败 → Cluster → PolicyRepair 短路
**ROI**: 🔴 高 — 打开"盯输出"可操作化的入口
**估量**: ~155 行新增，跨 2 个文件
**论文**: NeoSigma Self-Improving, ErrorProbe arXiv:2604.17658, TRACE arXiv:2604.05336

### 任务 1.1 — VsaFailureCluster 结构体

**文件**: `core/nt_core_experience/failure_trace.rs`

```rust
/// VSA 语义失败簇
#[derive(Clone, Debug)]
pub struct VsaFailureCluster {
    pub cluster_id: u64,                    // 确定性哈希簇 ID
    pub prototype: Vec<u8>,                 // 簇原型 VSA (平均/多数投票)
    pub member_indices: Vec<usize>,         // 在全局失败列表中的索引
    pub member_count: usize,
    pub success_count: usize,               // 簇内重试后的成功次数
    pub failure_rate: f64,                  // 簇成员失败率
    pub resolution_rate: f64,               // 修复成功率
    pub domain: String,                     // 推断领域
    pub first_seen: u64,                    // 首次出现 cycle
    pub last_seen: u64,                     // 最近出现 cycle
}
```

### 任务 1.2 — failure_clustering() 方法

**文件**: `core/nt_core_experience/failure_trace.rs`

```rust
impl ExplorationGraph {
    /// VSA Hamming 相似度聚类失败, 复用 RecurrenceDetector 机制
    pub fn failure_clustering(&self, threshold: f64) -> Vec<VsaFailureCluster> {
        let failures: Vec<_> = self.failures.iter().filter(|f| !f.resolved).collect();
        if failures.len() < 3 { return vec![]; }
        
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; failures.len()];
        
        for i in 0..failures.len() {
            if assigned[i] { continue; }
            let mut cluster = vec![i];
            assigned[i] = true;
            for j in (i+1)..failures.len() {
                if assigned[j] { continue; }
                let sim = hamming_similarity(
                    &failures[i].outcome_vector, 
                    &failures[j].outcome_vector
                );
                if sim >= threshold { cluster.push(j); assigned[j] = true; }
            }
            if cluster.len() >= 3 { clusters.push(cluster); }
        }
        // ... 转换为 VsaFailureCluster 向量
    }
}
```

**threshold = 0.78**: 与 RecurrenceDetector 一致，复用已验证的阈值。
**min_samples = 3**: 最小有意义的簇大小。

### 任务 1.3 — handle_failure_clustering() handler

**文件**: `consciousness.rs`

```rust
/// 每 5 周期执行: VSA 聚类失败 → 识别最大失败堆 → 短路 PolicyRepair
pub fn handle_failure_clustering(&mut self) {
    let graph = match &self.exploration_graph { Some(g) => g, None => return };
    let clusters = graph.failure_clustering(0.78);  // VSA clustering
    
    self.failure_cluster_count = clusters.len();
    
    for cl in &clusters {
        if cl.failure_rate > 0.5 && cl.resolution_rate < 0.3 {
            // Cluster → PolicyRepair 短路: 跳过 normal handler
            // 注入高优先级修复任务
            if let Some(repair) = &mut self.policy_repair_engine {
                repair.repair_cluster(cl.cluster_id, &cl.prototype, cl.domain.clone());
            }
        }
        // 记录到 Event Log
        if let Some(entry) = self.stream_buffer.last_mut() {
            entry.outcome.as_mut().map(|o| {
                o.error_category = Some(format!("cluster:{}", cl.cluster_id))
            });
        }
    }
}
```

---

## Phase 2: Workstream.md 人类可读记忆导出

**目标**: 将 VSA 内存中的决策状态 → Markdown 文件导出，补齐"持久化记忆"的最后一块拼图
**ROI**: 🔴 高 — 跨 session 人类可读，与现有 VSA 管线互补
**估量**: ~120 行, 1 个新文件
**参照**: Anthropic Workstream.md, Trellis spec, Self-Archaeology Pattern

### 任务 2.1 — WorkstreamExporter 模块

**文件**: `core/nt_core_consciousness/workstream_exporter.rs` (新建)

```rust
/// VSA 记忆 → Markdown 导出器
pub struct WorkstreamExporter {
    pub export_path: PathBuf,          // 默认 .neotrix/workstream.md
    pub last_export_cycle: u64,
    pub export_interval: u64,          // 每 10 cycle 写一次
}

impl WorkstreamExporter {
    /// 从 ConsciousnessIntegration 状态合成 Markdown
    pub fn export(&self, ci: &ConsciousnessIntegration) -> String {
        let mut md = String::new();
        
        // 1. 当前目标 — 从 GoalDirectedExecution 获取
        md.push_str("# Workstream\n\n");
        md.push_str("## Active Goal\n");
        if let Some(goal) = &ci.goal_execution {
            md.push_str(&format!("- **Goal**: {}\n", goal.current_goal()));
            md.push_str(&format!("- **Status**: {}\n", goal.status_str()));
            md.push_str(&format!("- **Progress**: {:.1}%\n", goal.progress_pct() * 100.0));
        }
        
        // 2. 最近决策 — 从 StreamBuffer 最后 5 条提取
        md.push_str("\n## Recent Decisions\n");
        for entry in ci.stream_buffer.iter().rev().take(5) {
            let outcome = match &entry.outcome {
                Some(o) if o.success => "✅",
                Some(_) => "❌",
                None => "⏳",
            };
            let domain = entry.prediction.as_ref()
                .map(|p| &p.domain)
                .unwrap_or(&"unknown".to_string());
            md.push_str(&format!("- {} [{}] ", outcome, domain));
            if let Some(p) = &entry.prediction {
                md.push_str(&format!("conf={:.2} ", p.confidence));
            }
            md.push_str("\n");
        }
        
        // 3. 阻塞点 — 从 StormBreaker / SelfHealing 状态获取
        md.push_str("\n## Blockers\n");
        // ...
        
        // 4. 失败簇 — 从 failure_clusters 获取
        md.push_str("\n## Failure Clusters\n");
        // ...
        
        md
    }
    
    /// 写文件 (原子写入: 先写临时文件再 rename)
    pub fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
        let tmp = path.with_extension(".md.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(tmp, path)?;
        Ok(())
    }
}
```

### 任务 2.2 — handle_workstream_export() handler

```rust
pub fn handle_workstream_export(&mut self) {
    if self.cycle_count % self.workstream_exporter.export_interval != 0 { return; }
    let md = self.workstream_exporter.export(self);
    let path = self.workstream_exporter.export_path.clone();
    if let Err(e) = self.workstream_exporter.write(&path, &md) {
        eprintln!("[workstream] write error: {}", e);
    }
}
```

**设计决策**: 不替代 VSA 记忆，只做**周期性快照导出**。VSA 是 main memory，Markdown 是 checkpoint。

---

## Phase 3: ToolContract 工具契约统一层

**目标**: 统一工具调用的 Schema 验证 + 权限策略 + 结果边界
**ROI**: 🔴 高 — 补齐 Fable 5 安全架构的最后一块
**估量**: ~250 行, 1 个新文件
**参照**: Anthropic Managed Agents 工具契约, Fable 5 系统卡

### 任务 3.1 — ToolContract 核心定义

**文件**: `nt_shield/tool_contract.rs` (新建)

```rust
/// 工具契约: Schema + 权限 + 边界 + 审计
pub struct ToolContract {
    pub tool_name: String,
    pub input_schema: serde_json::Value,       // JSON Schema 验证
    pub permission_policy: PermissionPolicy,    // 沙箱/写入/网络权限
    pub output_limits: OutputLimits,            // 大小/深度/超时
    pub rate_limit: RateLimit,                  // 频率控制
    pub cost_budget: CostBudget,                // Token/计费上限
}

pub enum PermissionPolicy {
    ReadOnly,                                    // 只读, 无副作用
    Sandboxed { allowed_paths: Vec<PathBuf> },   // 受限写入
    Unrestricted,                                // 管理员
}

pub struct OutputLimits {
    pub max_bytes: usize,                        // 默认 1MB
    pub max_depth: usize,                        // 嵌套深度
    pub timeout_ms: u64,                         // 超时
}

pub struct AuditRecord {
    pub timestamp: Instant,
    pub tool_name: String,
    pub input_hash: u64,
    pub output_size: usize,
    pub permission_violation: bool,
    pub duration_ms: u64,
}
```

### 任务 3.2 — ToolContractValidator

```rust
/// 契约验证器: 在执行前检查
pub struct ToolContractValidator {
    pub contracts: HashMap<String, ToolContract>,
    pub audit_log: VecDeque<AuditRecord>,       // 环形缓冲, 容量 1000
}

impl ToolContractValidator {
    pub fn validate_call(&mut self, tool: &str, input: &[u8]) -> Result<(), ContractViolation> {
        let contract = self.contracts.get(tool).ok_or(ContractViolation::UnknownTool)?;
        // 1. Schema 验证
        // 2. 权限检查
        // 3. 速率检查
        // 4. 预算检查
        Ok(())
    }
    
    pub fn record_audit(&mut self, record: AuditRecord) {
        self.audit_log.push_back(record);
        if self.audit_log.len() > 1000 { self.audit_log.pop_front(); }
    }
}
```

### 任务 3.3 — 集成点

- `consciousness.rs` 新增 `tool_contracts: ToolContractValidator` 字段
- 在 `handle_thinking` 或 `handle_goal_execution` 中调用 `validate_call()`
- `codex_flow.rs` 的现有验证逻辑委托到 `ToolContractValidator`

---

## Phase 4: 自我转录分析

**目标**: 停止向 DreamConsolidator 喂空数据 → 真实喂入 batch 决策 patterns → 两步诱导推理基元
**ROI**: 🟡 中 — 模块已存在仅需接线
**估量**: ~250 行, 跨 3 个文件
**论文**: CMU Reasoning Primitives arXiv:2606.02994, MARS arXiv:2601.11974

### 任务 4.1 — DreamConsolidator 真实喂入

**文件**: `run.rs:273`

```rust
// 旧: handle_dream_consolidate_feed("bg", &[])
// 新:
let patterns = extract_patterns_from_recent_decisions(&ci);
ci.handle_dream_consolidate_feed("bg", &patterns);
```

**辅助函数**:

```rust
fn extract_patterns_from_recent_decisions(ci: &ConsciousnessIntegration) -> Vec<String> {
    ci.stream_buffer.iter()
        .filter(|e| e.outcome.is_some())
        .take(10)
        .map(|e| {
            let domain = e.prediction.as_ref().map(|p| &p.domain).unwrap_or(&"unknown");
            let outcome = if e.outcome.as_ref().map(|o| o.success).unwrap_or(false) { "ok" } else { "fail" };
            format!("[{}] decision_{}", domain, outcome)
        })
        .collect()
}
```

### 任务 4.2 — 两步诱导 pass (CMU 风格)

**文件**: `core/nt_core_experience/dream.rs` — 新增方法

```rust
impl DreamConsolidator {
    /// 两步诱导: categorize+merge → synthesize → inject SkillAccumulator
    pub fn induce_primitives(&mut self, recent_patterns: &[String]) -> Vec<InducedPrimitive> {
        // Step 1: VSA 聚类同类 thinking pattern (categorize+merge)
        let mut clusters: HashMap<u64, Vec<String>> = HashMap::new();
        for p in recent_patterns {
            let key = simple_hash(p);  // 简化版: 实际用 VSA similarity
            clusters.entry(key).or_default().push(p.clone());
        }
        
        // Step 2: 从聚类合成规范形式 (synthesize)
        let mut primitives = Vec::new();
        for (key, members) in &clusters {
            if members.len() < 2 { continue; }
            primitives.push(InducedPrimitive {
                id: *key,
                name: synthesize_name(members),
                usage_count: members.len(),
                success_rate: 0.0,  // 需后续 outcomes 更新
                docstring: synthesize_docstring(members),
            });
        }
        primitives
    }
}
```

### 任务 4.3 — Transcript analysis handler

**文件**: `consciousness.rs`

```rust
/// 每 50 cycles 从 Event Log 提取 recurring 模式 → 注入 SkillAccumulator
pub fn handle_transcript_analysis(&mut self) {
    if self.cycle_count % 50 != 0 || self.cycle_count == 0 { return; }
    
    let patterns: Vec<String> = self.stream_buffer.iter()
        .filter(|e| e.outcome.is_some())
        .map(|e| format!("{:?}:{}", e.tag, e.outcome.as_ref().map(|o| o.success).unwrap_or(false)))
        .collect();
    
    if patterns.is_empty() { return; }
    
    // 两步诱导
    let primitives = self.dream_consolidator.induce_primitives(&patterns);
    
    // 注入 SkillAccumulator 作为 VSA skill
    for prim in &primitives {
        if prim.usage_count >= 3 && prim.success_rate >= 0.5 {
            if let Some(skill) = &mut self.skill_accumulator {
                // 将规范形式编码为 VSA skill
                let v = self.ngram_encoder.encode_text(&prim.docstring);
                skill.accumulate(v, prim.success_rate, &prim.name);
            }
        }
    }
}
```

---

## Phase 5: LFD 损失函数接口包装

**目标**: 将现有指标 (ECE/meta-d'/pass_rate/failure_rate) 包装为统一的 `LossFunction::eval()` 接口
**ROI**: 🟡 中 — 主要重构, 但启用量化"改进了多少"
**估量**: ~150 行, 1 个新文件

### 任务 5.1 — LossFunction trait + GoalLossFunction

**文件**: `core/nt_core_decision/loss_function.rs` (新建)

```rust
/// 损失函数: 将 outcome 映射到可优化标量
pub trait LossFunction: Send {
    fn name(&self) -> &str;
    fn eval(&mut self, prediction: &PredictionRecord, outcome: &OutcomeRecord) -> f64;
    fn trend(&self) -> f64;    // 近 N 步的斜率: 负 = 改进
}

pub struct CompositeLoss {
    pub ece_weight: f64,
    pub failure_cluster_weight: f64,
    pub pass_rate_weight: f64,
    pub calibration_error_weight: f64,
    // 从外部校准器读取
    pub calibration_engine: Option<Arc<Mutex<CalibrationEngine>>>,
}

impl LossFunction for CompositeLoss {
    fn eval(&mut self, prediction: &PredictionRecord, outcome: &OutcomeRecord) -> f64 {
        let mut loss = 0.0;
        // 1. ECE (Expected Calibration Error): 高 ECE = 高 loss
        if let Some(cal) = &self.calibration_engine {
            let cal = cal.lock().unwrap();
            loss += self.ece_weight * cal.last_ece;
            loss += self.calibration_error_weight * cal.last_calibration_error;
        }
        // 2. 失败惩罚
        if !outcome.success {
            loss += 0.5;  // 失败基础惩罚
        }
        // 3. 预测置信度惩罚 (过度自信 = 高 loss)
        if outcome.success {
            loss += self.ece_weight * (1.0 - prediction.confidence).max(0.0);
        } else {
            loss += self.ece_weight * prediction.confidence;  // 自信但失败
        }
        loss
    }
    
    fn trend(&self) -> f64 {
        // 委托到 CalibrationEngine 的趋势检测
        0.0  // placeholder
    }
}
```

### 任务 5.2 — 集成

- `GoalDirectedExecution` 添加 `loss_fn: Box<dyn LossFunction>` 字段
- `evaluate_outcome()` 内部调用 `loss_fn.eval(prediction, outcome)`
- 暴露 `loss_trend` 到 `ExperienceStats`

---

## Phase 6: 视觉验证 (Perceptual Hash 截图比较)

**目标**: 不依赖 LLM 的多模态自检 — perceptual hash diff + 异常检测
**ROI**: 🟡 中 — 补齐闭环, 但依赖截图源可用
**估量**: ~200 行, 1 个新文件
**参照**: Karpathy "stare at outputs" + Fable 5 visual verification

### 任务 6.1 — VisualVerifier 模块

**文件**: `nt_shield/visual_verifier.rs` (新建)

```rust
/// 轻量视觉验证器: 不依赖 LLM, 纯 perceptual hash
pub struct VisualVerifier {
    pub expected_hashes: VecDeque<(u64, String)>,  // (hash, description)
    pub threshold: f64,                             // 默认 0.9 (90% 相似)
}

impl VisualVerifier {
    /// 计算 perceptual hash (差异哈希, 64 bit)
    pub fn dhash(img: &[u8], width: u32, height: u32) -> u64 {
        // 缩小到 9x8 → 比较相邻像素 → 64 bit hash
        let mut hash = 0u64;
        for y in 0..8 {
            for x in 0..8 {
                let left = pixel_at(img, width, x, y);
                let right = pixel_at(img, width, x + 1, y);
                if left > right { hash |= 1 << (y * 8 + x); }
            }
        }
        hash
    }
    
    /// 验证: 实际截图 vs 期望结果
    pub fn verify(&self, actual: &[u8], expected_idx: usize) -> (bool, f64) {
        if let Some((expected_hash, _)) = self.expected_hashes.get(expected_idx) {
            let actual_hash = Self::dhash(actual, 0, 0);
            let similarity = 1.0 - (actual_hash ^ expected_hash).count_ones() as f64 / 64.0;
            (similarity >= self.threshold, similarity)
        } else {
            (false, 0.0)
        }
    }
}
```

**设计决策**: 用 dHash (差异哈希) 而非更复杂的 pHash/aHash。dHash 对亮度变化鲁棒，计算 O(n)，适合每 cycle 调用。

### 任务 6.2 — 集成

- `consciousness.rs` 新增 `visual_verifier: Option<VisualVerifier>` 字段
- 在 `handle_goal_execution` 末尾，当有截图源时调用 `visual_verifier.verify()`
- 结果写入 `OutcomeRecord.actual_vector` (编码 similarity 分数)

---

## Phase 7: 双层元反思

**目标**: 闭包内环 (per-cycle 指标) + 外环 (跨周期趋势) 的双层反思
**ROI**: 🟢 低 — 价值随数据量增长, 初始可推迟
**估量**: ~420 行, 跨 4 个文件
**论文**: Bilevel Autoresearch arXiv:2603.23420, HyperAgents arXiv:2603.19461, Self-Archaeology

### 任务 7.1 — 内环指标收集器

```rust
// consciousness.rs — 在 handle_outcome_comparison 末尾追加
pub fn collect_meta_metrics(&mut self) {
    let metrics = MetaMetrics {
        ece: self.calibration_engine.last_ece,
        meta_d: self.calibration_engine.last_meta_d,
        m_ratio: self.calibration_engine.last_m_ratio,
        calibration_error: self.calibration_engine.last_calibration_error,
        failure_count: self.failure_cluster_count,
        cycle: self.cycle_count,
    };
    self.meta_metrics_ring.push(metrics);
    if self.meta_metrics_ring.len() > 100 { self.meta_metrics_ring.pop_front(); }
}
```

### 任务 7.2 — 外环报告生成器

```rust
// metacognition_loop.rs — 新增 report_trend() 方法
pub fn report_trend(&self, ring: &VecDeque<MetaMetrics>, window: usize) -> MetaReport {
    let recent: Vec<_> = ring.iter().rev().take(window).cloned().collect();
    if recent.len() < 2 { return MetaReport::default(); }
    
    let first = recent.last().unwrap();
    let last = recent.first().unwrap();
    
    MetaReport {
        ece_trend: last.ece - first.ece,     // 负 = 改进
        meta_d_trend: last.meta_d - first.meta_d,
        m_ratio_trend: last.m_ratio - first.m_ratio,
        calibration_trend: last.calibration_error - first.calibration_error,
        failure_trend: last.failure_count as f64 - first.failure_count as f64,
        period_cycles: window as u64,
        interpretation: self.interpret(&recent),
    }
}
```

### 任务 7.3 — Belief Trajectory 输出

```rust
// consciousness.rs — 每 100 cycles
pub fn write_belief_trajectory(&mut self, path: &Path) {
    if self.cycle_count % 100 != 0 { return; }
    let report = self.meta_cognition.report_trend(&self.meta_metrics_ring, 50);
    let mut md = String::new();
    md.push_str(&format!("# Belief Trajectory at Cycle {}\n\n", self.cycle_count));
    md.push_str(&format!("- **ECE**: {:.4} (trend: {:.4})\n", 
        report.last_ece(), report.ece_trend));
    md.push_str(&format!("- **meta-d'**: {:.4} (trend: {:.4})\n",
        report.last_meta_d(), report.meta_d_trend));
    md.push_str(&format!("- **M-ratio**: {:.4} (trend: {:.4})\n",
        report.last_m_ratio(), report.m_ratio_trend));
    md.push_str(&format!("- **Failure Clusters**: {} (trend: {:.0})\n\n",
        report.last_failures(), report.failure_trend));
    md.push_str(&format!("**Interpretation**: {}\n", report.interpretation));
    std::fs::write(path, &md).ok();
}
```

---

## Phase 8: 编译验证 + 四维审计

**目标**: 确保所有变更通过编译 + 审计
**命令**:

```bash
cargo check -p neotrix --lib
cargo check -p neotrix --lib --tests

# 四维审计脚本
rg -n "\.unwrap\(\)" neotrix-core/src/core/nt_core_consciousness/ | grep -v "#\[cfg(test)\]" | grep -v "#\[test\]"
rg -n "#\[allow\(dead_code\)\]" neotrix-core/src/core/
rg -n "todo!" neotrix-core/src/core/
```

**验收标准**:
1. `cargo check -p neotrix --lib = 0 errors`
2. 新模块全有 `pub use` + mod.rs 注册
3. 无 new unwrap/expect in production paths
4. 无 new `#[allow(dead_code)]` / `todo!()`
5. 所有新字段在 `new()` / handler / stats / `run.rs` 四处接线

---

## 依赖图

```
Phase 0 (prediction loop)    ← 无前置依赖, 可独立开始
Phase 1 (failure clustering) ← 无前置依赖, 可独立开始
Phase 2 (workstream.md)      ← 依赖 Phase 0 (需要 outcome 字段)
Phase 3 (tool contract)      ← 无前置依赖, 可独立开始
Phase 4 (transcript analysis)← 依赖 Phase 0 (需要 prediction/outcome 喂入)
Phase 5 (LFD)                ← 依赖 Phase 0 + Phase 1 (需要指标输入)
Phase 6 (visual verify)      ← 无前置依赖, 可独立开始
Phase 7 (meta reflection)    ← 依赖 Phase 0 + Phase 1 + Phase 5
Phase 8 (compile + audit)    ← 依赖全部
```

独立零依赖的 Phase: **0, 1, 3, 6** — 可以并行开始。
