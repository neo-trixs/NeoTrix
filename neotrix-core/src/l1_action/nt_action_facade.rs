#![forbid(unsafe_code)]

//! L1 Action Facade — 行动层唯一对外门面
//!
//! Platform shells and upper layers only talk to this facade.
//! Internal modules (`nt_memory`, `nt_act`, `nt_io`) are not directly accessible.

use std::sync::Arc;

use crate::l1_action::nt_act::async_tool_executor::AsyncToolExecutor;
use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_store;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
use crate::l1_action::traits::LlmRouter;

// ════════════════════════════════════════════════════════════════
// L5 跨层引用收敛点（2026-10-03 修「跨域错位 + 冗余」双重缺陷）
// ════════════════════════════════════════════════════════════════
//
// **实测事实**（`bash scripts/check-layer-deps.sh` ⇒ 19 处已记录违规）：
// **7 个 L1 文件**直接 `use crate::l5_cognition::nt_crystal_core::…`，
// 其中 **3 个文件的 import 行字节完全相同**：
//   · `nt_model_cli.rs:26`         use …::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
//   · `nt_free_pool.rs:23`         （同上，逐字节一致）
//   · `nt_crystal_llm_bridge.rs:32`（同上，逐字节一致）
//
// ⇒ 同时是**跨域错位**（L1 越过 L2–L4 直取 L5）与**冗余**（同一条 import 抄三遍）。
//
// **为什么走本 facade 而不是就地改**：本文件是 L1 自己的门面，
// `check-layer-deps.sh` 明确把 facade bridge 当作 sanctioned channel 排除
// ⇒ 在此处转出是**该门认可的唯一合法通道**，而在 3 个消费方直接写
// `crate::l1_action::nt_action_facade::…` 同样不含层名字样。
//
// ⛔ 刻意**只转出这 5 个符号**，不整包 re-export `nt_crystal_core`：
// 整包转出等于给 L1 开了一扇通往后端的门，跨域错位只是换了个门牌号。
pub use crate::l5_cognition::nt_crystal_core::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
// **人机通道簇**（2026-10-03 第 2 批）。同样是 `nt_crystal_core` 的跨层引用，
// 但**单独成簇**，因为它与上面的「LLM 问答契约」是**两种不同性质的能力**：
// · 上面 3 个 = 「怎么问模型」⇒ L1 作为**调用方**
// · 下面 4 个 = 「怎么问人」⇒ L1 作为**通道提供方**（`nt_stdin_human` /
//   `nt_dialogue_tui` 正是 L1 的人类交互界面）
// ⇒ 混在一处会让「L1 何时依赖认知层」这个问题失去可读性。
//
// ⛔ **刻意不在此转出引擎簇**（`CrystalCore` / `NtCrystalTaskLoop` /
// `NtTaskLoopConfig` / `NtTaskLoopReport` / `NtProgressSink`，`nt_tui_app.rs` +
// `nt_dispatcher_core.rs` 需要）。理由：转出它们等于宣告「**L1 驱动认知引擎**」，
// 这是一个**架构立场**，不是机械去重，须单独裁决（是否该把 `nt_crystal_core`
// 重新分层？），⛔ 不在本批顺手决定。
pub use crate::l5_cognition::nt_crystal_core::{
    NtDemand, NtDemandKind, NtHumanChannel, NtHumanReply,
};

// ════════════════════════════════════════════════════════════════
// Public types
// ════════════════════════════════════════════════════════════════

/// A single search hit returned by the facade.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub score: f64,
    pub snippet: String,
}

/// Health snapshot for the entire L1 Action layer.
#[derive(Debug, Clone)]
pub struct LayerHealth {
    /// Aggregate health score in `[0.0, 1.0]`.
    pub score: f64,
    /// Names of sub-modules and their status.
    pub modules: Vec<String>,
}

/// Errors that can surface through the facade boundary.
#[derive(Debug)]
pub enum FacadeError {
    /// The KB search path failed.
    SearchFailed(String),
    /// The KB store path failed.
    StoreFailed(String),
    /// The facade has not been initialized with valid internals.
    NotInitialized,
}

impl std::fmt::Display for FacadeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchFailed(msg) => write!(f, "search failed: {msg}"),
            Self::StoreFailed(msg) => write!(f, "store failed: {msg}"),
            Self::NotInitialized => write!(f, "facade not initialized"),
        }
    }
}

impl std::error::Error for FacadeError {}

