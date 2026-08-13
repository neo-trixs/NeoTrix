//! nt_dscode_delegate — dscode subagents.ts 四角色并行 delegate 原语 (吸收 B-1)
//!
//! 证据: `notes/absorption-dscode-1.md` 条目 16 + Q1 (`subagents.ts:36-339`)。
//! 覆盖: 四角色 (explorer/implementer/reviewer/tester) 常量与角色属性,
//! delegate 任务类型, 并发上限 4 的有界并行原语 (`map_limited`),
//! 以及 `DSCODE_SUBAGENT_DEPTH` 嵌套深度守卫。
//!
//! 隔离约束: 本模块独立自包含, 仅依赖 `std` + `serde`, 不引用任何
//! 并行会话占用的模块 (`nt_agent_orchestrator/*`, `nt_core_task_dispatcher`,
//! `nt_core_self/attention_head` 均被并行会话改动)。接线点因此全部被占用,
//! 本文件为待接线新模块, 接线路径与 BLOCKED 标注见
//! `notes/absorption-dscode-b1-design.md`。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// 并发上限: `mapLimited(params.tasks, 4, ...)` (subagents.ts:57)
pub const DSCODE_MAX_CONCURRENT: usize = 4;
/// 单次 delegate 请求的任务数上限: schema 限定最多 8 个 (subagents.ts:12-23)
pub const DSCODE_MAX_TASKS: usize = 8;
/// 深度守卫环境变量名 (subagents.ts:40, 172)
pub const DSCODE_SUBAGENT_DEPTH_ENV: &str = "DSCODE_SUBAGENT_DEPTH";
/// 允许的最大嵌套深度: `>= 1` 时拒绝再 delegate
pub const DSCODE_MAX_SUBAGENT_DEPTH: i32 = 1;

/// 四角色 delegate 角色 (subagents.ts:12-23)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelegateRole {
    /// 只读调查: 语义/结构探索, low thinking, 宿主 cwd
    Explorer,
    /// 独立 worktree 生成改动, 产物为候选 diff
    Implementer,
    /// 只读审查: 对实现产物做证据驱动审查
    Reviewer,
    /// 测试诊断: 跑检查器/测试定位故障
    Tester,
}

impl DelegateRole {
    /// 全部四角色 (保持 schema 顺序)
    pub const ALL: [DelegateRole; 4] = [
        DelegateRole::Explorer,
        DelegateRole::Implementer,
        DelegateRole::Reviewer,
        DelegateRole::Tester,
    ];

    /// schema 字面量 (subagents.ts:13-18)
    pub fn label(&self) -> &'static str {
        match self {
            DelegateRole::Explorer => "explorer",
            DelegateRole::Implementer => "implementer",
            DelegateRole::Reviewer => "reviewer",
            DelegateRole::Tester => "tester",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "explorer" => Some(DelegateRole::Explorer),
            "implementer" => Some(DelegateRole::Implementer),
            "reviewer" => Some(DelegateRole::Reviewer),
            "tester" => Some(DelegateRole::Tester),
            _ => None,
        }
    }

    /// 只读角色: explorer/reviewer (subagents.ts:160-163)
    pub fn is_read_only(&self) -> bool {
        matches!(self, DelegateRole::Explorer | DelegateRole::Reviewer)
    }

    /// implementer 独享 git worktree 隔离 (subagents.ts:129-132)
    pub fn requires_worktree(&self) -> bool {
        matches!(self, DelegateRole::Implementer)
    }

    /// permission 模式: 只读角色 `plan`, 写角色 `auto` (subagents.ts:160-163)
    pub fn permission_mode(&self) -> &'static str {
        if self.is_read_only() {
            "plan"
        } else {
            "auto"
        }
    }

    /// 沙箱模式: 只读角色 `read-only`, 写角色 `workspace-write` (subagents.ts:160-163)
    pub fn sandbox_mode(&self) -> &'static str {
        if self.is_read_only() {
            "read-only"
        } else {
            "workspace-write"
        }
    }

    /// thinking 分级: explorer=low, 其余=max (subagents.ts:165)
    pub fn thinking(&self) -> &'static str {
        match self {
            DelegateRole::Explorer => "low",
            _ => "max",
        }
    }

    /// 注意力域提示 (映射到 nt_core_self AttentionDomain, 以字符串隔离并行占用类型):
    /// explorer→semantic, implementer→code, reviewer→self_reflection, tester→tool_use
    pub fn attention_domain(&self) -> &'static str {
        match self {
            DelegateRole::Explorer => "semantic",
            DelegateRole::Implementer => "code",
            DelegateRole::Reviewer => "self_reflection",
            DelegateRole::Tester => "tool_use",
        }
    }

    /// 供 `DelegateRequest` 校验的角色出现次数映射 (value=该角色被分配的任务数)
    pub fn role_counts(tasks: &[DelegateTask]) -> HashMap<DelegateRole, usize> {
        let mut counts: HashMap<DelegateRole, usize> = HashMap::new();
        for t in tasks {
            *counts.entry(t.role).or_insert(0) += 1;
        }
        counts
    }
}

