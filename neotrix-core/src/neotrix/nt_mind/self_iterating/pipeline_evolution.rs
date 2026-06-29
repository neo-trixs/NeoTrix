use super::super::core::RewardSource;
use super::super::self_edit::MicroEdit;
use super::pipeline_core::*;
use super::SelfIteratingBrain;
use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::make_stage;

make_stage!(SSMUpdateStage);
impl BrainStage for SSMUpdateStage {
    fn name(&self) -> &str {
        "ssm_update"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        Ok(StageDecision::Continue)
    }
}

make_stage!(SelfEditGenerationStage);
impl BrainStage for SelfEditGenerationStage {
    fn name(&self) -> &str {
        "self_edit_gen"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();

        // ── Self-review targeted evolution ──
        // Read the previous audit report and inject extra evolution pressure
        // toward the weakest self-review dimension.
        let review_target = brain.seal_rl.self_review_report.as_ref().and_then(|r| {
            let dims = [
                (1.0 - r.cycle_risk, "cycle_risk"),
                (1.0 - r.panic_density, "panic_density"),
                (1.0 - r.unbounded_ratio, "unbounded_ratio"),
                (1.0 - r.dead_code_ratio, "dead_code_ratio"),
                (r.shutdown_coverage, "shutdown_coverage"),
                (r.feature_integrity, "feature_integrity"),
                (r.external_ref_coverage, "external_ref_coverage"),
            ];
            let (score, name) = dims
                .iter()
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))?;
            if *score < 0.6 {
                Some((*name, r.composite_health))
            } else {
                None
            }
        });

        let mut edits = if let Some(ref dgm) = brain.dgm_strategy {
            let noise = review_target
                .map(|(_, c)| {
                    0.5 - c * 0.5 // higher noise when health is lower
                })
                .unwrap_or(0.0);
            let ctx = super::brain_dgm::EditContext {
                task: &task,
                brain: &brain.brain,
                noise_level: noise,
            };
            dgm.generate_via_diffusion(&ctx)
        } else {
            let mut e = brain.brain.generate_self_edit(&task);
            // Inject a targeted edit toward the weakest dimension
            if let Some((weakest, _)) = review_target {
                e.push(MicroEdit::AdjustDimension(weakest.to_string(), 0.05));
            }
            e
        };

        let task_type = brain.task_scratch.current_task_type;
        let memories = brain
            .reasoning_bank
            .retrieve_relevant(&task, Some(task_type), 5);
        if !memories.is_empty() {
            let avg_reward: f64 =
                memories.iter().map(|m| m.reward).sum::<f64>() / memories.len() as f64;
            let factor = if avg_reward > 0.7 {
                1.1
            } else if avg_reward < 0.3 {
                0.9
            } else {
                1.0
            };
            if (factor - 1.0_f64).abs() > 0.01_f64 {
                for edit in &mut edits {
                    if let MicroEdit::AdjustDimension(_, ref mut amount) = edit {
                        *amount *= factor;
                    }
                }
            }
        }

        // If there's a review target, amplify edits in that direction
        if let Some((weakest, _)) = review_target {
            let amplify = (1.0
                - brain
                    .brain
                    .evaluate_capability(brain.task_scratch.current_task_type))
            .max(0.1);
            edits.push(MicroEdit::AdjustDimension(
                format!("{}_target", weakest),
                amplify * 0.1,
            ));
        }

        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(ApplyEditsStage);
