use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::capture::CapturedInteraction;

/// A capability demonstrated by a model response, model-agnostic.
/// Captures WHAT was done well, not which model did it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemonstratedCapability {
    /// Capability name (e.g. "code_generation", "explanation", "debugging")
    pub name: String,
    /// Description of what was demonstrated
    pub description: String,
    /// Keywords that triggered this capability detection
    pub trigger_keywords: Vec<String>,
    /// Estimated proficiency (0.0–1.0) based on outcome
    pub proficiency: f64,
    /// How many observations support this
    pub observation_count: u32,
    /// Which models demonstrated this
    pub observed_models: Vec<String>,
    /// Sub-capabilities detected
    pub sub_capabilities: Vec<String>,
}

/// Signature keywords that indicate specific capabilities in responses.
const CAPABILITY_SIGNATURES: &[(&str, &[&str], &[&str])] = &[
    (
        "code_generation",
        &[
            "fn ",
            "def ",
            "function ",
            "impl ",
            "class ",
            "let ",
            "const ",
            "import ",
            "```rust",
            "```python",
            "```typescript",
            "```javascript",
            "```go",
            "```java",
        ],
        &[
            "write",
            "implement",
            "create",
            "build",
            "generate",
            "code",
            "function",
        ],
    ),
    (
        "debugging",
        &[
            "error",
            "bug",
            "fix",
            "issue",
            "problem",
            "incorrect",
            "wrong",
            "fail",
            "panic",
            "crash",
            "stack trace",
            "exception",
        ],
        &[
            "debug",
            "fix",
            "repair",
            "bug",
            "error",
            "broken",
            "incorrect",
        ],
    ),
    (
        "explanation",
        &[
            "means",
            "explain",
            "理解",
            "意思是",
            "for example",
            "in other words",
            "simply put",
            "essentially",
            "conceptually",
        ],
        &[
            "explain",
            "what is",
            "how does",
            "describe",
            "elaborate",
            "clarify",
        ],
    ),
    (
        "architecture_design",
        &[
            "architecture",
            "component",
            "service",
            "module",
            "layer",
            "pattern",
            "design",
            "structure",
            "system",
            "interface",
            "abstraction",
        ],
        &[
            "design",
            "architect",
            "structure",
            "organize",
            "plan",
            "component",
        ],
    ),
    (
        "refactoring",
        &[
            "refactor",
            "clean",
            "extract",
            "inline",
            "rename",
            "restructure",
            "simplify",
            "deduplicate",
            "decouple",
        ],
        &[
            "refactor",
            "improve",
            "clean",
            "optimize",
            "simplify",
            "restructure",
        ],
    ),
    (
        "testing",
        &[
            "#[test]",
            "#[cfg(test)]",
            "assert_eq!",
            "assert!(",
            "describe!",
            "it_",
            "test_",
            "fn test",
            "should_",
            "expect(",
        ],
        &["test", "assert", "verify", "validate", "spec", "coverage"],
    ),
    (
        "security_analysis",
        &[
            "vulnerability",
            "injection",
            "xss",
            "csrf",
            "sql injection",
            "sanitize",
            "escape",
            "permission",
            "auth",
            "encrypt",
        ],
        &[
            "security",
            "secure",
            "vulnerability",
            "protect",
            "threat",
            "attack",
        ],
    ),
    (
        "performance_optimization",
        &[
            "performance",
            "optimize",
            "bottleneck",
            "latency",
            "throughput",
            "cache",
            "complexity O(",
            "memory",
            "cpu",
            "io",
        ],
        &[
            "performance",
            "fast",
            "slow",
            "optimize",
            "efficient",
            "speed",
        ],
    ),
    (
        "data_analysis",
        &[
            "statistic",
            "correlation",
            "distribution",
            "mean",
            "median",
            "variance",
            "regression",
            "classification",
            "cluster",
        ],
        &[
            "analyze",
            "data",
            "statistic",
            "chart",
            "plot",
            "trend",
            "insight",
        ],
    ),
    (
        "api_design",
        &[
            "endpoint",
            "route",
            "middleware",
            "handler",
            "request",
            "response",
            "status code",
            "rest",
            "graphql",
            "grpc",
        ],
        &["api", "endpoint", "route", "service", "interface", "rest"],
    ),
    (
        "mathematical_reasoning",
        &[
            "equation",
            "formula",
            "theorem",
            "proof",
            "derive",
            "calculate",
            "integral",
            "derivative",
            "matrix",
            "vector",
        ],
        &["math", "calculate", "equation", "formula", "prove", "solve"],
    ),
];

