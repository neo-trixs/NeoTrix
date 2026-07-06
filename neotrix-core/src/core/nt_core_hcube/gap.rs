use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct GapReport {
    pub dim_index: usize,
    pub current_value: f64,
    pub target_value: f64,
    pub gap: f64,
    pub empty_regions: HashSet<usize>,
    pub underpopulated_regions: HashSet<usize>,
    pub sparsity_score: f64,
}

impl GapReport {
    pub fn new(dim_index: usize, current_value: f64, target_value: f64) -> Self {
        Self {
            dim_index, current_value, target_value,
            gap: target_value - current_value,
            empty_regions: HashSet::new(),
            underpopulated_regions: HashSet::new(),
            sparsity_score: 0.0,
        }
    }

    pub fn analyze(&self) -> Vec<String> {
        let mut findings = Vec::new();
        if self.gap > 0.0 {
            findings.push(format!("dim {} gap: {:.3} (cur={:.3}, target={:.3})",
                self.dim_index, self.gap, self.current_value, self.target_value));
        }
        if self.sparsity_score > 0.5 {
            findings.push(format!("sparsity {:.3} exceeds threshold", self.sparsity_score));
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_report_positive_gap() {
        let r = GapReport::new(3, 0.4, 0.9);
        assert!((r.gap - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_gap_report_negative_gap() {
        let r = GapReport::new(1, 0.8, 0.3);
        assert!((r.gap - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn test_gap_report_zero_gap() {
        let r = GapReport::new(0, 0.5, 0.5);
        assert!((r.gap - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_analyze_positive_gap_returns_finding() {
        let r = GapReport::new(2, 0.2, 0.8);
        let findings = r.analyze();
        assert!(!findings.is_empty());
        assert!(findings[0].contains("gap"));
    }

    #[test]
    fn test_analyze_zero_gap_returns_empty() {
        let r = GapReport::new(0, 0.5, 0.5);
        let findings = r.analyze();
        assert!(findings.is_empty());
    }

    #[test]
    fn test_analyze_sparsity_above_threshold() {
        let mut r = GapReport::new(0, 0.3, 0.7);
        r.sparsity_score = 0.8;
        let findings = r.analyze();
        assert!(findings.iter().any(|f| f.contains("sparsity")));
    }

    #[test]
    fn test_analyze_sparsity_below_threshold() {
        let mut r = GapReport::new(0, 0.3, 0.7);
        r.sparsity_score = 0.3;
        let findings = r.analyze();
        assert!(!findings.iter().any(|f| f.contains("sparsity")));
    }

    #[test]
    fn test_gap_report_empty_regions_default() {
        let r = GapReport::new(0, 0.0, 1.0);
        assert!(r.empty_regions.is_empty());
        assert!(r.underpopulated_regions.is_empty());
    }
}
