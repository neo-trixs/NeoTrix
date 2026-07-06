use serde::{Deserialize, Serialize};

/// Source tier: how primary/reliable the information source is
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum SourceTier {
    /// Primary source: direct observation, original document, first-hand testimony
    Primary = 5,
    /// Secondary source: analysis/citation of primary sources by qualified authority
    Secondary = 4,
    /// Tertiary source: compiled/encyclopedic reference
    Tertiary = 3,
    /// Hearsay: reported by someone who heard from someone else
    Hearsay = 2,
    /// Anonymous: unknown or unverifiable origin
    Anonymous = 1,
}

impl SourceTier {
    pub fn label(&self) -> &str {
        match self {
            SourceTier::Primary => "一手资料",
            SourceTier::Secondary => "二手资料",
            SourceTier::Tertiary => "三手汇编",
            SourceTier::Hearsay => "传闻",
            SourceTier::Anonymous => "匿名",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            SourceTier::Primary => 0.95,
            SourceTier::Secondary => 0.75,
            SourceTier::Tertiary => 0.50,
            SourceTier::Hearsay => 0.25,
            SourceTier::Anonymous => 0.10,
        }
    }
}

/// Peer review status for a publication
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    PeerReviewed,
    Preprint,
    ConferenceProceedings,
    SelfPublished,
    Unreviewed,
}

impl ReviewStatus {
    pub fn weight(&self) -> f64 {
        match self {
            ReviewStatus::PeerReviewed => 0.90,
            ReviewStatus::Preprint => 0.60,
            ReviewStatus::ConferenceProceedings => 0.70,
            ReviewStatus::SelfPublished => 0.30,
            ReviewStatus::Unreviewed => 0.15,
        }
    }
}

/// Chain of custody: how many transfers the evidence has gone through
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CustodyChain {
    pub depth: usize,
    pub all_documented: bool,
    pub gaps: usize,
}

impl CustodyChain {
    pub fn new(depth: usize, all_documented: bool, gaps: usize) -> Self {
        Self { depth, all_documented, gaps }
    }

    pub fn integrity_score(&self) -> f64 {
        let base = if self.all_documented { 0.95 } else { 0.50 };
        let gap_penalty = (self.gaps as f64 * 0.15).min(0.5);
        let depth_factor = (1.0 / (self.depth as f64).max(1.0)).max(0.5);
        (base - gap_penalty) * depth_factor
    }
}

/// Structured credibility assessment for a single evidence source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCredibility {
    pub source_tier: SourceTier,
    pub review_status: ReviewStatus,
    pub author_reputation: f64,
    pub institutional_backing: f64,
    pub citation_count: u64,
    pub temporal_proximity: f64,
    pub custody_chain: Option<CustodyChain>,
    pub independence_score: f64,
    pub cross_validation_count: u32,
}

impl Default for SourceCredibility {
    fn default() -> Self {
        Self {
            source_tier: SourceTier::Secondary,
            review_status: ReviewStatus::Unreviewed,
            author_reputation: 0.5,
            institutional_backing: 0.5,
            citation_count: 0,
            temporal_proximity: 0.5,
            custody_chain: None,
            independence_score: 0.5,
            cross_validation_count: 0,
        }
    }
}

impl SourceCredibility {
    pub fn new(tier: SourceTier) -> Self {
        Self { source_tier: tier, ..Default::default() }
    }

    pub fn overall_score(&self) -> f64 {
        let tier_w = self.source_tier.weight();
        let review_w = self.review_status.weight();
        let author_w = self.author_reputation;
        let inst_w = self.institutional_backing;
        let citation_factor = (self.citation_count as f64 * 0.02).min(0.2);
        let custody_w = self.custody_chain
            .map(|c| c.integrity_score())
            .unwrap_or(0.3);
        let cross_w = (self.cross_validation_count as f64 * 0.05).min(0.3);
        let composite = 0.35 * tier_w
            + 0.15 * review_w
            + 0.15 * author_w
            + 0.10 * inst_w
            + 0.08 * citation_factor
            + 0.10 * custody_w
            + 0.05 * self.temporal_proximity
            + 0.05 * self.independence_score
            + 0.05 * cross_w;
        composite.max(0.05).min(1.0)
    }

