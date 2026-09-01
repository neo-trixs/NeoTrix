use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;

// ===== Enums =====

// ===== Structs =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopSchedule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expr: String,
    pub task_type: String,
    pub task_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: i64,
    pub last_run_at: Option<i64>,
    pub next_run_at: Option<i64>,
    pub run_count: u64,
    pub success_count: u64,
    pub fail_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopExecution {
    pub id: String,
    pub schedule_id: String,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub result_summary: Option<String>,
    pub error: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStats {
    pub total_schedules: usize,
    pub active_schedules: usize,
    pub executed_today: usize,
    pub failed_today: usize,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
    pub next_scheduled_run: Option<String>,
}

// ===== CronParser =====

struct CronField {
    minute: Vec<u32>,
    hour: Vec<u32>,
    day_of_month: Vec<u32>,
    month: Vec<u32>,
    day_of_week: Vec<u32>,
}

fn parse_cron_field(field: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
    if field == "*" {
        return Ok((min..=max).collect());
    }

    let mut result = Vec::new();
    for part in field.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err("Empty field part".into());
        }

        if part.contains('/') {
            let parts: Vec<&str> = part.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid step: {}", part));
            }
            let range = parts[0].trim();
            let step: u32 = parts[1].trim().parse().map_err(|_| format!("Invalid step value: {}", parts[1]))?;
            if step == 0 {
                return Err("Step cannot be zero".into());
            }
            let (start, end) = if range == "*" {
                (min, max)
            } else {
                let v: u32 = range.parse().map_err(|_| format!("Invalid number: {}", range))?;
                if v < min || v > max {
                    return Err(format!("Value {} out of range [{}, {}]", v, min, max));
                }
                (v, max)
            };
            let mut v = start;
            while v <= end {
                result.push(v);
                v = v.checked_add(step).ok_or("Overflow in step")?;
            }
        } else if part.contains('-') {
            let parts: Vec<&str> = part.splitn(2, '-').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid range: {}", part));
            }
            let lo: u32 = parts[0].trim().parse().map_err(|_| format!("Invalid number: {}", parts[0]))?;
            let hi: u32 = parts[1].trim().parse().map_err(|_| format!("Invalid number: {}", parts[1]))?;
            if lo > hi || lo < min || hi > max {
                return Err(format!("Range {} out of bounds [{}, {}]", part, min, max));
            }
            for v in lo..=hi {
                result.push(v);
            }
        } else {
            let v: u32 = part.parse().map_err(|_| format!("Invalid number: {}", part))?;
            if v < min || v > max {
                return Err(format!("Value {} out of range [{}, {}]", v, min, max));
            }
            result.push(v);
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}

impl CronField {
    fn parse(expr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(format!("Expected 5 cron fields, got {}: '{}'", parts.len(), expr));
        }
        Ok(CronField {
            minute: parse_cron_field(parts[0], 0, 59)?,
            hour: parse_cron_field(parts[1], 0, 23)?,
            day_of_month: parse_cron_field(parts[2], 1, 31)?,
            month: parse_cron_field(parts[3], 1, 12)?,
            day_of_week: parse_cron_field(parts[4], 0, 6)?,
        })
    }

    fn matches(&self, dt: &chrono::NaiveDateTime) -> bool {
        self.minute.contains(&{ dt.minute() })
            && self.hour.contains(&{ dt.hour() })
            && self.day_of_month.contains(&{ dt.day() })
            && self.month.contains(&{ dt.month() })
            && self.day_of_week.contains(&{ dt.weekday().num_days_from_sunday() })
    }
}

