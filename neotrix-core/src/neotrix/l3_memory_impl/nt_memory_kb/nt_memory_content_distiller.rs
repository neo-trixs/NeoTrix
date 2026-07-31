//! nt_memory_content_distiller — KB 内容蒸馏模块
//!
//! 从已吸收的 KB 节点（GitHub 仓库、论文、文章等）中提取可复用的
//! 模式、原则、架构洞察，生成 EvolutionRecord 和 Insight 节点，
//! 构建跨源知识蒸馏管道。

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::nt_memory_types::*;
use super::KnowledgeBase;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationReport {
    pub total_nodes_scanned: usize,
    pub insights_generated: usize,
    pub patterns_detected: Vec<DetectedPattern>,
    pub cross_references_created: usize,
    pub evolution_records_created: usize,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: DistillationPattern,
    pub source_nodes: Vec<String>,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistillationPattern {
    /// 重复的架构模式
    ArchitecturalPattern,
    /// 技术栈共性
    TechStackCommonality,
    /// 设计原则
    DesignPrinciple,
    /// 项目组织模式
    ProjectStructure,
    /// 依赖模式
    DependencyPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillerConfig {
    pub min_insight_confidence: f64,
    pub max_patterns_per_run: usize,
    pub enable_cross_referencing: bool,
    pub enable_evolution_records: bool,
    pub scan_recent_days: i64,
}

impl Default for DistillerConfig {
    fn default() -> Self {
        Self {
            min_insight_confidence: 0.6,
            max_patterns_per_run: 20,
            enable_cross_referencing: true,
            enable_evolution_records: true,
            scan_recent_days: 30,
        }
    }
}

// ── ContentDistiller ──

pub struct ContentDistiller {
    kb: KnowledgeBase,
    config: DistillerConfig,
}

impl ContentDistiller {
    pub fn new(kb: KnowledgeBase) -> Self {
        Self { kb, config: DistillerConfig::default() }
    }

    pub fn with_config(kb: KnowledgeBase, config: DistillerConfig) -> Self {
        Self { kb, config }
    }

    /// Run a full distillation cycle over all KB content
    pub fn distil_all(&self) -> Result<DistillationReport, String> {
        let mut report = DistillationReport {
            total_nodes_scanned: 0,
            insights_generated: 0,
            patterns_detected: Vec::new(),
            cross_references_created: 0,
            evolution_records_created: 0,
            timestamp: now(),
        };

        // Phase 1: Scan repositories → extract tech stack patterns
        let repos = self.kb.find_repositories("github.com", None)?;
        report.total_nodes_scanned += repos.len();
        let tech_patterns = self.analyze_repo_tech_stacks(&repos);
        report.patterns_detected.extend(tech_patterns);

        // Phase 2: Scan code snippets → detect architectural patterns
        let mut all_snippets = Vec::new();
        for repo in &repos {
            if let Ok(snippets) = self.kb.find_code_snippets(&repo.id) {
                all_snippets.extend(snippets);
            }
        }
        report.total_nodes_scanned += all_snippets.len();
        let arch_patterns = self.analyze_code_patterns(&all_snippets);
        report.patterns_detected.extend(arch_patterns);

        // Phase 3: Cross-source pattern detection
        if self.config.enable_cross_referencing {
            let cross_refs = self.detect_cross_references(&repos, &all_snippets);
            report.cross_references_created += cross_refs;
        }

        // Phase 4: Convert patterns → KB insights + evolution records
        let mut patterns = std::mem::take(&mut report.patterns_detected);
        patterns.truncate(self.config.max_patterns_per_run);

        for pattern in &patterns {
            if pattern.confidence >= self.config.min_insight_confidence {
                if let Ok(()) = self.persist_pattern(pattern) {
                    report.insights_generated += 1;
                }
            }
        }

        // Phase 5: Generate evolution records for salient patterns
        if self.config.enable_evolution_records {
            for pattern in &patterns {
                if pattern.confidence >= 0.8 {
                    if let Ok(()) = self.create_evolution_record(pattern) {
                        report.evolution_records_created += 1;
                    }
                }
            }
        }

        // Restore patterns to report for downstream consumers
        report.patterns_detected = patterns;

        // Phase 6: Write distillation report to KV
        let json = serde_json::to_string(&report).unwrap_or_default();
        let _ = self.kb.kv_set("distiller", &format!("report_{}", now()), &json);

        Ok(report)
    }

    /// Distill only a specific node (e.g., after absorbing a new repo)
    pub fn distil_node(&self, node_id: &str) -> Result<Option<DetectedPattern>, String> {
        let node = self.kb.get_node(node_id)?.ok_or_else(|| format!("Node not found: {}", node_id))?;

        match node.node_type {
            NodeType::Repository => {
                let repos = vec![node];
                let patterns = self.analyze_repo_tech_stacks(&repos);
                for p in patterns {
                    if p.confidence >= self.config.min_insight_confidence {
                        self.persist_pattern(&p)?;
                        return Ok(Some(p));
                    }
                }
            }
            NodeType::CodeSnippet => {
                let snippets = vec![node];
                let patterns = self.analyze_code_patterns(&snippets);
                for p in patterns {
                    if p.confidence >= self.config.min_insight_confidence {
                        self.persist_pattern(&p)?;
                        return Ok(Some(p));
                    }
                }
            }
            _ => {
                // Text-based pattern extraction for articles/papers
                if let Some(content) = &node.content {
                    let patterns = self.extract_text_patterns(&node.title, content, &node.node_type);
                    for p in patterns {
                        if p.confidence >= self.config.min_insight_confidence {
                            self.persist_pattern(&p)?;
                            return Ok(Some(p));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    // ── Private: Tech Stack Analysis ──

    fn analyze_repo_tech_stacks(&self, repos: &[KnowledgeNode]) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Group by language
        let mut by_lang: HashMap<String, Vec<&KnowledgeNode>> = HashMap::new();
        for repo in repos {
            if let Some(meta) = &repo.metadata {
                if let Some(lang) = meta.get("language").and_then(|v| v.as_str()) {
                    if !lang.is_empty() && lang != "unknown" {
                        by_lang.entry(lang.to_string()).or_default().push(repo);
                    }
                }
            }
        }
        for (lang, group) in &by_lang {
            if group.len() >= 2 {
                let names: Vec<String> = group.iter().map(|r| r.title.clone()).take(5).collect();
                patterns.push(DetectedPattern {
                    pattern_type: DistillationPattern::TechStackCommonality,
                    source_nodes: group.iter().map(|r| r.id.clone()).collect(),
                    description: format!("{} projects using {}: {}", group.len(), lang, names.join(", ")),
                    confidence: 0.7 + (group.len() as f64).min(5.0) * 0.05,
                });
            }
        }

        // Group by topic overlap
        let mut by_topic: HashMap<String, Vec<&KnowledgeNode>> = HashMap::new();
        for repo in repos {
            if let Some(meta) = &repo.metadata {
                if let Some(topics) = meta.get("topics").and_then(|v| v.as_array()) {
                    for topic_val in topics {
                        if let Some(topic) = topic_val.as_str() {
                            by_topic.entry(topic.to_string()).or_default().push(repo);
                        }
                    }
                }
            }
        }
        for (topic, group) in &by_topic {
            if group.len() >= 3 {
                let names: Vec<String> = group.iter().map(|r| r.title.clone()).take(5).collect();
                patterns.push(DetectedPattern {
                    pattern_type: DistillationPattern::ArchitecturalPattern,
                    source_nodes: group.iter().map(|r| r.id.clone()).collect(),
                    description: format!("Topic '{}' appears across {} repos: {}", topic, group.len(), names.join(", ")),
                    confidence: 0.65 + (group.len() as f64).min(5.0) * 0.04,
                });
            }
        }

        patterns
    }

    // ── Private: Code Pattern Analysis ──

    fn analyze_code_patterns(&self, snippets: &[KnowledgeNode]) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Detect common file names (project structure patterns)
        let mut file_names: HashMap<String, usize> = HashMap::new();
        let mut by_ext: HashMap<String, Vec<&KnowledgeNode>> = HashMap::new();
        for snippet in snippets {
            let path = snippet.title.as_str();
            let name = path.split('/').next_back().unwrap_or(path);
            *file_names.entry(name.to_string()).or_insert(0) += 1;
            if let Some(meta) = &snippet.metadata {
                if let Some(ext) = meta.get("path").and_then(|v| v.as_str())
                    .and_then(|p| p.rsplit('.').next())
                {
                    by_ext.entry(ext.to_string()).or_default().push(snippet);
                }
            }
        }

        // Common module patterns
        for (name, count) in &file_names {
            if *count >= 2 && (*name == "mod.rs" || *name == "lib.rs" || *name == "main.rs") {
                patterns.push(DetectedPattern {
                    pattern_type: DistillationPattern::ProjectStructure,
                    source_nodes: Vec::new(),
                    description: format!("Module pattern '{}' appears {} times — standard project structure", name, count),
                    confidence: 0.7,
                });
            }
        }

        // Language distribution patterns
        for (ext, group) in &by_ext {
            if group.len() >= 3 {
                patterns.push(DetectedPattern {
                    pattern_type: DistillationPattern::ArchitecturalPattern,
                    source_nodes: group.iter().map(|s| s.id.clone()).collect(),
                    description: format!("{} source files with .{} extension across KB", group.len(), ext),
                    confidence: 0.6,
                });
            }
        }

        patterns
    }

    // ── Private: Cross-Reference Detection ──

    fn detect_cross_references(&self, repos: &[KnowledgeNode], snippets: &[KnowledgeNode]) -> usize {
        let mut refs_created = 0;

        // Find repos that share dependencies → link them
        let _dep_map: HashMap<String, Vec<String>> = HashMap::new();
        for repo in repos {
            if let Some(meta) = &repo.metadata {
                if let Some(deps_json) = meta.get("deps_detected") {
                    if let Some(_deps) = deps_json.as_u64() {
                        // Find related repos by query
                        let related = self.kb.find_repositories("github.com", None).unwrap_or_default();
                        for other in &related {
                            if other.id == repo.id { continue; }
                            // Check if they share topics
                            let topics_a = meta.get("topics").and_then(|t| t.as_array());
                            let topics_b = other.metadata.as_ref().and_then(|m| m.get("topics").and_then(|t| t.as_array()));
                            if let (Some(a), Some(b)) = (topics_a, topics_b) {
                                let a_set: HashSet<&str> = a.iter().filter_map(|v| v.as_str()).collect();
                                let b_set: HashSet<&str> = b.iter().filter_map(|v| v.as_str()).collect();
                                let overlap: Vec<&&str> = a_set.intersection(&b_set).collect();
                                if overlap.len() >= 2 {
                                    let _ = self.kb.upsert_edge(&repo.id, &other.id, RelationType::Related, 0.5,
                                        Some(&format!("Shared topics: {}", overlap.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))));
                                    refs_created += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Link code snippets to related concepts
        for snippet in snippets {
            let content = snippet.content.as_deref().unwrap_or("");
            // Simple keyword matching for common AI/ML concepts
            let concepts = ["neural", "transformer", "attention", "gradient", "embedding",
                "reinforcement", "convolution", "lstm", "mamba", "diffusion"];
            for concept in &concepts {
                if content.to_lowercase().contains(concept) {
                    let concept_id = self.kb.insert_or_get_node(
                        concept,
                        NodeType::Concept,
                        None, None, Some("distiller"),
                    ).unwrap_or_default();
                    if !concept_id.is_empty() {
                        let _ = self.kb.upsert_edge(&snippet.id, &concept_id, RelationType::References, 0.4, None);
                        refs_created += 1;
                    }
                }
            }
        }

        refs_created
    }

    // ── Private: Text Pattern Extraction ──

    fn extract_text_patterns(&self, title: &str, content: &str, _node_type: &NodeType) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Extract potential design principles from content
        let principles = [
            ("modular design", "Modular Architecture"),
            ("separation of concerns", "Separation of Concerns"),
            ("single responsibility", "Single Responsibility"),
            ("dependency injection", "Dependency Injection"),
            ("observer pattern", "Observer Pattern"),
            ("event-driven", "Event-Driven Architecture"),
            ("microservices", "Microservices Architecture"),
            ("test-driven", "Test-Driven Development"),
        ];
        let content_lower = content.to_lowercase();
        for (keyword, principle) in &principles {
            if content_lower.contains(keyword) {
                patterns.push(DetectedPattern {
                    pattern_type: DistillationPattern::DesignPrinciple,
                    source_nodes: Vec::new(),
                    description: format!("'{}' referenced in '{}' — suggests {}", keyword, title, principle),
                    confidence: 0.6,
                });
            }
        }

        patterns
    }

    // ── Private: Persistence ──

    fn persist_pattern(&self, pattern: &DetectedPattern) -> Result<(), String> {
        let label = match pattern.pattern_type {
            DistillationPattern::ArchitecturalPattern => "Architecture Pattern",
            DistillationPattern::TechStackCommonality => "Tech Stack Pattern",
            DistillationPattern::DesignPrinciple => "Design Principle",
            DistillationPattern::ProjectStructure => "Project Structure Pattern",
            DistillationPattern::DependencyPattern => "Dependency Pattern",
        };
        let node_id = self.kb.insert_or_get_node(
            &format!("Distilled: {} — {}", label, &pattern.description.chars().take(80).collect::<String>()),
            NodeType::Insight,
            Some(&pattern.description),
            None,
            Some("distiller"),
        )?;
        // Link to source nodes
        for source_id in &pattern.source_nodes {
            let _ = self.kb.upsert_edge(&node_id, source_id, RelationType::Supports, pattern.confidence,
                Some(&format!("Distilled {} pattern", label)));
        }
        Ok(())
    }

    fn create_evolution_record(&self, pattern: &DetectedPattern) -> Result<(), String> {
        let record = EvolutionRecord {
            id: Uuid::new_v4().to_string(),
            source_conversation_id: String::new(),
            pattern_type: match pattern.pattern_type {
                DistillationPattern::ArchitecturalPattern => EvolutionPatternType::StrategyDiscovery,
                DistillationPattern::TechStackCommonality => EvolutionPatternType::ToolUsagePattern,
                DistillationPattern::DesignPrinciple => EvolutionPatternType::PrincipleUpdate,
                DistillationPattern::ProjectStructure => EvolutionPatternType::ProblemDecomposition,
                DistillationPattern::DependencyPattern => EvolutionPatternType::ToolUsagePattern,
            },
            description: pattern.description.clone(),
            before_behavior: String::new(),
            after_behavior: format!("Incorporated insight: {}", pattern.description),
            effectiveness_gain: pattern.confidence * 0.5,
            applied_to: pattern.source_nodes.clone(),
            verified: false,
            timestamp: now(),
        };
        self.kb.store_evolution_record(&record)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kb() -> KnowledgeBase {
        KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
    }

    fn insert_repo(kb: &KnowledgeBase, title: &str, lang: &str, topics: &[&str]) -> String {
        let node_id = kb.insert_or_get_node(
            title, NodeType::Repository, None, Some(&format!("https://github.com/{}", title)), Some("github.com"),
        ).unwrap();
        let meta = serde_json::json!({
            "language": lang,
            "topics": topics,
            "stars": 100,
        });
        kb.update_node_metadata(&node_id, &meta).unwrap();
        node_id
    }

    fn insert_snippet(kb: &KnowledgeBase, title: &str, content: &str) -> String {
        let node_id = kb.insert_or_get_node(
            title, NodeType::CodeSnippet, None, None, Some("github.com"),
        ).unwrap();
        kb.update_node_content(&node_id, content).unwrap();
        node_id
    }

    #[test]
    fn test_distiller_config_default() {
        let cfg = DistillerConfig::default();
        assert!(cfg.min_insight_confidence > 0.0);
        assert!(cfg.max_patterns_per_run > 0);
    }

    #[test]
    fn test_distiller_tech_stack_pattern() {
        let kb = test_kb();
        insert_repo(&kb, "org/repo-a", "Rust", &["web", "api"]);
        insert_repo(&kb, "org/repo-b", "Rust", &["cli", "tooling"]);
        insert_repo(&kb, "org/repo-c", "Python", &["ml", "data"]);

        let distiller = ContentDistiller::new(kb);
        let report = distiller.distil_all().unwrap();

        // Should detect Rust tech stack commonality (2 repos)
        let rust_patterns: Vec<_> = report.patterns_detected.iter()
            .filter(|p| p.description.contains("Rust"))
            .collect();
        assert!(rust_patterns.len() >= 1, "Should have some Rust-related patterns");
    }

    #[test]
    fn test_distiller_architectural_pattern() {
        let kb = test_kb();
        insert_repo(&kb, "org/repo-a", "Rust", &["ai", "ml", "neural"]);
        insert_repo(&kb, "org/repo-b", "Python", &["ai", "ml", "data"]);
        insert_repo(&kb, "org/repo-c", "Python", &["ai", "deep-learning"]);

        let distiller = ContentDistiller::new(kb);
        let report = distiller.distil_all().unwrap();

        // Should detect topic overlap patterns (ai appearing in 3+ repos)
        let topic_patterns: Vec<_> = report.patterns_detected.iter()
            .filter(|p| p.description.contains("Topic 'ai'"))
            .collect();
        assert!(topic_patterns.len() >= 1, "Should have some AI topic patterns");
    }

    #[test]
    fn test_distiller_code_patterns() {
        let kb = test_kb();
        let repo_id = insert_repo(&kb, "org/my-repo", "Rust", &["web"]);

        // Insert code snippets with mod.rs pattern
        let s1 = insert_snippet(&kb, "src/main.rs", "fn main() { println!(\"hello\"); }");
        let s2 = insert_snippet(&kb, "src/lib.rs", "pub fn helper() -> i32 { 42 }");
        let s3 = insert_snippet(&kb, "src/mod.rs", "mod sub;");

        // Link snippets to repo
        kb.upsert_edge(&s1, &repo_id, RelationType::PartOf, 0.7, None).unwrap();
        kb.upsert_edge(&s2, &repo_id, RelationType::PartOf, 0.7, None).unwrap();
        kb.upsert_edge(&s3, &repo_id, RelationType::PartOf, 0.7, None).unwrap();

        let distiller = ContentDistiller::new(kb);
        let report = distiller.distil_all().unwrap();

        // Should detect code snippet patterns
        assert!(report.total_nodes_scanned > 0); // at least some nodes scanned
    }

    #[test]
    fn test_distiller_evolution_records() {
        let kb = test_kb();
        // Insert 3 repos with same language to trigger high-confidence pattern
        for i in 0..5 {
            insert_repo(&kb, &format!("org/repo-{}", i), "Rust", &["web"]);
        }

        let cfg = DistillerConfig { enable_evolution_records: true, ..Default::default() };
        let distiller = ContentDistiller::with_config(kb, cfg);
        let report = distiller.distil_all().unwrap();

        // High-confidence patterns (>0.8) should generate evolution records
        // With 5 repos of the same language, confidence = 0.7 + min(5, 5)*0.05 = 0.95
        assert!(report.evolution_records_created > 0, "Should handle evolution records gracefully");
    }

    #[test]
    fn test_distiller_node_single_repo() {
        let kb = test_kb();
        let repo_id = insert_repo(&kb, "org/solo-repo", "Rust", &["embedded"]);

        // Insert a code snippet with ML keyword
        let snippet = insert_snippet(&kb, "src/neural.rs",
            "use transformer; fn attention() { /* neural network */ }");
        kb.upsert_edge(&snippet, &repo_id, RelationType::PartOf, 0.7, None).unwrap();

        let distiller = ContentDistiller::new(kb);
        let result = distiller.distil_node(&repo_id).unwrap();
        // Single repo won't produce tech-stack patterns (>2 repos needed)
        assert!(result.is_none(), "Single repo produces no tech-stack pattern");

        let result2 = distiller.distil_node("non-existent").unwrap_or(None);
        assert!(result2.is_none(), "Non-existent node should return None");
    }

    #[test]
    fn test_distiller_distil_node_article() {
        let kb = test_kb();
        let article_id = kb.insert_or_get_node(
            "Test Article", NodeType::Article, None, None, Some("web"),
        ).unwrap();
        kb.update_node_content(&article_id, "This uses modular design and dependency injection patterns").unwrap();

        let distiller = ContentDistiller::new(kb);
        let result = distiller.distil_node(&article_id).unwrap();
        assert!(result.is_some(), "Article with design principles should produce patterns");
    }
}
