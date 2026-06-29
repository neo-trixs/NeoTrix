/// CrossRepoMapper — 跨仓库概念映射器
///
/// 当 ERLHeuristicPool 从不同 repo 或 domain 检索到启发式时,
/// 可能概念命名不一致 (如 "recalibrate" vs "calibration_cycle")。
/// CrossRepoMapper 维护跨仓库术语对齐表, 实现概念级别的通用性检测。

#[derive(Debug, Clone)]
pub struct ConceptMapping {
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub domain: String,
}

impl ConceptMapping {
    pub fn matches(&self, term: &str) -> bool {
        let lower = term.to_lowercase();
        self.canonical_name.to_lowercase() == lower
            || self.aliases.iter().any(|a| a.to_lowercase() == lower)
    }
}

#[derive(Debug, Clone)]
pub struct CrossRepoMapper {
    mappings: Vec<ConceptMapping>,
}

impl CrossRepoMapper {
    pub fn new() -> Self {
        // Pre-populate with common NeoTrix concept mappings
        let mut mapper = Self { mappings: Vec::new() };
        mapper.add_mapping("calibration", vec!["recalibrate", "calibration_cycle", "ece_drift"], "calibration");
        mapper.add_mapping("memory_consolidation", vec!["dream_cycle", "sleep_consolidation", "consolidation_pipeline"], "memory");
        mapper.add_mapping("curiosity_drive", vec!["exploration_bonus", "novelty_seeking", "information_gain"], "motivation");
        mapper.add_mapping("self_evolution", vec!["self_modify", "meta_evolution", "rsi_cycle"], "evolution");
        mapper.add_mapping("wiring", vec!["module_registration", "subsystem_connect", "pipe_wiring"], "architecture");
        mapper.add_mapping("heuristic", vec!["rule", "pattern", "guideline", "best_practice"], "knowledge");
        mapper.add_mapping("trace_analysis", vec!["weakness_mining", "reflection", "post_mortem"], "meta_cognition");
        mapper.add_mapping("anomaly_detection", vec!["outlier_detection", "spike_detection", "drift_detection"], "monitoring");
        mapper
    }

    pub fn add_mapping(&mut self, canonical: &str, aliases: Vec<&str>, domain: &str) {
        self.mappings.push(ConceptMapping {
            canonical_name: canonical.to_string(),
            aliases: aliases.into_iter().map(|a| a.to_string()).collect(),
            domain: domain.to_string(),
        });
    }

    /// 查找规范名
    pub fn resolve(&self, term: &str) -> Option<&str> {
        for m in &self.mappings {
            if m.matches(term) {
                return Some(&m.canonical_name);
            }
        }
        None
    }

    /// 检查两个术语是否指向同一概念
    pub fn are_equivalent(&self, term_a: &str, term_b: &str) -> bool {
        let resolved_a = self.resolve(term_a);
        let resolved_b = self.resolve(term_b);
        match (resolved_a, resolved_b) {
            (Some(a), Some(b)) => a == b,
            (None, None) => term_a.to_lowercase() == term_b.to_lowercase(),
            _ => false,
        }
    }

    /// 获取映射统计
    pub fn mapping_count(&self) -> usize { self.mappings.len() }

    pub fn summary(&self) -> String {
        format!("CrossRepoMapper: {} concept mappings", self.mappings.len())
    }
}

impl Default for CrossRepoMapper {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_canonical() {
        let mapper = CrossRepoMapper::new();
        assert_eq!(mapper.resolve("recalibrate"), Some("calibration"));
    }

    #[test]
    fn test_resolve_self() {
        let mapper = CrossRepoMapper::new();
        assert_eq!(mapper.resolve("calibration"), Some("calibration"));
    }

    #[test]
    fn test_resolve_unknown() {
        let mapper = CrossRepoMapper::new();
        assert!(mapper.resolve("nonexistent_term").is_none());
    }

    #[test]
    fn test_are_equivalent_positive() {
        let mapper = CrossRepoMapper::new();
        assert!(mapper.are_equivalent("recalibrate", "calibration_cycle"));
    }

    #[test]
    fn test_are_equivalent_negative() {
        let mapper = CrossRepoMapper::new();
        assert!(!mapper.are_equivalent("calibration", "curiosity_drive"));
    }

    #[test]
    fn test_are_equivalent_case_insensitive_fallback() {
        let mapper = CrossRepoMapper::new();
        assert!(mapper.are_equivalent("Hello", "hello"));
    }

    #[test]
    fn test_add_custom_mapping() {
        let mut mapper = CrossRepoMapper::new();
        mapper.add_mapping("custom", vec!["custom_alias"], "custom_domain");
        assert_eq!(mapper.resolve("custom_alias"), Some("custom"));
    }

    #[test]
    fn test_mapping_count() {
        let mapper = CrossRepoMapper::new();
        assert!(mapper.mapping_count() >= 8);
    }

    #[test]
    fn test_summary_format() {
        let mapper = CrossRepoMapper::new();
        let s = mapper.summary();
        assert!(s.contains("CrossRepoMapper"));
    }
}