fn parse_keyword_cron(keyword: &str) -> Result<String, String> {
    match keyword {
        "every_1h" => Ok("0 * * * *".into()),
        "every_6h" => Ok("0 */6 * * *".into()),
        "every_12h" => Ok("0 */12 * * *".into()),
        "every_24h" | "daily" => Ok("0 0 * * *".into()),
        "every_monday" => Ok("0 0 * * 1".into()),
        "every_weekday" => Ok("0 9 * * 1-5".into()),
        "every_week" | "weekly" => Ok("0 0 * * 0".into()),
        "every_month" | "monthly" => Ok("0 0 1 * *".into()),
        _ => {
            // Check if it's already a valid 5-field cron
            let trimmed = keyword.trim();
            if trimmed.split_whitespace().count() == 5 {
                Ok(trimmed.to_string())
            } else {
                Err(format!("Unknown cron keyword or invalid format: {}", keyword))
            }
        }
    }
}

pub fn parse_cron_expression(expression: &str) -> Result<String, String> {
    let trimmed = expression.trim();
    // Check if it's a human keyword first
    if trimmed.contains("every_") || trimmed == "daily" || trimmed == "weekly" || trimmed == "monthly" || trimmed == "weekdays" {
        return parse_keyword_cron(trimmed);
    }
    // Validate it parses as a 5-field cron
    let fields = CronField::parse(trimmed)?;
    // Normalize: reconstruct from fields
    Ok(format!(
        "{} {} {} {} {}",
        fields.minute.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
        fields.hour.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
        fields.day_of_month.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
        fields.month.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
        fields.day_of_week.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
    ))
}

fn calculate_next_run(cron_expr: &str, after: i64) -> Result<i64, String> {
    let resolved = parse_cron_expression(cron_expr)?;
    let fields = CronField::parse(&resolved)?;

    // Resolve keyword to full cron for parsing
    let start_ts = if after <= 0 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
    } else {
        after
    };

    let base = chrono::DateTime::from_timestamp(start_ts, 0)
        .ok_or("Invalid timestamp")?
        .naive_utc();

    // Check up to 2 years ahead (525600 minutes)
    let max_minutes: i64 = 525600;
    for offset in 1..=max_minutes {
        let candidate = base + chrono::Duration::minutes(offset);
        if fields.matches(&candidate) {
            return Ok(candidate.and_utc().timestamp());
        }
    }
    Err("Could not find next run time within 1 year".into())
}

// ===== State =====

const MAX_SCHEDULES: usize = 100;
const MAX_EXECUTIONS: usize = 1000;

struct LoopState {
    schedules: Vec<LoopSchedule>,
    executions: Vec<LoopExecution>,
}

impl Default for LoopState {
    fn default() -> Self {
        let now = now_ts();
        let next_6h = calculate_next_run("every_6h", now).unwrap_or(now + 21600);
        let next_24h = calculate_next_run("every_24h", now).unwrap_or(now + 86400);
        let next_monday = calculate_next_run("0 9 * * 1", now).unwrap_or(now + 604800);

        LoopState {
            schedules: vec![
                LoopSchedule {
                    id: "sched-pr".into(),
                    name: "PR Inspection".into(),
                    description: "Check open PRs for status changes and required reviews".into(),
                    cron_expr: "every_6h".into(),
                    task_type: "pr_inspection".into(),
                    task_config: serde_json::json!({}),
                    enabled: true,
                    created_at: now,
                    last_run_at: None,
                    next_run_at: Some(next_6h),
                    run_count: 0,
                    success_count: 0,
                    fail_count: 0,
                },
                LoopSchedule {
                    id: "sched-code-scan".into(),
                    name: "Daily Code Scan".into(),
                    description: "Periodic code quality scan for warnings and issues".into(),
                    cron_expr: "every_24h".into(),
                    task_type: "code_scan".into(),
                    task_config: serde_json::json!({}),
                    enabled: true,
                    created_at: now,
                    last_run_at: None,
                    next_run_at: Some(next_24h),
                    run_count: 0,
                    success_count: 0,
                    fail_count: 0,
                },
                LoopSchedule {
                    id: "sched-weekly".into(),
                    name: "Weekly Report".into(),
                    description: "Generate weekly summary report every Monday at 9 AM".into(),
                    cron_expr: "0 9 * * 1".into(),
                    task_type: "reminder".into(),
                    task_config: serde_json::json!({}),
                    enabled: false,
                    created_at: now,
                    last_run_at: None,
                    next_run_at: Some(next_monday),
                    run_count: 0,
                    success_count: 0,
                    fail_count: 0,
                },
            ],
            executions: Vec::new(),
        }
    }
}

