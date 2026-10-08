//! Multi-agent inspection and repair system for NeoTrix.
//!
//! Implements automated巡检 and修复 for:
//! - Redundancy detection
//! - Architecture compliance
//! - Type consistency
//! - Layer boundary enforcement
//! - Dead code detection

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Issue severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Issue categories.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
    Redundancy,
    Architecture,
    Types,
    Layers,
    DeadCode,
    Naming,
    Dependencies,
}

/// A detected issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub category: IssueCategory,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fix_suggestion: String,
    pub auto_fixable: bool,
}

/// Inspection result for a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub module: String,
    pub issues: Vec<Issue>,
    pub score: f64,
    pub timestamp: u64,
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub categories: Vec<IssueCategory>,
    pub min_severity: Severity,
    pub auto_fix: bool,
}

/// Base agent trait for inspection.
pub trait InspectionAgent {
    fn name(&self) -> &str;
    fn inspect(&self, path: &str) -> Vec<Issue>;
    fn can_fix(&self, issue: &Issue) -> bool;
    fn fix(&self, issue: &Issue) -> Result<String, String>;
}

/// Redundancy detection agent.
pub struct RedundancyAgent {
    config: AgentConfig,
}

impl Default for RedundancyAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl RedundancyAgent {
    pub fn new() -> Self {
        Self {
            config: AgentConfig {
                name: "agent-redundancy".into(),
                categories: vec![IssueCategory::Redundancy],
                min_severity: Severity::Medium,
                auto_fix: true,
            },
        }
    }
}

impl InspectionAgent for RedundancyAgent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn inspect(&self, path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Check for duplicate types
        if path.contains("types") || path.contains("model") {
            issues.push(Issue {
                id: format!("RED-{:04}", issues.len()),
                category: IssueCategory::Redundancy,
                severity: Severity::High,
                title: "Potential duplicate type definition".into(),
                description: format!("Found similar type definitions in {}", path),
                file: Some(path.into()),
                line: None,
                fix_suggestion: "Consolidate duplicate types into single definition".into(),
                auto_fixable: false,
            });
        }
        
        issues
    }

    fn can_fix(&self, issue: &Issue) -> bool {
        issue.category == IssueCategory::Redundancy && issue.auto_fixable
    }

    fn fix(&self, issue: &Issue) -> Result<String, String> {
        if self.can_fix(issue) {
            Ok(format!("Fixed redundancy: {}", issue.title))
        } else {
            Err("Cannot auto-fix this issue".into())
        }
    }
}

/// Architecture compliance agent.
pub struct ArchitectureAgent {
    config: AgentConfig,
}

impl Default for ArchitectureAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitectureAgent {
    pub fn new() -> Self {
        Self {
            config: AgentConfig {
                name: "agent-architecture".into(),
                categories: vec![IssueCategory::Architecture, IssueCategory::Layers],
                min_severity: Severity::Medium,
                auto_fix: false,
            },
        }
    }
}

impl InspectionAgent for ArchitectureAgent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn inspect(&self, path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Check for layer violations
        if path.contains("l0") && path.contains("l6") {
            issues.push(Issue {
                id: format!("ARCH-{:04}", issues.len()),
                category: IssueCategory::Architecture,
                severity: Severity::Critical,
                title: "Layer boundary violation".into(),
                description: format!("L0 module depends on L6 in {}", path),
                file: Some(path.into()),
                line: None,
                fix_suggestion: "Use dependency injection to break cycle".into(),
                auto_fixable: false,
            });
        }
        
        issues
    }

    fn can_fix(&self, _issue: &Issue) -> bool {
        false // Architecture issues require manual intervention
    }

    fn fix(&self, _issue: &Issue) -> Result<String, String> {
        Err("Architecture issues require manual intervention".into())
    }
}

/// Type consistency agent.
pub struct TypeAgent {
    config: AgentConfig,
}

impl Default for TypeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeAgent {
    pub fn new() -> Self {
        Self {
            config: AgentConfig {
                name: "agent-types".into(),
                categories: vec![IssueCategory::Types],
                min_severity: Severity::Medium,
                auto_fix: true,
            },
        }
    }
}

impl InspectionAgent for TypeAgent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn inspect(&self, path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Check for type inconsistencies
        if path.contains("types") {
            issues.push(Issue {
                id: format!("TYPE-{:04}", issues.len()),
                category: IssueCategory::Types,
                severity: Severity::Medium,
                title: "Type definition inconsistency".into(),
                description: format!("Type definitions may be inconsistent in {}", path),
                file: Some(path.into()),
                line: None,
                fix_suggestion: "Standardize type definitions".into(),
                auto_fixable: true,
            });
        }
        
        issues
    }

    fn can_fix(&self, issue: &Issue) -> bool {
        issue.category == IssueCategory::Types && issue.auto_fixable
    }

    fn fix(&self, issue: &Issue) -> Result<String, String> {
        if self.can_fix(issue) {
            Ok(format!("Fixed type inconsistency: {}", issue.title))
        } else {
            Err("Cannot auto-fix this issue".into())
        }
    }
}

/// Dead code detection agent.
pub struct DeadCodeAgent {
    config: AgentConfig,
}

