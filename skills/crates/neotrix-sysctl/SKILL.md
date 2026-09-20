# neotrix-sysctl

## Purpose
系统指标 FFI 专用 crate

## Trigger Words
- neotrix-sysctl
- sysctl
- system metrics
- FFI
- 系统指标
- 内存监控

## Content
- macOS: `sysctl` KERN_PROC_PID 读取
- Linux: `/proc/self/status` 解析
- 安全接口封装

## Key Functions
| Function | Purpose |
|----------|---------|
| `current_rss_bytes()` | 当前进程 RSS（常驻内存） |
| `physical_memory_bytes()` | 物理内存总量 |

## Safety
- 唯一允许 `unsafe` 的 crate
- FFI 调用封装在本 crate 内部
- 上层 crate 保持 `#![forbid(unsafe_code)]`

## Location
`crates/neotrix-sysctl/`

## Usage
```rust
use neotrix_sysctl::{current_rss_bytes, physical_memory_bytes};

let rss = current_rss_bytes();
let total = physical_memory_bytes();
```
