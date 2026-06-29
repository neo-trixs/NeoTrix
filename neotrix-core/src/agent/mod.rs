//! # NeoTrix Agent — Agent 运行时层
//!
//! 依赖 `core` 层，提供 Agent 执行循环、工具系统、Provider 路由等运行时能力。
//!
//! ## 子模块
//!
//! - `executor` — Agent 执行器（wrap ReasoningBrain）
//! - `absorb` — 吸收逻辑
//! - `provider` — LLM Provider 路由
//! - `adapters` — Protocol trait 适配器
//! - `team` — 多 Agent 协作系统（AgentTeam + 3 种 ProcessType）
//! - `workflow` — Workflow 引擎（Route/Parallel/Loop/Repeat）

pub mod absorb;
pub mod adapters;
pub mod blackboard;
pub mod cognitive_memory;
pub mod executor;
pub mod hooks;
pub mod persona;
pub mod skills;
pub mod step_generator;
pub mod sub_agent;
pub mod team;
pub mod tool;
pub mod tools;
pub mod workflow;
pub mod worktree;

pub mod bridge;
pub mod agent_bus;
pub mod decoder;
pub mod memory_optimizer;
pub mod playbook;

pub use absorb::AbsorbResult;
pub use executor::{Agent, AgentOutput, AgentStatus};
pub use skills::{
    Skill, SkillActivator, SkillDiscovery, SkillExecutor, SkillInjector, SkillMeta, SkillOutput,
    SkillSource, SkillsEngine,
};
pub use team::{AgentResult, AgentRole, AgentTeam, Coordinator, ProcessType};
pub use workflow::{Workflow, WorkflowEngine, WorkflowResult, WorkflowStep};

pub use agent_bus::{
    AgentBus, AgentBusMessage, BusStats, BusTask, BusTopic, SupervisorAgent, TaskClaim, TaskResult,
    WorkerAgent,
};
pub use hooks::{
    AgentHookRegistry, Hook, HookAction, HookContext, HookEvent, HookProfile, HookRegistry,
    QualityGateHook, SessionPersistenceHook,
};
pub use persona::{AgentPersona, AgentPersonaRegistry, ExperienceLevel, PersonaRole};
#[cfg(feature = "sandbox")]
pub use sub_agent::SandboxAgent;
pub use sub_agent::{
    SubAgentConfig, SubAgentEvent, SubAgentHandle, SubAgentPool, SubAgentResult, SubAgentStatus,
    SubAgentVariant,
};
pub mod agent_interface;
pub mod agent_workflow;
pub use agent_workflow::{AgentStep, AgentWorkflow, AgentWorkflowResult, PlanMode};
pub mod channel;
pub mod credit_assignment;
pub mod deps;
pub mod experience_pool;
pub mod proxy;
pub mod self_org;
pub mod tunnel;
