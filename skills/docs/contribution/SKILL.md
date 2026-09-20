# Contribution Guidelines

## Purpose
贡献指南和开发流程

## Trigger Words
- contributing
- contribution
- how to contribute
- 贡献指南
- 开发流程

## Content
- 开发环境设置
- 构建和测试命令
- 代码风格
- PR 流程
- Issue 模板

## File
`CONTRIBUTING.md`

## Development Setup
- Rust (edition 2021)
- Node.js 20+
- macOS: Xcode Command Line Tools
- Linux: libwebkit2gtk-4.1-dev etc.

## Build Commands
```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo check --lib -p neotrix         # Fast compile check
cargo test --lib -p neotrix          # Run library tests
cargo clippy --lib -p neotrix -- -D warnings  # Lint
```
