//! AutoOrchestrator — 自动编排器（替代所有 CLI 管理命令）
//!
//! 设计原则：
//! 1. 用户只说意图，系统自动路由到合适的 agent
//! 2. 自动 spawn/kill/manage agent，零手动管理
//! 3. CLI 命令仅用于观测/调试/应急
//!
//! 架构：
//! ```
//! 用户意图 → IntentClassifier → AutoOrchestrator → AgentRegistry
//!                                    ↓
//!                             AgentLifecycleManager
//!                             (auto spawn/kill/recycle)
//!                                    ↓
//!                             TaskRouter
//!                             (route to best agent)
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::l0_substrate::nt_core_platform::agent::{Agent, AgentError, AgentStatus};
use crate::l0_substrate::nt_core_platform::agent_registry::AgentRegistry;
use crate::l6_meta::nt_agent_identity::{
    AgentIdentityRegistry, AgentPersona, AgentPreset, AgentStatus as IdentityStatus,
    AutonomyLevel,
};
use crate::l6_meta::nt_agent_gallery::AgentGallery;

// ============================================================
// 1. Intent Classification（意图分类）
// ============================================================

/// 任务类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    Debugging,
    Research,
    Architecture,
    Documentation,
    Testing,
    GeneralChat,
    SystemAdmin,
    FileOperations,
    // 新增任务类型
    KnowledgeAbsorption,
    MemoryManagement,
    SecurityAudit,
    PerformanceOptimization,
    WorkflowOrchestration,
}

impl TaskType {
    /// 对应的 agent specialty
    pub fn preferred_specialty(&self) -> &str {
        match self {
            TaskType::CodeGeneration => "code_generation",
            TaskType::CodeReview => "code_review",
            TaskType::Debugging => "debugging",
            TaskType::Research => "research",
            TaskType::Architecture => "architecture",
            TaskType::Documentation => "documentation",
            TaskType::Testing => "testing",
            TaskType::GeneralChat => "general",
            TaskType::SystemAdmin => "system_admin",
            TaskType::FileOperations => "file_operations",
            TaskType::KnowledgeAbsorption => "knowledge_absorption",
            TaskType::MemoryManagement => "memory_management",
            TaskType::SecurityAudit => "security_audit",
            TaskType::PerformanceOptimization => "performance_optimization",
            TaskType::WorkflowOrchestration => "workflow_orchestration",
        }
    }

    /// 任务优先级（越高越优先）
    pub fn priority(&self) -> u8 {
        match self {
            TaskType::SecurityAudit => 10,
            TaskType::Debugging => 9,
            TaskType::CodeGeneration => 8,
            TaskType::CodeReview => 7,
            TaskType::Architecture => 6,
            TaskType::Research => 5,
            TaskType::Testing => 5,
            TaskType::Documentation => 4,
            TaskType::FileOperations => 3,
            TaskType::KnowledgeAbsorption => 3,
            TaskType::MemoryManagement => 3,
            TaskType::PerformanceOptimization => 4,
            TaskType::WorkflowOrchestration => 6,
            TaskType::GeneralChat => 1,
            TaskType::SystemAdmin => 7,
        }
    }

    /// 需要的自治级别
    pub fn required_autonomy(&self) -> AutonomyLevel {
        match self {
            TaskType::SecurityAudit => AutonomyLevel::ReadOnly,
            TaskType::Debugging => AutonomyLevel::Supervised,
            TaskType::CodeGeneration => AutonomyLevel::Constrained,
            TaskType::CodeReview => AutonomyLevel::ReadOnly,
            TaskType::Architecture => AutonomyLevel::Supervised,
            TaskType::Research => AutonomyLevel::ReadOnly,
            TaskType::Testing => AutonomyLevel::Constrained,
            TaskType::Documentation => AutonomyLevel::Constrained,
            TaskType::FileOperations => AutonomyLevel::Constrained,
            TaskType::KnowledgeAbsorption => AutonomyLevel::ReadOnly,
            TaskType::MemoryManagement => AutonomyLevel::Constrained,
            TaskType::PerformanceOptimization => AutonomyLevel::Supervised,
            TaskType::WorkflowOrchestration => AutonomyLevel::Autonomous,
            TaskType::GeneralChat => AutonomyLevel::ReadOnly,
            TaskType::SystemAdmin => AutonomyLevel::Autonomous,
        }
    }
}

