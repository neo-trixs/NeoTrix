use super::core::truncate;
use super::core::GoalLoop;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::goal_loop::tracker::GoalTracker;
use crate::neotrix::nt_mind::goal_loop::types::{GoalIterationRecord, GoalLoopState};
use crate::neotrix::nt_mind::memory::ReasoningMemory;
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::KnowledgeSource;
use log;

impl GoalLoop {
    pub fn auto_goal_candidates(brain: &SelfIteratingBrain, count: usize) -> Vec<String> {
        let mut candidates = Vec::new();
        let base = Self::auto_goal_generate(brain);
        candidates.push(base);

        if count > 1 {
            let cap_sum: f64 = brain.brain.capability.arr().iter().sum();
            let mem_count = brain.reasoning_bank.memories().len();

            candidates.push(format!(
                "explore new knowledge domains to expand capability beyond {:.2}",
                cap_sum
            ));

            if mem_count > 20 {
                candidates.push(format!(
                    "consolidate {} memories into reusable principles",
                    mem_count
                ));
            } else {
                candidates
                    .push("gather new experiences and store as reasoning memories".to_string());
            }

            if count > 3 && brain.iteration > 100 {
                candidates.push(format!(
                    "optimize self-iteration loop at iteration #{}",
                    brain.iteration
                ));
            }
        }
        candidates
    }

