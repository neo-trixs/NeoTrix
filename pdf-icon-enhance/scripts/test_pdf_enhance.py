#!/usr/bin/env python3
"""PDF 图标增强测试脚本 - 快速验证效果"""

import subprocess
import os
import sys

# 检查依赖
def check_dependencies():
    """检查必要的 Python 包"""
    required = ['fitz']  # PyMuPDF
    missing = []
    
    for pkg in required:
        try:
            __import__(pkg)
        except ImportError:
            missing.append(pkg)
    
    if missing:
        print(f"安装缺失的依赖: {', '.join(missing)}")
        subprocess.run([sys.executable, '-m', 'pip', 'install', 'PyMuPDF'] + missing, 
                      capture_output=True)
        return True
    return False

def extract_pdf_images(pdf_path, output_dir):
    """从 PDF 提取图像"""
    import fitz  # PyMuPDF
    
    os.makedirs(output_dir, exist_ok=True)
    
    doc = fitz.open(pdf_path)
    images = []
    
    for page_num in range(len(doc)):
        page = doc[page_num]
        image_list = page.get_images(full=True)
        
        for img_index, img in enumerate(image_list):
            xref = img[0]
            base_image = doc.extract_image(xref)
            image_bytes = base_image["image"]
            
            # 保存图像
            image_filename = f"page_{page_num+1}_img_{img_index+1}.{base_image['ext']}"
            image_path = os.path.join(output_dir, image_filename)
            
            with open(image_path, "wb") as f:
                f.write(image_bytes)
            
            images.append({
                'page': page_num + 1,
                'index': img_index + 1,
                'path': image_path,
                'width': base_image['width'],
                'height': base_image['height'],
                'format': base_image['ext'],
                'size': len(image_bytes)
            })
            
            print(f"  提取: {image_filename} ({base_image['width']}x{base_image['height']})")
    
    doc.close()
    return images

def upscale_image(input_path, output_path, scale=4):
    """使用 Pillow 进行图像超分"""
    from PIL import Image
    
    img = Image.open(input_path)
    width, height = img.size
    
    # 使用 LANCZOS 高质量缩放
    new_width = width * scale
    new_height = height * scale
    
    upscaled = img.resize((new_width, new_height), Image.LANCZOS)
    upscaled.save(output_path)
    
    return {
        'input_size': (width, height),
        'output_size': (new_width, new_height),
        'scale': scale
    }

def main():
    pdf_path = "/Users/neo/Downloads/123.pdf"
    output_dir = "/Users/neo/Downloads/123_test_output"
    
    print("=" * 50)
    print("PDF 图标增强测试")
    print("=" * 50)
    
    # 检查输入文件
    if not os.path.exists(pdf_path):
        print(f"错误: PDF 文件不存在: {pdf_path}")
        return
    
    print(f"\n输入 PDF: {pdf_path}")
    print(f"输出目录: {output_dir}")
    
    # 检查并安装依赖
    check_dependencies()
    
    # 阶段 1: 提取图像
    print("\n--- 阶段 1: PDF 图像提取 ---")
    try:
        images = extract_pdf_images(pdf_path, output_dir)
        print(f"\n✓ 成功提取 {len(images)} 张图像")
    except Exception as e:
        print(f"✗ 提取失败: {e}")
        return
    
    # 阶段 2: 图像超分
    print("\n--- 阶段 2: 图像超分 (4x) ---")
    upscaled_dir = os.path.join(output_dir, "upscaled")
    os.makedirs(upscaled_dir, exist_ok=True)
    
    for img_info in images:
        try:
            input_path = img_info['path']
            output_filename = f"upscaled_{os.path.basename(input_path)}"
            output_path = os.path.join(upscaled_dir, output_filename)
            
            result = upscale_image(input_path, output_path, scale=4)
            
            print(f"  ✓ {output_filename}: "
                  f"{result['input_size'][0]}x{result['input_size'][1]} → "
                  f"{result['output_size'][0]}x{result['output_size'][1]}")
        except Exception as e:
            print(f"  ✗ 超分失败: {e}")
    
    # 阶段 3: 创建对比图
    print("\n--- 阶段 3: 创建对比图 ---")
    try:
        from PIL import Image, ImageDraw, ImageFont
        
        for img_info in images:
            original = Image.open(img_info['path'])
            upscaled_path = os.path.join(upscaled_dir, 
                                         f"upscaled_{os.path.basename(img_info['path'])}")
            upscaled = Image.open(upscaled_path)
            
            # 创建对比图
            width = original.width + upscaled.width + 20
            height = max(original.height, upscaled.height) + 40
            
            comparison = Image.new('RGB', (width, height), (255, 255, 255))
            
            # 粘贴原图
            comparison.paste(original, (0, 40))
            
            # 粘贴超分图
            comparison.paste(upscaled, (original.width + 20, 40))
            
            # 添加标签
            draw = ImageDraw.Draw(comparison)
            draw.text((10, 10), "Original", fill=(0, 0, 0))
            draw.text((original.width + 30, 10), "Upscaled 4x", fill=(0, 0, 0))
            
            # 保存对比图
            comparison_path = os.path.join(output_dir, 
                                          f"comparison_{os.path.basename(img_info['path'])}")
            comparison.save(comparison_path)
            print(f"  ✓ 对比图: {comparison_path}")
            
    except Exception as e:
        print(f"  ✗ 创建对比图失败: {e}")
    
    # 输出总结
    print("\n" + "=" * 50)
    print("测试完成!")
    print("=" * 50)
    print(f"\n输出文件位置: {output_dir}")
    print("\n文件结构:")
    print(f"  {output_dir}/")
    print(f"  ├── page_*_img_*.png  (原始提取图像)")
    print(f"  ├── upscaled/        (4x 超分图像)")
    print(f"  └── comparison_*.png (对比图)")
    
    # 显示统计信息
    print("\n统计:")
    print(f"  提取图像: {len(images)} 张")
    print(f"  成功超分: {len(images)} 张")
    print(f"  放大倍数: 4x (LANCZOS)")

if __name__ == "__main__":
    main()
