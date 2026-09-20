//! 数据源模块 — 情报类外部数据源采集器 (12 个)
//!
//! 包含 EDGAR、GDELT、USGS、GDACS、UCDP、URLhaus、OFAC、Polymarket、AOI、
//! ADS-B、BGPView、OpenCorporates 等公开情报数据源的采集与解析。
//!
//! 注意: 学术/技术/新闻类数据源已整合到 source/ 模块统一管理，
//! 避免与 MediaSource 架构重复。

// ── 情报类 (12) ──────────────────────────────────────────────
pub mod nt_world_edgar;
pub mod nt_world_gdelt;
pub mod nt_world_usgs;
pub mod nt_world_gdacs;
pub mod nt_world_ucdp;
pub mod nt_world_urlhaus;
pub mod nt_world_ofac;
pub mod nt_world_polymarket;
pub mod nt_world_aoi;
pub mod nt_world_adsb;
pub mod nt_world_bgpview;
pub mod nt_world_opencorporates;
