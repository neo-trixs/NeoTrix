//! Data Visualization — NT-IO 数据可视化层
//! 
//! 吸收的能力模块 (markdown-viewer/skills):
//! - stencil_registry: 9514 mxgraph stencils, 60 类别
//! - architecture_templates: 8 架构类型模板
//! - plantuml_emitter: PlantUML 语法生成器
//! - template_library: 8 示例文件作为行为接地 (T3)

pub mod stencil_registry;
pub mod architecture_templates;
pub mod plantuml_emitter;
pub mod template_library;