//! Trade 模块测试套件
//!
//! 包含数据模型、知识库、流程引擎、事件总线的单元测试。
//!
//! audit 2026-09-27: 本目录原有 3 个文件未被 `mod` 声明, 从不参与编译
//! (3 个文件 / 4,899 LOC / 311 个 `#[test]`)。逐个实测后的真实处置如下,
//! 结论与直觉常常相反, 故写清楚以免下次重判:
//!
//! - `test_extractors.rs` (107 test) —— **已复活并声明**。
//!   原判「无法编译」只对了一半: 31 个错误里 21 个是机械性的
//!   (缺 import / 可见性), 修完 103 个直接通过。它还查出 2 个真实实现缺陷
//!   (chrome_decrypt 的 strip_prefix 接受空密文; NetworkLogEntry.status
//!   序列化成 null), 已修。另有 2 个测试自身写错 (载荷非 AES 块对齐,
//!   且断言随机字节能解出合法 UTF-8), 已改为断言其名所声称的事。
//!
//! - `test_orchestration.rs` (116 test) —— **保持未声明**。
//!   实测 101 个错误, 做完 import 对账后反而升到 120。根因是它把两个
//!   **同名但不同设计**的枚举当成一个:
//!     workers::WorkerType  = 能力维度 {Extract, Analyze, Write, Send, Track}
//!     orchestrator_v2.rs:70 = 业务维度 {Inquiry, Quotation, Contract,
//!                               Production, Logistics, Finance, Generic}
//!   它要 `WorkerType::{Generic, Inquiry, Quotation}` + `from_task_type`,
//!   而前者没有这些变体。它是一份「已被现行设计取代」的旧规格, 不是可修的
//!   测试。要接就得先裁决两套 taxonomy 哪个留 —— 属设计决策, 非机械修复。
//!
//! - `test_business.rs` (88 test) —— **已删除**。
//!   它测的 `nt_mind::sales_coaching` 已在 3bba2507「首批死码清除」中删除,
//!   且它依赖的 `SalesCoach` / `CustomerStage` / `PerformanceRecord`
//!   三个类型在全仓 0 处定义。保留一个测「已被主动删除的功能」的测试没有意义,
//!   故与该功能的删除决定保持一致。
//!
//! 附带成果: `nt_feel::writing_style`(1,241 LOC) 此前从未被 `mod` 声明,
//! 从未参与编译; 本次接上后 0 error 0 warning —— 它一直是好代码, 只是没通电。
//!
//! 残留的 UNDECLARED 由 scripts/check-truth-surface.sh 棘轮监控。

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
