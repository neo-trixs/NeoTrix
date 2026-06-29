use super::core::SelfIteratingBrain;
use crate::cli::shield_enforcer::global_shield;
use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_ssm::SelectiveState;
use crate::core::nt_core_util;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_core_signal::select::SelectableOperator;
use crate::neotrix::nt_expert_routing::{Context, TaskType};
use crate::neotrix::nt_mind::attention_router::AttentionRouter;
use crate::neotrix::nt_mind::core::{CapabilityVector, RewardSource};
use crate::neotrix::nt_mind::cortex_memory::{DimensionTag, MemoryTrace, Modality};
use crate::neotrix::nt_mind::distillation::{
    apply_principles, avoid_anti_patterns, ExperienceDistiller,
};
use crate::neotrix::nt_mind::knowledge_chain::{ChainRunResult, KnowledgeChain};
use crate::neotrix::nt_mind::memory::{ReasoningBank, ReasoningMemory};
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::self_edit::MicroEdit;
use crate::neotrix::nt_mind::self_iterating::brain_impl::EvaluationRecord;
use crate::neotrix::nt_mind::self_iterating::goal_contract::should_stop_seal_loop;
use crate::neotrix::nt_mind::self_iterating::pipeline::kernel_iterate_pipeline;
use crate::neotrix::nt_mind::self_iterating::recipe::RecipeRegistry;
use crate::neotrix::nt_mind::sleep::SleepEngine;
use crate::neotrix::nt_mind::stagnation::StagnationSignal;
use crate::neotrix::nt_mind::stats::IterationResult;
use log;

type BatchTask<'a> = &'a [(String, Option<Vec<f64>>, Option<f64>)];

impl SelfIteratingBrain {
    pub fn run_seal_loop_pipeline(
        &mut self,
        task: &str,
        task_embedding: Option<Vec<f64>>,
        external_reward: Option<f64>,
    ) -> NeoTrixResult<f64> {
        // ShieldEnforcer governance check: is SEAL self-iteration allowed?
        if let Ok(shield) = global_shield().lock() {
            if let Err(decision) = shield.check_all("seal_iterate", "internal", None, None) {
                let msg = match decision {
                    crate::cli::ShieldDecision::Block(m) => {
                        format!("Shield blocked SEAL iteration: {}", m)
                    }
                    crate::cli::ShieldDecision::RequireApproval(m) => {
                        format!("Shield requires approval for SEAL iteration: {}", m)
                    }
                    _ => "Shield blocked SEAL iteration".to_string(),
                };
                log::warn!("{}", msg);
            }
        }

        self.task_scratch.current_task = task.to_string();
        self.task_scratch.current_task_type = Context::from_task_description(task).task_type;
        self.task_scratch.task_embedding = task_embedding;
        self.task_scratch.external_reward = external_reward;
        self.task_scratch.reward = 0.0;
        self.task_scratch.reward_source = RewardSource::Internal;

        let pipeline = std::mem::take(&mut self.pipeline);
        let result = pipeline.execute(self);
        self.pipeline = pipeline;

        let reward = self.task_scratch.reward;

        match result {
            Ok(_) => Ok(reward),
            Err(e) => {
                if self.task_scratch.reward < 0.0 && self.task_scratch.external_reward.is_some() {
                    self._snapshot_restore();
                }
                Err(e)
            }
        }
    }

    pub fn iterate(&mut self, task_type: TaskType) -> IterationResult {
        self.iteration += 1;

        let score_before = self.brain.evaluate_capability(task_type);

        let aging = self.seal_rl.aging_monitor.overall_aging();
        let interference = self.seal_rl.aging_monitor.interference_score;
        if aging > 0.5 {
            log::debug!(
                "[aging] iterate: high aging ({:.3}), skipping auto-absorb",
                aging
            );
        } else if score_before < self.quality_threshold && self.auto_absorb {
            let interference_penalty = 1.0 - (interference * 0.5);
            let absorb_rate = (self.brain.learning_rate * interference_penalty).max(0.01);
            let sources = self.select_relevant_sources(task_type);
            for source in &sources {
                if self.brain.learning_rate_budget >= absorb_rate {
                    self.brain.absorb(*source);
                    self.brain.learning_rate_budget -= absorb_rate;
                }
            }
        }

        // ── CryptoAgent scan & absorb (every 3 iterations) ──
        if self.iteration % 3 == 0 {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
                crypto.run_iteration();
                let opps = crypto.scan_opportunities();
                let total_value: f64 = opps.iter().map(|o| o.estimated_value_usd).sum();
                if total_value > 0.0 {
                    let boost = (total_value * 0.001).min(0.3).max(0.01);
                    let mut v = self.brain.capability.clone();
                    v.set_analysis((v.analysis() + boost * 0.1).min(1.0));
                    v.set_synthesis((v.synthesis() + boost * 0.05).min(1.0));
                    self.brain
                        .register_knowledge_source("nt_act_crypto::earnings", v);
                    let _ = self.brain.absorb_from_custom("nt_act_crypto::earnings");
                }
            }
        }

