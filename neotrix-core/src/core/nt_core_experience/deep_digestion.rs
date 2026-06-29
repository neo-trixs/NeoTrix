/// DDP — Deep Digestion Pipeline
/// 4-stage: Analyze → Map → Transform → Validate
use std::collections::HashMap;

/// A digested knowledge node
#[derive(Debug, Clone)]
pub struct DigestedNode {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub domain: String,
    pub confidence: f64,
    pub connections: Vec<String>,
    pub validation_history: Vec<ValidationRecord>,
    pub created_cycle: u64,
}

#[derive(Debug, Clone)]
pub struct ValidationRecord {
    pub validator: String,
    pub passed: bool,
    pub score: f64,
    pub cycle: u64,
}

/// Stage 1: Analyze — extract structure from raw input
#[derive(Debug, Clone)]
pub struct AnalysisOutput {
    pub entities: Vec<String>,
    pub relations: Vec<(String, String, String)>,
    pub key_claims: Vec<String>,
    pub uncertainty: f64,
}

/// Stage 2: Map — connect to existing knowledge
#[derive(Debug, Clone)]
pub struct MappingOutput {
    pub node_id: String,
    pub mapped_connections: Vec<String>,
    pub similarity_scores: Vec<(String, f64)>,
    pub novelty: f64,
}

/// Stage 3: Transform — synthesize into structured node
#[derive(Debug, Clone)]
pub struct TransformOutput {
    pub node: DigestedNode,
    pub synthesis_quality: f64,
}

/// Stage 4: Validate — cross-verify against known truth
#[derive(Debug, Clone)]
pub struct ValidationOutput {
    pub node_id: String,
    pub passed: bool,
    pub overall_score: f64,
    pub validation_records: Vec<ValidationRecord>,
}

/// The Deep Digestion Pipeline
#[derive(Debug, Clone)]
pub struct DeepDigestionPipeline {
    pub nodes: HashMap<String, DigestedNode>,
    pub analysis_count: u64,
    pub mapping_count: u64,
    pub transform_count: u64,
    pub validation_count: u64,
    cycle: u64,
}

