#[derive(Debug, Clone)]
pub struct FusionGapEntry {
    pub name: String,
    pub paper_title: String,
    pub paper_url: String,
    pub theoretical_optimum: f64,
    pub implemented_level: f64,
    pub gap: f64,
    pub gap_description: String,
    pub impact: String,
    pub effort_to_close: String,
}

#[derive(Debug, Clone)]
pub struct FusionGapRegistry {
    pub entries: Vec<FusionGapEntry>,
}

impl FusionGapRegistry {
    pub fn register_defaults() -> Self {
        let entries = vec![
            FusionGapEntry {
                name: "ART tick scheduling".into(),
                paper_title: "Adaptive Resonance Theory (Grossberg 1976-2025)".into(),
                paper_url: "https://en.wikipedia.org/wiki/Adaptive_resonance_theory".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.35,
                gap: 0.65,
                gap_description: "Uses coherence proxy not real ART vigilance/matching; no incremental learning".into(),
                impact: "Suboptimal scheduling leads to ~2-3× more cycles on stable states; no stability-plasticity tradeoff".into(),
                effort_to_close: "High".into(),
            },
            FusionGapEntry {
                name: "RIIU variance-based weights".into(),
                paper_title: "Recurrent Independent Immune Unit (arXiv:2506.13825)".into(),
                paper_url: "https://arxiv.org/abs/2506.13825".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.30,
                gap: 0.70,
                gap_description: "Variance-based weight proxy not true differentiable Auto-Φ; no gradient ascent".into(),
                impact: "Fixed 0.3/0.3/0.4 weights cannot adapt to context; no end-to-end metagradient signal".into(),
                effort_to_close: "High".into(),
            },
            FusionGapEntry {
                name: "SCM multi-phase consolidation".into(),
                paper_title: "Sleep Consolidation Model (arXiv:2604.20943)".into(),
                paper_url: "https://arxiv.org/abs/2604.20943".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.40,
                gap: 0.60,
                gap_description: "Basic dedup + phase switch not full SCM NREM/REM; no noise reduction measurement".into(),
                impact: "Memory noise reduction is ~40% vs paper's 90.9%; no value-based forgetting".into(),
                effort_to_close: "Medium".into(),
            },
            FusionGapEntry {
                name: "Sutra rotation compiler ops".into(),
                paper_title: "Sutra: VSA-native programming language (arXiv:2605.20919)".into(),
                paper_url: "https://arxiv.org/abs/2605.20919".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.25,
                gap: 0.75,
                gap_description: "Rotation ops added but no full VSA-native language; no Kleene logic polynomial".into(),
                impact: "Ne compiler emits Rust not tensor ops; no gradient flow through Ne programs".into(),
                effort_to_close: "Very High".into(),
            },
            FusionGapEntry {
                name: "SEVerA/FGGM proof wrapper".into(),
                paper_title: "SEVerA: Verified Self-Evolution (arXiv:2603.25111)".into(),
                paper_url: "https://arxiv.org/abs/2603.25111".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.50,
                gap: 0.50,
                gap_description: "Wrapper works but no formal proof verification; no first-order logic theorem prover".into(),
                impact: "Cannot guarantee zero constraint violation; safety relies on heuristics not proofs".into(),
                effort_to_close: "Very High".into(),
            },
            FusionGapEntry {
                name: "MIRROR benchmark harness".into(),
                paper_title: "MIRROR: Metacognitive Benchmark (arXiv:2604.19809)".into(),
                paper_url: "https://arxiv.org/abs/2604.19809".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.30,
                gap: 0.70,
                gap_description: "Benchmark harness created but no full 8-experiment 4-level evaluation protocol".into(),
                impact: "Metacognitive accuracy measurement has low coverage; cannot detect scaffolding-blind spots".into(),
                effort_to_close: "Medium".into(),
            },
            FusionGapEntry {
                name: "Self-Model L0-L5 hierarchy".into(),
                paper_title: "Self-Model Hierarchy for Metacognitive AI (Self-Model Theory)".into(),
                paper_url: "https://opencode.ai".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.60,
                gap: 0.40,
                gap_description: "Hierarchy defined but not wired to real CI state; no predictive self-model".into(),
                impact: "Self-assessment is static snapshot; cannot anticipate future capability changes".into(),
                effort_to_close: "Medium".into(),
            },
            FusionGapEntry {
                name: "Meta-Evolution archive+utility".into(),
                paper_title: "Meta-Evolution: Self-Modifying Code Archive Loop (Evolutionary Computation)".into(),
                paper_url: "https://opencode.ai".into(),
                theoretical_optimum: 1.0,
                implemented_level: 0.20,
                gap: 0.80,
                gap_description: "Archive+utility implemented but no actual autonomous code modification; no sandbox testing".into(),
                impact: "Evolution loop is passive; no self-improvement without human trigger".into(),
                effort_to_close: "Very High".into(),
            },
        ];
        Self { entries }
    }

