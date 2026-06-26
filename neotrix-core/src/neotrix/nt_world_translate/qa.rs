#![allow(dead_code)]
/// Triangle verification loop for translation quality.
/// Translates A->B->A and checks semantic consistency.
use crate::core::nt_core_hcube::cosine_sim_u8;

/// Result of a single round-trip verification
#[derive(Debug, Clone)]
pub struct RoundtripResult {
    pub source_text: String,
    pub forward_translation: String,
    pub backward_translation: String,
    pub similarity: f64,
    pub consistency: ConsistencyGrade,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsistencyGrade {
    Exact,
    High,
    Medium,
    Low,
    Failed,
}

/// Triangle verifier — A->B->A round-trip
pub struct RoundtripVerifier {
    threshold_high: f64,
    threshold_medium: f64,
}

impl RoundtripVerifier {
    pub fn new() -> Self {
        Self {
            threshold_high: 0.85,
            threshold_medium: 0.65,
        }
    }

    pub fn verify(
        &self,
        source: &str,
        forward: &str,
        backward: &str,
        embed_fn: &dyn Fn(&str) -> Vec<u8>,
    ) -> RoundtripResult {
        let src_vec = embed_fn(source);
        let bwd_vec = embed_fn(backward);
        let sim = cosine_sim_u8(&src_vec, &bwd_vec);
        let grade = if (source == backward) || sim > 0.99 {
            ConsistencyGrade::Exact
        } else if sim >= self.threshold_high {
            ConsistencyGrade::High
        } else if sim >= self.threshold_medium {
            ConsistencyGrade::Medium
        } else if sim > 0.3 {
            ConsistencyGrade::Low
        } else {
            ConsistencyGrade::Failed
        };
        RoundtripResult {
            source_text: source.to_string(),
            forward_translation: forward.to_string(),
            backward_translation: backward.to_string(),
            similarity: sim,
            consistency: grade,
        }
    }
}

impl Default for RoundtripVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic consistency checker — compares meaning vectors
pub struct SemanticConsistency {
    verifier: RoundtripVerifier,
}

impl SemanticConsistency {
    pub fn new() -> Self {
        Self {
            verifier: RoundtripVerifier::new(),
        }
    }

    pub fn check(
        &self,
        source: &str,
        translation: &str,
        verify: &dyn Fn(&str) -> Vec<u8>,
    ) -> RoundtripResult {
        let src_vec = verify(source);
        let tgt_vec = verify(translation);
        let sim = cosine_sim_u8(&src_vec, &tgt_vec);
        RoundtripResult {
            source_text: source.to_string(),
            forward_translation: translation.to_string(),
            backward_translation: String::new(),
            similarity: sim,
            consistency: if sim > 0.85 {
                ConsistencyGrade::High
            } else if sim > 0.65 {
                ConsistencyGrade::Medium
            } else {
                ConsistencyGrade::Low
            },
        }
    }
}

impl Default for SemanticConsistency {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge case library for red-teaming translation quality
pub struct RedTeamingEdgeCases {
    cases: Vec<(String, String, String)>, // (source, expected_forward, source_lang)
}

impl RedTeamingEdgeCases {
    pub fn new() -> Self {
        Self {
            cases: vec![
                (
                    "Hello world.".to_string(),
                    "Hola mundo.".to_string(),
                    "en->es".to_string(),
                ),
                (
                    "I have 5 apples.".to_string(),
                    "Tengo 5 manzanas.".to_string(),
                    "en->es".to_string(),
                ),
                (
                    "Temperature is -5°C.".to_string(),
                    "La temperatura es -5°C.".to_string(),
                    "en->es".to_string(),
                ),
                ("".to_string(), "".to_string(), "en->es".to_string()),
                (
                    "A\nB\nC".to_string(),
                    "A\nB\nC".to_string(),
                    "en->es".to_string(),
                ),
            ],
        }
    }

    pub fn cases(&self) -> &[(String, String, String)] {
        &self.cases
    }
}

/// Main QA engine: runs triangle verification + semantic check + edge cases
pub struct TranslationQaEngine {
    roundtrip: RoundtripVerifier,
    semantic: SemanticConsistency,
    red_team: RedTeamingEdgeCases,
}

impl TranslationQaEngine {
    pub fn new() -> Self {
        Self {
            roundtrip: RoundtripVerifier::new(),
            semantic: SemanticConsistency::new(),
            red_team: RedTeamingEdgeCases::new(),
        }
    }

    /// Full QA pass
    pub fn evaluate(
        &self,
        source: &str,
        forward: &str,
        backward: &str,
        embed_fn: &dyn Fn(&str) -> Vec<u8>,
    ) -> QaReport {
        let roundtrip_result = self.roundtrip.verify(source, forward, backward, embed_fn);
        let semantic_result = self.semantic.check(source, forward, embed_fn);
        let pass = roundtrip_result.consistency != ConsistencyGrade::Failed
            && semantic_result.consistency != ConsistencyGrade::Low;
        QaReport {
            roundtrip: roundtrip_result,
            semantic: semantic_result,
            pass,
        }
    }

    /// Run edge case tests against a translator
    pub fn edge_case_report(
        &self,
        translator: &dyn Fn(&str, &str, &str) -> String,
    ) -> EdgeCaseReport {
        let mut results = Vec::new();
        for (src, _expected, _lang) in self.red_team.cases() {
            let forward = translator(src, "en", "es");
            let backward = translator(&forward, "es", "en");
            let embed: fn(&str) -> Vec<u8> = |s: &str| {
                let mut v = vec![0u8; 64];
                for (i, b) in s.bytes().enumerate() {
                    if i < 64 {
                        v[i] = b;
                    }
                }
                v
            };
            let result = self.roundtrip.verify(src, &forward, &backward, &embed);
            results.push(result);
        }
        let total = results.len();
        let passed = results
            .iter()
            .filter(|r| r.consistency != ConsistencyGrade::Failed)
            .count();
        EdgeCaseReport {
            results,
            passed,
            total,
        }
    }
}

impl Default for TranslationQaEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// QA report
#[derive(Debug, Clone)]
pub struct QaReport {
    pub roundtrip: RoundtripResult,
    pub semantic: RoundtripResult,
    pub pass: bool,
}

/// Edge case test report
#[derive(Debug, Clone)]
pub struct EdgeCaseReport {
    pub results: Vec<RoundtripResult>,
    pub passed: usize,
    pub total: usize,
}
