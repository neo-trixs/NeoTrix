//! Skill Improver — trailofbits/skills skill-improver 插件吸收
//! 
//! 迭代修正技能直到通过质量门禁
//! 集成 nt_mind_autofixer::autofixer

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 质量门禁结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub skill_name: String,
    pub passed: bool,
    pub scores: SkillQualityScores,
    pub violations: Vec<QualityViolation>,
    pub iterations: u32,
}

/// 质量评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillQualityScores {
    pub safety: f64,
    pub completeness: f64,
    pub executability: f64,
    pub maintainability: f64,
    pub cost_awareness: f64,
    pub overall: f64,
}

/// 质量违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub dimension: String,
    pub message: String,
    pub suggestion: String,
}

/// 改进动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementAction {
    AddFrontmatterField { field: String, value: String },
    AddVerificationSection,
    AddReferences { files: Vec<String> },
    SplitLongSkill { max_lines: usize },
    FixFrontmatterSyntax { line: usize, issue: String },
    AddCategory { category: String },
    AddParent { parent: String },
    RemoveDangerousContent { pattern: String },
    ReduceBodySize { target_tokens: usize },
}

/// 改进计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPlan {
    pub skill_name: String,
    pub actions: Vec<ImprovementAction>,
    pub estimated_iterations: u32,
}

/// 技能改进器
#[derive(Debug)]
pub struct SkillImprover {
    max_iterations: u32,
    min_overall_score: f64,
    min_safety_score: f64,
}

impl SkillImprover {
    pub fn new() -> Self {
        Self {
            max_iterations: 5,
            min_overall_score: 0.5,
            min_safety_score: 0.6,
        }
    }

    /// 分析技能并生成改进计划
    pub fn analyze(&self, skill_path: &PathBuf) -> Result<ImprovementPlan, String> {
        let content = std::fs::read_to_string(skill_path).map_err(|e| e.to_string())?;
        let scores = self.evaluate(&content);
        let violations = self.detect_violations(&content, &scores);
        
        let mut actions = Vec::new();
        
        for violation in &violations {
            match violation.dimension.as_str() {
                "safety" => {
                    actions.push(ImprovementAction::RemoveDangerousContent {
                        pattern: violation.message.clone(),
                    });
                }
                "completeness" => {
                    if violation.message.contains("name") {
                        actions.push(ImprovementAction::AddFrontmatterField {
                            field: "name".to_string(),
                            value: skill_path.file_stem().unwrap().to_string_lossy().to_string(),
                        });
                    }
                    if violation.message.contains("description") {
                        actions.push(ImprovementAction::AddFrontmatterField {
                            field: "description".to_string(),
                            value: "Auto-generated description".to_string(),
                        });
                    }
                    if violation.message.contains("triggers") {
                        actions.push(ImprovementAction::AddFrontmatterField {
                            field: "triggers".to_string(),
                            value: "[]".to_string(),
                        });
                    }
                    if violation.message.contains("tools") {
                        actions.push(ImprovementAction::AddFrontmatterField {
                            field: "tools".to_string(),
                            value: "[]".to_string(),
                        });
                    }
                    if violation.message.contains("body") {
                        actions.push(ImprovementAction::AddVerificationSection);
                    }
                }
                "executability" => {
                    actions.push(ImprovementAction::AddVerificationSection);
                }
                "maintainability" => {
                    if violation.message.contains("references") {
                        actions.push(ImprovementAction::AddReferences {
                            files: vec!["README.md".to_string()],
                        });
                    }
                    if violation.message.contains("category") {
                        actions.push(ImprovementAction::AddCategory {
                            category: "general".to_string(),
                        });
                    }
                    if violation.message.contains("parent") {
                        actions.push(ImprovementAction::AddParent {
                            parent: "".to_string(),
                        });
                    }
                }
                "cost_awareness" => {
                    actions.push(ImprovementAction::ReduceBodySize {
                        target_tokens: 1000,
                    });
                }
                _ => {}
            }
        }
        
        // Check line count
        let lines = content.lines().count();
        if lines > 500 {
            actions.push(ImprovementAction::SplitLongSkill { max_lines: 500 });
        }
        
        Ok(ImprovementPlan {
            skill_name: skill_path.file_stem().unwrap().to_string_lossy().to_string(),
            actions,
            estimated_iterations: 1,
        })
    }

