use std::collections::HashSet;

use super::super::self_iterating::ReasoningBrain;
use super::types::*;

pub struct SelfQuestioningPipeline {
    pub config: SelfQuestionConfig,
}

impl SelfQuestioningPipeline {
    pub fn new(config: SelfQuestionConfig) -> Self {
        Self { config }
    }

    pub fn run_round(
        &self,
        brain: &mut ReasoningBrain,
        _bank: &mut super::super::memory::ReasoningBank,
    ) -> SelfQuestionRoundResult {
        let domain = "code-analysis";
        let profile = self.build_profile(brain, domain);
        let trajectory = self.explore(brain, &profile);
        let generated = self.synthesize_tasks(&trajectory);

        let num_generated = generated.len();
        let curated = self.curate(brain, generated);
        let num_curated = curated.len();

        let avg_judge = if curated.is_empty() {
            0.0
        } else {
            curated.iter().map(|c| c.llm_judge_score).sum::<f64>() / curated.len() as f64
        };
        let sum_reward: f64 = curated.iter().map(|c| c.proxy_reward).sum();

        SelfQuestionRoundResult {
            num_tasks_generated: num_generated,
            num_tasks_curated: num_curated,
            avg_judge_score: avg_judge,
            sum_proxy_reward: sum_reward,
        }
    }

    pub fn build_profile(&self, _brain: &ReasoningBrain, domain: &str) -> EnvironmentProfile {
        match domain {
            "code-analysis" => EnvironmentProfile {
                domain: domain.to_string(),
                entities: vec![
                    EntityDesc {
                        name: "Function".into(),
                        attributes: vec![
                            "signature".into(),
                            "body".into(),
                            "complexity".into(),
                            "dependencies".into(),
                        ],
                        parent: None,
                    },
                    EntityDesc {
                        name: "Variable".into(),
                        attributes: vec![
                            "name".into(),
                            "type".into(),
                            "scope".into(),
                            "mutability".into(),
                        ],
                        parent: None,
                    },
                    EntityDesc {
                        name: "Module".into(),
                        attributes: vec!["path".into(), "exports".into(), "imports".into()],
                        parent: None,
                    },
                ],
                operations: vec![
                    OperationDesc {
                        name: "refactor".into(),
                        input_params: vec!["target".into(), "strategy".into()],
                        output_effect: "modified source".into(),
                    },
                    OperationDesc {
                        name: "analyze".into(),
                        input_params: vec!["target".into(), "depth".into()],
                        output_effect: "analysis report".into(),
                    },
                    OperationDesc {
                        name: "test".into(),
                        input_params: vec!["target".into(), "framework".into()],
                        output_effect: "test results".into(),
                    },
                ],
                constraints: vec![
                    "must compile".into(),
                    "backward compatible".into(),
                    "no unsafe code".into(),
                ],
            },
            "web-design" => EnvironmentProfile {
                domain: domain.to_string(),
                entities: vec![
                    EntityDesc {
                        name: "Component".into(),
                        attributes: vec![
                            "props".into(),
                            "state".into(),
                            "children".into(),
                            "style".into(),
                        ],
                        parent: None,
                    },
                    EntityDesc {
                        name: "Page".into(),
                        attributes: vec!["route".into(), "layout".into(), "sections".into()],
                        parent: None,
                    },
                ],
                operations: vec![OperationDesc {
                    name: "compose".into(),
                    input_params: vec!["components".into(), "layout".into()],
                    output_effect: "rendered page".into(),
                }],
                constraints: vec!["responsive".into(), "accessible".into()],
            },
            _ => EnvironmentProfile {
                domain: domain.to_string(),
                entities: vec![EntityDesc {
                    name: "Entity".into(),
                    attributes: vec!["name".into(), "type".into(), "properties".into()],
                    parent: None,
                }],
                operations: vec![OperationDesc {
                    name: "process".into(),
                    input_params: vec!["input".into()],
                    output_effect: "transformed output".into(),
                }],
                constraints: vec!["valid".into()],
            },
        }
    }

