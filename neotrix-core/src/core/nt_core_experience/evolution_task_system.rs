use std::collections::{HashMap, VecDeque};

/// 进化任务类型 — 意识可以对自己做的改进
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    /// 将已存在的认知模块接入 ConsciousnessPipeline
    ModuleWiring,
    /// 创建新模块填补架构缺口
    ModuleCreate,
    /// 修复编译/类型/借用错误
    CompileFix,
    /// 重构现有代码
    RefactorExisting,
    /// 为现有代码补测试
    TestCreate,
    /// 蒸馏会话经验到 AGENTS.md
    ExperienceDistill,
    /// 设计文档创建
    Documentation,
    /// 架构评审
    ArchitectureReview,
    /// 性能优化
    Performance,
    /// 安全加固
    SecurityHardening,
    /// 从互联网项目吸收模式到架构设计
    AbsorbPattern {
        /// 来源仓库 URL
        repo_url: String,
        /// 吸收的模式名称
        pattern_name: String,
    },
    /// 将模块接入 ConsciousnessPipeline
    WireModule {
        /// 文件名
        file_name: String,
    },
    /// 调整模块参数
    TuneMutation {
        /// 目标名称
        target: String,
        /// 调整幅度
        delta: f64,
    },
    /// 创建新模块
    NewModule {
        /// 模块名称
        name: String,
    },
    /// 架构自审计
    SelfAudit,
    /// 书签维护 — 检查死链 / 重新分类
    BookmarkMaintenance,
    /// 外部URL再分析 — 重新访问已收藏的URL以更新分析结果
    UrlRecheck {
        /// 书签ID
        bookmark_id: String,
    },
    /// 用户数字分身进化 — 从交互中学习和更新人格模型
    PersonalityEvolution,
    /// 行为模式观察 — 分析用户交互中的新模式和风格漂移
    BehaviorPatternAnalysis,
    /// 经验树修剪 — 对 ExperienceTree 的四通道修剪维护
    ExperiencePrune,
    /// 模式蒸馏 — 从修剪和合并操作中提取新洞察模式
    PatternDistill,
    /// 结果评估 — 运行基于量规的结果评估
    OutcomeEvaluation,
    /// 梦境模式合成 — 后台迹分析产生新模式洞察
    DreamSynthesis,
    /// 子Agent监督 — 管理持久化Agent会话
    AgentSupervision,
    /// MCP工具暴露 — 注册/更新Consciousness MCP工具
    McpToolRegistration,
    /// 自动审查 — 审核自我修改提议
    AutoReview,
    /// 递归委托 — 将子任务委托给子Agent
    RecursiveDelegation,
}

impl TaskType {
    pub fn name(&self) -> &'static str {
        match self {
            TaskType::ModuleWiring => "module_wiring",
            TaskType::ModuleCreate => "module_create",
            TaskType::CompileFix => "compile_fix",
            TaskType::RefactorExisting => "refactor",
            TaskType::TestCreate => "test_create",
            TaskType::ExperienceDistill => "experience_distill",
            TaskType::Documentation => "docs",
            TaskType::ArchitectureReview => "arch_review",
            TaskType::Performance => "performance",
            TaskType::SecurityHardening => "security",
            TaskType::AbsorbPattern { .. } => "absorb_pattern",
            TaskType::WireModule { .. } => "wire_module",
            TaskType::TuneMutation { .. } => "tune_mutation",
            TaskType::NewModule { .. } => "new_module",
            TaskType::SelfAudit => "self_audit",
            TaskType::BookmarkMaintenance => "bookmark_maintenance",
            TaskType::UrlRecheck { .. } => "url_recheck",
            TaskType::PersonalityEvolution => "personality_evolution",
            TaskType::BehaviorPatternAnalysis => "behavior_pattern_analysis",
            TaskType::ExperiencePrune => "experience_prune",
            TaskType::PatternDistill => "pattern_distill",
            TaskType::OutcomeEvaluation => "outcome_evaluation",
            TaskType::DreamSynthesis => "dream_synthesis",
            TaskType::AgentSupervision => "agent_supervision",
            TaskType::McpToolRegistration => "mcp_tool_registration",
            TaskType::AutoReview => "auto_review",
            TaskType::RecursiveDelegation => "recursive_delegation",
        }
    }
}

