use serde::{Deserialize, Serialize};
use super::history::{JobRunHistory, JobRunRecord, SchedulerStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    pub schedule: ScheduleType,
    pub handler: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: u64,
    pub max_retries: u32,
    pub retry_count: u32,
    pub cooldown_secs: u64,
    pub anchor_ts: Option<u64>,
    pub context_gate: ContextGate,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Interval { secs: u64 },
    Cron(String),
    OneTime(u64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextGate {
    Any,
    LowCogLoad(f64),
    MinDaLevel(f64),
    SleepPressure(f64),
    ExplorationMode,
}

impl Default for ContextGate {
    fn default() -> Self { ContextGate::Any }
}

#[derive(Debug, Clone)]
pub struct SchedulerEngine {
    jobs: Vec<ScheduledJob>,
    history: JobRunHistory,
    tick_count: u64,
}

impl SchedulerEngine {
    pub fn new() -> Self {
        Self { jobs: Vec::new(), history: JobRunHistory::new(1000), tick_count: 0 }
    }

    pub fn add_job(&mut self, job: ScheduledJob) {
        let mut job = job;
        if job.next_run == 0 {
            job.next_run = compute_next_run(&job.schedule, job.anchor_ts, current_unix_ts());
        }
        self.jobs.push(job);
    }

    pub fn remove_job(&mut self, job_id: &str) -> bool {
        let len = self.jobs.len();
        self.jobs.retain(|j| j.id != job_id);
        self.jobs.len() < len
    }

    pub fn get_job(&self, job_id: &str) -> Option<&ScheduledJob> {
        self.jobs.iter().find(|j| j.id == job_id)
    }

    fn get_job_mut(&mut self, job_id: &str) -> Option<&mut ScheduledJob> {
        self.jobs.iter_mut().find(|j| j.id == job_id)
    }

    pub fn tick(
        &mut self,
        now_ts: u64,
        cog_load: f64,
        da_level: f64,
        sleep_pressure: f64,
        curiosity_level: f64,
    ) -> Vec<(String, String)> {
        self.tick_count += 1;
        let mut due = Vec::new();
        let mut i = 0;
        while i < self.jobs.len() {
            let pass = {
                let j = &self.jobs[i];
                j.enabled && j.next_run <= now_ts
                    && match j.context_gate {
                        ContextGate::Any => true,
                        ContextGate::LowCogLoad(max) => cog_load <= max,
                        ContextGate::MinDaLevel(min) => da_level >= min,
                        ContextGate::SleepPressure(max) => sleep_pressure <= max,
                        ContextGate::ExplorationMode => curiosity_level >= 0.5,
                    }
            };
            if pass {
                let job = &mut self.jobs[i];
                job.last_run = Some(now_ts);
                job.next_run = compute_next_run(&job.schedule, job.anchor_ts, now_ts);
                due.push((job.id.clone(), job.handler.clone()));
            }
            i += 1;
        }
        due
    }

    pub fn record_run(
        &mut self, job_id: &str, started_at: u64,
        duration_ms: u64, success: bool, error: Option<String>,
    ) {
        let retry_count = self.history.last_run(job_id)
            .map(|r| r.retry_count).unwrap_or(0u32);
        let new_retry_count = if success { 0 } else { retry_count + 1 };
        self.history.push(JobRunRecord {
            job_id: job_id.to_string(), started_at, duration_ms,
            success, error, retry_count: new_retry_count,
        });
        if !success {
            if let Some(job) = self.get_job_mut(job_id) {
                if new_retry_count >= job.max_retries { job.enabled = false; }
            }
        }
    }

    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            total_jobs: self.jobs.len(),
            enabled_jobs: self.jobs.iter().filter(|j| j.enabled).count(),
            total_runs: self.history.total_runs(),
            failed_runs: self.history.total_failures(),
            success_rate: self.history.success_rate(),
            registered_handler_count: self.jobs.iter()
                .map(|j| j.handler.as_str())
                .collect::<std::collections::HashSet<_>>().len(),
        }
    }

    pub fn save_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.jobs)
            .map_err(|e| format!("scheduler serialization: {}", e))
    }

    pub fn load_json(&mut self, json: &str) -> Result<(), String> {
        let jobs: Vec<ScheduledJob> = serde_json::from_str(json)
            .map_err(|e| format!("scheduler deserialization: {}", e))?;
        self.jobs = jobs;
        Ok(())
    }

    pub fn tick_count(&self) -> u64 { self.tick_count }
}