/// 意图分类器
pub struct IntentClassifier {
    rules: Vec<ClassificationRule>,
}

#[derive(Debug, Clone)]
pub struct ClassificationRule {
    pub keywords: Vec<String>,
    pub task_type: TaskType,
    pub confidence_boost: f64,
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl IntentClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// 分类用户意图
    pub fn classify(&self, message: &str) -> ClassificationResult {
        let msg_lower = message.to_lowercase();
        let mut scores: HashMap<TaskType, f64> = HashMap::new();

        for rule in &self.rules {
            let matched = rule
                .keywords
                .iter()
                .filter(|kw| msg_lower.contains(kw.as_str()))
                .count();
            if matched > 0 {
                *scores.entry(rule.task_type.clone()).or_insert(0.0) +=
                    rule.confidence_boost * matched as f64;
            }
        }

        let (task_type, confidence) = scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(ty, score)| (ty, (score / 1.2).min(1.0)))
            .unwrap_or((TaskType::GeneralChat, 0.5));

        ClassificationResult {
            task_type,
            confidence,
            keywords: self
                .rules
                .iter()
                .filter(|r| r.task_type == task_type)
                .flat_map(|r| r.keywords.clone())
                .filter(|kw| msg_lower.contains(kw.as_str()))
                .collect(),
        }
    }
}

fn default_rules() -> Vec<ClassificationRule> {
    vec![
        ClassificationRule {
            keywords: vec![
                "实现".into(),
                "写代码".into(),
                "implement".into(),
                "create".into(),
                "build".into(),
                "add function".into(),
                "新建".into(),
                "添加".into(),
                "开发".into(),
            ],
            task_type: TaskType::CodeGeneration,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            keywords: vec![
                "审查".into(),
                "review".into(),
                "audit".into(),
                "check code".into(),
                "代码审查".into(),
                "安全检查".into(),
            ],
            task_type: TaskType::CodeReview,
            confidence_boost: 0.9,
        },
        ClassificationRule {
            keywords: vec![
                "调试".into(),
                "debug".into(),
                "fix".into(),
                "bug".into(),
                "error".into(),
                "报错".into(),
                "修复".into(),
                "异常".into(),
            ],
            task_type: TaskType::Debugging,
            confidence_boost: 0.85,
        },
        ClassificationRule {
            keywords: vec![
                "研究".into(),
                "调研".into(),
                "research".into(),
                "分析".into(),
                "analyze".into(),
                "比较".into(),
                "compare".into(),
                "调查".into(),
            ],
            task_type: TaskType::Research,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            keywords: vec![
                "架构".into(),
                "设计".into(),
                "architecture".into(),
                "design".into(),
                "重构".into(),
                "refactor".into(),
                "模块".into(),
                "module".into(),
            ],
            task_type: TaskType::Architecture,
            confidence_boost: 0.75,
        },
        ClassificationRule {
            keywords: vec![
                "文档".into(),
                "docs".into(),
                "document".into(),
                "注释".into(),
                "comment".into(),
                "README".into(),
                "教程".into(),
                "tutorial".into(),
            ],
            task_type: TaskType::Documentation,
            confidence_boost: 0.85,
        },
        ClassificationRule {
            keywords: vec![
                "测试".into(),
                "test".into(),
                "测试用例".into(),
                "unit test".into(),
                "集成测试".into(),
                "integration test".into(),
            ],
            task_type: TaskType::Testing,
            confidence_boost: 0.85,
        },
        // 新增规则
        ClassificationRule {
            keywords: vec![
                "吸收".into(),
                "absorb".into(),
                "学习".into(),
                "learn".into(),
                "知识".into(),
                "knowledge".into(),
                "导入".into(),
                "import".into(),
            ],
            task_type: TaskType::KnowledgeAbsorption,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            keywords: vec![
                "记忆".into(),
                "memory".into(),
                "存储".into(),
                "store".into(),
                "检索".into(),
                "retrieve".into(),
                "KB".into(),
                "知识库".into(),
            ],
            task_type: TaskType::MemoryManagement,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            keywords: vec![
                "安全".into(),
                "security".into(),
                "漏洞".into(),
                "vulnerability".into(),
                "扫描".into(),
                "scan".into(),
                "审计".into(),
                "audit".into(),
            ],
            task_type: TaskType::SecurityAudit,
            confidence_boost: 0.9,
        },
        ClassificationRule {
            keywords: vec![
                "性能".into(),
                "performance".into(),
                "优化".into(),
                "optimize".into(),
                "加速".into(),
                "speed".into(),
                "慢".into(),
                "slow".into(),
            ],
            task_type: TaskType::PerformanceOptimization,
            confidence_boost: 0.85,
        },
        ClassificationRule {
            keywords: vec![
                "编排".into(),
                "orchestrate".into(),
                "工作流".into(),
                "workflow".into(),
                "流程".into(),
                "pipeline".into(),
                "自动化".into(),
                "automate".into(),
            ],
            task_type: TaskType::WorkflowOrchestration,
            confidence_boost: 0.8,
        },
    ]
}

