//! NT-SHIELD 防御机制子模块
//!
//! 核心防御逻辑引擎:
//! - UnifiedDefenseLayer: 统一防御层
//! - RefusalTamperEngine: 拒绝篡改检测
//! - GuardrailTraversalEngine: 护栏穿越检测
//! - ReasoningProtectionEngine: 推理链保护
//! - AntiDistillationEngine: 反蒸馏防御

// 2026-10-04 **接线**（用户拍板选项 A）：四棵「环」子树此前
// **从未在 `mod.rs` 声明** ⇒ 2,152 行 Rust **从不参与编译**。
//
// 接线前我先做了三件事（因为「声明」≠「可用」）：
// ① **修 3 个红测试**（`54be0301`）：其中 `trust_classifier`
//    是**真安全缺陷**（「来源可信」曾能抵消「内容高危」⇒
//    prompt 注入只要来源标 `local` 就被判「中等可信」）⇒ 已修为
//    **内容高危 = 一票否决**。
// ② **验证它们能编译**（用 canary 注入类型错误，证明编译器
//    真的吃到这些文件，而不是「0 错误 = 没编译」）。
// ③ **两套检索门取证**：L4 `nt_retrieval_gate` 判「**要不要检索**」
//    （碰存储**之前**），ring_inner 判「**结果能不能给**」
//    （检索**之后**）⇒ **串联的两道闸，不是同一件事的两份实现**。
//
// 顺序有语义：core 在前、边界环居中、outer 收口、inner 最内。
pub mod ring_core;
pub mod ring_inner;
pub mod ring_outer;
pub mod ring_boundary;

pub mod unified_defense;
pub mod refusal_tamper;
pub mod guardrail_traversal;
pub mod reasoning_protection;
pub mod anti_distillation;

pub use unified_defense::UnifiedDefenseLayer;
pub use refusal_tamper::RefusalTamperEngine;
pub use guardrail_traversal::GuardrailTraversalEngine;
pub use reasoning_protection::ReasoningProtectionEngine;
pub use anti_distillation::AntiDistillationEngine;
