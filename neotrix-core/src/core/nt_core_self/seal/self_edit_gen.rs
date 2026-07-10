#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditType {
    Refactor,
    Optimize,
    Fix,
    Document,
    Feature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEdit {
    pub id: usize,
    pub edit_type: EditType,
    pub target_location: String,
    pub original_text: String,
    pub proposed_text: String,
    pub confidence: f64,
    pub expected_improvement: f64,
}

pub struct SelfEditGen {
    pub vocab_size: usize,
    pub hidden_dim: usize,
    pub temperature: f64,
    pub max_edits_per_step: usize,
    next_id: usize,
}

impl SelfEditGen {
    pub fn new(vocab_size: usize, hidden_dim: usize, temperature: f64) -> Self {
        Self {
            vocab_size,
            hidden_dim,
            temperature: temperature.max(0.01).min(2.0),
            max_edits_per_step: 5,
            next_id: 0,
        }
    }

    pub fn generate_edits(&self, code_context: &str, target: &str) -> Vec<SelfEdit> {
        let target_lower = target.to_lowercase();
        let context_lower = code_context.to_lowercase();

        let candidate_types: Vec<EditType> = if context_lower.contains("function")
            || context_lower.contains("fn")
        {
            vec![EditType::Refactor, EditType::Fix]
        } else if context_lower.contains("optimize") || context_lower.contains("perf") {
            vec![EditType::Optimize]
        } else if context_lower.contains("doc") || context_lower.contains("comment") {
            vec![EditType::Document]
        } else if target_lower.contains("feature") || target_lower.contains("new") {
            vec![EditType::Feature]
        } else {
            vec![
                EditType::Refactor,
                EditType::Optimize,
                EditType::Fix,
                EditType::Document,
                EditType::Feature,
            ]
        };

        let noise_scale = self.temperature * 0.2;
        let mut edits = Vec::new();
        for (i, et) in candidate_types.iter().enumerate().take(self.max_edits_per_step) {
            let noise: f64 = ((i as f64 + 1.0) * 1.13).sin() * noise_scale;
            let confidence = ((1.0 - self.temperature * 0.15) + noise).max(0.1).min(1.0);
            let improvement = (0.3 + (i as f64 * 0.27).cos() * 0.2).max(0.0).min(1.0);
            let location = format!("{}:L{}", target, i * 8 + 1);
            let original = code_context
                .lines()
                .nth(i % code_context.lines().count().max(1))
                .unwrap_or("")
                .to_string();
            let proposed = format!("{}_v{}", target, i);
            edits.push(SelfEdit {
                id: self.next_id.wrapping_add(i),
                edit_type: *et,
                target_location: location,
                original_text: original,
                proposed_text: proposed,
                confidence,
                expected_improvement: improvement,
            });
        }
        edits
    }

