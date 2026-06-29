use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_HISTORY_ENTRIES: usize = 1000;

// ── Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiRecord {
    pub date: String,
    pub repos_analyzed: u32,
    pub patterns_extracted: u32,
    pub capabilities_proposed: u32,
    pub articles_published: u32,
    pub geo_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    pub id: String,
    pub repo_url: String,
    pub repo_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub patterns_found: Vec<String>,
    pub capabilities_proposed: Vec<String>,
    pub content_angles: Vec<String>,
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRecord {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub task_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub version: String,
    pub last_updated: DateTime<Utc>,

    pub analyses: Vec<AnalysisRecord>,
    pub publications: Vec<PublishRecord>,
    pub tasks: Vec<TaskRecord>,
    pub kpi_history: Vec<KpiRecord>,

    pub total_analyses: u32,
    pub total_patterns: u32,
    pub total_capabilities: u32,
    pub total_publications: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            last_updated: Utc::now(),
            analyses: Vec::new(),
            publications: Vec::new(),
            tasks: Vec::new(),
            kpi_history: Vec::new(),
            total_analyses: 0,
            total_patterns: 0,
            total_capabilities: 0,
            total_publications: 0,
        }
    }
}

// ── Store ──────────────────────────────────────────────

pub struct Store {
    path: PathBuf,
    state: Arc<RwLock<AppState>>,
}

impl Store {
    pub fn new(data_dir: &PathBuf) -> Self {
        fs::create_dir_all(data_dir).expect("failed to create data directory");

        let state_path = data_dir.join("state.json");
        let state = if state_path.exists() {
            let content = fs::read_to_string(&state_path)
                .expect("failed to read state file");
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppState::default()
        };

        Self {
            path: state_path,
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
        self.state.read().await
    }

    #[allow(dead_code)]
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, AppState> {
        self.state.write().await
    }

    pub async fn persist(&self) {
        let state = self.state.read().await;
        let content = serde_json::to_string_pretty(&*state)
            .expect("serialize state");
        if let Err(e) = fs::write(&self.path, &content) {
            tracing::error!("failed to persist state: {e}");
        }
    }

    pub async fn record_analysis(
        &self,
        repo_url: &str,
        repo_name: &str,
        patterns: Vec<String>,
        capabilities: Vec<String>,
        angles: Vec<String>,
        report_path: Option<String>,
    ) {
        let mut state = self.state.write().await;
        let record = AnalysisRecord {
            id: uuid::Uuid::new_v4().to_string(),
            repo_url: repo_url.to_string(),
            repo_name: repo_name.to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            patterns_found: patterns.clone(),
            capabilities_proposed: capabilities.clone(),
            content_angles: angles,
            report_path,
        };
        state.total_analyses += 1;
        state.total_patterns += patterns.len() as u32;
        state.total_capabilities += capabilities.len() as u32;
        state.analyses.push(record);
        if state.analyses.len() > MAX_HISTORY_ENTRIES {
            let excess = state.analyses.len() - MAX_HISTORY_ENTRIES * 4 / 5;
            state.analyses.drain(0..excess);
        }
        state.last_updated = Utc::now();
        drop(state);
        self.persist().await;
    }

    pub async fn record_publication(
        &self,
        platform: &str,
        title: &str,
        url: Option<String>,
        status: &str,
    ) {
        let mut state = self.state.write().await;
        state.publications.push(PublishRecord {
            id: uuid::Uuid::new_v4().to_string(),
            platform: platform.to_string(),
            title: title.to_string(),
            url,
            published_at: Utc::now(),
            status: status.to_string(),
        });
        if state.publications.len() > MAX_HISTORY_ENTRIES {
            let excess = state.publications.len() - MAX_HISTORY_ENTRIES * 4 / 5;
            state.publications.drain(0..excess);
        }
        state.total_publications += 1;
        state.last_updated = Utc::now();
        drop(state);
        self.persist().await;
    }
}
