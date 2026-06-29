use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

const MAX_COMPLETED: usize = 1000;

/// Always-on task state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum AlwaysOnState {
    Idle,
    Scanning,
    Working,
    Reporting,
    Sleeping,
}

impl std::fmt::Display for AlwaysOnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Scanning => write!(f, "scanning"),
            Self::Working => write!(f, "working"),
            Self::Reporting => write!(f, "reporting"),
            Self::Sleeping => write!(f, "sleeping"),
        }
    }
}

/// Cron-like schedule expression: "every <N> <unit>" or "daily at <HH:MM>" or "hourly"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleExpr {
    Every { interval_secs: u64 },
    Daily { hour: u8, minute: u8 },
    Hourly,
    Weekly { weekday: u8, hour: u8, minute: u8 },
}

impl ScheduleExpr {
    pub fn parse(expr: &str) -> Result<Self, String> {
        let expr = expr.trim();
        if expr == "hourly" {
            return Ok(Self::Hourly);
        }
        if let Some(rest) = expr.strip_prefix("every ") {
            let rest = rest.trim();
            if let Ok(secs) = rest.parse::<u64>() {
                return Ok(Self::Every {
                    interval_secs: secs,
                });
            }
            if let Some(num) = rest.strip_suffix('m') {
                if let Ok(m) = num.parse::<u64>() {
                    return Ok(Self::Every {
                        interval_secs: m * 60,
                    });
                }
            }
            if let Some(num) = rest.strip_suffix('h') {
                if let Ok(h) = num.parse::<u64>() {
                    return Ok(Self::Every {
                        interval_secs: h * 3600,
                    });
                }
            }
            if let Some(num) = rest.strip_suffix('s') {
                if let Ok(s) = num.parse::<u64>() {
                    return Ok(Self::Every { interval_secs: s });
                }
            }
            return Err(format!("Invalid 'every' expression: '{expr}'. Use e.g. 'every 300', 'every 10m', 'every 1h'"));
        }
        if let Some(rest) = expr.strip_prefix("daily at ") {
            let rest = rest.trim();
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() == 2 {
                let hour = parts[0]
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
                let minute = parts[1]
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid minute: {}", parts[1]))?;
                if hour > 23 || minute > 59 {
                    return Err(format!("Invalid time: {hour}:{minute}"));
                }
                return Ok(Self::Daily { hour, minute });
            }
            return Err(format!(
                "Invalid daily expression: '{expr}'. Use 'daily at HH:MM'"
            ));
        }
        if let Some(rest) = expr.strip_prefix("weekly on ") {
            if let Some((wd, tm)) = rest.trim().split_once(" at ") {
                let weekday = wd
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid weekday: {wd}"))?;
                if weekday > 6 {
                    return Err(format!("Invalid weekday: {weekday}. Use 0=Mon to 6=Sun"));
                }
                let tp: Vec<&str> = tm.trim().split(':').collect();
                if tp.len() == 2 {
                    let hour = tp[0]
                        .parse::<u8>()
                        .map_err(|_| format!("Invalid hour: {}", tp[0]))?;
                    let minute = tp[1]
                        .parse::<u8>()
                        .map_err(|_| format!("Invalid minute: {}", tp[1]))?;
                    if hour > 23 || minute > 59 {
                        return Err(format!("Invalid time: {hour}:{minute}"));
                    }
                    return Ok(Self::Weekly {
                        weekday,
                        hour,
                        minute,
                    });
                }
            }
            return Err(format!(
                "Invalid weekly expression: '{expr}'. Use 'weekly on <0-6> at HH:MM'"
            ));
        }
        Err(format!("Invalid schedule expression: '{expr}'. Use 'every <N><unit>', 'daily at HH:MM', 'hourly', or 'weekly on <0-6> at HH:MM'"))
    }

