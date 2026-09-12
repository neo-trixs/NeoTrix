//! 路由子模块 — 智能路由、能力路由、子网格、免费 provider 选择

// ── 智能路由 ──────────────────────────────────────────────
pub mod intelligence;
pub use intelligence::*;

mod learned_router;

pub mod market_router;
pub use market_router::*;

pub mod routing_utils;
pub use routing_utils::*;

mod selection;

mod subgrid;

// ── 能力路由 & Agent 路由 ────────────────────────────────
mod capability_router;
pub use capability_router::CapabilityRouter;

mod agent_routing;
pub use agent_routing::{AgentRoutingTable, ProviderProfile, ProviderProfileManager};

mod inference_router;
pub use inference_router::{InferenceRouter, RouterConfig};

// ── 搜索 & 免费 provider ─────────────────────────────────
pub mod search_router;
pub use search_router::*;

pub mod free_providers;
pub use free_providers::*;
