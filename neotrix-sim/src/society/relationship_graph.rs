// RelationshipGraph - Social network between agents
// Tracks relationships, trust, sentiment, and social influence

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    Friend,
    Rival,
    Mate,
    Trade,
    Family,
    Leader,
    Follower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub rel_type: RelationshipType,
    pub strength: f32,      // 0-1
    pub trust: f32,         // -1 to 1
    pub interactions: u32,
    pub last_interaction_tick: u64,
}

pub struct RelationshipGraph {
    edges: Vec<Relationship>,
    adjacency: HashMap<String, Vec<usize>>,
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self { edges: Vec::new(), adjacency: HashMap::new() }
    }

    pub fn add_relationship(&mut self, from: &str, to: &str, rel_type: RelationshipType, strength: f32) {
        // Check if exists
        if let Some(indices) = self.adjacency.get(from) {
            for &idx in indices {
                if self.edges[idx].to == to {
                    self.edges[idx].strength = strength;
                    self.edges[idx].rel_type = rel_type;
                    return;
                }
            }
        }
        let idx = self.edges.len();
        self.edges.push(Relationship {
            from: from.to_string(),
            to: to.to_string(),
            rel_type,
            strength,
            trust: 0.0,
            interactions: 0,
            last_interaction_tick: 0,
        });
        self.adjacency.entry(from.to_string()).or_default().push(idx);
        self.adjacency.entry(to.to_string()).or_default().push(idx);
    }

    pub fn update_interaction(&mut self, from: &str, to: &str, trust_delta: f32, tick: u64) {
        if let Some(indices) = self.adjacency.get(from) {
            for &idx in indices {
                if self.edges[idx].to == to || self.edges[idx].from == to {
                    self.edges[idx].trust = (self.edges[idx].trust + trust_delta).clamp(-1.0, 1.0);
                    self.edges[idx].interactions += 1;
                    self.edges[idx].last_interaction_tick = tick;
                    self.edges[idx].strength = (self.edges[idx].strength + 0.05).min(1.0);
                    return;
                }
            }
        }
    }

    pub fn neighbors(&self, agent_id: &str) -> Vec<&Relationship> {
        self.adjacency.get(agent_id).map(|indices| {
            indices.iter().map(|&idx| &self.edges[idx]).collect()
        }).unwrap_or_default()
    }

    pub fn sentiment_between(&self, a: &str, b: &str) -> f32 {
        if let Some(indices) = self.adjacency.get(a) {
            for &idx in indices {
                if self.edges[idx].to == b || self.edges[idx].from == b {
                    return self.edges[idx].trust;
                }
            }
        }
        0.0
    }

    pub fn total_relationships(&self) -> usize { self.edges.len() }
    pub fn edges(&self) -> &[Relationship] { &self.edges }
}

impl Default for RelationshipGraph {
    fn default() -> Self { Self::new() }
}