    pub fn credibility_tier(&self) -> &str {
        let s = self.overall_score();
        if s >= 0.85 { "T1 高可信" }
        else if s >= 0.65 { "T2 可信" }
        else if s >= 0.45 { "T3 中等" }
        else if s >= 0.25 { "T4 低可信" }
        else { "T5 不可信" }
    }

    /// Bayesian Truth Serum (BTS) signal: how "surprisingly common" this evidence is
    /// Higher score = more truthful (per Prelec 2004)
    pub fn bts_signal(empirical_freq: f64, predicted_freq: f64) -> f64 {
        if empirical_freq <= 0.0 || predicted_freq <= 0.0 {
            return 0.0;
        }
        (empirical_freq / predicted_freq).ln()
    }
}

/// Multi-source aggregation of credibility scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredibilityAggregator {
    pub scores: Vec<SourceCredibility>,
}

impl Default for CredibilityAggregator {
    fn default() -> Self { Self::new() }
}

impl CredibilityAggregator {
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

    pub fn add(&mut self, cred: SourceCredibility) {
        self.scores.push(cred);
    }

    pub fn aggregate_weighted(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        let mut total = 0.0;
        let mut weight_sum = 0.0;
        for s in &self.scores {
            let w = s.overall_score();
            total += w * w;
            weight_sum += w;
        }
        if weight_sum > 0.0 { total / weight_sum } else { 0.0 }
    }

    /// Geometric mean as adversarial suppression: one low score collapses the average
    pub fn aggregate_geometric(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        let n = self.scores.len() as f64;
        let product: f64 = self.scores.iter()
            .map(|s| s.overall_score().max(0.01))
            .product();
        product.powf(1.0 / n)
    }

    pub fn diversity_score(&self) -> f64 {
        if self.scores.len() < 2 {
            return 0.0;
        }
        let tiers: Vec<u8> = self.scores.iter()
            .map(|s| match s.source_tier {
                SourceTier::Primary => 5,
                SourceTier::Secondary => 4,
                SourceTier::Tertiary => 3,
                SourceTier::Hearsay => 2,
                SourceTier::Anonymous => 1,
            })
            .collect();
        let unique = tiers.iter().collect::<std::collections::HashSet<&u8>>().len();
        unique as f64 / 5.0
    }

