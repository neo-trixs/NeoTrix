use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum EvidenceDiagnosticity {
    StronglySupports,    // ++
    Supports,            // +
    Neutral,             // 0
    Contradicts,         // -
    StronglyContradicts, // --
}

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub id: usize,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct AchMatrix {
    pub hypotheses: Vec<Hypothesis>,
    pub evidence: Vec<String>,
    pub matrix: HashMap<(usize, usize), EvidenceDiagnosticity>, // (hypothesis_idx, evidence_idx)
}

impl AchMatrix {
    pub fn new() -> Self {
        Self {
            hypotheses: Vec::new(),
            evidence: Vec::new(),
            matrix: HashMap::new(),
        }
    }

    pub fn add_hypothesis(&mut self, label: &str, description: &str) -> usize {
        let id = self.hypotheses.len();
        self.hypotheses.push(Hypothesis {
            id,
            label: label.to_string(),
            description: description.to_string(),
        });
        id
    }

    pub fn add_evidence(&mut self, evidence: &str) -> usize {
        self.evidence.push(evidence.to_string());
        self.evidence.len() - 1
    }

    pub fn set_diagnosticity(
        &mut self,
        hypothesis_idx: usize,
        evidence_idx: usize,
        d: EvidenceDiagnosticity,
    ) {
        self.matrix.insert((hypothesis_idx, evidence_idx), d);
    }

    /// Get the diagnosticity score as a numerical value
    pub fn diagnosticity_value(d: &EvidenceDiagnosticity) -> f64 {
        match d {
            EvidenceDiagnosticity::StronglySupports => 2.0,
            EvidenceDiagnosticity::Supports => 1.0,
            EvidenceDiagnosticity::Neutral => 0.0,
            EvidenceDiagnosticity::Contradicts => -1.0,
            EvidenceDiagnosticity::StronglyContradicts => -2.0,
        }
    }

    /// Calculate how diagnostic a piece of evidence is (how well it distinguishes hypotheses)
    pub fn evidence_diagnosticity(&self, evidence_idx: usize) -> f64 {
        let mut scores = Vec::new();
        for h in 0..self.hypotheses.len() {
            if let Some(d) = self.matrix.get(&(h, evidence_idx)) {
                scores.push(Self::diagnosticity_value(d));
            }
        }
        if scores.len() < 2 {
            return 0.0;
        }
        let variance = scores.iter().map(|s| s * s).sum::<f64>() / scores.len() as f64;
        variance.sqrt()
    }

