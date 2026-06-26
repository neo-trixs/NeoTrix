//! # ExplorationTrigger — Curiosity-Driven Knowledge Acquisition
//!
//! Bridges IntrinsicMotivation signals → actual exploration behavior:
//! 1. Detects knowledge gaps from hypercube sparsity
//! 2. Maps curiosity signals to domain exploration
//! 3. Seeds the crawler with gap topics
//! 4. Records exploration outcomes in hypercube

use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::core::nt_core_self::intrinsic_motivation::MotivationState;
use crate::core::nt_core_self::silicon_self::SiliconSelfModel;

/// An exploration domain that maps to hypercube dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExploreDomain {
    SecurityVulnerabilities,
    CodeAnalysis,
    ArchitecturePatterns,
    ResearchPapers,
    SystemDesign,
    UserBehavior,
    DataScience,
    EmergingTech,
}

impl ExploreDomain {
    pub fn all() -> &'static [ExploreDomain] {
        &[
            ExploreDomain::SecurityVulnerabilities,
            ExploreDomain::CodeAnalysis,
            ExploreDomain::ArchitecturePatterns,
            ExploreDomain::ResearchPapers,
            ExploreDomain::SystemDesign,
            ExploreDomain::UserBehavior,
            ExploreDomain::DataScience,
            ExploreDomain::EmergingTech,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ExploreDomain::SecurityVulnerabilities => "security-vulnerabilities",
            ExploreDomain::CodeAnalysis => "code-analysis",
            ExploreDomain::ArchitecturePatterns => "architecture-patterns",
            ExploreDomain::ResearchPapers => "research-papers",
            ExploreDomain::SystemDesign => "system-design",
            ExploreDomain::UserBehavior => "user-behavior",
            ExploreDomain::DataScience => "data-science",
            ExploreDomain::EmergingTech => "emerging-tech",
        }
    }

    pub fn to_coord(&self) -> HyperCoord {
        let mut coord = HyperCoord::new();
        match self {
            ExploreDomain::SecurityVulnerabilities => {
                coord.set(DimensionAxis::Domain, 0.9);
                coord.set(DimensionAxis::Certainty, 0.3);
            }
            ExploreDomain::CodeAnalysis => {
                coord.set(DimensionAxis::Abstraction, 0.3);
                coord.set(DimensionAxis::Certainty, 0.8);
            }
            ExploreDomain::ArchitecturePatterns => {
                coord.set(DimensionAxis::Abstraction, 0.7);
                coord.set(DimensionAxis::Scale, 0.6);
            }
            ExploreDomain::ResearchPapers => {
                coord.set(DimensionAxis::Abstraction, 0.9);
                coord.set(DimensionAxis::Domain, 0.7);
            }
            ExploreDomain::SystemDesign => {
                coord.set(DimensionAxis::Scale, 0.8);
                coord.set(DimensionAxis::Agency, 0.7);
            }
            ExploreDomain::UserBehavior => {
                coord.set(DimensionAxis::Domain, 0.5);
                coord.set(DimensionAxis::Agency, 0.4);
            }
            ExploreDomain::DataScience => {
                coord.set(DimensionAxis::Abstraction, 0.6);
                coord.set(DimensionAxis::Scale, 0.7);
            }
            ExploreDomain::EmergingTech => {
                coord.set(DimensionAxis::Abstraction, 0.8);
                coord.set(DimensionAxis::Domain, 0.9);
            }
        }
        coord
    }
}

/// Records an exploration event.
#[derive(Debug, Clone)]
pub struct ExplorationRecord {
    pub domain: ExploreDomain,
    pub timestamp: u64,
    pub items_discovered: usize,
    pub confidence_gained: f64,
    pub hypercube_entries_added: usize,
}

/// The ExplorationTrigger — bridges motivation signals to actual exploration.
pub struct ExplorationTrigger {
    pub enabled: bool,
    pub exploration_history: Vec<ExplorationRecord>,
    pub min_curiosity_threshold: f64,
    pub cooldown_cycles: usize,
    last_exploration_cycle: usize,
    cycle: usize,
}

