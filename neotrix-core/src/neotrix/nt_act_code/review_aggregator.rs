use crate::neotrix::nt_mind::code_review::{IssueSeverity, ReviewIssue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReviewResult {
    pub agent_name: String,
    pub dimension: String,
    pub issues: Vec<ReviewIssue>,
    pub score: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedReviewReport {
    pub file: String,
    pub agent_results: Vec<AgentReviewResult>,
    pub unique_issues: Vec<ReviewIssue>,
    pub grouped_by_severity: HashMap<String, Vec<ReviewIssue>>,
    pub consolidated_score: f64,
    pub total_agents: usize,
    pub total_issues: usize,
    pub unique_issue_count: usize,
    pub duration_ms: u64,
}

impl AggregatedReviewReport {
    pub fn critical(&self) -> &[ReviewIssue] {
        self.issues_by_sev("Critical")
    }
    pub fn high(&self) -> &[ReviewIssue] {
        self.issues_by_sev("High")
    }
    pub fn medium(&self) -> &[ReviewIssue] {
        self.issues_by_sev("Medium")
    }
    pub fn summary(&self) -> String {
        format!(
            "Review: {} agents, {:.1}% score, {} unique issues ({} critical, {} high)",
            self.total_agents,
            self.consolidated_score * 100.0,
            self.unique_issue_count,
            self.critical().len(),
            self.high().len(),
        )
    }

    fn issues_by_sev(&self, sev: &str) -> &[ReviewIssue] {
        self.grouped_by_severity
            .get(sev)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

pub struct ReviewAggregator;

impl ReviewAggregator {
    pub fn aggregate(file: &str, agent_results: Vec<AgentReviewResult>) -> AggregatedReviewReport {
        let start = Instant::now();

        let all_issues: Vec<&ReviewIssue> =
            agent_results.iter().flat_map(|r| r.issues.iter()).collect();

        let unique = Self::deduplicate(&all_issues);

        let mut grouped: HashMap<String, Vec<ReviewIssue>> = HashMap::new();
        for issue in &unique {
            let key = format!("{:?}", issue.severity);
            grouped.entry(key).or_default().push(issue.clone());
        }
        for key in ["Critical", "High", "Medium", "Low", "Info"] {
            grouped.entry(key.to_string()).or_default();
        }

        let severity_order = |s: &IssueSeverity| -> u8 {
            match s {
                IssueSeverity::Critical => 0,
                IssueSeverity::High => 1,
                IssueSeverity::Medium => 2,
                IssueSeverity::Low => 3,
                IssueSeverity::Info => 4,
            }
        };

        let mut flat_issues: Vec<ReviewIssue> = unique.into_iter().collect();
        flat_issues.sort_by_key(|i| severity_order(&i.severity));

        let n_agents = agent_results.len();
        let consolidated = if n_agents > 0 {
            agent_results.iter().map(|r| r.score).sum::<f64>() / n_agents as f64
        } else {
            1.0
        };

        let elapsed = start.elapsed().as_millis() as u64;
        let total_issues: usize = agent_results.iter().map(|r| r.issues.len()).sum();
        let unique_count = flat_issues.len();

        AggregatedReviewReport {
            file: file.to_string(),
            agent_results,
            unique_issues: flat_issues,
            grouped_by_severity: grouped,
            consolidated_score: consolidated,
            total_agents: n_agents,
            total_issues,
            unique_issue_count: unique_count,
            duration_ms: elapsed,
        }
    }

    fn deduplicate(issues: &[&ReviewIssue]) -> Vec<ReviewIssue> {
        let mut seen: HashMap<(u32, String, String), ReviewIssue> = HashMap::new();

        for &issue in issues {
            let line = issue.line.unwrap_or(0);
            let cat = format!("{:?}", issue.category);
            let msg_trunc: String = issue.message.chars().take(60).collect();

            let key = (line, cat, msg_trunc);

            let entry = seen.entry(key).or_insert_with(|| issue.clone());
            if severity_rank(&issue.severity) > severity_rank(&entry.severity) {
                *entry = issue.clone();
            }
        }

        let mut result: Vec<ReviewIssue> = seen.into_values().collect();
        result.sort_by_key(|i| severity_rank(&i.severity));
        result
    }
}

fn severity_rank(s: &IssueSeverity) -> u8 {
    match s {
        IssueSeverity::Critical => 0,
        IssueSeverity::High => 1,
        IssueSeverity::Medium => 2,
        IssueSeverity::Low => 3,
        IssueSeverity::Info => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::code_review::IssueCategory;

    fn make_issue(
        sev: IssueSeverity,
        line: Option<u32>,
        msg: &str,
        cat: IssueCategory,
    ) -> ReviewIssue {
        ReviewIssue {
            severity: sev,
            category: cat,
            message: msg.to_string(),
            line,
            suggestion: None,
        }
    }

    #[test]
    fn test_dedup_same_line_same_category() {
        let issues = vec![
            make_issue(
                IssueSeverity::Low,
                Some(10),
                "unwrap detected",
                IssueCategory::ErrorHandling,
            ),
            make_issue(
                IssueSeverity::Medium,
                Some(10),
                "unwrap detected",
                IssueCategory::ErrorHandling,
            ),
        ];
        let refs: Vec<&ReviewIssue> = issues.iter().collect();
        let deduped = ReviewAggregator::deduplicate(&refs);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].severity, IssueSeverity::Medium);
    }

    #[test]
    fn test_dedup_preserves_different_lines() {
        let issues = vec![
            make_issue(
                IssueSeverity::High,
                Some(5),
                "panic",
                IssueCategory::ErrorHandling,
            ),
            make_issue(
                IssueSeverity::High,
                Some(10),
                "panic",
                IssueCategory::ErrorHandling,
            ),
        ];
        let refs: Vec<&ReviewIssue> = issues.iter().collect();
        let deduped = ReviewAggregator::deduplicate(&refs);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_aggregate_empty() {
        let report = ReviewAggregator::aggregate("test.rs", vec![]);
        assert_eq!(report.total_agents, 0);
        assert_eq!(report.unique_issue_count, 0);
        assert!((report.consolidated_score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_keeps_highest_severity() {
        let result1 = AgentReviewResult {
            agent_name: "sec".into(),
            dimension: "Security".into(),
            issues: vec![make_issue(
                IssueSeverity::Medium,
                None,
                "test issue",
                IssueCategory::Security,
            )],
            score: 0.9,
            duration_ms: 10,
        };
        let result2 = AgentReviewResult {
            agent_name: "perf".into(),
            dimension: "Performance".into(),
            issues: vec![make_issue(
                IssueSeverity::High,
                None,
                "test issue",
                IssueCategory::Performance,
            )],
            score: 0.8,
            duration_ms: 10,
        };
        let report = ReviewAggregator::aggregate("f.rs", vec![result1, result2]);
        assert_eq!(report.unique_issue_count, 1);
        assert_eq!(report.unique_issues[0].severity, IssueSeverity::High);
    }

    #[test]
    fn test_severity_grouping_all_categories() {
        let issues = vec![
            make_issue(IssueSeverity::Critical, None, "c1", IssueCategory::Security),
            make_issue(IssueSeverity::High, None, "h1", IssueCategory::Performance),
            make_issue(
                IssueSeverity::Medium,
                None,
                "m1",
                IssueCategory::Correctness,
            ),
            make_issue(IssueSeverity::Low, None, "l1", IssueCategory::Style),
            make_issue(IssueSeverity::Info, None, "i1", IssueCategory::Testing),
        ];
        let result = AgentReviewResult {
            agent_name: "all".into(),
            dimension: "All".into(),
            issues,
            score: 0.7,
            duration_ms: 10,
        };
        let report = ReviewAggregator::aggregate("f.rs", vec![result]);
        assert_eq!(report.critical().len(), 1);
        assert_eq!(report.high().len(), 1);
        assert_eq!(report.medium().len(), 1);
        assert_eq!(report.grouped_by_severity.get("Low").unwrap().len(), 1);
        assert_eq!(report.grouped_by_severity.get("Info").unwrap().len(), 1);
    }
}
