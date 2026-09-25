//! # NT-IO multimodal_transform — 多模态→文本降维预处理阶段
//!
//! 吸收源: hqman/pi-deepseek-vision — 在 AgentLoop 进入 text-only 目标模型前，
//! 将消息中的 ImageContent 替换为"编号标记 + 纯文本分析"，目标模型始终不接触
//! 图像。vision 模型只做感知/分析，推理模型保持 text-only。
//!
//! 骨架阶段 (C0): 图片标记检测/替换 + 可插拔 VisionAnalyzer 已接 AgentLoop
//! 生产路径; 待完善: 真 vision 后端接入 / toolResult 图片批量变换 / 顺序保真
//! 与多图批处理。
//!
//! # Diagram/Chart Rendering (G17) — 图表渲染吸收
//!
//! 吸收源: pretty-mermaid-skills + diagram-design — Mermaid→ASCII 渲染、
//! 27 种视觉类型分类、语义/布局解耦。KB-落盘/CLI 显示路径的可读化产出:
//! `render_diagram(source)` 入口 (text-based source 语法)。
//! 零外部渲染依赖: 结构化 `DiagramModel` + box-drawing ASCII 渲染 + Mermaid 文本生成。
//! 语义模型 (节点/边/类型) 与布局表现 (ASCII 框线 / Mermaid 文本) 分离。

pub mod nt_audio_transform;
pub mod nt_image_transform;
pub mod nt_text_transform;
pub mod nt_transform_pipeline;
pub mod nt_transform_types;

pub use nt_audio_transform::*;
pub use nt_image_transform::*;
pub use nt_text_transform::*;
pub use nt_transform_pipeline::*;
pub use nt_transform_types::*;