impl From<FacadeError> for neotrix_types::NtError {
    fn from(err: FacadeError) -> Self {
        match err {
            FacadeError::SearchFailed(msg) => neotrix_types::NtError::OperationFailed(msg),
            FacadeError::StoreFailed(msg) => neotrix_types::NtError::OperationFailed(msg),
            FacadeError::NotInitialized => neotrix_types::NtError::InvalidState("facade not initialized".into()),
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Configuration (internal — not exposed in the public API)
// ════════════════════════════════════════════════════════════════

/// Builder input for `ActionFacade`.
pub struct ActionFacadeConfig {
    pub kb: Arc<KnowledgeBase>,
    pub tool_executor: Arc<AsyncToolExecutor>,
    pub llm_router: Arc<dyn LlmRouter>,
}

// ════════════════════════════════════════════════════════════════
// ActionFacade
// ════════════════════════════════════════════════════════════════

/// The sole facade for L1 Action layer access.
///
/// Platform shells and upper layers talk **only** to this struct.
/// Internal modules (`nt_memory`, `nt_act`, `nt_io`) remain hidden.
pub struct ActionFacade {
    kb: Option<Arc<KnowledgeBase>>,
    tool_executor: Option<Arc<AsyncToolExecutor>>,
    llm_router: Option<Arc<dyn LlmRouter>>,
}

impl ActionFacade {
    /// Construct an uninitialized facade.
    ///
    /// Call `init()` with real components before using any method.
    pub fn new() -> Self {
        Self {
            kb: None,
            tool_executor: None,
            llm_router: None,
        }
    }

    /// Inject real internal components.
    pub fn init(&mut self, config: ActionFacadeConfig) {
        self.kb = Some(config.kb);
        self.tool_executor = Some(config.tool_executor);
        self.llm_router = Some(config.llm_router);
    }

    // ── Search ──────────────────────────────────────────────────

    /// Search the knowledge base.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, FacadeError> {
        let kb = self.kb.as_ref().ok_or(FacadeError::NotInitialized)?;

        // 2026-09-27 除根 (自死锁): rebuild_bm25 内部自锁 kb.conn, 必须在此之前
        // 完成。原顺序先 raw_conn() 持守卫再 rebuild_bm25() → 同一线程二次锁
        // 非重入 Mutex → 搜索永久挂死, 并连带阻塞所有 KB 操作 (栈实证
        // __psynch_mutexwait @ kb_core.rs:343)。
        kb.rebuild_bm25();

        let conn = kb
            .raw_conn()
            .map_err(|e| FacadeError::SearchFailed(format!("KB lock: {e}")))?;

        let mut results = Vec::new();

        // BM25 path (always available)
        if let Ok(bm25_guard) = kb.bm25.read() {
            if let Some(ref bm25) = *bm25_guard {
                // 2026-09-27 修正: Bm25Index::search 返回 (score, doc_id) ——
                // 原先把 doc_id 直接当 title 返回, 搜索结果标题全是 id。
                // 按 id 回表取真实 title, 回表失败才退回 id。
                for (score, doc_id) in bm25.search(query, 10) {
                    let title =
                        nt_memory_store::get_node(&conn, &doc_id).ok().flatten().map_or_else(
                            || doc_id.clone(),
                            |node| node.title,
                        );
                    results.push(SearchResult {
                        title,
                        score,
                        snippet: String::new(),
                    });
                }
            }
        }

        // Fallback to LIKE search
        if results.is_empty() {
            let like_pattern = format!("%{query}%");
            let mut stmt = conn
                .prepare(
                    "SELECT title, COALESCE(summary, '') FROM nodes
                     WHERE title LIKE ?1 OR content LIKE ?1
                     ORDER BY updated_at DESC LIMIT 10",
                )
                .map_err(|e| FacadeError::SearchFailed(format!("stmt: {e}")))?;

            let rows = stmt
                .query_map([&like_pattern], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|e| FacadeError::SearchFailed(format!("query: {e}")))?;

            for row in rows.flatten() {
                results.push(SearchResult {
                    title: row.0,
                    score: 1.0,
                    snippet: row.1,
                });
            }
        }

        Ok(results)
    }

    // ── Store ───────────────────────────────────────────────────

    /// Store a knowledge node. Returns the node ID on success.
    pub fn store(
        &self,
        title: &str,
        node_type: &str,
        summary: &str,
    ) -> Result<String, FacadeError> {
        let kb = self.kb.as_ref().ok_or(FacadeError::NotInitialized)?;

        let node_id = kb
            .insert_or_get_node(title, neotrix_types::knowledge_access::NodeType::from_str(node_type), Some(summary), None, None)
            .map_err(|e| FacadeError::StoreFailed(format!("{e}")))?;

        Ok(node_id)
    }

    // ── Health ─────────────────────────────────────────────────

    /// Return a health snapshot for the L1 layer.
    pub fn health(&self) -> LayerHealth {
        let mut modules = Vec::new();
        let mut healthy_count = 0u32;
        let mut total = 0u32;

        // nt_memory
        total += 1;
        if self.kb.as_ref().map_or(false, |kb| kb.raw_conn().is_ok()) {
            modules.push("nt_memory:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_memory:down".into());
        }

        // nt_act
        total += 1;
        if self.tool_executor.is_some() {
            modules.push("nt_act:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_act:down".into());
        }

        // nt_io
        total += 1;
        if self
            .llm_router
            .as_ref()
            .map_or(false, |r| r.health_check().healthy)
        {
            modules.push("nt_io:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_io:down".into());
        }

        let score = if total == 0 {
            0.0
        } else {
            healthy_count as f64 / total as f64
        };

        LayerHealth { score, modules }
    }
}

impl Default for ActionFacade {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ActionFacade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionFacade")
            .field("initialized", &self.kb.is_some())
            .finish()
    }
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_act::async_tool_executor::AsyncToolExecutor;

    use crate::l1_action::traits::{
        CapabilityCategory, CapabilityHealth, ConstellationLevel, L1Capability, LlmRequest,
        LlmRoute,
    };

    struct DummyRouter;

    impl L1Capability for DummyRouter {
        fn capability_id(&self) -> &str {
            "test.router"
        }
        fn category(&self) -> CapabilityCategory {
            CapabilityCategory::Cognition
        }
        fn constellation(&self) -> ConstellationLevel {
            ConstellationLevel::C0Compiled
        }
        fn health_check(&self) -> CapabilityHealth {
            CapabilityHealth::default()
        }
        fn description(&self) -> &str {
            "Dummy router for tests"
        }
    }

    impl LlmRouter for DummyRouter {
        fn route(
            &self,
            _request: &LlmRequest,
        ) -> Result<LlmRoute, crate::l1_action::traits::CapabilityError> {
            Ok(LlmRoute {
                provider: "test".into(),
                model: "dummy".into(),
                estimated_cost: 0.0,
            })
        }
        fn providers(&self) -> Vec<String> {
            vec!["test".into()]
        }
    }

    fn test_facade() -> ActionFacade {
        let kb = Arc::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).expect("in-memory KB"),
        );
        let executor = Arc::new(AsyncToolExecutor::new());
        let router = Arc::new(DummyRouter);

