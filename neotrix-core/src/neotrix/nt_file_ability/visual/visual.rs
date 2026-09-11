//! doc7 吸收: 视觉理解 Prompt 路由 + 提取管线 (VisualExtractor)。
//! 来源: github.com/magicrew/doc7 internal/extract/prompt.go (MIT, absorbed 2026-08-13)。
//! 公理 (视觉理解是提取上限): 光栅化页面 → VLM 整页理解 → 保真 Markdown + grounding 校验。
//! 设计 (R-P42): 不建平行 provider, 模型通道经闭包注入 — 生产接 NT-IO LlmProvider,
//! 测试接假闭包 (零网络依赖)。管线核心 (准备/prompt/校验/报告) 为同步纯逻辑, 可测。

use serde::{Deserialize, Serialize};

use super::grounding::{ground_missing_tokens, GroundingReport};
use super::types::FileKind;

/// 视觉理解 prompt 名称
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualPromptKind {
    /// 文档页 (PDF/Office/图像/扫描件)
    Document,
    /// 演示幻灯片 (PPT/PPTX/ODP)
    Slide,
}

impl VisualPromptKind {
    /// 预留: 仅供测试断言 (name 往返) 与未来日志/调试输出使用, 非生产调用路径。
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            VisualPromptKind::Document => "document",
            VisualPromptKind::Slide => "slide",
        }
    }

    /// 按 FileKind 自动路由: 演示类 → Slide, 其余 → Document
    pub fn for_file_kind(kind: FileKind) -> Self {
        match kind {
            FileKind::Office(office_oxide::DocumentFormat::Pptx)
            | FileKind::Office(office_oxide::DocumentFormat::Ppt) => VisualPromptKind::Slide,
            _ => VisualPromptKind::Document,
        }
    }

    /// 返回完整视觉理解 prompt (doc7 documentPrompt/slidePrompt 忠实吸收)
    pub fn prompt(&self) -> &'static str {
        match self {
            VisualPromptKind::Document => DOC7_DOCUMENT_PROMPT,
            VisualPromptKind::Slide => DOC7_SLIDE_PROMPT,
        }
    }
}

/// doc7 documentPrompt — 整页保真转写 (表格/视觉/图/公式/页脚全覆盖)
pub const DOC7_DOCUMENT_PROMPT: &str = r#"Transcribe the entire document page as a faithful Markdown fragment from top to bottom. First scan the whole page, including small text at the edges and bottom, then write the transcription. Preserve the original language, reading order, hierarchy, and every readable word, number, unit, table row, chart point, formula, label, status, button, footnote, caption, and footer. Use headings, paragraphs, lists, tables, code blocks, and LaTeX as appropriate. Use $$...$$ for standalone displayed formulas and $...$ only for formulas embedded in prose.

Table rules:
- A single visual table must become one continuous Markdown table with one header row and the same column count on every data row.
- Bold, shaded, indented, subtotal, total, and section rows are still data rows; never turn them into a new header or a second table.
- Keep hierarchical row labels in the first column and preserve every value in its original column.
- Do not merge nearby labels into extra columns or invent blank header rows. If formatting is uncertain, preserve the values as ordered plain text rather than silently changing their relationships.

Visual rules:
- Describe every non-text visual only as one or more blockquotes in the form > [Visual] ...; never use Markdown image syntax, Mermaid, ASCII art, SVG, diagram code, or invented image links.
- For charts, preserve visible axes, legends, series, values, trends, and conclusions.
- For diagrams, workflows, and multi-part figures, write enough ordered prose that a reader could reconstruct the visible topology without seeing the page.
- State the visual type, title, overall reading direction, spatial arrangement of parts, input order, every readable node or label, and every visible directed connection in sequence.
- Explicitly preserve branches, merges, bypass connections, loops, parallel paths, repeated components and counts, nested or grouped regions, and the final visible destination. Distinguish separate paths instead of collapsing them into a summary.
- Audit each arrow endpoint before writing. For a long bypass arrow that crosses intermediate nodes, state the exact source and destination and name the skipped nodes; never attach an input to the nearest or lowest box merely because the arrow passes beside it.
- Describe only visible structure. Do not infer hidden nodes, implicit outputs, or relationships that are not shown.
- Never replace a visual with only its title or caption. Do not summarize, translate, infer, invent, or omit readable information. Mark unreadable content as "不可读" on Chinese pages or "unreadable" otherwise.

Return only Markdown without metadata, commentary, or enclosing code fences."#;

/// doc7 slidePrompt — 幻灯片保真转写
pub const DOC7_SLIDE_PROMPT: &str = r#"Transcribe the entire presentation slide as a faithful Markdown fragment. Use the visible title as the leading heading. Preserve the original language, reading order, hierarchy, and every readable bullet, label, example, number, unit, table cell, formula, brand, tool, and decision criterion. Use $$...$$ for standalone displayed formulas and $...$ only for formulas embedded in prose.

