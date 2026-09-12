pub mod traits;
pub mod nt_act;
pub mod nt_io;
pub mod nt_media; // 统一媒体能力 (detect/router/streaming)
pub mod nt_io_download; // 下载引擎 (自研，无外部依赖)
pub mod nt_memory;
pub mod nt_memory_spatial; // moved from L2 (no L2 deps, spatial storage belongs in L1)

// L1 基础设施层
pub mod nt_infra_tracing;
pub mod nt_infra_breaker;
pub mod nt_infra_semantic_router; // moved from L2 (no L2 deps, pure L1 infrastructure)
pub mod nt_infra_agent_card;
pub mod nt_infra_scatter_gather;
pub mod nt_infra_persistence;
pub mod nt_infra_learning;
pub mod nt_infra_integration;
