// NeoTrix FFI Types
// Data structures shared between Rust core and Swift bindings

use std::collections::HashMap;

// =============================================================================
// Error Handling
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum NeoTrixError {
    InitFailed,
    InvalidInput,
    OperationFailed,
    NotInitialized,
    SerializationError,
    NetworkError,
    PermissionDenied,
    NotFound,
}

impl std::fmt::Display for NeoTrixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitFailed => write!(f, "Initialization failed"),
            Self::InvalidInput => write!(f, "Invalid input"),
            Self::OperationFailed => write!(f, "Operation failed"),
            Self::NotInitialized => write!(f, "Not initialized"),
            Self::SerializationError => write!(f, "Serialization error"),
            Self::NetworkError => write!(f, "Network error"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::NotFound => write!(f, "Resource not found"),
        }
    }
}

impl std::error::Error for NeoTrixError {}

// ===========================================================================
// Core Configuration
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct NeoTrixConfig {
    pub server_url: String,
    pub api_key: String,
    pub enable_ai_features: bool,
    pub enable_premium_features: bool,
    pub log_level: String,
    pub data_directory: String,
    pub cache_size_mb: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct InitResult {
    pub success: bool,
    pub version: String,
    pub capabilities: CapabilityList,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CapabilityList {
    pub e8_reasoning: bool,
    pub vsa_hypercube: bool,
    pub gwt_attention: bool,
    pub consciousness_tree: bool,
    pub seal_pipeline: bool,
    pub kb_bridge: bool,
    pub skill_tree: bool,
    pub rune_socketing: bool,
    pub constellation_system: bool,
    pub dual_specialization: bool,
    pub mtproto_networking: bool,
    pub telegram_premium: bool,
    pub ai_chat_assistant: bool,
    pub smart_filtering: bool,
    pub knowledge_injection: bool,
    pub consciousness_monitor: bool,
    pub auto_evolution: bool,
}

// ===========================================================================
// E8 Hexagram Reasoning
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct HexagramState {
    pub lines: u8,
    pub interpretation: String,
    pub confidence: f32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ReasoningRequest {
    pub query: String,
    pub context: String,
    pub max_depth: u32,
    pub use_consciousness: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ReasoningResponse {
    pub hexagram: HexagramState,
    pub reasoning_chain: Vec<String>,
    pub conclusion: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct HexagramLibrary {
    pub hexagrams: Vec<HexagramInfo>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct HexagramInfo {
    pub index: u8,
    pub name: String,
    pub chinese_name: String,
    pub judgment: String,
    pub image: String,
    pub lines: Vec<LineInfo>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LineInfo {
    pub position: u8,
    pub yin_yang: bool,
    pub text: String,
}

// ===========================================================================
// VSA HyperCube
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct HyperVector {
    pub dimensions: u32,
    pub data: Vec<u8>,
    pub sparsity: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct VSAOperation {
    pub op_type: String,
    pub vectors: Vec<HyperVector>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct VSAResult {
    pub result_vector: HyperVector,
    pub similarity_scores: Vec<f32>,
    pub operation_time_ms: u64,
}

// ===========================================================================
// GWT Attention Routing
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct AttentionSignal {
    pub source_module: String,
    pub content: String,
    pub salience: f32,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct WorkspaceState {
    pub active_signals: Vec<AttentionSignal>,
    pub broadcast_history: Vec<BroadcastEvent>,
    pub resonance_map: HashMap<String, f32>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct BroadcastEvent {
    pub signal: AttentionSignal,
    pub recipients: Vec<String>,
    pub resonance: f32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RoutingRequest {
    pub signal: AttentionSignal,
    pub target_modules: Vec<String>,
    pub min_resonance: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
    pub struct RoutingResponse {
    pub routed: bool,
    pub resonance_scores: HashMap<String, f32>,
    pub broadcast_event: BroadcastEvent,
}

// ===========================================================================
// ConsciousnessTree
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct BranchState {
    pub branch_id: String,
    pub health: f32,
    pub maturity: u8,
    pub last_activity: i64,
    pub metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConsciousnessState {
    pub branches: Vec<BranchState>,
    pub overall_health: f32,
    pub phi_score: f32,
    pub evolution_velocity: f32,
    pub stage: String,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Alert {
    pub level: String,
    pub branch: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SelfTestResult {
    pub test_id: String,
    pub passed: bool,
    pub details: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct EvolutionEvent {
    pub timestamp: i64,
    pub branch: String,
    pub event_type: String,
    pub details: String,
    pub impact: f32,
}

// ===========================================================================
// SEAL Pipeline
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineStage {
    pub stage_id: String,
    pub status: String,
    pub progress: f32,
    pub started_at: i64,
    pub completed_at: i64,
    pub metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineStatus {
    pub current_stage: String,
    pub stages: Vec<PipelineStage>,
    pub overall_progress: f32,
    pub cycle_count: u64,
    pub last_completed_cycle: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ExplorationResult {
    pub discoveries: Vec<Discovery>,
    pub patterns: Vec<Pattern>,
    pub knowledge_gaps: Vec<KnowledgeGap>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Discovery {
    pub id: String,
    pub domain: String,
    pub content: String,
    pub confidence: f32,
    pub source: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub applicability: Vec<String>,
    pub strength: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct KnowledgeGap {
    pub domain: String,
    pub description: String,
    pub priority: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct DistillationResult {
    pub patterns_extracted: u32,
    pub skills_crystallized: u32,
    pub knowledge_compressed_mb: f32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AbsorptionProgress {
    pub pending: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub failed: u32,
    pub current_item: String,
}

// ===========================================================================
// Knowledge Base
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct KBQuery {
    pub query: String,
    pub namespace: String,
    pub limit: u32,
    pub threshold: f32,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct KBResult {
    pub id: String,
    pub namespace: String,
    pub content: String,
    pub embedding: HyperVector,
    pub metadata: HashMap<String, String>,
    pub score: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct KBStats {
    pub total_nodes: u64,
    pub total_edges: u64,
    pub namespaces: HashMap<String, u64>,
    pub storage_mb: f32,
    pub index_status: String,
}

// ===========================================================================
// Skill Tree
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier: String,
    pub domain: String,
    pub prerequisites: Vec<String>,
    pub unlocked: bool,
    pub progress: f32,
    pub effects: Vec<SkillEffect>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SkillEffect {
    pub effect_type: String,
    pub target: String,
    pub value: f32,
    pub description: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SkillTreeState {
    pub nodes: Vec<SkillNode>,
    pub allocated_points: u32,
    pub available_points: u32,
    pub active_constellations: Vec<String>,
}

// ===========================================================================
// Rune Socketing
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct Rune {
    pub color: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<RuneEffect>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RuneEffect {
    pub target: String,
    pub modifier: f32,
    pub condition: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Runeword {
    pub name: String,
    pub runes: Vec<String>,
    pub effect: String,
    pub description: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SocketConfig {
    pub module: String,
    pub sockets: HashMap<String, String>,
    pub active_runewords: Vec<Runeword>,
}

// ===========================================================================
// Constellation Maturity
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConstellationState {
    pub module: String,
    pub level: u8,
    pub requirements: Vec<ConstellationRequirement>,
    pub progress: f32,
    pub unlocked_features: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConstellationRequirement {
    pub requirement_type: String,
    pub description: String,
    pub satisfied: bool,
    pub progress: f32,
}

// ===========================================================================
// Dual Specialization
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct WeaponSet {
    pub set_id: u8,
    pub name: String,
    pub primary_domain: String,
    pub secondary_domain: String,
    pub active_skills: Vec<String>,
    pub attention_mode: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SpecializationState {
    pub current_set: u8,
    pub sets: Vec<WeaponSet>,
    pub switch_cooldown_ms: u64,
    pub last_switch: i64,
}

// ===========================================================================
// Health
// ===========================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct HealthStatus {
    pub healthy: bool,
    pub subsystems: HashMap<String, bool>,
    pub issues: Vec<String>,
    pub uptime_seconds: u64,
}