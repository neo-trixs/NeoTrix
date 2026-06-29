#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactTier {
    Tier1Experimental,    // Reproducible controlled experiments
    Tier2Observational,   // Multi-independent observer agreement
    Tier3Statistical,     // Statistical inference from data
    Tier4ExpertConsensus, // Independent expert consensus
    Tier5Institutional,   // Official institutional publication
    Tier6Anecdotal,       // Personal statement / hearsay
}

impl FactTier {
    pub fn label(&self) -> &'static str {
        match self {
            FactTier::Tier1Experimental => "T1:experimental",
            FactTier::Tier2Observational => "T2:observational",
            FactTier::Tier3Statistical => "T3:statistical",
            FactTier::Tier4ExpertConsensus => "T4:expert_consensus",
            FactTier::Tier5Institutional => "T5:institutional",
            FactTier::Tier6Anecdotal => "T6:anecdotal",
        }
    }

    pub fn confidence_base(&self) -> f64 {
        match self {
            FactTier::Tier1Experimental => 0.95,
            FactTier::Tier2Observational => 0.80,
            FactTier::Tier3Statistical => 0.65,
            FactTier::Tier4ExpertConsensus => 0.55,
            FactTier::Tier5Institutional => 0.40,
            FactTier::Tier6Anecdotal => 0.20,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FactTier::Tier1Experimental => {
                "Reproducible controlled experiment, p<0.01, replication>5"
            }
            FactTier::Tier2Observational => {
                "Multi-independent observer agreement, source independence>3"
            }
            FactTier::Tier3Statistical => {
                "Statistical inference: method transparency + adequate sample size"
            }
            FactTier::Tier4ExpertConsensus => {
                "Independent expert group consensus, COI audit available"
            }
            FactTier::Tier5Institutional => {
                "Official institutional publication + institutional bias audit needed"
            }
            FactTier::Tier6Anecdotal => {
                "Personal statement / hearsay — motivation + verifiability check needed"
            }
        }
    }

    pub fn all_tiers() -> &'static [FactTier; 6] {
        &[
            FactTier::Tier1Experimental,
            FactTier::Tier2Observational,
            FactTier::Tier3Statistical,
            FactTier::Tier4ExpertConsensus,
            FactTier::Tier5Institutional,
            FactTier::Tier6Anecdotal,
        ]
    }

    pub fn is_category_error(t1: FactTier, t2: FactTier) -> bool {
        let d = (t1 as i32 - t2 as i32).abs();
        d >= 3
    }

    fn downgrade(&self) -> FactTier {
        match self {
            FactTier::Tier1Experimental => FactTier::Tier2Observational,
            FactTier::Tier2Observational => FactTier::Tier3Statistical,
            FactTier::Tier3Statistical => FactTier::Tier4ExpertConsensus,
            FactTier::Tier4ExpertConsensus => FactTier::Tier5Institutional,
            FactTier::Tier5Institutional => FactTier::Tier6Anecdotal,
            FactTier::Tier6Anecdotal => FactTier::Tier6Anecdotal,
        }
    }

    fn upgrade(&self) -> FactTier {
        match self {
            FactTier::Tier1Experimental => FactTier::Tier1Experimental,
            FactTier::Tier2Observational => FactTier::Tier1Experimental,
            FactTier::Tier3Statistical => FactTier::Tier2Observational,
            FactTier::Tier4ExpertConsensus => FactTier::Tier3Statistical,
            FactTier::Tier5Institutional => FactTier::Tier4ExpertConsensus,
            FactTier::Tier6Anecdotal => FactTier::Tier5Institutional,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClaimTierAssignment {
    pub claim: String,
    pub assigned_tier: FactTier,
    pub confidence_adjustment: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct EvidenceChain {
    pub chain_depth: usize,
    pub direct_quotes: Vec<String>,
    pub independent_confirmations: usize,
    pub method_transparency: f64,
}

impl EvidenceChain {
    pub fn new() -> Self {
        Self {
            chain_depth: 0,
            direct_quotes: vec![],
            independent_confirmations: 0,
            method_transparency: 0.0,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.chain_depth = depth;
        self
    }

    pub fn with_confirmations(mut self, n: usize) -> Self {
        self.independent_confirmations = n;
        self
    }

    pub fn with_quotes(mut self, quotes: Vec<String>) -> Self {
        self.direct_quotes = quotes;
        self
    }

    pub fn with_transparency(mut self, t: f64) -> Self {
        self.method_transparency = t.clamp(0.0, 1.0);
        self
    }
}

impl Default for EvidenceChain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FactTierAnalyzer {
    pub tiers: Vec<FactTier>,
}

impl Default for FactTierAnalyzer {
    fn default() -> Self {
        Self {
            tiers: FactTier::all_tiers().to_vec(),
        }
    }
}

impl FactTierAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assign_tier(&self, claim: &str, source_description: &str) -> ClaimTierAssignment {
        let default_evidence = EvidenceChain::new();
        self.assign_tier_internal(claim, source_description, &default_evidence)
    }

    pub fn assign_tier_with_evidence(
        &self,
        claim: &str,
        source_description: &str,
        evidence: &EvidenceChain,
    ) -> ClaimTierAssignment {
        self.assign_tier_internal(claim, source_description, evidence)
    }

    fn assign_tier_internal(
        &self,
        claim: &str,
        source_description: &str,
        evidence: &EvidenceChain,
    ) -> ClaimTierAssignment {
        let lower = claim.to_lowercase();
        let source_lower = source_description.to_lowercase();

        let mut tier = if self.has_experimental_indicators(&lower, &source_lower) {
            FactTier::Tier1Experimental
        } else if self.has_observational_indicators(&lower, &source_lower) {
            FactTier::Tier2Observational
        } else if self.has_statistical_indicators(&lower) {
            FactTier::Tier3Statistical
        } else if self.has_expert_indicators(&source_lower) {
            FactTier::Tier4ExpertConsensus
        } else if self.has_institutional_indicators(&source_lower) {
            FactTier::Tier5Institutional
        } else {
            FactTier::Tier6Anecdotal
        };

        let credibility = self.assess_source_credibility(source_description);
        let mut adjustments: Vec<String> = Vec::new();

        if credibility < 0.3 {
            tier = tier.downgrade();
            adjustments.push(format!(
                "downgraded: source credibility {:.2} < 0.3",
                credibility
            ));
        }

        if evidence.chain_depth > 2 {
            tier = tier.downgrade();
            adjustments.push(format!(
                "downgraded: evidence chain depth {} > 2 (hearsay)",
                evidence.chain_depth
            ));
        }

        if evidence.independent_confirmations > 0 {
            tier = tier.upgrade();
            adjustments.push(format!(
                "upgraded: {} independent confirmation(s)",
                evidence.independent_confirmations
            ));
        }

        if evidence.method_transparency >= 0.7 {
            let before = tier;
            tier = tier.upgrade();
            if tier != before {
                adjustments.push(format!(
                    "upgraded: method transparency {:.2} >= 0.7",
                    evidence.method_transparency
                ));
            }
        }

        let confidence = tier.confidence_base();

        let mut reasoning = format!(
            "Assigned {} based on source: {}",
            tier.label(),
            source_description
        );
        if !adjustments.is_empty() {
            reasoning.push_str(" | ");
            reasoning.push_str(&adjustments.join("; "));
        }

        ClaimTierAssignment {
            claim: claim.to_string(),
            assigned_tier: tier,
            confidence_adjustment: confidence,
            reasoning,
        }
    }

    pub fn assess_source_credibility(&self, source: &str) -> f64 {
        let lower = source.to_lowercase();
        let mut score: f64 = 0.5;

        // Domain-based assessment
        if lower.contains(".gov") || lower.contains(".gov/") {
            score += 0.25;
        } else if lower.contains(".edu") || lower.contains(".edu/") || lower.contains(".ac.") {
            score += 0.20;
        } else if lower.contains(".int") || lower.contains(".mil") {
            score += 0.20;
        } else if lower.ends_with(".org") || lower.contains(".org/") {
            score += 0.10;
        }

        // Low-credibility domains
        if lower.contains("blogspot")
            || lower.contains("wordpress")
            || lower.contains("medium.com")
            || lower.contains("wix")
            || lower.contains("substack")
        {
            score -= 0.20;
        }

        // Author expertise indicators
        let expertise_indicators = [
            "ph.d",
            "phd",
            "professor",
            "prof.",
            "dr.",
            "m.d.",
            "m.d",
            "researcher at",
            "senior scientist",
            "principal investigator",
        ];
        for &indicator in &expertise_indicators {
            if lower.contains(indicator) {
                score += 0.15;
                break;
            }
        }

        // Publication venue quality
        if lower.contains("pubmed")
            || lower.contains("doi.org/10.")
            || lower.contains("nature")
            || lower.contains("science")
            || lower.contains("cell")
            || lower.contains("nejm")
            || lower.contains("lancet")
            || lower.contains("ieee")
            || lower.contains("acm")
            || lower.contains("springer")
        {
            score += 0.20;
        } else if lower.contains("arxiv") || lower.contains("biorxiv") || lower.contains("medrxiv")
        {
            score += 0.10;
        } else if lower.contains("cnn")
            || lower.contains("bbc")
            || lower.contains("reuters")
            || lower.contains("ap news")
            || lower.contains("npr")
            || lower.contains("the guardian")
            || lower.contains("nyt")
        {
            score += 0.05;
        }

        // Recency indicators
        if lower.contains("2026") || lower.contains("2025") {
            score += 0.10;
        } else if lower.contains("2024") {
            score += 0.05;
        } else if lower.contains("2014") || lower.contains("2015") || lower.contains("2016") {
            score -= 0.05;
        }

        score.clamp(0.0, 1.0)
    }

    pub fn detect_cross_reference(&self, claim: &str) -> bool {
        let lower = claim.to_lowercase();
        let patterns = [
            "according to",
            "cited by",
            "as reported",
            "as stated",
            "per ",
            "source says",
            "source said",
            "sources say",
            "sources said",
            "references",
            "citing",
            "cites",
            "quotes",
            "quoted",
            "reported by",
            "published in",
            "based on",
            "drawing on",
            "refers to",
            "referencing",
            "in the words of",
            "as noted by",
            "as mentioned by",
            "as described in",
            "per the",
            "sources indicate",
            "source indicates",
        ];
        patterns.iter().any(|p| lower.contains(p))
    }

    fn has_experimental_indicators(&self, claim: &str, source: &str) -> bool {
        let expt_words = [
            "clinical trial",
            "randomized",
            "meta-analysis",
            "p < 0.01",
            "p<0.01",
            "p < 0.001",
            "replication",
            "controlled study",
            "double-blind",
            "systematic review",
            "cochrane",
            "experiment",
            "laboratory",
        ];
        let has_claim = expt_words.iter().any(|w| claim.contains(w));
        let has_source_pub = source.contains("pubmed")
            || source.contains("nature")
            || source.contains("science")
            || source.contains("cell")
            || source.contains("nejm")
            || source.contains("lancet")
            || source.contains("doi.org/10.");
        has_claim || has_source_pub
    }

    fn has_observational_indicators(&self, claim: &str, source: &str) -> bool {
        let obs_words = [
            "observed",
            "observation",
            "witnessed",
            "recorded",
            "measured",
            "detected",
            "survey",
            "cohort",
            "epidemiological",
            "monitoring",
        ];
        let has_claim = obs_words.iter().any(|w| claim.contains(w));
        let has_independent = source.contains("multiple")
            || source.contains("independent")
            || source.contains("cross-reference")
            || source.contains("confirmed by");
        has_claim || has_independent
    }

    fn has_statistical_indicators(&self, claim: &str) -> bool {
        let stat_words = [
            "statistically",
            "significant",
            "p value",
            "confidence interval",
            "standard deviation",
            "mean",
            "median",
            "correlation",
            "regression",
            "probability",
            "likelihood",
            "percentage",
            "rate of",
            "incidence",
            "prevalence",
            "odds ratio",
            "risk factor",
        ];
        stat_words.iter().any(|w| claim.contains(w))
    }

    fn has_expert_indicators(&self, source: &str) -> bool {
        let expert_words = [
            "expert",
            "professor",
            "dr.",
            "phd",
            "researcher at",
            "university",
            "institute",
            "association of",
            "society of",
            "panel",
            "task force",
            "consensus statement",
            "white paper",
        ];
        expert_words.iter().any(|w| source.contains(w))
    }

    fn has_institutional_indicators(&self, source: &str) -> bool {
        let inst_domains = [
            ".gov",
            ".edu",
            ".org",
            ".int",
            ".mil",
            "who.int",
            "un.org",
            "world bank",
            "imf",
            "oecd",
            "government",
            "official",
            "ministry",
            "department of",
            "national",
            "federal",
            "state",
            "bureau",
            "agency",
        ];
        inst_domains.iter().any(|d| source.contains(d))
    }

    pub fn cross_tier_conflict(
        &self,
        a: &ClaimTierAssignment,
        b: &ClaimTierAssignment,
    ) -> Option<String> {
        if FactTier::is_category_error(a.assigned_tier, b.assigned_tier) {
            Some(format!(
                "Category error: {} (T{}:{}) vs {} (T{}:{})",
                a.claim,
                a.assigned_tier as i32,
                a.assigned_tier.label(),
                b.claim,
                b.assigned_tier as i32,
                b.assigned_tier.label(),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experimental_tier() {
        let a = FactTierAnalyzer::new();
        let r = a.assign_tier(
            "A double-blind clinical trial showed...",
            "Published in Nature Medicine, doi.org/10.1038/s41591-024-00001-x",
        );
        assert_eq!(r.assigned_tier, FactTier::Tier1Experimental);
    }

    #[test]
    fn test_anecdotal_default() {
        let a = FactTierAnalyzer::new();
        let r = a.assign_tier("I saw it happen with my own eyes", "Personal blog post");
        assert_eq!(r.assigned_tier, FactTier::Tier6Anecdotal);
    }

    #[test]
    fn test_category_error_detection() {
        assert!(FactTier::is_category_error(
            FactTier::Tier1Experimental,
            FactTier::Tier6Anecdotal
        ));
        assert!(!FactTier::is_category_error(
            FactTier::Tier3Statistical,
            FactTier::Tier4ExpertConsensus
        ));
    }

    #[test]
    fn test_statistical_tier() {
        let a = FactTierAnalyzer::new();
        let r = a.assign_tier(
            "There is a statistically significant correlation between smoking and lung cancer",
            "Research paper",
        );
        assert_eq!(r.assigned_tier, FactTier::Tier3Statistical);
    }

    #[test]
    fn test_source_credibility_gov() {
        let a = FactTierAnalyzer::new();
        let s = a.assess_source_credibility("Published on cdc.gov, 2025");
        assert!(s >= 0.70, "Score was {}", s);
    }

    #[test]
    fn test_source_credibility_blog() {
        let a = FactTierAnalyzer::new();
        let s = a.assess_source_credibility("Written on wordpress.com by an anonymous user");
        assert!(s <= 0.40, "Score was {}", s);
    }

    #[test]
    fn test_source_credibility_peer_reviewed() {
        let a = FactTierAnalyzer::new();
        let s = a.assess_source_credibility(
            "Dr. Smith, Professor at Harvard, published in Nature, 2026",
        );
        assert!(s >= 0.80, "Score was {}", s);
    }

    #[test]
    fn test_evidence_downgrade_deep_chain() {
        let a = FactTierAnalyzer::new();
        let evidence = EvidenceChain::new().with_depth(3);
        let r = a.assign_tier_with_evidence(
            "Government report states the policy was effective",
            "Official government source",
            &evidence,
        );
        // Tier5Institutional → downgraded to Tier6 (chain depth 3 > 2)
        assert_eq!(r.assigned_tier, FactTier::Tier6Anecdotal);
        assert!(r.reasoning.contains("hearsay"));
    }

    #[test]
    fn test_evidence_upgrade_confirmations() {
        let a = FactTierAnalyzer::new();
        let evidence = EvidenceChain::new().with_confirmations(3);
        let r = a.assign_tier_with_evidence(
            "I saw it happen with my own eyes",
            "Personal blog post",
            &evidence,
        );
        // Tier6Anecdotal → upgraded to Tier5Institutional (3 independent confirmations)
        assert_eq!(r.assigned_tier, FactTier::Tier5Institutional);
    }

    #[test]
    fn test_detect_cross_reference() {
        let a = FactTierAnalyzer::new();
        assert!(a.detect_cross_reference(
            "According to the IPCC report, global temperatures are rising"
        ));
        assert!(a.detect_cross_reference("As reported by Reuters, the deal has been signed"));
        assert!(!a.detect_cross_reference("I measured the temperature myself and it was 22°C"));
    }

    #[test]
    fn test_assign_tier_backward_compat() {
        let a = FactTierAnalyzer::new();
        let r1 = a.assign_tier(
            "A double-blind clinical trial showed...",
            "Published in Nature Medicine, doi.org/10.1038/s41591-024-00001-x",
        );
        assert_eq!(r1.assigned_tier, FactTier::Tier1Experimental);

        let r2 = a.assign_tier("I saw it happen with my own eyes", "Personal blog post");
        assert_eq!(r2.assigned_tier, FactTier::Tier6Anecdotal);

        let r3 = a.assign_tier(
            "There is a statistically significant correlation",
            "Research paper",
        );
        assert_eq!(r3.assigned_tier, FactTier::Tier3Statistical);

        assert!(FactTier::is_category_error(
            FactTier::Tier1Experimental,
            FactTier::Tier6Anecdotal
        ));
        assert!(!FactTier::is_category_error(
            FactTier::Tier3Statistical,
            FactTier::Tier4ExpertConsensus
        ));
    }
}
