//! OSINT 自动巡检 — 基于 OsintSource trait 的自动化巡检框架
//!
//! 支持定时巡检、增量发现、异常告警。

use super::{OsintConfig, OsintTarget};
use serde::{Deserialize, Serialize};

/// 巡检任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatrolTask {
    pub target: OsintTarget,
    pub modules: Vec<String>,
    pub interval_secs: u64,
    pub last_run: Option<i64>,
}

/// 巡检结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatrolResult {
    pub task: PatrolTask,
    pub findings_count: usize,
    pub new_findings: usize,
    pub errors: Vec<String>,
    pub timestamp: i64,
}

/// 巡检调度器
pub struct PatrolScheduler {
    tasks: Vec<PatrolTask>,
    config: OsintConfig,
}

impl PatrolScheduler {
    pub fn new(config: OsintConfig) -> Self {
        Self { tasks: Vec::new(), config }
    }

    /// 添加巡检任务
    pub fn add_task(&mut self, task: PatrolTask) {
        self.tasks.push(task);
    }

    /// 执行所有到期任务
    pub async fn run_due_tasks(&self) -> Vec<PatrolResult> {
        let now = chrono::Utc::now().timestamp();
        let mut results = Vec::new();

        for task in &self.tasks {
            let should_run = task.last_run
                .map(|lr| now - lr >= task.interval_secs as i64)
                .unwrap_or(true);

            if should_run {
                let result = self.execute_task(task).await;
                results.push(result);
            }
        }

        results
    }

    async fn execute_task(&self, task: &PatrolTask) -> PatrolResult {
        let client = super::default_client();
        let mut findings_count = 0;
        let mut errors = Vec::new();

        for module_name in &task.modules {
            match module_name.as_str() {
                "dns" => {
                    match super::dns::investigate(&task.target, &client, &self.config).await {
                        Ok(f) => findings_count += f.subdomains.len(),
                        Err(e) => errors.push(format!("dns: {e}")),
                    }
                }
                "http" => {
                    match super::http::investigate(&task.target, &client, &self.config).await {
                        Ok(f) => findings_count += f.endpoints.len(),
                        Err(e) => errors.push(format!("http: {e}")),
                    }
                }
                "network" => {
                    match super::network::investigate(&task.target, &client, &self.config).await {
                        Ok(f) => findings_count += f.services.len(),
                        Err(e) => errors.push(format!("network: {e}")),
                    }
                }
                "vuln" => {
                    match super::vuln::investigate(&task.target, &client, &self.config).await {
                        Ok(f) => findings_count += f.vulnerabilities.len(),
                        Err(e) => errors.push(format!("vuln: {e}")),
                    }
                }
                _ => errors.push(format!("unknown module: {}", module_name)),
            }
        }

        PatrolResult {
            task: task.clone(),
            findings_count,
            new_findings: 0,
            errors,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patrol_task_creation() {
        let task = PatrolTask {
            target: OsintTarget { domain: Some("example.com".into()), ..Default::default() },
            modules: vec!["dns".into(), "http".into()],
            interval_secs: 3600,
            last_run: None,
        };
        assert_eq!(task.modules.len(), 2);
    }
}