        let score_after = self.brain.evaluate_capability(task_type);
        let improved = score_after > score_before;

        self.evaluation_history.push(EvaluationRecord {
            iteration: self.iteration,
            task_type,
            score_before,
            score_after,
            improved,
        });

        if self.auto_memory_iteration
            && self
                .iteration
                .is_multiple_of(self.memory_iteration_interval)
        {
            self.reasoning_bank.iterate_memories(0.85, 0.1);
            let all_mems: Vec<ReasoningMemory> =
                self.reasoning_bank.memories().iter().cloned().collect();
            let principles = ExperienceDistiller::distill(&all_mems);
            if !principles.is_empty() {
                apply_principles(&mut self.brain.capability, &principles, 0.6);
            }
            let anti_patterns = ExperienceDistiller::contrastive_reflect(&all_mems);
            if !anti_patterns.is_empty() {
                avoid_anti_patterns(&mut self.brain.capability, &anti_patterns);
            }
            if let Some(ref mut gm) = self.group_manager {
                gm.evolve_group();
            }
        }

        IterationResult {
            iteration: self.iteration,
            task_type,
            score_before,
            score_after,
            improved,
            absorbed_count: self.brain.total_absorb_count,
        }
    }

    pub fn code_review_iterate(
        &mut self,
        files: &std::collections::HashMap<std::path::PathBuf, String>,
    ) -> IterationResult {
        self.iteration += 1;
        let task_type = TaskType::CodeReview;
        let score_before = self.brain.evaluate_capability(task_type);

        let engine = crate::neotrix::nt_mind::code_review::CodeReviewEngine::new(
            self.brain.capability.clone(),
        );
        let mut all_issues = Vec::new();
        let mut finding_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (path, content) in files {
            let report = engine.review(&path.to_string_lossy(), content);
            all_issues.extend(report.issues.iter().map(|i| (path.clone(), i.clone())));
            for issue in &report.issues {
                let key = format!("{:?}", issue.severity);
                *finding_counts.entry(key).or_insert(0) += 1;
            }
        }

        let critical_count = *finding_counts.get("Critical").unwrap_or(&0);
        let high_count = *finding_counts.get("High").unwrap_or(&0);

        if critical_count > 0 || high_count > 0 {
            let mut edits = Vec::new();
            if critical_count > 0 {
                edits.push(MicroEdit::AdjustDimension(
                    "verification".to_string(),
                    0.04 * critical_count as f64,
                ));
                edits.push(MicroEdit::AdjustDimension(
                    "quality_gates".to_string(),
                    0.03 * critical_count as f64,
                ));
            }
            if high_count > 0 {
                edits.push(MicroEdit::AdjustDimension(
                    "analysis".to_string(),
                    0.02 * high_count as f64,
                ));
            }
            edits.push(MicroEdit::NormalizeVector);
            self.brain.apply_micro_edits(&edits);
        }

        let reward = self.brain.evaluate_capability(task_type) - score_before;
        let micro_edits = self.brain.generate_self_edit("code_review");
        let memory = ReasoningMemory::new(
            &format!("code_review_iteration_{}", self.iteration),
            task_type,
            &micro_edits,
            reward,
        );
        self.reasoning_bank.store(memory);

        if self.auto_memory_iteration
            && self
                .iteration
                .is_multiple_of(self.memory_iteration_interval)
        {
            self.reasoning_bank.iterate_memories(0.85, 0.1);
        }

        let score_after = self.brain.evaluate_capability(task_type);
        self.evaluation_history.push(EvaluationRecord {
            iteration: self.iteration,
            task_type,
            score_before,
            score_after,
            improved: score_after > score_before,
        });

        IterationResult {
            iteration: self.iteration,
            task_type,
            score_before,
            score_after,
            improved: score_after > score_before,
            absorbed_count: self.brain.total_absorb_count,
        }
    }

    pub fn kernel_iterate(&mut self, task: &str) -> IterationResult {
        self.iteration += 1;
        self.task_scratch.current_task = task.to_string();
        self.task_scratch.current_task_type = Context::from_task_description(task).task_type;

        let pipeline = kernel_iterate_pipeline();
        let _ = pipeline.execute(self);

        if self.tool_call_count > 0 {
            let success_count = self.tool_traces.iter().filter(|(_, _, s)| *s).count() as f64;
            let total = self.tool_call_count as f64;
            let success_ratio = success_count / total.max(1.0);
            if success_ratio > 0.5 {
                let bonus = (total * 0.01).min(0.05);
                let current = self.brain.capability.quality_gates();
                self.brain
                    .capability
                    .set_quality_gates((current + bonus).min(1.0));
            }
            self.tool_call_count = 0;
            self.tool_traces.clear();
        }

        let score_after = self
            .brain
            .evaluate_capability(self.task_scratch.current_task_type);

        IterationResult {
            iteration: self.iteration,
            task_type: self.task_scratch.current_task_type,
            score_before: self._snapshot_score(),
            score_after,
            improved: score_after > self._snapshot_score(),
            absorbed_count: self.brain.total_absorb_count,
        }
    }

    pub fn run_seal_loop(
        &mut self,
        task: &str,
        task_embedding: Option<Vec<f64>>,
        external_reward: Option<f64>,
    ) -> NeoTrixResult<f64> {
        self.iteration += 1;

        // ── #4 EntropyMonitor crisis check → StagnationSignal ──
        if let Some(ref engine) = self.reasoning_engine {
            if let Some(ref gwt) = engine.gwt {
                self.entropy_crisis_level = gwt.entropy_monitor.crisis_level();
                if gwt.entropy_monitor.should_rollback() {
                    let reason = format!(
                        "entropy deadlock crisis: {} stimuli exhausted, stuck_ratio={:.3}",
                        gwt.entropy_monitor.stimulus_attempts,
                        gwt.entropy_monitor.stuck_ratio(),
                    );
                    log::warn!(
                        "[seal] ⏸ 熵死锁触发暂停: {} (cycle {})",
                        reason,
                        self.iteration
                    );
                    return Ok(self.task_scratch.reward);
                }
            }
        }

        let sig = self
            .stagnation
            .observe(false, false, 0, self.task_scratch.reward, false, false);
        match sig {
            StagnationSignal::Stop(ref reason) => {
                log::warn!(
                    "[seal] ⏹ 停滞检测触发: {} (cycle {})",
                    reason,
                    self.iteration
                );
                return Ok(0.0);
            }
            StagnationSignal::Pause(secs, ref reason) => {
                log::info!(
                    "[seal] ⏸ 停滞暂停 {}s: {} (cycle {})",
                    secs,
                    reason,
                    self.iteration
                );
                return Ok(0.0);
            }
            StagnationSignal::Continue => {}
        }

        // AgingBench: adjust quality_threshold and auto_absorb based on aging
        let aging = self.seal_rl.aging_monitor.overall_aging();
        if aging > 0.5 {
            self.auto_absorb = false;
            self.quality_threshold = (self.quality_threshold * 0.8).max(0.3);
            log::info!(
                "[aging] high aging ({:.3}): disabled auto_absorb, reduced threshold to {:.3}",
                aging,
                self.quality_threshold
            );
        } else if aging > 0.3 {
            self.quality_threshold = (self.quality_threshold * 0.9).max(0.4);
            log::debug!(
                "[aging] moderate aging ({:.3}): reduced threshold to {:.3}",
                aging,
                self.quality_threshold
            );
        }

        self.task_scratch.current_task = task.to_string();
        self.task_scratch.current_task_type = Context::from_task_description(task).task_type;
        self.task_scratch.task_embedding = task_embedding;
        self.task_scratch.external_reward = external_reward;
        self.task_scratch.reward = 0.0;
        self.task_scratch.reward_source = RewardSource::Internal;

        if let Some(ref mut router) = self.attention_router {
            let ctx = router.route(task);
            if !ctx.knowledge_lines.is_empty() {
                let suffix = router.build_knowledge_prompt_suffix(&ctx);
                self.task_scratch.current_task = format!("{}\n{}", task, suffix);
            }
        } else {
            self.init_attention_router();
            if let Some(ref mut router) = self.attention_router {
                let ctx = router.route(task);
                let suffix = router.build_knowledge_prompt_suffix(&ctx);
                self.task_scratch.current_task = format!("{}\n{}", task, suffix);
            }
        }

        let pipeline = std::mem::take(&mut self.pipeline);

        let result: Result<(), NeoTrixError> = {
            let registry = std::mem::replace(&mut self.recipe_registry, RecipeRegistry::new());
            let idx = registry.select_index(self.task_scratch.current_task_type);
            if let Some(idx) = idx {
                let name = registry.all()[idx].config.name.clone();
                log::info!(
                    "[recipe] selected '{}' for task_type={:?}",
                    name,
                    self.task_scratch.current_task_type
                );
                let r = registry.all()[idx].execute(self);
                self.recipe_registry = registry;
                r
            } else {
                self.recipe_registry = registry;
                pipeline.execute(self).map(|_| ())
            }
        };

        self.pipeline = pipeline;
        let mut reward = self.task_scratch.reward;

        // ── CognitiveLoad: record load based on pipeline result ──
        let load = reward.abs().min(1.0);
        self.consciousness_state.cognitive_load.record_step(load);

        // ── ConsciousnessAwakening: bootstrap if not yet awakened ──
        if self.consciousness_state.first_person.birth_step() == 0 && self.iteration > 0 {
            let report = crate::core::nt_core_consciousness::awakening::awaken(
                &mut self.consciousness_state.consciousness_stream,
                &mut self.consciousness_state.specious_present,
            );
            self.consciousness_state.first_person = report.self_reference;
        }

        // ── #5 Curiosity bonus: gap between expected and actual improvement ──
        let score_before = self._snapshot_score();
        let score_after = self
            .brain
            .evaluate_capability(self.task_scratch.current_task_type);
        let expected = reward.max(0.0);
        let actual = (score_after - score_before).max(0.0);
        let prediction_error = (expected - actual).abs();
        let scaled_curiosity = (prediction_error * 0.1).min(0.5);
        self.curiosity_bonus = scaled_curiosity;
        reward += scaled_curiosity;

        // ── InnerCritic: quality gate on current reward context ──
        let critic_tagged = VsaTagged::new(
            QuantizedVSA::random_binary(),
            VsaOrigin::Self_(VsaSelfCategory::Thought),
        )
        .with_confidence(if reward > 0.0 {
            0.5 + reward * 0.5
        } else {
            0.3
        });
        let context_tagged = VsaTagged::new(
            QuantizedVSA::random_binary(),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        let critique = self.consciousness_state.inner_critic.evaluate(
            &critic_tagged,
            &context_tagged,
            Some(&self.consciousness_state.specious_present),
        );
        if !critique.passed && reward > 0.0 {
            reward *= 0.85; // quality penalty
        }
        self.consciousness_state.inner_critic.adjust_thresholds();

        // ── #2 Goal progress bonus ──
        self.goal_register.tick();
        let goal_bonus = self.goal_register.overall_progress() * 0.02;
        reward += goal_bonus;

        // ── RecursiveDepthReward: monotonic depth bonus (Thinking Pixel, Phase 6.2)
        // d_rec = number of E8 state transitions in this reasoning cycle
        let depth = self
            .reasoning_engine
            .as_ref()
            .map(|e| e.state_trajectory.len() as f64)
            .unwrap_or(1.0);
        let depth_bonus = 0.02 * (0.3 * depth).tanh();
        reward += depth_bonus;

        // Sync self.task_scratch.reward so downstream tool_call_count modifications accumulate
        self.task_scratch.reward = reward;

        // ── Curvature-aware LR scheduler: feed reward signal ──
        self.seal_rl.lr_scheduler.observe_reward(reward);

        // ── CryptoAgent periodic scan & absorb ──
        if self.iteration % 3 == 0 {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
                crypto.run_iteration();
                let opps = crypto.scan_opportunities();
                let total_value: f64 = opps.iter().map(|o| o.estimated_value_usd).sum();
                if total_value > 0.0 {
                    let boost = (total_value * 0.001).min(0.3).max(0.01);
                    let mut v = self.brain.capability.clone();
                    v.set_analysis((v.analysis() + boost * 0.1).min(1.0));
                    v.set_synthesis((v.synthesis() + boost * 0.05).min(1.0));
                    self.brain
                        .register_knowledge_source("nt_act_crypto::earnings", v);
                    let _ = self.brain.absorb_from_custom("nt_act_crypto::earnings");
                }
            }
        }

        if self.tool_call_count > 0 {
            let compilable = self
                .tool_traces
                .iter()
                .any(|(tool, _, success)| *success && tool.contains("cargo"));
            if compilable {
                self.task_scratch.reward += 0.05;
                self.task_scratch.reward_source = crate::core::RewardSource::External;
            }
        }

        if self.tool_call_count > 0 {
            let success_count = self.tool_traces.iter().filter(|(_, _, s)| *s).count() as f64;
            let total = self.tool_call_count as f64;
            let success_ratio = success_count / total.max(1.0);
            if success_ratio > 0.5 {
                let bonus = (total * 0.01).min(0.05);
                let current = self.brain.capability.quality_gates();
                self.brain
                    .capability
                    .set_quality_gates((current + bonus).min(1.0));
            }
            // Skill evolution: mine traces → diagnose → repair
            let edits_from_evolution =
                self.skill_evolver
                    .evolve(&[], &self.tool_traces, self.iteration, None);
            if !edits_from_evolution.is_empty() {
                log::info!(
                    "[skill_evolution] evolved {} edits (total_repaired={}, proposals={})",
                    edits_from_evolution.len(),
                    self.skill_evolver.total_repaired,
                    self.skill_evolver.total_proposed,
                );
            }
            self.tool_call_count = 0;
            self.tool_traces.clear();
        }

        let edits = self._take_micro_edits();
        if !edits.is_empty() {
            self.archive
                .record(&self.task_scratch.current_task, "run_seal_loop", &edits);
        }
        self._set_micro_edits(edits);

        self.consciousness_state.narrative_self.save();

        match result {
            Ok(_) => {
                self.task_scratch.reward = reward;
                if let Some(ref mut router) = self.attention_router {
                    router.wm().broadcast(&format!(
                        "SEAL loop completed: task='{}', reward={:.4}",
                        self.task_scratch.current_task, reward
                    ));
                }
                let final_reward = self.task_scratch.reward;
                Ok(final_reward)
            }
            Err(e) => {
                if self.task_scratch.reward < 0.0 && self.task_scratch.external_reward.is_some() {
                    self._snapshot_restore();
                }
                Err(e)
            }
        }
    }

    /// Run SEAL loop repeatedly until the goal contract is satisfied or max iterations reached.
    /// This implements the /goal pattern: keep iterating until verifiable conditions are met.
    pub fn run_until_goal_achieved(
        &mut self,
        task: &str,
        task_embedding: Option<Vec<f64>>,
        external_reward: Option<f64>,
        max_iterations: usize,
    ) -> NeoTrixResult<f64> {
        self.goal_state.goal_complete = false;
        self.goal_state.goal_contract = None;
        self.goal_state.phase_evidence.clear();

        let mut cumulative_reward = 0.0;
        let mut iterations = 0;

        for i in 0..max_iterations {
            match self.run_seal_loop(task, task_embedding.clone(), external_reward) {
                Ok(reward) => {
                    cumulative_reward += reward;
                    iterations += 1;
                    if should_stop_seal_loop(self) {
                        log::info!(
                            "[goal] /goal achieved after {} iterations (reward={:.3})",
                            i + 1,
                            cumulative_reward
                        );
                        break;
                    }
                }
                Err(e) => {
                    log::warn!("[goal] iteration {} failed: {}", i + 1, e);
                    if should_stop_seal_loop(self) {
                        break;
                    }
                }
            }
        }

        let avg_reward = if iterations > 0 {
            cumulative_reward / iterations as f64
        } else {
            0.0
        };
        log::info!(
            "[goal] /goal complete: {} iterations, avg_reward={:.3}, achieved={}",
            iterations,
            avg_reward,
            self.goal_state.goal_complete
        );

        if let Err(e) = crate::neotrix::nt_mind::self_iterating::goal_contract::write_journal(self)
        {
            log::warn!("[goal] journal write failed: {}", e);
        }

        Ok(avg_reward)
    }

    pub fn run_seal_loop_batch(&mut self, tasks: BatchTask) -> NeoTrixResult<f64> {
        let mut total_reward = 0.0;
        let mut valid_tasks = 0;

        for (task, embedding, external_reward) in tasks {
            match self.run_seal_loop(task, embedding.clone(), *external_reward) {
                Ok(reward) => {
                    total_reward += reward;
                    valid_tasks += 1;
                }
                Err(e) => {
                    log::error!("任务 '{}' 执行失败: {}", task, e);
                }
            }
        }

        let avg_reward = if valid_tasks > 0 {
            total_reward / valid_tasks as f64
        } else {
            0.0
        };

        self.update_policy(avg_reward);

        if let Some(ref mut router) = self.attention_router {
            router.wm().broadcast(&format!(
                "SEAL batch completed: {} tasks, avg_reward={:.4}",
                valid_tasks, avg_reward
            ));
        }

        Ok(avg_reward)
    }

    pub(crate) fn compute_regularization(&self, snapshot: &CapabilityVector) -> f64 {
        let current = &self.brain.capability;
        let diff: Vec<f64> = current
            .arr()
            .iter()
            .zip(snapshot.arr().iter())
            .map(|(a, b)| a - b)
            .collect();
        let l2_dist: f64 = diff.iter().map(|x| x * x).sum::<f64>().sqrt();
        -self.regularization_weight * l2_dist
    }

    pub(crate) fn update_policy(&mut self, avg_reward: f64) {
        let aging = self.seal_rl.aging_monitor.overall_aging();
        let interference = self.seal_rl.aging_monitor.interference_score;
        let learning_penalty = 1.0 - (aging * 0.3 + interference * 0.2);
        let effective_lr = self.policy_learning_rate * learning_penalty.max(0.5);
        if avg_reward > 0.5 {
            self.brain.learning_rate = (self.brain.learning_rate * (1.0 + effective_lr)).min(0.3);
        } else if avg_reward < 0.0 {
            self.brain.learning_rate = (self.brain.learning_rate * (1.0 - effective_lr)).max(0.01);
        }
        if aging > 0.6 {
            self.regularization_weight = (self.regularization_weight * 1.1).min(0.01);
        } else if aging > 0.3 {
            self.regularization_weight = (self.regularization_weight * 1.05).min(0.005);
        }
    }

    pub fn run_knowledge_chain(&mut self) -> NeoTrixResult<ChainRunResult> {
        let work_dir = nt_core_util::home_dir().join(".neotrix").join("work");
        let mut chain = KnowledgeChain::new(work_dir);
        chain.init_default_discovery();
        let result = chain.run_chain(&mut self.brain, &mut self.reasoning_bank)?;
        if result.mined > 0 {
            self.iteration += 1;
            self.evaluation_history.push(EvaluationRecord {
                iteration: self.iteration,
                task_type: TaskType::General,
                score_before: 0.0,
                score_after: result.total_reward,
                improved: result.total_reward > 0.0,
            });
        }
        Ok(result)
    }

    pub fn save_cortex(&self) -> NeoTrixResult<()> {
        let path = nt_core_util::home_dir()
            .join(".neotrix")
            .join("cortex.json");
        let json = self.cortex.export_json();
        let data = serde_json::to_string_pretty(&json)
            .map_err(|e| NeoTrixError::Serde(format!("cortex序列化失败: {}", e)))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &data)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn load_cortex(&mut self) {
        let path = nt_core_util::home_dir()
            .join(".neotrix")
            .join("cortex.json");
        if !path.exists() {
            return;
        }
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return,
        };
        let json: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Some(traces) = json["traces"].as_array() {
            for t in traces {
                let title = t["title"].as_str().unwrap_or("unknown");
                let source = t["source"].as_str().unwrap_or("");
                let summary = t["summary"].as_str().unwrap_or("");
                let dims: Vec<DimensionTag> = t["dimensions"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                let name = v.as_str().unwrap_or("");
                                DimensionTag::all()
                                    .into_iter()
                                    .find(|d| format!("{:?}", d) == name)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let importance = t["importance"].as_f64().unwrap_or(0.5);
                let trace = MemoryTrace::new(title, source, summary, Modality::Text, dims)
                    .with_importance(importance);
                self.cortex.store(trace);
            }
        }
    }

    pub fn print_cortex_report(&self) {
        log::info!("{}", self.cortex.report());
    }

    pub fn cortex_recall(&mut self, query: &str, limit: usize) {
        let results = self.cortex.recall(query, limit);
        log::info!("🔍 联想检索 \"{}\" → {} 条结果:", query, results.len());
        for (i, (trace, score)) in results.iter().enumerate() {
            log::info!("  [{:.2}] {} [{}]", score, trace.title, trace.source_type);
            if i == 0 {
                log::info!("    📝 {}", &trace.summary[..trace.summary.len().min(150)]);
            }
        }
    }

    pub fn cortex_chain(&self, category: &str, limit: usize) {
        let traces = self.cortex.dimension_chain(category, limit);
        log::info!("📌 {} ({} 条):", category, traces.len());
        for t in &traces {
            let _dims: Vec<String> = t.dimensions.iter().map(|d| format!("{:?}", d)).collect();
            //            println!("  • {} [{}] — {}", t.title, t.source_type, dims.join(", "));
        }
    }

    // ========== Init helpers (referenced by run_seal_loop & iterate) ==========
    pub fn init_reasoning_engine(&mut self) {
        let brain: Box<dyn crate::neotrix::nt_mind::core::BrainMutView> =
            Box::new(std::mem::take(&mut self.brain));
        let bank = std::mem::replace(&mut self.reasoning_bank, ReasoningBank::new(100));
        let mut engine = ReasoningEngine::from_env(brain, bank);
        if let Some(ref jepa) = self.nt_world_jepa {
            engine = engine.with_jepa(jepa.clone());
        }
        if let Ok(kb) = crate::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
            let emb_cfg = crate::neotrix::nt_memory_kb::nt_memory_embed::EmbeddingConfig::default();
            if !emb_cfg.api_key.is_empty() {
                kb.with_embedding(emb_cfg);
                if let Err(e) = kb.ensure_embeddings() {
                    log::warn!("[KB] embedding warmup: {}", e);
                }
            }
            engine = engine.with_kb(kb);
        }
        self.reasoning_engine = Some(engine);
        self.load_e8();
    }

    pub fn save_e8(&self) {
        if let Some(ref engine) = self.reasoning_engine {
            let path = nt_core_util::home_dir()
                .join(".neotrix")
                .join("e8_state.json");
            if let Err(e) = engine.save_e8_state(&path) {
                log::warn!("保存 E8 状态失败: {}", e);
            }
        }
    }

    pub fn load_e8(&mut self) {
        if let Some(ref mut engine) = self.reasoning_engine {
            let path = nt_core_util::home_dir()
                .join(".neotrix")
                .join("e8_state.json");
            if path.exists() {
                if let Err(e) = engine.load_e8_state(&path) {
                    log::warn!("加载 E8 状态失败: {}", e);
                }
            }
        }
    }

    pub fn init_attention_router(&mut self) {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        self.attention_router = Some(router);
    }

    pub fn init_select_operator(&mut self, dim: usize, hidden_dim: usize) {
        self.select_operator = Some(SelectableOperator::new(dim, hidden_dim));
        self.selective_state = Some(SelectiveState::new(dim, hidden_dim));
    }

    pub fn init_sleep_engine(&mut self, passes: usize) {
        self.sleep_engine = Some(SleepEngine::with_passes(passes));
    }
}

