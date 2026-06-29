use super::core::GoalLoop;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::goal_loop::types::{PlanLevel, PlanTemplate};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::stats::IterationResult;
use crate::neotrix::nt_mind::KnowledgeSource;
use log;

impl GoalLoop {
    pub(crate) fn _is_complex_goal(&self, description: &str) -> bool {
        let keywords = [
            "design", "analyze", "compare", "research", "multiple", "both", "架构", "设计", "分析",
            "对比", "研究", "多个",
        ];
        let lower = description.to_lowercase();
        keywords.iter().any(|k| lower.contains(k))
    }

    fn _decompose_goal(&self, description: &str) -> Vec<String> {
        let lower = description.to_lowercase();
        let has_compare = lower.contains(" vs ")
            || lower.contains(" versus ")
            || lower.starts_with("compare ")
            || lower.starts_with("对比")
            || lower.contains("x vs ")
            || lower.contains(" 与 ");

        if has_compare {
            let separators = [" and ", " then ", " also "];
            let remaining = description.to_string();
            for sep in &separators {
                if remaining.contains(sep) {
                    let parts: Vec<&str> = remaining.split(sep).collect();
                    let mut sub_goals = Vec::new();
                    for part in parts {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() && trimmed.len() > 5 {
                            sub_goals.push(trimmed.to_string());
                        }
                    }
                    if !sub_goals.is_empty() {
                        return sub_goals;
                    }
                }
            }
            return vec![description.to_string()];
        }

        let separators = [" and ", " then ", " also ", " + ", ", ", "; "];
        for sep in &separators {
            if description.contains(sep) {
                let parts: Vec<&str> = description.split(sep).collect();
                let sub_goals: Vec<String> = parts
                    .iter()
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty() && p.len() > 5)
                    .collect();
                if !sub_goals.is_empty() {
                    return sub_goals;
                }
            }
        }

