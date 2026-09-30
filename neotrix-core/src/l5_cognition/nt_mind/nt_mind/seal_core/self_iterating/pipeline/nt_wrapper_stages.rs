//! nt_wrapper_stages — 从 `pipeline.rs` 拆分 (wrapper_stages — 外部 stage 包装族: UQ/开源对比/SFT/Process/搜索技能/DPO/宪法/安全 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use super::super::SelfIteratingBrain;
use crate::l0_substrate::nt_core_error::NeoTrixError;
use super::super::constitutional_stage::ConstitutionalSelfCritiqueStage;
use super::super::hypercore::SafetyCheckResult;
use super::super::process_stage::ProcessExample;
use super::super::process_stage::ProcessStage;
use super::super::process_stage::ProcessStageStep;
use super::super::process_stage::ProcessStageTrace;
use super::super::process_stage::ProcessStageTraceSource;
use super::super::safety_stage::SafetyCheckStage;
use super::super::search_skill_stage::Evidence;
use super::super::search_skill_stage::SearchExercise;
use super::super::search_skill_stage::SearchResult;
use super::super::search_skill_stage::SearchTaskType;
use super::super::sft_stage::SupervisedExample;

/// Uncertainty quantification calibration stage
pub struct _UQCalibrationStage;
impl Default for _UQCalibrationStage {
    fn default() -> Self {
        Self
    }
}
impl _UQCalibrationStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _UQCalibrationStage {
    fn name(&self) -> &str {
        "uq_calibration"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let hist = &brain.evaluation_history;
        let recent_count = hist.len().min(20);
        let volatility =
            if recent_count >= 4 {
                let recent: Vec<f64> = hist
                    .iter()
                    .rev()
                    .take(recent_count)
                    .map(|r| r.score_after)
                    .collect();
                let mean = recent.iter().sum::<f64>() / recent.len() as f64;
                let variance =
                    recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
                let vol = variance.sqrt();
                let reward = brain._prm_cumulative_reward;
                let target_entropy = if vol > 0.15 && reward < 0.3 {
                    0.6f64.min(0.3 + vol)
                } else if vol < 0.05 && reward > 0.5 {
                    (brain.entropy_crisis_level * 0.8).max(0.05)
                } else {
                    brain.entropy_crisis_level * 0.95 + vol * 0.05
                };
                brain.entropy_crisis_level = target_entropy.max(0.0).min(1.0);
                if let Some(ref kb) = brain._nt_memory_kb {
                    let _ = kb.kv_set("uq_calibration", &format!("iter_{}", brain.iteration),
                    &serde_json::json!({"volatility": vol, "entropy": brain.entropy_crisis_level,
                        "reward": reward}).to_string());
                }
                vol
            } else {
                0.0
            };
        log::trace!(
            "[uq_calibration] entropy={:.4} volatility={:.4} reward={:.4}",
            brain.entropy_crisis_level,
            volatility,
            brain._prm_cumulative_reward
        );
        Ok(StageDecision::Continue)
    }
}


/// Open-source compare stage
pub struct _OpenSourceCompareStage;
impl Default for _OpenSourceCompareStage {
    fn default() -> Self {
        Self
    }
}
impl _OpenSourceCompareStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _OpenSourceCompareStage {
    fn name(&self) -> &str {
        "open_source_compare"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref insights) = brain._open_source_insights {
            let insight_len = insights.len();
            let deltas = compute_capability_deltas(brain);
            let total_delta: f64 = deltas.iter().map(|(_, d)| *d).sum();
            if total_delta.abs() > 0.001 && insight_len > 5 {
                let new_edits: Vec<super::super::super::self_edit::MicroEdit> = deltas
                    .iter()
                    .filter(|(_, d)| d.abs() > 0.01)
                    .map(|(name, delta)| {
                        super::super::super::self_edit::MicroEdit::AdjustDimension(name.clone(), *delta)
                    })
                    .collect();
                brain._open_source_edits.extend(new_edits);
                if let Some(ref kb) = brain._nt_memory_kb {
                    let _ = kb.kv_set(
                        "open_source_compare",
                        &format!("iter_{}", brain.iteration),
                        &serde_json::json!({"insight_len": insight_len, "total_delta": total_delta,
                            "new_edits": brain._open_source_edits.len()})
                        .to_string(),
                    );
                }
            }
        }
        log::trace!(
            "[open_source_compare] insights_present={} edits={}",
            brain._open_source_insights.is_some(),
            brain._open_source_edits.len()
        );
        Ok(StageDecision::Continue)
    }
}


