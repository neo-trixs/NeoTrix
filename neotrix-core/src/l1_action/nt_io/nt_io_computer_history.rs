//! Computer History — Activity Tracking Across Apps/Sites
//!
//! Implements the ChatGPT pattern: track user activity across apps and websites
//! to build memories and context for agents.
//!
//! # Safety
//! - Activity tracking requires explicit user consent
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Activity type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivityType {
    /// App opened/focused
    AppFocus { app_name: String },
    /// App closed
    AppClose { app_name: String },
    /// Website visited
    WebsiteVisit { url: String, title: Option<String> },
    /// File opened
    FileOpen { path: String, app: Option<String> },
    /// File edited
    FileEdit { path: String, app: Option<String> },
    /// Terminal command executed
    TerminalCommand { command: String, working_dir: Option<String> },
    /// Screenshot taken
    Screenshot { path: String },
    /// Custom activity
    Custom { activity_type: String, data: HashMap<String, String> },
}

/// Recorded activity
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activity {
    /// Activity ID
    pub id: String,
    /// Activity type
    pub activity_type: ActivityType,
    /// Timestamp
    pub timestamp: String,
    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
    /// Application context
    pub app_context: Option<String>,
    /// Whether this activity was auto-captured
    pub auto_captured: bool,
}

/// Memory generated from activity
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivityMemory {
    /// Memory ID
    pub id: String,
    /// Summary
    pub summary: String,
    /// Source activities
    pub source_activities: Vec<String>,
    /// Generated at
    pub generated_at: String,
    /// Memory category
    pub category: MemoryCategory,
    /// Confidence score
    pub confidence: f64,
}

/// Memory category
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemoryCategory {
    /// User preference
    Preference,
    /// Workflow pattern
    Workflow,
    /// Project context
    Project,
    /// Tool usage
    ToolUsage,
    /// Schedule/routine
    Schedule,
    /// Custom
    Custom(String),
}

/// History configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryConfig {
    /// Whether tracking is enabled
    pub enabled: bool,
    /// Maximum history duration in days
    pub max_duration_days: u32,
    /// Maximum activities to store
    pub max_activities: usize,
    /// Apps to ignore
    pub ignored_apps: Vec<String>,
    /// Websites to ignore
    pub ignored_websites: Vec<String>,
    /// Whether to auto-generate memories
    pub auto_generate_memories: bool,
    /// Memory generation interval in minutes
    pub memory_interval_minutes: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default — requires explicit consent
            max_duration_days: 30,
            max_activities: 10000,
            ignored_apps: vec![
                "Screen Time".to_string(),
                "System Preferences".to_string(),
            ],
            ignored_websites: vec![
                "chrome://newtab".to_string(),
                "about:blank".to_string(),
            ],
            auto_generate_memories: true,
            memory_interval_minutes: 60,
        }
    }
}

/// Statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryStats {
    pub total_activities: usize,
    pub total_memories: usize,
    pub tracking_enabled: bool,
    pub unique_apps: usize,
    pub unique_websites: usize,
    pub oldest_activity: Option<String>,
    pub newest_activity: Option<String>,
}

// ============================================================================
// Computer History Engine
// ============================================================================

/// Activity tracking and memory generation engine
pub struct ComputerHistory {
    /// Recorded activities
    activities: Arc<RwLock<Vec<Activity>>>,
    /// Generated memories
    memories: Arc<RwLock<Vec<ActivityMemory>>>,
    /// Configuration
    config: HistoryConfig,
}

impl ComputerHistory {
    /// Create a new history engine
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            activities: Arc::new(RwLock::new(Vec::new())),
            memories: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Create with default config (disabled)
    pub fn with_defaults() -> Self {
        Self::new(HistoryConfig::default())
    }

    /// Enable tracking
    pub async fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable tracking
    pub async fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Record an activity
    pub async fn record(&self, activity: Activity) -> Result<(), HistoryError> {
        if !self.config.enabled {
            return Err(HistoryError::TrackingDisabled);
        }

        // Check if activity should be ignored
        if self.should_ignore(&activity) {
            return Ok(());
        }

        let mut activities = self.activities.write().await;
        activities.push(activity);

        // Trim old activities
        if activities.len() > self.config.max_activities {
            let excess = activities.len() - self.config.max_activities;
            activities.drain(0..excess);
        }

        Ok(())
    }

    /// Check if activity should be ignored
    fn should_ignore(&self, activity: &Activity) -> bool {
        match &activity.activity_type {
            ActivityType::AppFocus { app_name } => {
                self.config.ignored_apps.iter().any(|a| a == app_name)
            }
            ActivityType::WebsiteVisit { url, .. } => {
                self.config.ignored_websites.iter().any(|w| url.starts_with(w))
            }
            _ => false,
        }
    }