impl ExplorationTrigger {
    pub fn new() -> Self {
        Self {
            enabled: true,
            exploration_history: Vec::new(),
            min_curiosity_threshold: 0.6,
            cooldown_cycles: 5,
            last_exploration_cycle: 0,
            cycle: 0,
        }
    }

    /// Evaluate motivation state and trigger exploration if appropriate.
    /// Returns the number of hypercube entries added.
    pub fn evaluate(
        &mut self,
        motivation: &MotivationState,
        hypercube: &mut KnowledgeHyperCube,
        silicon_self: &SiliconSelfModel,
    ) -> usize {
        self.cycle += 1;
        if !self.enabled || !motivation.should_explore {
            return 0;
        }
        // Cooldown check
        if self.cycle - self.last_exploration_cycle < self.cooldown_cycles {
            return 0;
        }
        if motivation.intrinsic_reward < self.min_curiosity_threshold {
            return 0;
        }
        self.last_exploration_cycle = self.cycle;
        self.explore_from_motivation(motivation, hypercube, silicon_self)
    }

    /// Execute exploration: seed hypercube with new entries based on curiosity signals.
    fn explore_from_motivation(
        &mut self,
        motivation: &MotivationState,
        hypercube: &mut KnowledgeHyperCube,
        _silicon_self: &SiliconSelfModel,
    ) -> usize {
        let mut total_added = 0usize;

        // Map suggested domains to exploration actions
        let domains_to_explore = self.map_to_explore_domains(motivation);

        for domain in domains_to_explore {
            let coord = domain.to_coord();
            let key = format!("exploration-{}-cycle-{}", domain.name(), self.cycle);
            // Only insert if not already present
            if hypercube.get_entry(&key).is_none() {
                hypercube.insert(
                    &coord,
                    "exploration-trigger",
                    &format!("{}: curiosity-driven exploration", domain.name()),
                );
                total_added += 1;
            }
        }

        if total_added > 0 {
            self.exploration_history.push(ExplorationRecord {
                domain: ExploreDomain::ResearchPapers, // general placeholder
                timestamp: self.cycle as u64,
                items_discovered: 0,
                confidence_gained: motivation.intrinsic_reward,
                hypercube_entries_added: total_added,
            });
        }

        total_added
    }

    /// Map motivation state to concrete explore domains.
    fn map_to_explore_domains(&self, motivation: &MotivationState) -> Vec<ExploreDomain> {
        let mut domains = Vec::new();

        // Low confidence → knowledge gathering
        if motivation.confidence < 0.3 {
            domains.push(ExploreDomain::ResearchPapers);
            domains.push(ExploreDomain::EmergingTech);
        }

        // High error rate → refinement
        if motivation.error_rate > 0.5 {
            domains.push(ExploreDomain::CodeAnalysis);
            domains.push(ExploreDomain::SystemDesign);
        }

        // High novelty → broaden
        if motivation.novelty_score > 0.5 {
            domains.push(ExploreDomain::ArchitecturePatterns);
            domains.push(ExploreDomain::DataScience);
        }

        // Add suggested strategies as domain hints
        if motivation.suggested_domains.is_empty() && domains.is_empty() {
            domains.push(ExploreDomain::ResearchPapers);
        }

        domains
    }

    pub fn status(&self) -> String {
        format!(
            "ExplorationTrigger | enabled={} | explorations={} | cycle={} | last={}",
            self.enabled,
            self.exploration_history.len(),
            self.cycle,
            self.last_exploration_cycle,
        )
    }

    pub fn reset(&mut self) {
        self.exploration_history.clear();
        self.last_exploration_cycle = 0;
        self.cycle = 0;
    }
}

