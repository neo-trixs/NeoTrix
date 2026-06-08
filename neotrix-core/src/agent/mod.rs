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

pub mod executor;
pub mod absorb;
pub mod adapters;
pub mod team;
pub mod workflow;
pub mod skills;
pub mod tools;
pub mod hooks;
pub mod sub_agent;
pub mod persona;
pub mod blackboard;
pub mod cognitive_memory;
pub mod step_generator;
pub mod worktree;

pub mod playbook;
pub mod decoder;
pub mod memory_optimizer;
pub mod agent_bus;

pub use executor::{Agent, AgentStatus, AgentOutput};
pub use absorb::AbsorbResult;
pub use team::{AgentTeam, AgentRole, AgentResult, ProcessType, Coordinator};
pub use workflow::{Workflow, WorkflowStep, WorkflowEngine, WorkflowResult};
pub use skills::{SkillsEngine, Skill, SkillMeta, SkillSource, SkillOutput, SkillDiscovery, SkillActivator, SkillExecutor, SkillInjector};

pub use hooks::{HookRegistry, Hook, HookEvent, HookContext, HookAction, HookProfile, SessionPersistenceHook, QualityGateHook};
pub use persona::{AgentPersona, AgentPersonaRegistry, PersonaRole, ExperienceLevel};
pub use sub_agent::{SubAgentPool, SubAgentConfig, SubAgentStatus, SubAgentResult, SubAgentHandle, SubAgentEvent, SubAgentVariant};
#[cfg(feature = "sandbox")]
pub use sub_agent::SandboxAgent;
pub use agent_bus::{AgentBus, AgentBusMessage, BusTopic, BusTask, TaskClaim, TaskResult, BusStats, SupervisorAgent, WorkerAgent};
pub mod agent_interface;
