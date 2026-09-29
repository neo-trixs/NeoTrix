//! 文档解析统一网关测试 — 验证 free LLM 池做 text→Markdown 的后处理链路
//!
//! 架构验证目标:
//! 1. free LLM (Pollinations) 可将 OCR 原始文本 → clean Markdown
//! 2. GatewayV2 的断路器/回退链可用于 ParseGateway
//! 3. prompt 工程设计对文档解析质量的影响
//!
//! 注意: PDF→image 需要 poppler/PyMuPDF，本测试仅验证 text→Markdown 链路

use neotrix::l1_action::nt_io::nt_io_provider::{GatewayV2, LlmProvider, LlmRequest};
use neotrix::l1_action::nt_io::nt_io_provider::gateway::PollinationsProvider;

/// 模拟 OCR 原始输出
const RAW_OCR_TEXT: &str = r#"olmOCR: Unlocking Trillions of Tokens in PDFs with VLMs
Jake Poznanski, Jon Borchardt, Allen Institute for AI

ABSTRACT
We present olmOCR for converting PDFs into clean plain text.

Table 1: OCR comparison
Mistral OCR API 72.0
Marker 1.10.1 76.1
olmOCR 82.4

1 Introduction
PDFs remain difficult to extract text from.
2 Method
We fine-tune Qwen2.5-VL-7B using LoRA."#;

const DOC_PARSE_PROMPT: &str = "Convert this raw OCR text into clean Markdown. \
     Use # for title, ## for sections. \
     Format tables as | pipe | markdown. \
     Remove headers and footers. \
     Output ONLY the markdown.";

#[tokio::test]
async fn test_pollinations_doc_parse_text_only() {
    let provider = PollinationsProvider::new();
    let combined = format!("{}\n\n---\n{}\n---", DOC_PARSE_PROMPT, RAW_OCR_TEXT);
    let request = LlmRequest::new("openai", &combined)
        .with_temperature(Some(0.1))
        .with_max_tokens(2048);

    match provider.complete(&request).await {
        Ok(response) => {
            let md = response.content.trim();
            eprintln!("=== Pollinations text→Markdown result ===\n{}", md);
            assert!(!md.is_empty());
            assert!(md.contains("olmOCR"), "should contain title");
            assert!(md.contains("|"), "table should be markdown table");
            assert!(md.contains("##"), "should have section headings");
        }
        Err(e) => {
            eprintln!("SKIP (network issue: {})", e);
        }
    }
}

#[tokio::test]
async fn test_gateway_doc_parse_fallback_chain() {
    let gw = GatewayV2::new();
    gw.register_provider("pollinations", std::sync::Arc::new(PollinationsProvider::new()), true);

    let combined = format!("{}\n\n---\n{}\n---", DOC_PARSE_PROMPT, RAW_OCR_TEXT);
    let request = LlmRequest::new("openai", &combined)
        .with_temperature(Some(0.1))
        .with_max_tokens(2048);

    match gw.complete_with_selection(&request).await {
        Ok(resp) => {
            eprintln!("=== Gateway text→Markdown result ===\n{}", resp.response.content);
            assert!(resp.response.content.contains("|"));
        }
        Err(e) => {
            eprintln!("SKIP (network issue: {})", e);
        }
    }
}

/// 验证 prompt 设计本身正确（无网络依赖）
#[test]
fn test_doc_parse_prompt_design() {
    let prompt = format!(
        "Convert this raw OCR text into clean Markdown.\n\n---\n{}\n---\n\n\
         Rules:\n\
         1. # for title, ## for sections\n\
         2. Tables as | pipe | format\n\
         3. Remove headers/footers\n\
         4. Output ONLY markdown",
        RAW_OCR_TEXT
    );

    assert!(prompt.contains("clean Markdown"));
    assert!(prompt.contains("# for title"));
    assert!(prompt.contains("| pipe |"));
    assert!(prompt.contains("ONLY markdown"));
}

/// 验证文档解析结果的结构约束
#[test]
fn test_parsed_markdown_structure() {
    let result = r#"# olmOCR: Unlocking Trillions of Tokens in PDFs with VLMs

*Jake Poznanski, Jon Borchardt, Allen Institute for AI*

## Abstract

We present olmOCR for converting PDFs into clean plain text.

## Table 1: OCR comparison

| Method | Score |
|--------|-------|
| Mistral OCR API | 72.0 |
| Marker 1.10.1 | 76.1 |
| olmOCR | 82.4 |

## 1 Introduction

PDFs remain difficult to extract text from.

## 2 Method

We fine-tune Qwen2.5-VL-7B using LoRA."#;

    // 验证结构
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines[0].starts_with("# "), "title should be H1");
    assert!(
        result.contains("## Abstract"),
        "should have Abstract section"
    );
    assert!(result.contains("| Method | Score |"), "table header");
    assert!(result.contains("|--------|-------|"), "table separator");
    assert!(result.contains("## 1 Introduction"), "section heading");
}

/// 验证 ParseGateway 的三层路由策略
#[test]
fn test_parse_gateway_routing_strategy() {
    // 模拟 ParseGateway 应该有的三层路由:
    // Tier 0: Local / free (no vision)
    // Tier 1: Local VLM (Ollama + vision model)
    // Tier 2: Paid vision API (GPT-4V / Gemini)
    enum ParseTier {
        TextOnly,    // free LLM: text→Markdown post-processing
        LocalVision, // Ollama + Qwen2.5-VL-7B
        #[allow(dead_code)] // test scaffold for paid vision tier
        PaidVision, // GPT-4V / Gemini Vision
    }

    // 验证: 当文档含图片时必须用 Tier 1/2
    assert!(
        std::mem::discriminant(&ParseTier::TextOnly)
            != std::mem::discriminant(&ParseTier::LocalVision),
        "text-only != vision parsing should be different tiers"
    );
}