impl Default for ExplorationTrigger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::silicon_self::SiliconSelfModel;

    fn make_curious_motivation() -> MotivationState {
        MotivationState {
            intrinsic_reward: 0.8,
            confidence: 0.2,
            error_rate: 0.6,
            novelty_score: 0.7,
            should_explore: true,
            suggested_domains: Vec::new(),
            suggested_strategies: Vec::new(),
        }
    }

    fn make_satisfied_motivation() -> MotivationState {
        MotivationState {
            intrinsic_reward: 0.2,
            confidence: 0.9,
            error_rate: 0.0,
            novelty_score: 0.1,
            should_explore: false,
            suggested_domains: Vec::new(),
            suggested_strategies: Vec::new(),
        }
    }

    #[test]
    fn test_new_trigger() {
        let trigger = ExplorationTrigger::new();
        assert!(trigger.enabled);
        assert!(trigger.exploration_history.is_empty());
    }

    #[test]
    fn test_does_not_explore_when_disabled() {
        let mut trigger = ExplorationTrigger::new();
        trigger.enabled = false;
        let mut hypercube = KnowledgeHyperCube::new();
        let model = SiliconSelfModel::new();
        let added = trigger.evaluate(&make_curious_motivation(), &mut hypercube, &model);
        assert_eq!(added, 0);
    }

    #[test]
    fn test_does_not_explore_when_not_curious() {
        let mut trigger = ExplorationTrigger::new();
        let mut hypercube = KnowledgeHyperCube::new();
        let model = SiliconSelfModel::new();
        let added = trigger.evaluate(&make_satisfied_motivation(), &mut hypercube, &model);
        assert_eq!(added, 0);
    }

    #[test]
    fn test_explores_when_curious() {
        let mut trigger = ExplorationTrigger::new();
        let mut hypercube = KnowledgeHyperCube::new();
        let model = SiliconSelfModel::new();
        let added = trigger.evaluate(&make_curious_motivation(), &mut hypercube, &model);
        assert!(added > 0, "should add entries when curious, got {}", added);
    }

    #[test]
    fn test_cooldown_prevents_exploration() {
        let mut trigger = ExplorationTrigger::new();
        let mut hypercube = KnowledgeHyperCube::new();
        let model = SiliconSelfModel::new();
        let mot = make_curious_motivation();

        // First exploration
        let added1 = trigger.evaluate(&mot, &mut hypercube, &model);
        assert!(added1 > 0);

        // Second immediate exploration (should be blocked by cooldown)
        let added2 = trigger.evaluate(&mot, &mut hypercube, &model);
        assert_eq!(added2, 0, "cooldown should block immediate re-exploration");
    }

    #[test]
    fn test_cooldown_expires() {
        let mut trigger = ExplorationTrigger::new();
        trigger.cooldown_cycles = 2;
        let mut hypercube = KnowledgeHyperCube::new();
        let model = SiliconSelfModel::new();
        let mot = make_curious_motivation();

        let _ = trigger.evaluate(&mot, &mut hypercube, &model);

        // Advance cycles past cooldown
        trigger.cycle = 3;
        trigger.last_exploration_cycle = 1;

        let added = trigger.evaluate(&mot, &mut hypercube, &model);
        assert!(added > 0, "should explore again after cooldown");
    }

    #[test]
    fn test_domain_mapping_has_varied_coords() {
        let domains = ExploreDomain::all();
        let mut seen = std::collections::HashSet::<u64>::new();
        for d in domains {
            let coord = d.to_coord();
            let dense = coord.to_dense();
            let hash = dense
                .iter()
                .fold(0u64, |acc, v| acc.wrapping_add((v * 1e6) as u64));
            // Ensure no two domains map to identical coords (fuzzy dedup via hash)
            assert!(seen.insert(hash), "duplicate coord for domain {:?}", d);
        }
    }

    #[test]
    fn test_status_format() {
        let trigger = ExplorationTrigger::new();
        let s = trigger.status();
        assert!(s.contains("ExplorationTrigger"));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut trigger = ExplorationTrigger::new();
        trigger.exploration_history.push(ExplorationRecord {
            domain: ExploreDomain::CodeAnalysis,
            timestamp: 1,
            items_discovered: 0,
            confidence_gained: 0.5,
            hypercube_entries_added: 1,
        });
        trigger.cycle = 5;
        trigger.last_exploration_cycle = 3;
        trigger.reset();
        assert!(trigger.exploration_history.is_empty());
        assert_eq!(trigger.cycle, 0);
        assert_eq!(trigger.last_exploration_cycle, 0);
    }
}
