// ─── Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RelevanceSignals {
    pub direct_link: f64,
    pub source_overlap: f64,
    pub adamic_adar: f64,
    pub type_affinity: f64,
    pub combined: f64,
}

#[derive(Debug, Clone)]
pub struct Community {
    pub id: String,
    pub node_ids: Vec<String>,
    pub size: usize,
    pub internal_edges: usize,
    pub cohesion: f64,
    pub dominant_types: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct SurprisingConnection {
    pub source_id: String,
    pub source_title: String,
    pub target_id: String,
    pub target_title: String,
    pub edge_id: String,
    pub weight: f64,
    pub source_community: String,
    pub target_community: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub node_id: String,
    pub node_title: String,
    pub node_type: String,
    pub edge_count: usize,
    pub isolation_score: f64,
}

#[derive(Debug, Clone)]
pub struct BridgeNode {
    pub node_id: String,
    pub node_title: String,
    pub node_type: String,
    pub degree: usize,
    pub community_count: usize,
    pub bridge_score: f64,
}

// ─── Temporal Versioning Types ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TemporalFact {
    pub fact_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TemporalQuery {
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
    pub as_of: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::RelevanceSignals;

    #[test]
    fn test_relevance_signals_sorting() {
        let a = RelevanceSignals {
            direct_link: 6.0,
            source_overlap: 0.0,
            adamic_adar: 0.0,
            type_affinity: 0.0,
            combined: 6.0,
        };
        let b = RelevanceSignals {
            direct_link: 0.0,
            source_overlap: 0.0,
            adamic_adar: 1.5,
            type_affinity: 1.0,
            combined: 2.5,
        };
        assert!(a.combined > b.combined);
    }
}