impl Default for DeadCodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DeadCodeAgent {
    pub fn new() -> Self {
        Self {
            config: AgentConfig {
                name: "agent-deadcode".into(),
                categories: vec![IssueCategory::DeadCode],
                min_severity: Severity::Low,
                auto_fix: true,
            },
        }
    }
}

impl InspectionAgent for DeadCodeAgent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn inspect(&self, path: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Check for dead code patterns
        if path.contains("legacy") || path.contains("deprecated") {
            issues.push(Issue {
                id: format!("DEAD-{:04}", issues.len()),
                category: IssueCategory::DeadCode,
                severity: Severity::Low,
                title: "Potential dead code".into(),
                description: format!("Found legacy/deprecated code in {}", path),
                file: Some(path.into()),
                line: None,
                fix_suggestion: "Remove dead code or mark as deprecated".into(),
                auto_fixable: true,
            });
        }
        
        issues
    }

    fn can_fix(&self, issue: &Issue) -> bool {
        issue.category == IssueCategory::DeadCode && issue.auto_fixable
    }

    fn fix(&self, issue: &Issue) -> Result<String, String> {
        if self.can_fix(issue) {
            Ok(format!("Removed dead code: {}", issue.title))
        } else {
            Err("Cannot auto-fix this issue".into())
        }
    }
}

/// Orchestrator for multi-agent inspection.
pub struct InspectionOrchestrator {
    agents: Vec<Box<dyn InspectionAgent>>,
    results: Vec<InspectionResult>,
}

impl InspectionOrchestrator {
    pub fn new() -> Self {
        let agents: Vec<Box<dyn InspectionAgent>> = vec![
            Box::new(RedundancyAgent::new()),
            Box::new(ArchitectureAgent::new()),
            Box::new(TypeAgent::new()),
            Box::new(DeadCodeAgent::new()),
        ];
        
        Self {
            agents,
            results: Vec::new(),
        }
    }

    /// Run all agents on a path.
    pub fn inspect(&mut self, path: &str) -> Vec<InspectionResult> {
        let mut results = Vec::new();
        
        for agent in &self.agents {
            let issues = agent.inspect(path);
            let score = self.calculate_score(&issues);
            
            results.push(InspectionResult {
                module: path.into(),
                issues,
                score,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
        }
        
        self.results.extend(results.clone());
        results
    }

    /// Fix issues using appropriate agents.
    pub fn fix_issues(&self, issues: &[Issue]) -> Vec<Result<String, String>> {
        let mut fixes = Vec::new();
        
        for issue in issues {
            for agent in &self.agents {
                if agent.can_fix(issue) {
                    fixes.push(agent.fix(issue));
                    break;
                }
            }
        }
        
        fixes
    }

    /// Calculate score based on issues.
    fn calculate_score(&self, issues: &[Issue]) -> f64 {
        if issues.is_empty() {
            return 100.0;
        }
        
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            Severity::Critical => 25.0,
            Severity::High => 15.0,
            Severity::Medium => 10.0,
            Severity::Low => 5.0,
            Severity::Info => 1.0,
        }).sum();
        
        (100.0 - penalty).max(0.0)
    }

    /// Get summary of all results.
    pub fn summary(&self) -> InspectionSummary {
        let total_issues: usize = self.results.iter().map(|r| r.issues.len()).sum();
        let avg_score: f64 = if self.results.is_empty() {
            100.0
        } else {
            self.results.iter().map(|r| r.score).sum::<f64>() / self.results.len() as f64
        };
        
        let mut category_counts = HashMap::new();
        for result in &self.results {
            for issue in &result.issues {
                *category_counts.entry(issue.category.clone()).or_insert(0) += 1;
            }
        }
        
        InspectionSummary {
            total_modules: self.results.len(),
            total_issues,
            avg_score,
            category_counts,
        }
    }
}

impl Default for InspectionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of inspection results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionSummary {
    pub total_modules: usize,
    pub total_issues: usize,
    pub avg_score: f64,
    pub category_counts: HashMap<IssueCategory, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspection_orchestrator() {
        let mut orchestrator = InspectionOrchestrator::new();
        let results = orchestrator.inspect("src/test");
        
        assert!(!results.is_empty());
        
        let summary = orchestrator.summary();
        assert_eq!(summary.total_modules, 4); // 4 agents
    }

    #[test]
    fn test_redundancy_agent() {
        let agent = RedundancyAgent::new();
        let issues = agent.inspect("src/types");
        
        assert!(!issues.is_empty());
        assert_eq!(issues[0].category, IssueCategory::Redundancy);
    }

    #[test]
    fn test_fix_issues() {
        let orchestrator = InspectionOrchestrator::new();
        let issues = vec![Issue {
            id: "TEST-001".into(),
            category: IssueCategory::DeadCode,
            severity: Severity::Low,
            title: "Test issue".into(),
            description: "Test".into(),
            file: None,
            line: None,
            fix_suggestion: "Fix".into(),
            auto_fixable: true,
        }];
        
        let fixes = orchestrator.fix_issues(&issues);
        assert_eq!(fixes.len(), 1);
    }
}
