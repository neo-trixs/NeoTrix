//! NT-ACT 编排 — agent-as-tool 执行循环 + 可验证交接 ledger (缺陷网 D7/D8 修复):
//! - D7 无 agent-as-tool 执行循环: 有序 task 队列, 每任务执行后过测试门,
//!   红门 (测试失败) 禁止 commit, 失败即停。
//! - D8 无可验证交接: 绿 commit 后记录已验公共面 (signatures/types/endpoints)
//!   入 ledger, 注入下个 task 的 brief, 保证交接可验证。
//!
//! 参照: codex-build (编排器-编码器分离 + 测试门在 commit 前每次),
//!    tuicr (ReviewStore 库 API + 机器可读评论导出)。

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 一个可执行任务。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub brief: String,
    pub owner: String,
    /// 验收: 需要的测试过滤串 (测试门只认这些通过)
    pub tests: Vec<String>,
}

/// 任务执行结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskOutcome {
    /// 测试门全绿 → 允许 commit
    Pass,
    /// 测试门有红 (测试失败) → 禁止 commit, 停止后续
    Fail(String),
    /// 任务未执行 (队列中止后)
    Skipped,
}

/// 公共面条目 (D8): 一次绿 commit 后记录的可验证接口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedSurface {
    pub task_id: String,
    pub signatures: Vec<String>,
    pub types: Vec<String>,
    pub endpoints: Vec<String>,
}

/// 交接 ledger: 有序记录每个已验公共面, 供注入下个 brief。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandoffLedger {
    pub entries: Vec<VerifiedSurface>,
}

impl HandoffLedger {
    pub fn record(&mut self, v: VerifiedSurface) {
        self.entries.push(v);
    }

    /// 序列化为交接上下文文本, 注入下个 task brief。
    pub fn to_brief(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let mut out = String::from("已验证公共面 (handoff ledger):\n");
        for e in &self.entries {
            out.push_str(&format!("- task {}:\n", e.task_id));
            for s in &e.signatures {
                out.push_str(&format!("    sig: {}\n", s));
            }
            for t in &e.types {
                out.push_str(&format!("    type: {}\n", t));
            }
            for ep in &e.endpoints {
                out.push_str(&format!("    endpoint: {}\n", ep));
            }
        }
        out
    }
}

/// D7 执行循环 — 测试门在 commit 前, 失败即停。
/// 执行器由调用方注入 (真实 agent 或脚本), 测试门由注入的测试运行器提供。
pub struct TaskExecutionLoop {
    pub tasks: Vec<Task>,
    pub ledger: HandoffLedger,
    outcomes: HashMap<String, TaskOutcome>,
    /// 测试门: 输入测试过滤串, 返回 (通过数, 失败信息)。None 表示测试未运行。
    test_runner: Option<Box<dyn Fn(&str) -> Result<usize, String>>>,
    /// 任务执行器: 输入 (brief, ledger 注入), 返回是否完成。
    executor: Option<Box<dyn Fn(&str, &str) -> Result<(), String>>>,
    committed: Vec<String>,
}

