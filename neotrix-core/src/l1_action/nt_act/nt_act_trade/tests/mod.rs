//! Trade 模块测试套件
//!
//! 包含数据模型、知识库、流程引擎、事件总线的单元测试。
//!
//! audit 2026-09-27: 同目录下 `test_orchestration.rs` / `test_extractors.rs`
//! / `test_business.rs` 三个文件 (4,899 LOC / 311 个 `#[test]`) 未被声明, 从不参与编译。
//! 结论是「无法编译」, 但**原因经实测逐条核实**, 勿凭直觉再判:
//!
//! - `test_extractors.rs` —— 实测声明后 31 个错误 (已复现并回退):
//!     * 符号不存在: `ActivityLog` / `ExtractConfig` / `EmailConfig`
//!       (data_pipeline / platform_registry 模块本身存在, 是它们的 pub API 缺这些)
//!     * 方法不存在: `JoinfExtractor::{platform_id, platform_name}`
//!     * 可见性: `map_customer` / `map_email` / `map_interaction` 为私有
//!     * 仅缺 import: `Arc` / `DateTime`
//!   => 部分可修 (约一半是加 import / 放开 pub), 但需先裁决 API 归属。
//! - `test_orchestration.rs` —— 引用 `orchestrator_v2::{DomainWorkerType,
//!   DomainWorkerResult}`, 二者在全仓 0 处定义。注意 `TradeWorker` **确实存在**
//!   (`workers/mod.rs:126` 的 trait), 只是不在 `orchestrator_v2` 下, 别再误判为缺失。
//! - `test_business.rs` —— 引用的 `nt_mind::sales_coaching` 已在 3bba2507
//!   「首批死码清除」中删除; 且它以 `neotrix_core::` 自路径写在 `src/` 内,
//!   而 lib.rs 无 `extern crate self as neotrix_core`, 故该路径无法解析。
//!
//! 三者均为「先写测试骨架、后未实现/已重构」的历史残留。处置需先裁决
//! (补实现 or 删文件), 故此处保持未声明。由 scripts/check-truth-surface.sh 棘轮监控。

#![forbid(unsafe_code)]

#[cfg(test)]
mod data_model_tests;

#[cfg(test)]
mod knowledge_base_tests;

#[cfg(test)]
mod process_engine_tests;

#[cfg(test)]
mod event_bus_tests;

#[cfg(test)]
mod test_extractors;