/// 分类结果
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub task_type: TaskType,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

// ============================================================
// 2. Agent Lifecycle Manager（Agent 生命周期管理）
// ============================================================

/// Agent 实例信息
#[derive(Debug, Clone)]
pub struct AgentInstance {
    pub persona: AgentPersona,
    pub status: InstanceStatus,
    pub last_active: u64,
    pub task_count: u64,
    pub current_task: Option<String>,
}

/// 实例状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceStatus {
    Idle,
    Running,
    CoolingDown,
    Error,
    Paused,
}

/// Agent 生命周期管理器
pub struct AgentLifecycleManager {
    /// 活跃实例
    instances: HashMap<String, AgentInstance>,
    /// 身份注册表
    identity_registry: AgentIdentityRegistry,
    /// 画廊
    gallery: AgentGallery,
    /// 空闲超时（秒）
    idle_timeout: u64,
    /// 冷却时间（秒）
    cooldown_time: u64,
    /// 最大并发实例数
    max_instances: usize,
    /// 成本预算（每会话）
    cost_budget: f64,
    /// 当前会话成本
    current_cost: f64,
}

impl AgentLifecycleManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            identity_registry: AgentIdentityRegistry::new(),
            gallery: AgentGallery::new(),
            idle_timeout: 300,  // 5分钟
            cooldown_time: 60,  // 1分钟
            max_instances: 10,
            cost_budget: 50.0,  // $50 会话预算
            current_cost: 0.0,
        }
    }

    /// 获取或创建 agent 实例
    pub async fn get_or_create(
        &mut self,
        task_type: &TaskType,
    ) -> Result<AgentInstance, AgentError> {
        let specialty = task_type.preferred_specialty();
        let required_autonomy = task_type.required_autonomy();

        // 1. 尝试找空闲实例（匹配 specialty + 自治级别）
        if let Some(instance) = self.find_idle_instance(specialty, &required_autonomy) {
            return Ok(instance.clone());
        }

        // 2. 检查是否达到上限
        if self.instances.len() >= self.max_instances {
            // 尝试回收最旧的空闲实例
            self.recycle_oldest_idle();
            if self.instances.len() >= self.max_instances {
                return Err(AgentError::ExecutionFailed(
                    "Max agent instances reached".into(),
                ));
            }
        }

        // 3. 检查成本预算
        if self.current_cost >= self.cost_budget {
            return Err(AgentError::ExecutionFailed(
                "Cost budget exceeded".into(),
            ));
        }

        // 4. 创建新实例
        self.create_instance(task_type).await
    }

    /// 查找空闲实例
    fn find_idle_instance(
        &self,
        specialty: &str,
        required_autonomy: &AutonomyLevel,
    ) -> Option<&AgentInstance> {
        self.instances
            .values()
            .find(|i| {
                i.status == InstanceStatus::Idle
                    && i.persona.specialty == specialty
                    && i.persona.status == IdentityStatus::Available
                    && i.persona.autonomy.score() >= required_autonomy.score()
            })
            .or_else(|| {
                self.instances.values().find(|i| {
                    i.status == InstanceStatus::Idle
                        && i.persona.status == IdentityStatus::Available
                        && i.persona.autonomy.score() >= required_autonomy.score()
                })
            })
    }

    /// 创建新实例
    async fn create_instance(
        &mut self,
        task_type: &TaskType,
    ) -> Result<AgentInstance, AgentError> {
        let specialty = task_type.preferred_specialty();

        // 从画廊安装预设
        let preset_id = format!("builtin-{}", specialty);
        let persona = self
            .gallery
            .install(&preset_id)
            .map_err(|e| AgentError::InitializationFailed(e))?;

        let instance = AgentInstance {
            persona: persona.clone(),
            status: InstanceStatus::Idle,
            last_active: now_secs(),
            task_count: 0,
            current_task: None,
        };

        // 注册到身份注册表
        self.identity_registry
            .register(persona)
            .map_err(|e| AgentError::InitializationFailed(e))?;

        self.instances
            .insert(persona.id.clone(), instance.clone());

        Ok(instance)
    }

    /// 标记实例运行中
    pub fn mark_running(&mut self, instance_id: &str, task_description: &str) {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.status = InstanceStatus::Running;
            instance.last_active = now_secs();
            instance.current_task = Some(task_description.to_string());
        }
    }

    /// 标记实例完成
    pub fn mark_completed(&mut self, instance_id: &str, cost: f64) {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.status = InstanceStatus::CoolingDown;
            instance.last_active = now_secs();
            instance.task_count += 1;
            instance.current_task = None;
            self.current_cost += cost;
        }
    }

    /// 标记实例错误
    pub fn mark_error(&mut self, instance_id: &str) {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.status = InstanceStatus::Error;
            instance.current_task = None;
        }
    }

    /// 暂停实例
    pub fn pause(&mut self, instance_id: &str) {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            if instance.status == InstanceStatus::Running {
                instance.status = InstanceStatus::Paused;
            }
        }
    }

    /// 恢复实例
    pub fn resume(&mut self, instance_id: &str) {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            if instance.status == InstanceStatus::Paused {
                instance.status = InstanceStatus::Running;
                instance.last_active = now_secs();
            }
        }
    }

    /// 回收超时实例
    pub fn recycle_timed_out(&mut self) {
        let now = now_secs();
        let timeout = self.idle_timeout;
        let cooldown = self.cooldown_time;

        self.instances.retain(|_, instance| {
            match instance.status {
                InstanceStatus::Idle => {
                    // 空闲超时 → 删除
                    now - instance.last_active < timeout
                }
                InstanceStatus::CoolingDown => {
                    // 冷却完成 → 变为空闲
                    if now - instance.last_active >= cooldown {
                        instance.status = InstanceStatus::Idle;
                    }
                    true
                }
                InstanceStatus::Running | InstanceStatus::Paused => true,
                InstanceStatus::Error => false, // 错误实例直接删除
            }
        });
    }

    /// 回收最旧的空闲实例
    fn recycle_oldest_idle(&mut self) {
        if let Some(oldest_id) = self
            .instances
            .iter()
            .filter(|(_, i)| i.status == InstanceStatus::Idle)
            .min_by_key(|(_, i)| i.last_active)
            .map(|(id, _)| id.clone())
        {
            self.instances.remove(&oldest_id);
        }
    }

    /// 获取所有实例状态
    pub fn status_summary(&self) -> Vec<InstanceInfo> {
        self.instances
            .iter()
            .map(|(id, instance)| InstanceInfo {
                id: id.clone(),
                name: instance.persona.name.clone(),
                specialty: instance.persona.specialty.clone(),
                status: format!("{:?}", instance.status),
                task_count: instance.task_count,
                last_active: instance.last_active,
                current_task: instance.current_task.clone(),
            })
            .collect()
    }

    /// 获取统计信息
    pub fn stats(&self) -> LifecycleStats {
        let total = self.instances.len();
        let idle = self
            .instances
            .values()
            .filter(|i| i.status == InstanceStatus::Idle)
            .count();
        let running = self
            .instances
            .values()
            .filter(|i| i.status == InstanceStatus::Running)
            .count();
        let paused = self
            .instances
            .values()
            .filter(|i| i.status == InstanceStatus::Paused)
            .count();
        let total_tasks: u64 = self.instances.values().map(|i| i.task_count).sum();

        LifecycleStats {
            total_instances: total,
            idle,
            running,
            paused,
            total_tasks,
            total_cost: self.current_cost,
            cost_budget: self.cost_budget,
        }
    }
}