    pub fn pursue_iteration(&mut self, brain: &mut SelfIteratingBrain) -> bool {
        let (desc, is_complex, exhausted, is_terminal) = match self.active_goal.as_ref() {
            Some(t) => {
                let desc = t.description.clone();
                let complex = self._is_complex_goal(&desc);
                let exhausted = t.budget_exhausted();
                let terminal = t.state.is_terminal() || t.state == GoalLoopState::Paused;
                (desc, complex, exhausted, terminal)
            }
            None => return false,
        };

        if is_terminal {
            return false;
        }

        if let Some(state) = exhausted {
            if let Some(tracker) = self.active_goal.as_mut() {
                tracker.state = state;
            }
            return false;
        }

        if !self.rate_limiter.allow_call() {
            log::info!("[goal-loop] rate limit reached, skipping iteration");
            return true;
        }

        let score_before = brain.brain.evaluate_capability(TaskType::General);

        let result = if is_complex {
            self._execute_complex_iteration(brain, &desc)
        } else {
            brain.iterate(TaskType::General)
        };

        let score_after = brain.brain.evaluate_capability(TaskType::General);
        let improved = result.improved;
        let reward = score_after - score_before;

        let tracker = match self.active_goal.as_mut() {
            Some(t) => t,
            None => return true,
        };

        tracker.iterations_completed += 1;
        tracker.total_cost_estimate += 0.002;
        tracker.tokens_consumed += 5000;
        tracker.score_current = score_after;
        tracker.last_reward = reward;
        tracker.updated_at = chrono::Utc::now().to_rfc3339();

        if improved {
            tracker.stalled_count = 0;
            self.circuit_breaker.record_success();
        } else {
            tracker.stalled_count += 1;
            self.circuit_breaker.record_stall();
        }

        tracker.history.push(GoalIterationRecord {
            iteration: tracker.iterations_completed,
            score_before,
            score_after,
            reward,
            improved,
            cost_estimate: tracker.total_cost_estimate,
            tokens_used: tracker.tokens_consumed,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        if tracker.iterations_completed > 0 && tracker.history.len() <= 10 {
            let last = tracker
                .history
                .last()
                .expect("history has entries after pushing");
            let memory = ReasoningMemory::new(
                &format!(
                    "goal_iter: {} (#{})",
                    &tracker.description[..tracker.description.len().min(50)],
                    last.iteration
                ),
                TaskType::General,
                &[],
                last.reward,
            );
            brain.reasoning_bank.store(memory);
        }

        // NEW: Check LoopTemplate exit conditions first
        if !tracker.template_exit_conditions.is_empty() {
            let mut running_loop = crate::core::nt_core_experience::loop_templates::RunningLoop {
                template_id: tracker.loop_template_id.clone().unwrap_or_default(),
                current_step: 0,
                iteration_count: tracker.iterations_completed as usize,
                started_at: 0,
                status: crate::core::nt_core_experience::loop_templates::LoopStatus::Running,
                history: vec![],
            };
            let status = crate::core::nt_core_experience::loop_templates::ConsciousnessLoopEngine::check_conditions(
                &mut running_loop,
                &tracker.template_exit_conditions,
            );
            match status {
                crate::core::nt_core_experience::loop_templates::LoopStatus::Completed => {
                    tracker.state = GoalLoopState::Achieved;
                    return false;
                }
                crate::core::nt_core_experience::loop_templates::LoopStatus::Failed(_msg) => {
                    tracker.state = GoalLoopState::Unmet;
                    return false;
                }
                crate::core::nt_core_experience::loop_templates::LoopStatus::MaxIterationsReached => {
                    tracker.state = GoalLoopState::Unmet;
                    return false;
                }
                _ => {}
            }
        }
        // Also check max_iterations from template
        if tracker.config.max_iterations > 0
            && tracker.iterations_completed >= tracker.config.max_iterations
        {
            tracker.state = GoalLoopState::Unmet;
            return false;
        }

        if improved
            && reward >= tracker.config.improvement_threshold
            && tracker.stalled_count == 0
            && tracker.iterations_completed >= 2
        {
            tracker.state = GoalLoopState::Achieved;
            brain.brain.absorb(KnowledgeSource::AutonomousGoal);
            return false;
        }

        if tracker.stalled_count >= tracker.config.stall_threshold {
            tracker.state = GoalLoopState::Unmet;
            return false;
        }

        if let Some(state) = tracker.budget_exhausted() {
            tracker.state = state;
            return false;
        }

        true
    }

    pub fn pursue_all(&mut self, brain: &mut SelfIteratingBrain, max_loops: u64) -> String {
        let description = match &self.active_goal {
            Some(g) => g
                .loop_template_id
                .as_ref()
                .and_then(|tid| {
                    crate::core::nt_core_experience::loop_templates::default_templates()
                        .into_iter()
                        .find(|t| t.id == *tid)
                })
                .map(|t| t.goal)
                .unwrap_or_else(|| g.description.clone()),
            None => return "No active goal to pursue.".to_string(),
        };

        let mut logs = Vec::new();
        for i in 0..max_loops {
            let desc = description.clone();
            let result = brain.run_seal_loop(&desc, None, None);
            match result {
                Ok(reward) => {
                    let continue_loop = self.pursue_iteration(brain);
                    logs.push(format!("  [{}/{}] reward={:.4}", i + 1, max_loops, reward));
                    if !continue_loop {
                        let final_state = self
                            .active_goal
                            .as_ref()
                            .map(|g| g.state.label())
                            .unwrap_or("unknown");
                        logs.push(format!("  → Goal state: {}", final_state));
                        break;
                    }
                }
                Err(e) => {
                    logs.push(format!("  [{}/{}] error: {}", i + 1, max_loops, e));
                    break;
                }
            }
        }

        if let Some(ref goal) = self.active_goal {
            if goal.config.e8_priority_enabled {
                if let Some(ref engine) = brain.reasoning_engine {
                    self.apply_e8_priority(engine.current_state.mode);
                    logs.push(format!(
                        "  🜁 E8 priority adjusted (state={})",
                        engine.current_state.mode.mode_name()
                    ));
                }
            }
        }

        logs.join("\n")
    }

    pub fn auto_goal_generate(brain: &SelfIteratingBrain) -> String {
        let cap_sum: f64 = brain.brain.capability.arr().iter().sum();
        let mem_count = brain.reasoning_bank.memories().len();
        let iter = brain.iteration;
        let lr = brain.brain.learning_rate;

        if cap_sum < 10.0 {
            format!("improve general capability from {:.2} to 12.0+", cap_sum)
        } else if mem_count > 50 {
            format!(
                "consolidate and distill {} memories into principles",
                mem_count
            )
        } else if lr < 0.01 {
            "optimize learning parameters for faster adaptation".to_string()
        } else if iter % 50 < 25 {
            format!("explore new knowledge to raise capability {:.2}+", cap_sum)
        } else if iter > 0 && iter % 100 < 25 {
            "distill past session patterns to improve process efficiency".to_string()
        } else {
            format!("autonomous self-improvement iteration #{}+", iter)
        }
    }

    pub fn pursue_auto_iteration(&mut self, brain: &mut SelfIteratingBrain) -> bool {
        if self.circuit_breaker.is_open() {
            let elapsed = self
                .circuit_breaker
                .last_failure
                .map(|t| std::time::Instant::now().duration_since(t).as_secs())
                .unwrap_or(0);
            if self.circuit_breaker.last_stall_reason.is_none() {
                self.circuit_breaker.last_stall_reason = Some(self._analyze_stall(brain));
            }
            let reason = self
                .circuit_breaker
                .last_stall_reason
                .as_deref()
                .unwrap_or("unknown");
            if reason != "unknown" {
                log::warn!("[bg-goal] ⚠ stall analysis: {}", reason);
            }
            log::warn!(
                "[bg-goal] ⚠ circuit breaker open ({}/{} cooldown secs), skipping",
                elapsed,
                self.circuit_breaker.cooldown_secs
            );
            return false;
        }

        self.auto_plan(brain);
        if let Some(ref plan) = self.active_plan.clone() {
            if self.check_skip_condition(brain) {
                log::info!("[bg-plan] ⏭ skipping plan '{}' (condition met)", plan.name);
                return true;
            }
            if self.check_reflection_trigger(brain.iteration as usize) {
                log::info!("[bg-plan] 🔍 meta-reflection on plan '{}'", plan.name);
            }
        }

        if self
            .active_goal
            .as_ref()
            .is_some_and(|g| g.state.is_terminal())
        {
            let _ = brain.brain.save();
            let finished = self.active_goal.take().expect("confirmed terminal above");
            let reward = match finished.state {
                GoalLoopState::Achieved => 0.9,
                GoalLoopState::Unmet => 0.2,
                GoalLoopState::BudgetLimited => 0.5,
                _ => 0.5,
            };
            let memory = ReasoningMemory::new(
                &format!(
                    "goal {}: score={:.3} ({})",
                    &finished.description[..finished.description.len().min(30)],
                    finished.score_current,
                    finished.state.label()
                ),
                TaskType::General,
                &[],
                reward,
            );
            brain.reasoning_bank.store(memory);
            log::info!(
                "[bg-goal] goal '{}' → {}",
                truncate(&finished.description, 40),
                finished.state.label()
            );
            self.completed_goals.push(finished);

            if !self.goal_queue.is_empty() {
                if let Some(next) = self.dequeue_next() {
                    let desc = next.description.clone();
                    self.active_goal = Some(next);
                    log::info!("[bg-goal] dequeued next goal: {}", truncate(&desc, 40));
                }
            }
        }

        if self.active_goal.is_none() {
            let base_desc = Self::auto_goal_generate(brain);
            let desc = if let Some(ref mot) = self.motivation_hint {
                if mot.should_explore && !base_desc.contains("explore") {
                    format!(
                        "explore new cognitive strategies and knowledge sources (R_int={:.3})",
                        mot.intrinsic_reward
                    )
                } else if mot.error_rate > 0.3 {
                    format!(
                        "investigate and debug recent poor reflection traces (error={:.0}%)",
                        mot.error_rate * 100.0
                    )
                } else if mot.confidence < 0.4 {
                    format!(
                        "run validation and reinforce weak capabilities (conf={:.0}%)",
                        mot.confidence * 100.0
                    )
                } else {
                    base_desc
                }
            } else {
                base_desc
            };
            let cap = brain
                .brain
                .evaluate_capability(crate::neotrix::nt_expert_routing::TaskType::General);
            self.start_goal(brain, &desc, Some(Self::auto_goal_config()));
            self.prioritize_from_motivation();
            if let Some(ref g) = self.active_goal {
                log::info!(
                    "[bg-goal] 🎯 auto: {} (cap={:.3}, priority={})",
                    truncate(&g.description, 50),
                    cap,
                    g.priority.label()
                );
            }

            if self.goal_queue.len() < self.max_queue {
                let candidates = Self::auto_goal_candidates(brain, 3);
                for desc in candidates {
                    if self.goal_queue.iter().any(|g| g.description == desc) {
                        continue;
                    }
                    if self
                        .active_goal
                        .as_ref()
                        .map(|g| g.description == desc)
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    let tracker = GoalTracker::new(
                        uuid::Uuid::new_v4().to_string(),
                        desc.clone(),
                        Self::auto_goal_config(),
                    );
                    self.goal_queue.push(tracker);
                }
                self.goal_queue
                    .sort_by_key(|b| std::cmp::Reverse(b.priority));
            }

            if desc.contains("distill session") || desc.contains("distill past") {
                let suggestions = self.run_distillation();
                if !suggestions.is_empty() {
                    let _suggestion_text = suggestions.join("\n");
                    let memory = ReasoningMemory::new(
                        &format!("session_distillation: {}", &desc[..desc.len().min(40)]),
                        crate::neotrix::nt_expert_routing::TaskType::General,
                        &[],
                        0.85,
                    );
                    brain.reasoning_bank.store(memory);
                    log::info!(
                        "[bg-goal] 📝 distilled {} suggestions into ReasoningBank",
                        suggestions.len()
                    );
                }
            }
        }

        self.pursue_iteration(brain)
    }
}
