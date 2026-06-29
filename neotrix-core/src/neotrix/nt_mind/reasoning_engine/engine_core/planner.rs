use crate::core::nt_core_gwt::module_def::SpecialistType;
use crate::core::nt_core_self::{AttentionDomain, SkillCrystal, StrategyKind};
use crate::core::{optimal_starting_mode, rank_modes_for_task, ModeFit, ReasoningHexagram};
use crate::neotrix::nt_world_jepa::WorldModelState;

use super::CoreReasoningPlan;

use super::ReasoningEngine;

fn specialist_to_domains(st: &SpecialistType) -> Vec<AttentionDomain> {
    match st {
        SpecialistType::PatternMatcher => vec![
            AttentionDomain::PatternMatch,
            AttentionDomain::Code,
            AttentionDomain::Semantic,
        ],
        SpecialistType::AnomalyDetector => vec![
            AttentionDomain::RiskAssessment,
            AttentionDomain::Code,
            AttentionDomain::PatternMatch,
        ],
        SpecialistType::KnowledgeRetriever => vec![
            AttentionDomain::Semantic,
            AttentionDomain::PatternMatch,
            AttentionDomain::Temporal,
        ],
        SpecialistType::CodeAnalyzer => vec![
            AttentionDomain::Code,
            AttentionDomain::PatternMatch,
            AttentionDomain::RiskAssessment,
        ],
        SpecialistType::Planner => vec![
            AttentionDomain::Planning,
            AttentionDomain::GoalAlignment,
            AttentionDomain::Temporal,
        ],
        SpecialistType::KnowledgeIntegrator => vec![
            AttentionDomain::Semantic,
            AttentionDomain::Creativity,
            AttentionDomain::Temporal,
        ],
        SpecialistType::GoalPrioritizer => vec![
            AttentionDomain::GoalAlignment,
            AttentionDomain::Planning,
            AttentionDomain::Temporal,
        ],
        SpecialistType::RiskAssessor => vec![
            AttentionDomain::RiskAssessment,
            AttentionDomain::GoalAlignment,
            AttentionDomain::Code,
        ],
        SpecialistType::CreativityGenerator => vec![
            AttentionDomain::Creativity,
            AttentionDomain::Semantic,
            AttentionDomain::PatternMatch,
        ],
        SpecialistType::ReflectionEngine => vec![
            AttentionDomain::SelfReflection,
            AttentionDomain::Code,
            AttentionDomain::Planning,
        ],
        SpecialistType::MetaCognitionAnalyst => vec![
            AttentionDomain::SelfReflection,
            AttentionDomain::GoalAlignment,
            AttentionDomain::Planning,
        ],
        SpecialistType::AISecurity => vec![
            AttentionDomain::RiskAssessment,
            AttentionDomain::PatternMatch,
            AttentionDomain::Code,
        ],
        SpecialistType::ImageGenerator => vec![
            AttentionDomain::Creativity,
            AttentionDomain::PatternMatch,
            AttentionDomain::Semantic,
        ],
    }
}