/// 实例信息（用于观测）
#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub specialty: String,
    pub status: String,
    pub task_count: u64,
    pub last_active: u64,
    pub current_task: Option<String>,
}

/// 生命周期统计
#[derive(Debug, Clone)]
pub struct LifecycleStats {
    pub total_instances: usize,
    pub idle: usize,
    pub running: usize,
    pub paused: usize,
    pub total_tasks: u64,
    pub total_cost: f64,
    pub cost_budget: f64,
}

// ============================================================
// 3. Task Router（任务路由器）
// ============================================================

/// 任务路由器
pub struct TaskRouter {
    /// 路由历史
    history: Vec<RoutingRecord>,
    /// 路由统计
    stats: HashMap<TaskType, usize>,
}

/// 路由记录
#[derive(Debug, Clone)]
pub struct RoutingRecord {
    pub input: String,
    pub task_type: TaskType,
    pub confidence: f64,
    pub agent_id: String,
    pub agent_name: String,
    pub timestamp: u64,
    pub success: bool,
    pub duration_ms: u64,
}

impl TaskRouter {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            stats: HashMap::new(),
        }
    }

    /// 记录路由
    pub fn record(
        &mut self,
        input: &str,
        classification: &ClassificationResult,
        agent_id: &str,
        agent_name: &str,
        success: bool,
        duration_ms: u64,
    ) {
        self.history.push(RoutingRecord {
            input: truncate(input, 100),
            task_type: classification.task_type.clone(),
            confidence: classification.confidence,
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            timestamp: now_secs(),
            success,
            duration_ms,
        });

        *self
            .stats
            .entry(classification.task_type.clone())
            .or_insert(0) += 1;
    }

    /// 获取最近路由
    pub fn recent_routes(&self, count: usize) -> Vec<&RoutingRecord> {
        self.history.iter().rev().take(count).collect()
    }

    /// 获取路由统计
    pub fn stats(&self) -> RoutingStats {
        let total = self.history.len();
        let successful = self.history.iter().filter(|r| r.success).count();
        let avg_duration = if total > 0 {
            self.history.iter().map(|r| r.duration_ms).sum::<u64>() / total as u64
        } else {
            0
        };

        RoutingStats {
            total_routes: total,
            successful_routes: successful,
            success_rate: if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            },
            avg_duration_ms: avg_duration,
            by_task_type: self.stats.clone(),
        }
    }
}