    /// Score each hypothesis: how strongly it is contradicted (lower = better)
    pub fn score_hypotheses(&self) -> Vec<(usize, f64)> {
        let mut scores = Vec::new();
        for h in 0..self.hypotheses.len() {
            let mut contradiction_score = 0.0;
            for e in 0..self.evidence.len() {
                if let Some(d) = self.matrix.get(&(h, e)) {
                    let val = Self::diagnosticity_value(d);
                    // Negative = evidence contradicts = bad for this hypothesis
                    // We want the hypothesis with the LEAST contradiction
                    contradiction_score += val; // negative values hurt
                }
            }
            scores.push((h, contradiction_score));
        }
        // Sort by score descending (most positive = least contradicted)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Return the winning hypothesis (least contradicted)
    pub fn winning_hypothesis(&self) -> Option<&Hypothesis> {
        let scores = self.score_hypotheses();
        scores
            .first()
            .and_then(|(idx, _)| self.hypotheses.get(*idx))
    }

    /// Remove non-diagnostic evidence (evidence where all hypotheses agree)
    pub fn remove_non_diagnostic_evidence(&mut self) -> Vec<usize> {
        let mut to_remove = Vec::new();
        for e in 0..self.evidence.len() {
            let diag = self.evidence_diagnosticity(e);
            if diag < 0.1 {
                // All hypotheses agree or all disagree — not diagnostic
                to_remove.push(e);
            }
        }
        // Remove in reverse order
        for idx in to_remove.iter().rev() {
            self.matrix.retain(|(_, e), _| *e != *idx);
            self.evidence.remove(*idx);
            // Fix indices
            let mut new_matrix = HashMap::new();
            for ((h, e), d) in self.matrix.drain() {
                let new_e = if e > *idx { e - 1 } else { e };
                new_matrix.insert((h, new_e), d);
            }
            self.matrix = new_matrix;
        }
        to_remove
    }

    pub fn summary(&self) -> String {
        let _scores = self.score_hypotheses();
        let winner = self.winning_hypothesis();
        let mut s = format!(
            "ACH: {} hypotheses x {} evidence | ",
            self.hypotheses.len(),
            self.evidence.len()
        );
        if let Some(w) = winner {
            s.push_str(&format!("winner='{}'", w.label));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct AchEngine {
    pub matrices: Vec<AchMatrix>,
    pub max_matrices: usize,
}

impl Default for AchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AchEngine {
    pub fn new() -> Self {
        Self {
            matrices: Vec::new(),
            max_matrices: 10,
        }
    }

    pub fn create_matrix(&mut self) -> &mut AchMatrix {
        if self.matrices.len() >= self.max_matrices {
            self.matrices.remove(0);
        }
        self.matrices.push(AchMatrix::new());
        self.matrices.last_mut().unwrap()
    }

    pub fn run_ach(
        &mut self,
        question: &str,
        hypotheses: &[(&str, &str)],
        evidence_items: &[&str],
    ) -> String {
        let matrix = self.create_matrix();

        // Step 1: Add all hypotheses (before evidence)
        for (label, desc) in hypotheses {
            matrix.add_hypothesis(label, desc);
        }

        // Step 2: Add all evidence
        for ev in evidence_items {
            matrix.add_evidence(ev);
        }

        // Step 3: Auto-assign diagnosticity based on keyword overlap
        for h in 0..hypotheses.len() {
            for e in 0..evidence_items.len() {
                let h_lower = hypotheses[h].1.to_lowercase();
                let h_words: Vec<&str> = h_lower.split_whitespace().collect();
                let e_text = evidence_items[e].to_lowercase();
                let matching = h_words.iter().filter(|w| e_text.contains(*w)).count();
                let ratio = if h_words.is_empty() {
                    0.0
                } else {
                    matching as f64 / h_words.len() as f64
                };

                let d = if ratio > 0.5 {
                    EvidenceDiagnosticity::Supports
                } else if ratio > 0.2 {
                    EvidenceDiagnosticity::Neutral
                } else {
                    EvidenceDiagnosticity::Contradicts
                };
                matrix.set_diagnosticity(h, e, d);
            }
        }

        // Step 4: Remove non-diagnostic evidence
        let removed = matrix.remove_non_diagnostic_evidence();

        // Step 5: Determine winner
        let winner = matrix.winning_hypothesis();
        let scores = matrix.score_hypotheses();

        let mut result = format!("ACH: {}\n", question);
        result.push_str(&format!("  Hypotheses: {}\n", hypotheses.len()));
        result.push_str(&format!(
            "  Evidence: {} (removed {} non-diagnostic)\n",
            evidence_items.len(),
            removed.len()
        ));

        for (idx, score) in &scores {
            let marker = if let Some(w) = winner {
                if *idx == w.id {
                    "← WINNER"
                } else {
                    ""
                }
            } else {
                ""
            };
            if let Some(h) = matrix.hypotheses.get(*idx) {
                result.push_str(&format!("  [{:.1}] {} {}\n", score, h.label, marker));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ach() {
        let mut engine = AchEngine::new();
        let result = engine.run_ach(
            "What caused the economic downturn?",
            &[
                (
                    "supply_chain",
                    "Global supply chain disruptions due to pandemic",
                ),
                ("monetary_policy", "Central bank interest rate increases"),
                ("consumer_spending", "Decrease in consumer spending"),
            ],
            &[
                "Factories in Asia were shut down for 6 months",
                "The central bank raised rates by 2.5%",
                "Retail sales dropped 15% year-over-year",
            ],
        );
        assert!(result.contains("WINNER"));
        assert!(
            result.contains("supply_chain")
                || result.contains("monetary_policy")
                || result.contains("consumer_spending")
        );
    }

    #[test]
    fn test_empty_matrix() {
        let matrix = AchMatrix::new();
        assert!(matrix.winning_hypothesis().is_none());
        assert!(matrix.score_hypotheses().is_empty());
    }

    #[test]
    fn test_diagnosticity_removal() {
        let mut matrix = AchMatrix::new();
        matrix.add_hypothesis("H1", "First hypothesis");
        matrix.add_hypothesis("H2", "Second hypothesis");
        matrix.add_evidence("Non-diagnostic evidence that doesn't differentiate");
        matrix.add_evidence("Diagnostic evidence supporting H1");
        matrix.set_diagnosticity(0, 0, EvidenceDiagnosticity::Neutral);
        matrix.set_diagnosticity(1, 0, EvidenceDiagnosticity::Neutral);
        matrix.set_diagnosticity(0, 1, EvidenceDiagnosticity::StronglySupports);
        matrix.set_diagnosticity(1, 1, EvidenceDiagnosticity::StronglyContradicts);

        let removed = matrix.remove_non_diagnostic_evidence();
        assert!(!removed.is_empty());
        assert!(matrix.evidence.len() < 2);
    }
}
