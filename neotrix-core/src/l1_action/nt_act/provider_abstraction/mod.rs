/// 模型无关的 LLM 类型（Message/Response/Role/ModelInfo/ModelCost/…）。
///
/// ⚠️ 2026-10-04 之前本模块**从未被编译**：文件在此，`mod.rs` 无声明。
///   ⇒ 258 行模型抽象类型 + 4 个从未运行的测试整体离线，
///      且无任何生产/消费方引用（`rg provider_abstraction::models` 零命中）。
///
/// ⚠️⚠️ **注意：`models::Message` 与 `provider::Message` 是两个不同类型**
///   （各自独立定义，无 re-export 关系）。
///   ⇒ 接入**不会**产生类型冲突（不同模块），但也**不会**让两者统一。
///   若将来要把它们合并成一个正典类型，那是独立的一次重构，
///   本提交只负责「让它参与编译」，不做语义合并。
pub mod models;

pub mod provider;
pub mod config;
pub mod registry;
pub mod router;

pub use provider::{Message, CompletionRequest, CompletionResponse, LlmProvider};
pub use config::ProviderConfig;
pub use registry::ProviderRegistry;
pub use router::CostAwareRouter;
