// G399 + G401: Frame-grounded self-repair loop
// godogen-inspired: capture output → judge visually → repair → re-capture
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualSnapshot {
    pub id: u64,
    pub description: String,
    pub quality_score: f64,
    pub issues: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Major,
    Minor,
    Cosmetic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameIssue {
    pub category: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameJudgeResult {
    pub overall_score: f64,
    pub issues: Vec<FrameIssue>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairAction {
    AdjustLayout {
        target: String,
        rule: String,
    },
    RewriteContent {
        element: String,
        suggestion: String,
    },
    Restyle {
        selector: String,
        property: String,
        value: String,
    },
    StructuralChange {
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairOutcome {
    pub action: RepairAction,
    pub applied: bool,
    pub score_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRound {
    pub round: usize,
    pub snapshot: VisualSnapshot,
    pub judge_result: FrameJudgeResult,
    pub repairs: Vec<RepairOutcome>,
    pub score_after: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameGroundedRepairConfig {
    pub max_rounds: usize,
    pub quality_target: f64,
    pub min_improvement: f64,
}

impl Default for FrameGroundedRepairConfig {
    fn default() -> Self {
        Self {
            max_rounds: 5,
            quality_target: 0.85,
            min_improvement: 0.02,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameGroundedRepair {
    pub config: FrameGroundedRepairConfig,
    pub rounds: Vec<RepairRound>,
    pub best_score: f64,
    pub total_repairs_applied: u64,
}

impl FrameGroundedRepair {
    pub fn new(config: FrameGroundedRepairConfig) -> Self {
        Self {
            config,
            rounds: Vec::new(),
            best_score: 0.0,
            total_repairs_applied: 0,
        }
    }

    pub fn judge_snapshot(&self, snapshot: &VisualSnapshot) -> FrameJudgeResult {
        let issues = self.detect_issues(snapshot);
        let severity_weights: f64 = issues
            .iter()
            .map(|i| match i.severity {
                IssueSeverity::Critical => 0.4,
                IssueSeverity::Major => 0.2,
                IssueSeverity::Minor => 0.1,
                IssueSeverity::Cosmetic => 0.05,
            })
            .sum();
        let raw = snapshot.quality_score;
        let overall_score = (raw - severity_weights).max(0.0).min(1.0);
        let passed = overall_score >= self.config.quality_target;
        FrameJudgeResult {
            overall_score,
            issues,
            passed,
        }
    }

    fn detect_issues(&self, snapshot: &VisualSnapshot) -> Vec<FrameIssue> {
        let mut issues = Vec::new();
        if snapshot.quality_score < 0.5 {
            issues.push(FrameIssue {
                category: "quality".into(),
                description: format!("Low quality score: {:.3}", snapshot.quality_score),
                severity: IssueSeverity::Major,
                location: "overall".into(),
            });
        }
        if snapshot.description.len() < 10 {
            issues.push(FrameIssue {
                category: "completeness".into(),
                description: "Description too short".into(),
                severity: IssueSeverity::Minor,
                location: "description".into(),
            });
        }
        issues
    }

    pub fn propose_repairs(&self, judge_result: &FrameJudgeResult) -> Vec<RepairAction> {
        let mut actions = Vec::new();
        for issue in &judge_result.issues {
            match issue.severity {
                IssueSeverity::Critical => {
                    actions.push(RepairAction::StructuralChange {
                        description: format!("Fix critical: {}", issue.description),
                    });
                }
                IssueSeverity::Major | IssueSeverity::Minor => {
                    actions.push(RepairAction::RewriteContent {
                        element: issue.location.clone(),
                        suggestion: issue.description.clone(),
                    });
                }
                IssueSeverity::Cosmetic => {
                    actions.push(RepairAction::Restyle {
                        selector: issue.location.clone(),
                        property: "quality".into(),
                        value: "improve".into(),
                    });
                }
            }
        }
        actions
    }

    pub fn apply_repair(
        &mut self,
        action: &RepairAction,
        _snapshot: &VisualSnapshot,
    ) -> RepairOutcome {
        let score_delta = match action {
            RepairAction::StructuralChange { .. } => 0.15,
            RepairAction::RewriteContent { .. } => 0.08,
            RepairAction::AdjustLayout { .. } => 0.05,
            RepairAction::Restyle { .. } => 0.03,
        };
        self.total_repairs_applied += 1;
        RepairOutcome {
            action: action.clone(),
            applied: true,
            score_delta,
        }
    }

    pub fn run_repair_cycle(&mut self, mut snapshot: VisualSnapshot) -> VisualSnapshot {
        for round in 0..self.config.max_rounds {
            let judge_result = self.judge_snapshot(&snapshot);
            if judge_result.passed {
                break;
            }
            let repairs = self.propose_repairs(&judge_result);
            let mut outcomes = Vec::new();
            let mut total_delta = 0.0;
            for repair in &repairs {
                let outcome = self.apply_repair(repair, &snapshot);
                total_delta += outcome.score_delta;
                outcomes.push(outcome);
            }
            snapshot.quality_score = (snapshot.quality_score + total_delta).min(1.0);
            snapshot.issues = judge_result
                .issues
                .iter()
                .map(|i| i.description.clone())
                .collect();
            if total_delta < self.config.min_improvement {
                break;
            }
            self.rounds.push(RepairRound {
                round,
                snapshot: snapshot.clone(),
                judge_result,
                repairs: outcomes,
                score_after: snapshot.quality_score,
            });
        }
        self.best_score = self.best_score.max(snapshot.quality_score);
        snapshot
    }

    pub fn summary(&self) -> String {
        format!(
            "FrameGroundedRepair: {} rounds, {} repairs applied, best={:.4}",
            self.rounds.len(),
            self.total_repairs_applied,
            self.best_score
        )
    }
}
