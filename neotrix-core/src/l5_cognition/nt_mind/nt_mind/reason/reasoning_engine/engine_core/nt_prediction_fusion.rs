use std::collections::{BTreeMap};

use crate::l2_perception::nt_core_e8::domain_transition::{CoTLength, E8TaskType};
use crate::l2_perception::nt_core_e8::nt_core_fable_pattern::{ FablePhase};
use crate::l2_perception::nt_core_e8::nt_core_synthesis::{ SynthesisEffortTier};
use crate::l2_perception::nt_core_e8::nt_multimodal::{ MultimodalInput};
use crate::l2_perception::nt_core_e8::state_machine::E8StateMachine;
use crate::l2_perception::nt_core_e8::thinking_budget::DifficultyEstimator;

use crate::l0_substrate::nt_core_hex::{ ReasoningHexagram};
use crate::l0_substrate::nt_core_span::{
    AttributeValue, Span,
};
use super::nt_builders::ReasoningEngine;

impl ReasoningEngine {
    pub(crate) fn run_prediction_fusion(
        &mut self,
        task: &str,
        root_span: &Span,
        e8_machine: &mut E8StateMachine,
    ) {
        // E8 Prediction Oracle: compute prediction distribution for next E8 state
        // This provides a differentiable attention bridge to GWT via attention_weights()
        //
        // THREE WEAK LINKS REPAIRED:
        // 1. Domain-aware TM: E8DomainTransitionModel (170 lines, 6 sub-matrices, 12 tests)
        //    was completely orphaned — never used in production. Now domain-matrices are
        //    blended with the general matrix for task-type-specific transition priors.
        // 2. MCTS: predict_with_mcts() was constructed and injected but never called in
        //    production — only predict_distribution() (ensemble without lookahead) was used.
        //    MCTS adds 32-simulation beam search over E8 transition dynamics.
        // 3. save_e8() was defined but never called — all runtime learning lost on exit.
        //    Now persisted via the run_seal_loop() save_e8() call every 3 iterations.
        // Fable 5 effort tier: maps task difficulty + length to
        // Low/Medium/High/XHigh/Max, controlling sparse attention k,
        // MCTS simulations, and TTC rollout depth. Computed BEFORE the
        // oracle call so the MCTS budget is actually applied per effort tier
        // (previously the tier was only selected after prediction and never
        // reached the MCTS predictor, which ran with fixed budget 8/50).
        let difficulty = DifficultyEstimator::heuristic_difficulty(task, "reasoning");
        let effort_tier = self
            .effort_tier_selector
            .select_for_task(difficulty, task.len());
        self.last_effort_tier = Some(effort_tier);
        root_span.set_attribute(
            "effort_tier",
            AttributeValue::String(format!("{:?}", effort_tier)),
        );
        root_span.set_attribute(
            "effort_rollout_depth",
            AttributeValue::Int(effort_tier.rollout_depth() as i64),
        );
        root_span.set_attribute(
            "effort_sparse_k",
            AttributeValue::Int(effort_tier.sparse_k() as i64),
        );
        root_span.set_attribute(
            "effort_mcts_sims",
            AttributeValue::Int(effort_tier.mcts_simulations() as i64),
        );

        if let Some(ref mut oracle) = self.prediction_oracle {
            // Apply effort-tier budget to the MCTS predictor: Low tiers run
            // shallow/cheap lookahead, Max tiers run deep beam search. The
            // effort table (0/8/16/32/64 sims × 2/4/8/16/32 depth) mirrors
            // Qwen3/Fable 5 thinking-budget scaling. min 1 sim so Low still
            // produces a real rollout.
            let sims = effort_tier.mcts_simulations().max(1);
            let depth = effort_tier.rollout_depth().max(2);
            oracle.mcts.num_simulations = sims;
            oracle.mcts.max_depth = depth;
            root_span.set_attribute("e8_mcts_sims_effective", AttributeValue::Int(sims as i64));
            root_span.set_attribute("e8_mcts_depth_effective", AttributeValue::Int(depth as i64));
            let task_type = E8TaskType::detect(task);
            let current_mode = self.current_state.mode.0;
            let cot_length = CoTLength::from_tokens(task.len().max(100));
            let phase_step = self.state_trajectory.len().min(8);
            let current_phase = match phase_step {
                0 => FablePhase::Acknowledgment,
                1 => FablePhase::ProblemRestatement,
                2 => FablePhase::Decomposition,
                3 => FablePhase::FirstPrinciples,
                4 => FablePhase::SelfVerification,
                5 => FablePhase::AlternativeConsideration,
                6 => FablePhase::DeepDive,
                7 => FablePhase::Synthesis,
                _ => FablePhase::Conclusion,
            };

            // Domain-aware transition matrix: blend domain-specific + general matrix
            // based on detected task type. Falls back to raw observer TM if domain
            // model is not configured (backward compatible).
            let use_tm = if let Some(ref mut dtm) = self.domain_transition_model {
                let blended = dtm.blend(task_type);
                // Also record the current observer transition into the domain model
                // so each domain matrix accumulates task-type-specific patterns
                Some(blended)
            } else {
                self.observer.transition_matrix.clone()
            };

            if let Some(ref tm) = use_tm {
                // Record trajectory transitions into the domain model for future blending
                // Previously recorded a self-loop (cur, cur) which provided no meaningful
                // transition signal. Now iterates over state_trajectory windows to capture
                // actual state sequences — each (from, to) transition enriches the domain
                // sub-matrix for the detected task type.
                if let Some(ref mut dtm) = self.domain_transition_model {
                    if self.state_trajectory.len() >= 2 {
                        for w in self.state_trajectory.windows(2) {
                            dtm.record_transition(task_type, w[0].mode.0, w[1].mode.0);
                        }
                    }
                }

                // Use the real FablePatternMatcher (with community dataset weights) instead of
                // a fresh default, so the prediction oracle benefits from FableDistillationSeeder
                // knowledge across 2M+ community traces (GLM-5.2, Qwable-SDFT, Agentic-Distill, etc.)
                let pm = self.fable_matcher.clone().unwrap_or_default();
                // MCTS-enhanced prediction: blend ensemble (0.6) with MCTS beam search (0.4)
                // Previously only predict_distribution() was called in production — the MCTS
                // predictor was constructed and injected but never used. Now both are fused.
                let (dist, mcts_state, mcts_value, mcts_confidence) = oracle.predict_with_mcts(
                    tm,
                    current_mode,
                    task_type,
                    current_phase,
                    &pm,
                    cot_length,
                );
                let (best_state, best_prob) = dist.best();
                let effective = dist.effective_90pct_count();
                root_span
                    .set_attribute("e8_pred_best_state", AttributeValue::Int(best_state as i64));
                root_span.set_attribute("e8_pred_best_prob", AttributeValue::Float(best_prob));
                root_span
                    .set_attribute("e8_pred_mcts_state", AttributeValue::Int(mcts_state as i64));
                root_span.set_attribute("e8_pred_entropy", AttributeValue::Float(dist.entropy));
                root_span
                    .set_attribute("e8_pred_confidence", AttributeValue::Float(mcts_confidence));
                root_span.set_attribute("e8_pred_mcts_value", AttributeValue::Float(mcts_value));
                root_span.set_attribute(
                    "e8_pred_effective_90",
                    AttributeValue::Int(effective as i64),
                );
                root_span.set_attribute(
                    "e8_task_type",
                    AttributeValue::String(task_type.label().to_string()),
                );
                root_span.set_attribute(
                    "e8_current_phase",
                    AttributeValue::String(current_phase.label().to_string()),
                );
                root_span.set_attribute(
                    "e8_domain_blended",
                    AttributeValue::Bool(self.domain_transition_model.is_some()),
                );

                // Advance the E8 state machine toward the predicted best state so the
                // trajectory records real transitions instead of a frozen self-loop.
                // Previously E8StateMachine::transition() was never invoked in production
                // (HIGH-1 Track3): current_state stayed fixed, state_trajectory grew by
                // repeated identical states, and record_transition wrote only (cur, cur)
                // self-loops into the domain + general matrices. Now the prediction oracle
                // actually steers the state machine, respecting TTC budget gating.
                //
                // Confidence gate: only advance when the prediction carries real signal.
                // A near-uniform distribution (best_prob ~ 1/64, e.g. zero-data rows)
                // must not drive a random jump that would corrupt the learned transitions.
                let pred_hex = ReasoningHexagram::new(best_state);
                let pred_clear = best_prob >= 0.1 && best_state != current_mode;
                if pred_clear {
                    e8_machine.transition(pred_hex, task);
                    self.current_state = e8_machine.current_state;
                    // Guard against duplicate consecutive entries from the start-of-call push.
                    if self.state_trajectory.last().map(|s| s.mode.0)
                        != Some(self.current_state.mode.0)
                    {
                        self.state_trajectory.push(self.current_state);
                    }
                    root_span.set_attribute("e8_state_advanced", AttributeValue::Bool(true));
                } else {
                    root_span.set_attribute("e8_state_advanced", AttributeValue::Bool(false));
                }
                root_span.set_attribute(
                    "e8_trajectory_len",
                    AttributeValue::Int(self.state_trajectory.len() as i64),
                );

                // Store top-3 predictions as span attributes for GWT/SEAL consumption
                for (rank, &(state, prob)) in dist.top_5.iter().take(3).enumerate() {
                    root_span.set_attribute(
                        &format!("e8_pred_top{}", rank + 1),
                        AttributeValue::String(format!("s{}({:.3})", state, prob)),
                    );
                }

                // Fable 5 effort tier: computed above (before the oracle call) and
                // applied to the MCTS budget — reused here for the fusion pipeline's
                // sparse-k + thinking-budget scaling.
                let effort_tier = self
                    .last_effort_tier
                    .unwrap_or(crate::l5_cognition::nt_core_ttc::EffortTier::Medium);

                // ── 意识体内核融合管线 (Consciousness Core Fusion) ──────────────
                // Fuses the defining 2026 frontier-model innovations into a single
                // optimal prediction:
                //   1. K3 Quantile Balancing → dominance_capped_distribution (no aux loss)
                //   2. K3 AttnRes → depth-residual skip-connection across trajectory
                //   3. K3 sparse experts → effort-scaled sparse top-K
                //   4. DeepSeek-V4 mHC → Birkhoff doubly-stochastic projection (column-balanced)
                //   5. Gemini 3.6 → step-route cache reusing routing across seal-loop steps
                //   6. Qwen3/Fable 5 → effort tier (thinking budget) scales sparsity
                let traj_modes: Vec<u8> = self.state_trajectory.iter().map(|s| s.mode.0).collect();
                let synth_effort = match effort_tier {
                    crate::l5_cognition::nt_core_ttc::EffortTier::Low => SynthesisEffortTier::Low,
                    crate::l5_cognition::nt_core_ttc::EffortTier::Medium => SynthesisEffortTier::Medium,
                    crate::l5_cognition::nt_core_ttc::EffortTier::High => SynthesisEffortTier::High,
                    crate::l5_cognition::nt_core_ttc::EffortTier::XHigh => SynthesisEffortTier::XHigh,
                    crate::l5_cognition::nt_core_ttc::EffortTier::Max => SynthesisEffortTier::Max,
                };
                // Gemini 3.6 step-route cache: reuse routing decision for repeated
                // (task_type, phase, effort, source_bucket) contexts across the seal loop.
                let cache_key = crate::l2_perception::nt_core_e8::nt_core_synthesis::StepRouteCache::key(
                    task_type as u8,
                    phase_step as u8,
                    synth_effort.rank(),
                    current_mode,
                );
                let mut route_hit = false;
                // Fable 5 classifier-wrapped routing: high-risk contexts bypass
                // the aggressive frontier path through a conservative fallback.
                // Total route stats are needed on BOTH cache-hit and cache-miss:
                // the cache key carries no risk signal, so a routing cached for a
                // benign task in the same (task_type, phase, effort, bucket) would
                // otherwise be served unguarded to a high-risk request.
                let total_routes =
                    self.synthesis.step_route_cache.hits + self.synthesis.step_route_cache.misses;
                let route_hits = self.synthesis.step_route_cache.hits;
                if self.synthesis.fused_pipeline_enabled {
                    if let Some(cached_topk) = self.synthesis.step_route_cache.get(&cache_key) {
                        route_hit = true;
                        root_span
                            .set_attribute("synthesis_route_cache", AttributeValue::Bool(true));
                        // Blend cached top-k into the attention vector
                        let mut attn = vec![0.0f64; 64];
                        for (state, prob) in &cached_topk {
                            attn[(*state as usize).min(63)] = *prob;
                        }
                        // Safety classifier must run on cache hits too: re-gate the
                        // cached routing through the same frontier/conservative switch.
                        if self.synthesis.safety_router.allow_frontier(
                            task,
                            route_hits,
                            total_routes,
                        ) {
                            attn = self.synthesis.muon.condition_vector(&attn);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(true));
                        } else {
                            attn = self
                                .synthesis
                                .safety_router
                                .conservative_distribution(&attn);
                            root_span
                                .set_attribute("synthesis_safety", AttributeValue::Bool(false));
                            root_span.set_attribute(
                                "synthesis_safety_fallback",
                                AttributeValue::Bool(true),
                            );
                        }
                        self.last_e8_attention_weights = Some(attn);
                        let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                        self.last_e8_confidence = capped_confidence;
                    } else {
                        // Fused pipeline: dominance cap + AttnRes + effort sparse top-K
                        let mut fused = self.synthesis.fused_distribution(
                            tm,
                            current_mode,
                            &traj_modes,
                            synth_effort,
                        );
                        if self.synthesis.safety_router.allow_frontier(
                            task,
                            route_hits,
                            total_routes,
                        ) {
                            // DeepSeek-V4 Muon: condition the fused flow so transition
                            // columns stay well-conditioned (no rank collapse). Applied
                            // as an 8×8 Newton-Schulz orthogonalization of the
                            // 64-state attention distribution.
                            fused = self.synthesis.muon.condition_vector(&fused);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(true));
                        } else {
                            // Conservative path: damp aggressive distribution toward uniform
                            fused = self
                                .synthesis
                                .safety_router
                                .conservative_distribution(&fused);
                            root_span
                                .set_attribute("synthesis_safety", AttributeValue::Bool(false));
                            root_span.set_attribute(
                                "synthesis_safety_fallback",
                                AttributeValue::Bool(true),
                            );
                        }
                        let topk: Vec<(u8, f64)> = fused
                            .iter()
                            .enumerate()
                            .filter(|(_, &p)| p > 1e-4)
                            .map(|(i, &p)| (i as u8, p))
                            .collect();
                        self.synthesis.step_route_cache.put(cache_key, topk);
                        self.last_e8_attention_weights = Some(fused);
                        let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                        self.last_e8_confidence = capped_confidence;
                    }
                } else {
                    // Fallback: plain effort-scaled sparse attention
                    let attn = dist.attention_weights_sparse(0.8, effort_tier.sparse_k());
                    self.last_e8_attention_weights = Some(attn.to_vec());
                    let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                    self.last_e8_confidence = capped_confidence;
                }

                root_span.set_attribute("synthesis_route_hit", AttributeValue::Bool(route_hit));
                root_span.set_attribute(
                    "synthesis_route_hit_rate",
                    AttributeValue::Float(self.synthesis.step_route_cache.hit_rate()),
                );
                root_span.set_attribute(
                    "synthesis_active",
                    AttributeValue::Bool(self.synthesis.fused_pipeline_enabled),
                );
                for (k, v) in self.synthesis.telemetry() {
                    root_span.set_attribute(&k, AttributeValue::String(v));
                }

                let attn_ref = self.last_e8_attention_weights.as_deref().unwrap_or(&[]);
                let attn_owned: Vec<f64> = attn_ref.to_vec();
                root_span.set_attribute(
                    "e8_attn_entropy",
                    AttributeValue::Float(
                        attn_ref
                            .iter()
                            .filter(|&&p| p > 0.0)
                            .map(|&p| -p * p.log(2.0))
                            .sum::<f64>(),
                    ),
                );

                // Phase 10.1 — unified latent space: project the current E8 state
                // and the aggregated workspace into the shared space and surface
                // their cross-domain similarity for telemetry.
                if attn_ref.len() == 64 {
                    let e8_embed = self.unified_latent.project_e8(attn_ref);
                    let state_proj = self
                        .unified_latent
                        .project_e8_state(self.current_state.mode);
                    let cross = self.unified_latent.cosine(&e8_embed, &state_proj);
                    root_span.set_attribute("unified_e8_self_sim", AttributeValue::Float(cross));
                }

                // Phase 10.2 — latent reasoning: query episodic latent memory
                // for the current state's nearest neighbors (no text) and
                // broadcast the resulting direct E8 attention bias to GWT.
                let latent_retrieval = self.latent_reasoning.query_state(self.current_state.mode);
                if !latent_retrieval.neighbor_modes.is_empty() {
                    let (latent_weights, latent_bias) = self
                        .latent_reasoning
                        .to_gwt_attention(&latent_retrieval, 0.2);
                    root_span.set_attribute(
                        "latent_retrieval_top_sim",
                        AttributeValue::Float(
                            latent_retrieval
                                .similarities
                                .first()
                                .copied()
                                .unwrap_or(0.0),
                        ),
                    );
                    root_span.set_attribute(
                        "latent_memory_fill",
                        AttributeValue::Float(self.latent_reasoning.fill_ratio()),
                    );
                    // Merge latent episodic weights into the current-cycle fused
                    // attention so the GWT broadcast (which runs after the fusion
                    // pipeline) consumes them — previously they were set directly
                    // on the GWT *after* the broadcast already ran, i.e. dead output.
                    if let Some(attn) = self.last_e8_attention_weights.as_mut() {
                        if attn.len() == latent_weights.len() {
                            let lw = latent_bias.clamp(0.0, 0.5);
                            for (a, &l) in attn.iter_mut().zip(latent_weights.iter()) {
                                *a = *a * (1.0 - lw) + l * lw;
                            }
                            let s: f64 = attn.iter().sum();
                            if s > 0.0 {
                                for a in attn.iter_mut() {
                                    *a /= s;
                                }
                            }
                        }
                    }
                    // G24 latent feedback channel (arxiv 2608.08888): 检索结果经
                    // GWT 注意力消费后反馈回潜在记忆 — 被注意的隐藏状态强化,
                    // 形成无监督潜在闭环 (R-P79 接线, 非死代码)。
                    let feedback_updates = self.latent_reasoning.apply_feedback(&latent_retrieval);
                    root_span.set_attribute(
                        "latent_feedback_updates",
                        AttributeValue::Int(feedback_updates as i64),
                    );
                }

                // Phase 6.3 — recursive latent reasoning transformer: iterate
                // the fused attention vector (64-d E8 latent) in a continuous
                // latent space for MAX_LATENT_DEPTH steps. Deeper reasoning
                // accumulates depth-scaling reward (Thinking Pixel §3.3) which
                // is folded into the observer's step feedback, and convergence
                // is surfaced for telemetry. Wired into the reason hot path so
                // the trajectory actually advances (R-P79).
                let latent_input: Vec<f64> = attn_owned;
                if latent_input.len()
                    == crate::l2_perception::nt_core_e8::nt_latent_transformer::LATENT_HIDDEN_DIM
                {
                    let latent_state = self.latent_transformer.reason(
                        &latent_input,
                        crate::l2_perception::nt_core_e8::nt_latent_transformer::MAX_LATENT_DEPTH,
                    );
                    let depth = self.latent_transformer.current_depth();
                    let mag = self.latent_transformer.step_magnitude(&latent_state);
                    let depth_reward = self.latent_transformer.recursive_depth_reward(mag, depth);
                    let converged = self.latent_transformer.is_converged(1e-4);
                    root_span.set_attribute("lt_depth", AttributeValue::Int(depth as i64));
                    root_span.set_attribute("lt_depth_reward", AttributeValue::Float(depth_reward));
                    root_span.set_attribute("lt_converged", AttributeValue::Bool(converged));
                    // Fold the recursive-depth reward into the engine's step
                    // reward telemetry so deeper latent trajectories are
                    // observable downstream (PRM / SEAL consumers).
                    self.last_step_rewards
                        .push((format!("latent_transformer_depth_{}", depth), depth_reward));
                }

                // Phase 6.3 — sparse MoE routing: score E8 expert groups for the
                // current state + task, keep top-2 active, and mask the fused
                // attention vector so only active experts broadcast to GWT.
                // Sparsity is surfaced for telemetry; mass is conserved by the
                // router (apply_mask renormalizes). Previously SparseMoERouter
                // was only test-covered — now it gates the production attention.
                if self.last_e8_attention_weights.is_some() {
                    let cur_mode = self.current_state.mode.0;
                    let task_type =
                        crate::l2_perception::nt_core_e8::domain_transition::E8TaskType::detect(task);
                    let next_mass: Option<[f64; 8]> =
                        self.last_e8_attention_weights.as_ref().map(|w| {
                            let mut m = [0.0f64; 8];
                            for (i, &p) in w.iter().enumerate() {
                                let g = (i / 8).min(7);
                                m[g] += p;
                            }
                            m
                        });
                    let routing = self
                        .sparse_moe
                        .route(cur_mode, task_type, next_mass.as_ref());
                    let weights_ref = self.last_e8_attention_weights.as_ref();
                    if let Some(weights) = weights_ref {
                        if let Ok(arr) = <[f64; 64]>::try_from(weights.as_slice()) {
                            let masked = self.sparse_moe.apply_mask(&routing, &arr);
                            self.last_e8_attention_weights = Some(masked.to_vec());
                            root_span.set_attribute(
                                "sparse_moe_active_groups",
                                AttributeValue::String(format!("{:?}", routing.active_groups)),
                            );
                            root_span.set_attribute(
                                "sparse_moe_sparsity",
                                AttributeValue::Float(routing.sparsity()),
                            );
                            root_span.set_attribute(
                                "sparse_moe_retained_mass",
                                AttributeValue::Float(
                                    self.sparse_moe.retained_mass(&routing, &arr),
                                ),
                            );
                        }
                    }
                }

                // Phase 10.3 — multimodal fusion: encode the task as text (the
                // available modality in the reasoning loop) into the unified
                // latent space, route via GWT modal attention, and fuse. When the
                // task references an image or video file, the VisionBridge
                // extracts real pixel features upstream (the missing half of the
                // multimodal contract) so the Image modality is populated instead
                // of a no-op.
                let mut multi_input = MultimodalInput::text(task);
                let mut vision_source: Option<std::path::PathBuf> = None;
                if let Some(vid) = referenced_video_path(task) {
                    let frames = sample_video_frames(&vid, 6);
                    if let Some((feat, summary)) = aggregate_video_features(&frames) {
                        multi_input = multi_input.with_image(feat);
                        root_span.set_attribute(
                            "vision_bridge_video",
                            AttributeValue::String(vid.display().to_string()),
                        );
                        root_span.set_attribute(
                            "vision_bridge_video_frames",
                            AttributeValue::Int(summary.frames as i64),
                        );
                        root_span.set_attribute(
                            "vision_bridge_video_key_frames",
                            AttributeValue::Int(summary.key_frames as i64),
                        );
                        root_span.set_attribute(
                            "vision_bridge_video_classifications",
                            AttributeValue::String(summary.classifications),
                        );
                        vision_source = Some(vid);
                    }
                }
                if vision_source.is_none() {
                    if let Some(path) = referenced_image_path(task) {
                        if let Ok(bytes) = std::fs::read(&path) {
                            if let Ok((_evidence, feat)) =
                                crate::l2_perception::nt_core_e8::nt_multimodal::VisionBridge::analyze_cached(
                                    &bytes,
                                )
                            {
                                multi_input = multi_input.with_image(feat);
                                root_span.set_attribute(
                                    "vision_bridge_image",
                                    AttributeValue::String(path.display().to_string()),
                                );
                            }
                        }
                    }
                }
                let multi_embeds = self.multimodal.encode_all(&multi_input);
                if !multi_embeds.is_empty() {
                    let router_weights: BTreeMap<
                        crate::l5_cognition::nt_core_gwt::modality_router::Modality,
                        f64,
                    > = {
                        let mut m = BTreeMap::new();
                        if let Some(g) = &self.gwt {
                            for mod_i in crate::l5_cognition::nt_core_gwt::modality_router::Modality::ALL {
                                m.insert(mod_i, g.modality_router.weight_of(mod_i));
                            }
                        } else {
                            m.insert(
                                crate::l5_cognition::nt_core_gwt::modality_router::Modality::Text,
                                1.0,
                            );
                        }
                        m
                    };
                    let (fused, weights) = self.multimodal.fuse(&router_weights, &multi_embeds);
                    let fused_mode = self.multimodal.to_e8_mode(&fused);
                    root_span.set_attribute(
                        "multimodal_fused_mode",
                        AttributeValue::Int(fused_mode as i64),
                    );
                    root_span.set_attribute(
                        "multimodal_active_modalities",
                        AttributeValue::Int(weights.len() as i64),
                    );
                    self.latent_reasoning.record(
                        crate::l5_cognition::nt_core_hex::ReasoningHexagram::new(fused_mode),
                        self.last_e8_confidence,
                        "multimodal",
                    );
                }
            }
        }
    }
}

