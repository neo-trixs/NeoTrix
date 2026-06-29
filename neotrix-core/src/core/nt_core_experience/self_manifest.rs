/// SelfManifestGenerator — 结构化自模型清单
///
/// 与 SelfModelGenerator (markdown) 互补，输出 YAML 格式的结构化自模型。
/// 内容来自：
/// - TokenRegistry (设计标记系统)
/// - KnowledgeGraph (知识节点)
/// - 运行时统计 (执行次数 / 接线状态 / 健康度)
///
/// 输出到 `~/.neotrix/self-manifest.yaml`，供元层自省和外部工具消费。

use std::fs;
use std::path::PathBuf;

use super::design_token::TokenRegistry;
use super::knowledge_node::KnowledgeGraph;

/// 结构化自模型清单。
#[derive(Debug, Clone)]
pub struct SelfManifest {
    /// 架构版本
    pub architecture_version: String,
    /// 构建时间戳
    pub build: String,
    /// 平均健康度
    pub average_health: f64,
    /// 接线率
    pub wiring_rate: f64,
    /// 基元数量
    pub primitive_count: usize,
    /// 语义数量
    pub semantic_count: usize,
    /// 组件数量
    pub component_count: usize,
    /// 原则数量
    pub principle_count: usize,
    /// 模式数量
    pub pattern_count: usize,
    /// 决策数量
    pub decision_count: usize,
    /// 反模式数量
    pub antipattern_count: usize,
    /// 整体自评分
    pub self_score: f64,
    /// 最需要改进的三个方面
    pub top_improvements: Vec<String>,
}

/// 结构化清单生成器。
#[derive(Debug, Clone)]
pub struct SelfManifestGenerator {
    pub output_path: PathBuf,
    pub interval: u64,
    pub last_generated: u64,
    pub generation_count: u64,
    pub last_manifest: String,
}

impl Default for SelfManifestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfManifestGenerator {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let path = PathBuf::from(home).join(".neotrix").join("self-manifest.yaml");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        Self {
            output_path: path,
            interval: 50,
            last_generated: 0,
            generation_count: 0,
            last_manifest: String::new(),
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = path;
        self
    }

    /// 从设计标记和知识图谱生成结构化清单。
    pub fn generate(
        &mut self,
        cycle: u64,
        tokens: Option<&TokenRegistry>,
        knowledge: Option<&KnowledgeGraph>,
    ) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push("---".into());
        lines.push(format!("manifest_version: {}", self.generation_count));
        lines.push(format!("generated_at_cycle: {}", cycle));
        lines.push(String::new());

        // ── 设计标记摘要 ──
        lines.push("design_tokens:".into());
        if let Some(t) = tokens {
            lines.push(format!("  primitives: {}", t.primitives.len()));
            lines.push(format!("  semantics: {}", t.semantics.len()));
            lines.push(format!("  components: {}", t.components.len()));
            lines.push(format!("  average_health: {:.4}", t.average_health()));
            lines.push(format!("  wiring_rate: {:.4}", t.wiring_rate()));

            // 按域分组的基元
            lines.push("  domains:".into());
            let by_domain = t.primitives_by_domain();
            let mut domain_keys: Vec<_> = by_domain.keys().collect();
            domain_keys.sort_by_key(|d| d.name());
            for dom in domain_keys {
                if let Some(tokens_in_domain) = by_domain.get(dom) {
                    lines.push(format!("    {}:", dom.name()));
                    for p in tokens_in_domain {
                        lines.push(format!("      - {}", p.name()));
                    }
                }
            }

            // 组件状态
            lines.push("  components:".into());
            for comp in &t.components {
                lines.push(format!("    - name: \"{}\"", comp.name));
                lines.push(format!("      health: {:.4}", comp.health));
                lines.push(format!("      status: {}", comp.wiring_status.name()));
            }

            // 低健康度语义
            lines.push("  improvements:".into());
            let mut low_health: Vec<_> = t.semantics.iter().filter(|s| s.health < 0.4).collect();
            low_health.sort_by(|a, b| a.health.partial_cmp(&b.health).unwrap_or(std::cmp::Ordering::Equal));
            for s in low_health.iter().take(5) {
                lines.push(format!("    - primitive: \"{}\"", s.primitive.name()));
                lines.push(format!("      health: {:.4}", s.health));
                lines.push(format!("      intent: \"{}\"", s.intent));
            }
        } else {
            lines.push("  (unavailable)".into());
        }
        lines.push(String::new());

