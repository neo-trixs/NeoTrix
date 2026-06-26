use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub description: String,
    pub detected_elements: Vec<String>,
    pub dominant_colors: Vec<String>,
    pub layout_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transcription {
    pub text: String,
    pub language: String,
    pub confidence: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub size_delta: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensoryEventKind {
    Visual(AnalysisReport),
    Auditory(Transcription),
    Data(FileChange),
    Conversation(ConversationTurn),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensoryEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub kind: SensoryEventKind,
    pub source: String,
    pub priority: u8,
    pub confidence: f64,
    pub description: String,
    pub raw_data_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryMemory {
    pub events: Vec<SensoryEvent>,
    pub max_events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttentionTrigger {
    NovelEvent,
    RepeatedPattern,
    HighPriority,
    AnomalyDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMapping {
    pub mappings: HashMap<String, Vec<AttentionTrigger>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub turn_number: usize,
    pub user_message: String,
    pub system_response: String,
    pub intent_label: Option<String>,
    pub user_satisfaction: Option<f64>,
    pub duration_ms: u64,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationObserver {
    pub turns: Vec<ConversationTurn>,
    pub current_topic: Option<String>,
    pub topic_shifts: usize,
    pub max_turns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodViewReport {
    pub total_turns: usize,
    pub topic_shift_count: usize,
    pub avg_user_satisfaction: Option<f64>,
    pub dominant_intent: Option<String>,
    pub efficiency_ratio: f64,
    pub patterns_detected: Vec<String>,
    pub meta_insight: String,
    pub consciousness_dimension_scores: [f64; 3],
    pub dialogue_arc: Vec<String>,
    pub sentiment_trend: f64,
    pub repetition_detected: bool,
    pub repeated_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueArcAnalysis {
    pub phases: Vec<String>,
    pub sentiment_trend: f64,
    pub topic_shifts: usize,
    pub repetitions: Vec<(String, usize)>,
    pub tool_density: Vec<f64>,
    pub engagement_trend: f64,
    pub quality_per_phase: Vec<f64>,
}
