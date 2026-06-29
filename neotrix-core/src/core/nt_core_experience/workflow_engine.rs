use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// A single step in a workflow
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    /// Capability OID or handler name to dispatch
    pub target: String,
    /// How to pass previous step results
    pub input_mapping: InputMapping,
    /// What to do with output
    pub output_mapping: OutputMapping,
    /// Next step ID(s) — None means end of chain
    pub next: Vec<String>,
    /// Condition for branching (optional)
    pub condition: Option<String>,
    /// Max retries on failure
    pub max_retries: u32,
}

/// How to map input for a step
#[derive(Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InputMapping {
    /// Pass raw input from workflow start
    Passthrough,
    /// Use output of previous step
    FromPrevious,
    /// Use specific field from previous step output
    FromField(String),
    /// Static string
    Literal(String),
    /// Merge outputs of multiple previous steps
    Merge(Vec<String>),
}

/// How to map output of a step
#[derive(Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OutputMapping {
    /// Store as default result
    Store,
    /// Store with named key
    StoreAs(String),
    /// Discard,
    Discard,
}

/// A complete workflow definition
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub start_step: String,
    pub timeout_secs: u64,
}

/// Result of executing a single step
#[derive(Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub output: String,
    pub duration_ms: u64,
    pub success: bool,
    pub retries: u32,
    pub error: Option<String>,
}

/// Result of executing the full workflow
#[derive(Clone, Serialize, Deserialize)]
pub struct ExperienceWorkflowResult {
    pub definition_name: String,
    pub step_results: Vec<StepResult>,
    pub total_duration_ms: u64,
    pub all_success: bool,
    pub final_output: String,
    pub error: Option<String>,
}

/// Checkpoint state for workflow engine durability.
#[derive(Clone, Serialize, Deserialize)]
struct WorkflowCheckpoint {
    definitions: HashMap<String, WorkflowDefinition>,
    recent_results: Vec<ExperienceWorkflowResult>,
    max_concurrent: usize,
    max_workflow_history: usize,
}

