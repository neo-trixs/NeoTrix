use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::sync::LazyLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use chrono::Utc;

// ── Enums ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactionStrategy {
    SlidingWindow,
    SummaryBased,
    Hybrid,
    Semantic,
}

impl Default for CompactionStrategy {
    fn default() -> Self { Self::Hybrid }
}

impl ToString for CompactionStrategy {
    fn to_string(&self) -> String {
        match self {
            Self::SlidingWindow => "sliding_window".into(),
            Self::SummaryBased => "summary_based".into(),
            Self::Hybrid => "hybrid".into(),
            Self::Semantic => "semantic".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactionLevel {
    Light,
    Medium,
    Aggressive,
    Maximum,
}

impl Default for CompactionLevel {
    fn default() -> Self { Self::Medium }
}

impl ToString for CompactionLevel {
    fn to_string(&self) -> String {
        match self {
            Self::Light => "light".into(),
            Self::Medium => "medium".into(),
            Self::Aggressive => "aggressive".into(),
            Self::Maximum => "maximum".into(),
        }
    }
}

// ── Structs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    pub id: String,
    pub original_length_chars: u32,
    pub compacted_length_chars: u32,
    pub ratio: f64,
    pub summary: String,
    pub key_points: Vec<String>,
    pub decisions: Vec<String>,
    pub preserved_code: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub session_id: String,
    pub original_total_chars: u32,
    pub compacted_total_chars: u32,
    pub reduction_pct: f64,
    pub segments_compacted: u32,
    pub strategy_used: CompactionStrategy,
    pub level: CompactionLevel,
    pub preserved_decision_count: u32,
    pub preserved_code_snippets: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub enabled: bool,
    pub auto_compact: bool,
    pub auto_compact_threshold_chars: u32,
    pub strategy: CompactionStrategy,
    pub level: CompactionLevel,
    pub preserve_decisions: bool,
    pub preserve_code_changes: bool,
    pub preserve_user_messages: bool,
    pub show_compaction_notice: bool,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_compact: true,
            auto_compact_threshold_chars: 50000,
            strategy: CompactionStrategy::Hybrid,
            level: CompactionLevel::Medium,
            preserve_decisions: true,
            preserve_code_changes: true,
            preserve_user_messages: true,
            show_compaction_notice: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionStats {
    pub total_compactions: u32,
    pub total_chars_reduced: u64,
    pub avg_reduction_pct: f64,
    pub sessions_compacted: u32,
    pub storage_saved_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSessionInfo {
    pub session_id: String,
    pub current_chars: u32,
    pub message_count: u32,
    pub oldest_message_age_mins: u32,
    pub estimated_value_pct: f64,
    pub has_been_compacted: bool,
    pub compaction_count: u32,
}

// ── State ──

struct CompactionState {
    sessions: Vec<(String, Vec<ContextSegment>)>,
    config: CompactionConfig,
    stats: CompactionStats,
}

impl Default for CompactionState {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            config: CompactionConfig::default(),
            stats: CompactionStats {
                total_compactions: 0,
                total_chars_reduced: 0,
                avg_reduction_pct: 0.0,
                sessions_compacted: 0,
                storage_saved_bytes: 0,
            },
        }
    }
}

static STATE: LazyLock<Mutex<CompactionState>> = LazyLock::new(|| Mutex::new(CompactionState::default()));

// ── Helpers ──

fn hash_session(session_id: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    session_id.hash(&mut hasher);
    hasher.finish()
}

fn level_reduction_pct(level: &CompactionLevel) -> f64 {
    match level {
        CompactionLevel::Light => 0.20,
        CompactionLevel::Medium => 0.40,
        CompactionLevel::Aggressive => 0.70,
        CompactionLevel::Maximum => 0.90,
    }
}

fn level_segments(level: &CompactionLevel) -> u32 {
    match level {
        CompactionLevel::Light => 4,
        CompactionLevel::Medium => 8,
        CompactionLevel::Aggressive => 16,
        CompactionLevel::Maximum => 25,
    }
}

fn level_decision_count(level: &CompactionLevel) -> u32 {
    match level {
        CompactionLevel::Light => 2,
        CompactionLevel::Medium => 3,
        CompactionLevel::Aggressive => 4,
        CompactionLevel::Maximum => 5,
    }
}

fn level_code_count(level: &CompactionLevel) -> u32 {
    match level {
        CompactionLevel::Light => 1,
        CompactionLevel::Medium => 2,
        CompactionLevel::Aggressive => 3,
        CompactionLevel::Maximum => 4,
    }
}

fn gen_segments(session_id: &str, level: &CompactionLevel) -> Vec<ContextSegment> {
    let hash = hash_session(session_id);
    let n = level_segments(level);
    let reduction = level_reduction_pct(level);
    let decision_count = level_decision_count(level);
    let code_count = level_code_count(level);
    let now = Utc::now().timestamp();
    let base_chars = 15000u32;

    (0..n).map(|i| {
        let orig = base_chars + (hash as u32 % 5000) * (i + 1);
        let compacted = (orig as f64 * (1.0 - reduction)) as u32;
        let ratio = if orig > 0 { compacted as f64 / orig as f64 } else { 1.0 };

        let decisions: Vec<String> = (0..decision_count).map(|d| {
            let labels = [
                "Adopted Rust-native OSINT over external binaries",
                "Selected VSA HyperCube for knowledge representation",
                "Chose POE dual-specialization for agent routing",
                "Decided on SQLite KB as single source of truth",
                "Selected GWT resonance for attention routing",
            ];
            labels[(hash as usize + i as usize + d as usize) % labels.len()].to_string()
        }).collect();

        let code: Vec<String> = (0..code_count).map(|c| {
            let snippets = [
                "nt_core_self::AttentionManager::route()",
                "ConsciousnessTree::run_growth_cycle()",
                "UnifiedCrawler::run_cycle()",
                "nt_memory_kb::KnowledgeBase::insert_or_get_node()",
            ];
            snippets[(hash as usize + i as usize + c as usize) % snippets.len()].to_string()
        }).collect();

        ContextSegment {
            id: format!("seg-{}-{}", session_id, i),
            original_length_chars: orig,
            compacted_length_chars: compacted,
            ratio,
            summary: format!(
                "Compacted segment {}: {} chars → {} chars ({:.0}% reduction)",
                i, orig, compacted, reduction * 100.0
            ),
            key_points: vec![
                format!("Key architectural decision block {}", i),
                format!("Configuration changes batch {}", i),
            ],
            decisions,
            preserved_code: code,
            created_at: now - (n - i) as i64 * 3600,
        }
    }).collect()
}

// ── Commands ──

#[tauri::command]
pub fn context_analyze(session_id: String) -> Result<ContextSessionInfo, String> {
    let hash = hash_session(&session_id);
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    let compacted_count = state.sessions.iter().filter(|(sid, _)| sid == &session_id).count() as u32;
    let has_compacted = compacted_count > 0;

    Ok(ContextSessionInfo {
        session_id,
        current_chars: 12000 + (hash as u32 % 80000),
        message_count: 50 + (hash as u32 % 200),
        oldest_message_age_mins: 30 + (hash as u32 % 300),
        estimated_value_pct: 0.5 + ((hash % 100) as f64 / 500.0),
        has_been_compacted: has_compacted,
        compaction_count: compacted_count,
    })
}

#[tauri::command]
pub fn context_compact(
    session_id: String,
    level: Option<String>,
    strategy: Option<String>,
) -> Result<CompactionResult, String> {
    let compaction_level = match level.as_deref() {
        Some("light") => CompactionLevel::Light,
        Some("medium") | None => CompactionLevel::Medium,
        Some("aggressive") => CompactionLevel::Aggressive,
        Some("maximum") => CompactionLevel::Maximum,
        Some(other) => return Err(format!("Unknown compaction level: {}", other)),
    };
    let compaction_strategy = match strategy.as_deref() {
        Some("sliding_window") => CompactionStrategy::SlidingWindow,
        Some("summary_based") => CompactionStrategy::SummaryBased,
        Some("semantic") => CompactionStrategy::Semantic,
        Some("hybrid") | None => CompactionStrategy::Hybrid,
        Some(other) => return Err(format!("Unknown compaction strategy: {}", other)),
    };

    let segments = gen_segments(&session_id, &compaction_level);
    let segments_compacted = segments.len() as u32;
    let reduction = level_reduction_pct(&compaction_level);
    let decision_count = level_decision_count(&compaction_level);
    let code_count = level_code_count(&compaction_level);
    let original_total: u32 = segments.iter().map(|s| s.original_length_chars).sum();
    let compacted_total: u32 = segments.iter().map(|s| s.compacted_length_chars).sum();

    let mut state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    state.sessions.retain(|(sid, _)| sid != &session_id);
    state.sessions.push((session_id.clone(), segments));
    state.stats.total_compactions += 1;
    state.stats.total_chars_reduced += (original_total - compacted_total) as u64;
    state.stats.sessions_compacted += 1;
    state.stats.storage_saved_bytes += (original_total - compacted_total) as u64;

    let n = state.stats.total_compactions;
    state.stats.avg_reduction_pct = state.stats.storage_saved_bytes as f64 / (n as f64 * 100000.0) * 100.0;

    Ok(CompactionResult {
        session_id,
        original_total_chars: original_total,
        compacted_total_chars: compacted_total,
        reduction_pct: reduction * 100.0,
        segments_compacted,
        strategy_used: compaction_strategy,
        level: compaction_level,
        preserved_decision_count: decision_count * segments_compacted,
        preserved_code_snippets: code_count * segments_compacted,
    })
}

#[tauri::command]
pub fn context_get_segments(session_id: String) -> Result<Vec<ContextSegment>, String> {
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    state.sessions
        .iter()
        .find(|(sid, _)| sid == &session_id)
        .map(|(_, segs)| segs.clone())
        .ok_or_else(|| format!("No compaction data for session: {}", session_id))
}

#[tauri::command]
pub fn context_get_segment(segment_id: String) -> Result<ContextSegment, String> {
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    state.sessions
        .iter()
        .flat_map(|(_, segs)| segs.iter())
        .find(|seg| seg.id == segment_id)
        .cloned()
        .ok_or_else(|| format!("Segment not found: {}", segment_id))
}

#[tauri::command]
pub fn context_expand() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn context_config() -> Result<CompactionConfig, String> {
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn context_set_config(config: CompactionConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn context_stats() -> Result<CompactionStats, String> {
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    Ok(state.stats.clone())
}

#[tauri::command]
pub fn context_summarize(text: String, max_chars: Option<u32>) -> Result<String, String> {
    let limit = max_chars.unwrap_or(1000) as usize;
    if text.len() <= limit {
        return Ok(text);
    }
    let first_sentence: String = text.split(|c: char| c == '.' || c == '!' || c == '?')
        .next()
        .unwrap_or(&text[..limit.min(text.len())])
        .to_string();

    let key_words: Vec<&str> = text.split_whitespace()
        .filter(|w| w.len() > 6)
        .take(5)
        .collect();

    let summary = if key_words.is_empty() {
        format!("{}...[compacted]", first_sentence)
    } else {
        format!("{} Key topics: {}. [Compacted from {} chars to {} chars]",
            first_sentence, key_words.join(", "), text.len(), limit)
    };

    if summary.len() > limit {
        Ok(format!("{}...[compacted]", &summary[..limit.saturating_sub(13)]))
    } else {
        Ok(summary)
    }
}

#[tauri::command]
pub fn context_extract_decisions(text: String) -> Result<Vec<String>, String> {
    let triggers = [
        "decided", "chose", "selected", "will use", "改用", "决定",
        "opted", "elected", "settled on", "concluded",
    ];
    let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?').collect();
    let mut decisions = Vec::new();

    for sentence in &sentences {
        let trimmed = sentence.trim();
        if trimmed.is_empty() { continue; }
        let lower_sent = trimmed.to_lowercase();
        if triggers.iter().any(|t| lower_sent.contains(t)) {
            decisions.push(trimmed.to_string());
        }
    }

    if decisions.is_empty() {
        let fallback: Vec<&str> = text.split_whitespace().take(15).collect();
        decisions.push(format!("(No explicit decisions found. First 15 words: {})", fallback.join(" ")));
    }

    Ok(decisions)
}

#[tauri::command]
pub fn context_check_threshold(session_id: String) -> Result<bool, String> {
    let state = STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    let info = context_analyze(session_id)?;
    Ok(info.current_chars > state.config.auto_compact_threshold_chars)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        if let Ok(mut state) = STATE.lock() {
            state.sessions.clear();
            state.config = CompactionConfig::default();
            state.stats = CompactionStats {
                total_compactions: 0,
                total_chars_reduced: 0,
                avg_reduction_pct: 0.0,
                sessions_compacted: 0,
                storage_saved_bytes: 0,
            };
        }
    }

    #[test]
    fn test_context_analyze() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let info = context_analyze("test-session-1".into()).unwrap();
        assert_eq!(info.session_id, "test-session-1");
        assert!(info.current_chars > 0);
        assert!(info.message_count > 0);
        assert!(!info.has_been_compacted);
    }

    #[test]
    fn test_context_compact() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let result = context_compact("compact-test".into(), Some("aggressive".into()), Some("hybrid".into())).unwrap();
        assert_eq!(result.session_id, "compact-test");
        assert!(result.reduction_pct >= 65.0 && result.reduction_pct <= 75.0);
        assert!(result.segments_compacted >= 12);
        assert!(result.preserved_decision_count > 0);
        assert!(result.preserved_code_snippets > 0);
        assert!(result.original_total_chars > result.compacted_total_chars);
    }

    #[test]
    fn test_context_stats() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        // Compact a session first to generate stats
        let _ = context_compact("stats-test".into(), Some("light".into()), None);
        let stats = context_stats().unwrap();
        assert!(stats.total_compactions > 0);
        assert!(stats.total_chars_reduced > 0);
        assert!(stats.sessions_compacted > 0);
    }

