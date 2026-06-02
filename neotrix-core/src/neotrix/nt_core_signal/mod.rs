//! Signal 模块 - 选择性状态向量 Ψ
//! 基于 Mamba SSM 思想: 输入相关的选择性状态机制
//! 核心: 状态不再是静态的，而是输入内容的函数

// 子模块声明
pub mod core;          // MatrixError, SelectiveState, Vector, Matrix, 基础实现
pub mod select;        // SelectableOperator, SemanticBlock, SemanticType
pub mod history;       // StateHistory, ConsciousnessTier
pub mod attribution;   // SIGReg, AttributionSource, AttributionSummary
pub mod ops;           // 向量/矩阵运算函数

// Re-export 主要类型
pub use core::{Vector, Matrix, MatrixError, SelectiveState};
pub use select::{SelectableOperator, SemanticBlock, SemanticType};
pub use history::{StateHistory, ConsciousnessTier};
pub use attribution::{SIGReg, AttributionSource, AttributionSummary};
pub use ops::{
    l2_norm, softmax, relu, sigmoid, gelu,
    matrix_vector_mul, matrix_vector_mul_safe,
    dot_product, cosine_similarity, euclidean_distance,
    normalize, clamp,
};
