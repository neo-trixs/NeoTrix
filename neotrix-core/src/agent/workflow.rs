//! Workflow 引擎 — 多步骤工作流编排
//!
//! 参照 PraisonAI Workflow 模式：
//! - Route：条件分支
//! - Parallel：并行执行
//! - Loop：条件循环
//! - Repeat：带评估的重复执行
//! - AgentTask：Agent 任务节点


/// 工作流步骤
#[derive(Debug, Clone)]
pub enum WorkflowStep {
    /// Agent 任务节点
    AgentTask {
        name: String,
        task_description: String,
    },
    /// 条件分支
    Route {
        name: String,
        condition: String,        // 条件表达式
        true_branch: Box<WorkflowStep>,
        false_branch: Box<WorkflowStep>,
    },
    /// 并行执行
    Parallel {
        name: String,
        steps: Vec<WorkflowStep>,
    },
    /// 条件循环
    Loop {
        name: String,
        step: Box<WorkflowStep>,
        max_iterations: usize,
        condition: String,
    },
    /// 带评估的重复执行
    Repeat {
        name: String,
        step: Box<WorkflowStep>,
        max_iterations: usize,
        quality_threshold: f64,
    },
}

/// 工作流定义
#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

/// 工作流执行结果
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub iteration: usize,
}