        // ── 知识图谱摘要 ──
        lines.push("knowledge_graph:".into());
        if let Some(kg) = knowledge {
            lines.push(format!("  total_nodes: {}", kg.nodes.len()));
            lines.push(format!("  total_edges: {}", kg.edges.len()));
            let principles = kg.find_by_type(super::knowledge_node::NodeType::Principle);
            let patterns = kg.find_by_type(super::knowledge_node::NodeType::Pattern);
            let decisions = kg.find_by_type(super::knowledge_node::NodeType::Decision);
            let antipatterns = kg.find_by_type(super::knowledge_node::NodeType::Antipattern);
            lines.push(format!("  principles: {}", principles.len()));
            lines.push(format!("  patterns: {}", patterns.len()));
            lines.push(format!("  decisions: {}", decisions.len()));
            lines.push(format!("  antipatterns: {}", antipatterns.len()));

            // 高置信度原则
            let high_principles = kg.high_confidence_principles();
            if !high_principles.is_empty() {
                lines.push("  high_confidence_principles:".into());
                for p in &high_principles {
                    lines.push(format!("    - title: \"{}\"", p.title));
                    lines.push(format!("      confidence: {:.4}", p.confidence));
                    lines.push(format!("      evidence: {}", p.evidence_count));
                }
            }
        } else {
            lines.push("  (unavailable)".into());
        }
        lines.push(String::new());

        // ── 自评分 ──
        let health = tokens.map(|t| t.average_health()).unwrap_or(0.0);
        let wired = tokens.map(|t| t.wiring_rate()).unwrap_or(0.0);
        let principle_count = knowledge
            .map(|kg| kg.find_by_type(super::knowledge_node::NodeType::Principle).len())
            .unwrap_or(0);
        let self_score = health * 0.4 + wired * 0.3 + (principle_count as f64).min(10.0) / 10.0 * 0.3;

        lines.push("self_score:".into());
        lines.push(format!("  overall: {:.4}", self_score));
        lines.push(format!("  health_dimension: {:.4}", health));
        lines.push(format!("  wiring_dimension: {:.4}", wired));
        lines.push(format!("  knowledge_dimension: {:.4}", (principle_count as f64).min(10.0) / 10.0));
        lines.push(String::new());

        // ── 最需改进项 ──
        lines.push("top_improvements:".into());
        if let Some(t) = tokens {
            let mut low_components: Vec<_> = t.components.iter().filter(|c| c.health < 0.6).collect();
            low_components.sort_by(|a, b| a.health.partial_cmp(&b.health).unwrap_or(std::cmp::Ordering::Equal));
            for c in low_components.iter().take(3) {
                lines.push(format!("  - component: \"{}\"", c.name));
                lines.push(format!("    health: {:.4}", c.health));
                lines.push(format!("    status: {}", c.wiring_status.name()));
            }
        }
        if let Some(kg) = knowledge {
            let ap = kg.find_by_type(super::knowledge_node::NodeType::Antipattern);
            for a in ap.iter().take(2) {
                lines.push(format!("  - antipattern: \"{}\"", a.title));
                lines.push(format!("    confidence: {:.4}", a.confidence));
            }
        }
        lines.push("...".into());

        let manifest = lines.join("\n");
        self.last_manifest = manifest.clone();
        self.last_generated = cycle;
        self.generation_count += 1;

        if let Err(e) = fs::write(&self.output_path, &manifest) {
            log::warn!("self_manifest: write failed: {}", e);
        } else {
            log::info!(
                "self_manifest: written {} bytes to {} (generation #{})",
                manifest.len(),
                self.output_path.display(),
                self.generation_count,
            );
        }

        manifest
    }

    pub fn should_generate(&self, cycle: u64) -> bool {
        cycle > 0 && cycle >= self.last_generated + self.interval
    }

    pub fn last_manifest(&self) -> &str {
        &self.last_manifest
    }

    pub fn stats(&self) -> String {
        format!(
            "self_manifest: gen={} last_cycle={} interval={} path={}",
            self.generation_count,
            self.last_generated,
            self.interval,
            self.output_path.display(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::design_token::TokenRegistry;

    #[test]
    fn test_generate_with_tokens() {
        let mut gen = SelfManifestGenerator::new();
        let tokens = TokenRegistry::new();
        let kg = KnowledgeGraph::new();
        let result = gen.generate(1, Some(&tokens), Some(&kg));
        assert!(result.contains("design_tokens:"));
        assert!(result.contains("primitives:"));
        assert!(result.contains("self_score:"));
        assert!(result.contains("top_improvements:"));
    }

    #[test]
    fn test_generate_without_tokens() {
        let mut gen = SelfManifestGenerator::new();
        let result = gen.generate(1, None, None);
        assert!(result.contains("design_tokens:"));
        assert!(result.contains("(unavailable)"));
    }

    #[test]
    fn test_should_generate_interval() {
        let gen = SelfManifestGenerator::new();
        assert!(gen.should_generate(50));  // first time
        assert!(gen.should_generate(100)); // not generated yet
    }

    #[test]
    fn test_stats_output() {
        let gen = SelfManifestGenerator::new();
        assert!(gen.stats().contains("self_manifest:"));
    }
}