/// The main workflow engine
pub struct WorkflowEngine {
    /// Registered workflow definitions
    definitions: HashMap<String, WorkflowDefinition>,
    /// Max concurrent workflows
    max_concurrent: usize,
    pub max_workflow_history: usize,
    pub recent_results: Vec<ExperienceWorkflowResult>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            definitions: HashMap::new(),
            max_concurrent: 10,
            max_workflow_history: 50,
            recent_results: Vec::new(),
        };
        for def in default_workflows() {
            engine.definitions.insert(def.name.clone(), def);
        }
        engine
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Register a workflow definition
    pub fn register(&mut self, def: WorkflowDefinition) -> bool {
        let name = def.name.clone();
        self.definitions.insert(name, def);
        true
    }

    /// Get a workflow definition by name
    pub fn get(&self, name: &str) -> Option<&WorkflowDefinition> {
        self.definitions.get(name)
    }

    /// List registered workflow names
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.definitions.keys().map(|k| k.as_str()).collect();
        names.sort();
        names
    }

    /// Persist workflow engine state to a JSON file.
    /// Survives restarts: definitions + recent_results are preserved.
    pub fn save_state(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let checkpoint = WorkflowCheckpoint {
            definitions: self.definitions.clone(),
            recent_results: self.recent_results.clone(),
            max_concurrent: self.max_concurrent,
            max_workflow_history: self.max_workflow_history,
        };
        let json = serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| format!("serialize_failed:{}", e))?;
        std::fs::write(path.as_ref(), &json).map_err(|e| format!("write_failed:{}", e))?;
        Ok(())
    }

    /// Restore workflow engine state from a JSON file.
    /// Returns a new WorkflowEngine with preserved definitions + recent results.
    pub fn load_state(path: impl AsRef<Path>) -> Result<Self, String> {
        let json =
            std::fs::read_to_string(path.as_ref()).map_err(|e| format!("read_failed:{}", e))?;
        let checkpoint: WorkflowCheckpoint =
            serde_json::from_str(&json).map_err(|e| format!("deserialize_failed:{}", e))?;
        let mut engine = Self {
            definitions: checkpoint.definitions,
            max_concurrent: checkpoint.max_concurrent,
            max_workflow_history: checkpoint.max_workflow_history,
            recent_results: checkpoint.recent_results,
        };
        // Re-register default workflows for any that are missing
        for def in default_workflows() {
            engine.definitions.entry(def.name.clone()).or_insert(def);
        }
        Ok(engine)
    }

    /// Remove a workflow definition
    pub fn remove(&mut self, name: &str) -> bool {
        self.definitions.remove(name).is_some()
    }

    /// Start executing a workflow (synchronous for simplicity)
    pub fn execute<F>(
        &mut self,
        name: &str,
        input: &str,
        mut dispatch_fn: F,
    ) -> ExperienceWorkflowResult
    where
        F: FnMut(&str, &str) -> String,
    {
        let start_time = std::time::Instant::now();
        let def = match self.definitions.get(name) {
            Some(d) => d,
            None => {
                return ExperienceWorkflowResult {
                    definition_name: name.to_string(),
                    step_results: Vec::new(),
                    total_duration_ms: 0,
                    all_success: false,
                    final_output: String::new(),
                    error: Some(format!("workflow_not_found:{}", name)),
                };
            }
        };

        let mut step_outputs: HashMap<String, String> = HashMap::new();
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut all_success = true;
        let mut error: Option<String> = None;
        let mut final_output = String::new();

        // Build step lookup
        let step_map: HashMap<&str, &WorkflowStep> =
            def.steps.iter().map(|s| (s.id.as_str(), s)).collect();
        // Walk through steps
        let mut current_name: Option<String> = Some(def.start_step.clone());
        let mut previous_output = input.to_string();

        while let Some(ref sid) = current_name.clone() {
            let sid: &str = sid;
            let step = match step_map.get(sid) {
                Some(s) => s,
                None => {
                    let err = format!("step_not_found:{}", sid);
                    step_results.push(StepResult {
                        step_id: sid.to_string(),
                        output: String::new(),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        success: false,
                        retries: 0,
                        error: Some(err.clone()),
                    });
                    all_success = false;
                    error = Some(err);
                    break;
                }
            };

            // Evaluate condition
            if let Some(ref cond) = step.condition {
                if !Self::evaluate_condition(cond, &previous_output) {
                    // Skip this step — go to next
                    current_name = step.next.first().cloned();
                    continue;
                }
            }

            // Resolve input
            let step_input = Self::resolve_input(&step.input_mapping, &step_outputs, input);

            // Execute with retries
            let step_start = std::time::Instant::now();
            let mut step_success = false;
            let mut step_output = String::new();
            let mut step_error: Option<String> = None;
            let mut retries: u32 = 0;

            for attempt in 0..=step.max_retries {
                if attempt > 0 {
                    retries += 1;
                }
                step_output = dispatch_fn(&step.target, &step_input);
                // Check for failure pattern
                if step_output.starts_with("unknown_handler:")
                    || step_output.starts_with("unknown_capability:")
                    || step_output.starts_with("error:")
                {
                    step_error = Some(step_output.clone());
                    if attempt < step.max_retries {
                        continue;
                    }
                    break;
                }
                step_success = true;
                break;
            }

            let duration_ms = step_start.elapsed().as_millis() as u64;

            // Process output mapping
            match &step.output_mapping {
                OutputMapping::Store => {
                    step_outputs.insert(step.id.clone(), step_output.clone());
                    final_output = step_output.clone();
                }
                OutputMapping::StoreAs(key) => {
                    step_outputs.insert(key.clone(), step_output.clone());
                    final_output = step_output.clone();
                }
                OutputMapping::Discard => {}
            }

            step_results.push(StepResult {
                step_id: step.id.clone(),
                output: step_output.clone(),
                duration_ms,
                success: step_success,
                retries,
                error: step_error.clone(),
            });

            if !step_success {
                all_success = false;
                error = step_error.or_else(|| Some("step_failed".to_string()));
                break;
            }

            previous_output = step_output;

            // Determine next step
            current_name = step.next.first().cloned();
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        let result = ExperienceWorkflowResult {
            definition_name: name.to_string(),
            step_results,
            total_duration_ms,
            all_success,
            final_output,
            error,
        };

        // Store in history
        self.recent_results.push(result.clone());
        if self.recent_results.len() > self.max_workflow_history {
            self.recent_results.remove(0);
        }

        result
    }

    /// Evaluate a step condition
    pub fn evaluate_condition(condition: &str, previous_output: &str) -> bool {
        if condition == "success" {
            return true;
        }
        if condition == "failure" {
            return false;
        }
        if condition == "always" {
            return true;
        }
        if let Some(suffix) = condition.strip_prefix("contains:") {
            return previous_output.contains(suffix);
        }
        if let Some(suffix) = condition.strip_prefix("starts:") {
            return previous_output.starts_with(suffix);
        }
        if let Some(suffix) = condition.strip_prefix("matches:") {
            return previous_output == suffix;
        }
        // Default: always true
        true
    }

    /// Process input mapping
    pub fn resolve_input(
        mapping: &InputMapping,
        step_outputs: &HashMap<String, String>,
        initial_input: &str,
    ) -> String {
        match mapping {
            InputMapping::Passthrough => initial_input.to_string(),
            InputMapping::FromPrevious => {
                // Return the last stored output (or initial input)
                step_outputs
                    .values()
                    .last()
                    .cloned()
                    .unwrap_or_else(|| initial_input.to_string())
            }
            InputMapping::FromField(field) => step_outputs
                .get(field)
                .cloned()
                .unwrap_or_else(|| initial_input.to_string()),
            InputMapping::Literal(s) => s.clone(),
            InputMapping::Merge(keys) => {
                let mut merged = String::new();
                for key in keys {
                    if let Some(val) = step_outputs.get(key.as_str()) {
                        if !merged.is_empty() {
                            merged.push('\n');
                        }
                        merged.push_str(val);
                    }
                }
                if merged.is_empty() {
                    initial_input.to_string()
                } else {
                    merged
                }
            }
        }
    }

    /// Get recent results
    pub fn recent_results(&self) -> &[ExperienceWorkflowResult] {
        &self.recent_results
    }

    /// Summary for consciousness reporting
    pub fn summary(&self) -> String {
        let total_defs = self.definitions.len();
        let recent = self.recent_results.len();
        let last_status = self
            .recent_results
            .last()
            .map(|r| if r.all_success { "ok" } else { "fail" })
            .unwrap_or("none");
        format!(
            "workflow:{}_defs|{}_exec|last:{}",
            total_defs, recent, last_status
        )
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Build-in workflow definitions
pub fn default_workflows() -> Vec<WorkflowDefinition> {
    vec![
        WorkflowDefinition {
            name: "research.synthesize".to_string(),
            description: "Gather context, check evidence, generate response, verify faithfulness"
                .to_string(),
            start_step: "context_gather".to_string(),
            timeout_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "context_gather".to_string(),
                    target: "pipeline.context_gather".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("context".to_string()),
                    next: vec!["evidence_check".to_string()],
                    condition: None,
                    max_retries: 1,
                },
                WorkflowStep {
                    id: "evidence_check".to_string(),
                    target: "experience.faithfulness".to_string(),
                    input_mapping: InputMapping::FromPrevious,
                    output_mapping: OutputMapping::StoreAs("evidence".to_string()),
                    next: vec!["response_generate".to_string()],
                    condition: None,
                    max_retries: 1,
                },
                WorkflowStep {
                    id: "response_generate".to_string(),
                    target: "bridge".to_string(),
                    input_mapping: InputMapping::Merge(vec![
                        "context".to_string(),
                        "evidence".to_string(),
                    ]),
                    output_mapping: OutputMapping::StoreAs("response".to_string()),
                    next: vec!["faithfulness_check".to_string()],
                    condition: None,
                    max_retries: 2,
                },
                WorkflowStep {
                    id: "faithfulness_check".to_string(),
                    target: "faithfulness".to_string(),
                    input_mapping: InputMapping::FromField("response".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: Some("always".to_string()),
                    max_retries: 0,
                },
            ],
        },
        WorkflowDefinition {
            name: "consciousness.audit".to_string(),
            description: "Profile handlers, run introspection, collect metrics, generate report"
                .to_string(),
            start_step: "profiler_tick".to_string(),
            timeout_secs: 30,
            steps: vec![
                WorkflowStep {
                    id: "profiler_tick".to_string(),
                    target: "metrics".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("profile_data".to_string()),
                    next: vec!["introspection".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "introspection".to_string(),
                    target: "introspection".to_string(),
                    input_mapping: InputMapping::FromPrevious,
                    output_mapping: OutputMapping::StoreAs("introspect_data".to_string()),
                    next: vec!["metrics_collect".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "metrics_collect".to_string(),
                    target: "metrics".to_string(),
                    input_mapping: InputMapping::Merge(vec![
                        "profile_data".to_string(),
                        "introspect_data".to_string(),
                    ]),
                    output_mapping: OutputMapping::StoreAs("metrics_report".to_string()),
                    next: vec!["report".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "report".to_string(),
                    target: "bridge".to_string(),
                    input_mapping: InputMapping::FromField("metrics_report".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 1,
                },
            ],
        },
        WorkflowDefinition {
            name: "memory.consolidate".to_string(),
            description: "Reflect on experience, accumulate skills, calibrate, consolidate dreams"
                .to_string(),
            start_step: "experience_reflect".to_string(),
            timeout_secs: 45,
            steps: vec![
                WorkflowStep {
                    id: "experience_reflect".to_string(),
                    target: "pipeline.experience_reflect".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("reflection".to_string()),
                    next: vec!["skill_accumulate".to_string()],
                    condition: None,
                    max_retries: 1,
                },
                WorkflowStep {
                    id: "skill_accumulate".to_string(),
                    target: "pipeline.skill_accumulate".to_string(),
                    input_mapping: InputMapping::FromField("reflection".to_string()),
                    output_mapping: OutputMapping::StoreAs("skills".to_string()),
                    next: vec!["calibration".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "calibration".to_string(),
                    target: "calibration".to_string(),
                    input_mapping: InputMapping::Merge(vec![
                        "reflection".to_string(),
                        "skills".to_string(),
                    ]),
                    output_mapping: OutputMapping::StoreAs("calibrated".to_string()),
                    next: vec!["dream_consolidate".to_string()],
                    condition: None,
                    max_retries: 1,
                },
                WorkflowStep {
                    id: "dream_consolidate".to_string(),
                    target: "dream_consolidator".to_string(),
                    input_mapping: InputMapping::FromField("calibrated".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 2,
                },
            ],
        },
        WorkflowDefinition {
            name: "safety.scan".to_string(),
            description: "Run safety gate, health patrol, edit safety check, generate report"
                .to_string(),
            start_step: "safety_gate".to_string(),
            timeout_secs: 30,
            steps: vec![
                WorkflowStep {
                    id: "safety_gate".to_string(),
                    target: "consciousness.safety.gate".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("safety_result".to_string()),
                    next: vec!["health_patrol".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "health_patrol".to_string(),
                    target: "health.health_patrol".to_string(),
                    input_mapping: InputMapping::FromField("safety_result".to_string()),
                    output_mapping: OutputMapping::StoreAs("health_report".to_string()),
                    next: vec!["edit_safety".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "edit_safety".to_string(),
                    target: "bridge".to_string(),
                    input_mapping: InputMapping::Merge(vec![
                        "safety_result".to_string(),
                        "health_report".to_string(),
                    ]),
                    output_mapping: OutputMapping::StoreAs("edit_check".to_string()),
                    next: vec!["report".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "report".to_string(),
                    target: "bridge".to_string(),
                    input_mapping: InputMapping::FromField("edit_check".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 0,
                },
            ],
        },
        WorkflowDefinition {
            name: "evolve.step".to_string(),
            description: "Self-evolve, evaluate changes, archive, collect metrics".to_string(),
            start_step: "self_evolution".to_string(),
            timeout_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "self_evolution".to_string(),
                    target: "experience.self_evolution".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("evolution_result".to_string()),
                    next: vec!["evaluate".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "evaluate".to_string(),
                    target: "experience.evolution_bridge".to_string(),
                    input_mapping: InputMapping::FromField("evolution_result".to_string()),
                    output_mapping: OutputMapping::StoreAs("eval_result".to_string()),
                    next: vec!["archive".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "archive".to_string(),
                    target: "bridge".to_string(),
                    input_mapping: InputMapping::Merge(vec![
                        "evolution_result".to_string(),
                        "eval_result".to_string(),
                    ]),
                    output_mapping: OutputMapping::StoreAs("archive_result".to_string()),
                    next: vec!["metrics".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "metrics".to_string(),
                    target: "metrics".to_string(),
                    input_mapping: InputMapping::FromField("eval_result".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 0,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "test.wf".to_string(),
            description: "test".to_string(),
            start_step: "step1".to_string(),
            timeout_secs: 10,
            steps: vec![],
        };
        assert!(engine.register(def));
        assert!(engine.get("test.wf").is_some());
        assert!(engine.get("nonexistent").is_none());
    }

    #[test]
    fn test_register_duplicate_overwrites() {
        let mut engine = WorkflowEngine::new();
        let def1 = WorkflowDefinition {
            name: "dup".to_string(),
            description: "first".to_string(),
            start_step: "s1".to_string(),
            timeout_secs: 10,
            steps: vec![],
        };
        let def2 = WorkflowDefinition {
            name: "dup".to_string(),
            description: "second".to_string(),
            start_step: "s2".to_string(),
            timeout_secs: 20,
            steps: vec![],
        };
        engine.register(def1);
        engine.register(def2);
        let got = engine.get("dup").unwrap();
        assert_eq!(got.description, "second");
        assert_eq!(got.timeout_secs, 20);
    }

    #[test]
    fn test_remove() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "remove.me".to_string(),
            description: "test".to_string(),
            start_step: "s1".to_string(),
            timeout_secs: 10,
            steps: vec![],
        };
        engine.register(def);
        assert!(engine.remove("remove.me"));
        assert!(!engine.remove("remove.me"));
        assert!(engine.get("remove.me").is_none());
    }

    #[test]
    fn test_list_workflows() {
        let engine = WorkflowEngine::new();
        let names = engine.list();
        // Should have at least the 5 default workflows
        assert!(
            names.len() >= 5,
            "expected >= 5 workflows, got {}",
            names.len()
        );
        assert!(names.contains(&"research.synthesize"));
        assert!(names.contains(&"consciousness.audit"));
        assert!(names.contains(&"memory.consolidate"));
        assert!(names.contains(&"safety.scan"));
        assert!(names.contains(&"evolve.step"));
    }

    #[test]
    fn test_execute_simple_linear() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "linear.test".to_string(),
            description: "simple linear".to_string(),
            start_step: "a".to_string(),
            timeout_secs: 10,
            steps: vec![
                WorkflowStep {
                    id: "a".to_string(),
                    target: "step_a".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("result_a".to_string()),
                    next: vec!["b".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "b".to_string(),
                    target: "step_b".to_string(),
                    input_mapping: InputMapping::FromField("result_a".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 0,
                },
            ],
        };
        engine.register(def);

        let result = engine.execute("linear.test", "hello", |target, input| match target {
            "step_a" => format!("a:{}", input),
            "step_b" => format!("b:{}", input),
            _ => format!("unknown:{}", target),
        });

        assert!(result.all_success);
        assert_eq!(result.step_results.len(), 2);
        assert_eq!(result.final_output, "b:a:hello");
        assert_eq!(result.step_results[0].step_id, "a");
        assert_eq!(result.step_results[0].output, "a:hello");
        assert_eq!(result.step_results[1].step_id, "b");
        assert_eq!(result.step_results[1].output, "b:a:hello");
    }

    #[test]
    fn test_execute_with_condition_true() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "cond.true".to_string(),
            description: "condition passes".to_string(),
            start_step: "check".to_string(),
            timeout_secs: 10,
            steps: vec![
                WorkflowStep {
                    id: "check".to_string(),
                    target: "checker".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("check_out".to_string()),
                    next: vec!["then_step".to_string(), "else_step".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "then_step".to_string(),
                    target: "then_h".to_string(),
                    input_mapping: InputMapping::FromField("check_out".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: Some("contains:ok".to_string()),
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "else_step".to_string(),
                    target: "else_h".to_string(),
                    input_mapping: InputMapping::FromField("check_out".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: Some("contains:fail".to_string()),
                    max_retries: 0,
                },
            ],
        };
        engine.register(def);

        let result = engine.execute("cond.true", "ok_signal", |target, input| match target {
            "checker" => format!("result_is_ok"),
            "then_h" => format!("then_executed:{}", input),
            "else_h" => format!("else_executed:{}", input),
            _ => format!("unknown:{}", target),
        });

        assert!(result.all_success);
        assert_eq!(result.step_results.len(), 2);
        assert!(result.final_output.contains("then_executed"));
    }

    #[test]
    fn test_execute_with_condition_false() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "cond.false".to_string(),
            description: "condition fails".to_string(),
            start_step: "check".to_string(),
            timeout_secs: 10,
            steps: vec![
                WorkflowStep {
                    id: "check".to_string(),
                    target: "checker".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("check_out".to_string()),
                    next: vec!["then_step".to_string(), "else_step".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "then_step".to_string(),
                    target: "then_h".to_string(),
                    input_mapping: InputMapping::FromField("check_out".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: Some("contains:ok".to_string()),
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "else_step".to_string(),
                    target: "else_h".to_string(),
                    input_mapping: InputMapping::FromField("check_out".to_string()),
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: Some("contains:fail".to_string()),
                    max_retries: 0,
                },
            ],
        };
        engine.register(def);

        let result = engine.execute("cond.false", "bad", |target, input| match target {
            "checker" => format!("result_is_fail"),
            "then_h" => format!("then_executed:{}", input),
            "else_h" => format!("else_executed:{}", input),
            _ => format!("unknown:{}", target),
        });

        assert!(result.all_success);
        assert_eq!(result.step_results.len(), 2);
        assert!(result.final_output.contains("else_executed"));
    }

    #[test]
    fn test_execute_workflow_not_found() {
        let mut engine = WorkflowEngine::new();
        let result = engine.execute("does_not_exist", "input", |_, _| String::new());
        assert!(!result.all_success);
        assert!(result.error.unwrap().contains("workflow_not_found"));
    }

    #[test]
    fn test_input_mapping_passthrough() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "passthrough.test".to_string(),
            description: "test passthrough".to_string(),
            start_step: "s1".to_string(),
            timeout_secs: 10,
            steps: vec![WorkflowStep {
                id: "s1".to_string(),
                target: "handler".to_string(),
                input_mapping: InputMapping::Passthrough,
                output_mapping: OutputMapping::Store,
                next: vec![],
                condition: None,
                max_retries: 0,
            }],
        };
        engine.register(def);

        let result = engine.execute("passthrough.test", "raw_input", |_, input| {
            format!("got:{}", input)
        });

        assert!(result.all_success);
        assert_eq!(result.final_output, "got:raw_input");
    }

    #[test]
    fn test_input_mapping_from_previous() {
        let mut engine = WorkflowEngine::new();
        let def = WorkflowDefinition {
            name: "prev.test".to_string(),
            description: "test from_previous".to_string(),
            start_step: "a".to_string(),
            timeout_secs: 10,
            steps: vec![
                WorkflowStep {
                    id: "a".to_string(),
                    target: "h1".to_string(),
                    input_mapping: InputMapping::Passthrough,
                    output_mapping: OutputMapping::StoreAs("out_a".to_string()),
                    next: vec!["b".to_string()],
                    condition: None,
                    max_retries: 0,
                },
                WorkflowStep {
                    id: "b".to_string(),
                    target: "h2".to_string(),
                    input_mapping: InputMapping::FromPrevious,
                    output_mapping: OutputMapping::Store,
                    next: vec![],
                    condition: None,
                    max_retries: 0,
                },
            ],
        };
        engine.register(def);

        let result = engine.execute("prev.test", "start", |target, input| match target {
            "h1" => format!("h1_out"),
            "h2" => format!("h2:{}", input),
            _ => format!("unknown"),
        });

        assert!(result.all_success);
        // h2 gets FromPrevious which uses last stored output = h1_out
        assert_eq!(result.final_output, "h2:h1_out");
    }

    #[test]
    fn test_default_workflows_count() {
        let wfs = default_workflows();
        assert_eq!(wfs.len(), 5);
        let names: Vec<&str> = wfs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"research.synthesize"));
        assert!(names.contains(&"consciousness.audit"));
        assert!(names.contains(&"memory.consolidate"));
        assert!(names.contains(&"safety.scan"));
        assert!(names.contains(&"evolve.step"));
    }

    #[test]
    fn test_recent_results() {
        let mut engine = WorkflowEngine::new();
        assert!(engine.recent_results().is_empty());

        let def = WorkflowDefinition {
            name: "recent.test".to_string(),
            description: "test".to_string(),
            start_step: "s1".to_string(),
            timeout_secs: 10,
            steps: vec![WorkflowStep {
                id: "s1".to_string(),
                target: "h".to_string(),
                input_mapping: InputMapping::Passthrough,
                output_mapping: OutputMapping::Store,
                next: vec![],
                condition: None,
                max_retries: 0,
            }],
        };
        engine.register(def);
        let r1 = engine.execute("recent.test", "in1", |_, i| format!("out:{}", i));
        assert!(r1.all_success);
        assert_eq!(engine.recent_results().len(), 1);

        let r2 = engine.execute("recent.test", "in2", |_, i| format!("out:{}", i));
        assert!(r2.all_success);
        assert_eq!(engine.recent_results().len(), 2);

        assert_eq!(engine.recent_results()[1].final_output, "out:in2");
    }

    #[test]
    fn test_summary_format() {
        let engine = WorkflowEngine::new();
        let s = engine.summary();
        assert!(s.contains("workflow:"));
        assert!(s.contains("_defs|"));
        assert!(s.contains("_exec|"));
    }

    #[test]
    fn test_evaluate_condition_variants() {
        assert!(WorkflowEngine::evaluate_condition("success", "anything"));
        assert!(!WorkflowEngine::evaluate_condition("failure", "anything"));
        assert!(WorkflowEngine::evaluate_condition("always", "anything"));
        assert!(WorkflowEngine::evaluate_condition(
            "contains:hello",
            "hello world"
        ));
        assert!(!WorkflowEngine::evaluate_condition(
            "contains:hello",
            "world"
        ));
        assert!(WorkflowEngine::evaluate_condition("starts:abc", "abcdef"));
        assert!(!WorkflowEngine::evaluate_condition("starts:abc", "defabc"));
        assert!(WorkflowEngine::evaluate_condition("matches:exact", "exact"));
        assert!(!WorkflowEngine::evaluate_condition(
            "matches:exact",
            "not exact"
        ));
        assert!(WorkflowEngine::evaluate_condition(
            "unknown:tag",
            "anything"
        ));
    }

    #[test]
    fn test_execute_retry_on_failure() {
        let mut engine = WorkflowEngine::new();
        let mut call_count = 0;

        let def = WorkflowDefinition {
            name: "retry.test".to_string(),
            description: "test retry".to_string(),
            start_step: "s1".to_string(),
            timeout_secs: 10,
            steps: vec![WorkflowStep {
                id: "s1".to_string(),
                target: "unstable_handler".to_string(),
                input_mapping: InputMapping::Passthrough,
                output_mapping: OutputMapping::Store,
                next: vec![],
                condition: None,
                max_retries: 3,
            }],
        };
        engine.register(def);

        let result = engine.execute("retry.test", "in", |_target, input| {
            call_count += 1;
            if call_count < 3 {
                format!("error:failed_attempt_{}", call_count)
            } else {
                format!("success:{}", input)
            }
        });

        assert!(result.all_success);
        assert_eq!(result.step_results[0].retries, 2);
        assert_eq!(result.final_output, "success:in");
    }
}
