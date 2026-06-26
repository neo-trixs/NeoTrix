use crate::neotrix::nt_core_parallel::executor::OptimalTaskAllocator;
use crate::neotrix::nt_core_parallel::types::{AllocationStrategy, Task};
use crate::neotrix::nt_expert_routing::{Context, TaskType};
use crate::neotrix::nt_mind::group_contracts::GroupManager;

pub struct PlannerNode {
    _allocator: OptimalTaskAllocator,
    pub group_manager: Option<GroupManager>,
}

impl Default for PlannerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerNode {
    pub fn new() -> Self {
        Self {
            _allocator: OptimalTaskAllocator::new(AllocationStrategy::Hybrid),
            group_manager: None,
        }
    }

    pub fn with_group_manager(gm: GroupManager) -> Self {
        Self {
            _allocator: OptimalTaskAllocator::new(AllocationStrategy::Hybrid),
            group_manager: Some(gm),
        }
    }

    pub fn decompose(&self, goal: &str) -> Vec<Task> {
        let lower = goal.to_lowercase();

        // PM/Design workflow routing
        if lower.contains("prd")
            || lower.contains("product requirements")
            || lower.contains("prd generation")
        {
            return self.plan_prd(goal);
        }
        if lower.contains("competitive")
            || lower.contains("competitor")
            || lower.contains("comparison")
            || lower.contains("compare")
        {
            return self.plan_competitive_analysis(goal);
        }
        if lower.contains("ux audit")
            || lower.contains("ux review")
            || lower.contains("usability")
            || lower.contains("heuristic")
        {
            return self.plan_ux_audit(goal);
        }
        if lower.contains("experiment")
            || lower.contains("a/b test")
            || lower.contains("hypothesis")
            || lower.contains("ab test")
        {
            return self.plan_experiment(goal);
        }

        let ctx = Context::from_task_description(goal);
        let subtasks = match ctx.task_type {
            TaskType::Design | TaskType::UIDesign => self.plan_design(),
            TaskType::CodeAnalysis | TaskType::CodeGeneration | TaskType::CodeReview => {
                self.plan_code()
            }
            _ => self.plan_general(),
        };
        let enriched: Vec<String> = if let Some(ref gm) = self.group_manager {
            subtasks
                .into_iter()
                .map(|desc| {
                    let mut extra = Vec::new();
                    for word in desc.split_whitespace() {
                        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                        let matches = gm.match_cross_repo(clean);
                        if !matches.is_empty() {
                            let repos: Vec<&str> =
                                matches.iter().map(|m| m.to_repo.as_str()).collect();
                            extra.push(format!("[cross-repo: {} -> {}]", clean, repos.join(", ")));
                        }
                    }
                    if extra.is_empty() {
                        desc
                    } else {
                        format!("{} ({})", desc, extra.join("; "))
                    }
                })
                .collect()
        } else {
            subtasks
        };
        enriched
            .into_iter()
            .enumerate()
            .map(|(i, desc)| {
                let input: Vec<f64> = desc.bytes().map(|b| b as f64).collect();
                Task::new(format!("task_{}", i), input, i as i32)
            })
            .collect()
    }

    fn plan_design(&self) -> Vec<String> {
        vec![
            "分析设计需求".to_string(),
            "创建组件结构".to_string(),
            "实现样式系统".to_string(),
            "验证可访问性".to_string(),
        ]
    }

    fn plan_code(&self) -> Vec<String> {
        vec![
            "理解代码上下文".to_string(),
            "实现核心逻辑".to_string(),
            "添加错误处理".to_string(),
            "编写测试".to_string(),
        ]
    }

    fn plan_general(&self) -> Vec<String> {
        vec![
            "分析任务需求".to_string(),
            "执行任务".to_string(),
            "验证结果".to_string(),
        ]
    }

    pub fn plan_prd(&self, goal: &str) -> Vec<Task> {
        let subtasks = vec![
            format!("Research and gather context for PRD: {}", goal),
            format!(
                "Draft PRD with overview, problem statement, and target users: {}",
                goal
            ),
            format!("Review PRD for completeness and edge cases: {}", goal),
            format!("Refine PRD based on review feedback: {}", goal),
        ];
        self.build_tasks_from_subtasks(&subtasks)
    }

    pub fn plan_competitive_analysis(&self, goal: &str) -> Vec<Task> {
        let subtasks = vec![
            format!("Identify and fetch competitor information: {}", goal),
            format!("Compare features and build comparison matrix: {}", goal),
            format!("Analyze gaps and generate positioning report: {}", goal),
            format!("Recommend actions based on gap analysis: {}", goal),
        ];
        self.build_tasks_from_subtasks(&subtasks)
    }

    pub fn plan_ux_audit(&self, goal: &str) -> Vec<Task> {
        let subtasks = vec![
            format!("Run heuristic evaluation (Nielsen principles): {}", goal),
            format!("Check accessibility compliance (WCAG): {}", goal),
            format!("Analyze visual hierarchy and consistency: {}", goal),
            format!("Generate UX audit report with recommendations: {}", goal),
        ];
        self.build_tasks_from_subtasks(&subtasks)
    }

    pub fn plan_experiment(&self, goal: &str) -> Vec<Task> {
        let subtasks = vec![
            format!("Define hypothesis and success metrics: {}", goal),
            format!(
                "Design A/B test (sample size, duration, variants): {}",
                goal
            ),
            format!("Execute experiment and collect data: {}", goal),
            format!("Analyze results and generate report: {}", goal),
        ];
        self.build_tasks_from_subtasks(&subtasks)
    }

    fn build_tasks_from_subtasks(&self, subtasks: &[String]) -> Vec<Task> {
        subtasks
            .iter()
            .enumerate()
            .map(|(i, desc)| {
                let input: Vec<f64> = desc.bytes().map(|b| b as f64).collect();
                Task::new(format!("task_{}", i), input, i as i32)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_prd_returns_tasks() {
        let planner = PlannerNode::new();
        let tasks = planner.plan_prd("Create PRD for new feature");
        assert_eq!(tasks.len(), 4);
    }

    #[test]
    fn test_plan_competitive_analysis() {
        let planner = PlannerNode::new();
        let tasks = planner.plan_competitive_analysis("Compare with competitor X");
        assert_eq!(tasks.len(), 4);
    }

    #[test]
    fn test_plan_ux_audit() {
        let planner = PlannerNode::new();
        let tasks = planner.plan_ux_audit("Audit login page UX");
        assert_eq!(tasks.len(), 4);
    }

    #[test]
    fn test_plan_experiment() {
        let planner = PlannerNode::new();
        let tasks = planner.plan_experiment("Design A/B test for new onboarding");
        assert_eq!(tasks.len(), 4);
    }

    #[test]
    fn test_decompose_routes_to_pm() {
        let planner = PlannerNode::new();
        let tasks = planner.decompose("Create PRD for mobile app");
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_decompose_routes_to_competitive() {
        let planner = PlannerNode::new();
        let tasks = planner.decompose("competitive analysis of market leaders");
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_decompose_routes_to_ux() {
        let planner = PlannerNode::new();
        let tasks = planner.decompose("conduct ux audit for landing page");
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_decompose_routes_to_experiment() {
        let planner = PlannerNode::new();
        let tasks = planner.decompose("design experiment for new feature");
        assert!(!tasks.is_empty());
    }
}
