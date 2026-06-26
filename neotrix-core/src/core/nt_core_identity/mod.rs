//! # NeoTrix Identity Core — 自身特性记忆与LLM定位剥离
//!
//! NeoTrix 的持久身份层。IdentityCore 存储自身特性记忆（self_vsa、人格特质、
//! 核心价值观），独立于任何 LLM。SelfReasoner 在 VSA 空间中进行内部推理
//! （不需要 LLM）。CoprocessorBridge 管理 LLM 作为外部协处理器的调用策略
//! 和经验反哺管道。
//!
//! ## 架构
//!
//! ```text
//! IdentityCore (持久身份)
//!   ├── self_vsa: 根身份向量 ("我是 NeoTrix")
//!   ├── personality_traits: 人格特质向量
//!   ├── core_values: 核心价值观
//!   └── persistence: save/load to .neotrix/identity/
//!
//! SelfReasoner (内部推理)
//!   ├── VSA 空间思考链
//!   ├── 置信度估计 → 决定是否调用外挂
//!   └── 无需 LLM 即可运行
//!
//! CoprocessorBridge (LLM 外挂)
//!   ├── 调用时机决策
//!   ├── prompt 构建
//!   ├── 响应解析 + 洞察提取
//!   └── 经验反哺蒸馏
//! ```

pub mod between_sessions;
mod coproc_bridge;
pub mod cvo_role;
mod identity_boundary;
mod identity_core;
mod identity_evolution;
mod inter_session;
pub mod persistent_context;
mod self_reasoner;
mod value_gate;

pub use coproc_bridge::{CoprocessorBridge, CoprocessorResponse, DistilledInsight};
pub use identity_boundary::{
    AuditHook, BoundaryContext, BoundaryError, BoundaryHook, BoundaryHookInstance, BoundaryManager,
    BoundaryOp, CoherenceGuardHook, DriftCheckHook,
};
pub use identity_core::{HysteresisMetrics, IdentityCore, IdentitySnapshot};
pub use identity_evolution::{IdentityEvolution, IdentityEvolutionConfig, IdentityVersion};
pub use inter_session::{InterSessionReflector, ReflectionReport};
pub use self_reasoner::{ReasonSource, ReasoningStep, SelfReasoner};
pub use value_gate::{InsightVerdict, ValueAlignmentGate};
