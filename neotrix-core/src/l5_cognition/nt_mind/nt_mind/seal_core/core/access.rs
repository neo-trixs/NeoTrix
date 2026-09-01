use serde::{Deserialize, Serialize};
use super::knowledge_source::KnowledgeSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    pub identity: String,
    pub trust_score: f64,
    pub task_type: Option<String>,
    pub max_sources: usize,
    pub tags: Vec<String>,
}

impl AccessContext {
    pub fn new(identity: &str, trust_score: f64) -> Self {
        Self {
            identity: identity.to_string(),
            trust_score,
            task_type: None,
            max_sources: 5,
            tags: Vec::new(),
        }
    }

    pub fn with_task_type(mut self, task_type: &str) -> Self {
        self.task_type = Some(task_type.to_string());
        self
    }
}

impl Default for AccessContext {
    fn default() -> Self {
        Self {
            identity: "system".to_string(),
            trust_score: 1.0,
            task_type: None,
            max_sources: 5,
            tags: Vec::new(),
        }
    }
}

pub fn route_sources_by_context<'a>(
    sources: &'a [KnowledgeSource],
    context: &AccessContext,
    _task_type: Option<&str>,
) -> Vec<&'a KnowledgeSource> {
    let mut filtered: Vec<&KnowledgeSource> = Vec::new();

    for source in sources.iter() {
        match context.trust_score {
            ts if ts < 0.3 => {
                if matches!(source, KnowledgeSource::DesignPhilosophy) {
                    filtered.push(source);
                }
            }
            ts if ts < 0.7 => {
                if matches!(source, KnowledgeSource::BaseUI | KnowledgeSource::DesignPhilosophy) {
                    filtered.push(source);
                }
            }
            _ => {
                filtered.push(source);
            }
        }
    }

    filtered.truncate(context.max_sources);
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_context_new() {
        let ctx = AccessContext::new("alice", 0.8);
        assert_eq!(ctx.identity, "alice");
        assert!((ctx.trust_score - 0.8).abs() < 1e-9);
        assert_eq!(ctx.max_sources, 5);
        assert!(ctx.task_type.is_none());
    }

    #[test]
    fn test_access_context_default() {
        let ctx = AccessContext::default();
        assert_eq!(ctx.identity, "system");
        assert!((ctx.trust_score - 1.0).abs() < 1e-9);
        assert!(ctx.task_type.is_none());
    }

    #[test]
    fn test_access_context_with_task_type() {
        let ctx = AccessContext::new("bob", 0.5).with_task_type("code_review");
        assert_eq!(ctx.task_type, Some("code_review".to_string()));
    }

    #[test]
    fn test_route_sources_low_trust_only_design_philosophy() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::DesignPhilosophy,
            KnowledgeSource::BaseUI,
        ];
        let ctx = AccessContext::new("untrusted", 0.2);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 1);
        assert_eq!(*routed[0], KnowledgeSource::DesignPhilosophy);
    }

    #[test]
    fn test_route_sources_medium_trust_base_ui_and_design_philosophy() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::DesignPhilosophy,
            KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI,
        ];
        let ctx = AccessContext::new("semi_trusted", 0.5);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 2);
        assert!(routed.contains(&&KnowledgeSource::DesignPhilosophy));
        assert!(routed.contains(&&KnowledgeSource::BaseUI));
    }

    #[test]
    fn test_route_sources_high_trust_all_sources() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI,
            KnowledgeSource::CortexUI,
        ];
        let ctx = AccessContext::new("trusted", 0.9);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 4);
    }

    #[test]
    fn test_route_sources_truncates_to_max_sources() {
        let sources = vec![
            KnowledgeSource::HeroUI, KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI, KnowledgeSource::CortexUI,
            KnowledgeSource::AgenticDS, KnowledgeSource::DesignPhilosophy,
            KnowledgeSource::Hyperframes,
        ];
        let ctx = AccessContext { max_sources: 3, ..AccessContext::default() };
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 3);
    }

    #[test]
    fn test_route_sources_empty_sources() {
        let sources: Vec<KnowledgeSource> = vec![];
        let ctx = AccessContext::default();
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert!(routed.is_empty());
    }

    #[test]
    fn test_route_sources_low_trust_with_no_design_philosophy() {
        let sources = vec![KnowledgeSource::HeroUI, KnowledgeSource::BaseUI];
        let ctx = AccessContext::new("untrusted", 0.2);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert!(routed.is_empty());
    }

    #[test]
    fn test_route_sources_boundary_trust_0_3() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::DesignPhilosophy,
        ];
        let ctx = AccessContext::new("boundary", 0.3);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 1);
        assert_eq!(*routed[0], KnowledgeSource::DesignPhilosophy);
    }

    #[test]
    fn test_route_sources_boundary_trust_0_7() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
            KnowledgeSource::DesignPhilosophy,
        ];
        let ctx = AccessContext::new("boundary", 0.7);
        let routed = route_sources_by_context(&sources, &ctx, None);
        assert_eq!(routed.len(), 3);
    }
}
