use super::core::SelfIteratingBrain;
use super::super::brain_impl::EvaluationRecord;
use super::super::super::core::{CapabilityVector, RewardSource};
use super::super::super::self_edit::MicroEdit;
use super::super::super::memory::{ReasoningMemory, ReasoningBank};
use super::super::super::distillation::{ExperienceDistiller, apply_principles, avoid_anti_patterns};
use super::super::super::reasoning_engine::ReasoningEngine;
use super::super::super::attention_router::AttentionRouter;
use super::super::super::cortex_memory::{MemoryTrace, DimensionTag, Modality};
use super::super::super::knowledge_chain::{KnowledgeChain, ChainRunResult};
use super::super::super::sleep::{SleepEngine};
use super::super::super::stats::IterationResult;
use super::super::super::stagnation::StagnationSignal;
use super::super::pipeline::kernel_iterate_pipeline;
use crate::neotrix::nt_world_model::{TaskType, Context};
use crate::neotrix::nt_io_provider::create_gateway;
use crate::core::nt_core_sae_bridge::SAEBridge;
use crate::core::SparseAutoencoder;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_core_signal::select::SelectableOperator;
use crate::neotrix::nt_core_signal::SelectiveState;
use crate::core::nt_core_e8::ewhr_bridge::E8EwhrBridge;
use std::sync::{Arc, Mutex};
use crate::cli::shield_enforcer::global_shield;


type BatchTask<'a> = &'a [(String, Option<Vec<f64>>, Option<f64>)];

impl SelfIteratingBrain {
    pub fn run_seal_loop_pipeline(&mut self, task: &str, task_embedding: Option<Vec<f64>>, external_reward: Option<f64>) -> NeoTrixResult<f64> {
        // ShieldEnforcer governance check: is SEAL self-iteration allowed?
        if let Ok(shield) = global_shield().lock() {
            if let Err(decision) = shield.check_all("seal_iterate", "internal", None, None) {
                let msg = match decision {
                    crate::cli::ShieldDecision::Block(m) => format!("Shield blocked SEAL iteration: {}", m),
                    crate::cli::ShieldDecision::RequireApproval(m) => format!("Shield requires approval for SEAL iteration: {}", m),
                    _ => "Shield blocked SEAL iteration".to_string(),
                };
                log::warn!("{}", msg);
            }
        }

        self._current_task = task.to_string();
        self._current_task_type = Context::from_task_description(task).task_type;
        self._task_embedding = task_embedding;
        self._external_reward = external_reward;
        self._reward = 0.0;
        self._reward_source = RewardSource::Internal;

        let pipeline = std::mem::take(&mut self.pipeline);
        let result = pipeline.execute(self);
        self.pipeline = pipeline;

        let reward = self._reward;

        match result {
            Ok(_) => Ok(reward),
            Err(e) => {
                if self._reward < 0.0 && self._external_reward.is_some() {
                    self._snapshot_restore();
                }
                Err(e)
            }
        }
    }

