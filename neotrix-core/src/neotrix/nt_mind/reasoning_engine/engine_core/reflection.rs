use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_self::{AttentionDomain, ReflectionGrade, StrategyKind};
use crate::core::{MODE_DESCRIPTIONS, MODE_NAMES};

use super::CoreReasoningPlan;
use crate::neotrix::nt_mind::reasoning_types::ReasoningType;

use super::ReasoningEngine;

impl ReasoningEngine {
    /// Post-reasoning core review: feeds outcome back to GWT, ThinkingBridge, SiliconSelfModel, crystals, observer, and distiller.
    pub(super) fn core_review(&mut self, task: &str, outcome: &str, has_image: bool) {
        let success = !outcome.is_empty() && outcome.len() > 10;

        // GWT reward: broadcast outcome
        if let Some(ref mut gwt) = self.gwt {
            let reward = if success { 0.3 } else { -0.2 };
            gwt.broadcast(&format!(
                "outcome: {} (success={})",
                &task[..task.len().min(60)],
                success
            ));
            for (_, m) in gwt.specialists.iter_mut() {
                m.activation = (m.activation + reward).max(0.0).min(1.0);
            }
        }

        // ThinkingBridge → SiliconSelfModel recording
        if let Some(ref mut bridge) = self.thinking_bridge {
            bridge.observe_tool_use("reason", &outcome[..outcome.len().min(200)]);
            if success {
                bridge
                    .silicon
                    .attention_manager
                    .stimulate_domain(AttentionDomain::SelfReflection, 0.1);
            }
        } else if let Some(ref mut ss) = self.silicon_self {
            ss.observe(&format!(
                "outcome: {} (success={})",
                &task[..task.len().min(60)],
                success
            ));
        }

        // Core analysis blind-spot detection (observer)
        let strategy_dist: std::collections::HashMap<StrategyKind, usize> = {
            let mut m = std::collections::HashMap::new();
            for t in &self.traces {
                let kind = match t.reasoning_type {
                    ReasoningType::Conversation => StrategyKind::Direct,
                    ReasoningType::TaskSolving => StrategyKind::ChainOfThought,
                    ReasoningType::ErrorDebugging => StrategyKind::Reflection,
                    ReasoningType::KnowledgeQuery => StrategyKind::ToolAssisted,
                    ReasoningType::PrdGeneration => StrategyKind::Deliberate,
                    ReasoningType::General => StrategyKind::ChainOfThought,
                };
                *m.entry(kind).or_insert(0) += 1;
            }
            m
        };
        let domains = self
            .last_core_plan
            .as_ref()
            .map(|p| p.domains.clone())
            .unwrap_or_else(|| vec![AttentionDomain::Code, AttentionDomain::Planning]);
        let grades: Vec<ReflectionGrade> = self
            .traces
            .iter()
            .map(|t| {
                if t.success {
                    if t.outcome_score > 0.8 {
                        ReflectionGrade::Excellent
                    } else if t.outcome_score > 0.5 {
                        ReflectionGrade::Good
                    } else if t.outcome_score > 0.2 {
                        ReflectionGrade::Adequate
                    } else {
                        ReflectionGrade::Poor
                    }
                } else {
                    ReflectionGrade::Failed
                }
            })
            .collect();
        let errors: Vec<String> = self
            .traces
            .iter()
            .filter_map(|t| t.error_context.clone())
            .collect();
        let context_pct = (self.traces.len() as f64 / 50.0).min(1.0);
        let cognitive_spots = self.cognitive_eye.observe(
            strategy_dist,
            domains,
            context_pct,
            grades,
            errors,
            self.brain.capability(),
        );

        for spot in &cognitive_spots {
            for (dim_name, delta) in &spot.capability_deltas {
                if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                    let cur = self.brain.capability().arr()[idx];
                    self.brain.capability_mut().arr_mut()[idx] = (cur + delta).max(0.0).min(1.0);
                }
            }
        }
        if !cognitive_spots.is_empty() {
            self.brain.capability_mut().normalize();
            log::debug!(
                "[core_review] {} blind spots corrected",
                cognitive_spots.len()
            );
        }

        // Crystal outcome feedback
        if let Some(crystal_id) = self.last_crystal_used {
            if let Some(ref mut registry) = self.crystal_registry {
                registry.record_use(crystal_id, self.traces.len());
                let penalty = cognitive_spots.len() as f64 * 0.05;
                if let Some(crystal) = registry.crystals.iter_mut().find(|c| c.id == crystal_id) {
                    crystal.effectiveness = (crystal.effectiveness - penalty).max(0.1);
                    if cognitive_spots.is_empty() {
                        crystal.effectiveness = (crystal.effectiveness + 0.02).min(1.0);
                    }
                }
            }
        }

        // E8 RL policy update
        let reward = if success {
            0.5 - (cognitive_spots.len() as f64 * 0.1).min(0.4)
        } else {
            -0.3
        };
        if let Some(ref mut policy) = self.e8_policy {
            if let Some(ref plan) = self.last_core_plan {
                debug_assert_eq!(
                    policy.previous_mode(),
                    Some(plan.e8_mode),
                    "E8Policy previous_mode ({:?}) must match last_core_plan e8_mode ({:?})",
                    policy.previous_mode(),
                    plan.e8_mode,
                );
            }
            policy.update(reward);
            policy.decay_epsilon();
        }
        if let Some(ref mut learner) = self.e8_learner {
            if let Some(ref plan) = self.last_core_plan {
                learner.record(task, plan.e8_mode, reward, self.consciousness_iteration);
            }
        }

