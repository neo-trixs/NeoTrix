use std::path::Path;

use neotrix_types::core::file_parser::{BlockType, FileFormat, FileParser};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| usage_and_exit());
    if !Path::new(&path).is_file() {
        eprintln!("error: 文件不存在或不可读: {path}");
        std::process::exit(2);
    }
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: 读取失败 {path}: {e}");
            std::process::exit(2);
        }
    };
    let format = FileParser::detect_format(&path, "application/octet-stream", &data);
    if format != FileFormat::Pdf {
        eprintln!("error: 非 PDF 文件 (检测到 {:?}), 本工具仅处理 PDF", format);
        std::process::exit(3);
    }

    // 生产接线: parse_with_layout 提供 text + spatial (布局块)
    let res = FileParser::parse_with_layout(&path, "application/pdf", &data);
    println!("filename={}", res.filename);
    println!("format={:?} parse_success={}", res.format, res.parse_success);
    println!("size={}", res.size_bytes);

    // 生产接线: extract_pdf_pages 提供逐页文本 (lopdf 完整解析压缩流)
    let pages = FileParser::extract_pdf_pages(&data);
    println!("pages={}", pages.len());
    println!("=== TEXT BEGIN ===");
    for (page, text) in &pages {
        println!("--- page {page} ---");
        println!("{text}");
    }
    println!("=== TEXT END ===");

    // 布局块 (spatial): parse_with_layout 尽力而为, 真实压缩 PDF 可能为空
    if !res.spatial_blocks.is_empty() {
        println!("=== SPATIAL BLOCKS ({}): ===", res.spatial_blocks.len());
        for b in &res.spatial_blocks {
            let kind = match b.block_type {
                BlockType::TextBlock => "text",
                BlockType::ImageBlock => "image",
                BlockType::TableBlock => "table",
                BlockType::HeadingBlock => "heading",
            };
            println!(
                "x={:.1} y={:.1} w={:.1} h={:.1} type={kind} text={}",
                b.x, b.y, b.width, b.height, b.text
            );
        }
    } else {
        println!("=== SPATIAL BLOCKS: 0 (压缩流/复杂布局需 OCR 或 pdfplumber 级布局算法) ===");
    }

    // 无文本层检测: 页数 0 且无布局块 → 扫描版/图像型 PDF, 提示 OCR
    if pages.is_empty() && res.spatial_blocks.is_empty() {
        println!("=== NOTE: 未提取到文本 (可能为扫描版/图像型 PDF, 需 OCR) ===");
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage: extract_pdf <file.pdf>");
    std::process::exit(1);
}