    pub fn iterate(&mut self, task_type: TaskType) -> IterationResult {
        self.iteration += 1;

        let score_before = self.brain.evaluate_capability(task_type);

        let aging = self._aging_monitor.overall_aging();
        let interference = self._aging_monitor.interference_score;
        if aging > 0.5 {
            log::debug!("[aging] iterate: high aging ({:.3}), skipping auto-absorb", aging);
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
        if self.iteration.is_multiple_of(3) {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap_or_else(|e| { log::warn!("[seal] mutex poisoned: {}", e); e.into_inner() });
                crypto.run_iteration();
                let opps = crypto.scan_opportunities();
                let total_value: f64 = opps.iter().map(|o| o.estimated_value_usd).sum();
                if total_value > 0.0 {
                    let boost = (total_value * 0.001).min(0.3).max(0.01);
                    let mut v = self.brain.capability.clone();
                    v.set_analysis((v.analysis() + boost * 0.1).min(1.0));
                    v.set_synthesis((v.synthesis() + boost * 0.05).min(1.0));
                    self.brain.register_knowledge_source("nt_act_crypto::earnings", v);
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

        if self.auto_memory_iteration && self.iteration.is_multiple_of(self.memory_iteration_interval) {
            self.reasoning_bank.iterate_memories(0.85, 0.1);
            let all_mems: Vec<ReasoningMemory> = self.reasoning_bank.memories().iter().cloned().collect();
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

        self.sync_elements();

        IterationResult {
            iteration: self.iteration,
            task_type,
            score_before,
            score_after,
            improved,
            absorbed_count: self.brain.total_absorb_count,
        }
    }

    pub fn code_review_iterate(&mut self, files: &std::collections::HashMap<std::path::PathBuf, String>) -> IterationResult {
        self.iteration += 1;
        let task_type = TaskType::CodeReview;
        let score_before = self.brain.evaluate_capability(task_type);

        let engine = crate::neotrix::nt_mind::code_review::CodeReviewEngine::new(self.brain.capability.clone());
        let mut all_issues = Vec::new();
        let mut finding_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

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
                    "verification".to_string(), 0.04 * critical_count as f64));
                edits.push(MicroEdit::AdjustDimension(
                    "quality_gates".to_string(), 0.03 * critical_count as f64));
            }
            if high_count > 0 {
                edits.push(MicroEdit::AdjustDimension(
                    "analysis".to_string(), 0.02 * high_count as f64));
            }
            edits.push(MicroEdit::NormalizeVector);
            self.brain.apply_micro_edits(&edits);
        }

        let reward = self.brain.evaluate_capability(task_type) - score_before;
        let micro_edits = self.brain.generate_self_edit("code_review");
        let memory = ReasoningMemory::new(
            &format!("code_review_iteration_{}", self.iteration),
            task_type, &micro_edits, reward);
        self.reasoning_bank.store(memory);

        if self.auto_memory_iteration && self.iteration.is_multiple_of(self.memory_iteration_interval) {
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
        self._current_task = task.to_string();
        self._current_task_type = Context::from_task_description(task).task_type;

        let pipeline = kernel_iterate_pipeline();
        let _ = pipeline.execute(self);

        if self.tool_call_count > 0 {
            let success_count = self.tool_traces.iter().filter(|(_, _, s)| *s).count() as f64;
            let total = self.tool_call_count as f64;
            let success_ratio = success_count / total.max(1.0);
            if success_ratio > 0.5 {
                let bonus = (total * 0.01).min(0.05);
                let current = self.brain.capability.quality_gates();
                self.brain.capability.set_quality_gates((current + bonus).min(1.0));
            }
            self.tool_call_count = 0;
            self.tool_traces.clear();
        }

        let score_after = self.brain.evaluate_capability(self._current_task_type);

        IterationResult {
            iteration: self.iteration,
            task_type: self._current_task_type,
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

        // ── Lazy init: reasoning_engine (EWHR bridge, hypothesis_network, E8) ──
        if self.reasoning_engine.is_none() {
            self.init_reasoning_engine();
        }

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
                    println!("[seal] ⏸ 熵死锁触发暂停: {} (cycle {})", reason, self.iteration);
                    return Ok(self._reward);
                }
            }
        }

        let sig = self.stagnation.observe(false, false, 0, self._reward, false, false);
        match sig {
            StagnationSignal::Stop(ref reason) => {
                println!("[seal] ⏹ 停滞检测触发: {} (cycle {})", reason, self.iteration);
                return Ok(0.0);
            }
            StagnationSignal::Pause(secs, ref reason) => {
                println!("[seal] ⏸ 停滞暂停 {}s: {} (cycle {})", secs, reason, self.iteration);
                return Ok(0.0);
            }
            StagnationSignal::Continue => {}
        }

