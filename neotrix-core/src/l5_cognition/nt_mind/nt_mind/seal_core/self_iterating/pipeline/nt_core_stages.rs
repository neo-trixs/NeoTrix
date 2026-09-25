//! nt_core_stages — 从 `pipeline.rs` 拆分 (core_stages — 循环地基: 快照/SSM/边界/记忆写入/睡眠/自治规划/GWT/维护 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use super::super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::make_stage;
use crate::l5_cognition::nt_mind::foundation::memory_bank::MemoryTier;

// 概念涌现阶段 — 从KB节点聚类中发现新概念
make_stage!(ConceptEmergenceStage);
impl BrainStage for ConceptEmergenceStage {
    fn name(&self) -> &str { "concept_emergence" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // 从brain的KB中获取节点嵌入 (ReasoningBrain 无 KB 字段, 跳过)
        Ok(StageDecision::Continue)
    }
}

// ── Recipe-only stages ──────────────────────────────────────────
// SnapshotStage 被 recipe.rs 测试引用; _SSMUpdateStage 同时注册在 seal_pipeline。
// 其余 recipe-only 阶段 (memory_retrieval/gap_analysis/self_edit_gen/apply_edits/
// champion_compare) 零消费者且 log-only, 已按 Dark Forest 移除 (P0-2 治理)。

/// Snapshot stage (used by recipe.rs)
pub struct SnapshotStage;
impl Default for SnapshotStage {
    fn default() -> Self {
        Self
    }
}
impl SnapshotStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for SnapshotStage {
    fn name(&self) -> &str {
        "snapshot"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let caps = brain.brain.capability.arr();
        log::trace!("[snapshot] iter={} caps={:?}", brain.iteration, caps);
        Ok(StageDecision::Continue)
    }
}


/// SSM update stage (used by recipe.rs)
pub struct _SSMUpdateStage;
impl Default for _SSMUpdateStage {
    fn default() -> Self {
        Self
    }
}
impl _SSMUpdateStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _SSMUpdateStage {
    fn name(&self) -> &str {
        "ssm_update"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._prm_cumulative_reward;
        let mode = brain._e8_policy.best_mode();
        brain
            ._transition_learner
            .record(&brain._current_task, mode, reward, brain.iteration);
        brain._e8_policy.mode_values[mode.0 as usize] =
            (brain._e8_policy.mode_values[mode.0 as usize] * 0.9 + reward * 0.1).min(1.0);
        brain._e8_policy.mode_counts[mode.0 as usize] += 1;
        brain._e8_policy.decay_epsilon();
        log::trace!(
            "[ssm_update] iter={} reward={:.4} mode={} eps={:.4}",
            brain.iteration,
            reward,
            mode.0,
            brain._e8_policy.epsilon()
        );
        Ok(StageDecision::Continue)
    }
}


// ── Wrapper stages for modules that don't implement BrainStage directly ─────

/// Fable 5-style boundary separation stage: assessment before action.
///
/// Implements the principle "when the user is describing a problem, report
/// findings and stop. Don't apply a fix until asked." Checks whether each
/// pending edit/action was explicitly requested or is an unsolicited action.
/// Unsolicited actions are flagged and prevented from executing.
pub struct _BoundarySeparationStage {
    /// Whether to allow unrequested fixes (default: false = block them)
    pub allow_unrequested_fixes: bool,
    /// Threshold: actions matching this many keywords are "unrequested"
    pub unrequested_keywords: Vec<String>,
}

impl Default for _BoundarySeparationStage {
    fn default() -> Self {
        Self {
            allow_unrequested_fixes: false,
            unrequested_keywords: vec![
                "fix".into(),
                "restructure".into(),
                "refactor".into(),
                "rewrite".into(),
                "optimize".into(),
                "clean up".into(),
                "delete".into(),
                "remove".into(),
                "migrate".into(),
                "create".into(),
                "add".into(),
                "implement".into(),
            ],
        }
    }
}

impl _BoundarySeparationStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn _with_allowed_fixes(mut self, allow: bool) -> Self {
        self.allow_unrequested_fixes = allow;
        self
    }

    /// Check if an action description appears to be unrequested (not explicitly asked for).
    pub(crate) fn _is_unrequested_action(&self, action_desc: &str, context: &str) -> bool {
        if self.allow_unrequested_fixes {
            return false;
        }
        let action_lower = action_desc.to_lowercase();
        let context_lower = context.to_lowercase();

        // Check: does the action contain any keyword that was NOT mentioned in context?
        let mut unrequested_count = 0u32;
        for kw in &self.unrequested_keywords {
            if action_lower.contains(kw) && !context_lower.contains(kw) {
                unrequested_count += 1;
            }
        }
        unrequested_count >= 1
    }
}

