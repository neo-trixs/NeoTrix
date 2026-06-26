#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

// ============================================================================
// DelegationStatus — 子任务生命周期状态
// ============================================================================

/// 委托子任务的当前状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegationStatus {
    /// 创建后待调度
    Pending,
    /// 正在执行中
    InProgress,
    /// 执行完成
    Completed,
    /// 执行失败
    Failed,
    /// 被上层取消
    Cancelled,
    /// 超时停滞，等待回收
    Stuck,
}

impl DelegationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Stuck => "stuck",
        }
    }
}

// ============================================================================
// DelegationStrategy — 任务分派策略
// ============================================================================

/// 委托树遍历与分派策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegationStrategy {
    /// 广度优先：逐层展开，适合探索阶段
    BFS,
    /// 深度优先：一条路径到底再回溯，适合已知模式
    DFS,
    /// 自适应：探索时 BFS，遇到已知模式时 DFS
    Adaptive,
}

impl DelegationStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BFS => "bfs",
            Self::DFS => "dfs",
            Self::Adaptive => "adaptive",
        }
    }
}

// ============================================================================
// DelegateConfig — 递归委托全局配置
// ============================================================================

/// 递归委托引擎的参数配置
#[derive(Debug, Clone)]
pub struct DelegateConfig {
    /// 最大递归深度（默认 5）
    pub max_depth: u32,
    /// 每个节点的最大子节点数（默认 10）
    pub max_children_per_node: u32,
    /// 系统容纳的最大任务总数（默认 500）
    pub max_total_tasks: usize,
    /// 任务被视为超时停滞的 cycle 数（默认 50）
    pub stuck_timeout_cycles: u64,
    /// 任务分派策略
    pub delegation_strategy: DelegationStrategy,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_children_per_node: 10,
            max_total_tasks: 500,
            stuck_timeout_cycles: 50,
            delegation_strategy: DelegationStrategy::Adaptive,
        }
    }
}

// ============================================================================
// DelegationTask — 单个委托任务
// ============================================================================

/// 一个可递归委托的子任务
#[derive(Debug, Clone)]
pub struct DelegationTask {
    /// 全局唯一任务 ID
    pub id: u64,
    /// 父任务 ID（None 表示根任务）
    pub parent_id: Option<u64>,
    /// 任务描述
    pub description: String,
    /// 上下文信息
    pub context: String,
    /// 当前递归深度
    pub depth: u32,
    /// 任务状态
    pub status: DelegationStatus,
    /// 执行结果
    pub result: Option<String>,
    /// 创建时的 cycle 编号
    pub created_at: u64,
    /// 完成时的 cycle 编号
    pub completed_at: Option<u64>,
    /// 被分配给的子 agent 标识
    pub assigned_to: String,
}

// ============================================================================
// DelegationNode — 委托树节点
// ============================================================================

/// 委托树中的一个节点，包含任务本身及其子节点引用
#[derive(Debug, Clone)]
pub struct DelegationNode {
    /// 承载的任务
    pub task: DelegationTask,
    /// 子节点 ID 列表
    pub children: Vec<u64>,
    /// 该节点允许的最大深度（从根开始）
    pub max_depth: u32,
}

// ============================================================================
// DelegationSummary — 委托树统计摘要
// ============================================================================

/// 委托管理器的运行统计快照
#[derive(Debug, Clone)]
pub struct DelegationSummary {
    pub total_tasks: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub stuck: usize,
    pub avg_depth: f64,
    pub max_depth_reached: u32,
    pub completion_rate: f64,
}

// ============================================================================
// RecursiveDelegationManager — 递归委托管理器
// ============================================================================

/// RAO 风格的递归委托管理器。
///
/// 跟踪委托树、确保有界递归深度、收集执行结果。
/// 支持 BFS/DFS/Adaptive 三种分派策略，自动检测超时停滞任务。
pub struct RecursiveDelegationManager {
    /// 全量任务映射：task_id → DelegationNode
    tasks: HashMap<u64, DelegationNode>,
    /// 运行时配置
    config: DelegateConfig,
    /// 自增 ID 计数器
    next_id: u64,
    /// 执行顺序队列（按策略决定入队顺序）
    execution_order: VecDeque<u64>,
    /// 停滞追踪器：task_id → 状态未变的 cycle 数
    stuck_tracker: HashMap<u64, u64>,
}