    pub fn worst_gap(&self) -> f64 {
        self.highest_gap_safe()
    }

    /// Returns the highest gap value, or 0.0 if registry is empty.
    pub fn highest_gap_safe(&self) -> f64 {
        self.entries.iter().map(|e| e.gap).fold(0.0, f64::max)
    }

    pub fn mean_gap(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.entries.iter().map(|e| e.gap).sum();
        sum / self.entries.len() as f64
    }

    pub fn close_gap(&mut self, name: &str, delta: f64) {
        for entry in self.entries.iter_mut() {
            if entry.name == name {
                let new_level =
                    (entry.implemented_level + delta).clamp(0.0, entry.theoretical_optimum);
                entry.gap = entry.theoretical_optimum - new_level;
                entry.implemented_level = new_level;
                return;
            }
        }
    }

    pub fn sorted_by_gap(&self) -> Vec<&FusionGapEntry> {
        let mut sorted: Vec<&FusionGapEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| {
            b.gap
                .partial_cmp(&a.gap)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    pub fn report(&self) -> String {
        let sorted = self.sorted_by_gap();
        let mut out = String::new();
        out.push_str("# Fusion Gap Registry Report\n\n");
        out.push_str(&format!("Mean gap: {:.2}\n\n", self.mean_gap()));
        out.push_str("| # | Name | Gap | Implemented | Optimum | Effort |\n");
        out.push_str("|---|------|-----|-------------|---------|--------|\n");
        for (i, entry) in sorted.iter().enumerate() {
            out.push_str(&format!(
                "| {} | {} | {:.2} | {:.0}% | {:.0}% | {} |\n",
                i + 1,
                entry.name,
                entry.gap,
                entry.implemented_level * 100.0,
                entry.theoretical_optimum * 100.0,
                entry.effort_to_close,
            ));
        }
        out.push_str("\n## Details\n\n");
        for entry in sorted.iter() {
            out.push_str(&format!("### {}\n", entry.name));
            out.push_str(&format!("- **Paper**: {}\n", entry.paper_title));
            out.push_str(&format!("- **URL**: {}\n", entry.paper_url));
            out.push_str(&format!(
                "- **Gap**: {:.2} (opt={:.0}%, impl={:.0}%)\n",
                entry.gap,
                entry.theoretical_optimum * 100.0,
                entry.implemented_level * 100.0
            ));
            out.push_str(&format!("- **Missing**: {}\n", entry.gap_description));
            out.push_str(&format!("- **Impact**: {}\n", entry.impact));
            out.push_str(&format!("- **Effort**: {}\n\n", entry.effort_to_close));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_defaults_creates_eight_entries() {
        let registry = FusionGapRegistry::register_defaults();
        assert_eq!(registry.entries.len(), 8);
    }

    #[test]
    fn test_worst_gap_returns_largest_gap() {
        let registry = FusionGapRegistry::register_defaults();
        let worst = registry.worst_gap();
        assert!((worst - 0.80).abs() < 1e-9);
    }

    #[test]
    fn test_mean_gap_computation() {
        let registry = FusionGapRegistry::register_defaults();
        let sum: f64 = registry.entries.iter().map(|e| e.gap).sum();
        let expected = sum / 8.0;
        assert!((registry.mean_gap() - expected).abs() < 1e-9);
    }

    #[test]
    fn test_close_gap_increases_implemented_level() {
        let mut registry = FusionGapRegistry::register_defaults();
        let before = registry.entries[0].implemented_level;
        let before_gap = registry.entries[0].gap;
        registry.close_gap("ART tick scheduling", 0.10);
        let after = registry.entries[0].implemented_level;
        let after_gap = registry.entries[0].gap;
        assert!((after - before - 0.10).abs() < 1e-9);
        assert!((before_gap - after_gap - 0.10).abs() < 1e-9);
    }

    #[test]
    fn test_close_gap_clamps_to_optimum() {
        let mut registry = FusionGapRegistry::register_defaults();
        registry.close_gap("ART tick scheduling", 2.0);
        assert!((registry.entries[0].implemented_level - 1.0).abs() < 1e-9);
        assert!((registry.entries[0].gap).abs() < 1e-9);
    }

    #[test]
    fn test_sorted_by_gap_ordering() {
        let registry = FusionGapRegistry::register_defaults();
        let sorted = registry.sorted_by_gap();
        for i in 1..sorted.len() {
            assert!(sorted[i - 1].gap >= sorted[i].gap);
        }
    }

    #[test]
    fn test_report_contains_all_entries() {
        let registry = FusionGapRegistry::register_defaults();
        let report = registry.report();
        for entry in &registry.entries {
            assert!(report.contains(&entry.name));
        }
        assert!(report.contains("Fusion Gap Registry Report"));
    }
}
