//! # NeoTrix 核心模块
//!
//! 本地模块：nt_crystal_core, nt_file_ability, nt_capability_bridge,
//! nt_core_error, nt_core_event_bus, ffi, proxy_daemon_wrapper

#![forbid(unsafe_code)]

// ─── Minimal re-exports for binary/entry crates ──────────────────────────
pub use crate::l2_perception::nt_world::nt_world_model;
pub use crate::l1_action::nt_io::nt_io_hotreload;

// ─── Local modules ────────────────────────────────────────────────────────
pub mod nt_capability_bridge;

#[cfg(feature = "ios-bridge")]
pub mod ffi;

pub mod nt_core_error;
pub mod nt_core_event_bus;
pub mod nt_file_ability;
pub mod nt_crystal_core;
pub mod nt_jev;
pub mod proxy_daemon_wrapper;

// ─── Standalone crate re-export ──────────────────────────────────────────
pub use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, ConstellationLevel, Domain as CapabilityDomain,
    EvolutionAction, EvolutionEngine, EvolutionOp, EvolutionPlan, NodeLayer, RuneSocket,
};

// ─── Unified File Ability re-export ──────────────────────────────────────
pub use nt_file_ability::{
    check_health,
    consolidate_tables_with_mode, content_similarity,
    create_from_markdown, decode_bytes,
    detect_encoding, edit_pdf, edit_xlsx_table, embed_text, extract_text, extract_pdf_tables,
    extract_dir, merge_pdfs,
    load_snapshot,
    merge_tables_with, merge_tables_with_mode,
    normalize_column_name, read_csv, read_structured, read_xlsx_sheets_all, read_xlsx_table,
    FileModel,
    replace_placeholder, route_attention, save_edited, specialist_index, store_snapshot,
    suggest_schema, to_markdown, write_csv, write_json, write_xlsx_table, ConsolidationReport,
    ContentSnapshot, FileAbility, FileAbilityError, FileAbilitySelfTest, FileKind, FileOperation,
    ImageMetadata, MergeSchema, OcrEngine, OcrResult, PRICE_STANDARD_COLUMNS, PRICE_TABLE_SCHEMA,
    RuleBasedOcr, SchemaSuggestion, SchemaStore, MergeSchemaJson, SheetCellData,
    SheetCellValueType, SheetData, SheetMode,
    SheetRowData,
    StructuredData, TableData, TableEdit, TextEncoding, UnitRule, PdfEdit,
    DirExtractEntry, DirExtractReport,
    CollectionMergeRequest, MergeOutcome, MergeStrategy, collection_merge,
    SuperResolutionModel, SuperResolutionConfig, PdfIconEnhanceConfig, enhance_pdf_icons_with_config,
    E8StateTransition, GwtAttentionRouter, VsaEmbedding,
};

pub use nt_file_ability::merge_docx;

#[cfg(test)]
pub(crate) use nt_file_ability::{make_min_docx, make_min_pptx};
