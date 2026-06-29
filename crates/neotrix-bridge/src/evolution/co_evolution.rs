use std::collections::HashMap;
use crate::types::{Domain, CuriositySignal, VsaLight};

#[derive(Debug, Clone)]
pub struct SharedInsight {
    pub source_domain: Domain,
    pub pattern: String,
    pub confidence: f64,
    pub timestamp_ms: i64,
    pub effectiveness: f64,
    pub cross_domain_relevance: f64,
}

#[derive(Debug, Clone)]
pub struct CrossDomainLink {
    pub from: Domain,
    pub to: Domain,
    pub transfer_count: u64,
    pub avg_effectiveness: f64,
    pub last_transfer_ms: i64,
}

#[derive(Debug)]
pub struct EvolutionState {
    pub domain: Domain,
    pub reflection_count: u64,
    pub last_reflection_ms: i64,
    pub stagnation_cycles: u64,
    pub last_improvement_ms: i64,
    pub current_strategy: String,
}

#[derive(Debug)]
pub struct BridgeCoEvolution {
    pub shared_memory: Vec<SharedInsight>,
    pub links: Vec<CrossDomainLink>,
    pub states: HashMap<Domain, EvolutionState>,
    pub vsa: VsaLight,
    pub total_reflections: u64,
    pub total_consolidations: u64,
    pub total_redirections: u64,
    max_insights: usize,
}

impl Default for BridgeCoEvolution {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeCoEvolution {
    pub fn new() -> Self {
        let mut states = HashMap::new();
        for d in &[Domain::Crypto, Domain::Earn, Domain::Network, Domain::Crawl,
                     Domain::Social, Domain::Browse, Domain::Vision] {
            states.insert(*d, EvolutionState {
                domain: *d,
                reflection_count: 0,
                last_reflection_ms: 0,
                stagnation_cycles: 0,
                last_improvement_ms: chrono::Utc::now().timestamp_millis(),
                current_strategy: "explore".into(),
            });
        }
        Self {
            shared_memory: Vec::new(),
            links: Vec::new(),
            states,
            vsa: VsaLight::new(256),
            total_reflections: 0,
            total_consolidations: 0,
            total_redirections: 0,
            max_insights: 200,
        }
    }

    /// Heartbeat reflection: each domain records what it learned
    pub fn reflect(&mut self, domain: Domain, pattern: String, confidence: f64, effectiveness: f64) {
        self.total_reflections += 1;
        if let Some(state) = self.states.get_mut(&domain) {
            state.reflection_count += 1;
            state.last_reflection_ms = chrono::Utc::now().timestamp_millis();
            if effectiveness > 0.3 {
                state.stagnation_cycles = 0;
                state.last_improvement_ms = chrono::Utc::now().timestamp_millis();
            } else {
                state.stagnation_cycles += 1;
            }
        }

        let insight = SharedInsight {
            source_domain: domain,
            pattern,
            confidence,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            effectiveness,
            cross_domain_relevance: 0.0,
        };

        if self.shared_memory.len() >= self.max_insights {
            // Remove lowest-confidence insight
            if let Some(idx) = self.shared_memory.iter()
                .enumerate()
                .min_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
            {
                self.shared_memory.remove(idx);
            }
        }
        self.shared_memory.push(insight);
    }