    /// Generate memories from activities
    pub async fn generate_memories(&self) -> Result<usize, HistoryError> {
        if !self.config.auto_generate_memories {
            return Ok(0);
        }

        let activities = self.activities.read().await;
        let mut new_memories = Vec::new();

        // Simple pattern detection — group by app
        let mut app_groups: HashMap<String, Vec<&Activity>> = HashMap::new();
        for activity in activities.iter() {
            if let ActivityType::AppFocus { ref app_name } = activity.activity_type {
                app_groups.entry(app_name.clone()).or_default().push(activity);
            }
        }

        // Generate memory for frequently used apps
        for (app_name, app_activities) in &app_groups {
            if app_activities.len() >= 5 {
                let memory = ActivityMemory {
                    id: format!("mem-{}", uuid::Uuid::new_v4()),
                    summary: format!("Frequently used app: {} ({} sessions)", app_name, app_activities.len()),
                    source_activities: app_activities.iter().map(|a| a.id.clone()).collect(),
                    generated_at: chrono::Utc::now().to_rfc3339(),
                    category: MemoryCategory::ToolUsage,
                    confidence: 0.8,
                };
                new_memories.push(memory);
            }
        }

        let count = new_memories.len();
        let mut memories = self.memories.write().await;
        memories.extend(new_memories);

        Ok(count)
    }

    /// Get recent activities
    pub async fn recent(&self, limit: usize) -> Vec<Activity> {
        let activities = self.activities.read().await;
        activities.iter().rev().take(limit).cloned().collect()
    }

    /// Get memories
    pub async fn memories(&self, category: Option<MemoryCategory>) -> Vec<ActivityMemory> {
        let memories = self.memories.read().await;
        match category {
            Some(cat) => memories.iter().filter(|m| m.category == cat).cloned().collect(),
            None => memories.clone(),
        }
    }

    /// Search activities
    pub async fn search(&self, query: &str) -> Vec<Activity> {
        let activities = self.activities.read().await;
        let query_lower = query.to_lowercase();

        activities.iter()
            .filter(|a| {
                match &a.activity_type {
                    ActivityType::AppFocus { app_name } => app_name.to_lowercase().contains(&query_lower),
                    ActivityType::WebsiteVisit { url, title } => {
                        url.to_lowercase().contains(&query_lower) ||
                        title.as_ref().map(|t| t.to_lowercase().contains(&query_lower)).unwrap_or(false)
                    }
                    ActivityType::FileOpen { path, .. } => path.to_lowercase().contains(&query_lower),
                    ActivityType::TerminalCommand { command, .. } => command.to_lowercase().contains(&query_lower),
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> HistoryStats {
        let activities = self.activities.read().await;
        let memories = self.memories.read().await;

        let unique_apps: std::collections::HashSet<String> = activities.iter()
            .filter_map(|a| match &a.activity_type {
                ActivityType::AppFocus { app_name } => Some(app_name.clone()),
                _ => None,
            })
            .collect();

        let unique_websites: std::collections::HashSet<String> = activities.iter()
            .filter_map(|a| match &a.activity_type {
                ActivityType::WebsiteVisit { url, .. } => Some(url.clone()),
                _ => None,
            })
            .collect();

        HistoryStats {
            total_activities: activities.len(),
            total_memories: memories.len(),
            tracking_enabled: self.config.enabled,
            unique_apps: unique_apps.len(),
            unique_websites: unique_websites.len(),
            oldest_activity: activities.first().map(|a| a.timestamp.clone()),
            newest_activity: activities.last().map(|a| a.timestamp.clone()),
        }
    }

    /// Clear all history
    pub async fn clear(&self) {
        let mut activities = self.activities.write().await;
        activities.clear();
        let mut memories = self.memories.write().await;
        memories.clear();
    }
}

// ============================================================================
// Errors
// ============================================================================

/// History errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum HistoryError {
    #[error("tracking is disabled")]
    TrackingDisabled,

    #[error("activity recording failed: {0}")]
    RecordingFailed(String),

    #[error("memory generation failed: {0}")]
    MemoryGenerationFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_config_default() {
        let config = HistoryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_duration_days, 30);
    }

    #[tokio::test]
    async fn test_recording_disabled() {
        let history = ComputerHistory::with_defaults();
        let activity = Activity {
            id: "test".to_string(),
            activity_type: ActivityType::AppFocus { app_name: "Test".to_string() },
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            app_context: None,
            auto_captured: true,
        };

        let result = history.record(activity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_recording_enabled() {
        let mut history = ComputerHistory::with_defaults();
        history.enable().await;

        let activity = Activity {
            id: "test".to_string(),
            activity_type: ActivityType::AppFocus { app_name: "Test".to_string() },
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            app_context: None,
            auto_captured: true,
        };

        let result = history.record(activity).await;
        assert!(result.is_ok());
    }
}