impl RecursiveDelegationManager {
    /// 使用默认配置创建一个新管理器
    pub fn new(config: DelegateConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            config,
            next_id: 1,
            execution_order: VecDeque::new(),
            stuck_tracker: HashMap::new(),
        }
    }

    /// 委托一个新任务。
    ///
    /// `parent_id` 为 `None` 时创建根任务。
    /// 返回新任务 ID，或错误原因（深度超限、子女数超限、总任务超限）。
    pub fn delegate(
        &mut self,
        parent_id: Option<u64>,
        description: String,
        context: String,
        depth: u32,
    ) -> Result<u64, String> {
        // 校验递归深度
        if depth > self.config.max_depth {
            return Err(format!(
                "delegation depth {} exceeds max depth {}",
                depth, self.config.max_depth
            ));
        }

        // 校验父节点存在性及子女数
        if let Some(pid) = parent_id {
            let parent = self.tasks.get(&pid).ok_or_else(|| {
                format!("parent task {} not found", pid)
            })?;
            if parent.children.len() as u32 >= self.config.max_children_per_node {
                return Err(format!(
                    "parent task {} already has {} children (max {})",
                    pid,
                    parent.children.len(),
                    self.config.max_children_per_node
                ));
            }
        }

        // 总任务数上限检测与自动清理
        if self.tasks.len() >= self.config.max_total_tasks {
            self.prune();
            if self.tasks.len() >= self.config.max_total_tasks {
                return Err(format!(
                    "total tasks {} exceeds max {} after prune",
                    self.tasks.len(),
                    self.config.max_total_tasks
                ));
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let assigned_to = format!("sub_agent:{}", id);

        let task = DelegationTask {
            id,
            parent_id,
            description,
            context,
            depth,
            status: DelegationStatus::Pending,
            result: None,
            created_at: 0,
            completed_at: None,
            assigned_to,
        };

        let node = DelegationNode {
            task,
            children: Vec::new(),
            max_depth: self.config.max_depth,
        };

        self.tasks.insert(id, node);

        // 链接到父节点
        if let Some(pid) = parent_id {
            if let Some(parent) = self.tasks.get_mut(&pid) {
                parent.children.push(id);
            }
        }

        // 按策略入队
        match self.config.delegation_strategy {
            DelegationStrategy::BFS => {
                self.execution_order.push_back(id);
            }
            DelegationStrategy::DFS => {
                self.execution_order.push_front(id);
            }
            DelegationStrategy::Adaptive => {
                // 深度 ≤ 2 时 BFS（探索），深度 > 2 时 DFS（深耕）
                if depth <= 2 {
                    self.execution_order.push_back(id);
                } else {
                    self.execution_order.push_front(id);
                }
            }
        }

        Ok(id)
    }

    /// 获取任务的不可变引用
    pub fn get_task(&self, id: u64) -> Option<&DelegationNode> {
        self.tasks.get(&id)
    }

    /// 获取任务的可变引用
    pub fn get_task_mut(&mut self, id: u64) -> Option<&mut DelegationNode> {
        self.tasks.get_mut(&id)
    }

    /// 更新任务状态。终端状态会自动设置 `completed_at` 并清除停滞追踪。
    /// 返回 `false` 表示任务 ID 不存在。
    pub fn update_status(&mut self, task_id: u64, status: DelegationStatus) -> bool {
        let node = match self.tasks.get_mut(&task_id) {
            Some(n) => n,
            None => return false,
        };

        node.task.status = status;

        if status.is_terminal() {
            node.task.completed_at = Some(0); // 由调用方或 tick 填充真实值
            self.stuck_tracker.remove(&task_id);
        }

        true
    }

    /// 记录任务结果并将状态设为 Completed。
    /// 返回 `false` 表示任务 ID 不存在。
    pub fn record_result(&mut self, task_id: u64, result: String) -> bool {
        let node = match self.tasks.get_mut(&task_id) {
            Some(n) => n,
            None => return false,
        };

        node.task.result = Some(result);
        node.task.status = DelegationStatus::Completed;
        node.task.completed_at = Some(0);
        self.stuck_tracker.remove(&task_id);

        true
    }

    /// 从执行队列中取出下一个 Pending 状态的任务 ID。
    pub fn next_ready_task(&self) -> Option<u64> {
        for &id in &self.execution_order {
            if let Some(node) = self.tasks.get(&id) {
                if node.task.status == DelegationStatus::Pending {
                    return Some(id);
                }
            }
        }
        None
    }

    /// 每个 cycle 调用一次：检测超时停滞、按需清理已完成任务。
    ///
    /// `cycle` 是当前的 cycle 编号，用于判断停滞超时和记录时间戳。
    pub fn tick(&mut self, cycle: u64) {
        // 1. 检测 InProgress 任务是否超时
        let stuck_ids: Vec<u64> = self
            .tasks
            .iter()
            .filter_map(|(&id, node)| {
                if node.task.status != DelegationStatus::InProgress {
                    return None;
                }
                let counter = self.stuck_tracker.entry(id).or_insert(0);
                if *counter >= self.config.stuck_timeout_cycles {
                    Some(id)
                } else {
                    *counter += 1;
                    None
                }
            })
            .collect();

        for id in stuck_ids {
            if let Some(node) = self.tasks.get_mut(&id) {
                node.task.status = DelegationStatus::Stuck;
                node.task.completed_at = Some(cycle);
                self.stuck_tracker.remove(&id);
            }
        }

        // 2. 为已完成的任务记录时间戳
        for node in self.tasks.values_mut() {
            if node.task.completed_at == Some(0) {
                node.task.completed_at = Some(cycle);
            }
            if node.task.created_at == 0 {
                node.task.created_at = cycle;
            }
        }

        // 3. 任务总数超限时自动清理
        if self.tasks.len() > self.config.max_total_tasks {
            self.prune();
        }
    }

    /// 递归生成任务及其所有后代的摘要字符串。
    pub fn subtree_summary(&self, task_id: u64) -> String {
        let mut lines = Vec::new();
        self.build_subtree_summary(task_id, 0, &mut lines);
        lines.join("\n")
    }

    /// 递归辅助：构建子树摘要
    fn build_subtree_summary(
        &self,
        task_id: u64,
        indent: usize,
        lines: &mut Vec<String>,
    ) {
        let node = match self.tasks.get(&task_id) {
            Some(n) => n,
            None => {
                lines.push(format!("{}[task {} not found]", "  ".repeat(indent), task_id));
                return;
            }
        };

        let prefix = "  ".repeat(indent);
        let t = &node.task;
        let status_name = t.status.name();
        let result_preview = t
            .result
            .as_ref()
            .map(|r| {
                if r.len() > 40 {
                    format!("{:.40}...", r)
                } else {
                    r.clone()
                }
            })
            .unwrap_or_default();

        lines.push(format!(
            "{}#{} depth={} [{}] {} {}",
            prefix, t.id, t.depth, status_name, t.description, result_preview
        ));

        for child_id in &node.children {
            self.build_subtree_summary(*child_id, indent + 1, lines);
        }
    }

    /// 从指定任务追溯到根任务的完整路径。
    pub fn path_to_root(&self, task_id: u64) -> Vec<u64> {
        let mut path = Vec::new();
        let mut current = Some(task_id);

        while let Some(id) = current {
            path.push(id);
            current = self.tasks.get(&id).and_then(|n| n.task.parent_id);
        }

        path
    }

    /// 聚合全量任务统计摘要。
    pub fn summary(&self) -> DelegationSummary {
        let total_tasks = self.tasks.len();
        let mut pending = 0;
        let mut in_progress = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut cancelled = 0;
        let mut stuck = 0;
        let mut total_depth: u64 = 0;
        let mut max_depth_reached: u32 = 0;

        for node in self.tasks.values() {
            match node.task.status {
                DelegationStatus::Pending => pending += 1,
                DelegationStatus::InProgress => in_progress += 1,
                DelegationStatus::Completed => completed += 1,
                DelegationStatus::Failed => failed += 1,
                DelegationStatus::Cancelled => cancelled += 1,
                DelegationStatus::Stuck => stuck += 1,
            }
            total_depth += node.task.depth as u64;
            if node.task.depth > max_depth_reached {
                max_depth_reached = node.task.depth;
            }
        }

        let finished = completed + failed;
        let avg_depth = if total_tasks > 0 {
            total_depth as f64 / total_tasks as f64
        } else {
            0.0
        };
        let completion_rate = if total_tasks > 0 {
            finished as f64 / total_tasks as f64
        } else {
            0.0
        };

        DelegationSummary {
            total_tasks,
            pending,
            in_progress,
            completed,
            failed,
            cancelled,
            stuck,
            avg_depth,
            max_depth_reached,
            completion_rate,
        }
    }

    /// 清理已完成的叶子节点，释放任务槽位。
    ///
    /// 保留根节点和所有 InProgress 节点。
    /// 从叶子开始删除：先收集叶子节点的 completed/failed，迭代删除。
    pub fn prune(&mut self) {
        loop {
            let mut removed = false;

            // 找出可删除的节点：终端状态 + 无子节点正在运行
            let prune_ids: Vec<u64> = self
                .tasks
                .iter()
                .filter_map(|(&id, node)| {
                    if !node.task.status.is_terminal() {
                        return None;
                    }
                    // 检查所有子孙是否都已终止
                    if !self.all_descendants_terminal(id) {
                        return None;
                    }
                    Some(id)
                })
                .collect();

            for id in prune_ids {
                // 从父节点 children 列表中移除
                if let Some(node) = self.tasks.get(&id) {
                    if let Some(pid) = node.task.parent_id {
                        if let Some(parent) = self.tasks.get_mut(&pid) {
                            parent.children.retain(|&c| c != id);
                        }
                    }
                }
                self.tasks.remove(&id);
                self.execution_order.retain(|&e| e != id);
                self.stuck_tracker.remove(&id);
                removed = true;
            }

            if !removed {
                break;
            }
        }
    }

    /// 递归检查某个任务的所有后代是否都处于终端状态。
    fn all_descendants_terminal(&self, task_id: u64) -> bool {
        let node = match self.tasks.get(&task_id) {
            Some(n) => n,
            None => return true,
        };

        // 如果自己有子节点，检查所有子节点及其后代
        for child_id in &node.children {
            let child = match self.tasks.get(child_id) {
                Some(c) => c,
                None => continue,
            };
            if !child.task.status.is_terminal() {
                return false;
            }
            if !self.all_descendants_terminal(*child_id) {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> RecursiveDelegationManager {
        RecursiveDelegationManager::new(DelegateConfig::default())
    }

    #[test]
    fn test_delegate_no_parent_creates_root() {
        let mut mgr = default_manager();
        let id = mgr.delegate(None, "root task".into(), "root context".into(), 0).unwrap();
        let node = mgr.get_task(id).unwrap();
        assert_eq!(node.task.parent_id, None);
        assert_eq!(node.task.description, "root task");
        assert_eq!(node.task.depth, 0);
        assert_eq!(node.task.status, DelegationStatus::Pending);
    }

    #[test]
    fn test_delegate_with_parent_creates_child() {
        let mut mgr = default_manager();
        let root_id = mgr.delegate(None, "root".into(), "ctx".into(), 0).unwrap();
        let child_id = mgr.delegate(Some(root_id), "child".into(), "ctx2".into(), 1).unwrap();
        let child = mgr.get_task(child_id).unwrap();
        assert_eq!(child.task.parent_id, Some(root_id));
        assert_eq!(child.task.depth, 1);
        let parent = mgr.get_task(root_id).unwrap();
        assert!(parent.children.contains(&child_id));
    }

    #[test]
    fn test_depth_limit_returns_error() {
        let config = DelegateConfig {
            max_depth: 3,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let root_id = mgr.delegate(None, "root".into(), "ctx".into(), 0).unwrap();
        let r1 = mgr.delegate(Some(root_id), "l1".into(), "ctx".into(), 1);
        assert!(r1.is_ok());
        let r2 = mgr.delegate(Some(root_id), "l2".into(), "ctx".into(), 2);
        assert!(r2.is_ok());
        let r3 = mgr.delegate(Some(root_id), "l3".into(), "ctx".into(), 3);
        assert!(r3.is_ok());
        let r4 = mgr.delegate(Some(root_id), "l4".into(), "ctx".into(), 4);
        assert!(r4.is_err());
        assert!(r4.unwrap_err().contains("exceeds max depth"));
    }

    #[test]
    fn test_max_children_limit_returns_error() {
        let config = DelegateConfig {
            max_children_per_node: 2,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let root_id = mgr.delegate(None, "root".into(), "ctx".into(), 0).unwrap();
        let r1 = mgr.delegate(Some(root_id), "c1".into(), "ctx".into(), 1);
        assert!(r1.is_ok());
        let r2 = mgr.delegate(Some(root_id), "c2".into(), "ctx".into(), 1);
        assert!(r2.is_ok());
        let r3 = mgr.delegate(Some(root_id), "c3".into(), "ctx".into(), 1);
        assert!(r3.is_err());
        assert!(r3.unwrap_err().contains("already has 2 children"));
    }

    #[test]
    fn test_update_status_changes_status() {
        let mut mgr = default_manager();
        let id = mgr.delegate(None, "t".into(), "ctx".into(), 0).unwrap();
        assert!(mgr.update_status(id, DelegationStatus::InProgress));
        assert_eq!(mgr.get_task(id).unwrap().task.status, DelegationStatus::InProgress);
        assert!(mgr.update_status(id, DelegationStatus::Completed));
        assert_eq!(mgr.get_task(id).unwrap().task.status, DelegationStatus::Completed);
        assert!(mgr.get_task(id).unwrap().task.completed_at.is_some());
    }

    #[test]
    fn test_update_status_nonexistent_returns_false() {
        let mut mgr = default_manager();
        assert!(!mgr.update_status(999, DelegationStatus::Completed));
    }

    #[test]
    fn test_record_result_completes_with_result() {
        let mut mgr = default_manager();
        let id = mgr.delegate(None, "t".into(), "ctx".into(), 0).unwrap();
        assert!(mgr.record_result(id, "success".into()));
        let node = mgr.get_task(id).unwrap();
        assert_eq!(node.task.status, DelegationStatus::Completed);
        assert_eq!(node.task.result.as_deref(), Some("success"));
    }

    #[test]
    fn test_next_ready_task_returns_pending_in_order() {
        let mut mgr = default_manager();
        let id1 = mgr.delegate(None, "a".into(), "ctx".into(), 0).unwrap();
        let id2 = mgr.delegate(Some(id1), "b".into(), "ctx".into(), 1).unwrap();
        let id3 = mgr.delegate(Some(id1), "c".into(), "ctx".into(), 1).unwrap();

        // Adaptive 策略下 depth≤2 为 BFS: id1, id2, id3
        assert_eq!(mgr.next_ready_task(), Some(id1));
        mgr.update_status(id1, DelegationStatus::InProgress);
        // id1 已是 InProgress，应该跳过
        assert_eq!(mgr.next_ready_task(), Some(id2));
    }

    #[test]
    fn test_next_ready_task_returns_none_if_all_done() {
        let mut mgr = default_manager();
        let id = mgr.delegate(None, "t".into(), "ctx".into(), 0).unwrap();
        mgr.update_status(id, DelegationStatus::InProgress);
        mgr.update_status(id, DelegationStatus::Completed);
        assert_eq!(mgr.next_ready_task(), None);
    }

    #[test]
    fn test_tick_detects_stuck_after_timeout() {
        let config = DelegateConfig {
            stuck_timeout_cycles: 3,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let id = mgr.delegate(None, "t".into(), "ctx".into(), 0).unwrap();
        mgr.update_status(id, DelegationStatus::InProgress);

        mgr.tick(1);
        mgr.tick(2);
        mgr.tick(3);
        assert_eq!(mgr.get_task(id).unwrap().task.status, DelegationStatus::InProgress);

        mgr.tick(4); // 第 4 次 tick，counter = 3，触发超时
        assert_eq!(mgr.get_task(id).unwrap().task.status, DelegationStatus::Stuck);
    }

    #[test]
    fn test_path_to_root_returns_correct_chain() {
        let mut mgr = default_manager();
        let root = mgr.delegate(None, "root".into(), "ctx".into(), 0).unwrap();
        let l1 = mgr.delegate(Some(root), "l1".into(), "ctx".into(), 1).unwrap();
        let l2 = mgr.delegate(Some(l1), "l2".into(), "ctx".into(), 2).unwrap();

        let path = mgr.path_to_root(l2);
        assert_eq!(path, vec![l2, l1, root]);
    }

    #[test]
    fn test_path_to_root_nonexistent_returns_singleton() {
        let mgr = default_manager();
        let path = mgr.path_to_root(42);
        assert_eq!(path, vec![42]);
    }

    #[test]
    fn test_subtree_summary_includes_children() {
        let mut mgr = default_manager();
        let root = mgr.delegate(None, "root task".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(root), "child 1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(root), "child 2".into(), "ctx".into(), 1).unwrap();

        mgr.record_result(c1, "done".into());

        let summary = mgr.subtree_summary(root);
        assert!(summary.contains("root task"));
        assert!(summary.contains("child 1"));
        assert!(summary.contains("child 2"));
        assert!(summary.contains("[completed]"));
        assert!(summary.contains("[pending]"));
    }

    #[test]
    fn test_summary_returns_correct_counts() {
        let mut mgr = default_manager();
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        mgr.record_result(c1, "ok".into());
        mgr.update_status(c2, DelegationStatus::Failed);

        let s = mgr.summary();
        assert_eq!(s.total_tasks, 3);
        assert_eq!(s.pending, 1); // root still pending
        assert_eq!(s.completed, 1);
        assert_eq!(s.failed, 1);
        assert!((s.completion_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_prune_removes_completed_leaves() {
        let config = DelegateConfig {
            max_total_tasks: 3,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        mgr.record_result(c1, "ok".into());
        mgr.record_result(c2, "ok".into());

        mgr.prune();

        // 叶子节点 c1/c2 应被删除，root 保留（因 root 也已完成… 但我们没标记 root）
        // 实际上 root 还是 Pending，所以不会被剪
        assert!(mgr.get_task(r).is_some());
        // 但子节点都已完成，应该被删除
        assert!(mgr.get_task(c1).is_none());
        assert!(mgr.get_task(c2).is_none());
    }

    #[test]
    fn test_prune_preserves_in_progress() {
        let mut mgr = default_manager();
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        mgr.update_status(c1, DelegationStatus::InProgress);
        mgr.record_result(c2, "ok".into());

        mgr.prune();

        // c1 是 InProgress → 保留
        assert!(mgr.get_task(c1).is_some());
        // c2 是 Completed → 删除
        assert!(mgr.get_task(c2).is_none());
    }

    #[test]
    fn test_max_tasks_triggers_prune_on_delegate() {
        let config = DelegateConfig {
            max_total_tasks: 3,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        // 现在满了
        mgr.record_result(c1, "ok".into());
        mgr.record_result(c2, "ok".into());

        // 第 4 次委托应该触发 prune 然后成功
        let d4 = mgr.delegate(Some(r), "d4".into(), "ctx".into(), 1);
        assert!(d4.is_ok());
        // root + d4 = 2 个（c1/c2 被 prune 掉了）
        assert_eq!(mgr.tasks.len(), 2);
    }

    #[test]
    fn test_tick_sets_created_at_and_completed_at() {
        let mut mgr = default_manager();
        let id = mgr.delegate(None, "t".into(), "ctx".into(), 0).unwrap();
        mgr.tick(42);
        assert_eq!(mgr.get_task(id).unwrap().task.created_at, 42);

        mgr.update_status(id, DelegationStatus::Completed);
        mgr.tick(99);
        assert_eq!(mgr.get_task(id).unwrap().task.completed_at, Some(99));
    }

    #[test]
    fn test_execution_order_bfs() {
        let config = DelegateConfig {
            delegation_strategy: DelegationStrategy::BFS,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        // BFS: r, c1, c2
        assert_eq!(mgr.next_ready_task(), Some(r));
        mgr.update_status(r, DelegationStatus::InProgress);
        assert_eq!(mgr.next_ready_task(), Some(c1));
        mgr.update_status(c1, DelegationStatus::InProgress);
        assert_eq!(mgr.next_ready_task(), Some(c2));
    }

    #[test]
    fn test_execution_order_dfs() {
        let config = DelegateConfig {
            delegation_strategy: DelegationStrategy::DFS,
            ..Default::default()
        };
        let mut mgr = RecursiveDelegationManager::new(config);
        let r = mgr.delegate(None, "r".into(), "ctx".into(), 0).unwrap();
        let c1 = mgr.delegate(Some(r), "c1".into(), "ctx".into(), 1).unwrap();
        let c2 = mgr.delegate(Some(r), "c2".into(), "ctx".into(), 1).unwrap();

        // DFS: c2, c1, r (push_front 顺序)
        assert_eq!(mgr.next_ready_task(), Some(c2));
        mgr.update_status(c2, DelegationStatus::InProgress);
        assert_eq!(mgr.next_ready_task(), Some(c1));
        mgr.update_status(c1, DelegationStatus::InProgress);
        assert_eq!(mgr.next_ready_task(), Some(r));
    }
}