/// Extract a candidate image file path referenced in a task string.
///
/// Matches common image-path forms (quoted or bare `.png/.jpg/.jpeg/.webp/.gif`
/// or a `data:` style marker is handled by the caller). Returns the first path
/// that exists on disk so the VisionBridge can load real pixels.
pub(crate) fn referenced_image_path(task: &str) -> Option<std::path::PathBuf> {
    let lower = task.to_ascii_lowercase();
    for ext in [".png", ".jpg", ".jpeg", ".webp", ".gif"] {
        for (i, _) in lower.match_indices(ext) {
            let start = task[..i]
                .rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '(' || c == '[')
                .map(|p| p + 1)
                .unwrap_or(0);
            let end = task[i + ext.len()..]
                .find(|c: char| {
                    c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == ']' || c == ','
                })
                .map(|p| i + ext.len() + p)
                .unwrap_or(task.len());
            let candidate = task[start..end].trim();
            let path = std::path::PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Extract a candidate video file path referenced in a task string
/// (`.mp4/.mov/.webm/.mkv/.avi`). The caller samples a frame for the bridge.
pub(crate) fn referenced_video_path(task: &str) -> Option<std::path::PathBuf> {
    let lower = task.to_ascii_lowercase();
    for ext in [".mp4", ".mov", ".webm", ".mkv", ".avi"] {
        for (i, _) in lower.match_indices(ext) {
            let start = task[..i]
                .rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '(' || c == '[')
                .map(|p| p + 1)
                .unwrap_or(0);
            let end = task[i + ext.len()..]
                .find(|c: char| {
                    c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == ']' || c == ','
                })
                .map(|p| i + ext.len() + p)
                .unwrap_or(task.len());
            let candidate = task[start..end].trim();
            let path = std::path::PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Sample representative frames from a video via the ffmpeg subprocess.
///
/// Research findings (absorption_video.md): the *first keyframe* is often a
/// black/logo intro frame — sampling it alone is misleading. Scene-aware
/// sampling (scdet) picks one frame per visual scene; if scene detection finds
/// too few cuts (static/slideshow video) we fall back to uniform 1 FPS.
///
/// Returns PNG frame bytes, bounded to `max_frames` (default 6).
pub(crate) fn sample_video_frames(path: &std::path::Path, max_frames: usize) -> Vec<Vec<u8>> {
    let max_frames = max_frames.clamp(1, 50);
    let tmp = std::env::temp_dir();
    let base = format!(
        "nt_frame_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    // Pass 1: probe duration (ffprobe) for the uniform-fallback FPS.
    let duration: Option<f64> = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_str().unwrap_or(""),
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok());

    // Pass 2: scene-detected frames (select='gt(scene,T)' with T=0.3 default).
    let scene_out = tmp.join(format!("{}_s_%03d.png", base));
    let _ = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-i",
            path.to_str().unwrap_or(""),
            "-vf",
            "select='gt(scene,0.3)',scale=768:-2",
            "-frames:v",
            &max_frames.to_string(),
            scene_out.to_str().unwrap_or(""),
        ])
        .output();

    let mut frames: Vec<Vec<u8>> = collect_frame_pattern(&scene_out);

    // Fallback: too few scene cuts (static video) → uniform 1 FPS sampling.
    if frames.len() < 2 {
        if let Some(dur) = duration {
            if dur > 0.5 {
                let fps = (max_frames as f64 / dur).clamp(0.1, 30.0);
                let uni_out = tmp.join(format!("{}_u_%03d.png", base));
                let _ = std::process::Command::new("ffmpeg")
                    .args([
                        "-y",
                        "-v",
                        "error",
                        "-i",
                        path.to_str().unwrap_or(""),
                        "-vf",
                        &format!("fps={:.3},scale=768:-2", fps),
                        "-frames:v",
                        &max_frames.to_string(),
                        uni_out.to_str().unwrap_or(""),
                    ])
                    .output();
                frames = collect_frame_pattern(&uni_out);
            }
        }
    }

    // Cleanup all temp artifacts (scene + uniform patterns).
    if let Some(parent) = scene_out.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with(&base) && name.ends_with(".png") {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    }
    frames
}

/// Collect and sort PNG frame files matching the given ffmpeg output pattern.
pub(crate) fn collect_frame_pattern(pattern: &std::path::Path) -> Vec<Vec<u8>> {
    let parent = match pattern.parent() {
        Some(p) => p.to_path_buf(),
        None => return Vec::new(),
    };
    let stem = pattern
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&parent) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with(&stem) && name.ends_with(".png") {
                files.push(e.path());
            }
        }
    }
    files.sort();
    files
        .into_iter()
        .filter_map(|p| std::fs::read(&p).ok())
        .collect()
}

