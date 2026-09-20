use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Channel — Grok Bot / Slack-style named spaces for organized work.
///
/// Agents and humans communicate through channels. Each channel has
/// a topic, members, and message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub topic: String,
    pub kind: ChannelKind,
    pub members: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChannelKind {
    /// Open channel visible to all.
    Public,
    /// Restricted to listed members.
    Private,
    /// DM between two entities.
    Direct,
}

/// Thread — a reply chain within a channel (Slack threads).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub channel_id: String,
    pub parent_message_id: String,
    pub reply_count: u32,
    pub participants: Vec<String>,
}

/// Message in a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel_id: String,
    pub thread_id: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub kind: MessageKind,
    pub sequence: u64,
    pub timestamp: String,
    pub reactions: Vec<Reaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageKind {
    Text,
    System,
    Artifact,
    ToolCall,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub user_ids: Vec<String>,
}

/// Channel manager — coordinates multi-agent communication.
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
    messages: HashMap<String, Vec<ChannelMessage>>,
    threads: HashMap<String, Thread>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            messages: HashMap::new(),
            threads: HashMap::new(),
        }
    }

    /// Create a new channel.
    pub fn create_channel(&mut self, name: String, topic: String, kind: ChannelKind) -> Channel {
        let channel = Channel {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            topic,
            kind,
            members: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        tracing::info!("created channel: {name}");
        self.channels.insert(channel.id.clone(), channel.clone());
        channel
    }

    /// Add a member to a channel.
    pub fn add_member(&mut self, channel_id: &str, member_id: &str) {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            if !channel.members.contains(&member_id.to_string()) {
                channel.members.push(member_id.to_string());
            }
        }
    }

    /// Post a message to a channel.
    pub fn post_message(
        &mut self,
        channel_id: &str,
        author_id: &str,
        author_name: &str,
        content: String,
        kind: MessageKind,
    ) -> Option<ChannelMessage> {
        let channel = self.channels.get(channel_id)?;
        let sequence = self
            .messages
            .get(channel_id)
            .map(|m| m.len() as u64 + 1)
            .unwrap_or(1);

        let msg = ChannelMessage {
            id: uuid::Uuid::new_v4().to_string(),
            channel_id: channel_id.to_string(),
            thread_id: None,
            author_id: author_id.to_string(),
            author_name: author_name.to_string(),
            content,
            kind,
            sequence,
            timestamp: chrono::Utc::now().to_rfc3339(),
            reactions: Vec::new(),
        };

        self.messages
            .entry(channel_id.to_string())
            .or_default()
            .push(msg.clone());

        Some(msg)
    }

    /// Get messages in a channel.
    pub fn get_messages(&self, channel_id: &str) -> Vec<&ChannelMessage> {
        self.messages
            .get(channel_id)
            .map(|m| m.iter().collect())
            .unwrap_or_default()
    }

    /// List channels a member belongs to.
    pub fn list_member_channels(&self, member_id: &str) -> Vec<&Channel> {
        self.channels
            .values()
            .filter(|c| c.members.contains(&member_id.to_string()))
            .collect()
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}
