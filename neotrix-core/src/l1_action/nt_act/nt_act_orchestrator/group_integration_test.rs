use crate::neotrix::nt_act_orchestrator::planner::PlannerNode;

#[test]
fn test_planner_no_group_manager() {
    let planner = PlannerNode::new();
    let tasks = planner.decompose("some task");
    assert!(!tasks.is_empty(), "Should decompose without group manager");
}

#[test]
fn test_planner_decompose_with_goal() {
    let planner = PlannerNode::new();
    let tasks = planner.decompose("some generic goal");
    assert!(!tasks.is_empty(), "Should decompose with generic goal");
}
