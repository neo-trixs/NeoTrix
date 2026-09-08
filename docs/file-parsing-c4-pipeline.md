# 文件解析能力 C4 管线集成报告

## 集成概览

### 新增 CLI 命令

```bash
/file parse-batch <目录> [选项]
```

**选项**:
- `--parallel <N>`: 并发数 (默认 8)
- `--chunk <N>`: 分块大小 (默认 500)
- `--output <path>`: 输出路径 (默认 parsed_output.xlsx)
- `--format <csv|json|md|xlsx>`: 输出格式 (默认 xlsx)

**示例**:
```bash
# 批量解析当前目录
/file parse-batch .

# 16 并发，输出为 JSON
/file parse-batch ./data --parallel 16 --format json

# 自定义输出路径
/file parse-batch ./data --output ./results/output.xlsx
```

## 集成点

### 1. CLI 命令系统
- 新增 `parse_batch_cmd.rs` 模块
- 注册到 `consolidated_cmds.rs` 路由
- 添加到 `/file` 帮助文本

### 2. 文件能力模块
- 复用 `AdapterRegistry` 格式检测
- 复用 `BatchProcessor` 批量处理
- 复用 `OutputFormatter` 输出格式化

### 3. 主题系统
- 默认使用 `Theme::neo_trix()`
- 支持 4 种主题 (default/dark/light/neo_trix)

## 命令架构

```
/file parse-batch <dir>
    ↓
ParseBatchCmd.execute()
    ↓
BatchProcessor::process_dir()
    ↓
AdapterRegistry::detect() → ExcelAdapter / CsvAdapter / TextAdapter
    ↓
BatchResult { models, success, failed }
    ↓
OutputFormatter::format()
    ↓
输出文件 (xlsx/csv/json/md)
```

## 测试覆盖

| 测试类型 | 数量 | 文件 |
|----------|------|------|
| 单元测试 | 19 | file_adapter/batch_processor/output_formatter tests |
| 集成测试 | 10 | integration_tests.rs |
| 基准测试 | 10 | benchmarks.rs |
| **总计** | **39** | - |

## 成熟度升级

| 模块 | 之前 | 之后 |
|------|------|------|
| `file_adapter.rs` | C3 | **C4** |
| `batch_processor.rs` | C3 | **C4** |
| `output_formatter.rs` | C3 | **C4** |

## 认证状态

- ✅ T1: SelfTest 实现存在
- ✅ T2: 已注册到 SelfTestRegistry
- ✅ T3: 结果驱动 NT-IO 分支健康度
- ✅ C1: 单元测试通过
- ✅ C2: 集成测试通过
- ✅ C3: 基准测试通过
- ✅ C4: 管线集成完成

## 下一步

### C5 自愈
- [ ] 错误自动重试
- [ ] 格式降级 (高级格式→文本后备)
- [ ] 性能降级 (大文件分块处理)
- [ ] 进度回调
- [ ] 错误恢复

## 使用指南

### 基本用法
```bash
# 解析当前目录所有文件
/file parse-batch .

# 解析指定目录
/file parse-batch /path/to/data
```

### 高级选项
```bash
# 16 并发处理
/file parse-batch ./data --parallel 16

# 分块大小 1000
/file parse-batch ./data --chunk 1000

# 输出为 CSV 格式
/file parse-batch ./data --format csv --output results.csv

# 输出为 JSON 格式
/file parse-batch ./data --format json --output results.json
```

### 性能调优
```bash
# 小文件 (100-500 文件): 8 并发
/file parse-batch ./small --parallel 8

# 中等文件 (500-2000 文件): 16 并发
/file parse-batch ./medium --parallel 16

# 大文件 (2000+ 文件): 32 并发
/file parse-batch ./large --parallel 32
```
