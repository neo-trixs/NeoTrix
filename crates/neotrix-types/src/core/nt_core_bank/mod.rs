//! # 定位（2026-09-30 加注，消除「该用哪个」的歧义）
//!
//! 本模块是 **`neotrix-types` 的低层契约库**；上层实现在
//! `neotrix-core/src/l1_action/nt_core_bank`（体量同为 ~156 KB）。
//!
//! ⛔ **两者不是镜像冗余，不要试图合并或删除**。实测取证
//! （`docs/architecture/MIRROR-BANK-2026-09-30.md`）：
//! 99 个同名函数里 **26 个实现不同**、types 独有 **51 个函数**（core 全仓无同名）
//! 与 **39 个 pub 类型** ⇒ 强制合并会**丢失能力**。
//!
//! **依赖方向**：`neotrix-core` 依赖 `neotrix-types`（`Cargo.toml:97`），
//! 所以二者是**基类与实现**，不是同一物的两份拷贝。
//!
//! **给后来者（含 agent）的三条规则**：
//! 1. **新增能力加到上层** `neotrix-core/src/l1_action/nt_core_bank`，
//!    不要在本模块堆功能 —— 本模块只放**跨后端共用**的契约与纯函数。
//! 2. 本模块的 `pub fn` 是**给下游 crate 的 API 面**：
//!    「仓内零调用」≠ 无人使用（实测 25 个零调用函数全是 pub）。
//! 3. 两侧同名函数若有**实现差异**，那是**待定性的架构债**，不是 bug：
//!    见 `MIRROR-BANK-2026-09-30.md` §2/§4.3，改动前先配行为对位。

mod tier;
mod mem;
mod stats;
mod iteration;
mod pipeline;
mod offload;
mod l1;
mod bank;
mod seed_knowledge;

pub use tier::{MemoryTier, MemoryLifecycle, LifecycleAction, LifecycleConfig};
pub use mem::{ReasoningMemory, MemorySource, T3ViewType, T3Views, TemporalContext};
pub use stats::{ReasoningBankStats, MemoryDetailedStats};
pub use bank::ReasoningBank;
pub use iteration::{MemoryIterationResult, ConsolidationReport};
// 2026-09-30：`tokenize` / `rrf_fuse` 自此为单点真身，上层 neotrix-core 直接引用此处。
pub use iteration::{rrf_fuse, tokenize, RRF_K};
pub use pipeline::{PipelineConfig, PipelineState};
pub use offload::OffloadManager;
pub use l1::{L1Memory, SceneBlock, Persona, ExtractionPrompt};
