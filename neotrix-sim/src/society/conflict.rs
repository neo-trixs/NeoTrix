use serde::{Deserialize, Serialize};

use crate::agents::sim_agent::SimAgent;
use super::relationship_graph::RelationshipGraph;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ConflictType {
    #[default]
    ResourceCompetition,
    TerritoryDispute,
    IdeologicalClash,
    PersonalGrudge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resolution {
    Negotiated,
    Dominance { winner: String },
    Compromise { split: f32 },
    Avoidance { one_yields: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conflict {
    pub parties: Vec<String>,
    pub conflict_type: ConflictType,
    pub intensity: f32,
    pub resolution: Option<Resolution>,
    pub origin_tick: u64,
}

pub struct ConflictManager {
    pub active_conflicts: Vec<Conflict>,
    pub resolution_history: Vec<(Conflict, u64)>,
    pub max_active: usize,
}

impl ConflictManager {
    pub fn new() -> Self {
        Self {
            active_conflicts: Vec::new(),
            resolution_history: Vec::new(),
            max_active: 20,
        }
    }

    pub fn detect_conflicts(
        &self,
        agents: &[SimAgent],
        relationships: &RelationshipGraph,
        tick: u64,
    ) -> Vec<Conflict> {
        let mut detected = Vec::new();

        for i in 0..agents.len() {
            if !agents[i].core.alive { continue; }
            for j in (i + 1)..agents.len() {
                if !agents[j].core.alive { continue; }

                let a = &agents[i];
                let b = &agents[j];
                let dist = a.core.position.distance_to(&b.core.position);

                let sentiment = relationships.sentiment_between(&a.core.id, &b.core.id);

                if sentiment < -0.5 && dist < 80.0 {
                    detected.push(Conflict {
                        parties: vec![a.core.id.clone(), b.core.id.clone()],
                        conflict_type: ConflictType::PersonalGrudge,
                        intensity: (-sentiment).abs(),
                        resolution: None,
                        origin_tick: tick,
                    });
                }

                if dist < 30.0 && a.core.hunger > 60.0 && b.core.hunger > 60.0 {
                    detected.push(Conflict {
                        parties: vec![a.core.id.clone(), b.core.id.clone()],
                        conflict_type: ConflictType::ResourceCompetition,
                        intensity: (a.core.hunger + b.core.hunger) / 200.0,
                        resolution: None,
                        origin_tick: tick,
                    });
                }

                if a.personality.aggression > 0.7 && b.personality.aggression > 0.7 && dist < 60.0 {
                    let ideological_clash = (a.personality.cooperativeness - b.personality.cooperativeness).abs();
                    if ideological_clash > 0.4 {
                        detected.push(Conflict {
                            parties: vec![a.core.id.clone(), b.core.id.clone()],
                            conflict_type: ConflictType::IdeologicalClash,
                            intensity: ideological_clash,
                            resolution: None,
                            origin_tick: tick,
                        });
                    }
                }
            }
        }

        detected
    }

    pub fn register_conflict(&mut self, conflict: Conflict) {
        if self.active_conflicts.len() < self.max_active {
            self.active_conflicts.push(conflict);
        }
    }

    pub fn resolve(&mut self, conflict: &mut Conflict, agents: &[SimAgent]) {
        if conflict.resolution.is_some() { return; }

        let resolution = if conflict.intensity > 0.7 {
            let winner_id = self.determine_winner(&conflict.parties, agents);
            Resolution::Dominance { winner: winner_id }
        } else if conflict.intensity > 0.3 {
            Resolution::Compromise { split: 0.5 }
        } else {
            let yielder = conflict.parties[0].clone();
            Resolution::Avoidance { one_yields: yielder }
        };

        conflict.resolution = Some(resolution);
        conflict.intensity *= 0.3;

        self.resolution_history.push((conflict.clone(), 0));
        if self.resolution_history.len() > 100 {
            self.resolution_history.remove(0);
        }
    }

    pub fn escalate(&mut self, conflict: &mut Conflict) {
        conflict.intensity = (conflict.intensity + 0.15).min(1.0);
        if conflict.intensity > 0.9 {
            self.resolve(conflict, &[]);
        }
    }

    fn determine_winner(&self, parties: &[String], agents: &[SimAgent]) -> String {
        parties.iter()
            .max_by(|a_id, b_id| {
                let a_score = agents.iter()
                    .find(|ag| &ag.core.id == *a_id)
                    .map(|ag| ag.core.health + ag.personality.aggression * 20.0)
                    .unwrap_or(0.0);
                let b_score = agents.iter()
                    .find(|ag| &ag.core.id == *b_id)
                    .map(|ag| ag.core.health + ag.personality.aggression * 20.0)
                    .unwrap_or(0.0);
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap_or_else(|| parties[0].clone())
    }

    pub fn cleanup_resolved(&mut self) {
        self.active_conflicts.retain(|c| c.resolution.is_none());
    }

    pub fn active_count(&self) -> usize {
        self.active_conflicts.iter().filter(|c| c.resolution.is_none()).count()
    }
}

impl Default for ConflictManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_agent(id: u64, x: f32, y: f32, aggression: f32) -> SimAgent {
        let mut agent = SimAgent::new(id, Vec2::new(x, y));
        agent.personality.aggression = aggression;
        agent
    }

    #[test]
    fn detect_grudge_conflict() {
        let mut rg = RelationshipGraph::new();
        rg.add_relationship("0", "1", super::super::relationship_graph::RelationshipType::Rival, 0.8);
        rg.update_interaction("0", "1", -0.6, 0);

        let agents = vec![make_agent(0, 0.0, 0.0, 0.5), make_agent(1, 10.0, 0.0, 0.5)];
        let cm = ConflictManager::new();
        let conflicts = cm.detect_conflicts(&agents, &rg, 0);
        assert!(!conflicts.is_empty());
        assert_eq!(conflicts[0].conflict_type, ConflictType::PersonalGrudge);
    }

    #[test]
    fn resolve_dominance() {
        let mut cm = ConflictManager::new();
        let mut conflict = Conflict {
            parties: vec!["a".into(), "b".into()],
            conflict_type: ConflictType::PersonalGrudge,
            intensity: 0.8,
            resolution: None,
            origin_tick: 0,
        };
        let agents = vec![make_agent(0, 0.0, 0.0, 0.9), make_agent(1, 10.0, 0.0, 0.3)];
        cm.resolve(&mut conflict, &agents);
        assert!(conflict.resolution.is_some());
        assert!(matches!(conflict.resolution, Some(Resolution::Dominance { .. })));
    }

    #[test]
    fn escalate_increases_intensity() {
        let mut cm = ConflictManager::new();
        let mut conflict = Conflict {
            parties: vec!["a".into(), "b".into()],
            conflict_type: ConflictType::ResourceCompetition,
            intensity: 0.3,
            resolution: None,
            origin_tick: 0,
        };
        cm.escalate(&mut conflict);
        assert!(conflict.intensity > 0.3);
    }
}