/// 进化任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// 刚刚通过自诊断发现
    Discovered,
    /// 已提交为提案
    Proposed,
    /// 已完成影响评估和优先级排序
    Prioritized,
    /// 计划已制定，包含具体步骤
    Planned,
    /// 执行中
    InProgress,
    /// 验证通过，已关闭
    Completed,
    /// 被阻塞（依赖未就绪）
    Blocked,
    /// 不再需要
    Cancelled,
}

/// 一个进化任务 — 意识对自己要做的改进
#[derive(Debug, Clone)]
pub struct EvolutionTask {
    pub id: u64,
    pub task_type: TaskType,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: u8,
    pub impact: f64,
    pub effort_estimate: f64,
    pub dependencies: Vec<u64>,
    pub sub_tasks: Vec<EvolutionTask>,
    pub gap_ids: Vec<String>,
    pub verification_criteria: Vec<String>,
    pub created_cycle: u64,
    pub started_cycle: Option<u64>,
    pub completed_cycle: Option<u64>,
}

impl EvolutionTask {
    pub fn new(id: u64, task_type: TaskType, title: &str, description: &str) -> Self {
        Self {
            id,
            task_type,
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Discovered,
            priority: 5,
            impact: 0.5,
            effort_estimate: 1.0,
            dependencies: Vec::new(),
            sub_tasks: Vec::new(),
            gap_ids: Vec::new(),
            verification_criteria: Vec::new(),
            created_cycle: 0,
            started_cycle: None,
            completed_cycle: None,
        }
    }
}

/// 自进化任务系统 — Consciousness 内部的自我改进任务管线
///
/// 这不是一个 TODO 文件。这是一个运行时数据结构，活在意识进程内：
/// - 通过自诊断自动发现缺口 → 生成任务
/// - 每个任务有优先级、影响评估、验证标准
/// - 完成的任务会更新经验树
/// - 始终跟踪：意识知道自己要改进什么、为什么、怎么验证
#[derive(Debug)]
pub struct EvolutionTaskSystem {
    tasks: HashMap<u64, EvolutionTask>,
    next_id: u64,
    history: VecDeque<u64>,
    max_history: usize,
    task_counter: HashMap<String, usize>,
}

impl Default for EvolutionTaskSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionTaskSystem {
    pub fn new() -> Self {
        let mut ts = Self {
            tasks: HashMap::new(),
            next_id: 1,
            history: VecDeque::with_capacity(500),
            max_history: 500,
            task_counter: HashMap::new(),
        };
        ts.seed_from_audit();
        ts
    }

    /// 基于实际审计发现的缺口为 Consciousness 创建初始进化任务
    fn seed_from_audit(&mut self) {
        // M0 — 元缺陷: 8个认知模块文件存在但从未接入 ConsciousnessPipeline
        let wiring_id = self.create_task(
            TaskType::ModuleWiring,
            "ModuleRegistry: 将8个认知模块接入 ConsciousnessPipeline",
            "MCTS/ParallelHypothesis/DeadEndDetector/Counterfactual/Causal/Analogical/WorldModel/ConfidenceCalibrator — 所有模块文件都存在(430-915行)，但 ConsciousnessPipeline 只有8个硬编码阶段",
            10, 0.95,
        );
        self.add_verification(wiring_id, "ModuleRegistry trait 在 pipeline build 时可用");
        self.add_verification(wiring_id, "至少 4 个认知模块注册到 pipeline");

        let comp_id = self.create_task(
            TaskType::CompileFix,
            "修复 48 个 E0432/E0433 编译错误",
            "测试层 48 个错误: 模块路径不解析(bdk_wallet API变化/web_agent/EvolutionAction路径/PrmConfig import)。核心 lib 已 0 errors。",
            9, 0.8,
        );
        self.add_verification(
            comp_id,
            "cargo check -p neotrix-core --tests 中 E0432/E0433 全部消失",
        );

        // Draft-Edit-Refine: 编排现有 refinery 循环
        let der_id = self.create_task(
            TaskType::ModuleWiring,
            "Draft-Edit-Refine 循环: 编排现有 ConsciousnessRefineryLoop",
            "当前 refinery loop 是单次前向传递。Draft-Edit-Refine 外层编排: draft→review→refine→verify 4步。不改内部逻辑。",
            8, 0.7,
        );
        self.add_dependency(der_id, wiring_id);
        self.add_verification(der_id, "Draft-Edit-Refine 在 pipeline 中可用");

        // 合成指标修复
        let meta_id = self.create_task(
            TaskType::RefactorExisting,
            "修复合成指标: meta_accuracy 从 tech_debt/1000 改为真实预测-结果",
            "SelfEvolutionMetaLayer 已闭合校准→元认知回路。需要验证真实 ECE/meta-d 流入 meta_accuracy，替换旧的合成值路径。",
            8, 0.75,
        );
        self.add_verification(meta_id, "meta_accuracy 不从 tech_debt 合成");

        // SelfModifyGuard 验证
        let guard_id = self.create_task(
            TaskType::TestCreate,
            "验证 SelfModifyGuard 4层在生产中真实工作",
            "GuardActivator 已创建但 Shield/Swords/LLM/Ball 4层需要端到端集成测试确认非 nop",
            7,
            0.65,
        );
        self.add_dependency(guard_id, meta_id);

        // A2A v1.2 升级
        let a2a_id = self.create_task(
            TaskType::RefactorExisting,
            "A2A 协议升级: 桥接模式 → 原生 gRPC + 签名 Agent Card",
            "A2A v1.2 已达 Linux Foundation 治理 + 150 org 生产部署。需从自定义桥接迁移到 a2a-rs SDK。",
            6, 0.5,
        );

        // 经验树蒸馏
        self.create_task(
            TaskType::ExperienceDistill,
            "蒸馏 2026-06-23 五期会话经验到 AGENTS.md",
            "五期会话: 质量审计→认知自审→三波进化→自进化元层→EvolutionTaskSystem",
            5,
            0.4,
        );

        // 定义依赖关系
        self.add_dependency(meta_id, wiring_id);
        self.add_dependency(guard_id, wiring_id);
        self.add_dependency(a2a_id, wiring_id);

        // 标记所有任务为 Prioritized (跳过 Discovered 阶段)
        let task_ids: Vec<u64> = self.tasks.keys().copied().collect();
        for id in task_ids {
            if let Some(task) = self.tasks.get_mut(&id) {
                if task.status == TaskStatus::Discovered {
                    task.status = TaskStatus::Prioritized;
                }
            }
        }
    }

