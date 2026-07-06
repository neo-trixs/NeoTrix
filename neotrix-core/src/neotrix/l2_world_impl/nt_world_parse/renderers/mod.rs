//! 文档渲染器
//!
//! 将解析结果渲染为多种输出格式:
//! - markdown: 标准 Markdown
//! - json: JSON 树 (映射到 nt_memory_kb 的 22 节点类型/19 关系)
//! - chunks: 分块 (用于 KB 存储)

pub mod markdown_renderer;
pub mod json_renderer;
pub mod chunk_renderer;
#[cfg(feature = "office")]
pub mod office_renderer;
