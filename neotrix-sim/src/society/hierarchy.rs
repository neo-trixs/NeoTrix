use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agents::sim_agent::SimAgent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderRecord {
    pub agent_id: String,
    pub faction: String,
    pub elected_tick: u64,
    pub challenges_won: u32,
}

pub struct Hierarchy {
    pub leaders: HashMap<String, LeaderRecord>,
    pub dominance_scores: HashMap<String, f32>,
    pub reputation: HashMap<String, f32>,
    pub challenge_history: Vec<ChallengeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub challenger: String,
    pub leader: String,
    pub faction: String,
    pub result: bool,
    pub tick: u64,
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            leaders: HashMap::new(),
            dominance_scores: HashMap::new(),
            reputation: HashMap::new(),
            challenge_history: Vec::new(),
        }
    }

    pub fn update(&mut self, agents: &[SimAgent], tick: u64) {
        for agent in agents {
            if !agent.core.alive { continue; }
            let id = &agent.core.id;

            let dominance = self.dominance_scores.entry(id.clone()).or_insert(0.0);
            *dominance = agent.personality.aggression * 0.4
                + agent.personality.cooperativeness * 0.3
                + (agent.core.health / 100.0) * 0.2
                + (agent.core.energy / 100.0) * 0.1;

            let rep = self.reputation.entry(id.clone()).or_insert(0.0);
            if agent.personality.cooperativeness > 0.6 {
                *rep = (*rep + 0.02).min(1.0);
            }
            if agent.personality.aggression > 0.7 {
                *rep = (*rep - 0.01).max(-1.0);
            }
        }

        for record in self.leaders.values_mut() {
            if let Some(score) = self.dominance_scores.get(&record.agent_id) {
                if *score < 0.2 {
                    record.challenges_won = 0;
                }
            }
        }
    }

    pub fn elect_leader(&self, faction: &str) -> Option<String> {
        let candidates: Vec<(&String, &f32)> = self.dominance_scores.iter()
            .filter(|(id, _)| {
                self.leaders.values()
                    .find(|l| l.faction == faction && l.agent_id.as_str() == id.as_str())
                    .is_some()
                    || self.reputation.get(*id).copied().unwrap_or(0.0) > 0.3
            })
            .collect();

        candidates.into_iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id.clone())
    }

    pub fn assign_leader(&mut self, faction: &str, agent_id: &str, tick: u64) {
        self.leaders.insert(faction.to_string(), LeaderRecord {
            agent_id: agent_id.to_string(),
            faction: faction.to_string(),
            elected_tick: tick,
            challenges_won: 0,
        });
    }

    pub fn challenge(&mut self, challenger_id: &str, leader_faction: &str, tick: u64) -> bool {
        let leader_id = match self.leaders.get(leader_faction) {
            Some(l) => l.agent_id.clone(),
            None => return false,
        };

        let challenger_score = self.dominance_scores.get(challenger_id).copied().unwrap_or(0.0);
        let leader_score = self.dominance_scores.get(&leader_id).copied().unwrap_or(0.0);

        let challenger_wins = challenger_score > leader_score * 1.1;

        self.challenge_history.push(ChallengeRecord {
            challenger: challenger_id.to_string(),
            leader: leader_id.clone(),
            faction: leader_faction.to_string(),
            result: challenger_wins,
            tick,
        });

        if challenger_wins {
            self.leaders.insert(leader_faction.to_string(), LeaderRecord {
                agent_id: challenger_id.to_string(),
                faction: leader_faction.to_string(),
                elected_tick: tick,
                challenges_won: 1,
            });

            let challenger_rep = self.reputation.entry(challenger_id.to_string()).or_insert(0.0);
            *challenger_rep = (*challenger_rep + 0.3).min(1.0);
        }

        challenger_wins
    }

    pub fn get_leader(&self, faction: &str) -> Option<&str> {
        self.leaders.get(faction).map(|l| l.agent_id.as_str())
    }

    pub fn get_dominance(&self, agent_id: &str) -> f32 {
        self.dominance_scores.get(agent_id).copied().unwrap_or(0.0)
    }

    pub fn get_reputation(&self, agent_id: &str) -> f32 {
        self.reputation.get(agent_id).copied().unwrap_or(0.0)
    }
}

impl Default for Hierarchy {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_leader_agent(id: u64, aggression: f32, coop: f32) -> SimAgent {
        let mut agent = SimAgent::new(id, Vec2::new(0.0, 0.0));
        agent.personality.aggression = aggression;
        agent.personality.cooperativeness = coop;
        agent.core.health = 80.0;
        agent.core.energy = 80.0;
        agent
    }

    #[test]
    fn test_elect_leader() {
        let mut h = Hierarchy::new();
        let agents = vec![
            make_leader_agent(0, 0.8, 0.6),
            make_leader_agent(1, 0.5, 0.7),
        ];
        h.update(&agents, 0);
        h.assign_leader("warriors", "0", 0);

        let leader = h.elect_leader("warriors");
        assert!(leader.is_some());
    }

    #[test]
    fn test_challenge_succeeds() {
        let mut h = Hierarchy::new();
        let agents = vec![
            make_leader_agent(0, 0.3, 0.5),
            make_leader_agent(1, 0.9, 0.5),
        ];
        h.update(&agents, 0);
        h.assign_leader("warriors", "0", 0);

        let won = h.challenge("1", "warriors", 1);
        assert!(won);
        assert_eq!(h.get_leader("warriors").unwrap(), "1");
    }

    #[test]
    fn test_challenge_fails() {
        let mut h = Hierarchy::new();
        let agents = vec![
            make_leader_agent(0, 0.9, 0.5),
            make_leader_agent(1, 0.3, 0.5),
        ];
        h.update(&agents, 0);
        h.assign_leader("warriors", "0", 0);

        let won = h.challenge("1", "warriors", 1);
        assert!(!won);
        assert_eq!(h.get_leader("warriors").unwrap(), "0");
    }
}
