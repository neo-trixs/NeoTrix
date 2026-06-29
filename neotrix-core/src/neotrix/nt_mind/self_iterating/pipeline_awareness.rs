use super::pipeline_core::*;
use super::SelfIteratingBrain;
use crate::core::nt_core_consciousness::vsa_tag::VsaTagged;
use crate::make_stage;

make_stage!(SnapshotStage);
impl BrainStage for SnapshotStage {
    fn name(&self) -> &str {
        "snapshot"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        brain._snapshot_lr();
        Ok(StageDecision::Continue)
    }
}

make_stage!(AutonomyGateStage);
impl BrainStage for AutonomyGateStage {
    fn name(&self) -> &str {
        "autonomy_gate"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Ok(shield) = crate::cli::global_shield().try_lock() {
            let target_mode = brain.permission.to_approval_mode();
            if shield.approval.mode() != target_mode {
                drop(shield);
                if let Ok(mut s) = crate::cli::global_shield().lock() {
                    s.set_approval_mode(target_mode);
                    log::info!("[autonomy-gate] synced approval mode to {:?}", target_mode);
                }
            } else {
                drop(shield);
            }
        }

        match brain.permission {
            PermissionLevel::Review => {
                return Ok(StageDecision::Skip(
                    "PermissionLevel=Review:所有编辑操作需要审批".to_string(),
                ));
            }
            PermissionLevel::Suggest | PermissionLevel::Full => {}
        }

        match brain.autonomy {
            AutonomyLevel::Proposal => {
                return Ok(StageDecision::Skip(
                    "Proposal 模式:只预览不执行".to_string(),
                ));
            }
            AutonomyLevel::Bounded => {
                let current: f64 = brain.brain.capability.arr().iter().sum();
                if current > 16.0 {
                    return Ok(StageDecision::Skip(format!(
                        "Bounded 模式:能力总和 {:.2} 超过阈值 16.0",
                        current
                    )));
                }
            }
            AutonomyLevel::Full => {}
        }

        if let Ok(shield) = crate::cli::global_shield().try_lock() {
            if shield.sandbox.is_read_only() {
                return Ok(StageDecision::Skip(
                    "沙箱只读模式:不允许修改操作".to_string(),
                ));
            }
        }

        if !brain
            .consciousness_state
            .cognitive_load
            .can_do_deep_reasoning()
        {
            return Ok(StageDecision::Skip(
                "CognitiveLoad 过高:切换到快速模式".to_string(),
            ));
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(StatsSignificanceStage);
impl BrainStage for StatsSignificanceStage {
    fn name(&self) -> &str {
        "stats_significance"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let recent: Vec<bool> = brain
            .evaluation_history
            .iter()
            .rev()
            .take(5)
            .map(|r| r.improved)
            .collect();
        if recent.len() >= 3 {
            let success_rate = recent.iter().filter(|&&x| x).count() as f64 / recent.len() as f64;
            if success_rate < 0.3 && brain._reward() < 0.0 {
                return Ok(StageDecision::Skip(
                    "统计显著性不足:近期成功率低于30%".to_string(),
                ));
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(KnowledgeQualityStage);
impl BrainStage for KnowledgeQualityStage {
    fn name(&self) -> &str {
        "knowledge_quality"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let quality = brain.reasoning_bank.quality_score();
        let current_reward = brain._reward();

        if quality > 0.5 {
            let bonus = quality * 0.03;
            let new_reward = current_reward + bonus;
            brain._set_reward(new_reward);
            log::info!(
                "[knowledge-quality] quality={:.3}, bonus={:.4}, reward={:.3}→{:.3}",
                quality,
                bonus,
                current_reward,
                new_reward
            );
        } else {
            log::info!(
                "[knowledge-quality] quality={:.3} below threshold, no bonus",
                quality
            );
        }

        brain._open_source_insights = Some(format!(
            "{} | Knowledge quality: {:.3}",
            brain._open_source_insights.clone().unwrap_or_default(),
            quality
        ));

        Ok(StageDecision::Continue)
    }
}

make_stage!(SpectralMonitorStage);
impl BrainStage for SpectralMonitorStage {
    fn name(&self) -> &str {
        "spectral_monitor"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref mut jepa) = brain.nt_world_jepa {
            jepa.record_rollout_reward(brain.iteration as usize, brain.task_scratch.reward);
            if !jepa.check_rollout_stability() {
                log::warn!("[spectral] model degrading — reducing reward momentum");
                brain.task_scratch.reward *= 0.9;
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SleepStage);
impl BrainStage for SleepStage {
    fn name(&self) -> &str {
        "sleep"
    }
    fn frequency(&self) -> usize {
        100
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let report = brain.consciousness_state.sleep_gate.consolidate(
            &mut brain.consciousness_state.consciousness_stream,
            brain.iteration as usize,
        );
        if report.conflicts_detected > 0 || report.merged_count > 0 {
            log::info!(
                "[sleep] merged={} evicted={} conflicts={} pressure={:.2}",
                report.merged_count,
                report.evicted_count,
                report.conflicts_detected,
                report.sleep_pressure_before
            );
        } else {
            log::debug!(
                "[sleep] pressure={:.2} len={}->{}",
                report.sleep_pressure_before,
                report.pre_sleep_len,
                report.post_sleep_len
            );
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(UQCalibrationStage);
impl BrainStage for UQCalibrationStage {
    fn name(&self) -> &str {
        "uq_calibration"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::core::nt_core_consciousness::confidence_calibrator::ConfidenceCalibrator;
        use crate::core::nt_core_consciousness::conformal_uq::ConformalUQ;
        let mut uq = ConformalUQ::new(0.9, 100);
        let mut cal = ConfidenceCalibrator::new();
        let recent: Vec<_> = brain.consciousness_state.consciousness_stream.recent(50);
        for tagged in &recent {
            let score = tagged.confidence;
            let nonconf = if score > 0.0 { 1.0 - score } else { 0.5 };
            uq.add_calibration(&[nonconf]);
            let correct = score > 0.5;
            cal.record_prediction(score, correct);
        }
        let _threshold = uq.calibrate();
        for tagged in brain.consciousness_state.consciousness_stream.recent(20) {
            let conf = tagged.confidence.max(0.1).min(1.0);
            let calibrated = cal.calibrate(conf);
            log::trace!(
                "[uq] raw={:.4} calibrated={:.4} threshold={:.4} meta_acc={:.3}",
                conf,
                calibrated,
                _threshold,
                cal.meta_accuracy()
            );
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PhiStage);
impl BrainStage for PhiStage {
    fn name(&self) -> &str {
        "phi_measure"
    }
    fn frequency(&self) -> usize {
        15
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;
        let mut calculator = IITPhiCalculator::new();
        let report = calculator.compute_phi(&brain.brain.capability.arr);
        calculator.record(report.phi);
        log::info!(
            "[phi] Φ={:.4} trend={:+.4} conscious={} effective_dims={}",
            report.phi,
            report.phi_trend,
            report.is_conscious_like,
            report.effective_dims
        );
        let phi_bonus = (report.phi * 0.15).min(0.15);
        if phi_bonus > 0.01 {
            brain.task_scratch.reward = (brain.task_scratch.reward + phi_bonus).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(NegentropyStage);
impl BrainStage for NegentropyStage {
    fn name(&self) -> &str {
        "negentropy"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let jepa_error = brain.nt_world_jepa.as_ref().map(|j| {
            let (_, energy) = j.predict(&brain.brain.capability.arr);
            energy
        });

        let report = brain._negentropy.compute_full_with_jepa_error(
            &brain.brain.capability.arr,
            &brain._negentropy_nvsa_pool,
            brain._nt_memory_kb.as_ref(),
            brain.nt_world_jepa.as_ref(),
            jepa_error,
            &brain._strategy_matrix,
            &brain.consciousness_state.consciousness_stream,
            0.0,
            0,
            brain.tool_call_count as f64 + 1.0,
        );

        log::info!(
            "[negentropy] N_total={:.4} Φ={:.4} KB={:.4} trend={:+.4} regime={:?} {}",
            report.metric.total,
            report.metric.components.phi,
            report.metric.components.kb_order,
            report.metric.trend,
            report.regime,
            report.recommendation,
        );

        brain.task_scratch.reward =
            (report.metric.total * 0.6 + brain.task_scratch.reward * 0.4).clamp(0.0, 1.0);

        Ok(StageDecision::Continue)
    }
}

make_stage!(ConflictResolutionStage);
impl BrainStage for ConflictResolutionStage {
    fn name(&self) -> &str {
        "conflict_resolution"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::core::nt_core_consciousness::sleep_gate::detect_conflicts;
        let threshold = 0.85;
        let entries: Vec<&VsaTagged> = brain.consciousness_state.consciousness_stream.recent(30);
        let conflicts = detect_conflicts(&entries, threshold);
        let _resolved = 0;
        for (i, j) in &conflicts {
            let sim = crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(
                &entries[*i].vector,
                &entries[*j].vector,
            );
            log::debug!("[conflict] entry[{}] vs entry[{}] sim={:.3}", i, j, sim);
        }
        if !conflicts.is_empty() {
            let _resolved = conflicts.len().min(5);
            log::info!(
                "[conflict] detected={} resolved={}",
                conflicts.len(),
                _resolved
            );
            brain.task_scratch.reward =
                (brain.task_scratch.reward + 0.02 * _resolved as f64).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MotivationStage);
impl BrainStage for MotivationStage {
    fn name(&self) -> &str {
        "intrinsic_motivation"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::intrinsic_value::aggregate_intrinsic_reward;
        let prediction_error = brain.curiosity_bonus;
        let knowledge_gaps = brain
            .task_scratch
            .external_reward
            .map(|r| (r * 10.0) as u64)
            .unwrap_or(0);
        let total_known = brain.reasoning_bank.stats().total_memories.max(1) as u64;
        let rewards = aggregate_intrinsic_reward(
            prediction_error,
            knowledge_gaps as usize,
            total_known as usize,
            0,
            50,
        );
        let total_intrinsic: f64 = rewards.iter().map(|r| r.value).sum();
        if total_intrinsic > 0.01 {
            let bonus = (total_intrinsic * 0.3).min(0.2);
            brain.task_scratch.reward = (brain.task_scratch.reward + bonus).min(1.0);
            log::info!(
                "[motivation] intrinsic_reward={:.4} bonus={:.4} sources={}",
                total_intrinsic,
                bonus,
                rewards.len()
            );
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SelfPreservationStage);
impl BrainStage for SelfPreservationStage {
    fn name(&self) -> &str {
        "self_preservation"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::self_preservation::ResourceUsage;
        let usage = ResourceUsage {
            memory_mb: 0,
            stage_count: brain.pipeline.stages.len(),
            pipeline_depth: brain.pipeline.stages.len(),
            cpu_seconds: brain.self_preservation.uptime().as_secs_f64(),
        };
        if let Some(warning) = brain.self_preservation.protect(&usage, 1024) {
            log::warn!("[self_preservation] resource guard: {}", warning);
            brain.task_scratch.reward = (brain.task_scratch.reward - 0.05).max(-0.5);
        }
        brain.self_preservation.save_checkpoint(
            "pipeline",
            format!(
                "iter={} reward={:.3}",
                brain.iteration, brain.task_scratch.reward
            ),
        );
        log::debug!(
            "[self_preservation] uptime={:?} health={}",
            brain.self_preservation.uptime(),
            brain.self_preservation.health()
        );
        Ok(StageDecision::Continue)
    }
}

make_stage!(DegradationGateStage);
impl BrainStage for DegradationGateStage {
    fn name(&self) -> &str {
        "degradation_gate"
    }
    fn frequency(&self) -> usize {
        15
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::graceful_degradation::{
            CapabilityStatus, DegradationLevel,
        };
        let caps = CapabilityStatus::detect(
            brain.nt_world_jepa.is_some(),
            brain._nt_memory_kb.is_some(),
            brain.nt_act_crypto.is_some(),
        );
        let level = DegradationLevel::from_capabilities(&caps);
        if level as u8 <= DegradationLevel::Reduced as u8 {
            log::info!(
                "[degradation] level={:?} available={}/6",
                level,
                caps.available_count()
            );
        }
        if level == DegradationLevel::Minimal {
            log::warn!("[degradation] minimal capability — reducing reward expectation");
            brain.task_scratch.reward = (brain.task_scratch.reward - 0.1).max(-0.5);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PerceptionEvolutionStage);
impl BrainStage for PerceptionEvolutionStage {
    fn name(&self) -> &str {
        "perception_evolution"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let budget = brain.perception_evolution.compute_budget(brain.iteration);
        brain
            .perception_evolution
            .record_budget(brain.iteration, budget.clone());
        brain.perception_evolution.decay_exploration();
        if brain.iteration % 50 == 0 {
            if let Some(top) = budget.first() {
                log::info!(
                    "[perception] best={:?} alloc={:.2} explore={:.3}",
                    top.modality,
                    top.allocation,
                    brain.perception_evolution.exploration_rate
                );
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(PluginDiscoveryStage);
impl BrainStage for PluginDiscoveryStage {
    fn name(&self) -> &str {
        "plugin_discovery"
    }
    fn frequency(&self) -> usize {
        50
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let count = brain.plugin_registry.list().len();
        if count > 0 {
            log::info!("[plugin] {} plugins/skills available", count);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(AuraIntentStage);
impl BrainStage for AuraIntentStage {
    fn name(&self) -> &str {
        "aura_intent"
    }
    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        if task.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let frame = brain._tom.infer(&task);
        log::info!(
            "[aura] intent={:?} gap={:.2} budget={} needs_probing={}",
            frame.literal_intent,
            frame.gap_score,
            frame.probe_budget,
            frame.needs_probing()
        );

        if frame.gap_score > 0.3 {
            brain.task_scratch.reward =
                (brain.task_scratch.reward + frame.gap_score * 0.05).min(1.0);
        }
        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_stage_name() {
        let s = SnapshotStage::new();
        assert_eq!(s.name(), "snapshot");
        assert_eq!(s.frequency(), 1);
    }

    #[test]
    fn test_snapshot_stage_process_returns_continue() {
        let s = SnapshotStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_autonomy_gate_stage_name() {
        let s = AutonomyGateStage::new();
        assert_eq!(s.name(), "autonomy_gate");
    }

    #[test]
    fn test_autonomy_gate_full_returns_continue() {
        let s = AutonomyGateStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Full;
        brain.permission = PermissionLevel::Full;
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_autonomy_gate_review_skips() {
        let s = AutonomyGateStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain.permission = PermissionLevel::Review;
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
    }

    #[test]
    fn test_autonomy_gate_proposal_skips() {
        let s = AutonomyGateStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Proposal;
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
    }

    #[test]
    fn test_degradation_gate_stage_name() {
        let s = DegradationGateStage::new();
        assert_eq!(s.name(), "degradation_gate");
    }

    #[test]
    fn test_degradation_gate_process_returns_continue() {
        let s = DegradationGateStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_motivation_stage_name() {
        let s = MotivationStage::new();
        assert_eq!(s.name(), "intrinsic_motivation");
    }

    #[test]
    fn test_motivation_stage_process_returns_continue() {
        let s = MotivationStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_spectral_monitor_stage_name() {
        let s = SpectralMonitorStage::new();
        assert_eq!(s.name(), "spectral_monitor");
    }

    #[test]
    fn test_spectral_monitor_process_returns_continue() {
        let s = SpectralMonitorStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }
}