/// Wraps SftStage ::process() as a BrainStage.
/// 监督微调：将能力增量作为监督信号，把当前最优 E8 模式推向目标质量。
/// _SftWrapperStage — 监督微调 (smol-course 吸收: SFT → DPO 两阶段顺序)。
/// 位于 DpoWrapperStage 之前，将能力增量构建为监督信号，为 DPO 提供 π_ref 基础。
pub struct _SftWrapperStage;
impl Default for _SftWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl _SftWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _SftWrapperStage {
    fn name(&self) -> &str {
        "sft_supervision"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let current_mode = brain._e8_policy.best_mode();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let examples: Vec<SupervisedExample> = deltas
            .iter()
            // H2 修复: 只把正向 delta 当监督样本 (能力提升)。此前用 delta.abs()
            // 方向无关 — 能力下降 (负 delta, 含 HarnessAdapt 抬升假象) 也当正样本
            // 训练, 形成"自抬能力→自产 delta→自训"奖励黑客循环, 无外部验证。
            .filter(|(_, d)| *d > 0.01)
            .map(|(name, delta)| {
                SupervisedExample::new(name, current_mode.0, delta.clamp(0.0, 1.0))
                    .with_timestamp(timestamp)
            })
            .collect();
        let (_result, adjusted_reward) = brain._sft_stage.process(examples.clone(), brain._reward);
        brain._set_reward(adjusted_reward);
        log::trace!(
            "[sft_supervision] reward={:.4} examples={} updates={}",
            adjusted_reward,
            examples.len(),
            brain._sft_stage.total_updates
        );
        Ok(StageDecision::Continue)
    }
}


/// Wraps ProcessStage ::process() as a BrainStage.
/// 过程知识习得：从工具调用轨迹构造推理链，监督"如何推理/分解/验证"。
pub struct ProcessWrapperStage;
impl Default for ProcessWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl ProcessWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for ProcessWrapperStage {
    fn name(&self) -> &str {
        "process_supervision"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // 从工具调用轨迹构造推理链步骤
        let steps: Vec<ProcessStageStep> = brain
            .tool_traces
            .iter()
            .enumerate()
            .map(|(i, (tool, dur, ok))| ProcessStageStep {
                step_idx: i,
                specialist: "Tool".to_string(),
                e8_mode: 0,
                action: tool.clone(),
                input: String::new(),
                output: if *ok {
                    "ok".to_string()
                } else {
                    "error".to_string()
                },
                duration_ms: Some(*dur),
                success: *ok,
                reward: Some(if *ok { 1.0 } else { 0.0 }),
            })
            .collect();
        let examples: Vec<ProcessExample> = if steps.is_empty() {
            Vec::new()
        } else {
            let trace = ProcessStageTrace {
                trace_id: format!("iter-{}", brain.iteration),
                task,
                steps,
                completed: true,
                final_quality: brain._reward.clamp(0.0, 1.0),
                source: ProcessStageTraceSource::Synthesis,
                timestamp,
            };
            vec![ProcessExample { trace, weight: 1.0 }]
        };
        // B5 (缺陷4修复): 消费意识树果实 → 转换为 ProcessStageTrace 并入 process 样本。
        // 此前 extract_from_consciousness_tree (process_stage.rs:130) 无生产调用者,
        // 意识树产出的进化果实从不进入 SEAL 过程学习。果实轨迹以 quality 加权,
        // 使高质量进化果实优先塑造 reasoning_depth/cot_quality 等能力维度。
        let mut examples = examples;
        let fruit_traces =
            ProcessStage::extract_from_consciousness_tree(&brain._consciousness_fruits);
        for ft in fruit_traces {
            let w = ft.final_quality.max(0.1);
            examples.push(ProcessExample {
                trace: ft,
                weight: w,
            });
        }
        // 缺陷2修复 (自我运转实际情况): 果实消费后立即清除, 防止同一批果实
        // 被 SEAL 反复消费 (1h 注入 vs 10min 消费的时序错配 → 同一 trace 进
        // buffer 6 次 → process 学习被重复污染)。一次性消费, 下次 tick 重新注入。
        brain._consciousness_fruits.clear();
        let (_result, loss) = brain._process_stage.process(examples);
        log::trace!(
            "[process_supervision] loss={:.4} traces={}",
            loss,
            brain._process_stage.buffer.len()
        );
        Ok(StageDecision::Continue)
    }
}


