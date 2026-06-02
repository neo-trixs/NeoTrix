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
use crate::neotrix::error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::signal::select::SelectableOperator;
use crate::neotrix::signal::SelectiveState;


type BatchTask<'a> = &'a [(String, Option<Vec<f64>>, Option<f64>)];

impl SelfIteratingBrain {
    pub fn run_seal_loop_pipeline(&mut self, task: &str, task_embedding: Option<Vec<f64>>, external_reward: Option<f64>) -> NeoTrixResult<f64> {
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
        if self.iteration % 3 == 0 {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap();
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
        if self.iteration % 3 == 0 {
            if let Some(ref crypto_arc) = self.nt_act_crypto {
                let mut crypto = crypto_arc.lock().unwrap();
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
                println!("    📝 {}", &trace.summary[..trace.summary.len().min(150)]);
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
        let brain = std::mem::take(&mut self.brain);
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
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let path = std::path::PathBuf::from(&home).join(".neotrix").join("e8_state.json");
            if let Err(e) = engine.save_e8_state(&path) {
                eprintln!("[warn] 保存 E8 状态失败: {}", e);
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