    pub fn explore(
        &self,
        _brain: &mut ReasoningBrain,
        profile: &EnvironmentProfile,
    ) -> ExplorationTrajectory {
        let mut states = Vec::new();
        let mut actions = Vec::new();
        let mut observations = Vec::new();

        let b_steps = self.config.breadth_steps.min(10);
        let d_steps = self.config.depth_steps.min(5);

        for i in 0..b_steps {
            let entity_idx = i % profile.entities.len().max(1);
            let entity = &profile.entities[entity_idx];
            let attr_idx = i % entity.attributes.len().max(1);
            let attr = &entity.attributes[attr_idx];

            states.push(format!(
                "state_b_{}_entity_{}_attr_{}",
                i, entity.name, attr
            ));
            actions.push(format!("examine({}, {})", entity.name, attr));
            observations.push(format!(
                "Found attribute {} of {} with value pattern",
                attr, entity.name
            ));
        }

        for j in 0..d_steps {
            let op_idx = j % profile.operations.len().max(1);
            let op = &profile.operations[op_idx];
            let param = &op.input_params[0];

            states.push(format!("state_d_{}_op_{}", j, op.name));
            actions.push(format!("execute({}, {})", op.name, param));
            observations.push(format!(
                "Operation {} completed: {}",
                op.name, op.output_effect
            ));
        }

        ExplorationTrajectory {
            env_profile: profile.clone(),
            states,
            actions,
            observations,
            breadth_phase_steps: b_steps,
            depth_phase_steps: d_steps,
        }
    }

    pub fn synthesize_tasks(&self, traj: &ExplorationTrajectory) -> Vec<GeneratedTask> {
        let profile = &traj.env_profile;
        let max_tasks = self.config.max_tasks_per_round.min(5);
        let mut tasks = Vec::new();

        for (i, entity) in profile.entities.iter().enumerate() {
            if tasks.len() >= max_tasks {
                break;
            }
            let attr_desc = entity.attributes.join(", ");
            let query = format!(
                "Analyze the {} entity with attributes: {}. Propose a modification that improves {}'s structure while maintaining backward compatibility.",
                entity.name, attr_desc, entity.name
            );
            let solution = format!(
                "1. Identify current {} structure\n2. Evaluate {} attributes: {}\n3. Apply transformation\n4. Verify correctness",
                entity.name, entity.name, attr_desc
            );
            tasks.push(GeneratedTask {
                id: format!("sq-task-{:04}", i + 1),
                query,
                reference_solution: vec![solution],
                difficulty: 0.5 + (i as f64 * 0.1),
                style_hints: vec!["deterministic".into(), "conservative".into()],
                source_trajectory_id: format!(
                    "traj-{}",
                    traj.breadth_phase_steps + traj.depth_phase_steps
                ),
            });
        }

        for (j, op) in profile.operations.iter().enumerate() {
            if tasks.len() >= max_tasks {
                break;
            }
            let query = format!(
                "Apply the {} operation on the current environment. Parameters: {}. Expected outcome: {}.",
                op.name, op.input_params.join(", "), op.output_effect
            );
            tasks.push(GeneratedTask {
                id: format!("sq-task-{:04}", tasks.len() + 1),
                query,
                reference_solution: vec![
                    format!("1. Validate input params: {}", op.input_params.join(", ")),
                    format!("2. Execute {} with validated params", op.name),
                    format!("3. Verify {}", op.output_effect),
                ],
                difficulty: 0.6 + (j as f64 * 0.1),
                style_hints: vec!["operational".into(), "step-by-step".into()],
                source_trajectory_id: format!(
                    "traj-{}",
                    traj.breadth_phase_steps + traj.depth_phase_steps
                ),
            });
        }

        tasks
    }

    pub fn curate(
        &self,
        _brain: &mut ReasoningBrain,
        tasks: Vec<GeneratedTask>,
    ) -> Vec<CuratedTask> {
        let mut seen_queries: HashSet<String> = HashSet::new();
        let mut curated = Vec::new();

        for task in tasks {
            let query_lower = task.query.to_lowercase();
            let passed_dedup = seen_queries.insert(query_lower);

            let passed_feasibility = self.check_feasibility(&task);

            let attempt = task.reference_solution.first().cloned().unwrap_or_default();
            let judge_score = self.judge_internal(&task, &attempt);

            let proxy_reward =
                if passed_dedup && passed_feasibility && judge_score >= self.config.min_judge_score
                {
                    judge_score * 0.9 + 0.1
                } else {
                    judge_score * 0.3
                };

            curated.push(CuratedTask {
                task,
                passed_dedup,
                passed_feasibility,
                llm_judge_score: judge_score,
                proxy_reward,
            });
        }

        curated
    }

