use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::engine::ModelTier;

/// Bot Identity — a persistent agent with its own memory, tools, and workspace.
/// Inspired by Grok Bot: "Bots are persistent agents with their own identity,
/// memory, runtime, and tools."

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotIdentity {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,       // URL or emoji
    pub description: String,
    pub created_at: u64,
    pub last_active_at: u64,

    /// What the bot is doing right now
    pub status: BotStatus,

    /// Skills this bot has enabled
    pub skills: Vec<String>,

    /// Tools this bot can use
    pub tools: Vec<ToolPermission>,

    /// Accumulated memory/preferences
    pub memory: BotMemory,

    /// Which model tier to use
    pub model_tier: ModelTier,

    /// Work channels this bot belongs to
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BotStatus {
    Idle,
    Thinking,
    Working { current_action: String },
    WaitingForApproval { action: String },
    Blocked { reason: String },
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub name: String,
    pub requires_approval: bool,
    pub scopes: Vec<String>,
}

/// Persistent memory for a bot — preferences, lessons, context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotMemory {
    /// User preferences learned over time (report format, style, etc.)
    pub preferences: HashMap<String, String>,
    /// Lessons from corrections
    pub lessons: Vec<MemoryLesson>,
    /// Project context
    pub project_context: Vec<String>,
    /// Previous task history (last N tasks)
    pub task_history: Vec<TaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLesson {
    pub learned_at: u64,
    pub context: String,
    pub lesson: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: String,
    pub description: String,
    pub completed_at: u64,
    pub success: bool,
    pub summary: String,
}

/// A Channel — named workspace for organizing bot work (like Slack channels)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: u64,
    pub bot_ids: Vec<String>,
    pub message_count: u64,
}

/// Thread — conversational thread within a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub channel_id: String,
    pub title: Option<String>,
    pub created_at: u64,
    pub participants: Vec<String>,  // bot IDs
    pub message_count: u64,
    pub parent_thread_id: Option<String>,  // for nested threads
}

/// Artifact — durable output created by a bot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub bot_id: String,
    pub artifact_type: ArtifactType,
    pub title: String,
    pub content_path: String,   // file path on disk
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Document,
    Code,
    Design,
    Data,
    Report,
    Spreadsheet,
    Image,
    Other,
}

/// Bot Manager — manages all bot identities, channels, and coordination
pub struct BotManager {
    bots: HashMap<String, BotIdentity>,
    channels: HashMap<String, Channel>,
    threads: HashMap<String, Thread>,
}

impl BotManager {
    pub fn new() -> Self {
        Self {
            bots: HashMap::new(),
            channels: HashMap::new(),
            threads: HashMap::new(),
        }
    }

    /// Create a new bot with a specific role/personality
    pub fn create_bot(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        model_tier: ModelTier,
    ) -> Result<&BotIdentity, String> {
        let id = format!("bot_{}", uuid::Uuid::new_v4());
        let now = timestamp();
        let bot = BotIdentity {
            id: id.clone(),
            name: name.into(),
            avatar: None,
            description: description.into(),
            created_at: now,
            last_active_at: now,
            status: BotStatus::Idle,
            skills: Vec::new(),
            tools: Vec::new(),
            memory: BotMemory::default(),
            model_tier,
            channels: Vec::new(),
        };
        self.bots.insert(id.clone(), bot);
        self.bots.get(&id).ok_or("Bot not found after insert".into())
    }

    /// Create a channel for organizing work
    pub fn create_channel(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Result<&Channel, String> {
        let id = format!("ch_{}", uuid::Uuid::new_v4());
        let channel = Channel {
            id: id.clone(),
            name: name.into(),
            description,
            created_at: timestamp(),
            bot_ids: Vec::new(),
            message_count: 0,
        };
        self.channels.insert(id.clone(), channel);
        self.channels.get(&id).ok_or("Channel not found after insert".into())
    }

    /// Add a bot to a channel
    pub fn add_bot_to_channel(&mut self, bot_id: &str, channel_id: &str) -> Result<(), String> {
        let bot = self.bots.get_mut(bot_id).ok_or("Bot not found")?;
        let channel = self.channels.get_mut(channel_id).ok_or("Channel not found")?;
        if !bot.channels.contains(&channel_id.to_string()) {
            bot.channels.push(channel_id.to_string());
        }
        if !channel.bot_ids.contains(&bot_id.to_string()) {
            channel.bot_ids.push(bot_id.to_string());
        }
        Ok(())
    }

    /// Update bot status
    pub fn update_status(&mut self, bot_id: &str, status: BotStatus) -> Result<(), String> {
        let bot = self.bots.get_mut(bot_id).ok_or("Bot not found")?;
        bot.status = status;
        bot.last_active_at = timestamp();
        Ok(())
    }

    /// Record a lesson learned by a bot
    pub fn record_lesson(&mut self, bot_id: &str, context: String, lesson: String) -> Result<(), String> {
        let bot = self.bots.get_mut(bot_id).ok_or("Bot not found")?;
        bot.memory.lessons.push(MemoryLesson {
            learned_at: timestamp(),
            context,
            lesson,
            confidence: 0.8,
        });
        Ok(())
    }

    pub fn list_bots(&self) -> Vec<&BotIdentity> { self.bots.values().collect() }
    pub fn list_channels(&self) -> Vec<&Channel> { self.channels.values().collect() }
    pub fn get_bot(&self, id: &str) -> Option<&BotIdentity> { self.bots.get(id) }
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