    /// Consolidation: merge related insights across domains, form cross-domain links
    pub fn consolidate(&mut self) -> Vec<SharedInsight> {
        self.total_consolidations += 1;
        let mut consolidated = Vec::new();

        // Group by pattern similarity (same pattern string = related)
        let mut by_pattern: HashMap<String, Vec<SharedInsight>> = HashMap::new();
        for insight in &self.shared_memory {
            by_pattern.entry(insight.pattern.clone()).or_default().push(insight.clone());
        }

        for (pattern, group) in &by_pattern {
            if group.len() < 2 {
                continue;
            }
            let avg_eff: f64 = group.iter().map(|i| i.effectiveness).sum::<f64>() / group.len() as f64;
            let max_conf = group.iter().map(|i| i.confidence).fold(0.0_f64, f64::max);
            let domains: Vec<Domain> = group.iter().map(|i| i.source_domain).collect();

            // Create cross-domain links between pairs
            for i in 0..domains.len() {
                for j in (i + 1)..domains.len() {
                    let from = domains[i];
                    let to = domains[j];
                    if let Some(link) = self.links.iter_mut()
                        .find(|l| l.from == from && l.to == to || l.from == to && l.to == from)
                    {
                        link.transfer_count += 1;
                        link.avg_effectiveness = link.avg_effectiveness * 0.7 + avg_eff * 0.3;
                        link.last_transfer_ms = chrono::Utc::now().timestamp_millis();
                    } else {
                        self.links.push(CrossDomainLink {
                            from,
                            to,
                            transfer_count: 1,
                            avg_effectiveness: avg_eff,
                            last_transfer_ms: chrono::Utc::now().timestamp_millis(),
                        });
                        const MAX_LINKS: usize = 50000;
                        if self.links.len() > MAX_LINKS {
                            self.links.drain(0..MAX_LINKS / 5);
                        }
                    }
                }
            }

            consolidated.push(SharedInsight {
                source_domain: domains[0],
                pattern: format!("[consolidated] {}", pattern),
                confidence: max_conf,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                effectiveness: avg_eff,
                cross_domain_relevance: domains.len() as f64 / 7.0,
            });
        }

        // Prune low-confidence insights
        self.shared_memory.retain(|i| i.confidence > 0.2 || i.effectiveness > 0.5);

        consolidated
    }

    /// Redirection: find stagnant domains and suggest strategy pivot
    pub fn redirection_signals(&self) -> Vec<(Domain, String)> {
        let mut signals = Vec::new();
        for (domain, state) in &self.states {
            if state.stagnation_cycles >= 3 {
                let (_new_strategy, reason): (String, String) = match state.current_strategy.as_str() {
                    "explore" => (String::from("deepen"), String::from("stagnant after explore, try deepening")),
                    "deepen" => (String::from("cross_pollinate"), String::from("deepening stalled, try cross-domain")),
                    "cross_pollinate" => (String::from("reset"), String::from("cross-pollination exhausted, reset strategy")),
                    _ => (String::from("explore"), String::from("default redirect to explore")),
                };
                signals.push((*domain, reason));
            }
        }
        signals
    }

    /// Get curiosity signals enriched by co-evolution state
    pub fn enriched_curiosity(&self, base: &[CuriositySignal]) -> Vec<CuriositySignal> {
        let mut enriched: Vec<CuriositySignal> = base.to_vec();
        for signal in &mut enriched {
            let mut bonus = 0.0;
            // Bonus for stagnant domains
            if let Some(state) = self.states.get(&signal.domain) {
                if state.stagnation_cycles >= 2 {
                    bonus += 0.2;
                }
            }
            // Bonus from cross-domain links
            for link in &self.links {
                if (link.from == signal.domain || link.to == signal.domain)
                    && link.avg_effectiveness > 0.5 {
                        bonus += 0.1;
                    }
            }
            signal.novelty_estimate = (signal.novelty_estimate + bonus).min(1.0);
        }
        enriched
    }

    /// Strongest cross-domain link for curiosity routing
    pub fn strongest_link(&self, threshold: f64) -> Option<CrossDomainLink> {
        self.links.iter()
            .filter(|l| l.avg_effectiveness >= threshold)
            .max_by(|a, b| a.transfer_count.cmp(&b.transfer_count))
            .cloned()
    }

    pub fn heartbeat_tick(&mut self, domain: Domain) -> EvolutionBeat {
        let state = self.states.get(&domain);
        let stagnant = state.map(|s| s.stagnation_cycles >= 3).unwrap_or(false);
        let insights = self.shared_memory.iter()
            .filter(|i| i.source_domain == domain)
            .count();
        let cross_links = self.links.iter()
            .filter(|l| l.from == domain || l.to == domain)
            .count();

        EvolutionBeat {
            domain,
            reflection_count: state.map(|s| s.reflection_count).unwrap_or(0),
            stagnation_cycles: state.map(|s| s.stagnation_cycles).unwrap_or(0),
            stagnant,
            shared_insights: insights,
            cross_links,
            total_reflections: self.total_reflections,
            total_consolidations: self.total_consolidations,
            total_redirections: self.total_redirections,
        }
    }
}

#[derive(Debug)]
pub struct EvolutionBeat {
    pub domain: Domain,
    pub reflection_count: u64,
    pub stagnation_cycles: u64,
    pub stagnant: bool,
    pub shared_insights: usize,
    pub cross_links: usize,
    pub total_reflections: u64,
    pub total_consolidations: u64,
    pub total_redirections: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CuriositySignal;

