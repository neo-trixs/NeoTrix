//! # NeoTrix 统一文件能力 (Unified File Ability)
//!
//! 单一入口管理和操作所有文件 I/O：
//! - **Office 6 格式** (DOCX/XLSX/PPTX/DOC/XLS/PPT) — 由 `office_oxide` 提供
//!   读 (`plain_text`/`to_markdown`/`to_html`)、导 (`to_ir`/`save_as`)、
//!   编辑 (`EditableDocx`/`EditablePptx::replace_text`)
//! - **通用文本/PDF/图像/音频/视频** — 由 `neotrix-types::FileParser` 探测并提取
//! - **图像元数据** — 由 `image` crate 提取尺寸/通道/格式
//!
//! ## 纪律
//! - **R-P1**: 零 unsafe (office_oxide + FileParser + image 均纯 Rust)
//! - **R-P42**: 复用 core 既有成熟类型 (`ConstellationLevel`/`SelfTest`)，
//!   不平行重造枚举
//! - **Dark Forest**: 模块经能力树 `ConstellationLevel` 标记成熟度，
//!   经 `SelfTest` T1-T3 接线到意识树健康链，被流水线消费
//! - **指针守恒**: 单一 `path` 句柄，不复制状态
//!
//! ## 结构
//! 本文件为入口 re-export 根: 按职责拆分的子模块经 `pub use` 保持
//! `crate::neotrix::nt_file_ability::*` 公共 API 面完全不变。

mod core;
mod e8;
mod embedding;
mod encoding;
mod gwt;
mod helpers;
mod selftest;
mod event_types;
mod structured;
pub mod types;
mod doc_parse;
mod format_route;
pub mod image_super_resolution;
pub mod capability;
mod template_engine;
mod config_parser;
mod path_metadata;
mod table_presenter;
mod chunk_planner;

pub mod pdf;
pub mod excel;
pub mod merge;
pub mod visual;

pub use core::*;
pub use doc_parse::*;
pub use e8::*;
pub use embedding::*;
pub use encoding::*;
pub use gwt::*;
pub use helpers::*;
pub use selftest::*;
pub use event_types::*;
pub use structured::*;
pub use types::*;
pub use format_route::*;
pub use image_super_resolution::*;
pub use capability::*;
pub use template_engine::*;
pub use config_parser::*;
pub use path_metadata::*;
pub use table_presenter::*;
pub use chunk_planner::*;

pub use pdf::*;
pub use excel::*;
pub use merge::*;
pub use visual::*;

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;

/// 测试夹具重导出 (门面路径不变: `crate::neotrix::nt_file_ability::make_min_*`).
/// `merge::merge` 子模块测试经此路径复用 (R-P42)。
#[cfg(test)]
pub use test_helpers::{make_min_docx, make_min_pptx};
#[cfg(test)]
pub(crate) use test_helpers::make_min_docx_with_media;
