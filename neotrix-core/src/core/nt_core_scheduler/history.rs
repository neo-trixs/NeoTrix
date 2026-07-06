use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct JobRunRecord {
    pub job_id: String,
    pub started_at: u64,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_jobs: usize,
    pub enabled_jobs: usize,
    pub total_runs: u64,
    pub failed_runs: u64,
    pub success_rate: f64,
    pub registered_handler_count: usize,
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            enabled_jobs: 0,
            total_runs: 0,
            failed_runs: 0,
            success_rate: 1.0,
            registered_handler_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobRunHistory {
    records: VecDeque<JobRunRecord>,
    max_records: usize,
}

impl JobRunHistory {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: VecDeque::with_capacity(max_records.min(16)),
            max_records,
        }
    }

    pub fn push(&mut self, record: JobRunRecord) {
        if self.records.len() >= self.max_records {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }

    pub fn recent(&self, job_id: &str, limit: usize) -> Vec<&JobRunRecord> {
        self.records
            .iter()
            .rev()
            .filter(|r| r.job_id == job_id)
            .take(limit)
            .collect()
    }

    pub fn last_run(&self, job_id: &str) -> Option<&JobRunRecord> {
        self.records.iter().rev().find(|r| r.job_id == job_id)
    }

    pub fn failure_count(&self, job_id: &str, since: u64) -> usize {
        self.records
            .iter()
            .rev()
            .filter(|r| r.job_id == job_id && !r.success && r.started_at >= since)
            .count()
    }

    pub fn total_runs(&self) -> u64 {
        self.records.len() as u64
    }

    pub fn total_failures(&self) -> u64 {
        self.records.iter().filter(|r| !r.success).count() as u64
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.records.len();
        if total == 0 {
            return 1.0;
        }
        let successes = self.records.iter().filter(|r| r.success).count();
        successes as f64 / total as f64
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_empty() {
        let h = JobRunHistory::new(100);
        assert_eq!(h.total_runs(), 0);
        assert!((h.success_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_history_push_and_recent() {
        let mut h = JobRunHistory::new(100);
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 100,
            duration_ms: 50,
            success: true,
            error: None,
            retry_count: 0,
        });
        h.push(JobRunRecord {
            job_id: "b".into(),
            started_at: 200,
            duration_ms: 30,
            success: false,
            error: Some("err".into()),
            retry_count: 1,
        });
        assert_eq!(h.total_runs(), 2);
        assert_eq!(h.recent("a", 1).len(), 1);
        assert!(h.recent("a", 1)[0].success);
        assert_eq!(h.last_run("b").unwrap().error.as_deref(), Some("err"));
    }

    #[test]
    fn test_history_bounded() {
        let mut h = JobRunHistory::new(3);
        for i in 0..5 {
            h.push(JobRunRecord {
                job_id: "x".into(),
                started_at: i,
                duration_ms: 10,
                success: true,
                error: None,
                retry_count: 0,
            });
        }
        assert_eq!(h.total_runs(), 3);
    }

    #[test]
    fn test_history_success_rate() {
        let mut h = JobRunHistory::new(10);
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 0,
            duration_ms: 10,
            success: true,
            error: None,
            retry_count: 0,
        });
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 1,
            duration_ms: 10,
            success: false,
            error: Some("fail".into()),
            retry_count: 0,
        });
        assert!((h.success_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_history_failure_count() {
        let mut h = JobRunHistory::new(10);
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 100,
            duration_ms: 10,
            success: false,
            error: Some("e".into()),
            retry_count: 1,
        });
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 200,
            duration_ms: 10,
            success: true,
            error: None,
            retry_count: 0,
        });
        assert_eq!(h.failure_count("a", 150), 0);
        assert_eq!(h.failure_count("a", 50), 1);
    }

    #[test]
    fn test_history_clear() {
        let mut h = JobRunHistory::new(10);
        h.push(JobRunRecord {
            job_id: "a".into(),
            started_at: 0,
            duration_ms: 10,
            success: true,
            error: None,
            retry_count: 0,
        });
        h.clear();
        assert_eq!(h.total_runs(), 0);
    }
}
