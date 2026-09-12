//! L2 Perception Layer - World Modules
//!
//! 6 目录架构:
//!   crawl/       — 爬虫类 (crawl + browse + browse_auto)
//!   osint/       — 情报类 (osint + absorber)
//!   sense/       — 感知类 (sense + jepa + model)
//!   explore/     — 探索类 (map + cleanup)
//!   source/      — 源类 (media_source)
//!   data_source/ — 外部数据源采集器 (edgar/gdelt/usgs/gdacs 等 12 个)

pub mod l1_facade;
pub mod crawl;
pub mod osint;
pub mod sense;
pub mod explore;
pub mod source;

// 向后兼容别名
pub use crawl as nt_world_crawl;
pub use osint as nt_world_osint;
pub use sense as nt_world_sense;
pub use explore as nt_world_map;
pub use source as nt_world_media_source;

// Stub modules (legacy types still referenced by downstream code)
pub mod nt_world_model;
pub mod nt_world_jepa;
pub mod nt_world_model_v2;

// 单文件模块
pub mod port_service;
pub mod nt_world_code_search;
pub mod nt_world_e8;
pub mod nt_world_github_absorber;
// 数据源子目录 (12 个外部数据源采集器)
pub mod data_source;

// 向后兼容别名 — 旧路径 nt_world::nt_world_xxx 仍可用
pub use data_source::nt_world_edgar;
pub use data_source::nt_world_gdelt;
pub use data_source::nt_world_usgs;
pub use data_source::nt_world_gdacs;
pub use data_source::nt_world_ucdp;
pub use data_source::nt_world_urlhaus;
pub use data_source::nt_world_ofac;
pub use data_source::nt_world_polymarket;
pub use data_source::nt_world_aoi;
pub use data_source::nt_world_adsb;
pub use data_source::nt_world_bgpview;
pub use data_source::nt_world_opencorporates;
pub mod nt_world_intel_selftest;
pub mod nt_world_infer;
pub mod nt_world_scrape;
pub mod nt_world_search;
pub mod nt_world_prefetch;
pub mod nt_world_doc;
pub mod nt_world_exploration_engine;
pub mod nt_world_video_pipeline;
pub mod nt_world_novel;
pub mod nt_world_ods;
pub mod nt_world_monitor;
pub mod nt_world_osint_arsenal;
pub mod nt_world_myip;
pub mod nt_world_dsh_explore;
pub mod nt_world_agent_reach;
pub mod nt_world_semantic_extract;

// 资产测绘系统
pub mod asset_map;

// NLP能力模块
pub mod nt_nlp_capability;

// 通用能力模块
pub mod media_asset_registry;

// 动态记忆库
pub mod dynamic_memory_bank;

// 向后兼容别名
pub use media_asset_registry::AssetRegistry;