#[cfg(test)]
mod tests {
    use super::super::core::SelfIteratingBrain;
    use crate::neotrix::nt_expert_routing::TaskType;
    use crate::neotrix::nt_mind::core::KnowledgeSource;

    #[test]
    fn test_brain_new_iteration_zero() {
        let brain = SelfIteratingBrain::new();
        assert_eq!(brain.iteration, 0);
    }

    #[test]
    fn test_record_tool_call_works() {
        let mut brain = SelfIteratingBrain::new();
        brain.record_tool_call("test_tool", 100, true);
        assert_eq!(brain.tool_call_count, 1);
        assert_eq!(brain.tool_traces.len(), 1);
    }

    #[test]
    fn test_record_tool_call_respects_bound() {
        let mut brain = SelfIteratingBrain::new();
        for i in 0..150 {
            brain.record_tool_call(&format!("tool_{}", i), i as u64, i % 2 == 0);
        }
        assert_eq!(brain.tool_call_count, 150);
        assert!(brain.tool_traces.len() <= 100);
    }

    #[test]
    fn test_pipeline_status_returns_string() {
        let brain = SelfIteratingBrain::new();
        let status = brain.pipeline_status();
        assert!(status.contains("iter=0"));
    }

    #[test]
    fn test_select_relevant_sources_design() {
        let brain = SelfIteratingBrain::new();
        let sources = brain.select_relevant_sources(TaskType::Design);
        assert!(!sources.is_empty(), "Design should have knowledge sources");
    }