    #[test]
    fn test_reflection_adds_insight() {
        let mut evo = BridgeCoEvolution::new();
        evo.reflect(Domain::Crawl, "deep_exploration_works".into(), 0.8, 0.7);
        assert_eq!(evo.shared_memory.len(), 1);
        assert_eq!(evo.total_reflections, 1);
    }

    #[test]
    fn test_stagnation_tracking() {
        let mut evo = BridgeCoEvolution::new();
        for _ in 0..4 {
            evo.reflect(Domain::Crypto, "failed_swap".into(), 0.1, 0.05);
        }
        let state = evo.states.get(&Domain::Crypto).unwrap();
        assert!(state.stagnation_cycles >= 3);
    }

    #[test]
    fn test_redirection_on_stagnation() {
        let mut evo = BridgeCoEvolution::new();
        for _ in 0..5 {
            evo.reflect(Domain::Network, "timeout".into(), 0.1, 0.05);
        }
        let signals = evo.redirection_signals();
        assert!(signals.iter().any(|(d, _)| *d == Domain::Network));
    }

    #[test]
    fn test_consolidation_creates_links() {
        let mut evo = BridgeCoEvolution::new();
        evo.reflect(Domain::Crawl, "pattern_X".into(), 0.8, 0.7);
        evo.reflect(Domain::Network, "pattern_X".into(), 0.7, 0.6);
        evo.consolidate();
        assert!(!evo.links.is_empty());
        let link = &evo.links[0];
        assert!(link.transfer_count >= 1);
    }

    #[test]
    fn test_enriched_curiosity() {
        let mut evo = BridgeCoEvolution::new();
        for _ in 0..4 {
            evo.reflect(Domain::Crawl, "stuck".into(), 0.1, 0.05);
        }
        let bases = vec![
            CuriositySignal { domain: Domain::Crawl, query: "test".into(), novelty_estimate: 0.5, potential_negentropy: 0.5 },
        ];
        let enriched = evo.enriched_curiosity(&bases);
        assert!(enriched[0].novelty_estimate > 0.5);
    }

    #[test]
    fn test_heartbeat_tick() {
        let mut evo = BridgeCoEvolution::new();
        evo.reflect(Domain::Vision, "scene_ok".into(), 0.9, 0.8);
        let beat = evo.heartbeat_tick(Domain::Vision);
        assert!(!beat.stagnant);
        assert_eq!(beat.reflection_count, 1);
    }

    #[test]
    fn test_max_insights_pruning() {
        let mut evo = BridgeCoEvolution { max_insights: 5, ..BridgeCoEvolution::new() };
        for i in 0..10 {
            evo.reflect(Domain::System, format!("pattern_{}", i), 0.1, 0.1);
        }
        assert!(evo.shared_memory.len() <= 5);
    }

    #[test]
    fn test_strongest_link() {
        let mut evo = BridgeCoEvolution::new();
        evo.reflect(Domain::Crawl, "common".into(), 0.9, 0.8);
        evo.reflect(Domain::Network, "common".into(), 0.9, 0.8);
        evo.consolidate();
        let link = evo.strongest_link(0.5);
        assert!(link.is_some());
    }

    #[test]
    fn test_no_stagnation_with_good_effectiveness() {
        let mut evo = BridgeCoEvolution::new();
        for _ in 0..5 {
            evo.reflect(Domain::Browse, "good_results".into(), 0.9, 0.8);
        }
        let state = evo.states.get(&Domain::Browse).unwrap();
        assert_eq!(state.stagnation_cycles, 0);
    }

    #[test]
    fn test_redirection_cycles_strategies() {
        let mut evo = BridgeCoEvolution::new();
        let domain = Domain::Earn;
        for _ in 0..10 {
            evo.reflect(domain.clone(), "low_effect".into(), 0.1, 0.05);
        }
        let signals = evo.redirection_signals();
        assert!(signals.iter().any(|(d, _)| *d == domain));
    }

    #[test]
    fn test_cross_domain_insight_relevance() {
        let mut evo = BridgeCoEvolution::new();
        for d in &[Domain::Crawl, Domain::Network, Domain::Social] {
            evo.reflect(d.clone(), "shared_insight_abc".into(), 0.8, 0.7);
        }
        let cons = evo.consolidate();
        let found = cons.iter().any(|i| i.cross_domain_relevance > 0.3);
        assert!(found, "cross-domain relevance should reflect 3 domains / 7 total");
    }
}