        vec![description.to_string()]
    }

    pub(crate) fn _analyze_stall(&self, brain: &SelfIteratingBrain) -> String {
        let mem_count = brain.reasoning_bank.memories().len();
        let cap_sum: f64 = brain.brain.capability.arr().iter().sum();
        let mut reasons = Vec::new();

        if mem_count > 100 {
            reasons.push("memory_overload");
        }
        if cap_sum > 18.0 {
            reasons.push("capability_saturation");
        }
        if brain.iteration > 1000 {
            reasons.push("high_iteration_count");
        }

        if reasons.is_empty() {
            reasons.push("unknown");
        }
        reasons.join(",")
    }

    fn _run_single_goal(&mut self, brain: &mut SelfIteratingBrain, task: &str) -> IterationResult {
        if let Some(ref mut orch) = self.orchestrator {
            match orch.run_recursive_loop(task) {
                Ok(_result) => {
                    let score_before = brain.brain.evaluate_capability(TaskType::General);
                    brain.brain.absorb(KnowledgeSource::AutonomousGoal);
                    let score_after = brain.brain.evaluate_capability(TaskType::General);
                    return IterationResult {
                        iteration: brain.iteration,
                        task_type: TaskType::General,
                        score_before,
                        score_after,
                        improved: score_after > score_before,
                        absorbed_count: brain.brain.total_absorb_count,
                    };
                }
                Err(e) => {
                    log::warn!(
                        "[goal-loop] orchestrator failed ({}), falling back to agent team",
                        e
                    );
                }
            }
        }

        let score_before = brain.brain.evaluate_capability(TaskType::General);

        if let Some(ref team) = self.agent_team {
            match team.lock() {
                Ok(mut t) => {
                    let results = t.execute(task);
                    let successes = results.iter().filter(|r| r.success).count();
                    let total = results.len();
                    log::info!(
                        "[goal-loop] agent team '{}': {}/{} successful",
                        t.name,
                        successes,
                        total
                    );
                    let delta = if successes > 0 {
                        0.02 * successes as f64
                    } else {
                        0.0
                    };
                    if delta > 0.0 {
                        let current = brain.brain.capability.quality_gates();
                        brain
                            .brain
                            .capability
                            .set_quality_gates((current + delta).min(1.0));
                        brain.brain.total_absorb_count += 1;
                    }
                }
                Err(e) => {
                    log::error!("[goal-loop] agent team lock failed: {}", e);
                }
            }
        }

        let result = brain.iterate(TaskType::General);
        IterationResult {
            iteration: result.iteration,
            task_type: result.task_type,
            score_before,
            score_after: result.score_after,
            improved: result.score_after > score_before,
            absorbed_count: result.absorbed_count,
        }
    }

    pub(crate) fn _execute_complex_iteration(
        &mut self,
        brain: &mut SelfIteratingBrain,
        task: &str,
    ) -> IterationResult {
        let sub_goals = self._decompose_goal(task);
        if sub_goals.len() > 1 {
            let n = sub_goals.len();
            let mut total_before = 0.0;
            let mut total_after = 0.0;
            let mut any_improved = false;
            for sg in &sub_goals {
                let result = self._run_single_goal(brain, sg);
                total_before += result.score_before;
                total_after += result.score_after;
                if result.improved {
                    any_improved = true;
                }
            }
            return IterationResult {
                iteration: brain.iteration,
                task_type: TaskType::General,
                score_before: total_before / n as f64,
                score_after: total_after / n as f64,
                improved: any_improved,
                absorbed_count: brain.brain.total_absorb_count,
            };
        }

        self._run_single_goal(brain, task)
    }

    pub fn create_macro_plan(&self, brain: &SelfIteratingBrain) -> PlanTemplate {
        let base = Self::auto_goal_generate(brain);
        let cap_sum: f64 = brain.brain.capability.arr().iter().sum();
        let mem_count = brain.reasoning_bank.memories().len();

        let meso_plans = vec![
            PlanTemplate {
                level: PlanLevel::Meso,
                name: "assess".into(),
                description: format!(
                    "assess current capability ({:.2}) and memory ({})",
                    cap_sum, mem_count
                ),
                sub_plans: vec![
                    PlanTemplate {
                        level: PlanLevel::Micro,
                        name: "capability scan".into(),
                        description: "scan all capability dimensions for weaknesses".into(),
                        sub_plans: vec![],
                        skip_condition: None,
                        reflection_trigger: None,
                        expected_duration_cycles: 3,
                        completion_criteria: Some("all dims logged".into()),
                    },
                    PlanTemplate {
                        level: PlanLevel::Micro,
                        name: "memory review".into(),
                        description: "review recent reasoning memories for patterns".into(),
                        sub_plans: vec![],
                        skip_condition: Some(format!(
                            "memory < {}",
                            (mem_count as f64 * 0.5).max(5.0) as usize
                        )),
                        reflection_trigger: None,
                        expected_duration_cycles: 2,
                        completion_criteria: Some("patterns extracted".into()),
                    },
                ],
                skip_condition: Some(format!("capability > {:.1}", (cap_sum + 5.0).min(25.0))),
                reflection_trigger: None,
                expected_duration_cycles: 5,
                completion_criteria: Some("baseline established".into()),
            },
            PlanTemplate {
                level: PlanLevel::Meso,
                name: "execute".into(),
                description: base,
                sub_plans: vec![
                    PlanTemplate {
                        level: PlanLevel::Micro,
                        name: "action".into(),
                        description: "execute primary goal action".to_string(),
                        sub_plans: vec![],
                        skip_condition: None,
                        reflection_trigger: Some("after 5 iterations".into()),
                        expected_duration_cycles: 10,
                        completion_criteria: Some("goal progressed".into()),
                    },
                    PlanTemplate {
                        level: PlanLevel::Micro,
                        name: "verify".into(),
                        description: "verify progress and measure improvement".into(),
                        sub_plans: vec![],
                        skip_condition: None,
                        reflection_trigger: None,
                        expected_duration_cycles: 3,
                        completion_criteria: Some("delta measured".into()),
                    },
                ],
                skip_condition: None,
                reflection_trigger: Some("after 10 iterations".into()),
                expected_duration_cycles: 13,
                completion_criteria: Some("iteration complete".into()),
            },
        ];

        PlanTemplate {
            level: PlanLevel::Macro,
            name: format!("macro plan #{}", brain.iteration),
            description: format!("autonomous improvement cycle at iter {}", brain.iteration),
            sub_plans: meso_plans,
            skip_condition: None,
            reflection_trigger: Some("after 20 iterations".into()),
            expected_duration_cycles: 20,
            completion_criteria: None,
        }
    }

    pub fn drill_down(&mut self) -> Option<&PlanTemplate> {
        let sub_plans = self.active_plan.as_ref()?.sub_plans.clone();
        if sub_plans.is_empty() {
            return self.active_plan.as_ref();
        }
        let current = self.active_plan.take()?;
        self.plan_stack.push(current);
        self.active_plan = Some(
            sub_plans
                .into_iter()
                .next()
                .expect("sub_plans non-empty checked above"),
        );
        self.active_plan.as_ref()
    }

    pub fn check_skip_condition(&self, brain: &SelfIteratingBrain) -> bool {
        let plan = match self.active_plan.as_ref() {
            Some(p) => p,
            None => return false,
        };
        let cond = match plan.skip_condition.as_ref() {
            Some(c) => c,
            None => return false,
        };

        if cond.starts_with("capability > ") {
            let threshold: f64 = cond
                .trim_start_matches("capability > ")
                .parse()
                .unwrap_or(f64::MAX);
            let cap_sum: f64 = brain.brain.capability.arr().iter().sum();
            return cap_sum > threshold;
        }
        if cond.starts_with("memory < ") {
            let threshold: usize = cond.trim_start_matches("memory < ").parse().unwrap_or(0);
            let mem_count = brain.reasoning_bank.memories().len();
            return mem_count < threshold;
        }
        if cond.starts_with("iteration > ") {
            let threshold: u64 = cond
                .trim_start_matches("iteration > ")
                .parse()
                .unwrap_or(u64::MAX);
            return brain.iteration > threshold;
        }
        false
    }

    pub fn check_reflection_trigger(&self, iteration: usize) -> bool {
        let plan = match self.active_plan.as_ref() {
            Some(p) => p,
            None => return false,
        };
        let trigger = match plan.reflection_trigger.as_ref() {
            Some(t) => t,
            None => return false,
        };

        if trigger.starts_with("after ") && trigger.contains(" iterations") {
            let num_str = trigger
                .trim_start_matches("after ")
                .split_whitespace()
                .next()
                .unwrap_or("0");
            let interval: usize = num_str.parse().unwrap_or(usize::MAX);
            return interval > 0 && iteration > 0 && iteration.is_multiple_of(interval);
        }
        false
    }

    pub fn plan_summary(&self) -> String {
        let plan = match self.active_plan.as_ref() {
            Some(p) => p,
            None => return "No active plan.".to_string(),
        };

        let mut lines = Vec::new();
        lines.push(format!(
            "📋 {}: {} ({})",
            plan.level.label(),
            plan.name,
            plan.description
        ));
        for meso in &plan.sub_plans {
            let skip = meso.skip_condition.as_ref().map(|_| " ⏭").unwrap_or("");
            let refl = meso
                .reflection_trigger
                .as_ref()
                .map(|_| " 🔍")
                .unwrap_or("");
            lines.push(format!(
                "  ├─ {} {}{}{}",
                meso.level.label(),
                meso.name,
                skip,
                refl
            ));
            for micro in &meso.sub_plans {
                let mskip = micro.skip_condition.as_ref().map(|_| " ⏭").unwrap_or("");
                let mrefl = micro
                    .reflection_trigger
                    .as_ref()
                    .map(|_| " 🔍")
                    .unwrap_or("");
                lines.push(format!(
                    "  │  └─ {} {}{}{}",
                    micro.level.label(),
                    micro.name,
                    mskip,
                    mrefl
                ));
            }
        }
        lines.push(format!(
            "  └─ {} cycles est.",
            plan.expected_duration_cycles
        ));
        lines.join("\n")
    }

    pub fn auto_plan(&mut self, brain: &SelfIteratingBrain) {
        if self.active_plan.is_none() {
            let plan = self.create_macro_plan(brain);
            log::info!("[bg-plan] 📋 created macro plan '{}'", plan.name);
            self.active_plan = Some(plan);
            self.plan_stack.clear();
            return;
        }

        if self.check_skip_condition(brain) {
            if let Some(ref plan) = self.active_plan.clone() {
                if !plan.sub_plans.is_empty() {
                    let sub = plan.sub_plans.clone();
                    self.plan_stack.pop();
                    self.active_plan = Some(
                        sub.into_iter()
                            .next()
                            .expect("sub_plans non-empty checked above"),
                    );
                    if let Some(ref p) = self.active_plan {
                        log::info!("[bg-plan] ⏭ skipping to next sub-plan '{}'", p.name);
                    }
                }
            }
        }
    }
}