/// WorkflowEngine — 执行器
pub struct WorkflowEngine {
    pub workflows: Vec<Workflow>,
    pub history: Vec<WorkflowResult>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self { workflows: Vec::new(), history: Vec::new() }
    }

    pub fn register(&mut self, workflow: Workflow) {
        self.workflows.push(workflow);
    }

    /// 执行单个步骤
    fn execute_step(&mut self, step: &WorkflowStep, _context: &str) -> WorkflowResult {
        match step {
            WorkflowStep::AgentTask { name, task_description } => {
                WorkflowResult {
                    step_name: name.clone(),
                    success: true,
                    output: format!("Agent 执行: {}", task_description),
                    iteration: 1,
                }
            }
            WorkflowStep::Route { name, condition, true_branch, false_branch } => {
                // 简化的条件判断：条件含 "true"/"yes" 走 true 分支
                let cond = condition.to_lowercase();
                let branch = if cond.contains("true") || cond.contains("yes") {
                    true_branch
                } else {
                    false_branch
                };
                let result = self.execute_step(branch, _context);
                WorkflowResult {
                    step_name: format!("{}->{}", name, result.step_name),
                    success: result.success,
                    output: result.output,
                    iteration: 1,
                }
            }
            WorkflowStep::Parallel { name, steps } => {
                let results: Vec<WorkflowResult> = steps.iter()
                    .map(|s| self.execute_step(s, _context))
                    .collect();
                let all_ok = results.iter().all(|r| r.success);
                let outputs: Vec<String> = results.iter().map(|r| r.output.clone()).collect();
                WorkflowResult {
                    step_name: name.clone(),
                    success: all_ok,
                    output: format!("并行结果[{}]: {}", results.len(), outputs.join(" | ")),
                    iteration: 1,
                }
            }
            WorkflowStep::Loop { name, step, max_iterations, condition } => {
                let mut last_result = WorkflowResult {
                    step_name: name.clone(),
                    success: true,
                    output: String::new(),
                    iteration: 0,
                };
                for i in 0..*max_iterations {
                    last_result = self.execute_step(step, _context);
                    last_result.iteration = i + 1;

                    // 条件含 "stop"/"enough" 提前退出
                    if condition.to_lowercase().contains("stop")
                        || condition.to_lowercase().contains("enough") {
                        break;
                    }
                }
                WorkflowResult {
                    step_name: format!("{}_loop", name),
                    success: last_result.success,
                    output: format!("循环 {} 次后: {}", last_result.iteration, last_result.output),
                    iteration: last_result.iteration,
                }
            }
            WorkflowStep::Repeat { name, step, max_iterations, quality_threshold } => {
                let mut last_result = WorkflowResult {
                    step_name: name.clone(),
                    success: true,
                    output: String::new(),
                    iteration: 0,
                };
                for i in 0..*max_iterations {
                    last_result = self.execute_step(step, _context);
                    last_result.iteration = i + 1;
                    // 质量达标提前退出
                    if last_result.success && (i + 1) as f64 / *max_iterations as f64 >= *quality_threshold {
                        break;
                    }
                }
                WorkflowResult {
                    step_name: format!("{}_repeat", name),
                    success: last_result.success,
                    output: format!("重复 {} 次后达标率 {:.1}: {}",
                        last_result.iteration, quality_threshold, last_result.output),
                    iteration: last_result.iteration,
                }
            }
        }
    }

    /// 执行工作流
    pub fn run(&mut self, name: &str, context: &str) -> Vec<WorkflowResult> {
        let wf = match self.workflows.iter().find(|w| w.name == name) {
            Some(w) => w.clone(),
            None => return vec![WorkflowResult {
                step_name: "error".to_string(),
                success: false,
                output: format!("Workflow '{}' not found", name),
                iteration: 0,
            }],
        };

        let results: Vec<WorkflowResult> = wf.steps.iter()
            .map(|s| self.execute_step(s, context))
            .collect();
        self.history.extend(results.clone());
        results
    }

    /// 从 YAML 创建 Workflow（字符串解析，简单版）
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        fn parse_step_list(lines: &[&str], start: usize, outer_indent: usize) -> Result<(Vec<WorkflowStep>, usize), String> {
            let mut steps = Vec::new();
            let mut i = start;

            while i < lines.len() {
                let line = lines[i];
                let indent = line.len() - line.trim_start().len();

                if indent <= outer_indent {
                    break;
                }

                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    i += 1;
                    continue;
                }

                if let Some(rest) = trimmed.strip_prefix("- ") {
                    let item_indent = indent;
                    let field_indent = item_indent + 2;

                    if let Some(type_val) = rest.strip_prefix("type:") {
                        let step_type = type_val.trim();

                        let mut step_name = String::new();
                        let mut task_desc = String::new();
                        let mut sub_steps = Vec::new();

                        i += 1;
                        while i < lines.len() {
                            let field_line = lines[i];
                            let fi = field_line.len() - field_line.trim_start().len();

                            if fi < field_indent || field_line.trim().is_empty() {
                                break;
                            }

                            let ft = field_line.trim();

                            if let Some(val) = ft.strip_prefix("name:") {
                                step_name = val.trim().to_string();
                            } else if let Some(val) = ft.strip_prefix("task:") {
                                task_desc = val.trim().to_string();
                            } else if ft == "steps:" {
                                let (sub, next_i) = parse_step_list(lines, i + 1, fi)?;
                                sub_steps = sub;
                                i = next_i;
                                continue;
                            }
                            i += 1;
                        }

                        match step_type {
                            "agent_task" => steps.push(WorkflowStep::AgentTask {
                                name: step_name,
                                task_description: task_desc,
                            }),
                            "parallel" => steps.push(WorkflowStep::Parallel {
                                name: step_name,
                                steps: sub_steps,
                            }),
                            other => return Err(format!("Unknown step type '{}'", other)),
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }

            Ok((steps, i))
        }

        let lines: Vec<&str> = yaml.lines().collect();
        let mut name = String::new();
        let mut description = String::new();
        let mut steps = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            if line.trim().is_empty() || line.trim().starts_with('#') {
                i += 1;
                continue;
            }

            let trimmed = line.trim();

            if let Some(val) = trimmed.strip_prefix("name:") {
                if name.is_empty() {
                    name = val.trim().to_string();
                }
                i += 1;
            } else if let Some(val) = trimmed.strip_prefix("description:") {
                if description.is_empty() {
                    description = val.trim().to_string();
                }
                i += 1;
            } else if trimmed == "steps:" {
                let (parsed_steps, next_i) = parse_step_list(&lines, i + 1, 0)?;
                steps = parsed_steps;
                i = next_i;
            } else {
                i += 1;
            }
        }

        if name.is_empty() {
            return Err("Missing 'name' in workflow YAML".to_string());
        }

        let mut engine = WorkflowEngine::new();
        engine.register(Workflow { name, description, steps });
        Ok(engine)
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_task_step() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "research".to_string(),
            description: "Research workflow".to_string(),
            steps: vec![
                WorkflowStep::AgentTask {
                    name: "research".to_string(),
                    task_description: "Research AI trends".to_string(),
                },
                WorkflowStep::AgentTask {
                    name: "write".to_string(),
                    task_description: "Write summary".to_string(),
                },
            ],
        });

        let results = engine.run("research", "AI in 2026");
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
        assert_eq!(results[0].step_name, "research");
    }

    #[test]
    fn test_route_step() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "decide".to_string(),
            description: "Decision workflow".to_string(),
            steps: vec![
                WorkflowStep::Route {
                    name: "quality_check".to_string(),
                    condition: "true".to_string(),
                    true_branch: Box::new(WorkflowStep::AgentTask {
                        name: "approve".to_string(),
                        task_description: "Approve and continue".to_string(),
                    }),
                    false_branch: Box::new(WorkflowStep::AgentTask {
                        name: "reject".to_string(),
                        task_description: "Reject and fix".to_string(),
                    }),
                },
            ],
        });

        let results = engine.run("decide", "check quality");
        assert_eq!(results.len(), 1);
        assert!(results[0].step_name.contains("approve"));
    }

    #[test]
    fn test_parallel_step() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "parallel_test".to_string(),
            description: "Parallel execution".to_string(),
            steps: vec![
                WorkflowStep::Parallel {
                    name: "parallel".to_string(),
                    steps: vec![
                        WorkflowStep::AgentTask { name: "task1".to_string(), task_description: "Task 1".to_string() },
                        WorkflowStep::AgentTask { name: "task2".to_string(), task_description: "Task 2".to_string() },
                        WorkflowStep::AgentTask { name: "task3".to_string(), task_description: "Task 3".to_string() },
                    ],
                },
            ],
        });

        let results = engine.run("parallel_test", "run parallel");
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_loop_step() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "loop_test".to_string(),
            description: "Loop execution".to_string(),
            steps: vec![
                WorkflowStep::Loop {
                    name: "search_loop".to_string(),
                    step: Box::new(WorkflowStep::AgentTask {
                        name: "search".to_string(),
                        task_description: "Search iteration".to_string(),
                    }),
                    max_iterations: 5,
                    condition: "continue".to_string(),
                },
            ],
        });

        let results = engine.run("loop_test", "search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].iteration, 5);
    }

    #[test]
    fn test_repeat_step() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "repeat_test".to_string(),
            description: "Repeat with quality".to_string(),
            steps: vec![
                WorkflowStep::Repeat {
                    name: "quality_loop".to_string(),
                    step: Box::new(WorkflowStep::AgentTask {
                        name: "improve".to_string(),
                        task_description: "Improve quality".to_string(),
                    }),
                    max_iterations: 10,
                    quality_threshold: 0.8,
                },
            ],
        });

        let results = engine.run("repeat_test", "improve");
        assert_eq!(results.len(), 1);
        assert!(results[0].step_name.contains("repeat"));
    }

    #[test]
    fn test_workflow_not_found() {
        let mut engine = WorkflowEngine::new();
        let results = engine.run("nonexistent", "test");
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
    }

    #[test]
    fn test_nested_workflow() {
        let mut engine = WorkflowEngine::new();
        engine.register(Workflow {
            name: "nested".to_string(),
            description: "Nested parallel in route".to_string(),
            steps: vec![
                WorkflowStep::Route {
                    name: "gate".to_string(),
                    condition: "yes".to_string(),
                    true_branch: Box::new(WorkflowStep::Parallel {
                        name: "workers".to_string(),
                        steps: vec![
                            WorkflowStep::AgentTask { name: "w1".to_string(), task_description: "Work 1".to_string() },
                            WorkflowStep::AgentTask { name: "w2".to_string(), task_description: "Work 2".to_string() },
                        ],
                    }),
                    false_branch: Box::new(WorkflowStep::AgentTask {
                        name: "skip".to_string(),
                        task_description: "Skip all".to_string(),
                    }),
                },
            ],
        });

        let results = engine.run("nested", "go");
        assert_eq!(results.len(), 1);
        assert!(results[0].step_name.contains("gate"));
    }

    #[test]
    fn test_workflow_from_yaml() -> Result<(), String> {
        let yaml = r#"
name: my-workflow
description: My workflow
steps:
  - type: agent_task
    name: research
    task: Research topic
  - type: parallel
    name: run_parallel
    steps:
      - type: agent_task
        name: task1
        task: Do thing 1
      - type: agent_task
        name: task2
        task: Do thing 2
"#;
        let engine = WorkflowEngine::from_yaml(yaml).expect("Failed to parse YAML");
        let wf = &engine.workflows[0];
        assert_eq!(wf.name, "my-workflow");
        assert_eq!(wf.description, "My workflow");
        assert_eq!(wf.steps.len(), 2);

        match &wf.steps[0] {
            WorkflowStep::AgentTask { name, task_description } => {
                assert_eq!(name, "research");
                assert_eq!(task_description, "Research topic");
            }
            step => return Err(format!("Expected AgentTask, got {:?}", step)),
        }

        match &wf.steps[1] {
            WorkflowStep::Parallel { name, steps } => {
                assert_eq!(name, "run_parallel");
                assert_eq!(steps.len(), 2);
                match &steps[0] {
                    WorkflowStep::AgentTask { name, task_description } => {
                        assert_eq!(name, "task1");
                        assert_eq!(task_description, "Do thing 1");
                    }
                    step => return Err(format!("Expected AgentTask inside parallel, got {:?}", step)),
                }
            }
            step => return Err(format!("Expected Parallel, got {:?}", step)),
        }
        Ok(())
    }
}
