pub mod traits;
pub mod nt_act;
pub mod nt_io;
pub mod nt_io_download; // 下载引擎 (自研，无外部依赖)
pub mod nt_memory;

// L1 基础设施层
pub mod nt_infra_tracing;
pub mod nt_infra_breaker;
pub use crate::l2_perception::nt_sense::nt_infra_semantic_router;
pub mod nt_infra_agent_card;
pub mod nt_infra_scatter_gather;
pub mod nt_infra_persistence;
pub mod nt_infra_learning;
pub mod nt_infra_integration;