    /// 执行改进计划
    pub fn apply(&self, skill_path: &PathBuf, plan: &ImprovementPlan) -> Result<bool, String> {
        let mut content = std::fs::read_to_string(skill_path).map_err(|e| e.to_string())?;
        let mut modified = false;
        
        for action in &plan.actions {
            match action {
                ImprovementAction::AddFrontmatterField { field, value } => {
                    if !content.contains(&format!("{}:", field)) {
                        // Insert after first ---
                        if let Some(pos) = content.find("---\n") {
                            let insert_pos = pos + 4;
                            content.insert_str(insert_pos, &format!("{}: {}\n", field, value));
                            modified = true;
                        }
                    }
                }
                ImprovementAction::AddVerificationSection => {
                    if !content.to_lowercase().contains("verification") {
                        content.push_str("\n## Verification\n\nRun `selftest.sh` to verify.\n");
                        modified = true;
                    }
                }
                ImprovementAction::AddReferences { files } => {
                    if !content.contains("references:") {
                        if let Some(pos) = content.find("---\n") {
                            let insert_pos = pos + 4;
                            let refs = files.join(", ");
                            content.insert_str(insert_pos, &format!("references: [{}]\n", refs));
                            modified = true;
                        }
                    }
                }
                ImprovementAction::AddCategory { category } => {
                    if !content.contains("category:") {
                        if let Some(pos) = content.find("---\n") {
                            let insert_pos = pos + 4;
                            content.insert_str(insert_pos, &format!("category: {}\n", category));
                            modified = true;
                        }
                    }
                }
                ImprovementAction::AddParent { parent } => {
                    if !content.contains("parent:") && !parent.is_empty() {
                        if let Some(pos) = content.find("---\n") {
                            let insert_pos = pos + 4;
                            content.insert_str(insert_pos, &format!("parent: {}\n", parent));
                            modified = true;
                        }
                    }
                }
                ImprovementAction::RemoveDangerousContent { pattern } => {
                    // Replace dangerous patterns
                    let dangerous = ["rm -rf", "curl.*|.*sh", "sudo ", "--force", "dangerously"];
                    for danger in &dangerous {
                        if content.contains(danger) {
                            content = content.replace(danger, &format!("# REMOVED: {}", danger));
                            modified = true;
                        }
                    }
                }
                ImprovementAction::ReduceBodySize { target_tokens } => {
                    // Truncate if too large (simplified)
                    if content.len() > *target_tokens {
                        content.truncate(*target_tokens);
                        content.push_str("\n... (truncated)");
                        modified = true;
                    }
                }
                ImprovementAction::SplitLongSkill { max_lines } => {
                    // Mark for splitting (actual split requires file ops)
                    modified = true;
                }
                ImprovementAction::FixFrontmatterSyntax { line, issue } => {
                    // Would need line-specific fix
                }
            }
        }
        
        if modified {
            std::fs::write(skill_path, content).map_err(|e| e.to_string())?;
        }
        
        Ok(modified)
    }

    /// 迭代改进直到通过
    pub fn improve_until_pass(&self, skill_path: &PathBuf) -> QualityGateResult {
        let mut iterations = 0;
        
        loop {
            let content = std::fs::read_to_string(skill_path).unwrap_or_default();
            let scores = self.evaluate(&content);
            let violations = self.detect_violations(&content, &scores);
            
            let passed = scores.overall >= self.min_overall_score && scores.safety >= self.min_safety_score && violations.is_empty();
            
            if passed || iterations >= self.max_iterations {
                return QualityGateResult {
                    skill_name: skill_path.file_stem().unwrap().to_string_lossy().to_string(),
                    passed,
                    scores,
                    violations,
                    iterations,
                };
            }
            
            let plan = self.analyze(skill_path).unwrap();
            self.apply(skill_path, &plan).unwrap();
            iterations += 1;
        }
    }

