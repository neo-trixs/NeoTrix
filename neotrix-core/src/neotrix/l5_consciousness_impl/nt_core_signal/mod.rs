//! Signal 模块 - 选择性状态向量 Ψ
//! 基于 Mamba SSM 思想: 输入相关的选择性状态机制
//! 核心: 状态不再是静态的，而是输入内容的函数

// 子模块声明
pub mod core;          // MatrixError, SelectiveState, Vector, Matrix, 基础实现
pub mod select;        // SelectableOperator, SemanticBlock, SemanticType
pub mod ops;           // 向量/矩阵运算函数 (生产仅 cosine_similarity 被消费)

// Re-export 主要类型
pub use core::{Vector, Matrix, MatrixError, SelectiveState, SsdState, SSM_STATE_SIZE};
pub use select::{SelectableOperator, SemanticBlock, SemanticType, SsdOperator};
pub use ops::cosine_similarity;