    pub fn next_run(&self, from: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
        match *self {
            Self::Every { interval_secs } => from + chrono::Duration::seconds(interval_secs as i64),
            Self::Hourly => from + chrono::Duration::hours(1),
            Self::Daily { hour, minute } => {
                let naive = from.naive_utc();
                let today = naive.date();
                let target = match today.and_hms_opt(hour as u32, minute as u32, 0) {
                    Some(dt) => {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                    }
                    None => {
                        log::error!("always_on: invalid time {}:{} in Daily", hour, minute);
                        return from + chrono::Duration::hours(1);
                    }
                };
                if target > from {
                    target
                } else {
                    let tomorrow = today + chrono::Duration::days(1);
                    match tomorrow.and_hms_opt(hour as u32, minute as u32, 0) {
                        Some(dt) => chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            dt,
                            chrono::Utc,
                        ),
                        None => {
                            log::error!(
                                "always_on: invalid time {}:{} in Daily(tomorrow)",
                                hour,
                                minute
                            );
                            return from + chrono::Duration::hours(1);
                        }
                    }
                }
            }
            Self::Weekly {
                weekday,
                hour,
                minute,
            } => {
                let target_wd = match weekday {
                    0 => chrono::Weekday::Mon,
                    1 => chrono::Weekday::Tue,
                    2 => chrono::Weekday::Wed,
                    3 => chrono::Weekday::Thu,
                    4 => chrono::Weekday::Fri,
                    5 => chrono::Weekday::Sat,
                    _ => chrono::Weekday::Sun,
                };
                let mut days_ahead = 0i64;
                loop {
                    let candidate = from + chrono::Duration::days(days_ahead);
                    if candidate.weekday() == target_wd {
                        let date = candidate.naive_utc().date();
                        let target = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            match date.and_hms_opt(hour as u32, minute as u32, 0) {
                                Some(dt) => dt,
                                None => {
                                    log::error!(
                                        "always_on: invalid time {}:{} in Weekly",
                                        hour,
                                        minute
                                    );
                                    return from + chrono::Duration::hours(1);
                                }
                            },
                            chrono::Utc,
                        );
                        if target > from {
                            return target;
                        }
                    }
                    days_ahead += 1;
                    if days_ahead > 7 {
                        return from + chrono::Duration::days(7);
                    }
                }
            }
        }
    }
}