        let mut facade = ActionFacade::new();
        facade.init(ActionFacadeConfig {
            kb,
            tool_executor: executor,
            llm_router: router,
        });
        facade
    }

    #[test]
    fn new_is_uninitialized() {
        let f = ActionFacade::new();
        assert!(f.kb.is_none());
        assert!(f.health().score < 0.5);
    }

    #[test]
    fn search_empty_kb_returns_empty() {
        let facade = test_facade();
        let results = facade.search("anything").expect("search should not fail");
        assert!(results.is_empty());
    }

    #[test]
    fn store_and_search() {
        let facade = test_facade();
        let id = facade
            .store("Rust Facade Pattern", "concept", "L1 action layer facade")
            .expect("store");
        assert!(!id.is_empty());

        let results = facade.search("Facade").expect("search");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title == "Rust Facade Pattern"));
    }

    #[test]
    fn health_all_healthy() {
        let facade = test_facade();
        let h = facade.health();
        assert!((h.score - 1.0).abs() < f64::EPSILON);
        assert_eq!(h.modules.len(), 3);
        assert!(h.modules.iter().all(|m| m.ends_with(":ok")));
    }

    #[test]
    fn uninitialized_returns_not_initialized() {
        let facade = ActionFacade::new();
        assert!(matches!(
            facade.search("q"),
            Err(FacadeError::NotInitialized)
        ));
        assert!(matches!(
            facade.store("t", "n", "s"),
            Err(FacadeError::NotInitialized)
        ));
    }

    #[test]
    fn facade_error_display() {
        let e = FacadeError::SearchFailed("boom".into());
        assert_eq!(e.to_string(), "search failed: boom");

        let e = FacadeError::StoreFailed("oops".into());
        assert_eq!(e.to_string(), "store failed: oops");

        let e = FacadeError::NotInitialized;
        assert_eq!(e.to_string(), "facade not initialized");
    }

    #[test]
    fn search_result_is_clone() {
        let sr = SearchResult {
            title: "t".into(),
            score: 0.5,
            snippet: "s".into(),
        };
        let sr2 = sr.clone();
        assert_eq!(sr.title, sr2.title);
    }
}

