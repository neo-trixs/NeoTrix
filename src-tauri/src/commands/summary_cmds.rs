use std::collections::VecDeque;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::command;
use chrono::Utc;

const MAX_PLANS: usize = 50;
const MAX_ARTIFACTS: usize = 200;
const MAX_SOURCES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySession {
    pub id: String,
    pub start_time: i64,
    pub status: String,
    pub task_count: usize,
    pub plan_count: usize,
    pub artifact_count: usize,
    pub source_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPlan {
    pub id: String,
    pub title: String,
    pub status: String,
    pub progress_pct: f64,
    pub steps: Vec<SessionPlanStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPlanStep {
    pub id: String,
    pub description: String,
    pub status: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSource {
    pub id: String,
    pub title: String,
    pub url: String,
    pub relevance: f64,
    pub accessed_at: i64,
}

struct SummaryState {
    session: SummarySession,
    plans: VecDeque<SessionPlan>,
    artifacts: VecDeque<SessionArtifact>,
    sources: VecDeque<SessionSource>,
    step_counter: u64,
}

static SUMMARY: std::sync::LazyLock<Mutex<SummaryState>> = std::sync::LazyLock::new(|| {
    Mutex::new(SummaryState {
        session: SummarySession {
            id: String::new(),
            start_time: 0,
            status: "inactive".into(),
            task_count: 0,
            plan_count: 0,
            artifact_count: 0,
            source_count: 0,
        },
        plans: VecDeque::with_capacity(MAX_PLANS),
        artifacts: VecDeque::with_capacity(MAX_ARTIFACTS),
        sources: VecDeque::with_capacity(MAX_SOURCES),
        step_counter: 0,
    })
});

fn push_bounded<T>(deque: &mut VecDeque<T>, item: T, max: usize) {
    if deque.len() >= max {
        deque.pop_front();
    }
    deque.push_back(item);
}

#[command]
pub fn summary_active() -> Result<SummarySession, String> {
    let lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    Ok(lock.session.clone())
}

#[command]
pub fn summary_start() -> Result<String, String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().timestamp();
    let id = format!("ss-{}", now);
    lock.session = SummarySession {
        id,
        start_time: now,
        status: "active".into(),
        task_count: 0,
        plan_count: 0,
        artifact_count: 0,
        source_count: 0,
    };
    lock.plans.clear();
    lock.artifacts.clear();
    lock.sources.clear();
    lock.step_counter = 0;
    Ok(lock.session.id.clone())
}

#[command]
pub fn summary_pause() -> Result<(), String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    lock.session.status = "paused".into();
    Ok(())
}

#[command]
pub fn summary_resume() -> Result<(), String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    lock.session.status = "active".into();
    Ok(())
}

#[command]
pub fn summary_plans() -> Vec<SessionPlan> {
    SUMMARY.lock().map(|l| l.plans.iter().cloned().collect()).unwrap_or_default()
}

#[command]
pub fn summary_artifacts() -> Vec<SessionArtifact> {
    SUMMARY.lock().map(|l| l.artifacts.iter().cloned().collect()).unwrap_or_default()
}

#[command]
pub fn summary_sources() -> Vec<SessionSource> {
    SUMMARY.lock().map(|l| l.sources.iter().cloned().collect()).unwrap_or_default()
}

#[command]
pub fn summary_add_artifact(name: String, kind: String, path: String) -> Result<(), String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    let id = format!("art-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let size = std::path::Path::new(&path).is_file().then(|| {
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    }).unwrap_or(0);
    lock.session.artifact_count += 1;
    push_bounded(&mut lock.artifacts, SessionArtifact { id, name, kind, path, size }, MAX_ARTIFACTS);
    Ok(())
}

#[command]
pub fn summary_add_source(title: String, url: String, relevance: f64) -> Result<(), String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    let clamped_rel = relevance.max(0.0).min(1.0);
    let id = format!("src-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    lock.session.source_count += 1;
    push_bounded(&mut lock.sources, SessionSource {
        id, title, url, relevance: clamped_rel, accessed_at: Utc::now().timestamp(),
    }, MAX_SOURCES);
    Ok(())
}

#[command]
pub fn summary_add_plan(title: String, steps: Vec<String>) -> Result<String, String> {
    let mut lock = SUMMARY.lock().map_err(|e| e.to_string())?;
    if lock.session.id.is_empty() {
        return Err("No active session".into());
    }
    let plan_id = format!("plan-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let plan_steps: Vec<SessionPlanStep> = steps.into_iter().map(|desc| {
        lock.step_counter += 1;
        SessionPlanStep {
            id: format!("{}-step-{}", plan_id, lock.step_counter),
            description: desc,
            status: "pending".into(),
            duration_secs: 0,
        }
    }).collect();
    let progress = if plan_steps.is_empty() { 100.0 } else { 0.0 };
    lock.session.plan_count += 1;
    let plan = SessionPlan {
        id: plan_id.clone(),
        title,
        status: "in_progress".into(),
        progress_pct: progress,
        steps: plan_steps,
    };
    push_bounded(&mut lock.plans, plan, MAX_PLANS);
    Ok(plan_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_full_lifecycle() {
        let mut lock = SUMMARY.lock().unwrap();
        lock.session = SummarySession {
            id: String::new(), start_time: 0, status: "inactive".into(),
            task_count: 0, plan_count: 0, artifact_count: 0, source_count: 0,
        };
        lock.plans.clear();
        lock.artifacts.clear();
        lock.sources.clear();
        lock.step_counter = 0;
        drop(lock);

        let id = summary_start().unwrap();
        assert!(id.starts_with("ss-"));

        let steps = vec!["Research".into(), "Implement".into(), "Test".into()];
        let plan_id = summary_add_plan("Feature X".into(), steps).unwrap();
        assert!(plan_id.starts_with("plan-"));

        assert!(summary_add_artifact("diagram.png".into(), "image".into(), "/tmp/test.png".into()).is_ok());

        assert!(summary_add_source("Paper".into(), "https://example.com".into(), 0.85).is_ok());

        assert!(summary_pause().is_ok());
        let s = summary_active().unwrap();
        assert_eq!(s.status, "paused");

        assert!(summary_resume().is_ok());
        let s = summary_active().unwrap();
        assert_eq!(s.status, "active");

        let plans = summary_plans();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].steps.len(), 3);

        let arts = summary_artifacts();
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].name, "diagram.png");

        let srcs = summary_sources();
        assert_eq!(srcs.len(), 1);
        assert_eq!(srcs[0].title, "Paper");
        assert!((srcs[0].relevance - 0.85).abs() < 1e-9);
    }
}
