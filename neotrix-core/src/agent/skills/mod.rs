pub mod types;
pub mod registry;
pub mod execution;

pub use types::*;
pub use registry::*;
pub use execution::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_source_variants() {
        let local = SkillSource::LocalDir("./skills".into());
        match local { SkillSource::LocalDir(_) => {}, _ => panic!("wrong variant") }
        let gh = SkillSource::GitHub { owner: "user".into(), repo: "repo".into(), path: ".".into(), branch: None };
        match gh { SkillSource::GitHub { .. } => {}, _ => panic!("wrong variant") }
    }

    #[test]
    fn test_skill_stats_default_confidence() {
        let stats = SkillStats {
            use_count: 0, success_count: 0, confidence: 0.5,
            avg_execution_ms: 0.0, last_used: None, evolution_history: vec![],
        };
        assert!((stats.confidence - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_skill_discovery_new() {
        let disc = SkillDiscovery::new();
        assert!(!disc.search_paths.is_empty());
    }

    #[test]
    fn test_skill_discovery_add_search_path() {
        let mut disc = SkillDiscovery::new();
        let before = disc.search_paths.len();
        disc.add_search_path("/tmp/skills");
        assert_eq!(disc.search_paths.len(), before + 1);
    }

    #[test]
    fn test_skill_meta_default_fields() {
        let meta = SkillMeta {
            name: "test".into(), description: "desc".into(), version: "1.0".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        assert_eq!(meta.name, "test");
        assert!(meta.triggers.is_empty());
    }
}