// ─── L1→L3/L6 跨层引用收敛点（分层门 sanctioned channel）──────────────────────
// 本文件此前**只有内部 `use`、0 条 `pub use`**，故 L1 业务文件一律直引
// `crate::l3_embodiment::…` / `crate::l6_meta::…`，层名出现在业务代码里 ⇒
// 门记违规。走目标层 facade 无效（路径仍含层名），必须经**本层**门面。
//
// 路径一律沿用消费方**原本就在用**的路径（原代码已能编译 ⇒ 路径可证），
// 不重新定位定义处。ALLOW: 类型/函数直访，trait-object 不可行
// （同 l5_cognition/l1_facade.rs:125-128 的既有说明）。
// ⚠️ 本段按目标层**逐批追加**（先 L3/L6，再 L2/L4/L5），每批单独编译验证。
pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net::config::{reload, snapshot};
pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net::proxy_pool::ProxyPool;
pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net::rules::RuleEngine;
pub use crate::l3_embodiment::nt_shield_enforcer::global_shield;
pub use crate::l3_embodiment::nt_shield::guard::agent_guardrails::{
    GuardrailContext, GuardrailResult, GuardrailVerdict, PolicyEngine, ViolationSeverity,
};
pub use crate::l6_meta::nt_approval;
pub use crate::l6_meta::nt_approval::PendingAction;

// ─── L1→L2 收敛点（第二批；`check-layer-deps.sh` 规则是「层不得引用更高层」）──
//
// L2 批次的存在理由与上面 L3/L6 相同：`nt_model_cli.rs::capture_model_command`
// 直引 `crate::l2_perception::nt_world::social_access::probe::run_with_timeout`，
// 而门规定 **L1 不得引用 L2/L3/L4/L5/L6** ⇒ 该行记 `NEW layer violation`。
//
// 为何必须走**本层** facade（AGENTS.md §4.2）：改用「目标层的 facade」无效，
//   因为路径仍含层名，门照样命中。唯一合法通道是消费方自己那层的门面。
//
// ⛔ 不选替代方案「把 `capture_model_command` 整体搬进 L2」：那是**语义变更**
//   而非接线 —— 该函数是「模型询问」特有语义（去 ANSI / 空 stdout 判失败 /
//   超时与失败可区分，见其 doc 的「本文件唯一的进程执行出口」三职责），
//   搬层会牵动 `run_once` 与 `run_capture` 两个调用方，
//   而 `nt_model_cli.rs` 头注释正记着 2026-10-03「删转发层前没查全部调用方
//   ⇒ 把主路径一起打断」的教训。⇒ 本次只做**路径收敛**，不动语义。
//
// ✅ 可转安全性已核实：`RunOutcome`（`probe.rs:116`）是 `pub struct`
//   且字段全 `pub`（`success`/`stdout`/`stderr`/`latency`/`timed_out`）
//   ⇒ 无私有类型泄漏，facade 转出是合法的。
pub use crate::l2_perception::nt_world::social_access::probe::run_with_timeout;
pub use crate::l2_perception::nt_world::social_access::probe::RunOutcome;

// ─── L1→L5 第二批：`ConsciousnessRuntime` ──────────────────────────────────────
//
// `check-layer-deps.sh:89` 的 `check_layer "l1_action" … "l5_cognition" …`
// ⇒ L1 不得引用 L5。两条违规点**都在 `#[cfg(test)] mod tests` 内**：
//   · `l1_action/nt_capability_bridge.rs:640`
//   · `l1_action/nt_act/nt_act_trade/capability_registry.rs:1130`
// 两侧形态完全相同：`ConsciousnessRuntime::new()` 绑到 `_rt` 做播种。
//
// ⚠️ 为什么不改门去排除 `#[cfg(test)]`：门脚本 `:70` 明写
//   「string literals still count (conservative: **may over-report, never under-**)」
//   ⇒ 过报是**有意的保守设计**；放宽它会削弱门的捕获面，
//   而本次两条违规**是真的**（只是恰好在测试区）。⇒ 改引用路径，不改门。
//
// ✅ 可转安全性已核实：`ConsciousnessRuntime`（`consciousness_runtime.rs:64`）
//   是 `pub struct`，`new()`（`:111`）是 `pub fn` 且无参数 ⇒ 完整类型可转出。
//
// ⛔ 注意与本文件既有 `NtLlmAsk`/`NtTaskFusionError` 转出的区别：那些是
//   **生产**依赖 L5；本次两处是**测试**依赖 L5。走 facade 后层名不再出现在
//   业务文件里，两种情形一并收敛。
pub use crate::l5_cognition::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime;