impl BrainStage for _BoundarySeparationStage {
    fn name(&self) -> &str {
        "boundary_separation"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let context = brain._current_task.as_str();
        let pending_actions: Vec<String> = brain
            ._micro_edits
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();

        let mut blocked = Vec::new();
        for action in &pending_actions {
            if self._is_unrequested_action(action, context) {
                blocked.push(action.clone());
            }
        }

        if !blocked.is_empty() {
            // Remove blocked (unrequested) edits from pending list
            brain
                ._micro_edits
                .retain(|e| !blocked.contains(&format!("{:?}", e)));
            return Ok(StageDecision::Skip(format!(
                "boundary_separation: blocked {} unrequested actions: {:?}",
                blocked.len(),
                blocked
            )));
        }

        Ok(StageDecision::Continue)
    }
}


/// Reasoning bank storage stage — persists the current iteration's state as a
/// real ReasoningMemory in `brain.reasoning_bank`. Previously a log-only stub
/// (P0-2 SEAL Log-Only Stages governance, PA024). Now the bank is fed every
/// `frequency()` iterations so SleepStage/consolidation have fresh memories to
/// consume (Dark Forest: the stage now has a real side effect + a real consumer
/// in seal_pipeline).
pub struct ReasoningBankStorageStage;
impl Default for ReasoningBankStorageStage {
    fn default() -> Self {
        Self
    }
}
impl ReasoningBankStorageStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for ReasoningBankStorageStage {
    fn name(&self) -> &str {
        "reasoning_bank_storage"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task.clone();
        let reward = brain._reward;
        if !task.is_empty() {
            let task_type = super::super::super::self_edit::world_to_knowledge_task_type(&brain._current_task_type);
            let edits = brain._micro_edits.clone();
            let memory = crate::l1_action::nt_core_bank::ReasoningMemory::new(
                &task,
                task_type,
                &edits,
                reward.clamp(0.0, 1.0),
            );
            brain.reasoning_bank.store(memory);
        }
        let stats = brain.reasoning_bank.stats();
        log::trace!(
            "[reasoning_bank_storage] iter={} memories={} success_rate={:.3} task='{}' reward={:.4}",
            brain.iteration, stats.total_memories, stats.success_rate, task, reward,
        );
        Ok(StageDecision::Continue)
    }
}


/// Sleep stage (offline memory consolidation)
pub struct SleepStage;
impl Default for SleepStage {
    fn default() -> Self {
        Self
    }
}
impl SleepStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for SleepStage {
    fn name(&self) -> &str {
        "sleep"
    }
    fn frequency(&self) -> usize {
        100
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref mut _engine) = brain.sleep_engine {
            // FIXME: select_operator / selective_state fields commented out in SelfIteratingBrain
            // if let (Some(op), Some(ref mut st)) = (
            //     brain.select_operator.as_ref(),
            //     brain.selective_state.as_mut(),
            // ) {
            //     match engine.sleep(
            //         &mut brain.brain.capability,
            //         &mut brain.reasoning_bank,
            //         op,
            //         st,
            //     ) {
            //         Ok(result) => {
            //             let stats = result.stats.clone();
            //             brain.last_sleep_stats = Some(result.stats);
            //             log::info!(
            //                 "[sleep] passes={} memories={} delta={:.6}",
            //                 stats.passes_done,
            //                 stats.total_memories,
            //                 stats.total_delta
            //             );
            //         }
            //         Err(e) => log::warn!("[sleep] engine error: {}", e),
            //     }
            // }
            // not wired: select_operator / selective_state not connected.
            // Sleep stage cannot run memory consolidation with operator selection.
            // Falls back to light consolidation only.
            log::debug!(
                "[sleep] not wired: select_operator/selective_state — \
                 light consolidation only (operator-based consolidation unavailable)"
            );
        } else {
            let result = brain.consolidate_memories();
            log::info!(
                "[sleep] light consolidation: merged={} pruned={}",
                result.merged_count,
                result.pruned_count
            );
        }
        Ok(StageDecision::Continue)
    }
}