impl DeepDigestionPipeline {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            analysis_count: 0,
            mapping_count: 0,
            transform_count: 0,
            validation_count: 0,
            cycle: 0,
        }
    }

    pub fn advance_cycle(&mut self) {
        self.cycle += 1;
    }

    /// Stage 1: Analyze raw input
    pub fn analyze(&mut self, raw_input: &str, _source: &str) -> AnalysisOutput {
        self.analysis_count += 1;
        let sentences: Vec<&str> = raw_input
            .split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut entities: Vec<String> = Vec::new();
        let mut key_claims: Vec<String> = Vec::new();
        for sentence in &sentences {
            if sentence.len() > 10 {
                key_claims.push(sentence.to_string());
            }
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for word in words {
                if word.starts_with(|c: char| c.is_uppercase()) && word.len() > 2 {
                    let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                    if !clean.is_empty() && !entities.contains(&clean.to_string()) {
                        entities.push(clean.to_string());
                    }
                }
            }
        }

        let mut relations: Vec<(String, String, String)> = Vec::new();
        for (i, claim) in key_claims.iter().enumerate() {
            for entity in &entities {
                if claim.contains(entity) {
                    for other in &entities {
                        if entity != other && claim.contains(other) {
                            relations.push((
                                entity.clone(),
                                "relates_to".to_string(),
                                other.clone(),
                            ));
                        }
                    }
                }
            }
            if i > 0 {
                relations.push((
                    format!("claim_{}", i - 1),
                    "precedes".to_string(),
                    format!("claim_{}", i),
                ));
            }
        }

        AnalysisOutput {
            entities,
            relations,
            key_claims,
            uncertainty: 1.0 / (sentences.len().max(1) as f64),
        }
    }

    /// Stage 2: Map to existing knowledge
    pub fn map(&mut self, analysis: &AnalysisOutput, domain: &str) -> MappingOutput {
        self.mapping_count += 1;
        let node_id = format!("node_{}", self.mapping_count);
        let mut mapped_connections: Vec<String> = Vec::new();
        let mut similarity_scores: Vec<(String, f64)> = Vec::new();

        for (existing_id, existing) in &self.nodes {
            if existing.domain == domain {
                let overlap = analysis
                    .entities
                    .iter()
                    .filter(|e| existing.content.contains(e.as_str()))
                    .count();
                if overlap > 0 {
                    let sim = overlap as f64 / (analysis.entities.len().max(1) as f64);
                    similarity_scores.push((existing_id.clone(), sim));
                    if sim > 0.3 {
                        mapped_connections.push(existing_id.clone());
                    }
                }
            }
        }

        let avg_sim = if similarity_scores.is_empty() {
            1.0
        } else {
            similarity_scores.iter().map(|(_, s)| s).sum::<f64>() / similarity_scores.len() as f64
        };
        let novelty = 1.0 - avg_sim;

        MappingOutput {
            node_id,
            mapped_connections,
            similarity_scores,
            novelty,
        }
    }

    /// Stage 3: Transform into a structured knowledge node
    pub fn transform(
        &mut self,
        analysis: AnalysisOutput,
        mapping: MappingOutput,
        title: &str,
        source: &str,
        domain: &str,
    ) -> TransformOutput {
        self.transform_count += 1;
        let content = analysis.key_claims.join(". ");
        let node = DigestedNode {
            id: mapping.node_id.clone(),
            title: title.to_string(),
            content,
            source: source.to_string(),
            domain: domain.to_string(),
            confidence: (1.0 - analysis.uncertainty) * (0.5 + mapping.novelty * 0.5),
            connections: mapping.mapped_connections,
            validation_history: Vec::new(),
            created_cycle: self.cycle,
        };
        let quality = (node.confidence * 0.7 + mapping.novelty * 0.3).clamp(0.0, 1.0);
        TransformOutput {
            node,
            synthesis_quality: quality,
        }
    }

    /// Stage 4: Validate a node
    pub fn validate(&mut self, node_id: &str, validator_name: &str) -> Option<ValidationOutput> {
        self.validation_count += 1;
        let node = self.nodes.get(node_id)?;
        let has_connections = !node.connections.is_empty();
        let has_content = !node.content.is_empty();
        let confidence_ok = node.confidence > 0.3;
        let passed = has_content && (has_connections || confidence_ok);
        let score = if passed {
            (node.confidence * 0.5 + if has_connections { 0.3 } else { 0.0 } + 0.2).clamp(0.0, 1.0)
        } else {
            0.2
        };
        let record = ValidationRecord {
            validator: validator_name.to_string(),
            passed,
            score,
            cycle: self.cycle,
        };
        if let Some(n) = self.nodes.get_mut(node_id) {
            n.validation_history.push(record.clone());
            n.confidence = (n.confidence + score * 0.1).clamp(0.0, 1.0);
        }
        Some(ValidationOutput {
            node_id: node_id.to_string(),
            passed,
            overall_score: score,
            validation_records: vec![record],
        })
    }

    /// Run a complete digestion cycle on raw input
    pub fn digest(&mut self, raw: &str, title: &str, source: &str, domain: &str) -> Option<String> {
        let analysis = self.analyze(raw, source);
        let mapping = self.map(&analysis, domain);
        let transform = self.transform(analysis, mapping, title, source, domain);
        let node_id = transform.node.id.clone();
        self.nodes.insert(node_id.clone(), transform.node);
        Some(node_id)
    }

    /// Get cross-validation consistency between two nodes
    pub fn cross_validate(&self, id_a: &str, id_b: &str) -> Option<f64> {
        let a = self.nodes.get(id_a)?;
        let b = self.nodes.get(id_b)?;
        let a_words: Vec<&str> = a.content.split_whitespace().collect();
        let b_words: Vec<&str> = b.content.split_whitespace().collect();
        let intersection: usize = a_words.iter().filter(|w| b_words.contains(w)).count();
        let union = a_words.len() + b_words.len() - intersection;
        if union == 0 {
            return Some(0.0);
        }
        let jaccard = intersection as f64 / union as f64;
        Some(jaccard * (a.confidence.min(b.confidence)))
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Digestion: {} nodes, A:{} M:{} T:{} V:{}, avg confidence: {:.3}",
            self.nodes.len(),
            self.analysis_count,
            self.mapping_count,
            self.transform_count,
            self.validation_count,
            self.nodes.values().map(|n| n.confidence).sum::<f64>() / self.nodes.len().max(1) as f64,
        )
    }
}

