use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub id: u64,
    pub source_agent: u32,
    pub topic: String,
    pub content: String,
    pub reliability: f32,
    pub spread_count: u32,
    pub max_spread: u32,
    pub created_tick: u64,
}

pub struct GossipProtocol {
    pub messages: Vec<GossipMessage>,
    pub agent_memory: HashMap<u32, Vec<u64>>,
    pub next_id: u64,
    pub max_messages: usize,
    pub decay_rate: f32,
}

impl GossipProtocol {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            agent_memory: HashMap::new(),
            next_id: 0,
            max_messages: 1000,
            decay_rate: 0.1,
        }
    }

    pub fn create_message(
        &mut self,
        source: u32,
        topic: &str,
        content: &str,
        reliability: f32,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let msg = GossipMessage {
            id,
            source_agent: source,
            topic: topic.to_string(),
            content: content.to_string(),
            reliability,
            spread_count: 0,
            max_spread: 5,
            created_tick: 0,
        };

        self.messages.push(msg);
        self.agent_memory.entry(source).or_default().push(id);

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }

        id
    }

    pub fn spread(&mut self, agent_id: u32) -> Vec<GossipMessage> {
        let known: Vec<u64> = self
            .agent_memory
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();

        let mut to_spread = Vec::new();
        for msg in &mut self.messages {
            if !known.contains(&msg.id) && msg.spread_count < msg.max_spread {
                msg.spread_count += 1;
                to_spread.push(msg.clone());
            }
        }

        self.agent_memory
            .entry(agent_id)
            .or_default()
            .extend(to_spread.iter().map(|m| m.id));

        to_spread
    }

    pub fn receive(&mut self, agent_id: u32, message: GossipMessage) -> bool {
        let known = self
            .agent_memory
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();
        if known.contains(&message.id) {
            return false;
        }

        self.messages.push(message.clone());
        self.agent_memory
            .entry(agent_id)
            .or_default()
            .push(message.id);
        true
    }

    pub fn decay(&mut self) {
        for msg in &mut self.messages {
            msg.reliability *= 1.0 - self.decay_rate;
        }
        self.messages.retain(|m| m.reliability > 0.01);
    }

    pub fn get_messages_for_agent(&self, agent_id: u32) -> Vec<&GossipMessage> {
        let known = self
            .agent_memory
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();
        self.messages
            .iter()
            .filter(|m| known.contains(&m.id))
            .collect()
    }

    pub fn get_topic_messages(&self, topic: &str) -> Vec<&GossipMessage> {
        self.messages.iter().filter(|m| m.topic == topic).collect()
    }

    pub fn propagate(
        &mut self,
        agents: &[crate::agents::sim_agent::SimAgent],
        relationships: &super::relationship_graph::RelationshipGraph,
    ) {
        let alive_ids: Vec<u32> = agents.iter()
            .filter(|a| a.core.alive)
            .map(|a| a.id as u32)
            .collect();

        for &agent_id in &alive_ids {
            let neighbors: Vec<String> = relationships.neighbors(&agent_id.to_string())
                .iter()
                .filter(|r| r.trust > 0.0)
                .map(|r| r.to.clone())
                .collect();

            let spreadable: Vec<GossipMessage> = self.messages.iter()
                .filter(|m| m.source_agent != agent_id && m.spread_count < m.max_spread)
                .filter(|m| {
                    let agent_id_str = agent_id.to_string();
                    let known = self.agent_memory.get(&agent_id)
                        .map(|v| v.clone())
                        .unwrap_or_default();
                    !known.contains(&m.id)
                })
                .cloned()
                .collect();

            for msg in spreadable {
                if self.rng_f32() < 0.3 {
                    let mut new_msg = msg.clone();
                    new_msg.spread_count += 1;
                    new_msg.reliability *= 0.9;
                    self.messages.push(new_msg.clone());
                    self.agent_memory.entry(agent_id).or_default().push(new_msg.id);
                }
            }

            for neighbor_id in &neighbors {
                if let Ok(nid) = neighbor_id.parse::<u32>() {
                    let trust = relationships.sentiment_between(
                        &agent_id.to_string(), neighbor_id,
                    );
                    if trust > 0.2 {
                        let known = self.agent_memory.get(&agent_id)
                            .map(|v| v.clone())
                            .unwrap_or_default();
                        let to_forward: Vec<GossipMessage> = self.messages.iter()
                            .filter(|m| known.contains(&m.id) && m.spread_count < m.max_spread)
                            .cloned()
                            .collect();
                        for msg in to_forward {
                            if self.rng_f32() < trust * 0.5 {
                                self.receive(nid, msg);
                            }
                        }
                    }
                }
            }
        }
    }

    fn rng_f32(&self) -> f32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.next_id.hash(&mut hasher);
        (hasher.finish() % 1000) as f32 / 1000.0
    }

    pub fn tick(&mut self) {
        self.decay();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_creation() {
        let mut protocol = GossipProtocol::new();
        let id = protocol.create_message(0, "food", "Found berries", 0.8);
        assert_eq!(id, 0);
        assert_eq!(protocol.messages.len(), 1);
    }

    #[test]
    fn test_gossip_spread() {
        let mut protocol = GossipProtocol::new();
        protocol.create_message(0, "food", "Found berries", 0.8);

        let spread = protocol.spread(1);
        assert_eq!(spread.len(), 1);

        let spread2 = protocol.spread(1);
        assert_eq!(spread2.len(), 0);
    }

    #[test]
    fn test_gossip_receive() {
        let mut protocol = GossipProtocol::new();
        let msg = GossipMessage {
            id: 0,
            source_agent: 0,
            topic: "food".to_string(),
            content: "Found berries".to_string(),
            reliability: 0.8,
            spread_count: 1,
            max_spread: 5,
            created_tick: 0,
        };

        assert!(protocol.receive(1, msg.clone()));
        assert!(!protocol.receive(1, msg));
    }

    #[test]
    fn test_gossip_decay() {
        let mut protocol = GossipProtocol::new();
        protocol.create_message(0, "food", "Found berries", 1.0);
        protocol.decay();
        assert!(protocol.messages[0].reliability < 1.0);
    }
}