/// delegate 单任务: 角色 + 任务描述 (subagents.ts:12-23 `agentTaskSchema`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegateTask {
    pub role: DelegateRole,
    pub task: String,
}

/// delegate 请求: 任务列表 + 宿主工作区上下文
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegateRequest {
    pub tasks: Vec<DelegateTask>,
    /// 宿主 cwd (只读角色运行于此)
    pub cwd: String,
    /// git 仓库根 (implementer worktree 隔离基准)
    pub git_root: Option<String>,
}

/// implementer 产物: `git add -N -- .` + `git diff --binary --no-ext-diff` (subagents.ts:176-188)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDiff {
    /// worktree 路径 (已析构/保留由接线层决定)
    pub worktree_path: Option<String>,
    /// `git add -N` 登记的待并入文件路径
    pub intent_to_add: Vec<String>,
    /// `git diff --binary --no-ext-diff` 候选补丁正文
    pub diff: String,
}

/// delegate 执行结果 (映射 dscode `runSubagent` 返回 + NeoTrix `SubTaskResult` 形状)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegateResult {
    pub role: DelegateRole,
    pub task: String,
    pub success: bool,
    pub output: String,
    /// implementer 独有: 候选 diff
    pub candidate_diff: Option<CandidateDiff>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl DelegateResult {
    pub fn failed(role: DelegateRole, task: String, error: impl Into<String>) -> Self {
        Self {
            role,
            task,
            success: false,
            output: String::new(),
            candidate_diff: None,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    pub fn success_rate(results: &[DelegateResult]) -> f64 {
        if results.is_empty() {
            0.0
        } else {
            results.iter().filter(|r| r.success).count() as f64 / results.len() as f64
        }
    }
}

/// 深度守卫: 读取 `DSCODE_SUBAGENT_DEPTH` (默认 0)
pub fn subagent_depth() -> i32 {
    std::env::var(DSCODE_SUBAGENT_DEPTH_ENV)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

/// 已达到最大嵌套深度: 子 agent 禁止再 delegate (subagents.ts:40, 172)
pub fn is_nested_delegate() -> bool {
    subagent_depth() >= DSCODE_MAX_SUBAGENT_DEPTH
}

/// delegate 是否可用 (深度守卫 + 环境显式门控)
pub fn can_delegate() -> bool {
    if is_nested_delegate() {
        return false;
    }
    std::env::var("DSCODE_DELEGATE_DISABLED")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true)
}

/// 校验请求: 任务非空、≤ `DSCODE_MAX_TASKS`、角色字面量合法
pub fn validate_request(request: &DelegateRequest) -> Result<(), String> {
    if request.tasks.is_empty() {
        return Err("delegate: empty task list".into());
    }
    if request.tasks.len() > DSCODE_MAX_TASKS {
        return Err(format!(
            "delegate: too many tasks ({} > {})",
            request.tasks.len(),
            DSCODE_MAX_TASKS
        ));
    }
    if request.cwd.trim().is_empty() {
        return Err("delegate: cwd is required".into());
    }
    if request.tasks.iter().any(|t| t.task.trim().is_empty()) {
        return Err("delegate: task description cannot be empty".into());
    }
    Ok(())
}

/// 有界并行原语 — dscode `mapLimited` 语义 (std-only 线程池):
/// 并发上限 `limit`, 按输入顺序收集结果; worker panic 被捕获为单条 Err。
///
/// 实现: 共享索引队列 `Mutex<Vec<(usize, T)>>`, 固定 `limit` 个 worker 弹取,
/// 结果按输入序号写入共享槽位, 顺序保真。thread::spawn 生命周期用 `Arc<F>` 共享。
type DelegateOutcome<R> = Option<Result<R, String>>;

pub fn map_limited<T, R, F>(items: Vec<T>, limit: usize, f: F) -> Vec<Result<R, String>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
{
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let limit = limit.max(1);
    let queue: Arc<Mutex<Vec<(usize, T)>>> =
        Arc::new(Mutex::new(items.into_iter().enumerate().rev().collect()));
    let results: Arc<Mutex<Vec<DelegateOutcome<R>>>> =
        Arc::new(Mutex::new((0..n).map(|_| None).collect()));
    let poisoned = Arc::new(AtomicBool::new(false));

    let workers = limit.min(n);
    let mut handles = Vec::with_capacity(workers);
    let f = Arc::new(f);
    for _ in 0..workers {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        let poisoned = Arc::clone(&poisoned);
        let f = Arc::clone(&f);
        handles.push(std::thread::spawn(move || {
            loop {
                let job = {
                    let mut q = queue.lock().map_err(|_| ()).ok();
                    match q {
                        Some(ref mut guard) => guard.pop(),
                        None => {
                            poisoned.store(true, Ordering::SeqCst);
                            return;
                        }
                    }
                };
                let Some((idx, item)) = job else { break };
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(item)))
                    .map_err(|_| "delegate: worker panicked while executing task".to_string());                let mut guard = match results.lock() {
                    Ok(g) => g,
                    Err(_) => {
                        poisoned.store(true, Ordering::SeqCst);
                        return;
                    }
                };
                guard[idx] = Some(outcome);
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }

    let mut out: Vec<Result<R, String>> = Vec::with_capacity(n);
    match results.lock() {
        Ok(mut guard) => {
            for slot in guard.iter_mut().take(n) {
                out.push(slot.take().unwrap_or_else(|| {
                    if poisoned.load(Ordering::SeqCst) {
                        Err("delegate: worker aborted (mutex poisoned)".to_string())
                    } else {
                        Err("delegate: worker exited without a result".to_string())
                    }
                }));
            }
        }
        Err(_) => {
            out.extend((0..n).map(|_| Err("delegate: results mutex poisoned".to_string())));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(role: DelegateRole) -> DelegateTask {
        DelegateTask {
            role,
            task: format!("do {}", role.label()),
        }
    }

    #[test]
    fn test_role_labels_roundtrip() {
        for role in DelegateRole::ALL {
            assert_eq!(DelegateRole::from_label(role.label()), Some(role));
        }
        assert_eq!(DelegateRole::from_label("oracle"), None);
    }

    #[test]
    fn test_role_profiles() {
        assert!(DelegateRole::Explorer.is_read_only());
        assert!(DelegateRole::Reviewer.is_read_only());
        assert!(!DelegateRole::Implementer.is_read_only());
        assert!(!DelegateRole::Tester.is_read_only());

        assert!(DelegateRole::Implementer.requires_worktree());
        assert!(!DelegateRole::Explorer.requires_worktree());

        assert_eq!(DelegateRole::Explorer.permission_mode(), "plan");
        assert_eq!(DelegateRole::Implementer.permission_mode(), "auto");
        assert_eq!(DelegateRole::Explorer.sandbox_mode(), "read-only");
        assert_eq!(DelegateRole::Implementer.sandbox_mode(), "workspace-write");
        assert_eq!(DelegateRole::Explorer.thinking(), "low");
        assert_eq!(DelegateRole::Reviewer.thinking(), "max");
        assert_eq!(DelegateRole::Implementer.attention_domain(), "code");
        assert_eq!(DelegateRole::Tester.attention_domain(), "tool_use");
    }

    #[test]
    fn test_validate_request() {
        let good = DelegateRequest {
            tasks: vec![task(DelegateRole::Explorer)],
            cwd: "/repo".into(),
            git_root: Some("/repo".into()),
        };
        assert!(validate_request(&good).is_ok());

        let empty = DelegateRequest {
            tasks: vec![],
            cwd: "/repo".into(),
            git_root: None,
        };
        assert!(validate_request(&empty).is_err());

        let no_cwd = DelegateRequest {
            tasks: vec![task(DelegateRole::Tester)],
            cwd: "".into(),
            git_root: None,
        };
        assert!(validate_request(&no_cwd).is_err());

        let too_many = DelegateRequest {
            tasks: (0..=DSCODE_MAX_TASKS).map(|i| task(if i % 2 == 0 { DelegateRole::Explorer } else { DelegateRole::Tester })).collect(),
            cwd: "/repo".into(),
            git_root: None,
        };
        assert!(validate_request(&too_many).is_err());
    }

    #[test]
    fn test_role_counts() {
        let tasks = vec![
            task(DelegateRole::Explorer),
            task(DelegateRole::Implementer),
            task(DelegateRole::Explorer),
        ];
        let counts = DelegateRole::role_counts(&tasks);
        assert_eq!(counts.get(&DelegateRole::Explorer), Some(&2));
        assert_eq!(counts.get(&DelegateRole::Implementer), Some(&1));
        assert_eq!(counts.get(&DelegateRole::Tester), None);
    }

    #[test]
    fn test_depth_guard_default() {
        assert_eq!(subagent_depth(), 0);
        assert!(!is_nested_delegate());
    }

    #[test]
    fn test_map_limited_preserves_order() {
        let items: Vec<i32> = (0..10).collect();
        let results = map_limited(items, 4, |i| i * i);
        assert_eq!(results.len(), 10);
        for (idx, r) in results.iter().enumerate() {
            assert_eq!(r.as_ref().expect("ok"), &(idx as i32 * idx as i32));
        }
    }

    #[test]
    fn test_map_limited_limit_zero_clamped() {
        let items: Vec<i32> = vec![1, 2, 3];
        let results = map_limited(items, 0, |i| i + 1);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(Result::is_ok));
    }

    #[test]
    fn test_map_limited_empty() {
        let results: Vec<Result<i32, String>> = map_limited(Vec::new(), 4, |i: i32| i);
        assert!(results.is_empty());
    }

    #[test]
    fn test_map_limited_catches_panic() {
        let items: Vec<i32> = (0..6).collect();
        let results = map_limited(items, 2, |i| {
            if i == 3 {
                panic!("boom");
            }
            i
        });
        assert_eq!(results.len(), 6);
        assert!(results[0].is_ok());
        assert!(results[3].is_err());
        assert!(results[5].is_ok());
    }

    #[test]
    fn test_success_rate() {
        let results = vec![
            DelegateResult::failed(DelegateRole::Tester, "t".into(), "x"),
            DelegateResult {
                role: DelegateRole::Explorer,
                task: "e".into(),
                success: true,
                output: "ok".into(),
                candidate_diff: None,
                error: None,
                duration_ms: 5,
            },
        ];
        assert_eq!(DelegateResult::success_rate(&results), 0.5);
        assert_eq!(DelegateResult::success_rate(&[]), 0.0);
    }
}