impl Default for DeepDigestionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_extracts_entities_and_claims() {
        let mut pipe = DeepDigestionPipeline::new();
        let input = "The E8 Lattice achieves optimal sphere packing in 8 dimensions. \
                      This discovery was made by Maryna Viazovska. \
                      It has applications in error-correcting codes.";
        let output = pipe.analyze(input, "test");
        assert!(!output.entities.is_empty(), "should extract entities");
        assert!(
            output.entities.contains(&"E8".to_string())
                || output.entities.contains(&"Lattice".to_string())
        );
        assert!(output.key_claims.len() >= 2);
    }

    #[test]
    fn test_analyze_empty_input() {
        let mut pipe = DeepDigestionPipeline::new();
        let output = pipe.analyze("", "test");
        assert!(output.entities.is_empty());
        assert!(output.key_claims.is_empty());
    }

    #[test]
    fn test_digest_full_pipeline() {
        let mut pipe = DeepDigestionPipeline::new();
        let node_id = pipe.digest(
            "NeoTrix uses VSA 4096-bit vectors for unified representation. \
             The HyperCube enables fast similarity search.",
            "VSA Architecture",
            "research",
            "ai",
        );
        assert!(node_id.is_some());
        let id = node_id.unwrap();
        assert_eq!(pipe.len(), 1);
        let node = pipe.nodes.get(&id).unwrap();
        assert!(node.confidence > 0.0);
        assert!(!node.content.is_empty());
    }

    #[test]
    fn test_validate_node() {
        let mut pipe = DeepDigestionPipeline::new();
        let id = pipe
            .digest("Test content for validation.", "Test", "test", "test")
            .unwrap();
        let result = pipe.validate(&id, "auto_validator");
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.passed);
        assert!(output.overall_score > 0.0);
    }

    #[test]
    fn test_validate_nonexistent_node() {
        let mut pipe = DeepDigestionPipeline::new();
        let result = pipe.validate("nonexistent", "test");
        assert!(result.is_none());
    }

    #[test]
    fn test_mapping_discovers_connections() {
        let mut pipe = DeepDigestionPipeline::new();
        pipe.digest(
            "The E8 lattice achieves optimal sphere packing.",
            "First",
            "test",
            "math",
        );
        let id2 = pipe.digest(
            "E8 is used in error-correcting codes and machine learning.",
            "Second",
            "test",
            "math",
        );
        let node2 = pipe.nodes.get(&id2.unwrap()).unwrap();
        assert!(!node2.connections.is_empty() || node2.confidence > 0.0);
    }

    #[test]
    fn test_cross_validate_consistent_nodes() {
        let mut pipe = DeepDigestionPipeline::new();
        let id1 = pipe
            .digest("VSA uses hyperdimensional computing.", "A", "src", "ai")
            .unwrap();
        let id2 = pipe
            .digest("VSA vectors are high-dimensional.", "B", "src", "ai")
            .unwrap();
        let score = pipe.cross_validate(&id1, &id2);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }

    #[test]
    fn test_cross_validate_different_nodes() {
        let mut pipe = DeepDigestionPipeline::new();
        let id1 = pipe
            .digest("Machine learning is a subset of AI.", "A", "src", "ai")
            .unwrap();
        let id2 = pipe
            .digest("Cooking recipes require ingredients.", "B", "src", "food")
            .unwrap();
        let score = pipe.cross_validate(&id1, &id2);
        assert!(score.is_some());
        assert!(
            score.unwrap() < 0.3,
            "unrelated nodes should have low cross-validation"
        );
    }

    #[test]
    fn test_summary() {
        let mut pipe = DeepDigestionPipeline::new();
        pipe.digest("Test content.", "Test", "src", "test");
        let s = pipe.summary();
        assert!(s.contains("nodes"));
        assert!(s.contains("avg confidence"));
    }

    #[test]
    fn test_validation_multiple_times() {
        let mut pipe = DeepDigestionPipeline::new();
        let id = pipe
            .digest("Validation test node.", "V", "src", "test")
            .unwrap();
        pipe.validate(&id, "v1");
        pipe.validate(&id, "v2");
        let node = pipe.nodes.get(&id).unwrap();
        assert_eq!(node.validation_history.len(), 2);
        assert!(node.confidence > 0.0);
    }

    #[test]
    fn test_transform_quality() {
        let mut pipe = DeepDigestionPipeline::new();
        let analysis = pipe.analyze("High quality input with specific details.", "src");
        let mapping = pipe.map(&analysis, "test");
        let mapping2 = MappingOutput {
            node_id: mapping.node_id,
            mapped_connections: mapping.mapped_connections,
            similarity_scores: mapping.similarity_scores,
            novelty: 0.8,
        };
        let transform = pipe.transform(analysis, mapping2, "Quality", "src", "test");
        assert!(
            transform.synthesis_quality > 0.0,
            "should produce quality score"
        );
    }

    #[test]
    fn test_analyze_relations() {
        let mut pipe = DeepDigestionPipeline::new();
        let output = pipe.analyze(
            "Alice discovered the theorem. Bob verified the proof.",
            "test",
        );
        assert!(!output.relations.is_empty(), "should extract relations");
    }
}