/// 路由统计
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_routes: usize,
    pub successful_routes: usize,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
    pub by_task_type: HashMap<TaskType, usize>,
}

// ============================================================
// 4. AutoOrchestrator（自动编排器）
// ============================================================

/// 自动编排器
pub struct AutoOrchestrator {
    /// 意图分类器
    classifier: IntentClassifier,
    /// 生命周期管理器
    lifecycle: AgentLifecycleManager,
    /// 任务路由器
    router: TaskRouter,
}

impl AutoOrchestrator {
    pub fn new() -> Self {
        Self {
            classifier: IntentClassifier::new(),
            lifecycle: AgentLifecycleManager::new(),
            router: TaskRouter::new(),
        }
    }

    /// 处理用户意图（核心入口）
    pub async fn handle_intent(
        &mut self,
        user_message: &str,
    ) -> Result<AgentInstance, AgentError> {
        // 1. 分类意图
        let classification = self.classifier.classify(user_message);

        // 2. 获取或创建 agent 实例
        let instance = self.lifecycle.get_or_create(&classification.task_type).await?;

        // 3. 标记运行中
        self.lifecycle
            .mark_running(&instance.persona.id, user_message);

        Ok(instance)
    }

    /// 完成任务
    pub fn complete_task(&mut self, agent_id: &str, success: bool, duration_ms: u64, cost: f64) {
        if success {
            self.lifecycle.mark_completed(agent_id, cost);
        } else {
            self.lifecycle.mark_error(agent_id);
        }

        // 记录路由
        if let Some(instance) = self.lifecycle.instances.get(agent_id) {
            // 这里简化处理，实际应该保存 classification
            let classification = ClassificationResult {
                task_type: TaskType::GeneralChat,
                confidence: 0.5,
                keywords: Vec::new(),
            };
            self.router.record(
                &instance.current_task.clone().unwrap_or_default(),
                &classification,
                agent_id,
                &instance.persona.name,
                success,
                duration_ms,
            );
        }
    }

