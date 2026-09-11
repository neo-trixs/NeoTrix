use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: u32,
    pub receiver: Option<u32>,
    pub channel: String,
    pub content: String,
    pub priority: u8,
    pub timestamp: u64,
}

pub struct CommunicationChannel {
    pub messages: Vec<Message>,
    pub channels: HashMap<String, Vec<u64>>,
    pub agent_inbox: HashMap<u32, Vec<u64>>,
    pub next_id: u64,
    pub max_messages: usize,
}

impl CommunicationChannel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            channels: HashMap::new(),
            agent_inbox: HashMap::new(),
            next_id: 0,
            max_messages: 5000,
        }
    }

    pub fn send(&mut self, sender: u32, receiver: Option<u32>, channel: &str, content: &str, priority: u8, timestamp: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let msg = Message {
            id,
            sender,
            receiver,
            channel: channel.to_string(),
            content: content.to_string(),
            priority,
            timestamp,
        };

        self.messages.push(msg);

        self.channels.entry(channel.to_string()).or_default().push(id);

        if let Some(recv) = receiver {
            self.agent_inbox.entry(recv).or_default().push(id);
        }

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }

        id
    }

    pub fn receive(&self, agent_id: u32) -> Vec<&Message> {
        let inbox = self.agent_inbox.get(&agent_id).cloned().unwrap_or_default();
        self.messages.iter().filter(|m| inbox.contains(&m.id)).collect()
    }

    pub fn receive_channel(&self, channel: &str) -> Vec<&Message> {
        let ids = self.channels.get(channel).cloned().unwrap_or_default();
        self.messages.iter().filter(|m| ids.contains(&m.id)).collect()
    }

    pub fn broadcast(&mut self, sender: u32, channel: &str, content: &str, priority: u8, timestamp: u64, receivers: &[u32]) -> Vec<u64> {
        receivers.iter()
            .map(|&r| self.send(sender, Some(r), channel, content, priority, timestamp))
            .collect()
    }

    pub fn tick(&mut self) {
        self.messages.retain(|m| {
            let age = (self.next_id as i64 - m.id as i64) as u64;
            age < 100
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_send() {
        let mut channel = CommunicationChannel::new();
        let id = channel.send(0, Some(1), "chat", "Hello", 1, 0);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_message_receive() {
        let mut channel = CommunicationChannel::new();
        channel.send(0, Some(1), "chat", "Hello", 1, 0);
        let msgs = channel.receive(1);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_broadcast() {
        let mut channel = CommunicationChannel::new();
        let ids = channel.broadcast(0, "global", "Announcement", 2, 0, &[1, 2, 3]);
        assert_eq!(ids.len(), 3);
    }
}
