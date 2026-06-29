use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::capture::CapturedInteraction;

/// A knowledge fragment extracted from a model response.
/// Domain-specific, model-agnostic factual/insight capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    /// Unique ID
    pub id: String,
    /// Domain this knowledge belongs to
    pub domain: String,
    /// The extracted knowledge text
    pub content: String,
    /// Estimated accuracy/utility (0.0–1.0)
    pub confidence: f64,
    /// How many independent observations support this
    pub observation_count: u32,
    /// Which models expressed this knowledge
    pub observed_models: Vec<String>,
    /// Mutually consistent — true if multiple models said similar things
    pub cross_validated: bool,
    /// Tags for retrieval
    pub tags: Vec<String>,
}

/// Domain-specific extraction patterns.
/// Maps domain → (trigger keywords in prompt, extraction patterns in response)
const DOMAIN_PATTERNS: &[(&str, &[&str], &[&str])] = &[
    (
        "rust",
        &["rust", "cargo", "rustc", "rustacean"],
        &[
            "unsafe",
            "trait",
            "impl",
            "#[derive]",
            "let mut",
            "Arc<",
            "Rc<",
            "Box<",
            "Result<",
            "Option<",
            "match ",
            "enum ",
            "struct ",
        ],
    ),
    (
        "python",
        &["python", "django", "flask", "pytorch", "numpy", "pandas"],
        &[
            "def ",
            "class ",
            "import ",
            "yield",
            "async def",
            "decorator",
            "list comprehension",
            "generator",
            "context manager",
        ],
    ),
    (
        "javascript",
        &[
            "javascript",
            "typescript",
            "node",
            "react",
            "vue",
            "angular",
        ],
        &[
            "const ",
            "let ",
            "function",
            "=>",
            "async",
            "await",
            "Promise",
            "export",
            "import",
            "useState",
            "useEffect",
            "Component",
        ],
    ),
    (
        "system_design",
        &["system design", "architecture", "distributed", "scalable"],
        &[
            "load balancer",
            "database sharding",
            "caching",
            "message queue",
            "CDN",
            "microservice",
            "eventual consistency",
            "CAP theorem",
        ],
    ),
    (
        "algorithms",
        &[
            "algorithm",
            "complexity",
            "data structure",
            "sort",
            "search",
        ],
        &[
            "O(",
            "time complexity",
            "space complexity",
            "recursive",
            "dynamic programming",
            "binary search",
            "hash map",
            "tree",
        ],
    ),
    (
        "devops",
        &["deploy", "docker", "kubernetes", "ci/cd", "infrastructure"],
        &[
            "Dockerfile",
            "Kubernetes",
            "pod",
            "service mesh",
            "helm",
            "terraform",
            "ansible",
            "jenkins",
            "github actions",
        ],
    ),
    (
        "security",
        &[
            "security",
            "secure",
            "vulnerability",
            "harden",
            "penetration",
        ],
        &[
            "OWASP",
            "XSS",
            "CSRF",
            "SQL injection",
            "authentication",
            "authorization",
            "OAuth",
            "JWT",
            "encryption",
            "hash",
        ],
    ),
    (
        "database",
        &["database", "sql", "nosql", "postgresql", "mongodb", "redis"],
        &[
            "index",
            "query",
            "transaction",
            "ACID",
            "normalization",
            "denormalization",
            "join",
            "foreign key",
            "primary key",
        ],
    ),
];

/// Extracts knowledge fragments from model responses.
/// Domain-triggered extraction: when user asks about a domain,
/// extract and catalog the factual content from the response.
#[derive(Debug, Clone)]
pub struct KnowledgeExtractor {
    min_observations: usize,
    min_confidence: f64,
    extractions: u64,
    dedup_similarity_threshold: f64,
}

impl KnowledgeExtractor {
    pub fn new() -> Self {
        Self {
            min_observations: 1,
            min_confidence: 0.3,
            extractions: 0,
            dedup_similarity_threshold: 0.75,
        }
    }

    pub fn with_min_observations(mut self, min: usize) -> Self {
        self.min_observations = min;
        self
    }

    /// Extract knowledge from a batch of interactions.
    /// Returns deduplicated list of knowledge fragments.
    pub fn extract(&mut self, interactions: &[CapturedInteraction]) -> Vec<KnowledgeFragment> {
        // First pass: detect snippets per interaction
        let mut raw_fragments: Vec<KnowledgeFragment> = Vec::new();
        for interaction in interactions {
            let detected = self.detect_knowledge(interaction);
            raw_fragments.extend(detected);
        }

        // Second pass: deduplicate by domain+content similarity
        let mut merged: HashMap<String, Vec<KnowledgeFragment>> = HashMap::new();
        for frag in raw_fragments {
            merged.entry(frag.domain.clone()).or_default().push(frag);
        }

        let mut result = Vec::new();

        for (_domain, fragments) in merged {
            if fragments.len() < self.min_observations {
                continue;
            }

            // Merge fragments with similar content
            let mut merged_frags = self.merge_fragments(fragments);

            for frag in &mut merged_frags {
                if frag.confidence < self.min_confidence {
                    continue;
                }
                let mut models: Vec<String> = frag.observed_models.clone();
                models.sort();
                models.dedup();
                frag.observed_models = models;
                frag.cross_validated = frag.observed_models.len() > 1;
                self.extractions += 1;
                result.push(frag.clone());
            }
        }

        result.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result
    }

