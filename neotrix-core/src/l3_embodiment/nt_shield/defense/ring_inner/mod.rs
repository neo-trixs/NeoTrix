//! Ring Inner - 内环：输入消毒 + 信任分级
//!
//! 输入消毒 + 信任分级 + 检索门控 + 意图重映射

pub mod input_sanitizer;
pub mod trust_classifier;
pub mod retrieval_gate;
pub mod intent_remapper;

pub use input_sanitizer::InputSanitizer;
pub use trust_classifier::TrustClassifier;
pub use retrieval_gate::RetrievalGate;
pub use intent_remapper::IntentRemapper;

/// 内环验证结果
#[derive(Debug, Clone)]
pub struct InnerVerification {
    pub sanitized_input: String,
    pub trust_level: super::ring_core::TrustLevel,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub retrieval_allowed: bool,
    pub remapped_intent: Option<String>,
    pub signals: Vec<String>,
}
