#!/usr/bin/env python3
"""PDF 图标增强 v2 - 正确的图像嵌回"""

import pymupdf
from PIL import Image
import io
import os

def extract_and_enhance_pdf_v2(input_pdf, output_pdf, scale=4):
    """
    正确的 PDF 图标增强流程:
    1. 提取图像并保存
    2. 超分处理
    3. 创建新的 PDF 页面，包含增强后的图像
    """
    
    print(f"输入: {input_pdf}")
    print(f"输出: {output_pdf}")
    print(f"放大倍数: {scale}x")
    print("-" * 50)
    
    # 创建输出目录
    temp_dir = "/Users/neo/Downloads/pdf_temp"
    os.makedirs(temp_dir, exist_ok=True)
    
    # 打开原始 PDF
    doc_orig = pymupdf.open(input_pdf)
    page_orig = doc_orig[0]
    
    # 获取原始页面尺寸
    page_rect = page_orig.rect
    print(f"原始页面尺寸: {page_rect.width:.0f} x {page_rect.height:.0f} points")
    
    # 提取图像
    image_list = page_orig.get_images(full=True)
    print(f"发现 {len(image_list)} 个图像")
    
    if not image_list:
        print("未找到图像，跳过")
        doc_orig.close()
        return
    
    # 提取并保存图像
    enhanced_images = []
    
    for img_index, img in enumerate(image_list):
        xref = img[0]
        
        try:
            # 提取图像
            base_image = doc_orig.extract_image(xref)
            image_bytes = base_image["image"]
            ext = base_image["ext"]
            orig_width = base_image["width"]
            orig_height = base_image["height"]
            
            print(f"\n图像 {img_index + 1}: {orig_width}x{orig_height} ({ext})")
            
            # 用 Pillow 打开
            pil_image = Image.open(io.BytesIO(image_bytes))
            
            # 超分
            new_width = orig_width * scale
            new_height = orig_height * scale
            
            print(f"  超分中: {orig_width}x{orig_height} → {new_width}x{new_height}")
            upscaled = pil_image.resize((new_width, new_height), Image.LANCZOS)
            
            # 保存超分后的图像
            img_path = os.path.join(temp_dir, f"enhanced_{img_index}.{ext}")
            if ext.lower() in ['jpg', 'jpeg']:
                upscaled.save(img_path, format='JPEG', quality=95)
            else:
                upscaled.save(img_path, format='PNG')
            
            enhanced_images.append({
                'path': img_path,
                'orig_width': orig_width,
                'orig_height': orig_height,
                'new_width': new_width,
                'new_height': new_height
            })
            
            print(f"  ✓ 已保存: {img_path}")
            
        except Exception as e:
            print(f"  ✗ 失败: {e}")
    
    doc_orig.close()
    
    if not enhanced_images:
        print("没有图像被增强")
        return
    
    # 创建新的增强 PDF
    print("\n" + "-" * 50)
    print("创建增强 PDF...")
    
    # 使用原始 PDF 的页面尺寸
    doc_new = pymupdf.open()
    
    for img_info in enhanced_images:
        # 创建新页面 (使用原始页面尺寸)
        page_new = doc_new.new_page(width=page_rect.width, height=page_rect.height)
        
        # 插入增强后的图像 (适应页面大小)
        page_new.insert_image(page_rect, filename=img_info['path'])
        
        print(f"  ✓ 插入图像: {img_info['new_width']}x{img_info['new_height']}")
    
    # 保存增强后的 PDF
    doc_new.save(output_pdf, garbage=4, deflate=True)
    doc_new.close()
    
    # 清理临时文件
    for img_info in enhanced_images:
        os.remove(img_info['path'])
    os.rmdir(temp_dir)
    
    print("\n" + "=" * 50)
    print(f"✓ PDF 增强完成!")
    print(f"  输出: {output_pdf}")
    print(f"  文件大小: {os.path.getsize(output_pdf) / 1024:.1f} KB")
    print("=" * 50)

def main():
    input_pdf = "/Users/neo/Downloads/123.pdf"
    output_pdf = "/Users/neo/Downloads/123_enhanced_v2.pdf"
    
    print("=" * 50)
    print("PDF 图标增强 v2 - 正确嵌回")
    print("=" * 50)
    
    extract_and_enhance_pdf_v2(input_pdf, output_pdf, scale=4)

if __name__ == "__main__":
    main()
