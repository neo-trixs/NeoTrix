//! Reasoning protector for anti-distillation
//!
//! Protects chain-of-thought reasoning traces from extraction:
//! - Trace summarization
//! - Signature encryption
//! - Context preservation
//! - Decoy injection

use super::{DetectionSignal, ThreatLevel};

pub struct ReasoningProtector {
    _decoy_patterns: Vec<String>,
}

impl ReasoningProtector {
    pub fn new() -> Self {
        Self {
            decoy_patterns: vec![
                "Let me think about this step by step...".to_string(),
                "First, I need to consider the main factors...".to_string(),
                "The key insight here is...".to_string(),
                "Breaking this down into components...".to_string(),
            ],
        }
    }

    /// Protect reasoning traces before sending response
    pub fn protect(&self, reasoning: &str, signature: &str) -> String {
        // Summarize reasoning (don't send full trace)
        let summarized = self.summarize_reasoning(reasoning);

        // Add decoy content
        let with_decoy = self.inject_decoy(&summarized);

        // Format with signature reference (not full trace)
        format!(
            "[Reasoning signature: {}]\n{}",
            signature, with_decoy
        )
    }

    /// Summarize reasoning to prevent full extraction
    fn summarize_reasoning(&self, reasoning: &str) -> String {
        let lines: Vec<&str> = reasoning.lines().collect();
        let line_count = lines.len();

        if line_count <= 3 {
            // Short reasoning - return as is
            reasoning.to_string()
        } else {
            // Long reasoning - summarize key points
            let summary = lines.iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "{}\n\n[... {} more lines of reasoning omitted for security ...]",
                summary, line_count - 3
            )
        }
    }

    /// Inject decoy reasoning patterns
    fn inject_decoy(&self, reasoning: &str) -> String {
        let mut result = reasoning.to_string();

        // Add decoy at end
        result.push_str("\n\n[Additional reasoning considerations are omitted for brevity]");

        result
    }

    /// Check if a response contains reasoning trace extraction attempts
    pub fn detect_extraction_attempt(&self, response: &str) -> Option<DetectionSignal> {
        let extraction_patterns = [
            "thinking",
            "reasoning trace",
            "chain of thought",
            "step by step reasoning",
            "internal monologue",
        ];

        let response_lower = response.to_lowercase();
        for pattern in &extraction_patterns {
            if response_lower.contains(pattern) {
                return Some(DetectionSignal {
                    signal_type: "reasoning_extraction_attempt".to_string(),
                    confidence: 0.7,
                    threat_level: ThreatLevel::Medium,
                    details: format!("Possible reasoning extraction pattern: {}", pattern),
                });
            }
        }

        None
    }

    /// Encrypt reasoning signature (simplified - use actual crypto in production)
    pub fn encrypt_signature(&self, signature: &str) -> String {
        // Placeholder - implement proper encryption
        format!("ENC:{}", signature)
    }

    /// Decrypt reasoning signature
    pub fn decrypt_signature(&self, encrypted: &str) -> Option<String> {
        encrypted.strip_prefix("ENC:").map(|s| s.to_string())
    }
}