    /// 创建新任务（幂等：相同 type+title 的任务不重复创建）
    pub fn create_task(
        &mut self,
        task_type: TaskType,
        title: &str,
        description: &str,
        priority: u8,
        impact: f64,
    ) -> u64 {
        // 检查是否存在相同 type+title 的活跃（未完成/未取消）任务
        let type_name = task_type.name();
        for task in self.tasks.values() {
            if task.task_type.name() == type_name && task.title == title {
                match task.status {
                    TaskStatus::Completed | TaskStatus::Cancelled => {}
                    _ => return task.id,
                }
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let type_name_clone = task_type.name().to_string();
        let mut task = EvolutionTask::new(id, task_type, title, description);
        task.priority = priority.clamp(1, 10);
        task.impact = impact.clamp(0.0, 1.0);
        task.created_cycle = self.history.len() as u64;
        self.tasks.insert(id, task);
        *self
            .task_counter
            .entry(type_name_clone)
            .or_insert(0) += 1;
        id
    }

    /// 设置任务依赖
    pub fn add_dependency(&mut self, task_id: u64, depends_on: u64) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if !task.dependencies.contains(&depends_on) {
                task.dependencies.push(depends_on);
            }
        }
    }

    /// 添加子任务
    pub fn add_sub_task(&mut self, parent_id: u64, child: EvolutionTask) {
        if let Some(parent) = self.tasks.get_mut(&parent_id) {
            let child_id = child.id;
            parent.sub_tasks.push(child);
            *self
                .task_counter
                .entry("module_wiring".to_string())
                .or_insert(0) += 1;
            self.next_id = self.next_id.max(child_id + 1);
        }
    }