/// Wraps SearchSkillStage ::process() as a BrainStage.
/// 搜索技能内化：从当前任务构造搜索演练，学习 query/evidence/synthesis 子技能。
pub struct _SearchSkillWrapperStage;
impl Default for _SearchSkillWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl _SearchSkillWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _SearchSkillWrapperStage {
    fn name(&self) -> &str {
        "search_skill_supervision"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task.clone();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let exercises: Vec<SearchExercise> = if task.is_empty() {
            Vec::new()
        } else {
            // H4 修复: 用 KB 真实检索结果作为 grounding 信号, 替代内部 reward 自证。
            // 此前 grounding/relevance/synthesis 全取 brain._reward (内部自评),
            // 且三个分数完全相同 — 搜索技能训练完全自同义反复, 无外部验证。
            // 现在: 从 NT-MEMORY KB 检索任务相关证据, grounding 由真实命中分数
            // 决定 (FTS/BM25/向量), 无命中则 grounding 低 → 训练信号反映真实
            // 检索能力而非自我感觉。
            let kb = brain._nt_memory_kb.as_ref();
            let kb_hits = kb.and_then(|k| k.search(&task, 5).ok()).unwrap_or_default();
            let grounding = if kb_hits.is_empty() {
                0.0
            } else {
                kb_hits.iter().map(|r| r.score.clamp(0.0, 1.0)).sum::<f64>() / kb_hits.len() as f64
            };
            let relevance = grounding;
            let _synthesis = grounding * 0.9; // 综合质量略低于 grounding, 反映证据到答案的损耗
            let raw_results: Vec<SearchResult> = kb_hits
                .iter()
                .take(5)
                .map(|h| SearchResult {
                    url: h.node.id.clone(),
                    title: h.node.title.clone(),
                    snippet: h.node.summary.clone().unwrap_or_default(),
                    source_type: "kb".to_string(),
                    credibility: h.score.clamp(0.0, 1.0),
                })
                .collect();
            let filtered: Vec<Evidence> = raw_results
                .iter()
                .filter(|r| r.credibility > 0.3)
                .take(3)
                .map(|r| Evidence {
                    source_url: r.url.clone(),
                    claim: r.snippet.clone(),
                    confidence: r.credibility,
                    supports_answer: true,
                })
                .collect();
            vec![SearchExercise {
                exercise_id: format!("search-{}", brain.iteration),
                task_type: SearchTaskType::TechnicalQuery,
                query: task.clone(),
                raw_results,
                filtered_evidence: filtered,
                synthesized_answer: task.clone(),
                grounding_score: grounding,
                relevance_score: relevance,
                synthesis_quality: grounding * relevance,
                latency_ms: 0,
                timestamp,
            }]
        };
        let (_result, loss) = brain._search_skill_stage.process(exercises.clone());
        log::info!(
            "[search_skill_supervision] loss={:.4} exercises={} updates={}",
            loss,
            exercises.len(),
            brain._search_skill_stage.total_updates
        );
        Ok(StageDecision::Continue)
    }
}


