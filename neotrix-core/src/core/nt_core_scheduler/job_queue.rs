#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

fn now_secs() -> i64 {
    crate::core::nt_core_time::unix_now_secs() as i64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl JobPriority {
    pub fn rank(&self) -> u8 {
        match self {
            JobPriority::Low => 0,
            JobPriority::Medium => 1,
            JobPriority::High => 2,
            JobPriority::Critical => 3,
        }
    }
}

impl PartialOrd for JobPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.rank().cmp(&other.rank()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveJob {
    pub id: String,
    pub name: String,
    pub handler: String,
    pub priority: JobPriority,
    pub state: JobState,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_cycles: u64,
    pub cycles_running: u64,
    pub context: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub depends_on: Vec<String>,
    pub tags: Vec<String>,
    pub preemptible: bool,
}

impl CognitiveJob {
    pub fn new(name: &str, handler: &str, priority: JobPriority, context: &str) -> Self {
        let id = format!("job_{}_{}", now_secs(), rand_short());
        Self {
            id,
            name: name.to_string(),
            handler: handler.to_string(),
            priority,
            state: JobState::Pending,
            created_at: now_secs(),
            started_at: None,
            completed_at: None,
            max_retries: 3,
            retry_count: 0,
            timeout_cycles: 100,
            cycles_running: 0,
            context: context.to_string(),
            result: None,
            error: None,
            depends_on: Vec::new(),
            tags: Vec::new(),
            preemptible: true,
        }
    }

    pub fn with_dep(mut self, dep: &str) -> Self {
        self.depends_on.push(dep.to_string());
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_timeout(mut self, cycles: u64) -> Self {
        self.timeout_cycles = cycles;
        self
    }

    pub fn non_preemptible(mut self) -> Self {
        self.preemptible = false;
        self
    }
}

fn rand_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:04x}", n & 0xFFFF)
}

pub struct CognitiveJobQueue {
    jobs: Vec<CognitiveJob>,
    running_job: Option<String>,
    max_history: usize,
    paused_jobs: Vec<String>,
    next_id: u64,
}

impl CognitiveJobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            running_job: None,
            max_history: 200,
            paused_jobs: Vec::new(),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, job: CognitiveJob) -> String {
        let id = job.id.clone();
        self.jobs.push(job);
        id
    }

    pub fn dequeue_next(&mut self) -> Option<&mut CognitiveJob> {
        if self.running_job.is_some() {
            return None;
        }
        let idx = self.select_next()?;
        let job = &mut self.jobs[idx];
        job.state = JobState::Running;
        job.started_at = Some(now_secs());
        self.running_job = Some(job.id.clone());
        Some(job)
    }

    fn select_next(&self) -> Option<usize> {
        let mut best_idx = None;
        let mut best_priority = JobPriority::Low;
        let mut best_age = 0i64;
        for (i, job) in self.jobs.iter().enumerate() {
            if job.state != JobState::Pending {
                continue;
            }
            let deps_met = job.depends_on.iter().all(|dep_id| {
                self.jobs
                    .iter()
                    .any(|j| &j.id == dep_id && matches!(j.state, JobState::Completed))
            });
            if !deps_met {
                continue;
            }
            match best_idx {
                None => {
                    best_idx = Some(i);
                    best_priority = job.priority.clone();
                    best_age = now_secs() - job.created_at;
                }
                Some(_) => {
                    if job.priority > best_priority {
                        best_idx = Some(i);
                        best_priority = job.priority.clone();
                        best_age = now_secs() - job.created_at;
                    } else if job.priority == best_priority {
                        let age = now_secs() - job.created_at;
                        if age > best_age {
                            best_idx = Some(i);
                            best_age = age;
                        }
                    }
                }
            }
        }
        best_idx
    }

    pub fn try_preempt(&mut self, priority: JobPriority) -> bool {
        match &self.running_job {
            None => return true,
            Some(running_id) => {
                let running = self.jobs.iter().find(|j| &j.id == running_id.as_str());
                match running {
                    None => {
                        self.running_job = None;
                        return true;
                    }
                    Some(job) => {
                        if !job.preemptible {
                            return false;
                        }
                        if priority > job.priority {
                            if let Some(job) =
                                self.jobs.iter_mut().find(|j| &j.id == running_id.as_str())
                            {
                                job.state = JobState::Paused;
                                self.paused_jobs.push(job.id.clone());
                            }
                            self.running_job = None;
                            return true;
                        }
                        false
                    }
                }
            }
        }
    }

    pub fn complete_current(&mut self, result: &str) -> Option<String> {
        let id = self.running_job.take()?;
        let job = self.jobs.iter_mut().find(|j| j.id == id)?;
        job.state = JobState::Completed;
        job.completed_at = Some(now_secs());
        job.result = Some(result.to_string());
        Some(id)
    }

    pub fn fail_current(&mut self, error: &str) -> Option<String> {
        let id = self.running_job.take()?;
        let job = self.jobs.iter_mut().find(|j| j.id == id)?;
        if job.retry_count < job.max_retries {
            job.retry_count += 1;
            job.state = JobState::Pending;
            job.error = Some(format!("retry_{}:{}", job.retry_count, error));
            self.running_job = None;
        } else {
            job.state = JobState::Failed;
            job.completed_at = Some(now_secs());
            job.error = Some(error.to_string());
        }
        Some(id)
    }

    pub fn resume_paused(&mut self) -> Option<String> {
        let id = self.paused_jobs.pop()?;
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.state = JobState::Pending;
        }
        Some(id)
    }

    pub fn tick_cycle(&mut self) -> Vec<String> {
        let mut events = Vec::new();
        if let Some(ref running_id) = self.running_job.clone() {
            if let Some(job) = self.jobs.iter_mut().find(|j| &j.id == running_id.as_str()) {
                job.cycles_running += 1;
                if job.cycles_running >= job.timeout_cycles {
                    let r = self.fail_current("timeout");
                    if let Some(id) = r {
                        events.push(format!("timeout:{}", id));
                    }
                }
            }
        }
        if self.running_job.is_none() {
            if !self.paused_jobs.is_empty() {
                self.resume_paused();
            }
            if let Some(job) = self.dequeue_next() {
                events.push(format!("started:{}", job.id));
            }
        }
        events
    }

    pub fn cancel(&mut self, job_id: &str) -> bool {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
            job.state = JobState::Cancelled;
            if self.running_job.as_deref() == Some(job_id) {
                self.running_job = None;
            }
            true
        } else {
            false
        }
    }

    pub fn prune_history(&mut self) {
        let completed_count = self
            .jobs
            .iter()
            .filter(|j| {
                matches!(
                    j.state,
                    JobState::Completed | JobState::Failed | JobState::Cancelled
                )
            })
            .count();
        if completed_count > self.max_history {
            let to_remove = completed_count - self.max_history;
            self.jobs.retain(|j| {
                !matches!(
                    j.state,
                    JobState::Completed | JobState::Failed | JobState::Cancelled
                ) || {
                    let _ = to_remove;
                    true
                }
            });
            let mut removed = 0;
            self.jobs.retain(|j| {
                if matches!(
                    j.state,
                    JobState::Completed | JobState::Failed | JobState::Cancelled
                ) && removed < to_remove
                {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    pub fn stats(&self) -> String {
        let pending = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Pending)
            .count();
        let running = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Running)
            .count();
        let paused = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Paused)
            .count();
        let completed = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Completed)
            .count();
        let failed = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Failed)
            .count();
        let cancelled = self
            .jobs
            .iter()
            .filter(|j| j.state == JobState::Cancelled)
            .count();
        format!(
            "queue:P{}_R{}_P{}_C{}_F{}_X{}",
            pending, running, paused, completed, failed, cancelled
        )
    }

    pub fn pending_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| j.state == JobState::Pending)
            .count()
    }

    pub fn running_job_id(&self) -> Option<&str> {
        self.running_job.as_deref()
    }

    pub fn get_job(&self, id: &str) -> Option<&CognitiveJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    pub fn get_job_mut(&mut self, id: &str) -> Option<&mut CognitiveJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = CognitiveJobQueue::new();
        let job = CognitiveJob::new("test", "test_handler", JobPriority::Medium, "{}");
        let _id = queue.enqueue(job);
        let next = queue.dequeue_next();
        assert!(next.is_some());
        assert_eq!(next.unwrap().state, JobState::Running);
        assert!(queue.running_job_id().is_some());
    }

    #[test]
    fn test_priority_ordering() {
        let mut queue = CognitiveJobQueue::new();
        queue.enqueue(CognitiveJob::new("low", "h", JobPriority::Low, "{}"));
        queue.enqueue(CognitiveJob::new("high", "h", JobPriority::High, "{}"));
        let next = queue.dequeue_next().unwrap();
        assert_eq!(next.name, "high");
    }

    #[test]
    fn test_complete_and_fail() {
        let mut queue = CognitiveJobQueue::new();
        let id = queue.enqueue(CognitiveJob::new("test", "h", JobPriority::Medium, "{}"));
        queue.dequeue_next();
        let completed = queue.complete_current("ok");
        assert!(completed.is_some());
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.state, JobState::Completed);
    }

    #[test]
    fn test_preemption() {
        let mut queue = CognitiveJobQueue::new();
        queue.enqueue(CognitiveJob::new("low", "h", JobPriority::Low, "{}"));
        queue.dequeue_next();
        assert!(queue.try_preempt(JobPriority::High));
        let next = queue.dequeue_next().unwrap();
        assert_eq!(next.priority, JobPriority::High);
    }

    #[test]
    fn test_dependency() {
        let mut queue = CognitiveJobQueue::new();
        let dep = CognitiveJob::new("dep", "h", JobPriority::Medium, "{}");
        let dep_id = dep.id.clone();
        queue.enqueue(dep);
        let child = CognitiveJob::new("child", "h", JobPriority::High, "{}").with_dep(&dep_id);
        queue.enqueue(child);
        let first = queue.dequeue_next().unwrap();
        assert_eq!(first.name, "dep");
        queue.complete_current("ok");
        let second = queue.dequeue_next().unwrap();
        assert_eq!(second.name, "child");
    }

    #[test]
    fn test_stats() {
        let queue = CognitiveJobQueue::new();
        let s = queue.stats();
        assert!(s.contains("P0_R0_P0"));
    }

    #[test]
    fn test_tick_cycle() {
        let mut queue = CognitiveJobQueue::new();
        queue.enqueue(CognitiveJob::new("test", "h", JobPriority::Medium, "{}"));
        let events = queue.tick_cycle();
        assert!(events.len() >= 1);
        assert!(events[0].contains("started:"));
    }
}