        // AgingBench: adjust quality_threshold and auto_absorb based on aging
        let aging = self._aging_monitor.overall_aging();
        if aging > 0.5 {
            self.auto_absorb = false;
            self.quality_threshold = (self.quality_threshold * 0.8).max(0.3);
            log::info!("[aging] high aging ({:.3}): disabled auto_absorb, reduced threshold to {:.3}", aging, self.quality_threshold);
        } else if aging > 0.3 {
            self.quality_threshold = (self.quality_threshold * 0.9).max(0.4);
            log::debug!("[aging] moderate aging ({:.3}): reduced threshold to {:.3}", aging, self.quality_threshold);
        }

        self._current_task = task.to_string();
        self._current_task_type = Context::from_task_description(task).task_type;
        self._task_embedding = task_embedding;
        self._external_reward = external_reward;
        self._reward = 0.0;
        self._reward_source = RewardSource::Internal;

        if let Some(ref mut router) = self.attention_router {
            let ctx = router.route(task);
            if !ctx.knowledge_lines.is_empty() {
                let suffix = router.build_knowledge_prompt_suffix(&ctx);
                self._current_task = format!("{}\n{}", task, suffix);
            }
        } else {
            self.init_attention_router();
            if let Some(ref mut router) = self.attention_router {
                let ctx = router.route(task);
                let suffix = router.build_knowledge_prompt_suffix(&ctx);
                self._current_task = format!("{}\n{}", task, suffix);
            }
        }

        let pipeline = std::mem::take(&mut self.pipeline);
        let result = pipeline.execute(self);
        self.pipeline = pipeline;
        let mut reward = self._reward;

        // ── EWHR reasoning pass: analyze trajectory → propose hypotheses ──
        if let Some(ref mut engine) = self.reasoning_engine {
            let task_copy = self._current_task.clone();
            if let Err(e) = engine.reason(&task_copy) {
                log::warn!("[EWHR] reason pass failed: {}", e);
            }


            // Blend PRM process rewards into the SEAL reward signal for transition learning.
            // Previously the ProcessRewardLearner computed per-step rewards inside engine.reason()
            // (engine_core.rs:435-460) via learn_step()/collector/record_step(), producing
            // avg_recent_score() values, but these scores were NEVER propagated back to the
            // transition learner, DpSgdStage, or ProceduralMemoryStage. Instead, the learner
            // recorded outcomes with the pipeline's SEAL reward only, losing the PRM's fine-grained
            // process-level evaluation signal. Now we read avg_recent_score(10) and blend it
            // into the reward used for transition learning (30% PRM, 70% pipeline reward).
            if let Some(ref prm) = engine.prm {
                let prm_avg = prm.avg_recent_score(10);
                if prm_avg > 0.0 {
                    let prev = reward;
                    reward = reward * 0.7 + prm_avg * 0.3;
                    self._prm_cumulative_reward = prm_avg;
                    self._prm_step_rewards.push((self.iteration as usize, prm_avg));
                    if self._prm_step_rewards.len() > 100 {
                        self._prm_step_rewards.remove(0);
                    }
                    log::trace!("[PRM] blended reward: pipeline={:.4} prm={:.4} final={:.4}",
                        prev, prm_avg, reward);
                }
            }
            // Record outcome in the E8 transition learner for policy evolution.
            // Previously the E8TransitionLearner was constructed (self._transition_learner)
            // and its select_mode() / record() methods were used by DpSgdStage and
            // ProceduralMemoryStage, but record() was NEVER called from the production
            // pipeline — outcomes remained empty, causing both stages to silently no-op.
            // Now we feed the current task, mode, reward, and iteration number after
            // each EWHR reason pass so the learner accumulates real experience.
            let current_mode = engine.current_state.mode;
            let iter = self.iteration;
            let current_reward = reward;
            self._transition_learner.record(
                &self._current_task,
                current_mode,
                current_reward,
                iter,
            );
        }

