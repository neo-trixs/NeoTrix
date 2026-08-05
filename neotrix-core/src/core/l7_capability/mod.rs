//! # L7 — 能力层 (Capability)
//!
//! 能力的注册、调度、成熟度进化、星脉通信协议。
//! 科幻映射: 《本书记载》星脉通信 / Diaspora Polis / Matrix Key Maker
//!
//! ## 规则
//! - L7 是层间通信的唯一路由层
//! - 每个能力必须通过 L7 注册后才能被系统发现
//! - 调度必经 4 道大过滤器：权限 → 预算 → 熔断 → 谦逊
//! - L7 不执行能力，只调度

pub mod registry;
pub mod protocol;
pub mod gate;
pub mod mature;
pub mod observer;
pub mod skill_acquire;
pub mod group_evolve;
pub mod nt_act_orch_patterns;
pub mod a2a;
pub mod nt_core_antidistil;

pub mod nt_cap_media;
pub mod nt_cap_geo;
pub mod nt_core_orch_agent;

pub use registry::*;
pub use protocol::*;
pub use gate::*;
pub use mature::*;
pub use observer::*;
pub use skill_acquire::{
    SkillDoc, SkillMetrics, SkillBank, BankStats,
    SkillExtractor, SkillOptimizer, ValidationGate, RejectedEditBuffer,
    BoundedEdit, EditType, Trajectory, TrajectoryStep, TrajectoryOutcome,
    SkillManifest, SkillRegistry, ProgressiveDisclosure, DisclosureLevel,
};
pub use group_evolve::{
    GroupCoordinator, GroupCoordinatorConfig, GroupCoordinatorStats,
    PeerState, PeerStatus, ConsensusProposal, ConsensusState, Vote,
};
pub use nt_core_orch_agent::{
    SubagentConfig, SubagentInstance, SubagentStatus, AgentMessage, MessageType,
    SubagentManager, AgentPoolStats,
};
