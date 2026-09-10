use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmergencePattern {
    TradeNetwork,
    SocialHierarchy,
    CooperationCluster,
    Specialization,
    CulturalNorm,
    CollectiveAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceEvent {
    pub pattern: EmergencePattern,
    pub confidence: f64,
    pub participants: Vec<String>,
    pub first_detected_tick: u64,
    pub last_reinforced_tick: u64,
    pub reinforcement_count: u32,
    pub significance: f64,
}

pub struct SimAgentSnapshot {
    pub id: String,
    pub fitness: f64,
    pub skills: Vec<String>,
    pub position: [f32; 2],
}

pub struct RelationshipSummary {
    pub total_relationships: usize,
    pub avg_trust: f64,
    pub strong_bonds: usize,
    pub clusters: usize,
}

pub struct EconomySummary {
    pub total_trades: u64,
    pub trade_density: f64,
    pub resource_diversity: f64,
}

pub struct CultureSummary {
    pub meme_count: usize,
    pub norm_count: usize,
    pub adoption_rate: f64,
}

pub struct EmergenceDetector {
    events: Vec<EmergenceEvent>,
    history: Vec<EmergenceEvent>,
    baselines: HashMap<EmergencePattern, f64>,
    window_size: usize,
    max_events: usize,
}

impl EmergenceDetector {
    pub fn new() -> Self {
        let mut baselines = HashMap::new();
        baselines.insert(EmergencePattern::TradeNetwork, 0.0);
        baselines.insert(EmergencePattern::SocialHierarchy, 0.0);
        baselines.insert(EmergencePattern::CooperationCluster, 0.0);
        baselines.insert(EmergencePattern::Specialization, 0.0);
        baselines.insert(EmergencePattern::CulturalNorm, 0.0);
        baselines.insert(EmergencePattern::CollectiveAction, 0.0);

        Self {
            events: Vec::new(),
            history: Vec::new(),
            baselines,
            window_size: 100,
            max_events: 50,
        }
    }

    pub fn analyze(
        &mut self,
        agents: &[SimAgentSnapshot],
        relationships: &RelationshipSummary,
        economy: &EconomySummary,
        culture: &CultureSummary,
        tick: u64,
    ) -> Vec<EmergenceEvent> {
        let mut new_events = Vec::new();

        if let Some(e) = self.detect_trade_network(relationships, economy, agents.len(), tick) {
            new_events.push(e);
        }
        if let Some(e) = self.detect_social_hierarchy(relationships, agents, tick) {
            new_events.push(e);
        }
        if let Some(e) = self.detect_cooperation(relationships, agents, tick) {
            new_events.push(e);
        }
        if let Some(e) = self.detect_specialization(agents, tick) {
            new_events.push(e);
        }
        if let Some(e) = self.detect_cultural_norm(relationships, culture, tick) {
            new_events.push(e);
        }
        if let Some(e) = self.detect_collective_action(relationships, agents, tick) {
            new_events.push(e);
        }

        self.reinforce_existing(&new_events, tick);
        self.prune(tick);

        new_events
    }

    fn detect_trade_network(
        &mut self,
        relationships: &RelationshipSummary,
        economy: &EconomySummary,
        agent_count: usize,
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if agent_count == 0 {
            return None;
        }

        let expected_pairs = (agent_count * (agent_count - 1)) as f64 / 2.0;
        let density = economy.total_trades as f64 / expected_pairs.max(1.0);

        if density > 0.1 && economy.total_trades > (agent_count as u64) * 2 {
            let confidence = (density.min(1.0) * 0.5 + (economy.resource_diversity * 0.5)).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::TradeNetwork]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::TradeNetwork,
                confidence,
                participants: Vec::new(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn detect_social_hierarchy(
        &mut self,
        relationships: &RelationshipSummary,
        agents: &[SimAgentSnapshot],
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if relationships.total_relationships == 0 || agents.is_empty() {
            return None;
        }

        let strong_ratio = relationships.strong_bonds as f64 / relationships.total_relationships as f64;
        let low_trust = relationships.avg_trust < 0.5;

        if strong_ratio < 0.3 && low_trust {
            let fitness_std = self.compute_fitness_std(agents);
            let confidence = ((1.0 - strong_ratio) * 0.4 + fitness_std * 0.3 + (1.0 - relationships.avg_trust.max(0.0)) * 0.3).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::SocialHierarchy]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::SocialHierarchy,
                confidence,
                participants: agents.iter().map(|a| a.id.clone()).collect(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn detect_cooperation(
        &mut self,
        relationships: &RelationshipSummary,
        agents: &[SimAgentSnapshot],
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if agents.is_empty() {
            return None;
        }

        let expected_clusters = agents.len() / 3;
        let high_trust = relationships.avg_trust > 0.6;
        let few_clusters = relationships.clusters < expected_clusters.max(1);

        if high_trust && few_clusters {
            let confidence = (relationships.avg_trust * 0.6 + (1.0 - relationships.clusters as f64 / agents.len() as f64).max(0.0) * 0.4).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::CooperationCluster]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::CooperationCluster,
                confidence,
                participants: agents.iter().map(|a| a.id.clone()).collect(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn detect_specialization(
        &mut self,
        agents: &[SimAgentSnapshot],
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if agents.len() < 3 {
            return None;
        }

        let mut skill_sets: HashSet<Vec<String>> = HashSet::new();
        for agent in agents {
            let mut skills = agent.skills.clone();
            skills.sort();
            skill_sets.insert(skills);
        }

        if skill_sets.len() > 3 {
            let diversity = skill_sets.len() as f64 / agents.len() as f64;
            let confidence = (diversity * 0.7 + 0.3).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::Specialization]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::Specialization,
                confidence,
                participants: agents.iter().map(|a| a.id.clone()).collect(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn detect_cultural_norm(
        &mut self,
        _relationships: &RelationshipSummary,
        culture: &CultureSummary,
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if culture.norm_count > 0 && culture.adoption_rate > 0.5 {
            let confidence = (culture.adoption_rate * 0.6 + (culture.norm_count as f64 / 10.0).min(1.0) * 0.4).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::CulturalNorm]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::CulturalNorm,
                confidence,
                participants: Vec::new(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn detect_collective_action(
        &mut self,
        relationships: &RelationshipSummary,
        agents: &[SimAgentSnapshot],
        tick: u64,
    ) -> Option<EmergenceEvent> {
        if agents.is_empty() {
            return None;
        }

        let connectivity = relationships.total_relationships as f64 / agents.len() as f64;
        let high_trust = relationships.avg_trust > 0.7;
        let well_connected = connectivity > 2.0;

        if high_trust && well_connected {
            let confidence = (relationships.avg_trust * 0.5 + (connectivity / 5.0).min(1.0) * 0.5).min(1.0);
            let significance = confidence * (1.0 - self.baselines[&EmergencePattern::CollectiveAction]);

            return Some(EmergenceEvent {
                pattern: EmergencePattern::CollectiveAction,
                confidence,
                participants: agents.iter().map(|a| a.id.clone()).collect(),
                first_detected_tick: tick,
                last_reinforced_tick: tick,
                reinforcement_count: 0,
                significance,
            });
        }
        None
    }

    fn reinforce_existing(&mut self, new_events: &[EmergenceEvent], tick: u64) {
        for new_event in new_events {
            if let Some(existing) = self.events.iter_mut().find(|e| e.pattern == new_event.pattern) {
                existing.reinforcement_count += 1;
                existing.last_reinforced_tick = tick;
                existing.confidence = (existing.confidence + new_event.confidence) / 2.0;
                existing.significance = (existing.significance + new_event.significance) / 2.0;
            } else {
                self.events.push(new_event.clone());
                self.history.push(new_event.clone());
            }
        }
    }

    fn prune(&mut self, tick: u64) {
        let max_age = self.window_size as u64;
        self.events.retain(|e| tick.saturating_sub(e.last_reinforced_tick) < max_age);

        if self.events.len() > self.max_events {
            self.events.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap_or(std::cmp::Ordering::Equal));
            self.events.truncate(self.max_events);
        }

        for event in &self.events {
            *self.baselines.entry(event.pattern.clone()).or_insert(0.0) = event.confidence;
        }
    }

    fn compute_fitness_std(&self, agents: &[SimAgentSnapshot]) -> f64 {
        if agents.len() < 2 {
            return 0.0;
        }

        let mean = agents.iter().map(|a| a.fitness).sum::<f64>() / agents.len() as f64;
        let variance = agents.iter().map(|a| (a.fitness - mean).powi(2)).sum::<f64>() / agents.len() as f64;
        variance.sqrt()
    }

    pub fn active_events(&self) -> &[EmergenceEvent] {
        &self.events
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn significance_of(&self, pattern: &EmergencePattern) -> f64 {
        self.events
            .iter()
            .filter(|e| &e.pattern == pattern)
            .map(|e| e.significance)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn history(&self) -> &[EmergenceEvent] {
        &self.history
    }
}

impl Default for EmergenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agents(count: usize, same_skills: bool) -> Vec<SimAgentSnapshot> {
        (0..count)
            .map(|i| SimAgentSnapshot {
                id: format!("agent_{}", i),
                fitness: if same_skills { 0.5 } else { (i as f64) / count as f64 },
                skills: if same_skills {
                    vec!["gather".to_string()]
                } else {
                    match i % 4 {
                        0 => vec!["gather".to_string(), "craft".to_string()],
                        1 => vec!["trade".to_string(), "negotiate".to_string()],
                        2 => vec!["build".to_string(), "repair".to_string()],
                        _ => vec!["hunt".to_string(), "forage".to_string()],
                    }
                },
                position: [i as f32 * 10.0, i as f32 * 10.0],
            })
            .collect()
    }

    #[test]
    fn test_trade_network_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(10, true);
        let rels = RelationshipSummary {
            total_relationships: 20,
            avg_trust: 0.5,
            strong_bonds: 5,
            clusters: 2,
        };
        let econ = EconomySummary {
            total_trades: 30,
            trade_density: 0.15,
            resource_diversity: 0.8,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::TradeNetwork));
    }

    #[test]
    fn test_no_detection_with_low_activity() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(5, true);
        let rels = RelationshipSummary {
            total_relationships: 2,
            avg_trust: 0.6,
            strong_bonds: 1,
            clusters: 5,
        };
        let econ = EconomySummary {
            total_trades: 1,
            trade_density: 0.01,
            resource_diversity: 0.1,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_reinforcement_increases_count() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(10, true);
        let rels = RelationshipSummary {
            total_relationships: 20,
            avg_trust: 0.5,
            strong_bonds: 5,
            clusters: 2,
        };
        let econ = EconomySummary {
            total_trades: 30,
            trade_density: 0.15,
            resource_diversity: 0.8,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        detector.analyze(&agents, &rels, &econ, &culture, 1);
        detector.analyze(&agents, &rels, &econ, &culture, 2);

        let trade_event = detector.active_events().iter().find(|e| e.pattern == EmergencePattern::TradeNetwork);
        assert!(trade_event.is_some());
        assert!(trade_event.unwrap().reinforcement_count >= 1);
    }

    #[test]
    fn test_pruning_removes_old_events() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(10, true);
        let rels = RelationshipSummary {
            total_relationships: 20,
            avg_trust: 0.5,
            strong_bonds: 5,
            clusters: 2,
        };
        let econ = EconomySummary {
            total_trades: 30,
            trade_density: 0.15,
            resource_diversity: 0.8,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(detector.event_count() > 0);

        for tick in 2..150 {
            let empty_econ = EconomySummary {
                total_trades: 0,
                trade_density: 0.0,
                resource_diversity: 0.0,
            };
            let empty_culture = CultureSummary {
                meme_count: 0,
                norm_count: 0,
                adoption_rate: 0.0,
            };
            detector.analyze(&agents, &rels, &empty_econ, &empty_culture, tick);
        }

        assert_eq!(detector.event_count(), 0);
    }

    #[test]
    fn test_specialization_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(6, false);
        let rels = RelationshipSummary {
            total_relationships: 10,
            avg_trust: 0.4,
            strong_bonds: 2,
            clusters: 3,
        };
        let econ = EconomySummary {
            total_trades: 5,
            trade_density: 0.05,
            resource_diversity: 0.3,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::Specialization));
    }

    #[test]
    fn test_cooperation_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(9, true);
        let rels = RelationshipSummary {
            total_relationships: 30,
            avg_trust: 0.8,
            strong_bonds: 20,
            clusters: 2,
        };
        let econ = EconomySummary {
            total_trades: 5,
            trade_density: 0.02,
            resource_diversity: 0.2,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::CooperationCluster));
    }

    #[test]
    fn test_cultural_norm_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(5, true);
        let rels = RelationshipSummary {
            total_relationships: 10,
            avg_trust: 0.5,
            strong_bonds: 3,
            clusters: 1,
        };
        let econ = EconomySummary {
            total_trades: 3,
            trade_density: 0.05,
            resource_diversity: 0.3,
        };
        let culture = CultureSummary {
            meme_count: 5,
            norm_count: 3,
            adoption_rate: 0.7,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::CulturalNorm));
    }

    #[test]
    fn test_significance_calculation() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(10, true);
        let rels = RelationshipSummary {
            total_relationships: 20,
            avg_trust: 0.5,
            strong_bonds: 5,
            clusters: 2,
        };
        let econ = EconomySummary {
            total_trades: 30,
            trade_density: 0.15,
            resource_diversity: 0.8,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        detector.analyze(&agents, &rels, &econ, &culture, 1);
        let sig = detector.significance_of(&EmergencePattern::TradeNetwork);
        assert!(sig > 0.0);
        assert!(sig <= 1.0);
    }

    #[test]
    fn test_social_hierarchy_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(8, false);
        let rels = RelationshipSummary {
            total_relationships: 10,
            avg_trust: 0.3,
            strong_bonds: 2,
            clusters: 4,
        };
        let econ = EconomySummary {
            total_trades: 3,
            trade_density: 0.02,
            resource_diversity: 0.2,
        };
        let culture = CultureSummary {
            meme_count: 0,
            norm_count: 0,
            adoption_rate: 0.0,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::SocialHierarchy));
    }

    #[test]
    fn test_collective_action_detection() {
        let mut detector = EmergenceDetector::new();
        let agents = make_agents(6, true);
        let rels = RelationshipSummary {
            total_relationships: 20,
            avg_trust: 0.8,
            strong_bonds: 15,
            clusters: 1,
        };
        let econ = EconomySummary {
            total_trades: 10,
            trade_density: 0.08,
            resource_diversity: 0.4,
        };
        let culture = CultureSummary {
            meme_count: 3,
            norm_count: 1,
            adoption_rate: 0.4,
        };

        let events = detector.analyze(&agents, &rels, &econ, &culture, 1);
        assert!(events.iter().any(|e| e.pattern == EmergencePattern::CollectiveAction));
    }
}
