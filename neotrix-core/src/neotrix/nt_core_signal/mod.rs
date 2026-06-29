//! Signal 模块 - 选择性状态向量 Ψ
//! 基于 Mamba SSM 思想: 输入相关的选择性状态机制
//! 核心: 状态不再是静态的，而是输入内容的函数

// 子模块声明
pub mod attribution; // SIGReg, AttributionSource, AttributionSummary
pub mod core; // MatrixError, SelectiveState, Vector, Matrix, 基础实现
pub mod history; // StateHistory, ConsciousnessTier
pub mod ops;
pub mod select; // SelectableOperator, SemanticBlock, SemanticType // 向量/矩阵运算函数

// Re-export 主要类型
pub use attribution::{AttributionSource, AttributionSummary, SIGReg};
pub use core::{Matrix, MatrixError, Vector};
pub use history::{ConsciousnessTier, StateHistory};
pub use ops::{
    clamp, cosine_similarity, dot_product, euclidean_distance, gelu, l2_norm, matrix_vector_mul,
    matrix_vector_mul_safe, normalize, relu, sigmoid, softmax,
};
pub use select::{SelectableOperator, SemanticBlock, SemanticType};