impl ReasoningEngine {
    /// Core-First Reasoning: generate a structured reasoning plan from core analysis,
    /// before any LLM call. Uses GWT resonance + SiliconSelfModel + ThinkingBridge +
    /// crystal registry + E8 state.
    pub fn plan_reasoning(&mut self, task: &str) -> CoreReasoningPlan {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();
        self.consciousness_iteration += 1;

        let mode_name = mode.mode_name().to_string();
        let mode_desc = mode.mode_description().to_string();

        let mut domains = vec![AttentionDomain::Code, AttentionDomain::Planning];
        let mut specialist_label = "None".to_string();
        let mut guidance = Vec::new();
        let mut avoid_patterns = Vec::new();
        let mut matched_strategies: Vec<StrategyKind> = Vec::new();

        // ── Phase 0: World Model state injection ──
        if let Some(ref jepa) = self.jepa {
            let default_wm = WorldModelState::new();
            let current_state = self.nt_world_model.as_ref().unwrap_or(&default_wm);
            let predicted = jepa.predict_next_state(current_state);
            guidance.push(format!(
                "World model predicts: CPU at {:.0}%, errors at {:.1}%, queue depth {}",
                predicted.cpu_usage * 100.0,
                predicted.error_rate * 100.0,
                predicted.task_queue_depth,
            ));
            if let Some(ref current) = self.nt_world_model {
                let trends = predicted.describe_trend(current);
                guidance.push(format!(
                    "Current environment: CPU {:.0}%, memory {:.0}% available, iteration {}",
                    current.cpu_usage * 100.0,
                    current.memory_available * 100.0,
                    current.iteration_count,
                ));
                for trend in &trends {
                    guidance.push(format!("Trend: {}", trend));
                }
            }
        }

        // ── Phase 0.5: ReasoningDistiller recommendation ──
        if self.reasoning_distiller.total_observations() >= 3 {
            if let Some((rec_mode, rec_reason, top_approaches)) =
                self.reasoning_distiller.recommend_mode(task)
            {
                guidance.push(format!(
                    "ReasoningDistiller recommends E8 mode {} ({}) — top approaches: {}",
                    rec_mode,
                    rec_reason,
                    top_approaches.join(", "),
                ));
            }
            let profile = self.reasoning_distiller.mode_profile(mode.0);
            for line in &profile {
                guidance.push(format!("[distiller] {}", line));
            }
        }

        // ── Phase 1: GWT resonance analysis ──
        if let Some(ref mut gwt) = self.gwt {
            gwt.broadcast(&format!("task: {task}"));
            if gwt.specialists.is_empty() {
                gwt.register_default_specialists();
            }
            let states = crate::core::default_specialist_states();

            // ── KB-enriched broadcast ──
            let broadcast_content = match self.kb {
                Some(ref kb) => {
                    let kb_results = kb.query_broadcast_context(task, 5).unwrap_or_default();
                    if kb_results.is_empty() {
                        format!("task analysis: {task}")
                    } else {
                        let ctx: Vec<&str> =
                            kb_results.iter().map(|r| r.node.title.as_str()).collect();
                        format!(
                            "task analysis: {task}\nKnowledge context: {}",
                            ctx.join(", ")
                        )
                    }
                }
                None => format!("task analysis: {task}"),
            };
            gwt.resonant_broadcast(&broadcast_content, &states);

            if let Some(winner) = gwt.resonance_winner() {
                specialist_label = format!("{:?}", winner.specialist_type);
                domains = specialist_to_domains(&winner.specialist_type);
                guidance.push(format!(
                    "GWT specialist consensus: {:?} leads the resonance",
                    winner.specialist_type
                ));
            }

            gwt.decay_all(0.1);
        }

        // ── Phase 2: SiliconSelfModel trigger matching ──
        if let Some(ref mut ss) = self.silicon_self {
            ss.observe(&format!("plan: {task}"));
            matched_strategies = ss.match_triggers(task);
            for strat in &matched_strategies {
                guidance.push(format!("SiliconSelf trigger match: {:?}", strat));
            }
            let state = ss.current_state();
            guidance.push(format!(
                "SiliconSelf context: {:.0}% used, {} patterns active",
                state.context_usage * 100.0,
                state.active_patterns,
            ));
        }

        // ── Phase 3: ThinkingBridge observation ──
        if let Some(ref mut bridge) = self.thinking_bridge {
            bridge.observe_task(task);
            bridge.run_reflection_cycle();
            let profile = bridge.attention_profile_summary();
            guidance.push(format!("ThinkingBridge: {}", profile));
        }

        // ── Phase 4: Task-type-specific guidance ──
        {
            let lower = task.to_lowercase();
            if lower.contains("bug") || lower.contains("error") || lower.contains("fix") {
                guidance.push("Trace the root cause before proposing a fix".to_string());
                guidance.push("Check edge cases that could mask the real issue".to_string());
            }
            if lower.contains("design") || lower.contains("architect") || lower.contains("plan") {
                guidance.push("Consider trade-offs between competing approaches".to_string());
                guidance.push("Surface assumptions that constrain the design space".to_string());
            }
            if lower.contains("review") || lower.contains("audit") || lower.contains("check") {
                guidance.push("Inspect for correctness, safety, and performance".to_string());
                guidance
                    .push("Flag patterns that deviate from established conventions".to_string());
            }
        }

        // ── Phase 5: Crystal-guided strategy annotation ──
        if let Some(crystal_id) = self.last_crystal_used {
            guidance.push(format!(
                "Crystallized skill #{} guides this reasoning",
                crystal_id
            ));
        }

        // ── Phase 6: CognitiveObserver pre-check ──
        let known_spots = self.cognitive_eye.total_observations;
        if known_spots > 5 {
            avoid_patterns.push("Avoid over-indexing on the most recent trace".to_string());
        }
        if known_spots > 10 {
            avoid_patterns.push("Beware of confirmation bias from past successes".to_string());
        }

        // ── Phase 7: KB-enriched guidance ──
        if let Some(ref kb) = self.kb {
            let e8_tags = mode.task_recommendation();
            for tag in e8_tags.iter().take(3) {
                if let Ok(results) = kb.recommend_for_e8_mode(tag, 3) {
                    for r in &results {
                        guidance.push(format!("KB: {} — {}", tag, r.node.title));
                    }
                }
            }
        }

        // Decide strategy: mode_name match first, then SiliconSelf triggers, then CoT fallback
        let strategy = self.guide_strategy(&mode_name, &matched_strategies);

        // Sync the selected mode to E8Policy so its Q-update targets the correct mode
        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(mode);
        }

