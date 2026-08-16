//! # NT-CORE Data Pipeline — Unified Data Lifecycle Orchestrator
//!
//! Manages the full lifecycle of data flowing through NeoTrix:
//!
//! ```text
//! ACQUIRE ──► NORMALIZE ──► STORE ──► DISTILL ──► INDEX ──► CONSUME
//!   │            │            │          │           │          │
//!   │ crawlers   │ parsers    │ KB       │ pattern   │ search   │ engine_core
//!   │ scrapers   │ validators │ pool     │ discovery │ embed    │ GWT
//!   │ APIs       │ dedup      │ encrypt  │ evolution │ FTS5     │ SEAL
//! ```
//!
//! ## Architecture references
//!
//! - **Agentic Lakehouse** (Dremio, 2026): Single governed store + semantic layer
//!   for autonomous AI agents. NeoTrix KB serves the same role as Iceberg + Polaris.
//! - **LLM-Agent-UMF** (Hassouna et al., 2024): Modular agent decomposition into
//!   profile/memory/planning/action/security. Our pipeline stages map to memory+action.
//! - **Data Agent Architecture** (Sun et al., 2025): Explicit pipeline phases
//!   (perception → reasoning → planning → execution → reflection). Our stages map
//!   to perception (acquire) → reasoning (normalize) → execution (store).
//!
//! ## Usage
//!
//! ```ignore
//! let mut pipeline = PipelineOrchestrator::new("resource-lifecycle");
//! pipeline.register(Box::new(ProxyScraperAcquireStage::new()));
//! pipeline.register(Box::new(ProxySubscriptionAcquireStage::new()));
//! pipeline.register(Box::new(ResourceNormalizeStage::new()));
//! pipeline.register(Box::new(ProxyStoreStage::new(global_pool())));
//! let report = pipeline.run("proxies").await;
//! ```
//!
//! ## Built-in pipelines
//!
//! - `create_proxy_pipeline()` — proxy discovery → normalize → dedup → pool
//! - `create_llm_provider_pipeline()` — LLM discovery → validate → gateway
//! - `create_knowledge_pipeline()` — GitHub/ArXiv/Wiki → absorb → KB → distill

mod lineage;
mod pipeline;

pub use lineage::{DataLineage, LineageEntry};
pub use pipeline::{PipelineOrchestrator, PipelineRunReport, PipelineStage, StageResult};
