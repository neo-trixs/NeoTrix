//! L1 Facade — L2 感知层对 L1 共享类型的 re-export 门面
//!
//! L2 感知层通过此模块访问 L1 共享类型，避免散布 `use crate::l1_action::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

pub use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::NodeType;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_discovery_github_topics::DiscoveryPipelineConfig;
pub use neotrix_types::knowledge_access::RelationType;

// HTTP 集中门面 — nt_http 类型与函数
// NOTE: download_to_file, shared_blocking_client, run_blocking 在 nt_http 中为 pub(crate)
//       因此这里也必须用 pub(crate) re-export，不能 pub use。
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::DownloadOptions;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::download_to_file;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::shared_blocking_client;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::run_blocking;

// HTTP 工厂门面 — proxy_from_env + module re-export
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;
pub use crate::l1_action::nt_io::nt_io_http_factory as http_factory;

// Core edit types (MicroEdit, SelfEdit)
pub use crate::l1_action::nt_core_edit::{MicroEdit, SelfEdit};

// Core bank types (ReasoningBank, memory tier/lifecycle)
pub use crate::l1_action::nt_core_bank::{
    ReasoningMemory, T3Views, MemoryTier, MemoryLifecycle, ReasoningBank,
};

// Media types
pub use crate::l1_action::nt_media::detect::MediaKind;
pub use crate::l1_action::nt_media::{
    PipelineConfig, PipelineProgress, PipelineStatus, StreamingPipeline,
};
pub use crate::l1_action::nt_media::playback::{
    EnginePlaybackState, PlaybackController, PlaybackEngine, PlaybackHistory, PlaybackQueue,
    PlaybackRetry, PlaybackState, PlayMode, RepeatMode,
};

// Universal browser types
pub use crate::l1_action::nt_io::universal_browser::{UniversalBrowser, PlatformConfig};

// Voice types
pub use crate::l1_action::nt_act::nt_act_voice::{VoiceInput, VoiceSample};

// LLM provider types (from nt_io_provider)
pub use crate::l1_action::nt_io::nt_io_provider::{
    LlmProvider as IoLlmProvider, LlmRequest as IoLlmRequest, LlmProviderType,
};
pub use crate::l1_action::nt_io::nt_io_provider::common::factory::create_provider_from_type;

// Core LLM types (backward-compat re-exports from neotrix_types)
pub use crate::l1_action::nt_io::nt_io_llm::{Message, Role, FinishReason, DataTrust};
pub use crate::l1_action::nt_io::nt_io_provider::{LlmResponse, LlmError, Usage};

// 共享出口类型门面 — EgressRule/EgressPolicy (避免 L2→L3 向上依赖)
pub use crate::l1_action::nt_io::nt_io_provider::common::egress_types::{
    SandboxEgressRule as EgressRule,
    SandboxEgressPolicy as EgressPolicy,
};

// ── KnowledgeStore trait — 打断 L2→L1 KnowledgeBase 直接依赖 ──────
//
// L2 数据源只需 KnowledgeBase 的读写子集，通过此 trait 解耦。
// 实现留在 L1 facade，测试代码仍可直接用 KnowledgeBase concrete type。

pub use neotrix_types::knowledge_access::KnowledgeNode;

/// L2 感知层对 KB 的最小读写接口 — 数据源入库只依赖此 trait，不依赖 KnowledgeBase concrete type。
pub trait KnowledgeStore: Send + Sync {
    /// 按 URL 查找节点 (去重用)。
    fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String>;
    /// 插入或复用节点 (幂等写入)。
    fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String>;
}

impl KnowledgeStore for KnowledgeBase {
    fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String> {
        KnowledgeBase::find_node_by_url(self, url)
    }
    fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        KnowledgeBase::insert_or_get_node(self, title, node_type, summary, url, domain)
    }
}

// ─── L2→L3/L4/L5 跨层引用收敛点（分层门 sanctioned channel）──────────────────
// 以下符号此前由 L2 业务文件（data_source 15 个 + crawl/osint/e8/knowledge 等）
// **直引** `crate::l3_embodiment::…` / `l4_emotion::…` / `l5_cognition::…`，
// 层名出现在业务代码里 ⇒ 门记违规。走目标层 facade 无效（路径仍含层名），
// 必须经**本层**门面。
//
// 路径一律沿用消费方**原本就在用**的路径（原代码已能编译 ⇒ 路径可证）。
// ALLOW: 类型/函数直访，trait-object 不可行（同本文件 :64-71 KnowledgeStore
// 解耦注释一脉相承 —— 长期方向是 trait 化，本段是过渡收敛点）。
//
// ⚠️ `EgressRule` / `KnowledgeBase` / `NodeType` **本文件已导出**（:60/:6/:7），
//    故消费方直接改道即可。三处同源已核实：
//    · `EgressRule` = l1 `egress_types.rs:12 SandboxEgressRule`；
//      l3 `nt_shield_sandbox/mod.rs:75` 同样 `SandboxEgressRule as EgressRule`
//      ⇒ **同一类型**，改道类型安全（不换类型身份）。
//    · `NodeType`：`nt_memory_kb/mod.rs:109 pub use nt_memory_types::*`
//      ⇒ `nt_memory_kb::NodeType` 与 `nt_memory_kb::nt_memory_types::NodeType` 同源。
pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net::{Response, StealthHttpClient};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::fetch_safe_http;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_crawl::{
    discover_from_seed, extract_html_content, extract_links, run_crawl_cycle,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_store::{
    count_nodes, count_nodes_by_type_map, upsert_crawl_queue,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel;
pub use crate::l5_cognition::nt_core_prm::{AgentTrajectory, TrajectoryStep};
pub use crate::l5_cognition::nt_core::capability::types::CapabilityVector;
pub use crate::l5_cognition::nt_core_cad_consciousness::cad_experience_payload;
pub use crate::l5_cognition::nt_core_td;
pub use crate::l5_cognition::nt_core_ttc;