static STATE: std::sync::LazyLock<Mutex<LoopState>> =
    std::sync::LazyLock::new(|| Mutex::new(LoopState::default()));

// ===== Helpers =====

fn short_uid() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:08x}", nanos)
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn today_start_ts() -> i64 {
    use chrono::Utc;
    Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

fn simulate_task(task_type: &str) -> (String, Option<String>) {
    match task_type {
        "pr_inspection" => ("Checked 5 PRs, 2 need review".into(), None),
        "deploy_monitor" => ("All deployments healthy".into(), None),
        "code_scan" => ("Scanned 120 files, 3 warnings".into(), None),
        "reminder" => ("Reminder sent".into(), None),
        _ => ("Custom task executed".into(), None),
    }
}

// ===== Commands =====

#[command]
pub fn loop_create(
    name: String,
    description: String,
    cron_expr: String,
    task_type: String,
    task_config: Option<serde_json::Value>,
) -> Result<String, String> {
    // Validate cron expression
    parse_cron_expression(&cron_expr)?;

    let id = format!("loop-{}", short_uid());
    let now = now_ts();
    let next_run = calculate_next_run(&cron_expr, now).ok();

    let schedule = LoopSchedule {
        id: id.clone(),
        name,
        description,
        cron_expr,
        task_type,
        task_config: task_config.unwrap_or(serde_json::Value::Null),
        enabled: true,
        created_at: now,
        last_run_at: None,
        next_run_at: next_run,
        run_count: 0,
        success_count: 0,
        fail_count: 0,
    };

    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    if state.schedules.len() >= MAX_SCHEDULES {
        return Err(format!("Maximum of {} schedules reached", MAX_SCHEDULES));
    }
    state.schedules.push(schedule);
    Ok(id)
}

#[command]
pub fn loop_list() -> Result<Vec<LoopSchedule>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.schedules.clone())
}

#[command]
pub fn loop_get(id: String) -> Result<LoopSchedule, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state
        .schedules
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("Schedule not found: {}", id))
}

#[command]
pub fn loop_update(
    id: String,
    name: Option<String>,
    description: Option<String>,
    cron_expr: Option<String>,
    enabled: Option<bool>,
    task_config: Option<serde_json::Value>,
) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let sched = state
        .schedules
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Schedule not found: {}", id))?;

    if let Some(v) = name {
        sched.name = v;
    }
    if let Some(v) = description {
        sched.description = v;
    }
    if let Some(ref v) = cron_expr {
        parse_cron_expression(v)?;
        sched.cron_expr = v.clone();
        sched.next_run_at = calculate_next_run(v, now_ts()).ok();
    }
    if let Some(v) = enabled {
        sched.enabled = v;
    }
    if let Some(v) = task_config {
        sched.task_config = v;
    }
    Ok(())
}

#[command]
pub fn loop_delete(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let before = state.schedules.len();
    state.schedules.retain(|s| s.id != id);
    state.executions.retain(|e| e.schedule_id != id);
    if state.schedules.len() == before {
        return Err(format!("Schedule not found: {}", id));
    }
    Ok(())
}

#[command]
pub fn loop_enable(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let sched = state
        .schedules
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Schedule not found: {}", id))?;
    sched.enabled = true;
    Ok(())
}

#[command]
pub fn loop_disable(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let sched = state
        .schedules
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Schedule not found: {}", id))?;
    sched.enabled = false;
    Ok(())
}

