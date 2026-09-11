# PDF Icon Enhance — PDF 图标/图像优化模块

## 概述
基于 NeoTrix 内部能力架构的 PDF 图标/图像增强工具。通过 **Extract → Super-resolve → Embed-back** 三阶段流程，提升 PDF 中嵌入图像的清晰度。

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│  NeoTrix Consciousness Core (dispatch capability)          │
│  dispatch_internal_capability("pdf_enhance", summary)      │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────▼───────────────┐
        │   pdf_icon_enhance (orchestrator)  │
        └───────────────┬───────────────┘
                        │
    ┌───────────────────┼───────────────────┐
    │                   │                   │
    ▼                   ▼                   ▼
pdf_image_extract  image_super_res  embed_back (TODO)
    │                   │                   │
    │                   │                   │
    ▼                   ▼                   ▼
  JPEG/PNG          Lanczos/ONNX      lopdf insert
```

## 文件说明

### Rust 源码 (`src/`)

| 文件 | 功能 | 状态 |
|------|------|------|
| `pdf_image_extract.rs` | 从 PDF 提取 XObject 图像 | ✅ 已实现 |
| `image_super_resolution.rs` | 图像超分辨率 (Lanczos/ONNX) | ⚠️ Lanczos 可用, ONNX 待模型 |
| `pdf_icon_enhance.rs` | 流程编排器 | ⚠️ Embed-back TODO |

### Python 脚本 (`scripts/`)

| 文件 | 用途 |
|------|------|
| `pdf_enhance_full.py` | 完整流程 (嵌回有问题) |
| `pdf_enhance_v2.py` | 优化版本 (正确嵌回) ✅ |
| `create_comparison.py` | 创建对比图 |
| `test_pdf_enhance.py` | 提取测试 |

### 测试 (`tests/`)

| 文件 | 说明 |
|------|------|
| `123.pdf` | 原始 PDF (72KB) |
| `123_enhanced_v2.pdf` | 增强后 PDF (897KB) |

### 文档 (`docs/`)

| 文件 | 说明 |
|------|------|
| `pdf_comparison_v2.png` | 对比效果图 |

## 快速使用

### Python (推荐)

```bash
# 完整流程
python scripts/pdf_enhance_v2.py input.pdf output.pdf --scale 4

# 只提取图像
python scripts/test_pdf_enhance.py input.pdf

# 创建对比图
python scripts/create_comparison.py original.pdf enhanced.pdf output.png
```

### Rust (开发中)

```rust
use neotrix::nt_file_ability::pdf_icon_enhance::{enhance_pdf_icons, EnhanceConfig};

let config = EnhanceConfig {
    scale_factor: 4,
    model_path: None, // 使用 Lanczos
    output_path: "enhanced.pdf".into(),
};

let result = enhance_pdf_icons("input.pdf", &config)?;
```

## NeoTrix 集成

### 模块位置
- Rust 源码: `neotrix-core/src/neotrix/nt_file_ability/`
- 模块声明: `nt_file_ability.rs` 中 `mod pdf_image_extract; mod image_super_resolution; mod pdf_icon_enhance;`
- Dispatch 路由: `nt_core_consciousness_core.rs` L1638-1703

### 调用方式

```bash
# 通过 NeoTrix CLI
/file enhance input.pdf --scale 4

# 通过 Consciousness Core dispatch
dispatch_internal_capability("pdf_enhance", "input.pdf --scale 4")
```

## 依赖

### Rust
- `lopdf` — PDF 解析和修改
- `image` — 图像处理
- `ort` (可选) — ONNX Runtime 推理
- `ndarray` (可选) — 数组操作

### Python
- `PyMuPDF (fitz)` — PDF 处理
- `Pillow` — 图像处理

## 技术细节

### 图像提取
1. 遍历 PDF 页面的 `/XObject` 字典
2. 过滤 JPEG/PNG 图像 (通过 `/Filter` 字段)
3. 按尺寸过滤 (忽略小图标)
4. 解码并保存为独立图像文件

### 超分辨率
- **Lanczos**: 4x 上采样，保留细节
- **Bicubic**: 3x 上采样，较平滑
- **ONNX Real-ESRGAN**: AI 超分辨率 (需要模型文件)

### Embed-back (Python v2)
```python
# 正确方式：创建新页面，用 insert_image
page = doc.new_page(width=page_width, height=page_height)
page.insert_image(rect, filename=enhanced_path)
```

## 已知问题

1. **Rust embed-back TODO**: `pdf_icon_enhance.rs` 中的嵌回功能尚未实现
2. **ONNX 模型未下载**: Real-ESRGAN ONNX 模型需要单独下载
3. **Rust 编译超时**: `cargo check` 可能超时，需要 `cargo clean` 后重试

## 下一步

- [ ] 实现 Rust 版 embed-back (使用 lopdf insert_image)
- [ ] 下载并集成 Real-ESRGAN ONNX 模型
- [ ] 添加 consciousness core dispatch 路由测试
- [ ] 性能基准测试

## 参考

- NeoTrix Dev Rules: R-P1 (零 unsafe), R-P42 (复用类型), R-P110 (禁止 CLI 命令)
- Dark Forest 规则: 模块必须编译+测试+连接
- R-P79: 外部技术吸收必须同 session 接线到生产路径
