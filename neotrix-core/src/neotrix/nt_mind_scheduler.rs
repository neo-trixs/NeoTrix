// DEPRECATED: Superseded by core/nt_core_scheduler/ (OpenClaw-inspired scheduler with
// anchor mechanism, context gates, job history, auto-retry). Will be removed in a future
// cleanup pass. All new scheduling should use SchedulerEngine from nt_core_scheduler.

use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Cron(String),
    Interval(u64),
    OneTime(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    pub schedule: ScheduleType,
    pub task_type: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheduler {
    jobs: Vec<ScheduledJob>,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { jobs: Vec::new() }
    }

    pub fn add_job(&mut self, job: ScheduledJob) {
        self.jobs.push(job);
    }

    pub fn remove_job(&mut self, id: &str) -> bool {
        let len = self.jobs.len();
        self.jobs.retain(|j| j.id != id);
        self.jobs.len() < len
    }

    pub fn next_due(&self) -> Option<&ScheduledJob> {
        let now = unix_now();
        self.jobs.iter()
            .filter(|j| j.enabled && j.next_run <= now)
            .min_by_key(|j| j.next_run)
    }

    pub fn tick(&mut self) -> Vec<&ScheduledJob> {
        let now = unix_now();
        let indices: Vec<usize> = self.jobs.iter().enumerate()
            .filter(|(_, j)| j.enabled && j.next_run <= now)
            .map(|(i, _)| i)
            .collect();
        for &i in &indices {
            let job = &mut self.jobs[i];
            job.last_run = Some(now);
            match &job.schedule {
                ScheduleType::Interval(secs) => {
                    job.next_run = now + secs;
                }
                ScheduleType::OneTime(_) => {
                    job.enabled = false;
                }
                ScheduleType::Cron(expr) => {
                    job.next_run = next_cron_run(expr, now);
                }
            }
        }
        indices.into_iter().map(|i| &self.jobs[i]).collect()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create parent dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialization error: {}", e))?;
        fs::write(path, json).map_err(|e| format!("write error: {}", e))
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("deserialization error: {}", e))
    }
}

pub fn parse_cron(expr: &str) -> Result<Vec<u64>, String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("cron expression must have exactly 5 fields".into());
    }
    if parts[1] != "*" {
        return Err("hour-specific cron patterns not supported, use * for hour field".into());
    }
    let minute_field = parts[0];
    if minute_field == "*" {
        return Ok((0..60).collect());
    }
    if minute_field.contains(',') {
        let mut result = Vec::new();
        for part in minute_field.split(',') {
            result.extend(parse_single_minute_field(part)?);
        }
        result.sort();
        result.dedup();
        return Ok(result);
    }
    parse_single_minute_field(minute_field)
}

fn parse_single_minute_field(field: &str) -> Result<Vec<u64>, String> {
    if let Some(step_str) = field.strip_prefix("*/") {
        let step: u64 = step_str.parse().map_err(|_| format!("invalid step: {}", step_str))?;
        if step == 0 || step > 59 {
            return Err(format!("step must be 1..59, got: {}", step));
        }
        Ok((0..60).step_by(step as usize).collect())
    } else {
        let val: u64 = field.parse().map_err(|_| format!("invalid minute value: {}", field))?;
        if val > 59 {
            return Err(format!("minute value out of range: {}", val));
        }
        Ok(vec![val])
    }
}

