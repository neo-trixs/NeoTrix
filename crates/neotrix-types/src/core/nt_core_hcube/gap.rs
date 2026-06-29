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
