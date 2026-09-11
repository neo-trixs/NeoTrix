use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::economy::ResourceType;

pub type MessageId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    Predator,
    RivalFaction,
    NaturalDisaster,
    ResourceScarcity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    TradeOffer {
        resource: ResourceType,
        amount: f32,
        price: f32,
    },
    AllianceProposal {
        target_faction: String,
    },
    ThreatWarning {
        source: String,
        threat_type: ThreatType,
    },
    Gossip {
        topic: String,
        reliability: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub sender: String,
    pub channel: String,
    pub content: MessageContent,
    pub timestamp: u64,
    pub importance: f32,
}

pub struct CommunicationChannel {
    pub message_queue: Vec<Message>,
    pub channels: HashMap<String, Vec<MessageId>>,
    pub agent_inbox: HashMap<String, Vec<MessageId>>,
    pub next_id: MessageId,
    pub max_messages: usize,
}

impl CommunicationChannel {
    pub fn new() -> Self {
        Self {
            message_queue: Vec::new(),
            channels: HashMap::new(),
            agent_inbox: HashMap::new(),
            next_id: 0,
            max_messages: 5000,
        }
    }

    pub fn send(&mut self, msg: Message) {
        let id = self.next_id;
        self.next_id += 1;

        let mut msg = msg;
        msg.id = id;

        self.channels
            .entry(msg.channel.clone())
            .or_default()
            .push(id);

        self.agent_inbox
            .entry(msg.sender.clone())
            .or_default()
            .push(id);

        self.message_queue.push(msg);

        if self.message_queue.len() > self.max_messages {
            self.message_queue.remove(0);
        }
    }

    pub fn receive(&self, agent_id: &str, channel: &str) -> Vec<&Message> {
        let channel_ids = self.channels.get(channel)
            .cloned()
            .unwrap_or_default();

        self.message_queue.iter()
            .filter(|m| channel_ids.contains(&m.id))
            .collect()
    }

    pub fn broadcast(
        &mut self,
        sender: &str,
        channel: &str,
        content: MessageContent,
        importance: f32,
        timestamp: u64,
        receivers: &[String],
    ) {
        let msg = Message {
            id: 0,
            sender: sender.to_string(),
            channel: channel.to_string(),
            content,
            timestamp,
            importance,
        };
        self.send(msg);

        for receiver in receivers {
            let relay = Message {
                id: 0,
                sender: sender.to_string(),
                channel: channel.to_string(),
                content: MessageContent::Text("broadcast".to_string()),
                timestamp,
                importance,
            };
            let mut relay = relay;
            relay.sender = sender.to_string();
            self.agent_inbox.entry(receiver.clone()).or_default();
        }
    }

    pub fn create_channel(&mut self, name: String) {
        self.channels.entry(name).or_default();
    }

    pub fn tick(&mut self) {
        self.message_queue.retain(|m| {
            let age = (self.next_id as i64 - m.id as i64) as u64;
            age < 100
        });
    }

    pub fn total_messages(&self) -> usize {
        self.message_queue.len()
    }
}

impl Default for CommunicationChannel {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_text_message() {
        let mut ch = CommunicationChannel::new();
        ch.send(Message {
            id: 0,
            sender: "alice".into(),
            channel: "global".into(),
            content: MessageContent::Text("hello".into()),
            timestamp: 0,
            importance: 0.5,
        });
        assert_eq!(ch.total_messages(), 1);
    }

    #[test]
    fn test_receive_on_channel() {
        let mut ch = CommunicationChannel::new();
        ch.send(Message {
            id: 0,
            sender: "alice".into(),
            channel: "trade".into(),
            content: MessageContent::TradeOffer {
                resource: ResourceType::Food,
                amount: 10.0,
                price: 2.0,
            },
            timestamp: 0,
            importance: 0.7,
        });
        let msgs = ch.receive("bob", "trade");
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_create_channel() {
        let mut ch = CommunicationChannel::new();
        ch.create_channel("war_council".into());
        assert!(ch.channels.contains_key("war_council"));
    }

    #[test]
    fn test_threat_warning() {
        let mut ch = CommunicationChannel::new();
        ch.send(Message {
            id: 0,
            sender: "scout".into(),
            channel: "alerts".into(),
            content: MessageContent::ThreatWarning {
                source: "scout".into(),
                threat_type: ThreatType::RivalFaction,
            },
            timestamp: 10,
            importance: 0.9,
        });
        let msgs = ch.receive("leader", "alerts");
        assert_eq!(msgs.len(), 1);
    }
}
