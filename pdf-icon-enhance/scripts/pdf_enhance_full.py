#!/usr/bin/env python3
"""PDF 图标增强完整流程 - 提取 → 超分 → 嵌回"""

import pymupdf
from PIL import Image
import os
import io

def extract_and_enhance_pdf(input_pdf, output_pdf, scale=4):
    """
    完整的 PDF 图标增强流程:
    1. 提取 PDF 中的图像
    2. 对图像进行超分
    3. 将增强后的图像嵌回 PDF
    """
    
    print(f"输入: {input_pdf}")
    print(f"输出: {output_pdf}")
    print(f"放大倍数: {scale}x")
    print("-" * 50)
    
    # 打开 PDF
    doc = pymupdf.open(input_pdf)
    
    enhanced_count = 0
    
    for page_num in range(len(doc)):
        page = doc[page_num]
        print(f"\n处理第 {page_num + 1} 页...")
        
        # 获取页面上的所有图像
        image_list = page.get_images(full=True)
        print(f"  发现 {len(image_list)} 个图像")
        
        for img_index, img in enumerate(image_list):
            xref = img[0]
            
            try:
                # 提取图像
                base_image = doc.extract_image(xref)
                image_bytes = base_image["image"]
                ext = base_image["ext"]
                orig_width = base_image["width"]
                orig_height = base_image["height"]
                
                print(f"  图像 {img_index + 1}: {orig_width}x{orig_height} ({ext})")
                
                # 用 Pillow 打开图像
                pil_image = Image.open(io.BytesIO(image_bytes))
                
                # 超分
                new_width = orig_width * scale
                new_height = orig_height * scale
                
                upscaled = pil_image.resize((new_width, new_height), Image.LANCZOS)
                
                # 转换回字节
                img_buffer = io.BytesIO()
                if ext.lower() in ['jpg', 'jpeg']:
                    upscaled.save(img_buffer, format='JPEG', quality=95)
                else:
                    upscaled.save(img_buffer, format='PNG')
                
                new_image_bytes = img_buffer.getvalue()
                
                # 替换 PDF 中的图像
                doc.update_stream(xref, new_image_bytes)
                
                enhanced_count += 1
                print(f"    ✓ 增强完成: {orig_width}x{orig_height} → {new_width}x{new_height}")
                
            except Exception as e:
                print(f"    ✗ 增强失败: {e}")
    
    # 保存增强后的 PDF
    print("-" * 50)
    doc.save(output_pdf)
    doc.close()
    
    print(f"\n✓ PDF 增强完成!")
    print(f"  输出文件: {output_pdf}")
    print(f"  增强图像: {enhanced_count} 张")
    print(f"  文件大小: {os.path.getsize(output_pdf) / 1024:.1f} KB")
    
    return enhanced_count

def main():
    input_pdf = "/Users/neo/Downloads/123.pdf"
    output_pdf = "/Users/neo/Downloads/123_enhanced.pdf"
    
    print("=" * 50)
    print("PDF 图标增强 - 完整流程")
    print("=" * 50)
    
    # 检查输入文件
    if not os.path.exists(input_pdf):
        print(f"错误: 输入文件不存在: {input_pdf}")
        return
    
    # 执行增强
    try:
        enhanced_count = extract_and_enhance_pdf(input_pdf, output_pdf, scale=4)
        
        if enhanced_count > 0:
            print("\n" + "=" * 50)
            print("成功! 可以打开增强后的 PDF 查看效果")
            print("=" * 50)
        else:
            print("\n警告: 没有图像被增强")
            
    except Exception as e:
        print(f"\n错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()