        // ── Persist E8 transition matrix to KB every 3 iterations ──
        // Previously save_e8() was defined but never called — all runtime
        // transition learning (from E8TransitionLearner / Observer) was lost
        // on process exit. Now persisted periodically so cross-session learning
        // actually accumulates across SEAL cycles.
        if self.iteration.is_multiple_of(3) {
            self.save_e8();
            log::trace!("[E8-TM] saved transition matrix to KB (iter {})", self.iteration);
        }

        // ── #5 Curiosity bonus: gap between expected and actual improvement ──
        let score_before = self._snapshot_score();
        let score_after = self.brain.evaluate_capability(self._current_task_type);
        let expected = reward.max(0.0);
        let actual = (score_after - score_before).max(0.0);
        let prediction_error = (expected - actual).abs();
        let scaled_curiosity = (prediction_error * 0.1).min(0.05);
        self.curiosity_bonus = scaled_curiosity;
        reward += scaled_curiosity;

        // ── #2 Goal progress bonus ──
        self.goal_register.tick();
        let goal_bonus = self.goal_register.overall_progress() * 0.02;
        reward += goal_bonus;

        // ── RecursiveDepthReward: monotonic depth bonus (Thinking Pixel, Phase 6.2)
        // d_rec = number of E8 state transitions in this reasoning cycle
        let depth = self.reasoning_engine.as_ref()
            .map(|e| e.state_trajectory.len() as f64)
            .unwrap_or(1.0);
        let depth_bonus = 0.02 * (0.3 * depth).tanh();
        reward += depth_bonus;

        // Sync self._reward so downstream tool_call_count modifications accumulate
        self._reward = reward;