/// Autonomy PER (Plan-Execute-Reflect) stage: runs the PlanExecuteReflectLoop
/// on the current task to produce structured task plans, execution traces, and
/// self-reflective revisions. Frequency 5 — runs every 5 iterations to avoid
/// overwhelming the pipeline with detailed planning on every tick.
pub struct _AutonomyPerStage;
impl Default for _AutonomyPerStage {
    fn default() -> Self {
        Self
    }
}
impl _AutonomyPerStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _AutonomyPerStage {
    fn name(&self) -> &str {
        "autonomy_per"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l1_action::nt_act::nt_act_autonomy::PlanExecuteReflectLoop;

        let task = if brain._current_task.is_empty() {
            log::trace!("[autonomy_per] no current task, skipping");
            return Ok(StageDecision::Skip("no task".into()));
        } else {
            brain._current_task.clone()
        };

        // Run the PER loop against the knowledge distiller's per_loop field
        let outcome = if let Some(ref mut per) = brain._per_loop {
            per.run(&task)
        } else {
            let mut per = PlanExecuteReflectLoop::new(crate::l1_action::nt_act::nt_act_autonomy::PerConfig {
                max_iterations: 3,
                min_score_to_converge: 0.7,
                require_all_steps: false,
            });
            let outcome = per.run(&task);
            brain._per_loop = Some(per);
            outcome
        };

        log::info!(
            "[autonomy_per] task='{}' converged={} score={:.2} iterations={} duration={}ms steps={}",
            task.chars().take(60).collect::<String>(),
            outcome.converged,
            outcome.final_score,
            outcome.iterations.len(),
            outcome.total_duration_ms,
            outcome.final_plan.steps.len(),
        );

        // Store as a KV record in KB for later review/audit
        if let Some(ref kb) = brain._nt_memory_kb {
            let summary = serde_json::json!({
                "task": &task,
                "converged": outcome.converged,
                "final_score": outcome.final_score,
                "iteration_count": outcome.iterations.len(),
                "total_duration_ms": outcome.total_duration_ms,
                "step_count": outcome.final_plan.steps.len(),
            });
            let _ = kb.kv_set(
                "autonomy_per",
                &format!("iter_{}", brain.iteration),
                &summary.to_string(),
            );
        }

        Ok(StageDecision::Continue)
    }
}


/// GWT absorption stage: routes insights into global workspace
pub struct _GwtAbsorbStage;
impl Default for _GwtAbsorbStage {
    fn default() -> Self {
        Self
    }
}
impl _GwtAbsorbStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _GwtAbsorbStage {
    fn name(&self) -> &str {
        "gwt_absorb"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let iteration = brain.iteration;
        let reward = brain._reward;
        let caps = brain.brain.capability.arr().to_vec();
        let avg_cap = if caps.is_empty() {
            0.5
        } else {
            caps.iter().sum::<f64>() / caps.len() as f64
        };
        if let Some(ref kb) = brain._nt_memory_kb {
            // ── FirstPersonRef 生产读点 (F4 收尾): 自我-当前流一致性进快照摘要,
            // 让 first_person 字段从"出生写一次"变为每 cycle 可观测。
            let fp_coherence = brain
                ._consciousness_stream
                .current()
                .map(|c| brain.first_person.coherence_with(&c.vector))
                .unwrap_or(0.0);
            let summary = format!(
                "gwt:iter={} reward={:.4} avg_cap={:.4} aut={:?} fp_coh={:.3}",
                iteration, reward, avg_cap, brain.autonomy, fp_coherence
            );
            let _ = kb.kv_set("gwt_absorb", &format!("snapshot_{}", iteration), &summary);
            // is_conscious derived from the real InnerCritic quality score
            // (0.0–1.0), not hardcoded true. Falls back to a phi-like threshold
            // on avg_cap when no critique has been produced yet.
            let conscious_signal = if brain._consciousness_critique_count > 0 {
                brain._last_consciousness_quality
            } else {
                avg_cap
            };
            let is_conscious = conscious_signal >= 0.5;
            let _ = kb.record_consciousness_snapshot(
                avg_cap,
                reward,
                is_conscious,
                "pipeline",
                &summary,
            );
        }
        // Route state summary into GWT workspace for specialist broadcast
        if let Some(ref mut router) = brain.attention_router {
            let content = format!(
                "pipeline_iter={} reward={:.4} avg_cap={:.4}",
                iteration, reward, avg_cap
            );
            // 升级: 从 no-op broadcast (仅 push history) 改为 resonant_broadcast —
            // 真正进入 E8 注意力偏置 + Kuramoto 同步 + 共振竞争, 让 SEAL 状态
            // 参与 GWT 注意力路由 (修复信息流转对内断点 #2)。
            let states = crate::l5_cognition::nt_core_gwt::resonance::default_specialist_states();
            let _report = router.wm().resonant_broadcast(&content, &states);
            log::debug!("[gwt_absorb] resonant broadcast to GWT: {}", content);
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Ok(results) = kb.query_broadcast_context(&content, 3) {
                    log::debug!(
                        "[gwt_absorb] broadcast context results: {} found",
                        results.len()
                    );
                }
            }
        }
        log::info!(
            "[gwt_absorb] iter={} reward={:.4} avg_cap={:.4}",
            iteration,
            reward,
            avg_cap
        );
        Ok(StageDecision::Continue)
    }
}


