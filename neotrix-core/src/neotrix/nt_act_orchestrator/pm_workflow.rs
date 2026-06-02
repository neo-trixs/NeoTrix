use serde::{Deserialize, Serialize};
use crate::neotrix::nt_mind::goal_loop::priority::PriorityEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PMWorkflowType {
    PrdGeneration,
    CompetitiveAnalysis,
    ExperimentDesign,
    UxAudit,
    RoadmapPlanning,
    LaunchChecklist,
}

impl PMWorkflowType {
    pub fn label(&self) -> &'static str {
        match self {
            PMWorkflowType::PrdGeneration => "PRD Generation",
            PMWorkflowType::CompetitiveAnalysis => "Competitive Analysis",
            PMWorkflowType::ExperimentDesign => "Experiment Design",
            PMWorkflowType::UxAudit => "UX Audit",
            PMWorkflowType::RoadmapPlanning => "Roadmap Planning",
            PMWorkflowType::LaunchChecklist => "Launch Checklist",
        }
    }
}

pub struct QualityGate {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub check_fn: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl std::fmt::Debug for QualityGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QualityGate")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("required", &self.required)
            .finish()
    }
}

impl QualityGate {
    pub fn new(name: &str, description: &str, required: bool, check_fn: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required,
            check_fn,
        }
    }

    pub fn evaluate(&self, output: &str) -> bool {
        (self.check_fn)(output)
    }
}

pub struct PMNode {
    pub workflow: PMWorkflowType,
    pub quality_gates: Vec<QualityGate>,
    pub gate_results: Vec<(String, bool)>,
    pub priority_score: Option<f64>,
}

impl PMNode {
    pub fn new(workflow: PMWorkflowType) -> Self {
        let gates = Self::default_gates(workflow);
        Self {
            workflow,
            quality_gates: gates,
            gate_results: Vec::new(),
            priority_score: None,
        }
    }

    pub fn evaluate_gates(&mut self, output: &str) -> Vec<(String, bool)> {
        self.gate_results = self.quality_gates.iter()
            .map(|gate| (gate.name.clone(), gate.evaluate(output)))
            .collect();
        self.gate_results.clone()
    }

    pub fn all_required_pass(&self) -> bool {
        self.quality_gates.iter()
            .filter(|g| g.required)
            .all(|g| {
                self.gate_results.iter()
                    .find(|(name, _)| name == &g.name)
                    .map(|(_, passed)| *passed)
                    .unwrap_or(false)
            })
    }

    pub fn score(&self) -> f64 {
        if self.quality_gates.is_empty() {
            return 0.0;
        }
        let passed = self.gate_results.iter().filter(|(_, p)| *p).count() as f64;
        passed / self.quality_gates.len() as f64
    }

    pub fn weighted_score(&self) -> f64 {
        if self.quality_gates.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = self.quality_gates.iter().map(|g| if g.required { 3.0 } else { 1.0 }).sum();
        let earned: f64 = self.quality_gates.iter().zip(self.gate_results.iter())
            .filter(|(_, (_, passed))| *passed)
            .map(|(g, _)| if g.required { 3.0 } else { 1.0 })
            .sum();
        earned / total_weight
    }

    pub fn estimate_priority(&mut self, description: &str, complexity: f64) -> f64 {
        let engine = PriorityEngine::default();
        let score = engine.evaluate(description, complexity);
        self.priority_score = Some(score);
        score
    }