// ─── L1→L4 跨层引用收敛（接上批 L3/L6 段）────────────────────────────────────
// 路径沿用消费方原本就在用的路径（原代码已能编译 ⇒ 路径可证）。
// ⚠️ `nt_memory` 与 `nt_feel_facade` 是**模块型**引用（L1 门面需要它们做
// re-export 出口），故按模块整体转出；`InteractionType::Inquiry` /
// `LeadStage` / `KnowledgeBase::open` 等是类型上的关联项，类型转出后即可用。
pub use crate::l4_emotion::nt_feel_facade;
pub use crate::l4_emotion::nt_memory;
pub use crate::l4_emotion::nt_memory::addressable_store::AddressableStore;
pub use crate::l4_emotion::nt_memory::nt_memory_historian::{
    build_ewhr_router, EvidenceApiState,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_api::{
    build_kb_router, KbApiState,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_geo::query_bbox_with_cold;
pub use crate::l4_emotion::nt_memory::nt_memory_lead;
pub use crate::l4_emotion::nt_memory::nt_memory_lead::{InteractionType, LeadStage};

// ─── L1→L2 跨层引用收敛 ──────────────────────────────────────────────────────
// ⚠️ 模块型与类型型引用**都要查**（L4 子批与上上批各踩过一次 `E0432`）。
// 本段涉及的 4 个 l2 **模块**（`nt_core_sense` / `nt_core_llm` /
// `nt_core_code_search` / `nt_world::nt_world_mirror`）与 `nt_world::source` 均按
// **模块整体**转出，消费方以 `facade::模块::…` 访问其下项。
pub use crate::l2_perception::nt_core_code_search;
pub use crate::l2_perception::nt_core_knowledge::{
    publish_affective_observation, AffectiveFeedback, KnowledgeSource, TaskType,
};
pub use crate::l2_perception::nt_core_llm;
pub use crate::l2_perception::nt_core_sense;
pub use crate::l2_perception::nt_core_e8::nt_multimodal::model_supports_vision;
pub use crate::l2_perception::nt_core_e8::nt_multimodal::VisionBridge;
pub use crate::l2_perception::nt_world::nt_world_mirror;
pub use crate::l2_perception::nt_world::source;
pub use crate::l2_perception::nt_world::source::engine::MediaSource;

// ─── L1→L5 跨层引用收敛 ──────────────────────────────────────────────────────
// 路径沿用消费方原本就在用的路径（原代码已能编译 ⇒ 路径可证）。
pub use crate::l5_cognition::l1_facade::affective_interface::{
    AffectiveInterface, AffectiveReadout, GuideMode, ResponseIntent,
};
pub use crate::l5_cognition::l1_facade::self_audit::ToolGroundingMonitor;
pub use crate::l5_cognition::nt_core::capability::nt_core_antidistil::decompose::{
    DecomposeSuggestion, TaskDecomposer,
};
pub use crate::l5_cognition::nt_core::capability::types::CapabilityVector;
pub use crate::l5_cognition::nt_core::nt_crt::{CrtPlan, CrtTimeScale};
pub use crate::l5_cognition::nt_core_consciousness_tree::{ConsciousnessTree, NodeSnapshot};
pub use crate::l5_cognition::nt_core_context::revertible::{ClosureEffect, RevertibleContext};
pub use crate::l5_cognition::nt_core_cot_generator::{CoTGenerator, CoTOutput, DefaultCoTGenerator};
pub use crate::l5_cognition::nt_core_policy::E8Policy;
pub use crate::l5_cognition::nt_core_walsh::WalshMemoryIndex;
pub use crate::l5_cognition::reasoning_core::{ReasoningTrace, TraceSource};
