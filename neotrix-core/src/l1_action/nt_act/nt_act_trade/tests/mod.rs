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
//! - `test_orchestration.rs` (116 test) —— **已复活并声明, 116 全绿**。
//!   中途我曾判它「不可修, 属设计决策」—— **那个判断是错的**, 特此留档:
//!   我把 `DomainWorkerType`/`DomainWorkerResult` 路由到了 `workers` 模块,
//!   而它们其实是 `orchestrator_v2::{WorkerType, WorkerResult}` 的**旧名**。
//!   同名陷阱有两层, 都要注意:
//!     1. 两个 `WorkerType`: workers 是能力维度 {Extract,Analyze,Write,Send,Track},
//!        orchestrator_v2 是业务维度 {Inquiry,Quotation,Contract,...,Generic}
//!     2. 两个 `TradeWorker` trait: workers 的是 execute(WorkerTask) 且多
//!        worker_id/can_handle; orchestrator_v2 的是 execute(&TradeTask)。
//!        本文件两者都要用, 必须分别引入作用域。
//!   120 个错误里绝大多数是这类「import 指错模块」, 逐一改对后剩 6 个真问题。
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
//! 三个文件现已全部有归宿: 复活 2 (223 test) / 删除 1。棘轮基线归零
//! (scripts/check-truth-surface.sh 现为 0 条)。

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

#[cfg(test)]
mod test_orchestration;