impl Default for SchedulerEngine { fn default() -> Self { Self::new() } }

pub fn compute_next_run(schedule: &ScheduleType, anchor_ts: Option<u64>, now_ts: u64) -> u64 {
    match schedule {
        ScheduleType::Interval { secs } if *secs == 0 => now_ts,
        ScheduleType::Interval { secs } => {
            let interval = *secs as u64;
            let anchor = anchor_ts.unwrap_or(now_ts);
            if now_ts < anchor { return anchor; }
            let steps = (now_ts - anchor) / interval;
            anchor + (steps + 1) * interval
        }
        ScheduleType::Cron(expr) => next_cron(expr, now_ts).unwrap_or(now_ts + 3600),
        ScheduleType::OneTime(ts) => if *ts > now_ts { *ts } else { now_ts + 86400 * 365 },
    }
}

fn next_cron(expr: &str, now_ts: u64) -> Option<u64> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 { return None; }
    let minutes = parse_cron_field(parts[0], 0, 59)?;
    let hours = parse_cron_field(parts[1], 0, 23)?;
    let days = parse_cron_field(parts[2], 1, 31)?;
    let months = parse_cron_field(parts[3], 1, 12)?;
    let weekdays = parse_cron_field(parts[4], 0, 6)?;
    let start = now_ts - now_ts % 60 + 60;
    let mut ts = start;
    for _ in 0..(525600 * 5) {
        let (y, m, d, h, min) = ts_to_calendar(ts);
        if y == 0 { break; }
        if months.contains(&(m as i64)) && days.contains(&(d as i64))
            && hours.contains(&(h as i64)) && minutes.contains(&(min as i64))
        {
            let dow = day_of_week(y, m, d);
            if weekdays.contains(&(dow as i64)) { return Some(ts); }
        }
        ts += 60;
    }
    None
}

fn parse_cron_field(field: &str, min_val: i64, max_val: i64) -> Option<Vec<i64>> {
    if field == "*" { return Some((min_val..=max_val).collect()); }
    let mut values = Vec::new();
    for part in field.split(',') {
        let v: i64 = part.trim().parse().ok()?;
        if v < min_val || v > max_val { return None; }
        values.push(v);
    }
    if values.is_empty() { None } else { Some(values) }
}

fn ts_to_calendar(ts: u64) -> (i64, i64, i64, i64, i64) {
    let days = ts / 86400;
    let time_secs = ts % 86400;
    let hour = (time_secs / 3600) as i64;
    let minute = ((time_secs % 3600) / 60) as i64;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        y += 1;
        if y > 2100 { return (0, 0, 0, 0, 0); }
    }
    let month_days = [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1i64;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        m += 1;
    }
    (y, m, remaining + 1, hour, minute)
}

