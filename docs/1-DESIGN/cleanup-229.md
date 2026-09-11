# Cleanup #229: L2→L1 跨域引用修复

## 问题

L2 感知层 (`nt_world`) 中的 `nt_world_github_absorber.rs` 直接引用了 L1 行动层的具体类型和函数：

```rust
// 违规代码
use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadOptions;
use crate::l1_action::nt_memory::nt_memory_kb::nt_http::shared_blocking_client();
use crate::l1_action::nt_memory::nt_memory_kb::nt_http::run_blocking();
use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env();
```

这违反了六层架构原则：上层不应直接依赖下层具体实现。

## 修复方案

### 1. 扩展 `l1_facade.rs`

在 `neotrix-core/src/l2_perception/nt_world/l1_facade.rs` 中添加 HTTP 相关类型的 re-export：

```rust
// HTTP 集中门面 — nt_http 类型与函数
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadOptions;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::download_to_file;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadResult;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::shared_blocking_client;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::run_blocking;

// HTTP 工厂门面 — proxy_from_env
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;
```

### 2. 修改 `nt_world_github_absorber.rs`

替换直接引用为 facade 引用：

```rust
// Before
use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadOptions;

// After
use super::l1_facade::{DownloadOptions, download_to_file, shared_blocking_client, run_blocking, proxy_from_env};
```

## 影响范围

- 文件: `neotrix-core/src/l2_perception/nt_world/nt_world_github_absorber.rs`
- Facade: `neotrix-core/src/l2_perception/nt_world/l1_facade.rs`

## 验证

```bash
grep -rn "use crate::l1_action" neotrix-core/src/l2_perception --include="*.rs" | grep -v test | grep -v facade
```

修复后应无输出。

## 后续

- [ ] 编译验证: `cargo check -p neotrix`
- [ ] 单元测试: `cargo test -p neotrix --lib`
- [ ] 扩展 facade 覆盖其他 L2→L1 跨域引用