        // ReasoningDistiller: observe LLM response structure, correlate with E8 mode + GWT specialist
        let e8_mode = self
            .last_core_plan
            .as_ref()
            .map(|p| p.e8_mode.0)
            .unwrap_or(0);
        let specialist = self
            .last_core_plan
            .as_ref()
            .map(|p| p.specialist.as_str())
            .unwrap_or("unknown");
        let outcome_score = if success {
            0.5 + reward.max(0.0)
        } else {
            reward.max(0.0)
        };
        self.reasoning_distiller.observe(
            task,
            outcome,
            e8_mode,
            specialist,
            outcome_score,
            has_image,
            None,
        );

        let state_prev = vec![0.5, 0.5, 0.5];
        let state_cur = vec![
            self.current_state.mode.0 as f64 / 64.0,
            self.current_state.meta.0 as f64 / 3.0,
            outcome_score,
        ];
        let state_rnd = vec![0.0, 0.0, 0.0];
        let _msa_score = self
            .markov_check
            .evaluate(&state_prev, e8_mode, &state_cur, &state_rnd);

        // Record conversation for evolution training data
        self.record_conversation_evolution(task, outcome, success, cognitive_spots.len());
    }

    /// Record the external conversation as evolution training data.
    /// This feeds the meta-cognitive self-evolution loop.
    fn record_conversation_evolution(
        &self,
        task: &str,
        outcome: &str,
        success: bool,
        blind_spot_count: usize,
    ) {
        let Some(ref kb) = self.kb else { return };
        let plan = match self.last_core_plan {
            Some(ref p) => p,
            None => return,
        };
        let specialist_name = if let Some(ref gwt) = self.gwt {
            gwt.specialists
                .iter()
                .max_by(|a, b| {
                    a.1.activation
                        .partial_cmp(&b.1.activation)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(name, _)| name.clone())
                .unwrap_or_else(|| "unknown".into())
        } else {
            "unknown".into()
        };
        let action_count = self.traces.len() as u32;
        let error_count = self.traces.iter().filter(|t| !t.success).count() as u32;

        let record = crate::neotrix::nt_memory_kb::ConversationRecord {
            id: format!("conv_{}", self.consciousness_iteration),
            session_id: format!("session_{}", self.consciousness_iteration / 100),
            task_description: task.chars().take(120).collect(),
            user_intent: plan
                .guidance
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".into()),
            strategy_used: format!("{:?}", plan.strategy),
            e8_mode: format!("{:?}", plan.e8_mode),
            specialist_winner: specialist_name,
            actions_taken: vec![format!("plan: {:?}", plan)],
            obstacles_encountered: if !success {
                vec![format!(
                    "outcome: {}",
                    outcome.chars().take(80).collect::<String>()
                )]
            } else {
                Vec::new()
            },
            fix_patterns: if blind_spot_count > 0 {
                vec![format!("{} blind spots corrected", blind_spot_count)]
            } else {
                Vec::new()
            },
            outcome: if success {
                "success".into()
            } else {
                "failure".into()
            },
            effectiveness: if success {
                0.5 - (blind_spot_count as f64 * 0.1).min(0.4)
            } else {
                -0.3
            },
            reasoning_iterations: action_count,
            error_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };
        let _ = kb.store_conversation_record(&record);
    }

    pub fn reflect_on_trajectory(
        &mut self,
        task: &str,
    ) -> crate::neotrix::nt_core_error::NeoTrixResult<String> {
        let prev_state = self.current_state;
        self.current_state = self.current_state.reflect();
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();

        let trajectory_summary: String = self
            .state_trajectory
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "  {}. {} (meta={:02b})",
                    i, MODE_NAMES[s.mode.0 as usize], s.meta.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let ref_mode = self.current_state.mode;
        let mode_name = MODE_NAMES[ref_mode.0 as usize];
        let mode_desc = MODE_DESCRIPTIONS[ref_mode.0 as usize];

        let plan = CoreReasoningPlan {
            strategy: self.guide_strategy(mode_name, &[]),
            domains: vec![AttentionDomain::SelfReflection, AttentionDomain::Planning],
            e8_mode: ref_mode,
            mode_name: mode_name.to_string(),
            mode_desc: mode_desc.to_string(),
            crystal_used: self.last_crystal_used,
            specialist: "ReflectionEngine".to_string(),
            guidance: vec![
                "Reflect on the reasoning trajectory step by step".to_string(),
                trajectory_summary,
                "1. Was the initial mode selection optimal?".to_string(),
                "2. Was there a better path through the state space?".to_string(),
                "3. What would a complementary (opposite) approach reveal?".to_string(),
                "4. What principle can you extract for future tasks?".to_string(),
            ],
            avoid_patterns: vec![],
        };
        self.last_core_plan = Some(plan.clone());

        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(ref_mode);
        }

        let result = self.reason_with_plan(task, &plan);
        match result {
            Ok(text) => {
                self.core_review(task, &text, false);
                Ok(text)
            }
            Err(e) => {
                self.current_state = prev_state;
                self.state_trajectory.pop();
                self.core_review(task, "", false);
                Err(e)
            }
        }
    }
}