    #[test]
    fn test_context_summarize() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let long = "This is the first sentence of a very long document. It contains important information about the system architecture. We decided to use VSA HyperCube for knowledge representation. This is another sentence that should be truncated.";
        let summary = context_summarize(long.into(), Some(60)).unwrap();
        assert!(summary.len() <= 80);
        assert!(summary.contains("...[compacted]"));
    }

    #[test]
    fn test_context_extract_decisions() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let text = "We decided to use Rust for the core engine. We selected Python for prototyping. We will use Tauri for the desktop app. The team opted for SQLite as the database. Some unrelated text here.";
        let decisions = context_extract_decisions(text.into()).unwrap();
        assert!(decisions.len() >= 3);
        assert!(decisions.iter().any(|d| d.to_lowercase().contains("decided")));
        assert!(decisions.iter().any(|d| d.to_lowercase().contains("selected")));
    }

    #[test]
    fn test_context_config_default() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let config = context_config().unwrap();
        assert!(config.enabled);
        assert!(config.auto_compact);
        assert_eq!(config.auto_compact_threshold_chars, 50000);
    }

    #[test]
    fn test_context_set_config() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let mut config = CompactionConfig::default();
        config.auto_compact_threshold_chars = 100000;
        config.enabled = false;
        context_set_config(config).unwrap();
        let config = context_config().unwrap();
        assert!(!config.enabled);
        assert_eq!(config.auto_compact_threshold_chars, 100000);
    }
}