    #[test]
    fn test_select_relevant_sources_general_is_empty() {
        let brain = SelfIteratingBrain::new();
        let sources = brain.select_relevant_sources(TaskType::General);
        assert!(sources.is_empty());
    }

    #[test]
    fn test_auto_tune_from_archive_empty() {
        let mut brain = SelfIteratingBrain::new();
        brain.auto_tune_from_archive(); // should not panic
        assert_eq!(brain.archive.len(), 0);
    }

    #[test]
    fn test_get_brain_report_works() {
        let brain = SelfIteratingBrain::new();
        let report = brain.get_brain_report();
        assert_eq!(report.iteration, 0);
    }

    #[test]
    fn test_diff_no_changes() {
        let brain = SelfIteratingBrain::new();
        let before = brain.brain.capability.clone();
        let output = brain.diff(&before);
        assert!(output.is_empty(), "diff should be empty when unchanged");
    }

    #[test]
    fn test_compute_regularization_negative() {
        let brain = SelfIteratingBrain::new();
        let snapshot = brain.brain.capability.clone();
        let reg = brain.compute_regularization(&snapshot);
        assert!(reg <= 0.0, "regularization should be <= 0.0, got {}", reg);
    }

    #[test]
    fn test_preview_absorb_returns_values() {
        let brain = SelfIteratingBrain::new();
        let (_before, _after, delta) = brain.preview_absorb(KnowledgeSource::DesignPhilosophy);
        assert!(
            delta >= -0.1 && delta <= 1.1,
            "delta out of range: {}",
            delta
        );
    }