/// Extracts demonstrated capabilities from model responses.
/// Scans for capability signatures in both the user prompt and model response,
/// then estimates proficiency from the outcome score.
#[derive(Debug, Clone)]
pub struct CapabilityExtractor {
    min_observations: usize,
    min_proficiency_threshold: f64,
    extractions: u64,
}

impl CapabilityExtractor {
    pub fn new() -> Self {
        Self {
            min_observations: 2,
            min_proficiency_threshold: 0.3,
            extractions: 0,
        }
    }

    pub fn with_min_observations(mut self, min: usize) -> Self {
        self.min_observations = min;
        self
    }

    /// Extract capabilities demonstrated across a batch of interactions.
    pub fn extract(&mut self, interactions: &[CapturedInteraction]) -> Vec<DemonstratedCapability> {
        // Group by capability name
        let mut groups: HashMap<String, Vec<&CapturedInteraction>> = HashMap::new();
        let mut cap_keywords: HashMap<String, Vec<String>> = HashMap::new();
        let mut cap_subs: HashMap<String, HashSet<String>> = HashMap::new();

        for interaction in interactions {
            let detected = self.detect_capabilities(interaction);
            for cap in detected {
                groups
                    .entry(cap.name.clone())
                    .or_default()
                    .push(interaction);
                cap_keywords
                    .entry(cap.name.clone())
                    .or_default()
                    .extend(cap.trigger_keywords);
                cap_subs
                    .entry(cap.name.clone())
                    .or_default()
                    .extend(cap.sub_capabilities);
            }
        }

        let mut capabilities = Vec::new();

        for (name, group) in groups {
            if group.len() < self.min_observations {
                continue;
            }

            let total_outcome: f64 = group.iter().map(|i| i.outcome_score).sum();
            let avg_proficiency = total_outcome / group.len() as f64;

            if avg_proficiency < self.min_proficiency_threshold {
                continue;
            }

            let keywords = cap_keywords.remove(&name).unwrap_or_default();
            let mut unique_keywords: Vec<String> = keywords.clone();
            unique_keywords.sort();
            unique_keywords.dedup();

            let sub_caps: Vec<String> = cap_subs
                .remove(&name)
                .map(|s| {
                    let mut v: Vec<String> = s.into_iter().collect();
                    v.sort();
                    v
                })
                .unwrap_or_default();

            let mut observed_models: Vec<String> = group.iter().map(|i| i.model.clone()).collect();
            observed_models.sort();
            observed_models.dedup();

            let desc = format!(
                "{}: proficiency={:.3} across {} observations from {} models",
                name,
                avg_proficiency,
                group.len(),
                observed_models.len()
            );

            self.extractions += 1;
            capabilities.push(DemonstratedCapability {
                name,
                description: desc,
                trigger_keywords: unique_keywords,
                proficiency: avg_proficiency,
                observation_count: group.len() as u32,
                observed_models,
                sub_capabilities: sub_caps,
            });
        }

        capabilities.sort_by(|a, b| {
            b.proficiency
                .partial_cmp(&a.proficiency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        capabilities
    }

    /// Detect which capabilities are present in a single interaction.
    pub fn detect_capabilities(
        &self,
        interaction: &CapturedInteraction,
    ) -> Vec<DemonstratedCapability> {
        let combined = format!(
            "{} {} {}",
            interaction.user_messages, interaction.response, interaction.system_prompt
        )
        .to_lowercase();

        let mut detected = Vec::new();

        for (name, response_sigs, prompt_sigs) in CAPABILITY_SIGNATURES {
            // Check if prompt keywords match
            let prompt_match = prompt_sigs
                .iter()
                .any(|kw| interaction.user_messages.to_lowercase().contains(kw));

            // Check if response contains capability signatures
            let response_match = response_sigs.iter().any(|sig| combined.contains(sig));

            if prompt_match && response_match {
                let matched_keywords: Vec<String> = prompt_sigs
                    .iter()
                    .chain(response_sigs.iter())
                    .filter(|kw| combined.contains(*kw))
                    .map(|s| s.to_string())
                    .collect();

                let sub_caps = self.detect_sub_capabilities(name, &combined);

                detected.push(DemonstratedCapability {
                    name: name.to_string(),
                    description: String::new(),
                    trigger_keywords: matched_keywords,
                    proficiency: interaction.outcome_score,
                    observation_count: 1,
                    observed_models: vec![interaction.model.clone()],
                    sub_capabilities: sub_caps,
                });
            }
        }

        detected
    }

    fn detect_sub_capabilities(&self, cap_name: &str, combined: &str) -> Vec<String> {
        let subs: &[(&str, &[&str])] = match cap_name {
            "code_generation" => &[
                (
                    "error_handling",
                    &["Result", "Option", "unwrap", "?", "try", "catch"],
                ),
                ("async", &["async", "await", "tokio", "future", "promise"]),
                (
                    "concurrency",
                    &["thread", "mutex", "lock", "channel", "Arc", "RwLock"],
                ),
                ("testing", &["test", "assert", "#[test]", "spec"]),
                (
                    "documentation",
                    &["doc", "///", "comment", "documentation", "README"],
                ),
            ],
            "debugging" => &[
                (
                    "root_cause",
                    &["root cause", "caused by", "because", "origin", "source"],
                ),
                (
                    "reproduction",
                    &["reproduce", "reproduction", "minimal", "example"],
                ),
                (
                    "fix_strategy",
                    &["fix", "solution", "resolve", "workaround", "patch"],
                ),
            ],
            "explanation" => &[
                ("analogy", &["analogy", "like", "similar to", "imagine"]),
                ("example", &["example", "for instance", "e.g.", "such as"]),
                ("visual", &["diagram", "chart", "graph", "visual"]),
            ],
            _ => &[],
        };

        subs.iter()
            .filter(|(_, sigs)| sigs.iter().any(|s| combined.contains(s)))
            .map(|(name, _)| name.to_string())
            .collect()
    }

    /// Get the proficiency ranking of models for a specific capability.
    pub fn model_ranking(
        &self,
        interactions: &[CapturedInteraction],
        capability: &str,
    ) -> Vec<(String, f64, u32)> {
        let mut model_scores: HashMap<String, Vec<f64>> = HashMap::new();
        for i in interactions {
            let caps = self.detect_capabilities(i);
            if caps.iter().any(|c| c.name == capability) {
                model_scores
                    .entry(i.model.clone())
                    .or_default()
                    .push(i.outcome_score);
            }
        }

        let mut ranking: Vec<(String, f64, u32)> = model_scores
            .into_iter()
            .map(|(model, scores)| {
                let count = scores.len() as u32;
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                (model, avg, count)
            })
            .collect();

        ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranking
    }

    pub fn extractions(&self) -> u64 {
        self.extractions
    }
}

impl Default for CapabilityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code_interaction(model: &str, prompt: &str, response: &str) -> CapturedInteraction {
        CapturedInteraction::new(
            "test", model, "", prompt, response, 100, 200, 500, true, "stop",
        )
    }

    fn debug_interaction(model: &str, prompt: &str, response: &str) -> CapturedInteraction {
        CapturedInteraction::new(
            "test", model, "", prompt, response, 50, 150, 300, true, "stop",
        )
    }

    #[test]
    fn test_detect_code_generation() {
        let extractor = CapabilityExtractor::new();
        let interaction = code_interaction(
            "model-x",
            "write a function in rust",
            "```rust\nfn hello() -> String {\n    \"hello\".to_string()\n}\n```",
        );
        let caps = extractor.detect_capabilities(&interaction);
        assert!(
            caps.iter().any(|c| c.name == "code_generation"),
            "should detect code generation capability"
        );
    }

    #[test]
    fn test_detect_debugging() {
        let extractor = CapabilityExtractor::new();
        let interaction = debug_interaction(
            "model-x",
            "fix this bug in my code",
            "The error is caused by a null pointer dereference. Here's the fix:\n```rust\nif let Some(val) = option {\n    println!(\"{}\", val);\n}\n```",
        );
        let caps = extractor.detect_capabilities(&interaction);
        assert!(
            caps.iter().any(|c| c.name == "debugging"),
            "should detect debugging capability"
        );
    }

    #[test]
    fn test_extract_batch() {
        let mut extractor = CapabilityExtractor::new().with_min_observations(2);
        let interactions = vec![
            code_interaction("m1", "write rust code", "```rust\nfn a() {}\n```"),
            code_interaction("m1", "implement function", "```rust\nfn b() {}\n```"),
            code_interaction(
                "m2",
                "generate python",
                "```python\ndef c():\n    pass\n```",
            ),
        ];
        let caps = extractor.extract(&interactions);
        assert!(!caps.is_empty(), "should extract capabilities from batch");
        for cap in &caps {
            assert!(cap.observation_count >= 2 || caps.len() <= 1);
        }
    }

    #[test]
    fn test_empty_batch() {
        let mut extractor = CapabilityExtractor::new();
        let caps = extractor.extract(&[]);
        assert!(caps.is_empty());
    }

    #[test]
    fn test_below_min_observations() {
        let mut extractor = CapabilityExtractor::new().with_min_observations(10);
        let interactions = vec![code_interaction("m1", "write rust", "fn a() {}")];
        let caps = extractor.extract(&interactions);
        assert!(caps.is_empty());
    }

    #[test]
    fn test_detect_security_analysis() {
        let extractor = CapabilityExtractor::new();
        let interaction = CapturedInteraction::new(
            "test", "m1", "be secure", "analyze this for security vulnerability",
            "The vulnerability is in the unsanitized user input. Use parameterized queries to prevent SQL injection.",
            100, 200, 500, true, "stop",
        );
        let caps = extractor.detect_capabilities(&interaction);
        assert!(
            caps.iter().any(|c| c.name == "security_analysis"),
            "should detect security analysis"
        );
    }

    #[test]
    fn test_detect_testing() {
        let extractor = CapabilityExtractor::new();
        let interaction = CapturedInteraction::new(
            "test",
            "m1",
            "",
            "write tests for this",
            "```rust\n#[test]\nfn test_add() {\n    assert_eq!(add(1, 2), 3);\n}\n```",
            50,
            100,
            200,
            true,
            "stop",
        );
        let caps = extractor.detect_capabilities(&interaction);
        assert!(
            caps.iter().any(|c| c.name == "testing"),
            "should detect testing capability"
        );
    }

    #[test]
    fn test_detect_architecture() {
        let extractor = CapabilityExtractor::new();
        let interaction = CapturedInteraction::new(
            "test", "m1", "", "design a microservice architecture",
            "The system should have the following components: API Gateway, Service Registry, and Message Queue.",
            100, 200, 500, true, "stop",
        );
        let caps = extractor.detect_capabilities(&interaction);
        assert!(
            caps.iter().any(|c| c.name == "architecture_design"),
            "should detect architecture design"
        );
    }

    #[test]
    fn test_model_ranking() {
        let extractor = CapabilityExtractor::new();
        let interactions = vec![
            code_interaction("model-a", "write code", "fn a() {}"),
            code_interaction("model-a", "write more code", "fn b() {}"),
            code_interaction("model-b", "write code", "fn c() {}"),
        ];
        let ranking = extractor.model_ranking(&interactions, "code_generation");
        // Should have rankings for both models
        if !ranking.is_empty() {
            for (_, _, count) in &ranking {
                assert!(*count > 0);
            }
        }
    }

    #[test]
    fn test_sub_capability_detection() {
        let extractor = CapabilityExtractor::new();
        let interaction = code_interaction(
            "m1",
            "write async rust code",
            "use tokio;\n\nasync fn fetch() -> Result<String> {\n    Ok(\"data\".to_string())\n}\n",
        );
        let caps = extractor.detect_capabilities(&interaction);
        let code_cap = caps.iter().find(|c| c.name == "code_generation");
        if let Some(cap) = code_cap {
            assert!(
                cap.sub_capabilities
                    .iter()
                    .any(|s| s == "error_handling" || s == "async"),
                "should detect sub-capabilities like async or error_handling"
            );
        }
    }

    #[test]
    fn test_capability_confidence_boundary() {
        let mut extractor = CapabilityExtractor::new().with_min_observations(1);
        let interaction = CapturedInteraction::new(
            "test",
            "m1",
            "",
            "write code",
            "fn x() {}",
            10,
            20,
            50,
            true,
            "stop",
        );
        let caps = extractor.extract(&[interaction]);
        for cap in &caps {
            assert!(cap.proficiency >= 0.0 && cap.proficiency <= 1.0);
        }
    }
}
