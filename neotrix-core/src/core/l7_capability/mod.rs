//! # L7 — 能力层 (Capability)
//!
//! 能力的注册、调度、成熟度进化、星脉通信协议。
//! 科幻映射: 《本书记载》星脉通信 / Diaspora Polis / Matrix Key Maker
//!
//! ## 规则
//! - L7 是层间通信的唯一路由层
//! - 每个能力必须通过 L7 注册后才能被系统发现
//! - 调度门控：预算由 L1 rate_limiter 承载，谦逊/幻觉由 nt_core_gate 护栏承载，
//!   成熟度由 Constellation 承载（原 GreatFilterGate 已归档，输入指标无生产源）
//! - L7 不执行能力，只调度

pub mod a2a;
pub mod group_evolve;
pub mod mature;
pub mod nt_act_orch_patterns;
pub mod nt_core_antidistil;
pub mod observer;
pub mod protocol;
pub mod registry;
pub mod skill_acquire;

pub mod cluster_self_test;
pub mod nt_cap_geo;
pub mod nt_cap_media;
pub mod nt_core_grounded_gate;
pub mod nt_core_orch_agent;

pub use cluster_self_test::CapabilityClusterSelfTest;
pub use group_evolve::{
    ConsensusProposal, ConsensusState, GroupCoordinator, GroupCoordinatorConfig,
    GroupCoordinatorStats, PeerState, PeerStatus, Vote,
};
pub use mature::*;
pub use nt_core_grounded_gate::{
    AgentContract, Domain, Field, FieldType, GroundedCheck, GroundedDecision, GroundedGate,
};
pub use nt_core_orch_agent::{
    AgentMessage, AgentPoolStats, MessageType, SubagentConfig, SubagentInstance, SubagentManager,
    SubagentStatus,
};
pub use observer::*;
pub use protocol::*;
pub use registry::*;
pub use skill_acquire::{
    BankStats, BoundedEdit, DisclosureLevel, EditType, ProgressiveDisclosure, RejectedEditBuffer,
    SkillBank, SkillDoc, SkillExtractor, SkillManifest, SkillMetrics, SkillOptimizer,
    SkillRegistry, Trajectory, TrajectoryOutcome, TrajectoryStep, ValidationGate,
};