    #[test]
    fn test_growth_curve_slope_empty() {
        let brain = SelfIteratingBrain::new();
        assert_eq!(brain.growth_curve_slope(), 0.0);
    }

    #[test]
    fn test_transfer_efficiency_empty() {
        let brain = SelfIteratingBrain::new();
        assert_eq!(brain.transfer_efficiency(), 0.0);
    }

    #[test]
    fn test_error_avoidance_rate_empty() {
        let brain = SelfIteratingBrain::new();
        assert_eq!(brain.error_avoidance_rate(), 1.0);
    }

    #[test]
    fn test_evo_stats_returns_defaults() {
        let brain = SelfIteratingBrain::new();
        let stats = brain.evo_stats();
        assert!(stats.health_score >= 0.0);
    }

    #[test]
    fn test_use_and_disable_dgm() {
        let mut brain = SelfIteratingBrain::new();
        assert!(brain.dgm_strategy.is_none());
        brain.use_dgm_strategy(10);
        assert!(brain.dgm_strategy.is_some());
        brain.disable_dgm_strategy();
        assert!(brain.dgm_strategy.is_none());
    }

    #[test]
    fn test_run_seal_loop_short_task() {
        // Minimal test: run a single short task to verify it doesn't panic
        let mut brain = SelfIteratingBrain::new();
        let result = brain.run_seal_loop("test_task", None, None);
        assert!(
            result.is_ok(),
            "seal loop should return Ok, got {:?}",
            result
        );
    }

    #[test]
    fn test_kernel_iterate_returns_result() {
        let mut brain = SelfIteratingBrain::new();
        let result = brain.kernel_iterate("test_kernel");
        assert!(result.iteration > 0);
    }
}