    /// Detect knowledge domains in a single interaction.
    pub fn detect_knowledge(&self, interaction: &CapturedInteraction) -> Vec<KnowledgeFragment> {
        let lower_prompt = interaction.user_messages.to_lowercase();
        let lower_response = interaction.response.to_lowercase();

        let mut fragments = Vec::new();

        for (domain, prompt_triggers, response_markers) in DOMAIN_PATTERNS {
            let prompt_matches = prompt_triggers
                .iter()
                .any(|kw| lower_prompt.contains(&kw.to_lowercase()));
            let response_matches = response_markers
                .iter()
                .any(|m| lower_response.contains(&m.to_lowercase()));

            if prompt_matches && response_matches {
                // Extract the most relevant sentences
                let sentences =
                    self.extract_key_sentences(&interaction.response, response_markers, 3);
                for sentence in sentences {
                    let sentence_lower = sentence.to_lowercase();
                    let mut tags: Vec<String> = response_markers
                        .iter()
                        .filter(|m| sentence_lower.contains(&m.to_lowercase()))
                        .map(|m| m.to_string())
                        .collect();
                    tags.push(domain.to_string());
                    tags.sort();
                    tags.dedup();

                    fragments.push(KnowledgeFragment {
                        id: format!("kf_{}_{}", self.extractions + 1, fragments.len() + 1),
                        domain: domain.to_string(),
                        content: sentence,
                        confidence: interaction.outcome_score * 0.8 + 0.2,
                        observation_count: 1,
                        observed_models: vec![interaction.model.clone()],
                        cross_validated: false,
                        tags,
                    });
                }
            }
        }

        fragments
    }

    /// Extract key sentences from text that contain domain markers.
    fn extract_key_sentences(
        &self,
        text: &str,
        markers: &[&str],
        max_sentences: usize,
    ) -> Vec<String> {
        let sentences: Vec<String> = text
            .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() > 20 && s.len() < 500)
            .collect();

        // Score sentences by marker density
        let mut scored: Vec<(usize, String)> = sentences
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                let score = markers
                    .iter()
                    .filter(|m| s.to_lowercase().contains(*m))
                    .count();
                (score * 1000 + idx, s)
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored
            .into_iter()
            .take(max_sentences)
            .map(|(_, s)| s)
            .collect()
    }

    /// Merge fragments with similar content within the same domain.
    fn merge_fragments(&self, fragments: Vec<KnowledgeFragment>) -> Vec<KnowledgeFragment> {
        let mut merged: Vec<KnowledgeFragment> = Vec::new();
        let mut used = vec![false; fragments.len()];

        for i in 0..fragments.len() {
            if used[i] {
                continue;
            }
            let mut base = fragments[i].clone();
            used[i] = true;

            for j in (i + 1)..fragments.len() {
                if used[j] {
                    continue;
                }
                let sim = self.text_similarity(&base.content, &fragments[j].content);
                if sim > self.dedup_similarity_threshold {
                    base.observation_count += fragments[j].observation_count;
                    base.confidence = base.confidence.max(fragments[j].confidence);
                    base.observed_models
                        .extend(fragments[j].observed_models.clone());
                    base.tags.extend(fragments[j].tags.clone());
                    used[j] = true;
                }
            }

            base.tags.sort();
            base.tags.dedup();
            base.observed_models.sort();
            base.observed_models.dedup();

            merged.push(base);
        }

        merged
    }

    /// Simple word-overlap similarity for dedup.
    fn text_similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let words_a: HashSet<&str> = a_lower.split_whitespace().filter(|w| w.len() > 3).collect();
        let words_b: HashSet<&str> = b_lower.split_whitespace().filter(|w| w.len() > 3).collect();

        let intersection: HashSet<&&str> = words_a.intersection(&words_b).collect();
        let union: HashSet<&&str> = words_a.union(&words_b).collect();

        if union.is_empty() {
            return 0.0;
        }
        intersection.len() as f64 / union.len() as f64
    }

    pub fn extractions(&self) -> u64 {
        self.extractions
    }
}