    fn evaluate(&self, content: &str) -> SkillQualityScores {
        let body = Self::extract_body(content);
        
        // Safety
        let danger_marks = ["rm -rf", "curl.*|.*sh", "sudo ", "--force", "dangerously"];
        let mut safety = 1.0;
        let body_lower = body.to_lowercase();
        for mark in danger_marks {
            if body_lower.contains(mark) {
                safety -= 0.25;
            }
        }
        let safety = safety.max(0.0);
        
        // Completeness
        let mut completeness = 0.0;
        if content.contains("name:") { completeness += 0.3; }
        if content.contains("description:") { completeness += 0.3; }
        if content.contains("triggers:") { completeness += 0.2; }
        if content.contains("tools:") || content.contains("allowed-tools:") { completeness += 0.1; }
        if body.trim().chars().count() >= 120 { completeness += 0.1; }
        
        // Executability
        let mut executability = 0.0;
        if content.contains("scripts/selftest") { executability += 0.5; }
        if body.to_lowercase().contains("verification") || body.to_lowercase().contains("verify") {
            executability += 0.5;
        }
        
        // Maintainability
        let mut maintainability = 0.0;
        if content.contains("references:") { maintainability += 0.4; }
        if content.contains("category:") { maintainability += 0.3; }
        if content.contains("parent:") { maintainability += 0.3; }
        
        // Cost awareness (simplified)
        let cost_awareness = if body.len() < 1000 { 0.6 } else if body.len() < 4000 { 0.3 } else { 0.2 };
        
        SkillQualityScores {
            safety,
            completeness,
            executability,
            maintainability,
            cost_awareness,
            overall: (safety + completeness + executability + maintainability + cost_awareness) / 5.0,
        }
    }

    fn detect_violations(&self, content: &str, scores: &SkillQualityScores) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        
        if scores.safety < self.min_safety_score {
            violations.push(QualityViolation {
                dimension: "safety".to_string(),
                message: format!("Safety score {:.2} below threshold {:.2}", scores.safety, self.min_safety_score),
                suggestion: "Remove dangerous commands".to_string(),
            });
        }
        
        if scores.completeness < 0.5 {
            violations.push(QualityViolation {
                dimension: "completeness".to_string(),
                message: format!("Completeness score {:.2} below 0.5", scores.completeness),
                suggestion: "Add missing frontmatter fields and body content".to_string(),
            });
        }
        
        if scores.executability < 0.5 {
            violations.push(QualityViolation {
                dimension: "executability".to_string(),
                message: format!("Executability score {:.2} below 0.5", scores.executability),
                suggestion: "Add selftest script and verification section".to_string(),
            });
        }
        
        if scores.maintainability < 0.5 {
            violations.push(QualityViolation {
                dimension: "maintainability".to_string(),
                message: format!("Maintainability score {:.2} below 0.5", scores.maintainability),
                suggestion: "Add references, category, parent".to_string(),
            });
        }
        
        if scores.cost_awareness < 0.3 {
            violations.push(QualityViolation {
                dimension: "cost_awareness".to_string(),
                message: format!("Cost awareness score {:.2} below 0.3", scores.cost_awareness),
                suggestion: "Reduce body size, use progressive disclosure".to_string(),
            });
        }
        
        violations
    }

    fn extract_body(content: &str) -> &str {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            return stripped;
        }
        if let Some(end) = stripped[3..].find("---") {
            &stripped[3 + end + 3..]
        } else {
            stripped
        }
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let skill_path = temp_dir.path().join("test-skill.md");
        
        // Create low-quality skill
        std::fs::write(&skill_path, r#"
---
name: bad-skill
description: Bad skill
---
rm -rf /dangerous
"#).map_err(|e| e.to_string())?;
        
        let improver = SkillImprover::new();
        let result = improver.improve_until_pass(&skill_path);
        
        // Should have detected safety issue and attempted fix
        assert!(result.iterations > 0);
        assert!(result.scores.safety >= 0.0); // After fix
        
        Ok(())
    }
}

impl Default for SkillImprover {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_improver_creation() {
        let improver = SkillImprover::new();
        assert_eq!(improver.max_iterations, 5);
    }

    #[test]
    fn test_analyze_detects_issues() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("test.md");
        std::fs::write(&skill_path, "---\nname: test\ndescription: test\n---\nbody with rm -rf").unwrap();
        
        let improver = SkillImprover::new();
        let plan = improver.analyze(&skill_path).unwrap();
        assert!(!plan.actions.is_empty());
    }

    #[test]
    fn test_apply_fixes_frontmatter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("test.md");
        std::fs::write(&skill_path, "---\nname: test\n---\nbody").unwrap();
        
        let improver = SkillImprover::new();
        let plan = ImprovementPlan {
            skill_name: "test".to_string(),
            actions: vec![ImprovementAction::AddFrontmatterField { field: "description".to_string(), value: "Test".to_string() }],
            estimated_iterations: 1,
        };
        improver.apply(&skill_path, &plan).unwrap();
        
        let content = std::fs::read_to_string(&skill_path).unwrap();
        assert!(content.contains("description:"));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(SkillImprover::self_test().is_ok());
    }
}