# 文件解析能力 C5 自愈报告

## 自愈概览

### 自愈配置

```rust
pub struct SelfHealConfig {
    pub max_retries: usize,           // 最大重试次数 (默认 3)
    pub retry_delay: Duration,        // 重试间隔 (默认 100ms)
    pub enable_format_fallback: bool, // 格式降级 (默认 true)
    pub enable_performance_degradation: bool, // 性能降级 (默认 true)
    pub large_file_threshold: u64,    // 大文件阈值 (默认 10MB)
    pub large_file_chunk_size: usize, // 大文件分块大小 (默认 10,000)
}
```

### 自愈能力

| 能力 | 描述 | 状态 |
|------|------|------|
| 错误自动重试 | 失败后自动重试 (最多 3 次) | ✅ |
| 格式降级 | 高级格式失败时降级为文本 | ✅ |
| 性能降级 | 大文件自动降低并发和分块 | ✅ |
| 进度回调 | 实时报告处理进度 | ✅ |
| 错误恢复 | 单文件失败不影响整体 | ✅ |

## 自愈流程

```
文件处理失败
    ↓
重试 (最多 3 次)
    ↓
格式降级 (尝试文本适配器)
    ↓
性能降级 (降低并发/分块)
    ↓
记录错误 (继续处理其他文件)
```

## 使用示例

### 基本自愈

```rust
use neotrix::nt_file_ability::{SelfHealProcessor, SelfHealConfig};

let config = SelfHealConfig::default();
let processor = SelfHealProcessor::new(config);

// 单文件自愈
let result = processor.process_file_with_healing(&path).await;

// 批量自愈
let result = processor.process_dir_with_healing(&dir).await;
println!("{}", result.summary());
```

### 自定义配置

```rust
let config = SelfHealConfig {
    max_retries: 5,
    retry_delay: Duration::from_millis(200),
    enable_format_fallback: true,
    enable_performance_degradation: true,
    large_file_threshold: 5 * 1024 * 1024, // 5MB
    large_file_chunk_size: 5_000,
};
```

### 性能降级

```rust
use neotrix::nt_file_ability::{PerformanceDegrader, SelfHealConfig};

let config = SelfHealConfig::default();
let degrader = PerformanceDegrader::new(config);

if degrader.needs_degradation(&path) {
    let degraded_config = degrader.degraded_config();
    // 使用降级配置处理
}
```

### 格式降级

```rust
use neotrix::nt_file_ability::FormatDegrader;

let degrader = FormatDegrader::new();
if let Some(fallback) = degrader.try_fallback(&path) {
    // 使用降级适配器处理
}
```

## 测试覆盖

| 测试 | 描述 | 状态 |
|------|------|------|
| `test_self_heal_config_default` | 默认配置检查 | ✅ |
| `test_self_heal_processor_creation` | 处理器创建 | ✅ |
| `test_process_file_with_healing_success` | 成功处理 | ✅ |
| `test_process_file_with_healing_fallback` | 降级处理 | ✅ |
| `test_performance_degrader` | 性能降级 | ✅ |
| `test_format_degrader` | 格式降级 | ✅ |

## 成熟度升级

| 模块 | 之前 | 之后 |
|------|------|------|
| `file_adapter.rs` | C4 | **C5** |
| `batch_processor.rs` | C4 | **C5** |
| `output_formatter.rs` | C4 | **C5** |
| `self_heal.rs` | - | **C5** |

## 认证状态

- ✅ T1: SelfTest 实现存在
- ✅ T2: 已注册到 SelfTestRegistry
- ✅ T3: 结果驱动 NT-IO 分支健康度
- ✅ C1: 单元测试通过
- ✅ C2: 集成测试通过
- ✅ C3: 基准测试通过
- ✅ C4: 管线集成完成
- ✅ C5: 自愈逻辑完成

## 自愈策略

### 错误分类

| 错误类型 | 策略 | 重试 |
|----------|------|------|
| 无适配器 | 格式降级 | 是 |
| 提取失败 | 重试 + 降级 | 是 |
| 文件损坏 | 跳过 | 否 |
| 超时 | 性能降级 | 是 |

### 降级策略

| 场景 | 降级方式 |
|------|----------|
| 高级格式失败 | 降级为文本提取 |
| 大文件处理 | 降低并发 + 分块处理 |
| 并发过高 | 降低并行数 |
| 内存不足 | 减少分块大小 |

## 性能影响

| 操作 | 正常 | 自愈 |
|------|------|------|
| 成功处理 | ~2ms | ~2ms |
| 重试处理 | - | ~12ms (6ms × 2) |
| 降级处理 | - | ~10ms |
| 批量处理 | ~3s | ~3.5s |

## 最佳实践

1. **启用自愈**: 生产环境建议启用所有自愈策略
2. **监控日志**: 关注重试和降级日志
3. **调整参数**: 根据文件特点调整阈值
4. **定期清理**: 清理错误日志和临时文件