    fn default_gates(workflow: PMWorkflowType) -> Vec<QualityGate> {
        match workflow {
            PMWorkflowType::PrdGeneration => vec![
                QualityGate::new("has_overview", "PRD must have overview section", true,
                    Box::new(|o| o.contains("Overview") || o.contains("overview"))),
                QualityGate::new("has_problem", "PRD must state the problem", true,
                    Box::new(|o| o.contains("Problem"))),
                QualityGate::new("has_metrics", "PRD should define success metrics", false,
                    Box::new(|o| o.contains("Metric") || o.contains("metric") || o.contains("KPI"))),
                QualityGate::new("has_risks", "PRD should identify risks", false,
                    Box::new(|o| o.contains("Risk") || o.contains("risk"))),
            ],
            PMWorkflowType::UxAudit => vec![
                QualityGate::new("has_accessibility", "UX audit must check accessibility", true,
                    Box::new(|o| o.contains("accessibility") || o.contains("Accessibility") || o.contains("WCAG"))),
                QualityGate::new("has_heuristics", "UX audit should use heuristic evaluation", false,
                    Box::new(|o| o.contains("heuristic") || o.contains("Nielsen"))),
                QualityGate::new("has_recommendations", "UX audit should have recommendations", false,
                    Box::new(|o| o.contains("recommend") || o.contains("Recommend") || o.contains("suggest"))),
            ],
            PMWorkflowType::CompetitiveAnalysis => vec![
                QualityGate::new("has_comparison", "Must include comparison data", true,
                    Box::new(|o| o.contains("comparison") || o.contains("Comparison") || o.contains("vs") || o.contains("VS"))),
                QualityGate::new("has_gaps", "Should identify gaps", false,
                    Box::new(|o| o.contains("gap") || o.contains("Gap") || o.contains("missing"))),
            ],
            PMWorkflowType::ExperimentDesign => vec![
                QualityGate::new("has_hypothesis", "Must have a hypothesis", true,
                    Box::new(|o| o.contains("hypothesis") || o.contains("Hypothesis"))),
                QualityGate::new("has_metrics", "Must define success metrics", true,
                    Box::new(|o| o.contains("metric") || o.contains("Metric"))),
                QualityGate::new("has_sample_size", "Should estimate sample size", false,
                    Box::new(|o| o.contains("sample") || o.contains("Sample"))),
            ],
            _ => vec![
                QualityGate::new("completeness", "Basic completeness check", true,
                    Box::new(|o| o.len() > 50)),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_node_creation() {
        let node = PMNode::new(PMWorkflowType::PrdGeneration);
        assert_eq!(node.workflow, PMWorkflowType::PrdGeneration);
        assert!(!node.quality_gates.is_empty());
    }

    #[test]
    fn test_quality_gate_evaluate() {
        let gate = QualityGate::new("test", "test gate", true, Box::new(|o| o.contains("hello")));
        assert!(gate.evaluate("hello world"));
        assert!(!gate.evaluate("goodbye world"));
    }

    #[test]
    fn test_prd_gates_pass() {
        let mut node = PMNode::new(PMWorkflowType::PrdGeneration);
        let output = "# Overview\n## Problem Statement\n## Success Metrics\n## Risks";
        let _results = node.evaluate_gates(output);
        assert!(node.all_required_pass());
        assert!(node.score() > 0.5);
    }

    #[test]
    fn test_prd_gates_fail() {
        let mut node = PMNode::new(PMWorkflowType::PrdGeneration);
        let _results = node.evaluate_gates("hello world");
        assert!(!node.all_required_pass());
    }

    #[test]
    fn test_weighted_score() {
        let mut node = PMNode::new(PMWorkflowType::ExperimentDesign);
        let output = "Hypothesis: X improves Y\nMetrics: conversion rate\n";
        node.evaluate_gates(output);
        let ws = node.weighted_score();
        assert!(ws > 0.0);
    }

    #[test]
    fn test_priority_estimation() {
        let mut node = PMNode::new(PMWorkflowType::PrdGeneration);
        let score = node.estimate_priority("urgent critical nt_shield feature for platform", 5.0);
        assert!(score > 10.0);
        assert!(node.priority_score.is_some());
    }

    #[test]
    fn test_priority_estimation_no_critical() {
        let mut node = PMNode::new(PMWorkflowType::UxAudit);
        let score = node.estimate_priority("polish button colors in settings", 1.0);
        assert!(score < 50.0);
        assert!(node.priority_score.is_some());
    }

    #[test]
    fn test_workflow_label() {
        assert_eq!(PMWorkflowType::PrdGeneration.label(), "PRD Generation");
        assert_eq!(PMWorkflowType::UxAudit.label(), "UX Audit");
        assert_eq!(PMWorkflowType::LaunchChecklist.label(), "Launch Checklist");
    }

    #[test]
    fn test_quality_gate_debug() {
        let gate = QualityGate::new("name", "desc", true, Box::new(|_| true));
        let debug = format!("{:?}", gate);
        assert!(debug.contains("name"));
        assert!(debug.contains("desc"));
    }

    #[test]
    fn test_competitive_gates() {
        let mut node = PMNode::new(PMWorkflowType::CompetitiveAnalysis);
        let output = "Comparison of features vs CompetitorX\nGap analysis shows missing capability";
        let _results = node.evaluate_gates(output);
        assert!(node.all_required_pass());
    }

    #[test]
    fn test_roadmap_fallback_gates() {
        let mut node = PMNode::new(PMWorkflowType::RoadmapPlanning);
        let output = "A longer output that should pass the basic completeness check since it is more than fifty characters total in length";
        let _results = node.evaluate_gates(output);
        assert!(node.all_required_pass());
    }

    #[test]
    fn test_launch_checklist_fallback() {
        let mut node = PMNode::new(PMWorkflowType::LaunchChecklist);
        let output = "Short";
        let _results = node.evaluate_gates(output);
        assert!(!node.all_required_pass());
    }

    #[test]
    fn test_zero_gates_score() {
        let mut node = PMNode::new(PMWorkflowType::PrdGeneration);
        node.quality_gates.clear();
        assert_eq!(node.score(), 0.0);
        assert_eq!(node.weighted_score(), 0.0);
    }

}
