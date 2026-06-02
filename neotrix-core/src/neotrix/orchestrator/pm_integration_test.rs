use super::planner::PlannerNode;
use super::state_graph::StateGraph;
use super::state_graph::ArtifactState;
use crate::neotrix::nt_mind::goal_loop::priority::{PriorityEngine, MoscowClass};
use crate::neotrix::nt_mind::experiment::{Hypothesis, ExperimentDesigner};
use crate::neotrix::nt_mind::ux_review::UxReviewEngine;
use crate::neotrix::nt_mind::core::CapabilityVector;

#[test]
fn test_priority_engine_feeds_goal_loop() {
    let engine = PriorityEngine::default();
    let goals = vec![
        ("tweak button padding".to_string(), 1.0),
        ("fix critical auth bypass".to_string(), 5.0),
        ("add search feature".to_string(), 3.0),
    ];
    let ranked = engine.rank(&goals);
    assert_eq!(ranked[0], 1);
    assert_eq!(ranked[2], 0);
}

#[test]
fn test_planner_prd_decomposes_to_tasks() {
    let planner = PlannerNode::new();
    let tasks = planner.plan_prd("Create PRD for new onboarding flow");
    assert_eq!(tasks.len(), 4);
}

#[test]
fn test_planner_competitive_analysis_decomposes() {
    let planner = PlannerNode::new();
    let tasks = planner.plan_competitive_analysis("Compare with competitor X");
    assert_eq!(tasks.len(), 4);
}

#[test]
fn test_state_graph_pm_plan_topology() {
    let mut graph = StateGraph::new();
    graph.build_pm_plan("Create PRD", 4);
    assert!(graph.node("research").is_some());
    assert!(graph.node("draft").is_some());
    assert!(graph.node("review").is_some());
    assert!(graph.node("refine").is_some());

    let sorted = graph.topological_sort().expect("graph should be sortable");
    let pos_research = sorted.iter().position(|id| id == "research").expect("research node should exist in sorted order");
    let pos_draft = sorted.iter().position(|id| id == "draft").expect("draft node should exist in sorted order");
    let pos_review = sorted.iter().position(|id| id == "review").expect("review node should exist in sorted order");
    let pos_refine = sorted.iter().position(|id| id == "refine").expect("refine node should exist in sorted order");
    assert!(pos_research < pos_draft);
    assert!(pos_draft < pos_review);
    assert!(pos_review < pos_refine);
}

#[test]
fn test_priority_to_moscow_mapping() {
    let engine = PriorityEngine::default();
    let must_have = engine.evaluate("fix critical nt_shield vulnerability", 3.0);
    let could_have = engine.evaluate("polish button hover state", 1.0);
    assert!(must_have > could_have);
    assert_eq!(engine.to_moscow(must_have), MoscowClass::MustHave);
}

#[test]
fn test_experiment_to_priority_ranking() {
    let h = Hypothesis::new("H-001", "New feature improves retention",
        vec!["retention".to_string()], 0.5);
    let design = ExperimentDesigner::design_ab_test(&h);
    assert!(design.min_sample_size > 0);

    let control = vec![1.0, 1.2, 1.1, 0.9, 1.0, 1.1, 1.0, 1.2, 0.9, 1.1];
    let treatment = vec![2.0, 2.2, 2.1, 1.9, 2.0, 2.1, 2.0, 2.2, 1.9, 2.1];
    let result = ExperimentDesigner::analyze_results(&control, &treatment, "H-001");
    assert!(result.significant);

    let engine = PriorityEngine::default();
    let priority = engine.evaluate("Implement feature H-001: proven significant", 3.0);
    assert!(priority > 10.0);
}

#[test]
fn test_ux_review_feeds_back_to_capability() {
    let cap = CapabilityVector::default();
    let engine = UxReviewEngine::new(cap);
    let report = engine.review("login", "A login form with submit button");
    let edits = engine.issues_to_micro_edits(&report);
    assert!(!edits.is_empty());
    assert!(report.accessibility_score < 1.0);
}

#[test]
fn test_full_pm_chain_in_dag() {
    let mut graph = StateGraph::new();
    graph.build_pm_plan("Design experiment for onboarding", 4);
    assert_eq!(graph.nodes.len(), 4);
    graph.mark_done("research").expect("research node should exist");
    graph.mark_done("draft").expect("draft node should exist");
    graph.mark_done("review").expect("review node should exist");
    graph.mark_done("refine").expect("refine node should exist");

    for node in graph.nodes.values() {
        assert_eq!(node.state, ArtifactState::Done);
    }
}