    /// Modified PageRank over evidence sources: sources cited by credible sources gain credibility
    pub fn propagate_trust(&self, citation_graph: &[(usize, usize)]) -> Vec<f64> {
        let n = self.scores.len();
        if n == 0 {
            return vec![];
        }
        let damp = 0.85;
        let mut ranks = vec![1.0 / n as f64; n];
        for _ in 0..20 {
            let mut new_ranks = vec![(1.0 - damp) / n as f64; n];
            let mut out_degree = vec![0usize; n];
            for &(from, _) in citation_graph {
                if from < n { out_degree[from] += 1; }
            }
            for &(from, to) in citation_graph {
                if from < n && to < n && out_degree[from] > 0 {
                    new_ranks[to] += damp * ranks[from] / out_degree[from] as f64;
                }
            }
            ranks = new_ranks;
        }
        ranks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_tier_weight_ordering() {
        assert!(SourceTier::Primary.weight() > SourceTier::Secondary.weight());
        assert!(SourceTier::Secondary.weight() > SourceTier::Hearsay.weight());
    }

    #[test]
    fn test_source_credibility_primary_high_score() {
        let cred = SourceCredibility {
            source_tier: SourceTier::Primary,
            review_status: ReviewStatus::PeerReviewed,
            author_reputation: 0.9,
            institutional_backing: 0.9,
            citation_count: 100,
            temporal_proximity: 0.9,
            custody_chain: Some(CustodyChain::new(1, true, 0)),
            independence_score: 0.9,
            cross_validation_count: 10,
        };
        let score = cred.overall_score();
        assert!(score > 0.85);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_source_credibility_anonymous_low_score() {
        let cred = SourceCredibility {
            source_tier: SourceTier::Anonymous,
            review_status: ReviewStatus::Unreviewed,
            author_reputation: 0.1,
            institutional_backing: 0.1,
            citation_count: 0,
            temporal_proximity: 0.1,
            custody_chain: None,
            independence_score: 0.1,
            cross_validation_count: 0,
        };
        let score = cred.overall_score();
        assert!(score < 0.3);
    }

    #[test]
    fn test_custody_chain_integrity() {
        let good = CustodyChain::new(1, true, 0);
        let bad = CustodyChain::new(5, false, 3);
        assert!(good.integrity_score() > bad.integrity_score());
    }

    #[test]
    fn test_credibility_aggregator_weighted() {
        let mut agg = CredibilityAggregator::new();
        agg.add(SourceCredibility {
            source_tier: SourceTier::Primary,
            review_status: ReviewStatus::PeerReviewed,
            author_reputation: 0.9,
            institutional_backing: 0.9,
            citation_count: 100,
            temporal_proximity: 0.9,
            custody_chain: Some(CustodyChain::new(1, true, 0)),
            independence_score: 0.9,
            cross_validation_count: 10,
        });
        agg.add(SourceCredibility {
            source_tier: SourceTier::Secondary,
            review_status: ReviewStatus::Preprint,
            author_reputation: 0.6,
            institutional_backing: 0.7,
            citation_count: 20,
            temporal_proximity: 0.7,
            custody_chain: Some(CustodyChain::new(2, true, 0)),
            independence_score: 0.7,
            cross_validation_count: 3,
        });
        let agg_score = agg.aggregate_weighted();
        assert!(agg_score > 0.5);
        assert!(agg_score <= 1.0);
    }

    #[test]
    fn test_geometric_mean_penalizes_low_scores() {
        let mut agg = CredibilityAggregator::new();
        agg.add(SourceCredibility {
            source_tier: SourceTier::Primary,
            review_status: ReviewStatus::PeerReviewed,
            ..Default::default()
        });
        agg.add(SourceCredibility {
            source_tier: SourceTier::Anonymous,
            review_status: ReviewStatus::Unreviewed,
            ..Default::default()
        });
        // Set explicit scores through fields
        agg.scores[0].author_reputation = 0.9;
        agg.scores[0].institutional_backing = 0.9;
        agg.scores[0].cross_validation_count = 10;
        agg.scores[1].author_reputation = 0.05;
        agg.scores[1].institutional_backing = 0.05;
        let arithmetic = agg.aggregate_weighted();
        let geometric = agg.aggregate_geometric();
        assert!(geometric <= arithmetic);
    }

    #[test]
    fn test_diversity_score() {
        let mut agg = CredibilityAggregator::new();
        agg.add(SourceCredibility::new(SourceTier::Primary));
        agg.add(SourceCredibility::new(SourceTier::Secondary));
        agg.add(SourceCredibility::new(SourceTier::Hearsay));
        assert!(agg.diversity_score() > 0.3);
    }

    #[test]
    fn test_bts_signal_positive() {
        let signal = SourceCredibility::bts_signal(0.8, 0.3);
        assert!(signal > 0.0);
    }

    #[test]
    fn test_bts_signal_negative() {
        let signal = SourceCredibility::bts_signal(0.2, 0.7);
        assert!(signal < 0.0);
    }

    #[test]
    fn test_trust_propagation() {
        let mut agg = CredibilityAggregator::new();
        agg.add(SourceCredibility::new(SourceTier::Primary));
        agg.add(SourceCredibility::new(SourceTier::Secondary));
        agg.add(SourceCredibility::new(SourceTier::Tertiary));
        let graph = vec![(0, 1), (1, 2)];
        let ranks = agg.propagate_trust(&graph);
        assert_eq!(ranks.len(), 3);
        assert!(ranks[0] > 0.0);
    }
}