fn next_cron_run(expr: &str, after: u64) -> u64 {
    let minutes = parse_cron(expr).unwrap_or_default();
    if minutes.is_empty() {
        return after + 3600;
    }
    let current_minute = (after / 60) % 60;
    let hour_start = after - (after % 3600);
    for &m in &minutes {
        if m > current_minute {
            return hour_start + m * 60;
        }
    }
    hour_start + 3600 + minutes[0] * 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_new_empty() {
        let s = Scheduler::new();
        assert!(s.next_due().is_none());
        let mut s = Scheduler::new();
        assert_eq!(s.tick().len(), 0);
    }

    #[test]
    fn test_scheduler_add_remove() {
        let mut s = Scheduler::new();
        let job = ScheduledJob {
            id: "test-1".into(),
            name: "Test".into(),
            schedule: ScheduleType::Interval(60),
            task_type: "knowledge_mine".into(),
            enabled: true,
            last_run: None,
            next_run: 0,
        };
        s.add_job(job);
        assert!(s.next_due().is_some());
        assert!(s.remove_job("test-1"));
        assert!(!s.remove_job("test-1"));
        assert!(s.next_due().is_none());
    }

    #[test]
    fn test_scheduler_tick_interval() {
        let mut s = Scheduler::new();
        let now = unix_now();
        s.add_job(ScheduledJob {
            id: "tick-test".into(),
            name: "Tick".into(),
            schedule: ScheduleType::Interval(86400),
            task_type: "evolve".into(),
            enabled: true,
            last_run: None,
            next_run: 0,
        });
        let due = s.tick();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "tick-test");
        assert!(due[0].last_run.expect("last_run should be set after tick") >= now);
        assert!(due[0].next_run >= now + 86400);
    }

    #[test]
    fn test_scheduler_tick_onetime() {
        let mut s = Scheduler::new();
        let _now = unix_now();
        s.add_job(ScheduledJob {
            id: "one-shot".into(),
            name: "OneTime".into(),
            schedule: ScheduleType::OneTime(0),
            task_type: "evolve".into(),
            enabled: true,
            last_run: None,
            next_run: 0,
        });
        let due = s.tick();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "one-shot");
        assert!(!due[0].enabled);
    }

    #[test]
    fn test_scheduler_disabled_job_not_due() {
        let mut s = Scheduler::new();
        s.add_job(ScheduledJob {
            id: "disabled".into(),
            name: "Disabled".into(),
            schedule: ScheduleType::Interval(60),
            task_type: "metacognition".into(),
            enabled: false,
            last_run: None,
            next_run: 0,
        });
        assert!(s.next_due().is_none());
        assert_eq!(s.tick().len(), 0);
    }

    #[test]
    fn test_parse_cron_every_minute() {
        let m = parse_cron("* * * * *").expect("parse_cron with '*' should return all 60 minutes");
        assert_eq!(m.len(), 60);
        assert_eq!(m[0], 0);
        assert_eq!(m[59], 59);
    }

    #[test]
    fn test_parse_cron_every_5_minutes() {
        let m = parse_cron("*/5 * * * *").expect("parse_cron with '*/5' should succeed");
        assert_eq!(m, vec![0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55]);
    }

    #[test]
    fn test_parse_cron_hourly_at_minute_0() {
        let m = parse_cron("0 * * * *").expect("parse_cron with '0 * * * *' should succeed");
        assert_eq!(m, vec![0]);
    }

    #[test]
    fn test_parse_cron_minute_list() {
        let m = parse_cron("0,30 * * * *").expect("parse_cron with '0,30' should succeed");
        assert_eq!(m, vec![0, 30]);
    }

    #[test]
    fn test_parse_cron_invalid_hour() {
        let r = parse_cron("30 9 * * *");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("hour-specific"));
    }

    #[test]
    fn test_parse_cron_wrong_fields() {
        let r = parse_cron("0 * * *");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_cron_invalid_step() {
        let r = parse_cron("*/0 * * * *");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_cron_out_of_range() {
        let r = parse_cron("60 * * * *");
        assert!(r.is_err());
    }

    #[test]
    fn test_next_cron_run_same_minute() {
        let expr = "30 * * * *";
        let base = 3600 * 10 + 30 * 60;
        let next = next_cron_run(expr, base);
        assert_eq!(next, 3600 * 11 + 30 * 60);
    }

    #[test]
    fn test_next_cron_run_wrap_hour() {
        let expr = "5 * * * *";
        let base = 3600 * 10 + 30 * 60;
        let next = next_cron_run(expr, base);
        assert_eq!(next, 3600 * 11 + 5 * 60);
    }

    #[test]
    fn test_scheduler_persist_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir for persist test should succeed");
        let path = dir.path().join("scheduler.json");
        let mut s = Scheduler::new();
        s.add_job(ScheduledJob {
            id: "persist-test".into(),
            name: "Persist".into(),
            schedule: ScheduleType::Interval(300),
            task_type: "evolve".into(),
            enabled: true,
            last_run: Some(1000),
            next_run: 2000,
        });
        s.save(&path).expect("Scheduler::save should write successfully");
        let loaded = Scheduler::load(&path).expect("Scheduler::load should read back successfully");
        assert_eq!(loaded.jobs.len(), 1);
        assert_eq!(loaded.jobs[0].id, "persist-test");
        assert_eq!(loaded.jobs[0].next_run, 2000);
    }
}
