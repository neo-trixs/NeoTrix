//! Provider 目录 / 发现 / 注册
pub mod provider_catalog;
pub mod free_catalog;
pub mod registry;
pub mod discovery;
pub mod gateway_adapter;
pub mod model_pool;
pub mod cli_free_source; // 外部 CLI 免费模型发现源 — free 模型进池（opencode 仅为默认后端）
pub use provider_catalog::*;
pub use free_catalog::*;
pub use registry::*;
pub use discovery::*;
pub use gateway_adapter::GatewayV2Adapter;
pub use model_pool::{UnifiedModelPool, UnifiedModelEntry, ModelSource, LocalGgufSource, CloudFreeSource, LocalEndpointSource};
pub use cli_free_source::{CliFreeSource, parse_models_list, is_free_model_id, is_free_model_id_with, zen_base_url, FreebuffFreeSource, freebuff_base_url};
