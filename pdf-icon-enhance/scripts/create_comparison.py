#!/usr/bin/env python3
"""PDF 增强效果预览 - 创建对比图"""

import pymupdf
from PIL import Image
import io
import os

def compare_pdfs(original_pdf, enhanced_pdf, output_image):
    """创建原始和增强 PDF 的对比图"""
    
    # 打开两个 PDF
    doc_orig = pymupdf.open(original_pdf)
    doc_enh = pymupdf.open(enhanced_pdf)
    
    # 渲染第一页为图像
    page_orig = doc_orig[0]
    page_enh = doc_enh[0]
    
    # 设置渲染分辨率 (DPI)
    dpi = 150
    
    # 渲染为图像
    mat = pymupdf.Matrix(dpi/72, dpi/72)
    
    pix_orig = page_orig.get_pixmap(matrix=mat)
    img_orig = Image.frombytes("RGB", [pix_orig.width, pix_orig.height], pix_orig.samples)
    
    pix_enh = page_enh.get_pixmap(matrix=mat)
    img_enh = Image.frombytes("RGB", [pix_enh.width, pix_enh.height], pix_enh.samples)
    
    # 创建对比图
    width = img_orig.width + img_enh.width + 40
    height = max(img_orig.height, img_enh.height) + 80
    
    comparison = Image.new('RGB', (width, height), (255, 255, 255))
    
    # 粘贴原始 PDF
    comparison.paste(img_orig, (0, 60))
    
    # 粘贴增强 PDF
    comparison.paste(img_enh, (img_orig.width + 40, 60))
    
    # 添加标签
    from PIL import ImageDraw, ImageFont
    draw = ImageDraw.Draw(comparison)
    
    # 尝试使用系统字体
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 24)
    except:
        font = ImageFont.load_default()
    
    draw.text((10, 10), "原始 PDF (72KB)", fill=(0, 0, 255), font=font)
    draw.text((img_orig.width + 50, 10), "增强 PDF (816KB) - 4x 超分", fill=(0, 128, 0), font=font)
    
    # 添加分隔线
    draw.line([(img_orig.width + 20, 60), (img_orig.width + 20, height)], fill=(200, 200, 200), width=2)
    
    # 保存
    comparison.save(output_image, quality=95)
    
    doc_orig.close()
    doc_enh.close()
    
    return {
        'original_size': (img_orig.width, img_orig.height),
        'enhanced_size': (img_enh.width, img_enh.height),
        'output': output_image
    }

def main():
    original = "/Users/neo/Downloads/123.pdf"
    enhanced = "/Users/neo/Downloads/123_enhanced.pdf"
    output = "/Users/neo/Downloads/pdf_comparison.png"
    
    print("创建 PDF 增强对比图...")
    
    result = compare_pdfs(original, enhanced, output)
    
    print(f"✓ 对比图已生成: {result['output']}")
    print(f"  原始 PDF 渲染尺寸: {result['original_size'][0]}x{result['original_size'][1]}")
    print(f"  增强 PDF 渲染尺寸: {result['enhanced_size'][0]}x{result['enhanced_size'][1]}")

if __name__ == "__main__":
    main()