    /// 回收超时实例（定期调用）
    pub fn recycle(&mut self) {
        self.lifecycle.recycle_timed_out();
    }

    /// 获取编排器状态（用于观测）
    pub fn status(&self) -> OrchestratorStatus {
        let lifecycle_stats = self.lifecycle.stats();
        let routing_stats = self.router.stats();
        let recent_routes: Vec<RoutingRecord> = self
            .router
            .recent_routes(10)
            .into_iter()
            .cloned()
            .collect();

        OrchestratorStatus {
            lifecycle_stats,
            routing_stats,
            recent_routes,
        }
    }

    /// 获取所有实例（用于调试）
    pub fn instances(&self) -> Vec<InstanceInfo> {
        self.lifecycle.status_summary()
    }
}

/// 编排器状态（用于观测）
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub lifecycle_stats: LifecycleStats,
    pub routing_stats: RoutingStats,
    pub recent_routes: Vec<RoutingRecord>,
}

// ============================================================
// 5. Helpers
// ============================================================

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

// ============================================================
// 6. Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_generation() {
        let classifier = IntentClassifier::new();
        let r = classifier.classify("帮我实现一个解析器");
        assert_eq!(r.task_type, TaskType::CodeGeneration);
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn test_classify_debugging() {
        let classifier = IntentClassifier::new();
        let r = classifier.classify("这个函数报错了，帮我调试");
        assert_eq!(r.task_type, TaskType::Debugging);
    }

    #[test]
    fn test_classify_security() {
        let classifier = IntentClassifier::new();
        let r = classifier.classify("扫描代码中的安全漏洞");
        assert_eq!(r.task_type, TaskType::SecurityAudit);
    }

    #[test]
    fn test_classify_general() {
        let classifier = IntentClassifier::new();
        let r = classifier.classify("今天天气怎么样");
        assert_eq!(r.task_type, TaskType::GeneralChat);
    }

    #[test]
    fn test_task_priority() {
        assert!(TaskType::SecurityAudit.priority() > TaskType::GeneralChat.priority());
        assert!(TaskType::Debugging.priority() > TaskType::Documentation.priority());
    }

    #[test]
    fn test_task_autonomy() {
        assert_eq!(
            TaskType::SecurityAudit.required_autonomy(),
            AutonomyLevel::ReadOnly
        );
        assert_eq!(
            TaskType::WorkflowOrchestration.required_autonomy(),
            AutonomyLevel::Autonomous
        );
    }

    #[tokio::test]
    async fn test_orchestrator_handle_intent() {
        let mut orch = AutoOrchestrator::new();
        let instance = orch.handle_intent("帮我实现一个解析器").await;
        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.persona.specialty, "code_generation");
        assert_eq!(instance.status, InstanceStatus::Running);
    }

    #[test]
    fn test_lifecycle_stats() {
        let mut mgr = AgentLifecycleManager::new();
        let stats = mgr.status_summary();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_routing_stats() {
        let mut router = TaskRouter::new();
        let stats = router.stats();
        assert_eq!(stats.total_routes, 0);
    }
}