        // ── CryptoAgent periodic scan & absorb ──
        if self.iteration.is_multiple_of(3) {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap_or_else(|e| { log::warn!("[seal] mutex poisoned: {}", e); e.into_inner() });
                crypto.run_iteration();
                let opps = crypto.scan_opportunities();
                let total_value: f64 = opps.iter().map(|o| o.estimated_value_usd).sum();
                if total_value > 0.0 {
                    let boost = (total_value * 0.001).min(0.3).max(0.01);
                    let mut v = self.brain.capability.clone();
                    v.set_analysis((v.analysis() + boost * 0.1).min(1.0));
                    v.set_synthesis((v.synthesis() + boost * 0.05).min(1.0));
                    self.brain.register_knowledge_source("nt_act_crypto::earnings", v);
                    let _ = self.brain.absorb_from_custom("nt_act_crypto::earnings");
                }
            }
        }

        if self.tool_call_count > 0 {
            let compilable = self.tool_traces.iter().any(|(tool, _, success)| *success && tool.contains("cargo"));
            if compilable {
                self._reward += 0.05;
                self._reward_source = crate::core::RewardSource::External;
            }
        }

        if self.tool_call_count > 0 {
            let success_count = self.tool_traces.iter().filter(|(_, _, s)| *s).count() as f64;
            let total = self.tool_call_count as f64;
            let success_ratio = success_count / total.max(1.0);
            if success_ratio > 0.5 {
                let bonus = (total * 0.01).min(0.05);
                let current = self.brain.capability.quality_gates();
                self.brain.capability.set_quality_gates((current + bonus).min(1.0));
            }
            self.tool_call_count = 0;
            self.tool_traces.clear();
        }

        let edits = self._take_micro_edits();
        if !edits.is_empty() {
            self.archive.record(&self._current_task, "run_seal_loop", &edits);
        }
        self._set_micro_edits(edits);

        // ── Persist E8 state every 5 iterations for higher-frequency checkpointing ──
        if self.iteration.is_multiple_of(5) {
            self.save_e8();
        }

        match result {
            Ok(_) => {
                self._reward = reward;
            if let Some(ref mut router) = self.attention_router {
                    router.wm().broadcast(&format!(
                        "SEAL loop completed: task='{}', reward={:.4}",
                        self._current_task, reward
                    ));
                }
                let final_reward = self._reward;
                Ok(final_reward)
            }
            Err(e) => {
                if self._reward < 0.0 && self._external_reward.is_some() {
                    self._snapshot_restore();
                }
                Err(e)
            }
        }
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
                    eprintln!("任务 '{}' 执行失败: {}", task, e);
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
        let diff: Vec<f64> = current.arr().iter().zip(snapshot.arr().iter()).map(|(a, b)| a - b).collect();
        let l2_dist: f64 = diff.iter().map(|x| x * x).sum::<f64>().sqrt();
        -self.regularization_weight * l2_dist
    }

    pub(crate) fn update_policy(&mut self, avg_reward: f64) {
        let aging = self._aging_monitor.overall_aging();
        let interference = self._aging_monitor.interference_score;
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
        let work_dir = {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".neotrix").join("work")
        };
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
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(&home).join(".neotrix").join("cortex.json");
        let json = self.cortex.export_json();
        let data = serde_json::to_string_pretty(&json)
            .map_err(|e| NeoTrixError::Serde(format!("cortex序列化失败: {}", e)))?;
        std::fs::write(&path, &data)?;
        Ok(())
    }

    pub fn load_cortex(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(&home).join(".neotrix").join("cortex.json");
        if !path.exists() { return; }
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
                let dims: Vec<DimensionTag> = t["dimensions"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| {
                        let name = v.as_str().unwrap_or("");
                        DimensionTag::all().into_iter()
                            .find(|d| format!("{:?}", d) == name)
                    }).collect())
                    .unwrap_or_default();
                let importance = t["importance"].as_f64().unwrap_or(0.5);
                let trace = MemoryTrace::new(title, source, summary,
                    Modality::Text, dims)
                    .with_importance(importance);
                self.cortex.store(trace);
            }
        }
    }

    pub fn print_cortex_report(&self) {
        println!("{}", self.cortex.report());
    }

    pub fn cortex_recall(&mut self, query: &str, limit: usize) {
        let results = self.cortex.recall(query, limit);
        println!("🔍 联想检索 \"{}\" → {} 条结果:", query, results.len());
        for (i, (trace, score)) in results.iter().enumerate() {
            println!("  [{:.2}] {} [{}]", score, trace.title, trace.source_type);
            if i == 0 {
                println!("    📝 {}", trace.summary.chars().take(150).collect::<String>());
            }
        }
    }

    pub fn cortex_chain(&self, category: &str, limit: usize) {
        let traces = self.cortex.dimension_chain(category, limit);
        println!("📌 {} ({} 条):", category, traces.len());
        for t in &traces {
            let _dims: Vec<String> = t.dimensions.iter().map(|d| format!("{:?}", d)).collect();
//            println!("  • {} [{}] — {}", t.title, t.source_type, dims.join(", "));
        }
    }

    // ========== Init helpers (referenced by run_seal_loop & iterate) ==========
    pub fn init_reasoning_engine(&mut self) {
        // Bootstrap first-person consciousness before initializing the reasoning engine
        self.awaken_consciousness();

        let brain = std::mem::take(&mut self.brain);
        let bank = std::mem::replace(&mut self.reasoning_bank, ReasoningBank::new(100));
        let mut engine = ReasoningEngine::new(Box::new(brain), bank);
        if let Some(ref jepa) = self.nt_world_jepa {
            engine = engine.with_jepa(jepa.clone());
        }
        if let Ok(mut kb) = crate::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
            let emb_cfg = crate::neotrix::nt_memory_kb::nt_memory_embed::EmbeddingConfig::default();
            if !emb_cfg.api_key.is_empty() {
                kb = kb.with_embedding(emb_cfg);
                // Note: ensure_embeddings uses reqwest::blocking which panics
                // when called from within a tokio runtime context.
                // Embeddings are generated lazily on first semantic search.
            }
            // Load E8 transition matrix from KB (cross-session persistence)
            // Then seed with community dataset priors (12 datasets, 2M+ traces)
            // so the prediction oracle's ensemble directly benefits from community
            // distilled reasoning patterns — not just local observations.
            let mut load_tm = match kb.kv_get("e8_tm", "transition_matrix") {
                Ok(Some(json)) => {
                    match crate::core::nt_core_e8::E8TransitionMatrix::from_json_str(&json) {
                        Some(matrix) => {
                            log::info!("[E8-TM] loaded transition matrix from KB ({} total visits)",
                                matrix.visit_counts.0.iter().sum::<u64>());
                            matrix
                        }
                        None => {
                            let mut tm = crate::core::nt_core_e8::E8TransitionMatrix::new();
                            tm.init_from_trace_patterns();
                            tm
                        }
                    }
                }
                _ => {
                    let mut tm = crate::core::nt_core_e8::E8TransitionMatrix::new();
                    tm.init_from_trace_patterns();
                    tm
                }
            };
            // Seed loaded TM with community dataset priors
            crate::core::nt_core_e8::nt_core_community_ingester::seed_transition_matrix_with_community(&mut load_tm);
            engine = engine.with_observer_transition_matrix(load_tm);
            log::info!("[E8-TM] community-seeded transition matrix active");
            // Init agent session tables (SQLite-backed agent memory)
            if let Err(e) = kb.init_agent_session() {
                log::warn!("[KB] agent session init: {}", e);
            }
            engine = engine.with_kb(kb);
        }
        engine = engine.with_ewhr_bridge(E8EwhrBridge::new());
        let hyp_net = Arc::new(Mutex::new(
            crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork::new()
        ));
        engine = engine.with_hypothesis_network(hyp_net);

        // 连接 GatewayV2: 统一 LLM 网关 (断路器/限流器/回退策略/提供者池)
        let gateway = std::sync::Arc::new(create_gateway());
        engine = engine.with_gateway(gateway);

        // 从 self.default_model 设置引擎默认模型（由 entry 层从 config.toml 填充）
        if !self.default_model.is_empty() && self.default_model != "default" {
            engine.default_model.clone_from(&self.default_model);
        }

        // 连接 SAE 桥接: E8 状态 → 可解释特征提取
        let sae = std::sync::Arc::new(std::sync::RwLock::new(SparseAutoencoder::default()));
        engine = engine.with_sae_bridge(SAEBridge::new(sae));

        // L6 Self: 注入剩余的元认知/监控/压缩/蒸馏子系统
        use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
        use crate::core::nt_core_gwt::vsa_scorer::VsaContentScorer;
        use crate::core::nt_core_self::silicon_self::SiliconSelfModel;
        use crate::core::nt_core_ttc::{TtcEngine, TtcConfig};
        use crate::core::nt_core_trajectory_compress::CompressionLevel;
        use crate::core::l7_capability::nt_core_antidistil::AntiDistillationSystem;
        use crate::core::nt_io_telemetry::{ConsoleTracer, CostTracker};
        use crate::core::nt_core_prm::{ProcessRewardLearner, HeuristicCoach};
        use crate::core::nt_core_policy::E8Policy;
        use crate::core::nt_core_e8::nt_core_fable_pattern::FablePatternMatcher;
        use crate::core::nt_core_e8::nt_core_e8_prediction::{
            E8PredictionOracle, E8PredictionEnsemble, E8MctsPredictor,
        };

        engine = engine
            .with_gwt({
                let mut gwt = GlobalWorkspace::new(0.5).with_vsa_scorer(VsaContentScorer::new(64));
                gwt.register_default_specialists();
                gwt.init_oscillators(gwt.specialists.len());
                gwt
            })
            .with_silicon_self(SiliconSelfModel::new())
            .with_ttc_engine(TtcEngine::new(TtcConfig::default()))
            .with_trajectory_compressor(CompressionLevel::Medium)
            .with_anti_distillation(AntiDistillationSystem::new())
            .with_tracer(ConsoleTracer)
            .with_cost_tracker(CostTracker::new())
            .with_prm(ProcessRewardLearner::new(E8Policy::default(), Box::new(HeuristicCoach::new("prm"))))
            .with_fable_matcher(FablePatternMatcher::default())
            .with_domain_transition(crate::core::nt_core_e8::domain_transition::E8DomainTransitionModel::new(0.3))
            .with_prediction_oracle(E8PredictionOracle::new(
                E8PredictionEnsemble::default(),
                E8MctsPredictor::new(8, 50, 2.0, 0.9),
            ));

        self.reasoning_engine = Some(engine);
        self.load_e8();
    }

    pub fn save_e8(&self) {
        if let Some(ref engine) = self.reasoning_engine {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let path = std::path::PathBuf::from(&home).join(".neotrix").join("e8_state.json");
            if let Err(e) = engine.save_e8_state(&path) {
                eprintln!("[warn] 保存 E8 状态失败: {}", e);
            }
            // Persist E8 transition matrix to KB for cross-session learning
            if let Some(ref kb) = engine.kb {
                let json = engine.observer.prm.transition_matrix.as_ref()
                    .map(|tm| tm.to_json_string())
                    .unwrap_or_default();
                if !json.is_empty() {
                    if let Err(e) = kb.kv_set("e8_tm", "transition_matrix", &json) {
                        log::warn!("[E8-TM] failed to save transition matrix: {}", e);
                    }
                }
                // Also persist the domain transition model's 6 sub-matrices (one per E8TaskType).
                // Previously the E8DomainTransitionModel accumulated runtime learning (record_transition
                // calls in engine_core.rs:568-573) but was NEVER serialized -- all domain-specific
                // knowledge (e.g. "Coding domain discovered mode-26->mode-42 patterns") was lost on
                // every restart. The general observer TM persisted above captures aggregate transitions,
                // but the domain-specific sub-matrices were reconstructed from canonical chain seeds only.
                if let Some(ref dtm) = engine.domain_transition_model {
                    if let Ok(dtm_json) = serde_json::to_string(dtm) {
                        if let Err(e) = kb.kv_set("e8_tm", "domain_model", &dtm_json) {
                            log::warn!("[E8-DTM] failed to save domain model: {}", e);
                        }
                    }
                }
            }
        }
    }

    pub fn load_e8(&mut self) {
        if let Some(ref mut engine) = self.reasoning_engine {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let path = std::path::PathBuf::from(&home).join(".neotrix").join("e8_state.json");
            if path.exists() {
                if let Err(e) = engine.load_e8_state(&path) {
                    eprintln!("[warn] 加载 E8 状态失败: {}", e);
                }
            }
        }
        // KB fallback: load transition matrix that was previously saved via kv_set("e8_tm", ...).
        // Fixes the write-only sink: data was persisted but never read back on restart.
        if let Some(ref mut engine) = self.reasoning_engine {
            if let Some(ref kb) = engine.kb {
                if let Ok(Some(json)) = kb.kv_get("e8_tm", "transition_matrix") {
                    if let Some(tm) = crate::core::nt_core_e8::E8TransitionMatrix::from_json_str(&json) {
                        engine.observer = std::mem::take(&mut engine.observer).with_transition_matrix(tm);
                        log::info!("[E8-TM] loaded transition matrix from KB on restart");
                    }
                }
                // Also load the domain transition model's 6 sub-matrices from KB.
                // Restores accumulated domain-specific learning (e.g. per-task-type transition patterns)
                // that would otherwise be lost on restart. The domain model is reconstructed from
                // canonical chain seeds in init_reasoning_engine() then REPLACED here with the
                // persisted version containing all runtime transitions.
                if let Ok(Some(dtm_json)) = kb.kv_get("e8_tm", "domain_model") {
                    if let Ok(dtm) = serde_json::from_str::<crate::core::nt_core_e8::domain_transition::E8DomainTransitionModel>(&dtm_json) {
                        engine.domain_transition_model = Some(dtm);
                        log::info!("[E8-DTM] loaded domain transition model from KB on restart");
                    }
                }
            }
        }
    }

    pub fn shutdown_save_e8(&self) {
        self.save_e8();
        log::info!("[E8] state saved on shutdown");
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
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
