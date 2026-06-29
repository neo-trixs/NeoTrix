use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::bus::AgentCommunicationBus;
use super::message::{
    AgentId, AgentMessage, AgentRole, AgentStatus, MessageContent, MessagePriority,
};
use crate::core::nt_core_experience::context_compression::{CompressConfig, ContextCompressor};
use crate::core::nt_core_experience::handler_tier::{HandlerRegistry, LoadTier};
use crate::core::nt_core_util;

static NEXT_SUBAGENT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubAgentCapability {
    Coder,
    Reviewer,
    Researcher,
    Tester,
    Integrator,
    Documenter,
    Visualizer,
    SecurityAuditor,
    Planner,
    InfraOps,
}

impl SubAgentCapability {
    pub fn label(&self) -> &'static str {
        match self {
            SubAgentCapability::Coder => "coder",
            SubAgentCapability::Reviewer => "reviewer",
            SubAgentCapability::Researcher => "researcher",
            SubAgentCapability::Tester => "tester",
            SubAgentCapability::Integrator => "integrator",
            SubAgentCapability::Documenter => "documenter",
            SubAgentCapability::Visualizer => "visualizer",
            SubAgentCapability::SecurityAuditor => "security_auditor",
            SubAgentCapability::Planner => "planner",
            SubAgentCapability::InfraOps => "infra_ops",
        }
    }
}

/// SubAgent 隔离策略
/// Codex: Cloud Sandbox（容器级隔离）
/// Claude Code: Git Worktree（分支级隔离）
/// NeoTrix: 双模式透明切换
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationStrategy {
    /// 本地终端直接执行（默认，低开销）
    Local,
    /// Git Worktree 分支隔离（适合并行修改同一仓库）
    GitWorktree,
    /// 云沙箱容器隔离（适合不信任代码/批量执行）
    CloudSandbox,
}

impl Default for IsolationStrategy {
    fn default() -> Self {
        IsolationStrategy::Local
    }
}