#[command]
pub fn loop_execute_now(id: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let sched = state
        .schedules
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Schedule not found: {}", id))?
        .clone();

    let run_id = format!("exec-{}", short_uid());
    let started = now_ts();

    if state.executions.len() >= MAX_EXECUTIONS {
        state.executions.remove(0);
    }
    state.executions.push(LoopExecution {
        id: run_id.clone(),
        schedule_id: id.clone(),
        status: "running".into(),
        started_at: started,
        completed_at: None,
        duration_ms: None,
        result_summary: None,
        error: None,
        output: None,
    });
    drop(state);

    // Simulate work
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elapsed_ms = (now_ts() - started).max(1) as u64 * 1000;
    let (summary, error) = simulate_task(&sched.task_type);

    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    if let Some(run) = state.executions.iter_mut().find(|r| r.id == run_id) {
        run.status = if error.is_some() { "failed".to_string() } else { "completed".to_string() };
        run.completed_at = Some(now_ts());
        run.duration_ms = Some(elapsed_ms);
        run.result_summary = Some(summary.clone());
        run.error = error.clone();
        run.output = Some(summary.clone());
    }

    if let Some(s) = state.schedules.iter_mut().find(|s| s.id == id) {
        s.last_run_at = Some(now_ts());
        s.next_run_at = calculate_next_run(&s.cron_expr, now_ts()).ok();
        s.run_count += 1;
        if error.is_some() {
            s.fail_count += 1;
        } else {
            s.success_count += 1;
        }
    }

    Ok(summary)
}

#[command]
pub fn loop_execution_history(
    id: String,
    count: Option<usize>,
) -> Result<Vec<LoopExecution>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let max = count.unwrap_or(50).min(200);
    let mut history: Vec<_> = state
        .executions
        .iter()
        .filter(|e| e.schedule_id == id)
        .cloned()
        .collect();
    history.reverse();
    history.truncate(max);
    Ok(history)
}

#[command]
pub fn loop_stats() -> Result<LoopStats, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let total_schedules = state.schedules.len();
    let active_schedules = state.schedules.iter().filter(|s| s.enabled).count();
    let today = today_start_ts();

    let today_execs: Vec<_> = state.executions.iter().filter(|e| e.started_at >= today).collect();
    let executed_today = today_execs.len();
    let failed_today = today_execs.iter().filter(|e| e.status == "failed").count();

    let total_all = state.executions.len();
    let total_completed = state.executions.iter().filter(|e| e.status == "completed").count();
    let success_rate = if total_all > 0 {
        total_completed as f64 / total_all as f64
    } else {
        1.0
    };

    let avg_duration_ms = {
        let sum: u64 = state
            .executions
            .iter()
            .filter(|e| e.status == "completed")
            .filter_map(|e| e.duration_ms)
            .sum();
        let n = state
            .executions
            .iter()
            .filter(|e| e.status == "completed" && e.duration_ms.is_some())
            .count();
        if n > 0 {
            sum as f64 / n as f64
        } else {
            0.0
        }
    };

    let next_scheduled_run = state
        .schedules
        .iter()
        .filter(|s| s.enabled)
        .filter_map(|s| s.next_run_at)
        .min()
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_else(|| ts.to_string())
        });

    Ok(LoopStats {
        total_schedules,
        active_schedules,
        executed_today,
        failed_today,
        avg_duration_ms,
        success_rate,
        next_scheduled_run,
    })
}

#[command]
pub fn loop_next_scheduled() -> Result<Option<LoopSchedule>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let now = now_ts();
    let next = state
        .schedules
        .iter()
        .filter(|s| s.enabled)
        .filter(|s| s.next_run_at.map(|n| n <= now + 86400 * 365).unwrap_or(false))
        .min_by_key(|s| s.next_run_at.unwrap_or(i64::MAX))
        .cloned();
    // Only return if next_run_at is actually in the future
    match next {
        Some(s) if s.next_run_at.map(|n| n > now).unwrap_or(false) => Ok(Some(s)),
        _ => Ok(None),
    }
}