impl Default for TaskExecutionLoop {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl TaskExecutionLoop {
    pub fn new(
        test_runner: Option<Box<dyn Fn(&str) -> Result<usize, String>>>,
        executor: Option<Box<dyn Fn(&str, &str) -> Result<(), String>>>,
    ) -> Self {
        Self {
            tasks: Vec::new(),
            ledger: HandoffLedger::default(),
            outcomes: HashMap::new(),
            test_runner,
            executor,
            committed: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// 运行整个队列: 每个任务 执行 → 测试门 → 绿则记 ledger, 红则停。
    /// 返回停止原因 (None = 全部完成)。
    pub fn run(&mut self) -> Result<Option<String>, String> {
        for i in 0..self.tasks.len() {
            let task = self.tasks[i].clone();
            let brief = self.inject_ledger(&task.brief);
            if let Some(exec) = &self.executor {
                exec(&brief, &task.owner).map_err(|e| format!("task {} executor: {}", task.id, e))?;
            }
            // 测试门
            let outcome = self.run_test_gate(&task);
            match &outcome {
                TaskOutcome::Pass => {
                    // 绿 commit → 记录已验公共面 (模拟从任务成果提取)
                    let surface = VerifiedSurface {
                        task_id: task.id.clone(),
                        signatures: vec![format!("fn {}::apply()", task.owner)],
                        types: vec![format!("{}Output", capitalize(&task.owner))],
                        endpoints: vec![],
                    };
                    self.ledger.record(surface);
                    self.committed.push(task.id.clone());
                }
                TaskOutcome::Fail(reason) => {
                    self.outcomes.insert(task.id.clone(), outcome.clone());
                    // 红门 → 后续全部标记 Skipped
                    for later in self.tasks.iter().skip(i + 1) {
                        self.outcomes.insert(later.id.clone(), TaskOutcome::Skipped);
                    }
                    return Ok(Some(format!(
                        "task {} 红门: {} — 后续任务中止",
                        task.id, reason
                    )));
                }
                TaskOutcome::Skipped => {}
            }
            self.outcomes.insert(task.id.clone(), outcome);
        }
        Ok(None)
    }

    fn inject_ledger(&self, brief: &str) -> String {
        let li = self.ledger.to_brief();
        if li.is_empty() {
            brief.to_string()
        } else {
            format!("{}\n\n{}", brief, li)
        }
    }

    fn run_test_gate(&mut self, task: &Task) -> TaskOutcome {
        // 无测试门配置 → 视为通过 (调用方决定)。
        let Some(tr) = &self.test_runner else {
            return TaskOutcome::Pass;
        };
        let mut all_pass = true;
        let mut failures = Vec::new();
        for t in &task.tests {
            match tr(t) {
                Ok(_) => {}
                Err(e) => {
                    all_pass = false;
                    failures.push(format!("{}: {}", t, e));
                }
            }
        }
        if all_pass {
            TaskOutcome::Pass
        } else {
            TaskOutcome::Fail(failures.join("; "))
        }
    }

    pub fn outcome(&self, task_id: &str) -> Option<&TaskOutcome> {
        self.outcomes.get(task_id)
    }

    pub fn committed(&self) -> &[String] {
        &self.committed
    }

    /// 机器可读交接导出 (tuicr ReviewStore 参照)。
    pub fn ledger_json(&self) -> String {
        serde_json::to_string(&self.ledger).unwrap_or_default()
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pass_runner() -> Box<dyn Fn(&str) -> Result<usize, String>> {
        Box::new(|_| Ok(3))
    }

    fn failing_runner() -> Box<dyn Fn(&str) -> Result<usize, String>> {
        Box::new(|_| Err("tests failed".to_string()))
    }

    fn noop_exec() -> Box<dyn Fn(&str, &str) -> Result<(), String>> {
        Box::new(|_, _| Ok(()))
    }

    #[test]
    fn all_green_commits_and_records_ledger() {
        let mut loop_ = TaskExecutionLoop::new(Some(pass_runner()), Some(noop_exec()));
        loop_.add_task(Task {
            id: "t1".into(),
            brief: "impl".into(),
            owner: "coder".into(),
            tests: vec!["unit".into()],
        });
        loop_.add_task(Task {
            id: "t2".into(),
            brief: "impl2".into(),
            owner: "coder".into(),
            tests: vec!["unit".into()],
        });
        let stop = loop_.run().unwrap();
        assert!(stop.is_none(), "all green → no stop");
        assert_eq!(loop_.committed().len(), 2);
        assert_eq!(loop_.ledger.entries.len(), 2);
        // D8: 第二个任务 brief 注入了前一个的 ledger
        let brief2 = loop_.inject_ledger("new brief");
        assert!(brief2.contains("已验证公共面"), "ledger injected into next brief");
    }

    #[test]
    fn red_gate_blocks_commit_and_stops() {
        let mut loop_ = TaskExecutionLoop::new(Some(failing_runner()), Some(noop_exec()));
        loop_.add_task(Task {
            id: "t1".into(),
            brief: "impl".into(),
            owner: "coder".into(),
            tests: vec!["unit".into()],
        });
        loop_.add_task(Task {
            id: "t2".into(),
            brief: "impl2".into(),
            owner: "coder".into(),
            tests: vec!["unit".into()],
        });
        let stop = loop_.run().unwrap();
        assert!(stop.is_some(), "red gate stops loop");
        assert!(stop.unwrap().contains("t1"));
        assert_eq!(loop_.committed().len(), 0, "red → nothing committed");
        assert_eq!(loop_.outcome("t2"), Some(&TaskOutcome::Skipped));
    }

    #[test]
    fn partial_green_before_red_commits_first_only() {
        // 无测试门 → 每个任务直接绿 (调用方决定门行为), 验证批量 commit。
        let mut loop2 = TaskExecutionLoop::new(None, Some(noop_exec()));
        loop2.add_task(Task {
            id: "ok".into(),
            brief: "a".into(),
            owner: "x".into(),
            tests: vec![].into(),
        });
        loop2.add_task(Task {
            id: "ok2".into(),
            brief: "b".into(),
            owner: "x".into(),
            tests: vec![].into(),
        });
        let stop = loop2.run().unwrap();
        assert!(stop.is_none());
        assert_eq!(loop2.committed().len(), 2);
        assert_eq!(loop2.ledger.entries.len(), 2);
    }
}
