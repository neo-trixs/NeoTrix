//! Trade 模块测试套件
//!
//! 包含数据模型、知识库、流程引擎、事件总线的单元测试。

#![forbid(unsafe_code)]

#[cfg(test)]
mod data_model_tests;

#[cfg(test)]
mod knowledge_base_tests;

#[cfg(test)]
mod process_engine_tests;

#[cfg(test)]
mod event_bus_tests;
