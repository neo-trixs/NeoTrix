pub mod session_lifecycle;
pub mod worktree_manager;
pub mod session_manager;
pub mod spawn_flow;
pub mod task_execution;
pub mod orchestrator_agent;
pub mod arbiter_mediation;
pub mod expert_team_diff;
pub mod plugin_harness;

pub use session_lifecycle::{SessionState, AgentSession};
pub use worktree_manager::{WorktreeManager, Worktree};
pub use session_manager::{SessionManager, SessionConfig};
pub use spawn_flow::{SpawnManager, SpawnConfig, SpawnResult};
pub use orchestrator_agent::{OrchestratorAgent, AgentRole, WorkerStatus};
pub use arbiter_mediation::{
    ArbiterMediator, MediationIssue, MediationReport, AgentJudgement, ArbitrationVerdict,
    MediationFinding,
};
pub use expert_team_diff::{ExpertTeamWriter, DiffEntry, DiffStatus, DiffError};
pub use plugin_harness::{PluginHarness, PluginSpec, PluginStatus, PluginLifecycle, SimplePlugin};
