//! 文档后处理器
//!
//! 在解析后对结果进行增强:
//! - table: 表格检测与结构化
//! - math: 公式检测与 LaTeX 转换
//! - cleanup: 噪音去除/格式化修复

pub mod table_processor;
pub mod math_processor;
pub mod cleanup_processor;
