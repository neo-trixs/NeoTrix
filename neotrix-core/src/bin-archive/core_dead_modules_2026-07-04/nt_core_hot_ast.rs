use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotState {
    pub meta_representation: Vec<f64>,
    pub thought_about: String,
    pub confidence: f64,
    pub recursion_depth: usize,
}

impl HotState {
    pub fn new(thought_about: &str, confidence: f64, recursion_depth: usize) -> Self {
        let mut repr = Vec::with_capacity(64);
        let mut hasher = DefaultHasher::new();
        hasher.write(thought_about.as_bytes());
        let hash = hasher.finish();
        for i in 0..64 {
            let byte = ((hash >> ((i % 8) * 8)) & 0xFF) as u8;
            repr.push(byte as f64 / 255.0);
        }
        HotState {
            meta_representation: repr,
            thought_about: thought_about.to_string(),
            confidence: confidence.max(0.0).min(1.0),
            recursion_depth,
        }
    }

    pub fn label(&self) -> String {
        format!(
            "HOT(depth={}, about=\"{}\", conf={:.3})",
            self.recursion_depth,
            &self.thought_about.chars().take(40).collect::<String>(),
            self.confidence
        )
    }
}

pub fn hot_reflect(base_thought: &str) -> HotState {
    let commentary = format!("I am thinking about: {}", base_thought);
    HotState::new(&commentary, 0.7, 1)
}

pub fn recursive_hot(base_thought: &str, max_depth: usize) -> Vec<HotState> {
    let mut states = Vec::with_capacity(max_depth);
    let mut current = base_thought.to_string();
    for depth in 1..=max_depth {
        let reflected = format!("I notice that I am thinking: {}", current);
        let confidence = 0.8_f64.powi(depth as i32);
        let state = HotState::new(&reflected, confidence, depth);
        states.push(state);
        current = reflected;
    }
    states
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionVector {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub intent: String,
}

impl AttentionVector {
    pub fn new(source: &str, target: &str, weight: f64, intent: &str) -> Self {
        AttentionVector {
            source: source.to_string(),
            target: target.to_string(),
            weight: weight.max(0.0).min(1.0),
            intent: intent.to_string(),
        }
    }
}

pub struct AttentionSelfModel;

impl AttentionSelfModel {
    pub fn self_attend(state: &HotState) -> Vec<AttentionVector> {
        let n = state.meta_representation.len();
        let mut vectors = Vec::new();
        for i in 0..n.min(16) {
            for j in (i + 1)..n.min(16) {
                let w = (state.meta_representation[i]
                    - state.meta_representation[j])
                    .abs();
                if w > 0.05 {
                    vectors.push(AttentionVector::new(
                        &format!("dim_{}", i),
                        &format!("dim_{}", j),
                        w,
                        &state.thought_about,
                    ));
                }
            }
        }
        if vectors.is_empty() {
            vectors.push(AttentionVector::new(
                "self",
                "self",
                1.0,
                &state.thought_about,
            ));
        }
        vectors
    }

    pub fn meta_attention(attention_vectors: &[AttentionVector]) -> f64 {
        if attention_vectors.is_empty() {
            return 0.0;
        }
        let mean_weight: f64 =
            attention_vectors.iter().map(|v| v.weight).sum::<f64>() / attention_vectors.len() as f64;
        let variance: f64 = attention_vectors
            .iter()
            .map(|v| (v.weight - mean_weight).powi(2))
            .sum::<f64>()
            / attention_vectors.len() as f64;
        (mean_weight * 0.7 + (1.0 - variance.sqrt()) * 0.3).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedReport {
    pub hot_states: Vec<HotState>,
    pub attention_vectors: Vec<AttentionVector>,
    pub meta_attention_score: f64,
    pub integrated_confidence: f64,
    pub summary: String,
}

pub fn integrate_hot_ast(
    hot_states: Vec<HotState>,
    attention_vectors: Vec<AttentionVector>,
) -> IntegratedReport {
    let meta_attention_score = AttentionSelfModel::meta_attention(&attention_vectors);

    let avg_hot_confidence = if hot_states.is_empty() {
        0.0
    } else {
        hot_states.iter().map(|h| h.confidence).sum::<f64>() / hot_states.len() as f64
    };

    let integrated_confidence =
        (avg_hot_confidence * 0.6 + meta_attention_score * 0.4).max(0.0).min(1.0);

    let summary = format!(
        "Integrated consciousness report: {} higher-order thoughts, \
         {} attention vectors, meta-attention {:.3}, \
         integrated confidence {:.3}",
        hot_states.len(),
        attention_vectors.len(),
        meta_attention_score,
        integrated_confidence,
    );

    IntegratedReport {
        hot_states,
        attention_vectors,
        meta_attention_score,
        integrated_confidence,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reflect_basic() {
        let state = hot_reflect("E8 reasoning");
        assert!(state.recursion_depth == 1);
        assert!(state.thought_about.contains("E8 reasoning"));
        assert!(state.confidence > 0.0);
    }

    #[test]
    fn test_recursive_hot_depth() {
        let states = recursive_hot("base thought", 3);
        assert_eq!(states.len(), 3);
        for (i, s) in states.iter().enumerate() {
            assert_eq!(s.recursion_depth, i + 1);
            assert!(s.label().contains(&format!("depth={}", i + 1)));
        }
    }

    #[test]
    fn test_recursive_hot_confidence_decays() {
        let states = recursive_hot("test", 5);
        for w in states.windows(2) {
            assert!(w[0].confidence >= w[1].confidence);
        }
    }

    #[test]
    fn test_self_attend_produces_vectors() {
        let state = HotState::new("attention test", 0.9, 1);
        let vectors = AttentionSelfModel::self_attend(&state);
        assert!(!vectors.is_empty());
        for v in &vectors {
            assert!(v.weight >= 0.0 && v.weight <= 1.0);
            assert!(!v.source.is_empty());
            assert!(!v.target.is_empty());
        }
    }

    #[test]
    fn test_meta_attention_well_behaved() {
        let vectors = vec![
            AttentionVector::new("a", "b", 0.8, "test"),
            AttentionVector::new("c", "d", 0.6, "test"),
            AttentionVector::new("e", "f", 0.9, "test"),
        ];
        let score = AttentionSelfModel::meta_attention(&vectors);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_meta_attention_empty() {
        let score = AttentionSelfModel::meta_attention(&[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_integrate_hot_ast() {
        let hot = recursive_hot("consciousness", 2);
        let state = HotState::new("integration", 0.8, 1);
        let attn = AttentionSelfModel::self_attend(&state);
        let report = integrate_hot_ast(hot, attn);
        assert_eq!(report.hot_states.len(), 2);
        assert!(!report.attention_vectors.is_empty());
        assert!(report.integrated_confidence >= 0.0 && report.integrated_confidence <= 1.0);
        assert!(!report.summary.is_empty());
    }

    #[test]
    fn test_hot_state_label_format() {
        let state = HotState::new("test thought", 0.75, 2);
        let label = state.label();
        assert!(label.contains("HOT(depth=2"));
        assert!(label.contains("conf=0.750"));
    }

    #[test]
    fn test_attention_vector_new_clamps_weight() {
        let v = AttentionVector::new("src", "tgt", 1.5, "test");
        assert!(v.weight <= 1.0);
        let v2 = AttentionVector::new("src", "tgt", -0.5, "test");
        assert!(v2.weight >= 0.0);
    }
}