    fn check_feasibility(&self, task: &GeneratedTask) -> bool {
        let has_query = !task.query.is_empty();
        let has_solution = !task.reference_solution.is_empty();
        let valid_difficulty = task.difficulty >= 0.0 && task.difficulty <= 1.0;
        has_query && has_solution && valid_difficulty
    }

    pub fn judge(&self, _brain: &mut ReasoningBrain, task: &GeneratedTask, attempt: &str) -> f64 {
        self.judge_internal(task, attempt)
    }

    fn judge_internal(&self, task: &GeneratedTask, attempt: &str) -> f64 {
        if attempt.is_empty() {
            return 0.0;
        }

        let solution_text = task.reference_solution.join(" ");
        let solution_words: HashSet<&str> = solution_text.split_whitespace().collect();
        let attempt_words: HashSet<&str> = attempt.split_whitespace().collect();

        if solution_words.is_empty() {
            return 0.5;
        }

        let intersection: HashSet<&&str> = solution_words.intersection(&attempt_words).collect();
        let jaccard = intersection.len() as f64 / solution_words.len() as f64;

        (jaccard * 0.7 + 0.3).min(1.0)
    }

    pub fn explore_with_guidance(
        &self,
        profile: &EnvironmentProfile,
        pool: &crate::neotrix::nt_mind::self_questioning::experience::ExperiencePool,
    ) -> ExplorationTrajectory {
        let mut states = Vec::new();
        let mut actions = Vec::new();
        let mut observations = Vec::new();

        let patterns = pool.extract_patterns(&profile.domain);
        let b_steps = self.config.breadth_steps.min(10);
        let d_steps = self.config.depth_steps.min(5);

        let mut biased_actions: Vec<String> = Vec::new();
        let bias_factor = 0.6;

        for i in 0..b_steps {
            let entity_idx = i % profile.entities.len().max(1);
            let entity = &profile.entities[entity_idx];
            let attr_idx = i % entity.attributes.len().max(1);
            let attr = &entity.attributes[attr_idx];

            let action;
            if !patterns.is_empty() && (i as f64 / b_steps as f64) < bias_factor {
                let pattern_idx = i % patterns.len();
                let pattern = &patterns[pattern_idx];
                action = format!(
                    "guided_{}({}, {})",
                    pattern.trim_start_matches("pattern:"),
                    entity.name,
                    attr
                );
            } else {
                action = format!("examine({}, {})", entity.name, attr);
            }

            biased_actions.push(action.clone());
            states.push(format!(
                "state_b_{}_entity_{}_attr_{}",
                i, entity.name, attr
            ));
            actions.push(action);
            observations.push(format!("Guided exploration: {} of {}", attr, entity.name));
        }

        for j in 0..d_steps {
            let op_idx = j % profile.operations.len().max(1);
            let op = &profile.operations[op_idx];
            let param = &op.input_params[0];

            let action;
            let pattern_hit = biased_actions.iter().find(|a| a.contains(&op.name));
            if let Some(hit) = pattern_hit {
                action = format!(
                    "guided_execute({}, {}) <- bias_from({})",
                    op.name, param, hit
                );
            } else {
                action = format!("execute({}, {})", op.name, param);
            }

            states.push(format!("state_d_{}_op_{}", j, op.name));
            actions.push(action);
            observations.push(format!(
                "Guided operation {} completed: {}",
                op.name, op.output_effect
            ));
        }

        ExplorationTrajectory {
            env_profile: profile.clone(),
            states,
            actions,
            observations,
            breadth_phase_steps: b_steps,
            depth_phase_steps: d_steps,
        }
    }

    pub fn apply_experience_bias(&self, actions: &mut Vec<String>, patterns: &[String]) {
        if patterns.is_empty() {
            return;
        }
        let bias_rate = 0.5;
        let max_biased = (actions.len() as f64 * bias_rate).ceil() as usize;
        let mut biased = 0;

        for action in actions.iter_mut() {
            if biased >= max_biased {
                break;
            }
            if let Some(pattern) = patterns.iter().find(|p| {
                let action_clean = action.trim_start_matches("guided_");
                p.contains(action_clean)
            }) {
                *action = format!("biased({})", pattern);
                biased += 1;
            }
        }
    }

    pub fn enqueue_goals(&self, tasks: &[CuratedTask]) -> usize {
        tasks
            .iter()
            .filter(|t| {
                t.passed_dedup
                    && t.passed_feasibility
                    && t.llm_judge_score >= self.config.min_judge_score
            })
            .count()
    }
}
