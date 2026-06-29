use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Audience {
    Executive,
    Board,
    Team,
}

#[derive(Debug, Clone)]
pub struct StakeholderReport {
    pub audience: Audience,
    pub title: String,
    pub summary: String,
    pub key_metrics: Vec<(String, f64)>,
    pub recommendations: Vec<String>,
    pub risks: Vec<String>,
    pub next_steps: Vec<String>,
}

impl StakeholderReport {
    pub fn to_markdown(&self) -> String {
        let audience_label = match self.audience {
            Audience::Executive => "Executive Summary",
            Audience::Board => "Board Update",
            Audience::Team => "Team Report",
        };
        let mut md = format!("# {}\n\n", audience_label);
        md.push_str(&format!("**{}**\n\n", self.title));
        md.push_str(&format!("{}\n\n", self.summary));

        if !self.key_metrics.is_empty() {
            md.push_str("## Key Metrics\n\n");
            md.push_str("| Metric | Value |\n|--------|-------|\n");
            for (k, v) in &self.key_metrics {
                md.push_str(&format!("| {} | {:.2} |\n", k, v));
            }
            md.push('\n');
        }

        if !self.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for r in &self.recommendations {
                md.push_str(&format!("- {}\n", r));
            }
            md.push('\n');
        }

        if !self.risks.is_empty() {
            md.push_str("## Risks\n\n");
            for r in &self.risks {
                md.push_str(&format!("- 🔴 {}\n", r));
            }
            md.push('\n');
        }

        if !self.next_steps.is_empty() {
            md.push_str("## Next Steps\n\n");
            for (i, s) in self.next_steps.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, s));
            }
            md.push('\n');
        }

        md
    }
}

pub struct StakeholderCommunicator {
    pub tone_adjustments: HashMap<Audience, f64>,
}

impl Default for StakeholderCommunicator {
    fn default() -> Self {
        let mut tone_adjustments = HashMap::new();
        tone_adjustments.insert(Audience::Executive, 0.9);
        tone_adjustments.insert(Audience::Board, 0.7);
        tone_adjustments.insert(Audience::Team, 0.5);
        Self { tone_adjustments }
    }
}

impl StakeholderCommunicator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn executive_report(
        &self,
        title: &str,
        metrics: Vec<(String, f64)>,
        recommendations: Vec<String>,
    ) -> StakeholderReport {
        StakeholderReport {
            audience: Audience::Executive,
            title: title.to_string(),
            summary: format!("Executive-level overview focusing on strategic impact and ROI. Technical detail level: {:.0}%.", self.tone_adjustments[&Audience::Executive] * 100.0),
            key_metrics: metrics,
            recommendations,
            risks: vec![],
            next_steps: vec![],
        }
    }

    pub fn board_report(
        &self,
        title: &str,
        summary: &str,
        metrics: Vec<(String, f64)>,
        risks: Vec<String>,
        next_steps: Vec<String>,
    ) -> StakeholderReport {
        StakeholderReport {
            audience: Audience::Board,
            title: title.to_string(),
            summary: summary.to_string(),
            key_metrics: metrics,
            recommendations: vec![],
            risks,
            next_steps,
        }
    }

    pub fn team_report(
        &self,
        title: &str,
        summary: &str,
        technical_items: Vec<String>,
    ) -> StakeholderReport {
        StakeholderReport {
            audience: Audience::Team,
            title: title.to_string(),
            summary: summary.to_string(),
            key_metrics: vec![],
            recommendations: technical_items,
            risks: vec![],
            next_steps: vec![],
        }
    }

    pub fn from_metrics(
        &self,
        audience: Audience,
        title: &str,
        metrics: HashMap<String, f64>,
    ) -> StakeholderReport {
        let metric_vec: Vec<(String, f64)> = metrics.into_iter().collect();
        let summary = match audience {
            Audience::Executive => {
                "Performance summary with strategic recommendations.".to_string()
            }
            Audience::Board => "Board-level update on project status and risk profile.".to_string(),
            Audience::Team => "Detailed technical report for team execution.".to_string(),
        };
        StakeholderReport {
            audience,
            title: title.to_string(),
            summary,
            key_metrics: metric_vec,
            recommendations: vec![],
            risks: vec![],
            next_steps: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executive_report() {
        let comm = StakeholderCommunicator::new();
        let report = comm.executive_report(
            "Q3 Performance",
            vec![("Revenue".to_string(), 1.2e6), ("Growth".to_string(), 0.15)],
            vec!["Invest in AI pipeline".to_string()],
        );
        assert_eq!(report.audience, Audience::Executive);
        assert_eq!(report.key_metrics.len(), 2);
    }

    #[test]
    fn test_board_report() {
        let comm = StakeholderCommunicator::new();
        let report = comm.board_report(
            "Board Update",
            "All milestones on track",
            vec![("Velocity".to_string(), 0.85)],
            vec!["Staffing gap".to_string()],
            vec!["Hire Q4".to_string()],
        );
        assert_eq!(report.audience, Audience::Board);
        assert!(!report.risks.is_empty());
    }

    #[test]
    fn test_team_report() {
        let comm = StakeholderCommunicator::new();
        let report = comm.team_report(
            "Sprint Review",
            "Completed 8/10 stories",
            vec!["Refactor auth module".to_string()],
        );
        assert_eq!(report.audience, Audience::Team);
    }

    #[test]
    fn test_from_metrics() {
        let comm = StakeholderCommunicator::new();
        let mut metrics = HashMap::new();
        metrics.insert("Accuracy".to_string(), 0.95);
        let report = comm.from_metrics(Audience::Executive, "Model Report", metrics);
        assert_eq!(report.key_metrics.len(), 1);
    }

    #[test]
    fn test_markdown_executive() {
        let comm = StakeholderCommunicator::new();
        let report = comm.executive_report(
            "Test",
            vec![("Score".to_string(), 0.99)],
            vec!["Ship it".to_string()],
        );
        let md = report.to_markdown();
        assert!(md.contains("Executive Summary"));
        assert!(md.contains("Score"));
    }

    #[test]
    fn test_markdown_board() {
        let comm = StakeholderCommunicator::new();
        let report = comm.board_report(
            "Board Update",
            "Status green",
            vec![],
            vec!["Risk A".to_string()],
            vec!["Action B".to_string()],
        );
        let md = report.to_markdown();
        assert!(md.contains("Board Update"));
        assert!(md.contains("Risk A"));
        assert!(md.contains("Action B"));
    }

    #[test]
    fn test_markdown_team() {
        let comm = StakeholderCommunicator::new();
        let report = comm.team_report("Sprint", "Done", vec!["Fix bug".to_string()]);
        let md = report.to_markdown();
        assert!(md.contains("Team Report"));
        assert!(md.contains("Fix bug"));
    }

    #[test]
    fn test_default_tone() {
        let comm = StakeholderCommunicator::new();
        assert_eq!(comm.tone_adjustments[&Audience::Executive], 0.9);
        assert_eq!(comm.tone_adjustments[&Audience::Team], 0.5);
    }
}
