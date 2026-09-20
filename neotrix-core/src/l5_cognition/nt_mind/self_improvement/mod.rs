//! Self-Improvement Loop — MemGPT-inspired agent self-editing and sleep-time compute
//!
//! This module implements the core self-improvement loop for nt_mind:
//!
//! - **Memory-as-Filesystem** (`memory_filesystem`): Agent-editable memory blocks
//!   with label-based indexing (core/recall/archive). R-MEM09 compliance.
//!
//! - **Self-Edit Loop** (`self_edit_loop`): Propose → validate → apply pipeline
//!   for memory modifications with injection detection and audit trail.
//!
//! - **Sleep-Time Compute** (`sleep_compute`): Consolidation, reflection, and
//!   pruning of memory blocks. R-P121 decay, R-P122 localized maintenance.
//!
//! # Architecture
//!
//! ```text
//! Agent Conversation Turn
//!   ├── MemoryFilesystem (read/write blocks)
//!   ├── SelfEditLoop (propose → validate → apply)
//!   │     └── ValidationRules (injection, length, audit)
//!   └── SleepComputer (idle-time maintenance)
//!         ├── consolidate() — recall → core promotion
//!         ├── reflect()     — pattern detection
//!         └── prune()       — decay-based removal
//! ```

pub mod memory_filesystem;
pub mod self_edit_loop;
pub mod sleep_compute;

pub use memory_filesystem::{
    EditResult, MemoryBlock, MemoryEditProposal, MemoryFilesystem, MemoryLabel,
};
pub use self_edit_loop::{
    EditAuditEntry, EditLoopStats, EditProposal, SelfEditLoop, ValidationResult,
    ValidationRule,
};
pub use sleep_compute::{
    ConsolidatedMemory, ConsolidationAction, ConsolidationConfig, ConsolidationPipeline,
    ConsolidationResult, IdentityPersistence, IdentitySnapshot, MemoryPattern, MemoryRef,
    MetaInsight, PruneResult, ReflectionEngine, ReflectionResult, SessionSummary, SleepComputer,
    SleepConfig,
};