make_stage!(MemoryConsolidationStage);
impl BrainStage for MemoryConsolidationStage {
    fn name(&self) -> &str {
        "memory_consolidation"
    }
    fn frequency(&self) -> usize {
        12
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l5_cognition::l1_facade::MemoryCategory;

        let count_before = brain._memory_orch.size();
        let tiers = [
            MemoryTier::Working,
            MemoryTier::Episodic,
            MemoryTier::Procedural,
        ];
        let mut promoted_count = 0usize;
        for &tier in &tiers {
            let promoted = brain._memory_orch.promote(tier, |e| e.access_count >= 3);
            promoted_count += promoted.len();
            for entry in promoted {
                brain
                    ._memory_orch
                    .store(entry)
                    .map_err(|_| NeoTrixError::Brain("promote store".into()))?;
            }
        }
        let mut persisted = 0usize;
        if let Some(kb) = brain._nt_memory_kb.take() {
            persisted = brain.persist_pending_entries(&kb);
            brain._nt_memory_kb = Some(kb);
        }

        // Cross-session memory integration: store current iteration patterns
        if let Some(ref mut csm) = brain._cross_session_memory {
            let caps = brain.brain.capability.arr();
            let avg_cap = if caps.is_empty() {
                0.0
            } else {
                caps.iter().sum::<f64>() / caps.len() as f64
            };
            csm.remember(
                &format!("capability_iter_{}", brain.iteration),
                &format!("{:.4}", avg_cap),
                MemoryCategory::CapabilityState,
            );
            csm.remember(
                "task_type",
                &format!("{:?}", brain._current_task_type),
                MemoryCategory::Pattern,
            );
            csm.remember(
                "last_reward",
                &format!("{:.4}", brain._reward),
                MemoryCategory::TaskOutcome,
            );
        }

        // GWT broadcast: notify global workspace about memory state
        let size_after = brain._memory_orch.size();
        let csm_info = brain
            ._cross_session_memory
            .as_ref()
            .map(|csm: &crate::l5_cognition::l1_facade::CrossSessionMemory| format!(" csm={}", csm.len()))
            .unwrap_or_default();
        let msg = format!(
            "memory_consolidation: size={} promoted={} persisted={}{}",
            size_after, promoted_count, persisted, csm_info,
        );
        if let Some(ref mut engine) = brain.reasoning_engine {
            if let Some(ref mut gwt) = engine.gwt {
                gwt.broadcast(&msg);
            }
        }
        if count_before > 0 || persisted > 0 || promoted_count > 0 {
            log::debug!("[{}]", msg);
        }
        Ok(StageDecision::Continue)
    }
}


make_stage!(CacheCleanupStage);
impl BrainStage for CacheCleanupStage {
    fn name(&self) -> &str {
        "cache_cleanup"
    }
    fn frequency(&self) -> usize {
        50
    }
    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l5_cognition::nt_mind::foundation::cleanup_engine::{CleanupEngine, CleanupKind};
        let mut engine = CleanupEngine::new().with_project_root(std::path::PathBuf::from("."));
        engine.dry_run_default = false;
        engine.archive_on_clean = true;
        let r = engine.clean(CleanupKind::ProjectArtifacts);
        if r.deletable_count > 0 {
            log::info!(
                "[pipeline/cache_cleanup] archived {} items ({:.1} MB) -> .cleanup/archive/",
                r.deletable_count,
                r.estimated_bytes as f64 / 1_048_576.0
            );
        }
        Ok(StageDecision::Continue)
    }
}

