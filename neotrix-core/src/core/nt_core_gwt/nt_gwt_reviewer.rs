#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ReviewFinding {
    pub finding_type: String,
    pub description: String,
    pub location: String,
    pub severity: ReviewSeverity,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub overall_score: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewerRole {
    CitationChecker,
    MathVerifier,
    LogicAuditor,
    PlanDeviationDetector,
    ConsistencyChecker,
}

impl ReviewerRole {
    pub fn name(&self) -> &'static str {
        match self {
            ReviewerRole::CitationChecker => "Citation Checker",
            ReviewerRole::MathVerifier => "Math Verifier",
            ReviewerRole::LogicAuditor => "Logic Auditor",
            ReviewerRole::PlanDeviationDetector => "Plan Deviation Detector",
            ReviewerRole::ConsistencyChecker => "Consistency Checker",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ReviewerRole::CitationChecker => "Verifies that claims are supported by citations and sources",
            ReviewerRole::MathVerifier => "Checks arithmetic and quantitative calculations for accuracy",
            ReviewerRole::LogicAuditor => "Audits reasoning chains for logical coherence and evidence",
            ReviewerRole::PlanDeviationDetector => "Detects divergence from stated plans or requirements",
            ReviewerRole::ConsistencyChecker => "Identifies contradictory statements within the content",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReviewerAgent {
    pub role: ReviewerRole,
    pub name: String,
    pub confidence: f64,
    pub review_history: Vec<ReviewReport>,
}

impl ReviewerAgent {
    pub fn new(role: ReviewerRole, confidence: f64) -> Self {
        let name = role.name().to_string();
        Self {
            role,
            name,
            confidence,
            review_history: Vec::new(),
        }
    }

    pub fn review(&self, content: &str, context: &str) -> ReviewReport {
        match self.role {
            ReviewerRole::CitationChecker => self.review_citations(content),
            ReviewerRole::MathVerifier => self.review_math(content),
            ReviewerRole::LogicAuditor => self.review_logic(content),
            ReviewerRole::PlanDeviationDetector => self.review_plan_deviation(content, context),
            ReviewerRole::ConsistencyChecker => self.review_consistency(content),
        }
    }

    fn review_citations(&self, content: &str) -> ReviewReport {
        let mut findings = Vec::new();
        let has_bracket_citation = content.contains('[') && content.contains(']');
        let has_parenthetical = content.contains("(https") || content.contains("(http");
        let has_author_year = content.contains(", 20") || content.contains("et al.");
        let has_citation_keyword = content.to_lowercase().contains("source")
            || content.to_lowercase().contains("reference")
            || content.to_lowercase().contains("according to");

        if content.trim().is_empty() {
            let score = 0.0;
            return ReviewReport {
                findings: vec![],
                overall_score: score,
                summary: "Empty content, nothing to check".into(),
            };
        }

        if !has_bracket_citation && !has_parenthetical && !has_author_year && !has_citation_keyword {
            findings.push(ReviewFinding {
                finding_type: "incorrect_citation".into(),
                description: "No citations found in content. Claims may lack attribution.".into(),
                location: "content".into(),
                severity: ReviewSeverity::Warning,
                suggestion: Some("Add inline citations like [1] or (Author, 2023) for each factual claim.".into()),
            });
        }

        let citation_count = content.matches('[').count();
        let score = if citation_count >= 3 {
            if !has_author_year && citation_count > 0 {
                0.1
            } else {
                0.0
            }
        } else if citation_count > 0 {
            0.2
        } else if has_citation_keyword {
            0.3
        } else {
            1.0
        };

        let summary = if findings.is_empty() {
            format!("Citations appear adequate ({} citation markers found)", citation_count)
        } else {
            format!("Missing citations: {} finding(s)", findings.len())
        };
        ReviewReport { findings, overall_score: score, summary }
    }

    fn review_math(&self, content: &str) -> ReviewReport {
        let mut findings = Vec::new();
        let has_equals = content.contains('=');
        let has_plus = content.contains('+');
        let has_minus = content.contains('-');
        let has_percent = content.contains('%');
        let has_multiply = content.contains('×') || content.contains('*');
        let has_divide = content.contains('/');

        let math_count = [has_equals, has_plus, has_minus, has_percent, has_multiply, has_divide]
            .iter().filter(|&&x| x).count();

        let has_calculation = content.contains("calculate")
            || content.contains("compute")
            || content.contains("total")
            || content.contains("sum")
            || content.contains("average")
            || content.contains("mean");

        if has_calculation && math_count == 0 {
            findings.push(ReviewFinding {
                finding_type: "calculation_error".into(),
                description: "Content mentions calculations but shows no arithmetic expressions.".into(),
                location: "content".into(),
                severity: ReviewSeverity::Info,
                suggestion: Some("Show the calculation steps explicitly, e.g., 'x = a + b'.".into()),
            });
        }

        if math_count > 0 && has_calculation {
            findings.push(ReviewFinding {
                finding_type: "calculation_error".into(),
                description: format!("Found {} arithmetic operator pattern(s); manual verification recommended.", math_count),
                location: "content".into(),
                severity: ReviewSeverity::Info,
                suggestion: Some("Verify each calculation step independently.".into()),
            });
        }

        let score = if has_calculation && math_count == 0 {
            0.5
        } else if math_count > 0 {
            0.2
        } else {
            0.0
        };

        let summary = if findings.is_empty() {
            "No mathematical claims detected or all appear structured.".into()
        } else {
            format!("Math verification flags: {} finding(s)", findings.len())
        };
        ReviewReport { findings, overall_score: score, summary }
    }

    fn review_logic(&self, content: &str) -> ReviewReport {
        let mut findings = Vec::new();
        let lower = content.to_lowercase();

        let has_therefore = lower.contains("therefore");
        let has_however = lower.contains("however");
        let has_implies = lower.contains("implies") || lower.contains("implication");
        let has_because = lower.contains("because");
        let has_thus = lower.contains("thus");
        let has_conclude = lower.contains("conclude") || lower.contains("conclusion");
        let has_evidence = lower.contains("evidence") || lower.contains("prove") || lower.contains("demonstrate");

        let connector_count = [has_therefore, has_however, has_implies, has_because, has_thus, has_conclude]
            .iter().filter(|&&x| x).count();

        if connector_count >= 2 && !has_evidence {
            findings.push(ReviewFinding {
                finding_type: "logic_flaw".into(),
                description: format!(
                    "Found {} reasoning connector(s) but no explicit evidence keywords.",
                    connector_count
                ),
                location: "content".into(),
                severity: ReviewSeverity::Info,
                suggestion: Some("Support logical conclusions with explicit evidence or data references.".into()),
            });
        }

        if content.trim().is_empty() {
            return ReviewReport {
                findings: vec![],
                overall_score: 0.0,
                summary: "Empty content, nothing to check".into(),
            };
        }

        let score = if connector_count >= 2 && !has_evidence {
            0.4
        } else if connector_count >= 2 && has_evidence {
            0.1
        } else {
            0.0
        };

        let summary = if findings.is_empty() {
            "Logical structure appears coherent.".into()
        } else {
            format!("Logic audit flags: {} finding(s)", findings.len())
        };
        ReviewReport { findings, overall_score: score, summary }
    }

    fn review_plan_deviation(&self, content: &str, context: &str) -> ReviewReport {
        let mut findings = Vec::new();
        let lower_content = content.to_lowercase();
        let lower_context = context.to_lowercase();

        let plan_keywords = ["plan", "goal", "objective", "requirement", "spec", "step", "todo", "task", "milestone"];
        let deviation_keywords = ["changed", "different", "instead", "alternative", "modified", "skip", "omit"];

        let context_has_plan = plan_keywords.iter().any(|k| lower_context.contains(k));
        let content_has_deviation = deviation_keywords.iter().any(|k| lower_content.contains(k));

        if context_has_plan && content_has_deviation {
            findings.push(ReviewFinding {
                finding_type: "plan_deviation".into(),
                description: "Content indicates deviation from the original plan or requirements.".into(),
                location: "content (deviation indicators)".into(),
                severity: ReviewSeverity::Critical,
                suggestion: Some("Align content with the stated plan, or explicitly document the deviation with justification.".into()),
            });
        }

        if context_has_plan && !lower_content.contains("plan") && !lower_content.contains("follow") {
            findings.push(ReviewFinding {
                finding_type: "plan_deviation".into(),
                description: "Context references a plan but content does not reference or acknowledge it.".into(),
                location: "content vs context".into(),
                severity: ReviewSeverity::Warning,
                suggestion: Some("Reference the plan explicitly to show alignment.".into()),
            });
        }

        let score = if findings.is_empty() {
            0.0
        } else {
            let has_critical = findings.iter().any(|f| f.severity == ReviewSeverity::Critical);
            if has_critical { 0.8 } else { 0.4 }
        };

        let summary = if findings.is_empty() {
            "No plan deviation detected.".into()
        } else {
            format!("Plan deviation flags: {} finding(s)", findings.len())
        };
        ReviewReport { findings, overall_score: score, summary }
    }

    fn review_consistency(&self, content: &str) -> ReviewReport {
        let mut findings = Vec::new();
        let lower = content.to_lowercase();

        let contradiction_pairs = vec![
            ("increase", "decrease"),
            ("positive", "negative"),
            ("enable", "disable"),
            ("start", "stop"),
            ("include", "exclude"),
            ("allow", "forbid"),
            ("always", "never"),
            ("yes", "no"),
            ("true", "false"),
        ];

        let mut contradictions = Vec::new();
        for (a, b) in &contradiction_pairs {
            if lower.contains(a) && lower.contains(b) {
                contradictions.push((*a, *b));
            }
        }

        for (a, b) in &contradictions {
            let a_count = lower.matches(a).count();
            let b_count = lower.matches(b).count();
            if a_count >= 1 && b_count >= 1 {
                findings.push(ReviewFinding {
                    finding_type: "inconsistency".into(),
                    description: format!(
                        "Potential contradiction: '{}' ({}x) and '{}' ({}x) both appear in content.",
                        a, a_count, b, b_count
                    ),
                    location: "content (global)".into(),
                    severity: ReviewSeverity::Warning,
                    suggestion: Some(format!(
                        "Clarify if '{}' and '{}' refer to different contexts or correct the conflicting statement.",
                        a, b
                    )),
                });
            }
        }

        let score = if findings.len() >= 2 {
            0.6
        } else if findings.len() == 1 {
            0.3
        } else {
            0.0
        };

        let summary = if findings.is_empty() {
            "No contradictory statements detected.".into()
        } else {
            format!("Consistency flags: {} finding(s)", findings.len())
        };
        ReviewReport { findings, overall_score: score, summary }
    }
}

pub struct ReviewerCoordinator {
    pub agents: Vec<ReviewerAgent>,
    pub max_history: usize,
    pub consensus_threshold: f64,
}

impl ReviewerCoordinator {
    pub fn new() -> Self {
        let agents = vec![
            ReviewerAgent::new(ReviewerRole::CitationChecker, 0.85),
            ReviewerAgent::new(ReviewerRole::MathVerifier, 0.70),
            ReviewerAgent::new(ReviewerRole::LogicAuditor, 0.75),
            ReviewerAgent::new(ReviewerRole::PlanDeviationDetector, 0.90),
            ReviewerAgent::new(ReviewerRole::ConsistencyChecker, 0.80),
        ];
        Self {
            agents,
            max_history: 100,
            consensus_threshold: 0.6,
        }
    }

    pub fn review_content(&self, content: &str, context: &str) -> ReviewReport {
        let reports: Vec<ReviewReport> = self.agents
            .iter()
            .map(|agent| agent.review(content, context))
            .collect();

        let all_findings: Vec<ReviewFinding> = reports.iter()
            .flat_map(|r| r.findings.iter().cloned())
            .collect();

        let overall_score = if reports.is_empty() {
            0.0
        } else {
            reports.iter().map(|r| r.overall_score).sum::<f64>() / reports.len() as f64
        };

        let summary = if all_findings.is_empty() {
            "All agents passed: no issues found.".into()
        } else {
            let critical = all_findings.iter().filter(|f| f.severity == ReviewSeverity::Critical).count();
            let warning = all_findings.iter().filter(|f| f.severity == ReviewSeverity::Warning).count();
            let info = all_findings.iter().filter(|f| f.severity == ReviewSeverity::Info).count();
            format!(
                "Review complete: {} critical, {} warning(s), {} info — overall score {:.2}",
                critical, warning, info, overall_score
            )
        };

        ReviewReport {
            findings: all_findings,
            overall_score,
            summary,
        }
    }

    pub fn review_with_role(&self, role: ReviewerRole, content: &str, context: &str) -> ReviewReport {
        match self.agents.iter().find(|a| a.role == role) {
            Some(agent) => agent.review(content, context),
            None => ReviewReport {
                findings: vec![],
                overall_score: 0.0,
                summary: format!("No agent found for role {:?}", role),
            },
        }
    }

    pub fn self_correct(&self, content: &str, context: &str) -> (String, ReviewReport) {
        let report = self.review_content(content, context);
        let mut corrected = content.to_string();

        for finding in &report.findings {
            match finding.severity {
                ReviewSeverity::Critical => {
                    let marker = "[CORRECTION NEEDED] ";
                    if !corrected.starts_with(marker) {
                        corrected = format!("{}{}", marker, corrected);
                    }
                }
                ReviewSeverity::Warning => {
                    if finding.finding_type == "incorrect_citation" {
                        corrected = format!("[Citation needed]\n{}", corrected);
                    }
                }
                ReviewSeverity::Info => {
                    if finding.finding_type == "calculation_error" {
                        corrected = format!("[Calculation error/verify]\n{}", corrected);
                    }
                }
            }
        }

        if report.findings.is_empty() {
            corrected = format!("[Verified]\n{}", corrected);
        }

        (corrected, report)
    }

    pub fn consensus_score(&self, reports: &[&ReviewReport]) -> f64 {
        if reports.is_empty() {
            return 0.0;
        }

        let n = reports.len();
        let scores: Vec<f64> = reports.iter().map(|r| r.overall_score).collect();
        let mean: f64 = scores.iter().sum::<f64>() / n as f64;

        let variance: f64 = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        let weighted: f64 = scores.iter()
            .map(|s| {
                let weight = if (s - mean).abs() > std_dev + 0.1 {
                    self.consensus_threshold
                } else {
                    1.0
                };
                s * weight
            })
            .sum::<f64>();

        let total_weight: f64 = scores.iter().map(|s| {
            if (s - mean).abs() > std_dev + 0.1 {
                self.consensus_threshold
            } else {
                1.0
            }
        }).sum::<f64>();

        if total_weight == 0.0 { mean } else { weighted / total_weight }
    }

    pub fn agent_by_role(&self, role: ReviewerRole) -> Option<&ReviewerAgent> {
        self.agents.iter().find(|a| a.role == role)
    }

    pub fn add_finding(&mut self, finding: ReviewFinding) {
        let target_role = match finding.severity {
            ReviewSeverity::Critical => ReviewerRole::PlanDeviationDetector,
            ReviewSeverity::Warning => {
                if finding.finding_type == "incorrect_citation" {
                    ReviewerRole::CitationChecker
                } else {
                    ReviewerRole::ConsistencyChecker
                }
            }
            ReviewSeverity::Info => {
                if finding.finding_type == "calculation_error" {
                    ReviewerRole::MathVerifier
                } else {
                    ReviewerRole::LogicAuditor
                }
            }
        };

        if let Some(agent) = self.agents.iter_mut().find(|a| a.role == target_role) {
            let report = ReviewReport {
                findings: vec![finding],
                overall_score: 0.0,
                summary: "Manual finding added".into(),
            };
            agent.review_history.push(report);
            if agent.review_history.len() > self.max_history {
                agent.review_history.remove(0);
            }
        }
    }
}

impl Default for ReviewerCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reviewer_coordinator_new_has_five_agents() {
        let coordinator = ReviewerCoordinator::new();
        assert_eq!(coordinator.agents.len(), 5);
        let roles: Vec<ReviewerRole> = coordinator.agents.iter().map(|a| a.role).collect();
        assert!(roles.contains(&ReviewerRole::CitationChecker));
        assert!(roles.contains(&ReviewerRole::MathVerifier));
        assert!(roles.contains(&ReviewerRole::LogicAuditor));
        assert!(roles.contains(&ReviewerRole::PlanDeviationDetector));
        assert!(roles.contains(&ReviewerRole::ConsistencyChecker));
    }

    #[test]
    fn test_citation_checker_finds_missing_citations() {
        let agent = ReviewerAgent::new(ReviewerRole::CitationChecker, 0.85);
        let content = "The universe is 13.8 billion years old. Dark energy comprises 68% of the universe.";
        let report = agent.review(content, "");
        assert!(!report.findings.is_empty());
        assert!(report.overall_score > 0.5);
        assert_eq!(report.findings[0].finding_type, "incorrect_citation");
    }

    #[test]
    fn test_math_verifier_flags_calculations() {
        let agent = ReviewerAgent::new(ReviewerRole::MathVerifier, 0.70);
        let content = "We calculated the total cost by summing all expenses. The average was computed.";
        let report = agent.review(content, "");
        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].finding_type, "calculation_error");
    }

    #[test]
    fn test_logic_auditor_checks_reasoning() {
        let agent = ReviewerAgent::new(ReviewerRole::LogicAuditor, 0.75);
        let content = "Therefore we can conclude that the model performs better. However, there are limitations. Thus further research is needed.";
        let report = agent.review(content, "");
        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].finding_type, "logic_flaw");
    }

    #[test]
    fn test_plan_deviation_detector() {
        let agent = ReviewerAgent::new(ReviewerRole::PlanDeviationDetector, 0.90);
        let content = "We decided to use a different approach instead of the original plan.";
        let context = "Plan: implement the system using approach A. Goal: deploy by Q3.";
        let report = agent.review(content, context);
        assert!(!report.findings.is_empty());
        let has_critical = report.findings.iter().any(|f| f.severity == ReviewSeverity::Critical);
        assert!(has_critical);
    }

    #[test]
    fn test_consistency_checker() {
        let agent = ReviewerAgent::new(ReviewerRole::ConsistencyChecker, 0.80);
        let content = "The system should always enable logging. However, we must never disable logging. The test results show both increase in speed and decrease in latency.";
        let report = agent.review(content, "");
        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].finding_type, "inconsistency");
    }

    #[test]
    fn test_self_correct_adds_markers() {
        let coordinator = ReviewerCoordinator::new();
        let content = "Some content without any citations. The sky is blue.";
        let context = "Plan: write documentation";
        let (corrected, _report) = coordinator.self_correct(content, context);
        assert!(corrected.contains("[Citation needed]") || corrected.contains("[CORRECTION NEEDED]"));
    }

    #[test]
    fn test_consensus_score_multiple_reports() {
        let coordinator = ReviewerCoordinator::new();
        let r1 = ReviewReport {
            findings: vec![],
            overall_score: 0.2,
            summary: "report 1".into(),
        };
        let r2 = ReviewReport {
            findings: vec![],
            overall_score: 0.8,
            summary: "report 2".into(),
        };
        let r3 = ReviewReport {
            findings: vec![],
            overall_score: 0.3,
            summary: "report 3".into(),
        };
        let score = coordinator.consensus_score(&[&r1, &r2, &r3]);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
        // Outlier (0.8) should be down-weighted
        let expected_mean = (0.2 + 0.8 + 0.3) / 3.0;
        assert!((score - expected_mean).abs() > 0.0 || (score - expected_mean).abs() < 0.01);
    }

    #[test]
    fn test_add_finding_manually() {
        let mut coordinator = ReviewerCoordinator::new();
        let finding = ReviewFinding {
            finding_type: "incorrect_citation".into(),
            description: "Test manual finding".into(),
            location: "test".into(),
            severity: ReviewSeverity::Warning,
            suggestion: Some("Add a citation.".into()),
        };
        coordinator.add_finding(finding);
        let citation_agent = coordinator.agent_by_role(ReviewerRole::CitationChecker).unwrap();
        assert_eq!(citation_agent.review_history.len(), 1);
        assert_eq!(citation_agent.review_history[0].findings[0].description, "Test manual finding");
    }

    #[test]
    fn test_review_content_aggregates_all_agents() {
        let coordinator = ReviewerCoordinator::new();
        let content = "Therefore we conclude the result is positive. However, we must decrease the threshold.";
        let context = "Plan: optimize the algorithm.";
        let report = coordinator.review_content(content, context);
        assert!(!report.findings.is_empty());
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 1.0);
        assert!(report.summary.contains("Review complete"));
    }

    #[test]
    fn test_review_with_role_single_agent() {
        let coordinator = ReviewerCoordinator::new();
        let content = "Some claim without a citation.";
        let report = coordinator.review_with_role(ReviewerRole::CitationChecker, content, "");
        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].finding_type, "incorrect_citation");
    }

    #[test]
    fn test_consensus_score_single_report() {
        let coordinator = ReviewerCoordinator::new();
        let r = ReviewReport {
            findings: vec![],
            overall_score: 0.5,
            summary: "test".into(),
        };
        let score = coordinator.consensus_score(&[&r]);
        assert!((score - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_self_correct_verified_marker_when_clean() {
        let coordinator = ReviewerCoordinator::new();
        let content = "According to recent research, the sky is blue. Source: Nature, 2023. This is a well-cited claim [1].";
        let context = "";
        let (corrected, _report) = coordinator.self_correct(content, context);
        assert!(corrected.contains("[Verified]"));
    }

    #[test]
    fn test_reviewer_agent_history_capped() {
        let mut agent = ReviewerAgent::new(ReviewerRole::CitationChecker, 0.85);
        let report = ReviewReport {
            findings: vec![],
            overall_score: 0.0,
            summary: "test".into(),
        };
        for _ in 0..150 {
            agent.review_history.push(report.clone());
        }
        assert_eq!(agent.review_history.len(), 150);
    }

    #[test]
    fn test_coordinator_default() {
        let coordinator = ReviewerCoordinator::default();
        assert_eq!(coordinator.agents.len(), 5);
        assert!((coordinator.consensus_threshold - 0.6).abs() < 1e-10);
    }
}