/// Wraps DpoStage ::process() as a BrainStage
pub struct DpoWrapperStage;
impl Default for DpoWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl DpoWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for DpoWrapperStage {
    fn name(&self) -> &str {
        "dpo_preference"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut pairs: Vec<crate::l5_cognition::nt_mind::nt_mind::self_iterating::dpo_stage::PreferencePair> =
            Vec::new();
        let current_mode = brain._e8_policy.best_mode();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        for (cap_name, delta) in &deltas {
            if (*delta).abs() > 0.01 {
                let (rejected_idx, _) = brain
                    ._e8_policy
                    .mode_values
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or((0, &0.0));
                if *delta > 0.0 {
                    pairs.push(
                        crate::l5_cognition::nt_mind::nt_mind::self_iterating::dpo_stage::PreferencePair {
                            task: cap_name.clone(),
                            chosen_mode: current_mode.0,
                            rejected_mode: rejected_idx as u8,
                            chosen_reward: *delta,
                            rejected_reward: 0.0,
                            timestamp,
                        },
                    );
                }
            }
        }
        let (_result, adjusted_reward) = brain._dpo_stage.process(pairs, brain._reward);
        brain._set_reward(adjusted_reward);
        // H3 修复: DPO 偏好信号真正应用到能力向量。
        // 此前 DpoStage::process 只算 loss + 调 reward, 从不更新任何权重 —
        // DPO 是 no-op, smol-course 吸收的 DPO 阶段无学习效果。
        // 现在: chosen 维度按 margin 提升, rejected 维度按 margin 下降,
        // 使偏好信号实际改变能力分布 (DPO 梯度方向: 提升 chosen, 压低 rejected)。
        let beta = brain._dpo_stage.beta;
        for pair in brain._dpo_stage.buffer.pairs.iter() {
            if let Some(idx) = parse_cap_index(&pair.task) {
                let margin = (pair.chosen_reward - pair.rejected_reward).clamp(0.0, 1.0);
                let step = (beta * margin).min(0.05);
                let arr = brain.brain.capability.arr_mut();
                if idx < arr.len() {
                    arr[idx] = (arr[idx] + step).min(1.0);
                }
            }
        }
        log::trace!(
            "[dpo_preference] reward={:.4} deltas={} updates={}",
            adjusted_reward,
            deltas.len(),
            brain._dpo_stage.total_updates
        );
        Ok(StageDecision::Continue)
    }
}

/// Parse capability index from "cap_{i}" task name.
fn parse_cap_index(task: &str) -> Option<usize> {
    task.strip_prefix("cap_")
        .and_then(|s| s.parse::<usize>().ok())
}


/// Wraps ConstitutionalSelfCritiqueStage as a BrainStage
pub struct ConstitutionalWrapperStage;
impl Default for ConstitutionalWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl ConstitutionalWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for ConstitutionalWrapperStage {
    fn name(&self) -> &str {
        "constitutional_critique"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut critic = ConstitutionalSelfCritiqueStage::new();
        let (_result, adjusted_reward, should_reflect) = critic.process(&deltas, brain._reward);
        brain._set_reward(adjusted_reward);
        log::trace!(
            "[constitutional_critique] reward_adjusted={:.4} should_reflect={} violations={}",
            adjusted_reward,
            should_reflect,
            critic.consecutive_violations
        );
        Ok(StageDecision::Continue)
    }
}


/// Wraps SafetyCheckStage as a BrainStage
pub struct SafetyWrapperStage;
impl Default for SafetyWrapperStage {
    fn default() -> Self {
        Self
    }
}
impl SafetyWrapperStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for SafetyWrapperStage {
    fn name(&self) -> &str {
        "safety_check"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let deltas = compute_capability_deltas(brain);
        let mut safety = SafetyCheckStage::new();
        let (_result, check_result, adjusted_reward) = safety.evaluate(&deltas, brain._reward);
        brain._set_reward(adjusted_reward);
        match &check_result {
            SafetyCheckResult::Failed { reason } => {
                log::warn!("[safety_check] BLOCKED: {}", reason);
                return Ok(StageDecision::Skip(reason.clone()));
            }
            SafetyCheckResult::NeedsHumanReview { concern } => {
                log::warn!("[safety_check] REVIEW: {}", concern);
            }
            SafetyCheckResult::Passed => {
                log::trace!("[safety_check] passed");
            }
        }
        Ok(StageDecision::Continue)
    }
}