Visual rules:
- Describe every chart, matrix, funnel, workflow, quadrant, screenshot, or other non-text visual only as one or more blockquotes in the form > [Visual] ...; never use Markdown image syntax, Mermaid, ASCII art, SVG, diagram code, or invented image links.
- For charts, preserve visible axes, legends, series, values, trends, and conclusions.
- For diagrams, workflows, and multi-part figures, write enough ordered prose that a reader could reconstruct the visible topology without seeing the slide.
- State the visual type, title, overall reading direction, spatial arrangement of parts, input order, every readable node or label, and every visible directed connection in sequence.
- Explicitly preserve branches, merges, bypass connections, loops, parallel paths, repeated components and counts, nested or grouped regions, comparisons, and the final visible destination. Distinguish separate paths instead of collapsing them into a summary.
- Audit each arrow endpoint before writing. For a long bypass arrow that crosses intermediate nodes, state the exact source and destination and name the skipped nodes; never attach an input to the nearest or lowest box merely because the arrow passes beside it.
- Describe only visible structure. Do not infer hidden nodes, implicit outputs, or relationships that are not shown.
- Never replace a visual with only its title or caption. Do not summarize, translate, infer, invent, or omit readable information. Mark unreadable content as "不可读" on Chinese slides or "unreadable" otherwise.

Return only Markdown without metadata, commentary, or enclosing code fences."#;

/// VLM 模型调用闭包: 输入 (prompt, base64 图像, mime), 输出 (转写文本, 失败原因)
pub type VlmCall = dyn Fn(&str, &str, &str) -> std::result::Result<String, String>;

/// 视觉提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExtractConfig {
    /// 图像最大字节 (超限降级, doc7 默认 9MB)
    pub max_image_bytes: usize,
    /// 上下文降级次数 (doc7 --context-fallbacks 默认 2)
    pub context_fallbacks: usize,
    /// 最小图像边长 (降级下限, doc7 --min-image-dimension 默认 720)
    pub min_image_dimension: u32,
    /// 是否启用文本层 grounding 校验 (doc7 --text-grounding)
    pub text_grounding: bool,
    /// 是否重试 (doc7 RetryCount)
    pub retry_count: usize,
}

impl Default for VisualExtractConfig {
    fn default() -> Self {
        Self {
            max_image_bytes: 9 * 1024 * 1024,
            context_fallbacks: 2,
            min_image_dimension: 720,
            text_grounding: false,
            retry_count: 2,
        }
    }
}

/// 视觉提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExtractResult {
    pub markdown: String,
    pub prompt_kind: VisualPromptKind,
    pub grounding: Option<GroundingReport>,
    pub context_fallbacks_used: usize,
    pub image_max_dimension: u32,
    pub failed: bool,
    pub error: Option<String>,
    /// 公理一 (视觉理解是提取上限) 量化: 提取是否在保真上限内。
    /// true = 未接地 token 占比低于阈值 (可靠性 ≥ 0.5) 或未启用 grounding。
    /// false = 提取超出保真上限, 调用方应二次校正或标记人工复核。
    pub extraction_bound_ok: bool,
}

/// 执行一次 VLM 视觉提取 (doc7 单页管线核心)。
/// - `image_b64`: base64 编码的图像 (PNG/JPEG)
/// - `source_text`: 嵌入文本层 (PDF/Office 文本层), 供 grounding; 无则传 ""
/// - `kind`: 文件类型 → prompt 路由
/// - `call`: 模型闭包 (prompt, image_b64, mime) -> Result<String>
pub fn visual_extract(
    kind: FileKind,
    image_b64: &str,
    source_text: &str,
    config: &VisualExtractConfig,
    call: &VlmCall,
) -> VisualExtractResult {
    let prompt_kind = VisualPromptKind::for_file_kind(kind);
    let prompt = prompt_kind.prompt();
    let mut result = VisualExtractResult {
        markdown: String::new(),
        prompt_kind,
        grounding: None,
        context_fallbacks_used: 0,
        image_max_dimension: 0,
        failed: false,
        error: None,
        extraction_bound_ok: true,
    };

    // 图像大小上限校验 → 超限标记需要降级 (调用方决定, 此处报告)
    if image_b64.len() > config.max_image_bytes {
        result.error = Some(format!(
            "image exceeds {} bytes ({} actual); lower resolution or increase max_image_bytes",
            config.max_image_bytes,
            image_b64.len()
        ));
        result.failed = true;
        return result;
    }

    // 调用模型 (temperature 0 由调用方 provider 配置, 此处重试语义)
    let mut last_err = None;
    let mut content = String::new();
    for attempt in 0..=config.retry_count {
        match call(prompt, image_b64, "image/png") {
            Ok(text) => {
                content = text;
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(e);
                // 最后尝试失败才中断
                if attempt == config.retry_count {
                    break;
                }
            }
        }
    }
    if let Some(e) = last_err {
        result.error = Some(format!("VLM call failed after {} attempts: {e}", config.retry_count + 1));
        result.failed = true;
        return result;
    }

    result.markdown = content;

    // grounding 校验 (可选): 嵌入文本层关键 token 缺失检测
    if config.text_grounding && !source_text.trim().is_empty() {
        let report = ground_missing_tokens(source_text, &result.markdown);
        // 公理一 (视觉理解是提取上限): 可靠性 < 0.5 → 提取超出保真上限。
        // 调用方据此二次校正或标记人工复核 (R-P79 生产接线决策点)。
        result.extraction_bound_ok = report.reliability_score >= 0.5;
        result.grounding = Some(report);
    }

    result
}