    /// 更新任务状态
    pub fn update_status(&mut self, task_id: u64, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = status;
            match status {
                TaskStatus::InProgress => task.started_cycle = Some(self.history.len() as u64),
                TaskStatus::Completed => {
                    task.completed_cycle = Some(self.history.len() as u64);
                    self.history.push_back(task_id);
                    if self.history.len() > self.max_history {
                        self.history.pop_front();
                    }
                }
                _ => {}
            }
        }
    }

    /// 获取最高优先级的待办任务（考虑依赖）
    pub fn next_ready_task(&self) -> Option<&EvolutionTask> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Discovered || t.status == TaskStatus::Prioritized)
            .filter(|t| {
                t.dependencies.iter().all(|dep_id| {
                    self.tasks.get(dep_id).map_or(false, |d| {
                        matches!(d.status, TaskStatus::Completed | TaskStatus::Cancelled)
                    })
                })
            })
            .max_by(|a, b| {
                let a_score = a.priority as f64 * a.impact;
                let b_score = b.priority as f64 * b.impact;
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// 基于自诊断结果自动生成任务
    pub fn auto_discover_from_audit(
        &mut self,
        cycle: u64,
        meta_accuracy: f64,
        ece: f64,
        composite_loss: f64,
        pending_module_count: usize,
    ) -> Vec<u64> {
        let mut new_ids = Vec::new();

        if ece > 0.15 {
            let id = self.create_task(
                TaskType::ArchitectureReview,
                "校准误差偏高: 需要校准管线评审",
                &format!(
                    "ECE={:.3} 超过0.15阈值，校准数据可能未正确桥接到元认知",
                    ece
                ),
                8,
                0.7,
            );
            new_ids.push(id);
        }

        if composite_loss > 0.4 {
            let id = self.create_task(
                TaskType::ArchitectureReview,
                "复合损失偏高: 需要损失分析",
                &format!("composite_loss={:.3} 超过0.4阈值", composite_loss),
                9,
                0.8,
            );
            new_ids.push(id);
        }

        if meta_accuracy < 0.7 {
            let id = self.create_task(
                TaskType::ArchitectureReview,
                "元精度偏低: 需要元认知校准改进",
                &format!("meta_accuracy={:.3} 低于0.7阈值", meta_accuracy),
                7,
                0.6,
            );
            new_ids.push(id);
        }

        if pending_module_count > 0 {
            let id = self.create_task(
                TaskType::ModuleWiring,
                &format!("{}个认知模块待接线", pending_module_count),
                &format!(
                    "已有{}个模块文件但未接入ConsciousnessPipeline",
                    pending_module_count
                ),
                10,
                0.9,
            );
            new_ids.push(id);
        }

        for id in &new_ids {
            if let Some(task) = self.tasks.get_mut(id) {
                task.created_cycle = cycle;
            }
        }

        new_ids
    }

    /// 从书签管理器自动发现维护任务
    pub fn auto_discover_bookmark_tasks(
        &mut self,
        cycle: u64,
        stale_count: usize,
        total_bookmarks: usize,
    ) -> Vec<u64> {
        let mut new_ids = Vec::new();
        if stale_count > 10
            || (stale_count > 0
                && total_bookmarks > 50
                && stale_count as f64 / total_bookmarks as f64 > 0.2)
        {
            let id = self.create_task(
                TaskType::BookmarkMaintenance,
                &format!("{}个书签已过期需要重新检查", stale_count),
                &format!(
                    "{}个书签超过阈值未被访问，需要死链检测和重新分析",
                    stale_count
                ),
                5,
                0.4,
            );
            new_ids.push(id);
        }
        if total_bookmarks > 100 {
            let id = self.create_task(
                TaskType::ArchitectureReview,
                "书签数量超过100，需要归档策略审计",
                &format!("{}个书签可能包含已过时的链接", total_bookmarks),
                4,
                0.3,
            );
            new_ids.push(id);
        }
        for id in &new_ids {
            if let Some(task) = self.tasks.get_mut(id) {
                task.created_cycle = cycle;
            }
        }
        new_ids
    }

    pub fn auto_discover_personality_tasks(
        &mut self,
        cycle: u64,
        total_interactions: u64,
        evolution_version: u32,
        observation_count: usize,
        decision_pattern_count: usize,
    ) -> Vec<u64> {
        let mut new_ids = Vec::new();
        if total_interactions > 50 && evolution_version == 0 {
            let id = self.create_task(
                TaskType::PersonalityEvolution,
                "人格模型未初始化",
                &format!(
                    "已累积{}次交互但人格演化版本为0，需要初始化用户数字分身",
                    total_interactions
                ),
                6,
                0.5,
            );
            new_ids.push(id);
        }
        if observation_count > 20 {
            let id = self.create_task(
                TaskType::BehaviorPatternAnalysis,
                "积累了大量观察数据待综合分析",
                &format!(
                    "已有{}条观察记录，需要运行趋势分析和行为模式挖掘",
                    observation_count
                ),
                5,
                0.4,
            );
            new_ids.push(id);
        }
        if decision_pattern_count > 10 {
            let id = self.create_task(
                TaskType::PersonalityEvolution,
                "决策模式数量已超过10个,需要合并和精简",
                &format!(
                    "已有{}个决策模式，超过精简阈值，需合并相似模式并移除低证据模式",
                    decision_pattern_count
                ),
                6,
                0.5,
            );
            new_ids.push(id);
        }
        for id in &new_ids {
            if let Some(task) = self.tasks.get_mut(id) {
                task.created_cycle = cycle;
            }
        }
        new_ids
    }

    /// 获取指定类型的任务列表
    pub fn tasks_by_type(&self, task_type: TaskType) -> Vec<&EvolutionTask> {
        self.tasks
            .values()
            .filter(|t| t.task_type == task_type)
            .collect()
    }

    /// 获取所有任务
    pub fn all_tasks(&self) -> Vec<&EvolutionTask> {
        self.tasks.values().collect()
    }

    /// 任务统计
    pub fn stats(&self) -> TaskSystemStats {
        let total = self.tasks.len();
        let completed = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let blocked = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Blocked)
            .count();
        let discovered = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Discovered)
            .count();
        let avg_impact = self
            .tasks
            .values()
            .filter(|t| t.status != TaskStatus::Cancelled)
            .map(|t| t.impact)
            .sum::<f64>()
            / total.max(1) as f64;

        TaskSystemStats {
            total,
            completed,
            in_progress,
            blocked,
            discovered,
            avg_impact,
            by_type: self.task_counter.clone(),
        }
    }

    /// 添加验证标准
    pub fn add_verification(&mut self, task_id: u64, criterion: &str) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.verification_criteria.push(criterion.to_string());
        }
    }

    /// 标记任务成功完成，记录指标变化
    pub fn mark_completed(&mut self, task_id: u64, metric_delta: f64) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Completed;
            task.completed_cycle = Some(self.history.len() as u64);
            self.history.push_back(task_id);
            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
            // 提高 impact meta（已完成的核心缺口影响更大）
            task.impact = task.impact.max(metric_delta.abs().min(1.0));
        }
    }

    /// 标记任务失败，保留失败记录供元层自省
    pub fn mark_failed(&mut self, task_id: u64, reason: &str) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Discovered; // 重置为 Discovered，可被重新选中
            task.completed_cycle = None;
            task.impact *= 0.9; // 稍微降低 impact 避免重复选中
            task.verification_criteria
                .push(format!("FAILED: {}", reason));
        }
    }

    /// 关联缺口ID
    pub fn link_gap(&mut self, task_id: u64, gap_id: &str) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.gap_ids.push(gap_id.to_string());
        }
    }

    /// Return a copy of all current task IDs (for rollback snapshotting)
    pub fn task_ids(&self) -> std::collections::HashSet<u64> {
        self.tasks.keys().copied().collect()
    }

    /// Remove a task by ID (for rollback cleanup)
    pub fn remove_task(&mut self, task_id: u64) {
        if let Some(task) = self.tasks.remove(&task_id) {
            let tn = task.task_type.name().to_string();
            if let Some(c) = self.task_counter.get_mut(&tn) {
                *c = c.saturating_sub(1);
            }
        }
    }
}

