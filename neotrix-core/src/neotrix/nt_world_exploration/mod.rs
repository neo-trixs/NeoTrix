//! # nt_world_exploration — 统一外部探索域
//!
//! 架构:
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │  ExplorationOrchestrator                                 │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
//! │  │ Browser  │ │   API    │ │  Search  │ │   File   │  │
//! │  │ Source   │ │  Source  │ │  Source  │ │  Source  │  │
//! │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
//! │       └────────────┼────────────┼────────────┘          │
//! │                    ▼            ▼                        │
//! │           ┌─────────────────────────┐                   │
//! │           │  NegentropyPipeline     │                   │
//! │           │  is_novel → purity →    │                   │
//! │           │  info_gain → rank       │                   │
//! │           └────────────┬────────────┘                   │
//! │                        ▼                                │
//! │              NegentropyScore > 0.25                     │
//! │                        → 注入意识核心                    │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! 反馈闭环:
//! - 好奇心 → 决定哪个源被调度
//! - 知识缺口 → seed_from_gaps() → API/Search 自动补查询
//! - 吸收 → 更新 known_concepts → 影响后续信息增益计算

pub mod browsing_agent;
pub mod content;
pub mod integration;
pub mod orchestrator;
pub mod pipeline;
pub mod shortcut_detector;
pub mod source_trait;
pub mod sources;
pub mod tool_discovery;
pub mod tree_seeker;

pub use content::{Engagement, ExplorationSourceType, NegentropyScore, SourceContent};
pub use integration::{
    exploration_curiosity_cycle, exploration_urgency, seed_orchestrator_from_curiosity,
};
pub use orchestrator::{ExplorationOrchestrator, OrchestratorStats, ScheduleDecision};
pub use pipeline::NegentropyPipeline;
pub use shortcut_detector::{ShortcutDetector, ShortcutSignal, ShortcutStats, ShortcutType};
pub use source_trait::ExplorationSource;
pub use tool_discovery::{
    DiscoveryAttempt, DiscoveryRequest, DiscoveryStats, SemanticToolRouter, ToolCapability,
    ToolDiscoveryEngine,
};
pub use tree_seeker::{
    BranchStatus, TreeSeekerBranch, TreeSeekerConfig, TreeSeekerManager, TreeSeekerStats,
};
