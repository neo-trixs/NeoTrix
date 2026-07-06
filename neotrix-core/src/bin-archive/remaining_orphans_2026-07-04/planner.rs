use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningStrategy {
    TopDown,
    BottomUp,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: usize,
    pub description: String,
    pub required_tool: Option<String>,
    pub depends_on: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub task: String,
    pub steps: Vec<Step>,
    pub strategy: PlanningStrategy,
}

#[derive(Debug, Clone)]
pub struct Planner {
    max_depth: usize,
    strategy: PlanningStrategy,
    tool_map: HashMap<String, String>,
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    pub fn new() -> Self {
        Self {
            max_depth: 5,
            strategy: PlanningStrategy::Adaptive,
            tool_map: HashMap::new(),
        }
    }

    pub fn with_strategy(strategy: PlanningStrategy) -> Self {
        Self {
            max_depth: 5,
            strategy,
            tool_map: HashMap::new(),
        }
    }

    pub fn plan(&self, task: &str, available_tools: &[String]) -> Plan {
        let steps = self.decompose(task, available_tools);
        Plan {
            task: task.to_string(),
            steps,
            strategy: self.strategy,
        }
    }

    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    pub fn register_tool(&mut self, name: &str, description: &str) {
        self.tool_map.insert(name.to_string(), description.to_string());
    }

    fn decompose(&self, task: &str, tools: &[String]) -> Vec<Step> {
        let lower = task.to_lowercase();
        let separators = [",", ";", "\n", " and ", " then ", "之后", "然后"];

        let mut raw_parts: Vec<String> = Vec::new();
        let mut buffer = String::new();

        for ch in task.chars() {
            let sep = separators.iter().find(|s| buffer.ends_with(*s));
            if let Some(s) = sep {
                let trimmed = buffer.trim_end_matches(s).trim().to_string();
                if !trimmed.is_empty() {
                    raw_parts.push(trimmed);
                }
                buffer.clear();
            } else {
                buffer.push(ch);
            }
        }
        let trimmed = buffer.trim().to_string();
        if !trimmed.is_empty() {
            raw_parts.push(trimmed);
        }

        if raw_parts.is_empty() {
            raw_parts.push(task.to_string());
        }

        let effective_strategy = match self.strategy {
            PlanningStrategy::Adaptive => {
                if raw_parts.len() > 3 || lower.contains("complex") || lower.contains("multi-step") {
                    PlanningStrategy::TopDown
                } else {
                    PlanningStrategy::BottomUp
                }
            }
            other => other,
        };

        match effective_strategy {
            PlanningStrategy::TopDown => self.decompose_top_down(&raw_parts, tools),
            PlanningStrategy::BottomUp => self.decompose_bottom_up(&raw_parts, tools),
            _ => self.decompose_top_down(&raw_parts, tools),
        }
    }

    fn decompose_top_down(&self, parts: &[String], tools: &[String]) -> Vec<Step> {
        let depth = parts.len().min(self.max_depth);
        let mut steps = Vec::with_capacity(depth);
        let mut step_id = 0;

        for (i, part) in parts.iter().enumerate().take(depth) {
            let matched_tool = self.match_tool(part, tools);
            let deps = if i > 0 { vec![i - 1] } else { vec![] };
            steps.push(Step {
                id: step_id,
                description: part.clone(),
                required_tool: matched_tool,
                depends_on: deps,
            });
            step_id += 1;
        }

        steps
    }

    fn decompose_bottom_up(&self, parts: &[String], tools: &[String]) -> Vec<Step> {
        let mut steps: Vec<Step> = parts
            .iter()
            .enumerate()
            .map(|(i, part)| {
                let matched_tool = self.match_tool(part, tools);
                let mut deps = Vec::new();
                if i > 0 {
                    deps.push(i - 1);
                }
                Step {
                    id: i,
                    description: part.clone(),
                    required_tool: matched_tool,
                    depends_on: deps,
                }
            })
            .collect();

        if steps.len() > 2 {
            let last = steps.len() - 1;
            let second_last = steps.len() - 2;
            if last > 0 && second_last > 0 {
                steps[last].depends_on.push(0);
            }
            steps[last].description = format!("Verify and finalize: {}", steps[last].description);
        }

        steps
    }

    fn match_tool(&self, _part: &str, tools: &[String]) -> Option<String> {
        if tools.is_empty() {
            return None;
        }
        let lower = _part.to_lowercase();
        for tool in tools {
            let tool_lower = tool.to_lowercase();
            if lower.contains(&tool_lower) || tool_lower.contains(&lower) {
                return Some(tool.clone());
            }
        }
        if let Some(first) = tools.first() {
            return Some(first.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_creation() {
        let planner = Planner::new();
        assert_eq!(planner.max_depth, 5);
    }

    #[test]
    fn test_plan_top_down() {
        let planner = Planner::with_strategy(PlanningStrategy::TopDown);
        let plan = planner.plan("Research, Implement, Test", &[]);
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.steps[0].description, "Research, Implement, Test");
    }

    #[test]
    fn test_plan_with_tools() {
        let planner = Planner::new();
        let tools = vec!["search".to_string(), "code".to_string()];
        let plan = planner.plan("search for docs then code the feature", &tools);
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_adaptive_strategy_complex_task() {
        let planner = Planner::with_strategy(PlanningStrategy::Adaptive);
        let plan = planner.plan("complex multi-step analysis", &[]);
        assert_eq!(plan.strategy, PlanningStrategy::Adaptive);
    }

    #[test]
    fn test_register_tool() {
        let mut planner = Planner::new();
        planner.register_tool("code_generator", "Generates Rust code");
        let plan = planner.plan("generate code", &["code_generator".to_string()]);
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_dependency_chain() {
        let planner = Planner::new();
        let plan = planner.plan("step1, step2, step3", &[]);
        if plan.steps.len() > 1 {
            assert_eq!(plan.steps[1].depends_on, vec![0]);
        }
    }
}
