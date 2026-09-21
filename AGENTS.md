# NeoTrix — Agent Guide (slim)

> 全量 codebase 索引已归档：`docs/architecture/CODEBASE-WIKI-2026-09-21.md`
> （2426 `.rs` / 798707 行快照，含文件树 / Domain 表 / Key Types / CLI 表）。
> 查文件位置用 `glob`/`grep` 现查，不要把全量表塞回本文件。本文件保持 < 100 行。

## Build & Test

```bash
cargo check --all-targets -p neotrix    # 快速检查
cargo test -p neotrix --lib             # 单元测试
cargo build -p neotrix                  # 完整构建
```

结构性改动后：`cargo clean && cargo build` 跑两遍，以拿到真实错误数。

## 模块前缀规范

- 所有模块名用 `nt_` 前缀（如 `nt_core_cache`、`nt_mind`、`nt_shield`）。
- 分层：`l0_substrate` / `l1_action` / `l2_perception` / `l3_embodiment` / `l4_emotion` / `l5_cognition` / `l6_meta`，详见 `docs/architecture/ARCHITECTURE.md`。
- 编码标准见 `RUST-STANDARDS.md`（生产代码禁 `unwrap`/`expect`/`panic!`，错误用 `?` 传播）。

## 硬规则

- `#![forbid(unsafe_code)]` —— 永不加 `unsafe`。
- 编辑后必须重读文件验证落盘（R-P16）。
- 外部技术必须同会话接到生产可用（R-P79）。

## 正典索引

- 架构：`docs/architecture/ARCHITECTURE.md`、`FUSION-ARCHITECTURE.md`
- 文档规范：`DOCUMENTATION-MAP.md`
- 路线图：`ARCHITECTURE-MAP-ROADMAP-V2.md`
