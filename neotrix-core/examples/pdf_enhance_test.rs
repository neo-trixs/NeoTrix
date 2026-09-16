//! PDF 图标增强完整测试
//! 
//! 测试流程: PDF → 提取图像 → 超分增强 → 输出结果

use std::path::Path;
use neotrix::neotrix::nt_file_ability::{
    extract_pdf_images, enhance_pdf_icons_with_config,
    PdfImageExtractConfig, PdfIconEnhanceConfig,
    SuperResolutionConfig, SuperResolutionModel,
};

fn main() {
    let pdf_path = Path::new("/Users/neo/Downloads/123.pdf");
    let output_dir = Path::new("/Users/neo/Downloads/123_extracted");
    let enhanced_pdf = Path::new("/Users/neo/Downloads/123_enhanced.pdf");
    
    println!("=== PDF 图标增强测试 ===\n");
    
    // 检查输入文件
    if !pdf_path.exists() {
        eprintln!("错误: PDF 文件不存在: {}", pdf_path.display());
        std::process::exit(1);
    }
    
    println!("输入 PDF: {}", pdf_path.display());
    println!("输出目录: {}", output_dir.display());
    println!("增强 PDF: {}\n", enhanced_pdf.display());
    
    // 阶段 1: 提取 PDF 图像
    println!("--- 阶段 1: PDF 图像提取 ---");
    let extract_config = PdfImageExtractConfig {
        min_dimension: 16,  // 降低阈值以捕获更多图像
        min_bytes: 50,
        filter_unicolor: false,
        ..Default::default()
    };
    
    match extract_pdf_images(pdf_path, output_dir, &extract_config) {
        Ok(result) => {
            println!("✓ 提取成功!");
            println!("  总页数: {}", result.total_pages);
            println!("  提取图像: {} 张", result.images.len());
            println!("  耗时: {}ms", result.processing_time_ms);
            
            for (i, img) in result.images.iter().enumerate() {
                println!("\n  图像 {}:", i + 1);
                println!("    页码: {}", img.page + 1);
                println!("    xref: {}", img.xref);
                println!("    尺寸: {}x{}", img.width, img.height);
                println!("    通道: {}", img.color_channels);
                println!("    格式: {}", img.original_format);
                println!("    路径: {}", img.output_path);
                println!("    大小: {} bytes", img.size_bytes);
            }
        }
        Err(e) => {
            eprintln!("✗ 提取失败: {}", e);
            std::process::exit(1);
        }
    }
    
    // 阶段 2: 图像超分测试
    println!("\n--- 阶段 2: 图像超分测试 ---");
    let sr_config = SuperResolutionConfig {
        model: SuperResolutionModel::Lanczos,
        scale: 4,
        ..Default::default()
    };
    
    let mut resolver = neotrix::neotrix::nt_file_ability::ImageSuperResolver::with_config(sr_config);
    
    // 测试单张图像超分
    let test_image = output_dir.read_dir()
        .expect("无法读取输出目录")
        .filter_map(|e| e.ok())
        .find(|e| {
            let path = e.path();
            path.extension().map(|ext| ext == "png").unwrap_or(false)
        });
    
    if let Some(entry) = test_image {
        let input = entry.path();
        let output = output_dir.join(format!("{}_4x.png", 
            input.file_stem().unwrap().to_string_lossy()));
        
        println!("测试超分: {}", input.display());
        let result = resolver.upscale(&input, &output);
        
        if result.success {
            println!("✓ 超分成功!");
            println!("  输入尺寸: {}x{}", result.input_size.0, result.input_size.1);
            println!("  输出尺寸: {}x{}", result.output_size.0, result.output_size.1);
            println!("  放大倍数: {:.1}x", result.actual_scale);
            println!("  耗时: {}ms", result.processing_time_ms);
            println!("  输出: {}", result.output_path);
        } else {
            println!("✗ 超分失败: {:?}", result.error);
        }
    } else {
        println!("⚠ 未找到 PNG 图像进行超分测试");
    }
    
    // 阶段 3: 完整 PDF 增强
    println!("\n--- 阶段 3: 完整 PDF 增强 ---");
    let enhance_config = PdfIconEnhanceConfig {
        output_pdf: Some(enhanced_pdf.to_path_buf()),
        super_resolution: SuperResolutionConfig {
            model: SuperResolutionModel::Lanczos,
            scale: 4,
            ..Default::default()
        },
        keep_backup: true,
        ..Default::default()
    };
    
    match enhance_pdf_icons_with_config(pdf_path, enhance_config) {
        Ok(result) => {
            println!("✓ PDF 增强完成!");
            println!("  输入: {}", result.input_pdf);
            println!("  输出: {}", result.output_pdf);
            println!("  提取图像: {} 张", result.images_extracted);
            println!("  成功增强: {} 张", result.images_enhanced);
            println!("  失败: {} 张", result.images_failed);
            println!("  总耗时: {}ms", result.total_time_ms);
            
            if !result.enhanced_images.is_empty() {
                println!("\n  增强详情:");
                for img in &result.enhanced_images {
                    println!("    {}x{} → {}x{} ({:.1}x)", 
                        img.original_size.0, img.original_size.1,
                        img.enhanced_size.0, img.enhanced_size.1,
                        img.scale);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ PDF 增强失败: {}", e);
        }
    }
    
    // 输出统计
    println!("\n--- 统计信息 ---");
    let stats = resolver.statistics();
    println!("总处理: {} 张", stats.total_processed);
    println!("成功: {} 张", stats.successful);
    println!("失败: {} 张", stats.failed);
    println!("平均耗时: {}ms", stats.avg_processing_time_ms);
    
    println!("\n=== 测试完成 ===");
}
