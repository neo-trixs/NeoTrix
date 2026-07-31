use std::collections::VecDeque;
use crate::core::nt_core_gwt::module_def::{SpecialistType, SpecialistModule};
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_hcube::gap::GapReport;
use crate::neotrix::nt_mind::exploration_pipeline::ExploreDomain;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuriosityLevel {
    Calm,
    Interested,
    Curious,
    IntenselyCurious,
}

impl CuriosityLevel {
    pub fn salience_multiplier(&self) -> f64 {
        match self {
            CuriosityLevel::Calm => 0.0,
            CuriosityLevel::Interested => 0.3,
            CuriosityLevel::Curious => 0.6,
            CuriosityLevel::IntenselyCurious => 0.9,
        }
    }
}

pub struct CuriositySignal {
    pub domain: ExploreDomain,
    pub intensity: f64,
    pub description: String,
    pub gap_report: Option<GapReport>,
    pub suggested_search_terms: Vec<String>,
}

pub struct CuriosityDrive {
    pub signals: VecDeque<CuriositySignal>,
    pub max_signals: usize,
    pub curiosity_level: CuriosityLevel,
    pub total_gaps_detected: u64,
    pub last_curiosity_ignition: Option<i64>,
    pub exploration_queries_generated: u64,
}

impl CuriosityDrive {
    pub fn new() -> Self {
        Self {
            signals: VecDeque::new(),
            max_signals: 20,
            curiosity_level: CuriosityLevel::Calm,
            total_gaps_detected: 0,
            last_curiosity_ignition: None,
            exploration_queries_generated: 0,
        }
    }

    pub fn ingest_gap_reports(&mut self, reports: &[GapReport]) {
        let high_gap_count = reports.iter().filter(|r| r.gap > 0.3).count();
        let total_sparsity: f64 = reports.iter().map(|r| r.sparsity_score).sum();
        let avg_sparsity = if reports.is_empty() { 0.0 } else { total_sparsity / reports.len() as f64 };

        self.total_gaps_detected += high_gap_count as u64;

        self.curiosity_level = if avg_sparsity > 0.7 {
            CuriosityLevel::IntenselyCurious
        } else if avg_sparsity > 0.4 {
            CuriosityLevel::Curious
        } else if avg_sparsity > 0.2 {
            CuriosityLevel::Interested
        } else {
            CuriosityLevel::Calm
        };

        for report in reports.iter().filter(|r| r.gap > 0.3 || r.sparsity_score > 0.4) {
            let domain = self.gap_to_domain(report);
            let search_terms = self.generate_search_terms(report);

            self.signals.push_back(CuriositySignal {
                domain,
                intensity: (report.gap + report.sparsity_score) / 2.0,
                description: format!("dim={}: gap={:.2}, sparsity={:.2}, empty={}", report.dim_index, report.gap, report.sparsity_score, report.empty_regions.len()),
                gap_report: Some(report.clone()),
                suggested_search_terms: search_terms,
            });
        }

        while self.signals.len() > self.max_signals {
            self.signals.pop_front();
        }
    }

    fn gap_to_domain(&self, report: &GapReport) -> ExploreDomain {
        match report.dim_index {
            0 | 1 => ExploreDomain::Wiki,
            2 | 3 => ExploreDomain::Papers,
            4 | 5 => ExploreDomain::GitHub,
            _ => ExploreDomain::General,
        }
    }

    fn generate_search_terms(&self, report: &GapReport) -> Vec<String> {
        let dim_names = ["Time", "Abstraction", "Domain", "Modality", "Culture", "Scale", "Certainty", "Agency"];
        let dim_name = dim_names.get(report.dim_index).unwrap_or(&"Unknown");
        let terms = vec![
            format!("knowledge gap {} dimension", dim_name),
            format!("{} research frontier", dim_name),
        ];
        terms
    }

    pub fn top_signals(&self, n: usize) -> Vec<&CuriositySignal> {
        let mut sorted: Vec<_> = self.signals.iter().collect();
        sorted.sort_by(|a, b| b.intensity.partial_cmp(&a.intensity).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    pub fn register_into_gwt(&self, gw: &mut GlobalWorkspace) {
        let intensity = self.curiosity_level.salience_multiplier();
        if intensity < 0.3 {
            return;
        }

        let specialist = SpecialistModule::new(
            SpecialistType::KnowledgeIntegrator,
            "CuriosityDriver".to_string(),
        );
        gw.register(specialist);

        let top = self.top_signals(3);
        for signal in &top {
            let search_specialist = SpecialistModule::new(
                SpecialistType::PatternMatcher,
                format!("Curious:{}", signal.description.chars().take(20).collect::<String>()),
            );
            gw.register(search_specialist);
        }
    }

    pub fn drain_queries(&mut self) -> Vec<String> {
        let mut queries = Vec::new();
        for signal in &self.signals {
            if signal.intensity > 0.5 {
                queries.extend(signal.suggested_search_terms.clone());
            }
        }
        self.exploration_queries_generated += queries.len() as u64;
        queries
    }

    pub fn summary(&self) -> String {
        format!(
            "Curiosity: {:?} | {} signals pending | {} gaps total | {} queries generated",
            self.curiosity_level,
            self.signals.len(),
            self.total_gaps_detected,
            self.exploration_queries_generated,
        )
    }
}

impl Default for CuriosityDrive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::gap::GapReport;

    fn sample_gap_report(dim: usize, gap: f64) -> GapReport {
        let mut report = GapReport::new(dim, 0.5, 0.6);
        report.gap = gap;
        report.sparsity_score = gap;
        report
    }

    #[test]
    fn test_new_curiosity_is_calm() {
        let drive = CuriosityDrive::new();
        assert_eq!(drive.curiosity_level, CuriosityLevel::Calm);
    }

    #[test]
    fn test_ingest_gaps_raises_curiosity() {
        let mut drive = CuriosityDrive::new();
        let reports = vec![
            sample_gap_report(0, 0.8),
            sample_gap_report(1, 0.7),
            sample_gap_report(2, 0.6),
        ];
        drive.ingest_gap_reports(&reports);
        assert_eq!(drive.curiosity_level, CuriosityLevel::IntenselyCurious);
        assert!(drive.total_gaps_detected >= 3);
    }

    #[test]
    fn test_low_gaps_low_curiosity() {
        let mut drive = CuriosityDrive::new();
        let reports = vec![sample_gap_report(0, 0.1)];
        drive.ingest_gap_reports(&reports);
        assert_eq!(drive.curiosity_level, CuriosityLevel::Calm);
    }

    #[test]
    fn test_top_signals_returns_sorted() {
        let mut drive = CuriosityDrive::new();
        drive.ingest_gap_reports(&[
            sample_gap_report(0, 0.9),
            sample_gap_report(1, 0.3),
        ]);
        let top = drive.top_signals(1);
        assert_eq!(top.len(), 1);
        assert!(top[0].intensity > 0.5);
    }

    #[test]
    fn test_drain_queries() {
        let mut drive = CuriosityDrive::new();
        drive.ingest_gap_reports(&[sample_gap_report(0, 0.8)]);
        let queries = drive.drain_queries();
        assert!(!queries.is_empty());
    }

    #[test]
    fn test_drain_queries_low_intensity() {
        let mut drive = CuriosityDrive::new();
        drive.ingest_gap_reports(&[sample_gap_report(0, 0.1)]);
        let queries = drive.drain_queries();
        assert!(queries.is_empty());
    }
}
