#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionOutput {
    pub conclusion: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
    pub contradictions: Vec<String>,
    pub reflection_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionHead {
    pub id: usize,
    pub perspective: String,
    pub confidence: f64,
    pub biases: Vec<String>,
    pub forward_weight: f64,
    pub backward_weight: f64,
}

impl ReflectionHead {
    pub fn new(id: usize, perspective: String) -> Self {
        Self {
            id,
            perspective,
            confidence: 0.8,
            biases: Vec::new(),
            forward_weight: 0.6,
            backward_weight: 0.4,
        }
    }

    pub fn forward_reflect(&self, observation: &str) -> ReflectionOutput {
        let conclusion = format!("{} analysis of: {}", self.perspective, observation);
        let words: Vec<&str> = observation.split_whitespace().collect();
        let mut evidence = Vec::new();
        let mut contradictions = Vec::new();

        for word in &words {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.is_empty() {
                continue;
            }
            if self.perspective.to_lowercase().contains(&clean.to_lowercase()) {
                evidence.push(format!("word '{}' aligns with {}", clean, self.perspective));
            } else {
                for bias in &self.biases {
                    if clean.to_lowercase().contains(&bias.to_lowercase()) {
                        evidence.push(format!("word '{}' matches bias '{}'", clean, bias));
                    } else {
                        contradictions.push(format!("word '{}' does not match bias '{}'", clean, bias));
                    }
                }
            }
        }

        if evidence.is_empty() {
            evidence.push("observation recorded for analysis".to_string());
        }

        let perspective_words: Vec<&str> = self.perspective.split_whitespace().collect();
        let match_count = words
            .iter()
            .filter(|w| {
                let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
                perspective_words
                    .iter()
                    .any(|pw| pw.to_lowercase() == clean.to_lowercase())
            })
            .count();
        let total = words.len().max(1);
        let perspective_match = match_count as f64 / total as f64;
        let conf = (self.confidence * (0.5 + 0.5 * perspective_match))
            .max(0.0)
            .min(1.0);

        ReflectionOutput {
            conclusion,
            confidence: conf,
            supporting_evidence: evidence,
            contradictions,
            reflection_depth: words.len() as u32,
        }
    }

    pub fn backward_reflect(&self, hypothesis: &str) -> ReflectionOutput {
        let conclusion = format!("{} backward analysis of: {}", self.perspective, hypothesis);
        let words: Vec<&str> = hypothesis.split_whitespace().collect();
        let mut evidence = Vec::new();
        let mut contradictions = Vec::new();

        for word in &words {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.is_empty() {
                continue;
            }
            if self.perspective.to_lowercase().contains(&clean.to_lowercase()) {
                evidence.push(format!(
                    "hypothesis term '{}' aligns with {}",
                    clean, self.perspective
                ));
            } else {
                for bias in &self.biases {
                    if clean.to_lowercase().contains(&bias.to_lowercase()) {
                        evidence.push(format!(
                            "hypothesis term '{}' matches bias '{}'",
                            clean, bias
                        ));
                    } else {
                        contradictions.push(format!(
                            "hypothesis term '{}' contradicts bias '{}'",
                            clean, bias
                        ));
                    }
                }
            }
        }

        if evidence.is_empty() {
            evidence.push("hypothesis reviewed for consistency".to_string());
        }

        let conf = (self.confidence * self.backward_weight).max(0.0).min(1.0);

        ReflectionOutput {
            conclusion,
            confidence: conf,
            supporting_evidence: evidence,
            contradictions,
            reflection_depth: (words.len() / 2).max(1) as u32,
        }
    }

    pub fn update_confidence(&mut self, new_confidence: f64) {
        self.confidence = new_confidence.max(0.0).min(1.0);
    }
}

impl Default for ReflectionHead {
    fn default() -> Self {
        Self::new(0, "default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_creation() {
        let head = ReflectionHead::new(1, "analytical".to_string());
        assert_eq!(head.id, 1);
        assert_eq!(head.perspective, "analytical");
        assert!((head.confidence - 0.8).abs() < 1e-6);
        assert!((head.forward_weight - 0.6).abs() < 1e-6);
        assert!((head.backward_weight - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_forward_reflect_basic() {
        let head = ReflectionHead::new(0, "test".to_string());
        let out = head.forward_reflect("some observation");
        assert!(out.conclusion.contains("test analysis of:"));
        assert!(out.confidence >= 0.0 && out.confidence <= 1.0);
        assert!(out.reflection_depth > 0);
    }

    #[test]
    fn test_forward_reflect_with_biases() {
        let mut head = ReflectionHead::new(0, "security".to_string());
        head.biases.push("alert".to_string());
        let out = head.forward_reflect("system alert detected");
        assert!(!out.supporting_evidence.is_empty());
        assert!(out.supporting_evidence.iter().any(|e| e.contains("alert")));
    }

    #[test]
    fn test_backward_reflect_basic() {
        let head = ReflectionHead::new(2, "diagnostic".to_string());
        let out = head.backward_reflect("hypothesis about root cause");
        assert!(!out.conclusion.is_empty());
        assert!(out.confidence >= 0.0 && out.confidence <= 1.0);
    }

    #[test]
    fn test_update_confidence() {
        let mut head = ReflectionHead::new(0, "adaptive".to_string());
        head.update_confidence(0.85);
        assert!((head.confidence - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_clamping() {
        let mut head = ReflectionHead::new(0, "clamp".to_string());
        head.update_confidence(1.5);
        assert!((head.confidence - 1.0).abs() < 1e-6);
        head.update_confidence(-0.5);
        assert!((head.confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_forward_backward_weights() {
        let head = ReflectionHead::new(0, "weighted".to_string());
        let fwd = head.forward_reflect("test");
        let bwd = head.backward_reflect("test");
        assert!(fwd.confidence >= bwd.confidence);
    }

    #[test]
    fn test_default_impl() {
        let head = ReflectionHead::default();
        assert_eq!(head.id, 0);
        assert_eq!(head.perspective, "default");
    }
}
