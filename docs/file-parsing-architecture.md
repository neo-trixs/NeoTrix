# NeoTrix 统一文件解析架构设计

## 问题总结

### 当前痛点
1. **格式碎片化**: xlsx 用 calamine, docx 用 office_oxide, pdf 用 PyMuPDF, csv 用 Rust csv
2. **批量处理缺失**: 无进度追踪、无错误恢复、无并行处理
3. **格式转换断层**: xls→xlsx 需要外部工具，无统一转换管线
4. **内存效率低**: 大文件全量加载，无法流式处理
5. **输出格式单一**: 无内置 styled Excel/CSV 输出能力

### 今日经验
- calamine `open_workbook_auto` 统一 .xls/.xlsx/.xlsm/.xlsb/.ods 读取
- office_oxide 提供 6 格式文本提取 (text/markdown/html)
- styled Excel 输出需要专门的样式引擎
- 批量处理需要跳过空行、进度追踪、错误隔离

## 统一架构设计

### 核心原则
1. **单一事实源**: 所有格式通过统一适配器注册表
2. **分层管线**: 类型探测 → 格式路由 → 结构提取 → 输出格式化
3. **错误隔离**: 单文件失败不影响批量处理
4. **内存安全**: 大文件流式处理，分块读取

### 架构图
```
┌─────────────────────────────────────────────────────────┐
│                    Unified File Parser                    │
├─────────────────────────────────────────────────────────┤
│  L0: Type Detection (magika/content fingerprint)         │
│      ↓                                                  │
│  L1: Format Router (extension + magic bytes)             │
│      ↓                                                  │
│  L2: Adapter Registry (pluggable parsers)                │
│      ├── Excel: calamine (auto mode)                     │
│      ├── Word: office_oxide                              │
│      ├── PDF: pdf-extract / poppler                      │
│      ├── CSV: csv-rs                                     │
│      ├── Image: image-rs + OCR                           │
│      └── Text: encoding-sniff                            │
│      ↓                                                  │
│  L3: Unified FileModel                                   │
│      ├── meta: type, size, hash, mtime                   │
│      ├── text: normalized text (markdown)                │
│      ├── structure: type-specific structure              │
│      └── evidence: provenance anchors                    │
│      ↓                                                  │
│  L4: Output Formatter                                    │
│      ├── styled-xlsx: themed Excel output                │
│      ├── csv: UTF-8 BOM                                  │
│      ├── json: structured data                           │
│      └── markdown: documentation                         │
└─────────────────────────────────────────────────────────┘
```

### 适配器注册表
```rust
pub trait FileAdapter: Send + Sync {
    /// 检测是否支持该格式
    fn can_handle(&self, path: &Path, magic: &[u8]) -> bool;
    
    /// 提取文本 (markdown 格式)
    fn extract_text(&self, path: &Path) -> Result<String>;
    
    /// 提取结构化数据 (可选)
    fn extract_structure(&self, path: &Path) -> Result<Option<FileStructure>>;
    
    /// 获取支持的格式
    fn supported_formats(&self) -> &[&str];
}

pub struct AdapterRegistry {
    adapters: Vec<Box<dyn FileAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut registry = Self { adapters: Vec::new() };
        
        // 注册适配器 (按优先级排序)
        registry.register(Box::new(ExcelAdapter::new()));  // calamine auto
        registry.register(Box::new(WordAdapter::new()));   // office_oxide
        registry.register(Box::new(PdfAdapter::new()));    // pdf-extract
        registry.register(Box::new(CsvAdapter::new()));    // csv-rs
        registry.register(Box::new(TextAdapter::new()));   // encoding-sniff
        
        registry
    }
    
    pub fn detect(&self, path: &Path) -> Option<&dyn FileAdapter> {
        // 1. 先用扩展名快速匹配
        // 2. 再用 magic bytes 精确匹配
        // 3. 返回第一个匹配的适配器
    }
}
```