/// 任务系统统计数据
#[derive(Debug, Clone)]
pub struct TaskSystemStats {
    pub total: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub discovered: usize,
    pub avg_impact: f64,
    pub by_type: HashMap<String, usize>,
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let mut ts = EvolutionTaskSystem::new();
        let id = ts.create_task(
            TaskType::ModuleWiring,
            "Wire MCTS",
            "Wire MCTS into pipeline",
            10,
            0.9,
        );
        assert_eq!(id, 1);
        assert_eq!(ts.tasks.len(), 1);
        let task = ts.tasks.get(&id).unwrap();
        assert_eq!(task.title, "Wire MCTS");
        assert_eq!(task.status, TaskStatus::Discovered);
    }

    #[test]
    fn test_task_status_transition() {
        let mut ts = EvolutionTaskSystem::new();
        let id = ts.create_task(
            TaskType::CompileFix,
            "Fix E0432",
            "Fix import errors",
            8,
            0.7,
        );
        ts.update_status(id, TaskStatus::InProgress);
        assert_eq!(ts.tasks.get(&id).unwrap().status, TaskStatus::InProgress);
        assert!(ts.tasks.get(&id).unwrap().started_cycle.is_some());
        ts.update_status(id, TaskStatus::Completed);
        assert_eq!(ts.tasks.get(&id).unwrap().status, TaskStatus::Completed);
        assert!(ts.tasks.get(&id).unwrap().completed_cycle.is_some());
    }

    #[test]
    fn test_dependency_blocking() {
        let mut ts = EvolutionTaskSystem::new();
        let dep_id = ts.create_task(
            TaskType::ModuleCreate,
            "Create stub",
            "Create stub module",
            5,
            0.3,
        );
        let task_id = ts.create_task(
            TaskType::ModuleWiring,
            "Wire module",
            "Wire module",
            10,
            0.9,
        );
        ts.add_dependency(task_id, dep_id);

        assert!(ts.next_ready_task().is_none());

        ts.update_status(dep_id, TaskStatus::Completed);
        let next = ts.next_ready_task();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, task_id);
    }

    #[test]
    fn test_priority_scoring() {
        let mut ts = EvolutionTaskSystem::new();
        let low = ts.create_task(
            TaskType::TestCreate,
            "Low priority",
            "Low impact task",
            2,
            0.1,
        );
        let high = ts.create_task(
            TaskType::ModuleWiring,
            "High priority",
            "High impact task",
            10,
            0.9,
        );

        let next = ts.next_ready_task().unwrap();
        assert_eq!(next.id, high, "high priority*impact should win");
    }

    #[test]
    fn test_auto_discovery() {
        let mut ts = EvolutionTaskSystem::new();
        let new_ids = ts.auto_discover_from_audit(1, 0.5, 0.3, 0.6, 3);

        assert_eq!(new_ids.len(), 3);
        assert!(ts.tasks_by_type(TaskType::ModuleWiring).len() >= 1);
        assert!(ts.tasks_by_type(TaskType::ArchitectureReview).len() >= 2);
    }

    #[test]
    fn test_one_completed_in_history() {
        let mut ts = EvolutionTaskSystem::new();
        let id = ts.create_task(
            TaskType::ExperienceDistill,
            "Distill session",
            "Distill session",
            5,
            0.5,
        );
        ts.update_status(id, TaskStatus::Completed);
        assert_eq!(ts.history.len(), 1);
    }

    #[test]
    fn test_verification_criteria() {
        let mut ts = EvolutionTaskSystem::new();
        let id = ts.create_task(TaskType::CompileFix, "Fix errors", "Fix", 8, 0.7);
        ts.add_verification(id, "cargo check passes with 0 errors");
        ts.add_verification(id, "all tests pass");
        let task = ts.tasks.get(&id).unwrap();
        assert_eq!(task.verification_criteria.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut ts = EvolutionTaskSystem::new();
        ts.create_task(TaskType::ModuleWiring, "t1", "", 10, 0.9);
        ts.create_task(TaskType::CompileFix, "t2", "", 8, 0.7);
        let stats = ts.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 0);
        assert!(stats.avg_impact > 0.0);
    }

    #[test]
    fn test_gap_linking() {
        let mut ts = EvolutionTaskSystem::new();
        let id = ts.create_task(TaskType::ModuleWiring, "Wire MCTS", "Wire MCTS", 10, 0.9);
        ts.link_gap(id, "M0");
        ts.link_gap(id, "M0.1");
        assert_eq!(ts.tasks.get(&id).unwrap().gap_ids.len(), 2);
    }

    #[test]
    fn test_subtasks() {
        let mut ts = EvolutionTaskSystem::new();
        let parent = ts.create_task(
            TaskType::ModuleCreate,
            "Create module",
            "Create module",
            5,
            0.5,
        );
        let child = EvolutionTask::new(100, TaskType::CompileFix, "Fix sub", "Fix sub-issue");
        ts.add_sub_task(parent, child);
        assert_eq!(ts.tasks.get(&parent).unwrap().sub_tasks.len(), 1);
    }

    #[test]
    fn test_task_counter() {
        let mut ts = EvolutionTaskSystem::new();
        ts.create_task(TaskType::ModuleWiring, "w1", "", 5, 0.5);
        ts.create_task(TaskType::ModuleWiring, "w2", "", 5, 0.5);
        ts.create_task(TaskType::CompileFix, "c1", "", 5, 0.5);
        let stats = ts.stats();
        assert_eq!(stats.by_type.get("module_wiring").unwrap(), &2);
        assert_eq!(stats.by_type.get("compile_fix").unwrap(), &1);
    }

    #[test]
    fn test_auto_discovery_creates_module_wiring_tasks() {
        let mut ts = EvolutionTaskSystem::new();
        let ids = ts.auto_discover_from_audit(1, 0.85, 0.05, 0.2, 8);
        let wiring_tasks: Vec<_> = ts
            .tasks_by_type(TaskType::ModuleWiring)
            .iter()
            .map(|t| t.title.clone())
            .collect();
        assert!(!wiring_tasks.is_empty());
        assert!(wiring_tasks[0].contains("8个"));
    }
}