/// SubAgent 失败恢复策略
/// 模仿 Claude Code 的 Dynamic Workflows / Codex 的 Pipeline Error Recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// 不做恢复，直接上报失败
    None,
    /// 重试相同 capability（最多 retry_limit 次）
    Retry { retry_limit: u32 },
    /// 升级到更强 capability 重试
    Escalate {
        retry_limit: u32,
        target: SubAgentCapability,
    },
    /// 电路熔断：连续失败 blocking_count 次后暂停该类型任务
    CircuitBreaker {
        blocking_count: u32,
        cooldown_secs: u64,
    },
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        RecoveryStrategy::Retry { retry_limit: 2 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentConfig {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub capability: SubAgentCapability,
    pub max_context_tokens: usize,
    pub max_iterations: u32,
    pub timeout_secs: u64,
    pub allowed_handlers: Vec<String>,
    pub isolation: IsolationStrategy,
    pub recovery: RecoveryStrategy,
    pub sandbox_root: Option<String>,
}

impl Default for SubAgentConfig {
    fn default() -> Self {
        Self {
            agent_id: AgentId::with_random_instance("subagent", "1.0"),
            role: AgentRole::Specialist,
            capability: SubAgentCapability::Coder,
            max_context_tokens: 8192,
            max_iterations: 10,
            timeout_secs: 300,
            allowed_handlers: Vec::new(),
            isolation: IsolationStrategy::default(),
            recovery: RecoveryStrategy::default(),
            sandbox_root: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentContext {
    pub conversation_id: u64,
    pub task_description: String,
    pub artifacts: Vec<String>,
    pub constraints: Vec<String>,
    pub dependency_ids: Vec<u64>,
    pub created_at: u64, // ms since epoch (for serde compat)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    pub agent_id: AgentId,
    pub task_description: String,
    pub output: String,
    pub artifacts: Vec<String>,
    pub confidence: f64,
    pub iterations_used: u32,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

pub struct SubAgentRuntime {
    pub config: SubAgentConfig,
    pub context: SubAgentContext,
    pub handler_registry: HandlerRegistry,
    pub iteration: u32,
    pub start_time: Instant,
    pub result: Option<SubAgentResult>,
    /// 上下文压缩器 — 防溢出 (Claude Code 1M Context 自动压缩模式)
    pub compressor: Option<ContextCompressor>,
    /// 隔离沙箱 (Codex Cloud Sandbox 模式)
    pub sandbox_path: Option<std::path::PathBuf>,
    /// 失败计数 — 用于 RecoveryStrategy::CircuitBreaker
    pub consecutive_failures: u32,
    /// 熔断冷却截止时间
    pub circuit_open_until: Option<Instant>,
}

impl SubAgentRuntime {
    pub fn new(config: SubAgentConfig, task: &str, bus: &mut AgentCommunicationBus) -> Self {
        let id = NEXT_SUBAGENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let conv_id = id;
        let mut registry = HandlerRegistry::new();
        for h in &config.allowed_handlers {
            registry.register(h, LoadTier::Hot);
        }
        registry.register_many(&[
            ("context_gather", LoadTier::Hot),
            ("decision_compress", LoadTier::Hot),
            ("experience_reflect", LoadTier::Warm),
            ("goal_execution", LoadTier::Hot),
            ("proof_search", LoadTier::Hot),
            ("cognitive_load", LoadTier::Hot),
        ]);
        let _ = bus.register_agent(config.agent_id.clone(), AgentStatus::Idle);
        let sandbox_path = match config.isolation {
            IsolationStrategy::CloudSandbox => {
                let root = config.sandbox_root.clone().unwrap_or_else(|| {
                    let base = nt_core_util::home_dir().to_string_lossy().to_string();
                    format!("{}/.neotrix/sandbox/agent-{}", base, id)
                });
                let path = std::path::PathBuf::from(&root);
                let _ = std::fs::create_dir_all(&path);
                Some(path)
            }
            IsolationStrategy::GitWorktree => {
                let base = nt_core_util::home_dir().to_string_lossy().to_string();
                let root = format!("{}/.neotrix/worktree/agent-{}", base, id);
                let path = std::path::PathBuf::from(&root);
                let _ = std::fs::create_dir_all(&path);
                Some(path)
            }
            IsolationStrategy::Local => None,
        };
        let compressor = Some(ContextCompressor::new(CompressConfig {
            max_tokens: config.max_context_tokens,
            ..CompressConfig::default()
        }));
        Self {
            config,
            context: SubAgentContext {
                conversation_id: conv_id,
                task_description: task.to_string(),
                artifacts: Vec::new(),
                constraints: Vec::new(),
                dependency_ids: Vec::new(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            },
            handler_registry: registry,
            iteration: 0,
            start_time: Instant::now(),
            result: None,
            compressor,
            sandbox_path,
            consecutive_failures: 0,
            circuit_open_until: None,
        }
    }

    pub fn step(&mut self, bus: &mut AgentCommunicationBus) -> SubAgentStepResult {
        self.iteration += 1;
        if self.iteration > self.config.max_iterations {
            return SubAgentStepResult::MaxIterationsReached;
        }
        if self.start_time.elapsed() > Duration::from_secs(self.config.timeout_secs) {
            return SubAgentStepResult::Timeout;
        }
        let messages: Vec<_> = bus
            .deliver()
            .into_iter()
            .filter(|m| m.is_intended_for(&self.config.agent_id))
            .collect();
        for msg in messages {
            self.handle_message(&msg, bus);
        }
        SubAgentStepResult::Continue
    }

    pub fn handle_message(&mut self, msg: &AgentMessage, bus: &mut AgentCommunicationBus) {
        match &msg.content {
            MessageContent::TaskRequest {
                description,
                constraints,
                ..
            } => {
                self.context.task_description.push_str("\n");
                self.context.task_description.push_str(description);
                self.context.constraints.extend(constraints.clone());
                let status = AgentMessage::new(
                    self.config.agent_id.clone(),
                    msg.recipients.clone(),
                    MessageContent::StatusUpdate {
                        status: AgentStatus::Busy,
                        progress: 0.3,
                        message: format!(
                            "Acknowledged task: {}",
                            &description[..description.len().min(60)]
                        ),
                    },
                    MessagePriority::Normal,
                    Duration::from_secs(60),
                )
                .reply_to_id(msg.id)
                .with_conversation(msg.conversation_id);
                if let Err(e) = bus.send(status) {
                    log::warn!("[sub_agent] bus send status failed: {}", e);
                }
            }
            MessageContent::Query { question, .. } => {
                let response = AgentMessage::new(
                    self.config.agent_id.clone(),
                    vec![msg.sender.clone()],
                    MessageContent::Response {
                        answer: format!(
                            "SubAgent {} processing query: {}",
                            self.config.agent_id, question
                        ),
                        sources: vec![],
                    },
                    MessagePriority::Normal,
                    Duration::from_secs(60),
                )
                .with_conversation(msg.conversation_id);
                if let Err(e) = bus.send(response) {
                    log::warn!("[sub_agent] bus send response failed: {}", e);
                }
            }
            MessageContent::Coordination {
                action: crate::core::nt_core_agent::message::CoordinationAction::Cancel,
                ..
            } => {
                self.result = Some(SubAgentResult {
                    agent_id: self.config.agent_id.clone(),
                    task_description: self.context.task_description.clone(),
                    output: String::new(),
                    artifacts: self.context.artifacts.clone(),
                    confidence: 0.0,
                    iterations_used: self.iteration,
                    duration_ms: self.start_time.elapsed().as_millis() as u64,
                    success: false,
                    error: Some("Cancelled".to_string()),
                });
            }
            _ => {}
        }
    }

    /// 检查并压缩上下文 — Claude Code 风格的自适应压缩
    pub fn compress_context_if_needed(&mut self) -> Option<String> {
        let compressor = self.compressor.as_mut()?;
        if !compressor.should_compress() {
            return None;
        }
        let report = compressor.compress();
        let _ = report;
        // 压缩后清理冗余的任务描述 — 保留关键部分
        if self.context.task_description.len() > 1024 {
            let truncated: String = self.context.task_description.chars().take(512).collect();
            self.context.task_description = format!(
                "{}...[+{}.chars compressed]",
                truncated,
                self.context.task_description.len() - 512,
            );
        }
        Some("context compressed".into())
    }

    /// 检查电路熔断状态 — Codex 式的错误隔离
    pub fn is_circuit_open(&self) -> bool {
        match self.circuit_open_until {
            Some(until) => Instant::now() < until,
            None => false,
        }
    }

    /// 记录失败并检查是否需要熔断 — 返回 true 表示已熔断
    pub fn record_failure_and_check_circuit(&mut self) -> bool {
        self.consecutive_failures += 1;
        if let RecoveryStrategy::CircuitBreaker {
            blocking_count,
            cooldown_secs,
        } = self.config.recovery
        {
            if self.consecutive_failures >= blocking_count {
                self.circuit_open_until = Some(Instant::now() + Duration::from_secs(cooldown_secs));
                return true;
            }
        }
        false
    }

    /// 重置失败计数 — 成功后调用
    pub fn reset_failure_count(&mut self) {
        self.consecutive_failures = 0;
        self.circuit_open_until = None;
    }

    pub fn complete(&mut self, output: String, confidence: f64) -> SubAgentResult {
        let result = SubAgentResult {
            agent_id: self.config.agent_id.clone(),
            task_description: self.context.task_description.clone(),
            output,
            artifacts: self.context.artifacts.clone(),
            confidence,
            iterations_used: self.iteration,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            success: true,
            error: None,
        };
        self.result = Some(result.clone());
        result
    }

    pub fn fail(&mut self, error: String) -> SubAgentResult {
        self.record_failure_and_check_circuit();
        let result = SubAgentResult {
            agent_id: self.config.agent_id.clone(),
            task_description: self.context.task_description.clone(),
            output: String::new(),
            artifacts: self.context.artifacts.clone(),
            confidence: 0.0,
            iterations_used: self.iteration,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            success: false,
            error: Some(error),
        };
        self.result = Some(result.clone());
        result
    }
}

pub enum SubAgentStepResult {
    Continue,
    MaxIterationsReached,
    Timeout,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDecomposition {
    pub sub_tasks: Vec<SubTaskSpec>,
    pub dependency_graph: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskSpec {
    pub id: usize,
    pub description: String,
    pub capability: SubAgentCapability,
    pub constraints: Vec<String>,
    pub expected_artifacts: Vec<String>,
    pub recovery: RecoveryStrategy,
}

impl SubTaskSpec {
    pub fn new(id: usize, description: &str, capability: SubAgentCapability) -> Self {
        Self {
            id,
            description: description.to_string(),
            capability,
            constraints: Vec::new(),
            expected_artifacts: Vec::new(),
            recovery: RecoveryStrategy::default(),
        }
    }

    pub fn with_recovery(mut self, recovery: RecoveryStrategy) -> Self {
        self.recovery = recovery;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<String>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.expected_artifacts = artifacts;
        self
    }
}

impl TaskDecomposition {
    pub fn ready_tasks(&self, completed: &[usize]) -> Vec<usize> {
        let completed_set: std::collections::HashSet<usize> = completed.iter().copied().collect();
        let mut ready = Vec::new();
        'outer: for task in &self.sub_tasks {
            if completed_set.contains(&task.id) {
                continue;
            }
            for (from, to) in &self.dependency_graph {
                if *to == task.id && !completed_set.contains(from) {
                    continue 'outer;
                }
            }
            ready.push(task.id);
        }
        ready
    }

    pub fn all_completed(&self, completed: &[usize]) -> bool {
        let completed_set: std::collections::HashSet<usize> = completed.iter().copied().collect();
        self.sub_tasks.iter().all(|t| completed_set.contains(&t.id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadAgentPlan {
    pub goal: String,
    pub decomposition: TaskDecomposition,
    pub strategy: String,
    pub created_at: String,
}