### 批量处理管线
```rust
pub struct BatchProcessor {
    registry: AdapterRegistry,
    config: BatchConfig,
}

pub struct BatchConfig {
    pub max_parallel: usize,           // 并行数 (默认 CPU 核数)
    pub chunk_size: usize,             // 分块大小 (默认 1000 文件)
    pub skip_errors: bool,             // 跳过错误文件
    pub progress_callback: Option<Box<dyn Fn(Progress)>>,
}

pub struct Progress {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub current_file: PathBuf,
    pub elapsed: Duration,
}

impl BatchProcessor {
    pub async fn process_dir(
        &self,
        dir: &Path,
        output: &Path,
        format: OutputFormat,
    ) -> Result<BatchResult> {
        // 1. 扫描目录
        let files = self.scan_dir(dir)?;
        
        // 2. 分块处理
        for chunk in files.chunks(self.config.chunk_size) {
            // 3. 并行提取
            let results = self.process_chunk(chunk).await?;
            
            // 4. 合并输出
            self.merge_results(&results, output, format)?;
        }
        
        Ok(BatchResult { ... })
    }
}
```

### 输出格式化器
```rust
pub trait OutputFormatter: Send + Sync {
    fn format(&self, data: &[FileModel], output: &Path) -> Result<()>;
}

pub struct StyledExcelFormatter {
    pub theme: Theme,
    pub header_style: CellStyle,
    pub data_style: CellStyle,
}

pub struct Theme {
    pub primary: String,      // 主色 (表头背景)
    pub accent: String,       // 强调色
    pub bg_even: String,      // 偶数行背景
    pub bg_odd: String,       // 奇数行背景
    pub font: String,         // 字体
}
```

## 实现计划

### Phase 1: 统一适配器注册表 (C4)
- [ ] 定义 `FileAdapter` trait
- [ ] 实现 `ExcelAdapter` (calamine auto)
- [ ] 实现 `WordAdapter` (office_oxide)
- [ ] 实现 `CsvAdapter` (csv-rs)
- [ ] 实现 `TextAdapter` (encoding-sniff)
- [ ] 注册表自动检测

### Phase 2: 批量处理管线 (C4)
- [ ] 目录扫描 (递归/过滤)
- [ ] 分块处理 + 进度追踪
- [ ] 错误隔离 + 重试机制
- [ ] 并行处理 (tokio)

### Phase 3: 输出格式化 (C5)
- [ ] Styled Excel 输出 (office_oxide)
- [ ] CSV 输出 (UTF-8 BOM)
- [ ] JSON 输出 (结构化)
- [ ] 主题系统 (可配置)

### Phase 4: 高级特性 (C5)
- [ ] 流式处理 (大文件)
- [ ] 格式转换管线
- [ ] OCR 支持 (图像文件)
- [ ] 增量处理 (只处理变更)

## CLI 命令设计

### 统一解析命令
```bash
# 单文件解析
/file parse <path> [--format json|csv|markdown]

# 批量解析
/file parse-batch <dir> [--output <path>] [--format xlsx|csv|json] [--parallel 4]

# 格式转换
/file convert <input> <output> [--theme default|dark|light]

# 样式化输出
/file styled-output <dir> [--output <path>] [--theme neo-trix]
```

### 示例
```bash
# 解析单个 Excel 文件
/file parse report.xlsx --format json

# 批量解析目录，输出 styled Excel
/file parse-batch /path/to/excels --output summary.xlsx --theme neo-trix

# 转换 xls 到 xlsx
/file convert old.xls new.xlsx
```

## 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| Excel 读取 | calamine (auto) | 统一 .xls/.xlsx/.xlsm/.xlsb/.ods |
| Excel 写入 | office_oxide | 风格化输出，零 C 依赖 |
| Word/PPT | office_oxide | 6 格式统一 API |
| PDF | pdf-extract | 纯 Rust，表格提取 |
| CSV | csv-rs | 高性能，UTF-8 支持 |
| 图像 | image-rs | 解码 + 元数据 |
| OCR | tesseract-rs | 图像文字识别 |
| 并行 | tokio | 异步处理 |
| 进度 | indicatif | 终端进度条 |

## 性能目标

| 场景 | 目标 | 当前 |
|------|------|------|
| 单文件解析 | <10ms | ~5ms (calamine) |
| 批量 1000 文件 | <5s | ~30s (串行) |
| 大文件 (100MB) | <1s | OOM (全量加载) |
| 格式转换 | <50ms | 无 |

## 安全考虑

1. **文件大小限制**: 单文件最大 500MB
2. **内存限制**: 流式处理，分块读取
3. **路径验证**: 防止路径遍历攻击
4. **内容消毒**: 清除恶意公式/宏