#[command]
pub fn loop_validate_cron(expression: String) -> Result<bool, String> {
    match parse_cron_expression(&expression) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[command]
pub fn loop_tick() -> Result<Vec<String>, String> {
    let now = now_ts();

    // Collect due schedules info in one pass (drop lock immediately)
    let due: Vec<(String, String, String, String)> = {
        let state = STATE.lock().map_err(|e| e.to_string())?;
        state
            .schedules
            .iter()
            .filter(|s| s.enabled)
            .filter(|s| s.next_run_at.map(|n| n <= now).unwrap_or(false))
            .map(|s| (s.id.clone(), s.name.clone(), s.task_type.clone(), s.cron_expr.clone()))
            .collect()
    };

    let mut executed: Vec<String> = Vec::new();

    for (id, name, task_type, cron_expr) in &due {
        let run_id = format!("exec-{}", short_uid());
        let started = now;

        // Create execution record
        {
            let mut state = STATE.lock().map_err(|e| e.to_string())?;
            if state.executions.len() >= MAX_EXECUTIONS {
                state.executions.remove(0);
            }
            state.executions.push(LoopExecution {
                id: run_id.clone(),
                schedule_id: id.clone(),
                status: "running".into(),
                started_at: started,
                completed_at: None,
                duration_ms: None,
                result_summary: None,
                error: None,
                output: None,
            });
        }

        // Simulate work (shorter than execute_now since this runs in background)
        std::thread::sleep(std::time::Duration::from_millis(200));

        let elapsed_ms = (now_ts() - started).max(1) as u64 * 1000;
        let (summary, error) = simulate_task(task_type);

        // Update execution record + schedule
        {
            let mut state = STATE.lock().map_err(|e| e.to_string())?;
            if let Some(run) = state.executions.iter_mut().find(|r| r.id == run_id) {
                run.status = if error.is_some() { "failed".to_string() } else { "completed".to_string() };
                run.completed_at = Some(now_ts());
                run.duration_ms = Some(elapsed_ms);
                run.result_summary = Some(summary.clone());
                run.error = error.clone();
                run.output = Some(summary.clone());
            }
            if let Some(s) = state.schedules.iter_mut().find(|s| s.id == *id) {
                s.last_run_at = Some(now_ts());
                s.next_run_at = calculate_next_run(cron_expr, now_ts()).ok();
                s.run_count += 1;
                if error.is_some() {
                    s.fail_count += 1;
                } else {
                    s.success_count += 1;
                }
            }
        }

        executed.push(name.clone());
    }

    Ok(executed)
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_create_and_list() {
        let id = loop_create(
            "Test Loop".into(),
            "Test description".into(),
            "every_1h".into(),
            "reminder".into(),
            None,
        )
        .unwrap();
        assert!(id.starts_with("loop-"));

        let list = loop_list().unwrap();
        assert!(list.iter().any(|s| s.id == id));
    }

    #[test]
    fn test_loop_execute_now() {
        let id = loop_create(
            "Exec Test".into(),
            "Execution test".into(),
            "every_6h".into(),
            "code_scan".into(),
            None,
        )
        .unwrap();
        let result = loop_execute_now(id.clone()).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains("Scanned") || result.contains("Checked") || result.contains("sent") || result.contains("executed"));

        let sched = loop_get(id).unwrap();
        assert!(sched.last_run_at.is_some());
        assert_eq!(sched.run_count, 1);
    }

    #[test]
    fn test_loop_stats() {
        let stats = loop_stats().unwrap();
        assert!(stats.total_schedules >= 3);
        assert!(stats.active_schedules >= 2);
    }

    #[test]
    fn test_loop_validate_cron() {
        assert!(loop_validate_cron("every_1h".into()).unwrap());
        assert!(loop_validate_cron("0 */6 * * *".into()).unwrap());
        assert!(!loop_validate_cron("invalid cron".into()).unwrap());
        assert!(loop_validate_cron("every_24h".into()).unwrap());
        assert!(loop_validate_cron("0 9 * * 1-5".into()).unwrap());
        assert!(!loop_validate_cron("".into()).unwrap());
    }

    #[test]
    fn test_loop_tick() {
        // Verify tick returns empty when nothing is due
        let executed = loop_tick().unwrap();
        // Should be empty since next_run_at is in the future
        assert!(executed.is_empty());
    }
}