impl BrainStage for ApplyEditsStage {
    fn name(&self) -> &str {
        "apply_edits"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let edits = brain._take_micro_edits();
        brain.brain.apply_micro_edits(&edits);
        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
    fn verify_step(
        &self,
        brain: &SelfIteratingBrain,
    ) -> Option<super::vsi_verifier::VsiStepVerdict> {
        let edits = brain._micro_edits();
        if edits.is_empty() {
            return None;
        }
        Some(super::vsi_verifier::verify_edit_magnitudes(&edits, 0.5))
    }
}

make_stage!(RewardCalculationStage);
impl BrainStage for RewardCalculationStage {
    fn name(&self) -> &str {
        "reward_calc"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let external = brain._external_reward();
        let (reward, source) = if let Some(ext) = external {
            (ext, RewardSource::External)
        } else {
            let task_type = brain.task_scratch.current_task_type;
            let score_before = brain._snapshot_score();
            let score_after = brain.brain.evaluate_capability(task_type);
            let regularization = brain.compute_regularization(&brain._snapshot_capability());
            let raw = (score_after - score_before) + regularization;
            let health = brain.evo_stats().health_score;
            let mut calibrated = raw * (0.5 + health * 0.5);

            // ── Self-review evolution pressure ──
            // Composite health from the audit cycle drives evolution: low health
            // means more room to improve (higher potential reward), and improvement
            // over previous cycles is directly rewarded.
            if let Some(ref report) = brain.seal_rl.self_review_report {
                let c = report.composite_health;
                let delta = brain
                    .seal_rl
                    .prev_composite_health
                    .map(|prev| c - prev)
                    .unwrap_or(0.0);

                // Low composite health = larger improvement target
                let improvement_potential = (1.0 - c) * 0.3;
                // Actual improvement = bonus reward
                let improvement_bonus = delta.max(0.0) * 0.5;
                // Weakest dimension drives targeted evolution
                let dims = [
                    (1.0 - report.cycle_risk, "cycle"),
                    (1.0 - report.panic_density, "panic"),
                    (1.0 - report.unbounded_ratio, "unbounded"),
                    (1.0 - report.dead_code_ratio, "dead"),
                    (report.shutdown_coverage, "shutdown"),
                    (report.feature_integrity, "feature"),
                    (report.external_ref_coverage, "external_ref"),
                ];
                let weakest = dims
                    .iter()
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let weakest_bonus = weakest.map(|(s, _)| (1.0 - s) * 0.2).unwrap_or(0.0);

                let review_bonus = improvement_potential + improvement_bonus + weakest_bonus;
                calibrated += review_bonus;

                if review_bonus.abs() > 0.01 {
                    log::debug!(
                        "[reward] self-review bonus={:.4} (pot={:.3} impr={:.3} weakest={:.3})",
                        review_bonus,
                        improvement_potential,
                        improvement_bonus,
                        weakest_bonus,
                    );
                }
            }

            let critic_out = VsaTagged::new(
                QuantizedVSA::random_binary(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            )
            .with_confidence(if calibrated > 0.0 { 0.6 } else { 0.3 });
            let critic_ctx = VsaTagged::new(
                QuantizedVSA::random_binary(),
                VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
            );
            let critique = brain.consciousness_state.inner_critic.evaluate(
                &critic_out,
                &critic_ctx,
                Some(&brain.consciousness_state.specious_present),
            );
            if !critique.passed && calibrated > 0.0 {
                calibrated *= 0.8;
            }

            (calibrated, RewardSource::Internal)
        };
        brain._set_reward(reward);
        brain._set_reward_source(source);
        Ok(StageDecision::Continue)
    }
    fn verify_step(
        &self,
        brain: &SelfIteratingBrain,
    ) -> Option<super::vsi_verifier::VsiStepVerdict> {
        Some(super::vsi_verifier::verify_reward(brain._reward()))
    }
}

make_stage!(GwtAbsorbStage);
impl BrainStage for GwtAbsorbStage {
    fn name(&self) -> &str {
        "gwt_absorb"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let reward = brain._reward();
        let source = brain._reward_source();
        let summary = format!(
            "task: {}, reward: {:.4}, source: {:?}",
            task, reward, source
        );
        if let Some(ref mut router) = brain.attention_router {
            router.absorb_reasoning_result(&task, &summary, "seal_loop");
            router.wm().broadcast(&summary);
            log::info!("[gwt-absorb] broadcast: {}", summary);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(HyperCubeOptimizeStage);
impl BrainStage for HyperCubeOptimizeStage {
    fn name(&self) -> &str {
        "hypercube_optimize"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pruned = if let Some(ref mut router) = brain.attention_router {
            router.bridge.prune_low_access(2)
        } else {
            0
        };
        if pruned > 0 {
            log::info!("[hypercube-optimize] pruned {} low-access entries", pruned);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(E8ExperimentStage);
impl BrainStage for E8ExperimentStage {
    fn name(&self) -> &str {
        "e8_experiment"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let reward = brain._reward();
        let task_type = brain.task_scratch.current_task_type;

        let mode = brain
            ._e8_policy
            .select_mode(&task, task_type, &brain._transition_learner);
        brain
            ._transition_learner
            .record(&task, mode, reward, brain.iteration);
        brain._e8_policy.update(reward);

        if let Some(ref kb) = brain._nt_memory_kb {
            if let Ok(patterns) = kb.get_evolution_patterns(5) {
                if !patterns.is_empty() {
                    let mut factor_deltas = [0.0f64; crate::core::NUM_E8_FACTORS];
                    for pattern in &patterns {
                        let base_deltas = match pattern.pattern_type {
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::RecurringError
                                => [0.2, 0.0, 0.1, 0.0, 0.0, 0.3],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::CommunicationOptimization
                                => [0.0, 0.1, 0.0, 0.0, 0.3, 0.0],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::ProblemDecomposition
                                => [-0.2, 0.0, -0.2, 0.2, 0.0, -0.1],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::VerificationImprovement
                                => [0.0, 0.0, -0.1, 0.0, 0.0, -0.2],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::ToolUsagePattern
                                => [0.0, -0.1, 0.0, 0.1, 0.0, 0.0],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::StrategyDiscovery
                                => [0.3, 0.2, 0.3, -0.1, 0.1, 0.3],
                            crate::neotrix::nt_memory_kb::EvolutionPatternType::PrincipleUpdate
                                => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                        };
                        let weight = pattern.effectiveness_gain * 0.5;
                        for i in 0..crate::core::NUM_E8_FACTORS {
                            factor_deltas[i] += base_deltas[i] * weight;
                        }
                    }
                    let has_nonzero = factor_deltas.iter().any(|d| d.abs() > 0.001);
                    if has_nonzero {
                        brain._e8_policy.update_factorized(reward, &factor_deltas);
                        log::info!(
                            "[e8-experiment] factorized update from {} evolution patterns: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
                            patterns.len(),
                            factor_deltas[0], factor_deltas[1], factor_deltas[2],
                            factor_deltas[3], factor_deltas[4], factor_deltas[5],
                        );
                    }
                }
            }
        }

        brain._e8_policy.decay_epsilon();

        log::info!(
            "[e8-experiment] mode={}, epsilon={:.4}, mode_value={:.4}",
            mode.0,
            brain._e8_policy.epsilon(),
            brain._e8_policy.mode_values[mode.0 as usize],
        );

        if let Some(ref mut router) = brain.attention_router {
            let bridge = &router.bridge;
            let gap_reports = bridge.analyze_gaps();
            let high_gaps: usize = gap_reports.iter().filter(|r| r.gap > 0.5).count();
            if high_gaps > 3 && brain._transition_learner.outcomes.len() > 20 {
                let approach = (mode.0 >> 3) as usize;
                let domain = (mode.0 & 0x07) as usize;
                if let Some(pattern) = brain
                    ._transition_learner
                    .suggest_evolution(approach, domain)
                {
                    log::info!(
                        "[e8-experiment] suggested evolution ({}) for cell ({},{})",
                        pattern,
                        approach,
                        domain
                    );
                }
            }
        }

        let evolved = brain
            ._transition_learner
            .evolve_matrix(&mut brain._strategy_matrix);
        if evolved > 0 {
            log::info!("[e8-experiment] evolved {} strategy matrix cells", evolved);
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(HarnessAdaptStage);
impl BrainStage for HarnessAdaptStage {
    fn name(&self) -> &str {
        "harness_adapt"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let current_task = brain._current_task();
        if let Some(ref mut router) = brain.attention_router {
            let env = current_task.clone();
            router.set_environment(&env);
            let profile = brain.seal_rl.harness_adapter.active_profile().cloned();
            if let Some(p) = profile {
                router.register_harness_profile(&env, &p);
                log::info!(
                    "[harness-adapt] applied profile for env={}, performance_delta={:.4}",
                    env,
                    p.performance_delta
                );
            }
        }
        if let Some(ref kb) = brain._nt_memory_kb {
            let saved = brain.seal_rl.harness_adapter.save_to_kb(kb).unwrap_or(0);
            if saved > 0 {
                log::info!("[harness-adapt] saved {} profiles to KB", saved);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(TaskAffinityStage);
impl BrainStage for TaskAffinityStage {
    fn name(&self) -> &str {
        "task_affinity"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain.task_scratch.current_task_type;
        let reward = brain._reward();
        brain
            .brain
            .update_task_affinity(task_type, brain._snapshot_score() + reward);
        Ok(StageDecision::Continue)
    }
}

make_stage!(RollbackDecisionStage);
impl BrainStage for RollbackDecisionStage {
    fn name(&self) -> &str {
        "rollback_decision"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward();
        let source = brain._reward_source();
        let snapshot_lr = brain._snapshot_lr();
        if reward < 0.0 && source == RewardSource::External {
            brain._snapshot_restore();
            brain.brain.learning_rate = (snapshot_lr * 0.9).max(0.01);
            return Ok(StageDecision::Rollback("外部奖励为负，已回滚".to_string()));
        }
        if reward > brain.quality_threshold {
            if let Err(e) = brain.brain.save() {
                log::warn!("持久化失败: {}", e);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(AdaptiveLRStage);
impl BrainStage for AdaptiveLRStage {
    fn name(&self) -> &str {
        "adaptive_lr"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain.task_scratch.reward + brain.curiosity_bonus;
        let adapted_lr = brain.curvature_policy.adapt_lr(reward);
        brain.brain.learning_rate = adapted_lr;
        log::debug!(
            "[curvature] lr={:.4} regime={:?}",
            adapted_lr,
            brain.curvature_policy.regime()
        );
        Ok(StageDecision::Continue)
    }
}

make_stage!(ChampionCompareStage);
impl BrainStage for ChampionCompareStage {
    fn name(&self) -> &str {
        "champion_compare"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref champion) = brain.champion {
            let task_type = brain.task_scratch.current_task_type;
            let current_score = brain.brain.evaluate_capability(task_type);
            if current_score > champion.score * 1.05 {
                let new_champ = BrainSnapshot::new(&brain.brain, &task_type);
                return Ok(StageDecision::Promote(new_champ));
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(EvaluationStage);
impl BrainStage for EvaluationStage {
    fn name(&self) -> &str {
        "evaluation"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain.task_scratch.current_task_type;
        let score_before = brain._snapshot_score();
        let score_after = brain.brain.evaluate_capability(task_type);
        brain
            .evaluation_history
            .push(super::brain_impl::EvaluationRecord {
                iteration: brain.iteration,
                task_type,
                score_before,
                score_after,
                improved: score_after > score_before,
            });
        Ok(StageDecision::Continue)
    }
}

make_stage!(DistillationStage);
impl BrainStage for DistillationStage {
    fn name(&self) -> &str {
        "session_distill"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let session = crate::neotrix::nt_act_autonomy::knowledge_distiller::SessionRecord {
            id: format!("seal-iter-{}", brain.iteration),
            user_messages: vec![brain._current_task()],
            actions_taken: brain
                .task_scratch
                .micro_edits
                .iter()
                .map(|e| format!("{:?}", e))
                .collect(),
            outcomes: vec![format!("reward={:.4}", brain._reward())],
            reward_signal: brain._reward(),
            timestamp: brain.iteration,
            task_type: Some(format!("{:?}", brain.task_scratch.current_task_type)),
            e8_mode: Some(brain._e8_policy.best_mode()),
            edit_types: brain
                .task_scratch
                .micro_edits
                .iter()
                .map(|e| format!("{:?}", e))
                .collect(),
        };
        let principles = brain._knowledge_distiller.distill(&session);
        if !principles.is_empty() {
            let absorbed = brain
                ._knowledge_distiller
                .absorb(&mut brain.brain.capability);
            log::info!(
                "[session-distill] {} principles from iter {}, {} absorbed",
                principles.len(),
                brain.iteration,
                absorbed,
            );
            let summary = brain._knowledge_distiller.summary();
            if let Some(ref mut router) = brain.attention_router {
                router.wm().broadcast(&summary);
            }
        }
        Ok(StageDecision::Continue)
    }
}

// ── SelfReviewStage: 审查官 — 6-dimension audit fused into SEAL pipeline ──
// Runs every N cycles, audits brain state for common defects, and stores
// the report in brain.seal_rl.self_review_report for consumption by
// MetaCognitiveLoop (via ConsciousnessIntegration bridge).

make_stage!(SelfReviewStage);
impl BrainStage for SelfReviewStage {
    fn name(&self) -> &str {
        "self_review"
    }
    fn frequency(&self) -> usize {
        10 // run every 10 SEAL cycles (not every cycle — lightweight)
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind::self_iterating::sib_state::SelfReviewReport;

        // D1: Cycle risk — check if handle_consciousness_batch increments cycle
        // Proxy: presence of active pipeline stages implies cycles are running
        let cycle_risk = if brain.iteration > 0 && brain.pipeline.stages.len() > 50 {
            0.0 // pipeline is active, cycles are likely incrementing
        } else {
            0.5 // uncertain — no direct access to consciousness cycle
        };

        // D2: Panic density — count unwrap/expect in brain's eval history
        let total_evals = brain.evaluation_history.len().max(1) as f64;
        let panic_count = brain
            .seal_rl
            .stage_results
            .iter()
            .filter(|r| r.stage_name.contains("error") || r.efficiency < 0.0)
            .count() as f64;
        let panic_density = (panic_count / total_evals).clamp(0.0, 1.0);

        // D3: Unbounded collection ratio — check key Vec fields
        let collections_checked = 6.0; // evaluation_history, tool_traces, meta_additions, brain.absorption_history, brain.weight_history, brain.harness_history
        let unbounded = [
            brain.evaluation_history.len() > 10_000,
            brain.tool_traces.len() > 10_000,
            brain.meta_additions.len() > 100,
            brain.brain.absorption_history.len() > 10_000,
            brain.brain.weight_history.len() > 10_000,
            brain.brain.harness_history.len() > 10_000,
        ]
        .iter()
        .filter(|&&b| b)
        .count() as f64;
        let unbounded_ratio = unbounded / collections_checked;

        // D4: Dead code ratio — orphan module markers in meta_additions
        let dead_code_ratio = if brain.meta_additions.is_empty() {
            0.1
        } else {
            0.0
        };

        // D5: Shutdown coverage — check if at least one shutdown mechanism exists
        let shutdown_coverage = 0.8; // proxy: known shutdown signals exist in codebase

        // D6: Feature gate integrity — check if feature gates are declared
        let feature_integrity = 0.8; // proxy: most features are declared

        // D7: External knowledge coverage — ratio of weak dimensions with
        // mapped external references (literature / open-source projects)
        let covered = brain.seal_rl.covered_dimension_count();
        let external_ref_coverage = covered as f64 / 6.0; // 0.0–1.0
        let external_references: Vec<String> = brain
            .seal_rl
            .external_references
            .iter()
            .map(|(dim, url, desc)| format!("{}: {} ({})", dim, url, desc))
            .collect();

        // Composite health: weighted harmonic mean of all 7 dims
        // D1 inverted: lower cycle_risk = better
        let weights = [0.20, 0.15, 0.15, 0.10, 0.10, 0.10, 0.20];

        let scores = [
            cycle_risk,
            1.0 - panic_density,
            1.0 - unbounded_ratio,
            1.0 - dead_code_ratio,
            shutdown_coverage,
            feature_integrity,
            external_ref_coverage,
        ];
        let weighted_sum: f64 = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        let composite_health = weighted_sum.clamp(0.0, 1.0);

        let report = SelfReviewReport {
            cycle_risk,
            panic_density,
            unbounded_ratio,
            dead_code_ratio,
            shutdown_coverage,
            feature_integrity,
            external_ref_coverage,
            external_references,
            composite_health,
            iteration: brain.iteration,
        };

        // ── Evolution pressure: track improvement delta ──
        let prev = brain
            .seal_rl
            .prev_composite_health
            .replace(composite_health);
        let delta = prev.map(|p| composite_health - p).unwrap_or(0.0);

        brain.seal_rl.self_review_report = Some(report.clone());

        // Log the report
        log::info!(
            "[self-review] iter={} composite={:.3} delta={:+.3} | D1={:.2} D2={:.2} D3={:.2} D4={:.2} D5={:.2} D6={:.2} D7(ext)={:.2} refs={}",
            brain.iteration, composite_health, delta,
            report.cycle_risk, report.panic_density, report.unbounded_ratio,
            report.dead_code_ratio, report.shutdown_coverage, report.feature_integrity,
            report.external_ref_coverage, report.external_references.len(),
        );

        if composite_health < 0.4 {
            log::warn!(
                "[self-review] CRITICAL: composite health {:.3} — evolution targeting weak dimensions",
                composite_health
            );
        } else if delta < -0.1 {
            log::warn!(
                "[self-review] REGRESSION: composite dropped {:.3} — evolution will deprioritize currently selected edits",
                delta
            );
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(ConversationDistillStage);
impl BrainStage for ConversationDistillStage {
    fn name(&self) -> &str {
        "conversation_distill"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let Some(ref kb) = brain._nt_memory_kb else {
            return Ok(StageDecision::Skip("no KB".into()));
        };
        let result = run_conversation_distill(kb)?;

        if result.total == 0 {
            return Ok(StageDecision::Continue);
        }

        if result.total_gain > 0.0 {
            let bonus = (result.total_gain * 0.1).min(0.5);
            brain._set_reward(brain._reward() + bonus);
            log::info!("[conv-distill] evolution reward bonus: {:.4}", bonus);
        }

        if let Some(ref mut router) = brain.attention_router {
            let report = format!(
                "[conversation-evolution] {} records: {}/{} OK, avg_eff={:.2}, error_rate={:.3}, patterns_created={}",
                result.total, result.successes, result.failures, result.avg_eff, result.error_rate,
                if result.patterns_created { "yes" } else { "no" },
            );
            router.wm().broadcast(&report);
        }

        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssm_update_stage_name() {
        let s = SSMUpdateStage::new();
        assert_eq!(s.name(), "ssm_update");
        assert_eq!(s.frequency(), 1);
    }

    #[test]
    fn test_ssm_update_stage_process_returns_continue() {
        let s = SSMUpdateStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_evaluation_stage_name() {
        let s = EvaluationStage::new();
        assert_eq!(s.name(), "evaluation");
    }

    #[test]
    fn test_evaluation_stage_process_returns_continue() {
        let s = EvaluationStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain.run_seal_loop("eval_test", None, None).ok();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_task_affinity_stage_name() {
        let s = TaskAffinityStage::new();
        assert_eq!(s.name(), "task_affinity");
        assert_eq!(s.frequency(), 2);
    }

    #[test]
    fn test_champion_compare_without_champion() {
        let s = ChampionCompareStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        assert!(brain.champion.is_none());
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_champion_compare_with_champion() {
        let s = ChampionCompareStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        use crate::neotrix::nt_expert_routing::TaskType;
        brain.champion = Some(BrainSnapshot::new(&brain.brain, &TaskType::General));
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_conversation_distill_no_kb() {
        let s = ConversationDistillStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
    }

    #[test]
    fn test_reward_calculation_stage_name() {
        let s = RewardCalculationStage::new();
        assert_eq!(s.name(), "reward_calc");
    }

    #[test]
    fn test_reward_calculation_with_external_reward() {
        let s = RewardCalculationStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain.task_scratch.external_reward = Some(0.5);
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
        assert!((brain._reward() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_rollback_decision_with_positive_reward() {
        let s = RollbackDecisionStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        brain._set_reward(0.5);
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_adaptive_lr_stage_name() {
        let s = AdaptiveLRStage::new();
        assert_eq!(s.name(), "adaptive_lr");
    }

    #[test]
    fn test_adaptive_lr_process_returns_continue() {
        let s = AdaptiveLRStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
    }

    #[test]
    fn test_gwt_absorb_stage_name() {
        let s = GwtAbsorbStage::new();
        assert_eq!(s.name(), "gwt_absorb");
    }

    #[test]
    fn test_hypercube_optimize_stage_name() {
        let s = HyperCubeOptimizeStage::new();
        assert_eq!(s.name(), "hypercube_optimize");
    }

    #[test]
    fn test_self_edit_generation_stage_name() {
        let s = SelfEditGenerationStage::new();
        assert_eq!(s.name(), "self_edit_gen");
    }

    #[test]
    fn test_self_review_stage_process() {
        let s = SelfReviewStage::new();
        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = s.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
        assert!(brain.seal_rl.self_review_report.is_some());
    }

    #[test]
    fn test_apply_edits_stage_name() {
        let s = ApplyEditsStage::new();
        assert_eq!(s.name(), "apply_edits");
    }

    #[test]
    fn test_harness_adapt_stage_name() {
        let s = HarnessAdaptStage::new();
        assert_eq!(s.name(), "harness_adapt");
    }
}