/// Aggregated multi-frame video features (research: absorption_video.md).
///
/// Decodes each sampled frame via the VisionBridge, drops Blank/near-duplicate
/// frames (phash distance ≤ 10) so logo/black intro frames don't dominate the
/// signature, then mean-pools the surviving 64-dim feature vectors and
/// re-normalizes into the unified latent space. Classification counts let the
/// reasoner see *how* the video looks (scenes vs document-like vs photo).
pub(crate) struct VideoFeatureSummary {
    pub(crate) frames: usize,
    pub(crate) key_frames: usize,
    pub(crate) classifications: String,
}

pub(crate) fn aggregate_video_features(frames: &[Vec<u8>]) -> Option<(Vec<f64>, VideoFeatureSummary)> {
    use crate::l2_perception::nt_core_e8::nt_multimodal::{ImageClass, VisionBridge};
    const PHASH_DUP_THRESHOLD: u32 = 10;

    let mut pooled: Vec<f64> = Vec::new();
    let mut kept: Vec<u64> = Vec::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut key_frames = 0usize;

    for bytes in frames {
        let (ev, feat) = VisionBridge::analyze_cached(bytes).ok()?;
        if ev.classification == ImageClass::Blank {
            continue; // black-screen/logo intro frames add no visual signal.
        }
        let is_dup = kept
            .iter()
            .any(|&h| VisionBridge::phash_distance(h, ev.phash) <= PHASH_DUP_THRESHOLD);
        if is_dup {
            continue;
        }
        kept.push(ev.phash);
        key_frames += 1;
        *class_counts
            .entry(ev.classification.as_str().to_string())
            .or_insert(0) += 1;
        if pooled.is_empty() {
            pooled = feat;
        } else {
            for (p, f) in pooled.iter_mut().zip(feat.iter()) {
                *p += f;
            }
        }
    }

    if key_frames == 0 {
        return None;
    }
    let norm: f64 = pooled.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 1e-12 {
        for p in pooled.iter_mut() {
            *p /= norm;
        }
    }
    let classifications = if class_counts.is_empty() {
        "none".to_string()
    } else {
        class_counts
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    };
    Some((
        pooled,
        VideoFeatureSummary {
            frames: frames.len(),
            key_frames,
            classifications,
        },
    ))
}