        let plan = CoreReasoningPlan {
            strategy,
            domains,
            e8_mode: mode,
            mode_name,
            mode_desc,
            crystal_used: self.last_crystal_used,
            specialist: specialist_label,
            guidance,
            avoid_patterns,
        };
        self.last_core_plan = Some(plan.clone());
        plan
    }

    /// Resolve strategy from mode name and SiliconSelf matched triggers.
    pub(super) fn guide_strategy(&self, mode_name: &str, matched: &[StrategyKind]) -> StrategyKind {
        let from_mode = match mode_name {
            "Reflection" => StrategyKind::Reflection,
            "ChainOfThought" | "Chain of Thought" => StrategyKind::ChainOfThought,
            "Deliberate" => StrategyKind::Deliberate,
            "Decompose" | "Decomposition" | "RecursiveDecomposition" => {
                StrategyKind::RecursiveDecomposition
            }
            "Direct" | "Conversation" => StrategyKind::Direct,
            "ToolAssisted" => StrategyKind::ToolAssisted,
            "IterativeRefinement" => StrategyKind::IterativeRefinement,
            _ => StrategyKind::ChainOfThought,
        };
        if let Some(first) = matched.first() {
            if *first != StrategyKind::Direct {
                return *first;
            }
        }
        from_mode
    }

    /// Select reasoning mode guided by crystallized skills when available.
    /// Falls back to default keyword-based optimal_starting_mode when no crystals match.
    pub(crate) fn select_mode(&mut self, task: &str) -> ReasoningHexagram {
        let default_mode = optimal_starting_mode(task);
        self.last_crystal_used = None;
        let Some(ref registry) = self.crystal_registry else {
            return default_mode;
        };
        let lower = task.to_lowercase();

        let matching: Vec<&SkillCrystal> = registry
            .crystals
            .iter()
            .filter(|c| c.effectiveness > 0.4)
            .filter(|c| {
                c.tags.iter().any(|t| lower.contains(&t.to_lowercase()))
                    || c.name
                        .to_lowercase()
                        .split_whitespace()
                        .any(|w| lower.contains(w))
            })
            .collect();

        if matching.is_empty() {
            return default_mode;
        }

        let best = matching
            .iter()
            .max_by(|a, b| {
                a.effectiveness
                    .partial_cmp(&b.effectiveness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        self.last_crystal_used = Some(best.id);

        let mut best_score = 0u32;
        let mut best_idx = 0u8;
        for bits in 0..64u8 {
            let state = ReasoningHexagram(bits);
            let keywords = state.task_recommendation();
            let keyword_score: u32 = keywords
                .iter()
                .map(|kw| if lower.contains(kw) { 1 } else { 0 })
                .sum();
            let crystal_boost: u32 = best
                .tags
                .iter()
                .filter(|t| keywords.iter().any(|kw| kw.contains(&t.to_lowercase())))
                .count() as u32
                * 2;
            let total = keyword_score + crystal_boost;
            if total > best_score {
                best_score = total;
                best_idx = bits;
            }
        }
        ReasoningHexagram(best_idx)
    }

    pub fn navigate_to_state(&self, task: &str) -> ReasoningHexagram {
        optimal_starting_mode(task)
    }

    pub fn rank_states_for_task(&self, task: &str, top_k: usize) -> Vec<ModeFit> {
        rank_modes_for_task(task, top_k)
    }
}