    pub fn score_edit(&self, edit: &SelfEdit) -> f64 {
        let type_base = match edit.edit_type {
            EditType::Refactor => 0.9,
            EditType::Fix => 0.8,
            EditType::Optimize => 0.7,
            EditType::Feature => 0.5,
            EditType::Document => 0.3,
        };
        (type_base * 0.5 + edit.confidence * 0.3 + edit.expected_improvement * 0.2).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_gen() {
        let gen = SelfEditGen::new(1000, 64, 0.8);
        assert_eq!(gen.vocab_size, 1000);
        assert_eq!(gen.hidden_dim, 64);
        assert!((gen.temperature - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_temperature_clamped_high() {
        let gen = SelfEditGen::new(100, 32, 5.0);
        assert!((gen.temperature - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_temperature_clamped_low() {
        let gen = SelfEditGen::new(100, 32, -1.0);
        assert!((gen.temperature - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_generate_edits_returns_non_empty() {
        let gen = SelfEditGen::new(100, 32, 0.7);
        let edits = gen.generate_edits("fn foo() {}", "target_func");
        assert!(!edits.is_empty());
        assert!(edits.len() <= gen.max_edits_per_step);
    }

    #[test]
    fn test_generate_edits_unique_ids_within_call() {
        let gen = SelfEditGen::new(100, 32, 0.7);
        let edits = gen.generate_edits("code", "t");
        let mut ids: Vec<usize> = edits.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), edits.len());
    }

    #[test]
    fn test_target_with_fn_yields_refactor_or_fix() {
        let gen = SelfEditGen::new(100, 32, 0.5);
        let edits = gen.generate_edits("fn compute() {}", "func");
        for e in &edits {
            assert!(matches!(e.edit_type, EditType::Refactor | EditType::Fix));
        }
    }

    #[test]
    fn test_target_with_function_yields_refactor_or_fix() {
        let gen = SelfEditGen::new(100, 32, 0.5);
        let edits = gen.generate_edits("function process()", "func");
        for e in &edits {
            assert!(matches!(e.edit_type, EditType::Refactor | EditType::Fix));
        }
    }

    #[test]
    fn test_target_with_optimize_yields_optimize() {
        let gen = SelfEditGen::new(100, 32, 0.5);
        let edits = gen.generate_edits("optimize loop for perf", "perf");
        for e in &edits {
            assert_eq!(e.edit_type, EditType::Optimize);
        }
    }

    #[test]
    fn test_target_with_doc_yields_document() {
        let gen = SelfEditGen::new(100, 32, 0.5);
        let edits = gen.generate_edits("doc string comment", "docs");
        for e in &edits {
            assert_eq!(e.edit_type, EditType::Document);
        }
    }

    #[test]
    fn test_temperature_affects_confidence() {
        let gen_hot = SelfEditGen::new(100, 32, 1.8);
        let gen_cold = SelfEditGen::new(100, 32, 0.1);
        let hot_edits = gen_hot.generate_edits("fn a() {}", "fn");
        let cold_edits = gen_cold.generate_edits("fn a() {}", "fn");
        let hot_conf: f64 = hot_edits.iter().map(|e| e.confidence).sum();
        let cold_conf: f64 = cold_edits.iter().map(|e| e.confidence).sum();
        assert!(cold_conf >= hot_conf || (cold_conf - hot_conf).abs() < 0.2);
    }

    #[test]
    fn test_max_edits_respected() {
        let mut gen = SelfEditGen::new(100, 32, 0.7);
        gen.max_edits_per_step = 3;
        let edits = gen.generate_edits("code", "t");
        assert_eq!(edits.len(), 3);
    }

    #[test]
    fn test_score_edit_type_priority() {
        let gen = SelfEditGen::new(100, 32, 0.8);
        let base = |t: EditType| -> SelfEdit {
            SelfEdit {
                id: 0,
                edit_type: t,
                target_location: "x".into(),
                original_text: "old".into(),
                proposed_text: "new".into(),
                confidence: 0.5,
                expected_improvement: 0.5,
            }
        };
        let refactor = gen.score_edit(&base(EditType::Refactor));
        let fix = gen.score_edit(&base(EditType::Fix));
        let optimize = gen.score_edit(&base(EditType::Optimize));
        let feature = gen.score_edit(&base(EditType::Feature));
        let document = gen.score_edit(&base(EditType::Document));
        assert!(refactor >= fix);
        assert!(fix >= optimize);
        assert!(optimize >= feature);
        assert!(feature >= document);
    }

    #[test]
    fn test_score_edit_bounds() {
        let gen = SelfEditGen::new(100, 32, 0.8);
        let edit = SelfEdit {
            id: 0,
            edit_type: EditType::Optimize,
            target_location: "x".into(),
            original_text: "a".into(),
            proposed_text: "b".into(),
            confidence: 0.5,
            expected_improvement: 0.5,
        };
        let score = gen.score_edit(&edit);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_generate_edits_no_context_still_works() {
        let gen = SelfEditGen::new(100, 32, 0.7);
        let edits = gen.generate_edits("", "test");
        assert!(!edits.is_empty());
    }
}