impl Default for KnowledgeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    fn interaction(model: &str, prompt: &str, response: &str) -> CapturedInteraction {
        CapturedInteraction::new(
            "test", model, "", prompt, response, 50, 100, 200, true, "stop",
        )
    }

    #[test]
    fn test_detect_rust_knowledge() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "how do I use trait objects in rust",
            "Trait objects in Rust use dyn keyword and are stored behind a pointer like Box<dyn Trait>. They enable dynamic dispatch at runtime.",
        );
        let fragments = extractor.detect_knowledge(&interaction);
        assert!(
            fragments.iter().any(|f| f.domain == "rust"),
            "should detect rust domain knowledge"
        );
    }

    #[test]
    fn test_detect_python_knowledge() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "explain python decorators",
            "A decorator in Python is a function that takes another function and extends its behavior without explicitly modifying it. Use @decorator syntax.",
        );
        let fragments = extractor.detect_knowledge(&interaction);
        assert!(
            fragments.iter().any(|f| f.domain == "python"),
            "should detect python domain knowledge"
        );
    }

    #[test]
    fn test_empty_interaction() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction("m1", "hello", "hi there");
        let fragments = extractor.detect_knowledge(&interaction);
        assert!(fragments.is_empty());
    }

    #[test]
    fn test_extract_batch() {
        let mut extractor = KnowledgeExtractor::new();
        let interactions = vec![
            interaction(
                "m1",
                "rust traits",
                "Traits in Rust are similar to interfaces. They define shared behavior.",
            ),
            interaction(
                "m2",
                "rust ownership",
                "Ownership is Rust's unique memory management system.",
            ),
        ];
        let fragments = extractor.extract(&interactions);
        assert!(!fragments.is_empty());
    }

    #[test]
    fn test_text_similarity() {
        let extractor = KnowledgeExtractor::new();
        let sim = extractor.text_similarity(
            "Rust uses ownership model for memory safety",
            "Rust ownership model ensures memory safety without garbage collector",
        );
        assert!(sim > 0.0, "similar texts should have positive similarity");
        assert!(
            sim < 1.0,
            "non-identical texts should not have 1.0 similarity"
        );
    }

    #[test]
    fn test_identical_text_similarity() {
        let extractor = KnowledgeExtractor::new();
        let sim = extractor.text_similarity(
            "Rust ownership model ensures memory safety",
            "Rust ownership model ensures memory safety",
        );
        assert!(
            (sim - 1.0).abs() < 0.001,
            "identical texts should have 1.0 similarity"
        );
    }

    #[test]
    fn test_detect_system_design() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "design a scalable chat system",
            "Use a message queue like Kafka for event streaming. The system should have a load balancer and database sharding.",
        );
        let fragments = extractor.detect_knowledge(&interaction);
        assert!(
            fragments.iter().any(|f| f.domain == "system_design"),
            "should detect system design knowledge"
        );
    }

    #[test]
    fn test_confidence_scaling() {
        let mut extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "rust traits explained",
            "Traits in Rust define shared behavior across types.",
        );
        let mut fragments = extractor.extract(&[interaction]);
        for f in &fragments {
            assert!(f.confidence > 0.0 && f.confidence <= 1.0);
        }
    }

    // Helper with custom outcome for consistency test
    fn interaction_custom(
        model: &str,
        prompt: &str,
        response: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency: u64,
        success: bool,
        finish: &str,
    ) -> CapturedInteraction {
        CapturedInteraction::new(
            "test",
            model,
            "",
            prompt,
            response,
            prompt_tokens,
            completion_tokens,
            latency,
            success,
            finish,
        )
    }

    #[test]
    fn test_cross_validation_flag() {
        let mut extractor = KnowledgeExtractor::new();
        let interactions = vec![
            interaction_custom(
                "m1",
                "rust ownership",
                "Ownership is Rust's memory management system",
                50,
                100,
                200,
                true,
                "stop",
            ),
            interaction_custom(
                "m2",
                "rust borrow checker",
                "The borrow checker enforces ownership rules at compile time",
                50,
                100,
                200,
                true,
                "stop",
            ),
        ];
        let fragments = extractor.extract(&interactions);
        for f in &fragments {
            if f.observed_models.len() > 1 {
                assert!(f.cross_validated);
            }
        }
    }

    #[test]
    fn test_knowledge_tags() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "rust traits vs interfaces",
            "Traits in Rust are like interfaces but with associated types.",
        );
        let fragments = extractor.detect_knowledge(&interaction);
        for f in &fragments {
            assert!(!f.tags.is_empty(), "knowledge fragments should have tags");
            assert!(f.tags.contains(&"rust".to_string()));
        }
    }

    #[test]
    fn test_extract_multiple_domains() {
        let extractor = KnowledgeExtractor::new();
        let interaction = interaction(
            "m1",
            "deploy a secure docker container",
            "Authentication uses JWT tokens. The container is deployed via Kubernetes with a Dockerfile.",
        );
        let fragments = extractor.detect_knowledge(&interaction);
        let domains: std::collections::HashSet<String> =
            fragments.iter().map(|f| f.domain.clone()).collect();
        assert!(
            domains.len() >= 2,
            "should detect multiple domains: {:?}",
            domains
        );
    }
}