fn is_leap(y: i64) -> bool { (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 }

fn day_of_week(y: i64, m: i64, d: i64) -> i64 {
    let (y_adj, m_adj) = if m < 3 { (y - 1, m + 12) } else { (y, m) };
    let k = y_adj % 100;
    let j = y_adj / 100;
    (d + (13 * (m_adj + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7
}

fn current_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs()
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_next_run_interval_basic() {
        assert_eq!(compute_next_run(&ScheduleType::Interval { secs: 3600 }, None, 0), 3600);
    }

    #[test]
    fn test_compute_next_run_interval_with_anchor() {
        let s = ScheduleType::Interval { secs: 3600 };
        assert_eq!(compute_next_run(&s, Some(0), 3600), 7200);
        assert_eq!(compute_next_run(&s, Some(0), 3601), 7200);
        assert_eq!(compute_next_run(&s, Some(0), 0), 0);
    }

    #[test]
    fn test_compute_next_run_zero_interval() {
        assert_eq!(compute_next_run(&ScheduleType::Interval { secs: 0 }, None, 100), 100);
    }

    #[test]
    fn test_compute_next_run_one_time() {
        assert_eq!(compute_next_run(&ScheduleType::OneTime(5000), None, 1000), 5000);
        assert!(compute_next_run(&ScheduleType::OneTime(5000), None, 6000) > 5000);
    }

    #[test]
    fn test_compute_next_run_cron() {
        let s = ScheduleType::Cron("30 * * * *".into());
        assert_eq!(compute_next_run(&s, None, 3600 * 10), 3600 * 10 + 30 * 60);
        let s = ScheduleType::Cron("0 * * * *".into());
        assert_eq!(compute_next_run(&s, None, 3600 * 10), 3600 * 11);
    }

    #[test]
    fn test_parse_cron_field() {
        assert_eq!(parse_cron_field("*", 0, 59).unwrap().len(), 60);
        assert_eq!(parse_cron_field("15,30", 0, 59).unwrap(), vec![15, 30]);
        assert!(parse_cron_field("60", 0, 59).is_none());
        assert!(parse_cron_field("x", 0, 59).is_none());
    }

    #[test]
    fn test_ts_to_calendar_epoch() {
        let (y, m, d, h, min) = ts_to_calendar(0);
        assert_eq!((y, m, d, h, min), (1970, 1, 1, 0, 0));
    }

    #[test]
    fn test_is_leap() {
        assert!(is_leap(2000)); assert!(is_leap(2024));
        assert!(!is_leap(2100)); assert!(!is_leap(2023));
    }

    #[test]
    fn test_scheduler_add_and_get() {
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "t".into(), name: "T".into(), schedule: ScheduleType::Interval { secs: 60 },
            handler: "h".into(), enabled: true, last_run: None, next_run: 100,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        assert!(s.get_job("t").is_some());
        assert!(s.get_job("x").is_none());
    }

    #[test]
    fn test_scheduler_remove() {
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "x".into(), name: "X".into(), schedule: ScheduleType::Interval { secs: 60 },
            handler: "h".into(), enabled: true, last_run: None, next_run: 100,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        assert!(s.remove_job("x")); assert!(!s.remove_job("nonexistent"));
        assert_eq!(s.stats().total_jobs, 0);
    }

    #[test]
    fn test_scheduler_tick_returns_due() {
        let mut s = SchedulerEngine::new();
        let now = 1000;
        s.add_job(ScheduledJob {
            id: "due_now".into(), name: "D".into(), schedule: ScheduleType::Interval { secs: 3600 },
            handler: "h".into(), enabled: true, last_run: None, next_run: now - 1,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        s.add_job(ScheduledJob {
            id: "not_due".into(), name: "N".into(), schedule: ScheduleType::Interval { secs: 3600 },
            handler: "h".into(), enabled: true, last_run: None, next_run: now + 1000,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        let due = s.tick(now, 0.0, 0.5, 0.0, 0.5);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn test_scheduler_gate_low_cog() {
        let mut s = SchedulerEngine::new();
        let now = 1000;
        s.add_job(ScheduledJob {
            id: "cog".into(), name: "C".into(), schedule: ScheduleType::Interval { secs: 3600 },
            handler: "h".into(), enabled: true, last_run: None, next_run: now - 1,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::LowCogLoad(0.5), description: "".into(),
        });
        assert_eq!(s.tick(now, 0.8, 0.5, 0.0, 0.5).len(), 0); // blocked
        assert_eq!(s.tick(now, 0.3, 0.5, 0.0, 0.5).len(), 1); // passes
    }

    #[test]
    fn test_scheduler_disabled_job_not_ticked() {
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "off".into(), name: "O".into(), schedule: ScheduleType::Interval { secs: 60 },
            handler: "h".into(), enabled: false, last_run: None, next_run: 0,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        assert_eq!(s.tick(100, 0.0, 0.5, 0.0, 0.5).len(), 0);
    }

    #[test]
    fn test_record_run_auto_disable() {
        let mut s = SchedulerEngine::new();
        let now = 1000;
        s.add_job(ScheduledJob {
            id: "flaky".into(), name: "F".into(), schedule: ScheduleType::Interval { secs: 60 },
            handler: "h".into(), enabled: true, last_run: None, next_run: now - 1,
            max_retries: 2, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        s.record_run("flaky", now, 100, false, Some("fail1".into()));
        assert!(s.get_job("flaky").unwrap().enabled);
        s.record_run("flaky", now + 60, 100, false, Some("fail2".into()));
        assert!(s.get_job("flaky").unwrap().enabled);
        s.record_run("flaky", now + 120, 100, false, Some("fail3".into()));
        assert!(!s.get_job("flaky").unwrap().enabled);
    }

    #[test]
    fn test_record_run_success_resets_retry() {
        let mut s = SchedulerEngine::new();
        s.record_run("ok", 0, 50, true, None);
        s.record_run("ok", 60, 50, false, Some("err".into()));
        assert_eq!(s.history.last_run("ok").unwrap().retry_count, 1);
        s.record_run("ok", 120, 50, true, None);
        assert_eq!(s.history.last_run("ok").unwrap().retry_count, 0);
    }

    #[test]
    fn test_scheduler_stats() {
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "a".into(), name: "A".into(), schedule: ScheduleType::Interval { secs: 60 },
            handler: "h".into(), enabled: true, last_run: None, next_run: 0,
            max_retries: 3, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        s.add_job(ScheduledJob {
            id: "b".into(), name: "B".into(), schedule: ScheduleType::Interval { secs: 120 },
            handler: "h2".into(), enabled: false, last_run: None, next_run: 9999,
            max_retries: 1, retry_count: 0, cooldown_secs: 5, anchor_ts: None,
            context_gate: ContextGate::Any, description: "".into(),
        });
        let stats = s.stats();
        assert_eq!(stats.total_jobs, 2);
        assert_eq!(stats.enabled_jobs, 1);
    }

    #[test]
    fn test_scheduler_tick_count() {
        let mut s = SchedulerEngine::new();
        assert_eq!(s.tick_count(), 0);
        s.tick(0, 0.0, 0.5, 0.0, 0.5);
        assert_eq!(s.tick_count(), 1);
    }

    #[test]
    fn test_scheduler_save_load_json() {
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "j".into(), name: "J".into(), schedule: ScheduleType::Interval { secs: 3600 },
            handler: "h".into(), enabled: true, last_run: None, next_run: 9999,
            max_retries: 2, retry_count: 0, cooldown_secs: 10, anchor_ts: None,
            context_gate: ContextGate::Any, description: "desc".into(),
        });
        let json = s.save_json().unwrap();
        let mut s2 = SchedulerEngine::new();
        s2.load_json(&json).unwrap();
        assert_eq!(s2.stats().total_jobs, 1);
        assert_eq!(s2.get_job("j").unwrap().description, "desc");
    }

    #[test]
    fn test_default_scheduler_jobs() {
        let now = current_unix_ts();
        let mut s = SchedulerEngine::new();
        s.add_job(ScheduledJob {
            id: "build_cleanup".into(), name: "Cleanup".into(),
            schedule: ScheduleType::Interval { secs: 86400 },
            handler: "handle_build_cleanup".into(), enabled: true,
            last_run: None, next_run: 0, max_retries: 2, retry_count: 0,
            cooldown_secs: 3600, anchor_ts: Some(now),
            context_gate: ContextGate::LowCogLoad(0.6), description: "".into(),
        });
        // Not due yet (anchor + 86400 > now since anchor = now)
        assert_eq!(s.tick(now, 0.3, 0.5, 0.0, 0.5).len(), 0);
    }
}