/// A persistent always-on task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlwaysOnTask {
    pub id: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub interval_secs: u64,
    pub max_runs: u64,
    pub run_count: u64,
    pub state: AlwaysOnState,
    pub output_files: Vec<String>,
    pub last_output: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub schedule: Option<ScheduleExpr>,
    #[serde(default)]
    pub cron_description: Option<String>,
    #[serde(default)]
    pub paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedTask {
    pub description: String,
    pub priority: u8,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkReport {
    pub task_id: String,
    pub description: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub files_produced: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    pub scan_count: usize,
    pub tasks_executed: usize,
    pub tasks_completed: usize,
    pub duration_ms: u64,
    pub reports: Vec<WorkReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub enabled: bool,
    pub state: String,
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub uptime_secs: u64,
    pub last_cycle: Option<String>,
}

/// Always-on engine for persistent background task execution
pub struct AlwaysOnEngine {
    pub enabled: bool,
    pub state: AlwaysOnState,
    pub tasks: Vec<AlwaysOnTask>,
    pub scan_interval_secs: u64,
    pub idle_cooldown_secs: u64,
    pub max_concurrent: u32,
    pub auto_queue: bool,
    pub completed_task_ids: Vec<String>,
    started_at: Option<Instant>,
    last_cycle: Option<chrono::DateTime<chrono::Utc>>,
}

impl AlwaysOnEngine {
    pub fn new() -> Self {
        Self {
            enabled: false,
            state: AlwaysOnState::Idle,
            tasks: Vec::new(),
            scan_interval_secs: 60,
            idle_cooldown_secs: 300,
            max_concurrent: 1,
            auto_queue: true,
            completed_task_ids: Vec::new(),
            started_at: None,
            last_cycle: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.enabled {
            return Err("Already running".into());
        }
        self.enabled = true;
        self.state = AlwaysOnState::Idle;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Err("Not running".into());
        }
        self.enabled = false;
        self.state = AlwaysOnState::Idle;
        Ok(())
    }

    pub fn scan_cycle(&mut self) -> Vec<ScannedTask> {
        self.state = AlwaysOnState::Scanning;
        let mut discovered = Vec::new();

        // Auto-discover tasks from critical directories
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let neotrix_dir = home.join(".neotrix");

        // Check for monitoring files
        let monitor_path = neotrix_dir.join("always_on_tasks.txt");
        if monitor_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&monitor_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        if let Some(desc) = line.strip_prefix("monitor:") {
                            discovered.push(ScannedTask {
                                description: desc.trim().to_string(),
                                priority: 3,
                                source: "monitor_file".into(),
                            });
                        } else if let Some(desc) = line.strip_prefix("recurring:") {
                            discovered.push(ScannedTask {
                                description: desc.trim().to_string(),
                                priority: 2,
                                source: "recurring_file".into(),
                            });
                        }
                    }
                }
            }
        }

        // Check for incomplete work
        let goals_path = neotrix_dir.join("goals.json");
        if goals_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&goals_path) {
                if let Ok(goals) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(items) = goals.get("items").and_then(|v| v.as_array()) {
                        for item in items {
                            let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
                            if status == "pending" || status == "in_progress" {
                                if let Some(desc) = item.get("description").and_then(|v| v.as_str())
                                {
                                    discovered.push(ScannedTask {
                                        description: desc.to_string(),
                                        priority: 1,
                                        source: "goal_queue".into(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        self.state = AlwaysOnState::Idle;
        discovered
    }

    pub fn add_recurring(&mut self, description: &str, interval_secs: u64) -> String {
        let id = format!("ao_{}", self.tasks.len() + 1);
        let task = AlwaysOnTask {
            id: id.clone(),
            description: description.to_string(),
            created_at: chrono::Utc::now(),
            last_run: None,
            interval_secs,
            max_runs: 1000,
            run_count: 0,
            state: AlwaysOnState::Idle,
            output_files: Vec::new(),
            last_output: None,
            tags: vec!["recurring".into()],
            schedule: None,
            cron_description: None,
            paused: false,
        };
        self.tasks.push(task);
        id
    }

    pub fn add_oneshot(&mut self, description: &str) -> String {
        let id = format!("ao_{}", self.tasks.len() + 1);
        let task = AlwaysOnTask {
            id: id.clone(),
            description: description.to_string(),
            created_at: chrono::Utc::now(),
            last_run: None,
            interval_secs: 0,
            max_runs: 1,
            run_count: 0,
            state: AlwaysOnState::Idle,
            output_files: Vec::new(),
            last_output: None,
            tags: vec!["oneshot".into()],
            schedule: None,
            cron_description: None,
            paused: false,
        };
        self.tasks.push(task);
        id
    }

    pub fn add_scheduled(&mut self, description: &str, schedule: ScheduleExpr) -> String {
        let cron_desc = match &schedule {
            ScheduleExpr::Every { interval_secs } => format!("every {interval_secs}s"),
            ScheduleExpr::Daily { hour, minute } => format!("daily at {hour:02}:{minute:02}"),
            ScheduleExpr::Hourly => "hourly".into(),
            ScheduleExpr::Weekly {
                weekday,
                hour,
                minute,
            } => {
                format!("weekly on {weekday} at {hour:02}:{minute:02}")
            }
        };
        let id = format!("sched_{}", self.tasks.len() + 1);
        let task = AlwaysOnTask {
            id: id.clone(),
            description: description.to_string(),
            created_at: chrono::Utc::now(),
            last_run: None,
            interval_secs: match &schedule {
                ScheduleExpr::Every { interval_secs } => *interval_secs,
                _ => 0,
            },
            max_runs: u64::MAX,
            run_count: 0,
            state: AlwaysOnState::Idle,
            output_files: Vec::new(),
            last_output: None,
            tags: vec!["scheduled".into()],
            schedule: Some(schedule),
            cron_description: Some(cron_desc),
            paused: false,
        };
        self.tasks.push(task);
        id
    }

    pub fn list_scheduled(&self) -> Vec<&AlwaysOnTask> {
        self.tasks.iter().filter(|t| t.schedule.is_some()).collect()
    }

    pub fn pause_scheduled(&mut self, id: &str) -> Result<(), String> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Task not found: {id}"))?;
        if task.schedule.is_none() {
            return Err(format!("Task {id} is not a scheduled task"));
        }
        task.paused = true;
        Ok(())
    }

    pub fn resume_scheduled(&mut self, id: &str) -> Result<(), String> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Task not found: {id}"))?;
        if task.schedule.is_none() {
            return Err(format!("Task {id} is not a scheduled task"));
        }
        task.paused = false;
        Ok(())
    }

    pub fn list_tasks(&self, filter: Option<&str>) -> Vec<&AlwaysOnTask> {
        match filter {
            Some("recurring") => self
                .tasks
                .iter()
                .filter(|t| t.tags.contains(&"recurring".into()))
                .collect(),
            Some("oneshot") => self
                .tasks
                .iter()
                .filter(|t| t.tags.contains(&"oneshot".into()))
                .collect(),
            Some("active") => self
                .tasks
                .iter()
                .filter(|t| t.run_count < t.max_runs)
                .collect(),
            Some("completed") => {
                let completed_ids: std::collections::HashSet<String> =
                    self.completed_task_ids.iter().cloned().collect();
                self.tasks
                    .iter()
                    .filter(|t| completed_ids.contains(&t.id))
                    .collect()
            }
            _ => self.tasks.iter().collect(),
        }
    }

    pub fn remove_task(&mut self, id: &str) -> Result<(), String> {
        let idx = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("Task not found: {}", id))?;
        self.tasks.remove(idx);
        Ok(())
    }

    pub fn status(&self) -> EngineStatus {
        EngineStatus {
            enabled: self.enabled,
            state: self.state.to_string(),
            total_tasks: self.tasks.len(),
            active_tasks: self
                .tasks
                .iter()
                .filter(|t| t.run_count < t.max_runs)
                .count(),
            completed_tasks: self.completed_task_ids.len(),
            uptime_secs: self.started_at.map(|s| s.elapsed().as_secs()).unwrap_or(0),
            last_cycle: self.last_cycle.map(|c| c.to_rfc3339()),
        }
    }

    pub fn full_cycle(&mut self) -> Result<CycleReport, String> {
        let start = Instant::now();
        self.state = AlwaysOnState::Scanning;

        // 1. Discover candidate tasks
        let discovered = self.scan_cycle();
        let scan_count = discovered.len();

        // Auto-add discovered tasks
        if self.auto_queue {
            for task in &discovered {
                if !self.tasks.iter().any(|t| t.description == task.description) {
                    self.add_oneshot(&task.description);
                }
            }
        }

        // 2. Execute pending tasks
        self.state = AlwaysOnState::Working;
        let mut reports = Vec::new();
        let mut executed = 0;

        let pending: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.run_count < t.max_runs)
            .filter(|(_, t)| {
                if t.paused {
                    return false;
                }
                if let Some(ref sched) = t.schedule {
                    match t.last_run {
                        Some(last) => chrono::Utc::now() >= sched.next_run(last),
                        None => true,
                    }
                } else {
                    if t.interval_secs == 0 {
                        return true;
                    }
                    match t.last_run {
                        Some(last) => {
                            let elapsed = (chrono::Utc::now() - last).num_seconds() as u64;
                            elapsed >= t.interval_secs
                        }
                        None => true,
                    }
                }
            })
            .map(|(i, _)| i)
            .take(self.max_concurrent as usize)
            .collect();

        for &idx in &pending {
            let task = &mut self.tasks[idx];
            task.state = AlwaysOnState::Working;
            let desc = task.description.clone();
            let task_id = task.id.clone();

            // Simulate work — log and record
            task.last_run = Some(chrono::Utc::now());
            task.run_count += 1;
            let output = format!(
                "[always_on] executed task: {} (run #{})",
                desc, task.run_count
            );
            log::info!("{}", output);

            task.last_output = Some(output.clone());
            task.state = AlwaysOnState::Idle;

            let report = WorkReport {
                task_id,
                description: desc,
                success: true,
                output,
                duration_ms: 10,
                files_produced: Vec::new(),
            };

            reports.push(report);
            executed += 1;

            // Check if completed
            if task.run_count >= task.max_runs {
                self.completed_task_ids.push(task.id.clone());
                if self.completed_task_ids.len() > MAX_COMPLETED {
                    self.completed_task_ids
                        .drain(0..self.completed_task_ids.len() - MAX_COMPLETED);
                }
            }
        }

        // 3. Report
        self.state = AlwaysOnState::Reporting;
        let duration = start.elapsed();
        self.last_cycle = Some(chrono::Utc::now());

        self.state = AlwaysOnState::Sleeping;
        if executed == 0 {
            self.state = AlwaysOnState::Idle;
        }

        Ok(CycleReport {
            scan_count,
            tasks_executed: executed,
            tasks_completed: reports.iter().filter(|r| r.success).count(),
            duration_ms: duration.as_millis() as u64,
            reports,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let neotrix_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".neotrix");
        std::fs::create_dir_all(&neotrix_dir).map_err(|e| e.to_string())?;

        let data = serde_json::json!({
            "enabled": self.enabled,
            "state": self.state.to_string(),
            "tasks": self.tasks,
            "completed_task_ids": self.completed_task_ids,
            "scan_interval_secs": self.scan_interval_secs,
            "idle_cooldown_secs": self.idle_cooldown_secs,
        });
        let path = neotrix_dir.join("always_on.json");
        let tmp = path.with_extension("tmp");
        std::fs::write(
            &tmp,
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load() -> Self {
        let neotrix_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".neotrix");
        let path = neotrix_dir.join("always_on.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let enabled = data
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let state_str = data.get("state").and_then(|v| v.as_str()).unwrap_or("idle");
                    let state = match state_str {
                        "scanning" => AlwaysOnState::Scanning,
                        "working" => AlwaysOnState::Working,
                        "reporting" => AlwaysOnState::Reporting,
                        "sleeping" => AlwaysOnState::Sleeping,
                        _ => AlwaysOnState::Idle,
                    };
                    let scan_interval = data
                        .get("scan_interval_secs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(60);
                    let idle_cooldown = data
                        .get("idle_cooldown_secs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(300);
                    let tasks: Vec<AlwaysOnTask> = data
                        .get("tasks")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let completed: Vec<String> = data
                        .get("completed_task_ids")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let started = if enabled { Some(Instant::now()) } else { None };
                    return Self {
                        enabled,
                        state,
                        tasks,
                        scan_interval_secs: scan_interval,
                        idle_cooldown_secs: idle_cooldown,
                        max_concurrent: 1,
                        auto_queue: true,
                        completed_task_ids: completed,
                        started_at: started,
                        last_cycle: None,
                    };
                }
            }
        }
        Self::new()
    }
}

/// Global always-on engine singleton
pub static ALWAYS_ON_ENGINE: LazyLock<Mutex<AlwaysOnEngine>> =
    LazyLock::new(|| Mutex::new(AlwaysOnEngine::load()));

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // TODO: add #[serial] to any new tests that use global singletons
    #[test]
    fn test_always_on_engine_new() {
        let engine = AlwaysOnEngine::new();
        assert!(!engine.enabled);
        assert_eq!(engine.state, AlwaysOnState::Idle);
        assert!(engine.tasks.is_empty());
    }

    #[test]
    fn test_always_on_engine_start_stop() {
        let mut engine = AlwaysOnEngine::new();
        assert!(engine.start().is_ok());
        assert!(engine.enabled);
        assert!(engine.start().is_err());
        assert!(engine.stop().is_ok());
        assert!(!engine.enabled);
        assert!(engine.stop().is_err());
    }

    #[test]
    fn test_always_on_add_recurring() {
        let mut engine = AlwaysOnEngine::new();
        let id = engine.add_recurring("check system health", 300);
        assert!(id.starts_with("ao_"));
        assert_eq!(engine.tasks.len(), 1);
        assert_eq!(engine.tasks[0].description, "check system health");
        assert_eq!(engine.tasks[0].interval_secs, 300);
        assert!(engine.tasks[0].tags.contains(&"recurring".into()));
    }

    #[test]
    fn test_always_on_add_oneshot() {
        let mut engine = AlwaysOnEngine::new();
        let _id = engine.add_oneshot("clean temp files");
        assert_eq!(engine.tasks.len(), 1);
        assert_eq!(engine.tasks[0].max_runs, 1);
        assert!(engine.tasks[0].tags.contains(&"oneshot".into()));
    }

    #[test]
    fn test_always_on_list_tasks() {
        let mut engine = AlwaysOnEngine::new();
        engine.add_recurring("health", 300);
        engine.add_oneshot("cleanup");
        assert_eq!(engine.list_tasks(None).len(), 2);
        assert_eq!(engine.list_tasks(Some("recurring")).len(), 1);
        assert_eq!(engine.list_tasks(Some("oneshot")).len(), 1);
    }

    #[test]
    fn test_always_on_remove_task() {
        let mut engine = AlwaysOnEngine::new();
        let id = engine.add_oneshot("test");
        assert_eq!(engine.tasks.len(), 1);
        assert!(engine.remove_task(&id).is_ok());
        assert!(engine.tasks.is_empty());
        assert!(engine.remove_task("nonexistent").is_err());
    }

    #[test]
    fn test_always_on_status() {
        let engine = AlwaysOnEngine::new();
        let status = engine.status();
        assert!(!status.enabled);
        assert_eq!(status.total_tasks, 0);
        assert_eq!(status.state, "idle");
    }

    #[test]
    fn test_always_on_full_cycle() {
        let mut engine = AlwaysOnEngine::new();
        engine.max_concurrent = 2;
        engine.add_oneshot("task1");
        engine.add_oneshot("task2");
        let report = engine.full_cycle().unwrap();
        assert_eq!(report.tasks_executed, 2);
        assert_eq!(report.tasks_completed, 2);
    }

    #[test]
    fn test_always_on_cycle_no_tasks() {
        let mut engine = AlwaysOnEngine::new();
        let report = engine.full_cycle().unwrap();
        assert_eq!(report.tasks_executed, 0);
        assert_eq!(report.tasks_completed, 0);
        assert_eq!(report.tasks_executed, 0);
        assert_eq!(report.tasks_completed, 0);
    }

    #[test]
    fn test_always_on_save_load_roundtrip() {
        let mut engine = AlwaysOnEngine::new();
        engine.add_recurring("health check", 600);
        engine.add_oneshot("cleanup");
        assert!(engine.save().is_ok());
        let loaded = AlwaysOnEngine::load();
        assert_eq!(loaded.tasks.len(), 2);
        assert_eq!(loaded.tasks[0].description, "health check");
        assert_eq!(loaded.tasks[1].description, "cleanup");
    }

    #[test]
    fn test_schedule_expr_parse_every() {
        let s = ScheduleExpr::parse("every 300").unwrap();
        assert!(matches!(s, ScheduleExpr::Every { interval_secs: 300 }));
    }

    #[test]
    fn test_schedule_expr_parse_daily() {
        let s = ScheduleExpr::parse("daily at 09:30").unwrap();
        assert!(matches!(
            s,
            ScheduleExpr::Daily {
                hour: 9,
                minute: 30
            }
        ));
    }

    #[test]
    fn test_schedule_expr_parse_hourly() {
        let s = ScheduleExpr::parse("hourly").unwrap();
        assert!(matches!(s, ScheduleExpr::Hourly));
    }

    #[test]
    fn test_schedule_expr_parse_weekly() {
        let s = ScheduleExpr::parse("weekly on 1 at 14:00").unwrap();
        assert!(matches!(
            s,
            ScheduleExpr::Weekly {
                weekday: 1,
                hour: 14,
                minute: 0
            }
        ));
    }

    #[test]
    fn test_schedule_expr_parse_invalid() {
        assert!(ScheduleExpr::parse("invalid").is_err());
        assert!(ScheduleExpr::parse("every").is_err());
    }

    #[test]
    fn test_schedule_expr_next_run_every() {
        let s = ScheduleExpr::Every { interval_secs: 60 };
        let now = chrono::Utc::now();
        let next = s.next_run(now);
        let diff = (next - now).num_seconds();
        assert!(diff >= 58 && diff <= 62);
    }

    #[test]
    fn test_schedule_expr_next_run_daily_tomorrow() {
        let s = ScheduleExpr::Daily {
            hour: 23,
            minute: 0,
        };
        let today_noon = chrono::Utc::now()
            .with_hour(12)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();
        let next = s.next_run(today_noon);
        let diff = (next - today_noon).num_seconds();
        assert!(diff > 0, "next_run should return a future time");
    }

    #[test]
    fn test_always_on_add_scheduled() {
        let mut engine = AlwaysOnEngine::new();
        let id = engine.add_scheduled("daily health", ScheduleExpr::Daily { hour: 6, minute: 0 });
        assert!(id.starts_with("sched_"));
        assert_eq!(engine.tasks.len(), 1);
        assert!(engine.tasks[0].schedule.is_some());
        assert!(engine.tasks[0].cron_description.is_some());
        assert_eq!(engine.tasks[0].max_runs, u64::MAX);
        assert!(engine.tasks[0].tags.contains(&"scheduled".into()));
    }

    #[test]
    fn test_always_on_list_scheduled() {
        let mut engine = AlwaysOnEngine::new();
        engine.add_recurring("normal", 300);
        engine.add_scheduled("sched1", ScheduleExpr::Hourly);
        engine.add_scheduled("sched2", ScheduleExpr::Daily { hour: 9, minute: 0 });
        let scheduled = engine.list_scheduled();
        assert_eq!(scheduled.len(), 2);
    }

    #[test]
    fn test_always_on_pause_resume_scheduled() {
        let mut engine = AlwaysOnEngine::new();
        let id = engine.add_scheduled("test", ScheduleExpr::Hourly);
        assert!(!engine.tasks[0].paused);
        assert!(engine.pause_scheduled(&id).is_ok());
        assert!(engine.tasks[0].paused);
        assert!(engine.resume_scheduled(&id).is_ok());
        assert!(!engine.tasks[0].paused);
        assert!(engine.pause_scheduled("nonexistent").is_err());
    }

    #[test]
    fn test_always_on_pause_non_scheduled() {
        let mut engine = AlwaysOnEngine::new();
        let id = engine.add_oneshot("test");
        assert!(engine.pause_scheduled(&id).is_err());
    }
}
