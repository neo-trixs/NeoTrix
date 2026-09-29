# Contributing to NeoTrix

## Development Setup

### Prerequisites

- **Rust** (edition 2021): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** 20+: For Tauri desktop frontend
- **System deps (macOS)**: Xcode Command Line Tools (`xcode-select --install`)
- **System deps (Linux)**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Build & Test

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo check --lib -p neotrix         # Fast compile check (library only)
cargo check --features full --lib    # Check with all features enabled
cargo test --lib -p neotrix          # Run library tests
cargo clippy --lib -p neotrix -- -D warnings  # Lint
cargo fmt --check                    # Format check
```

### Desktop App

桌面端已于 2026-09-28 迁到**独立仓库** `~/Downloads/Neo/neobot`（提交 `d5413335`
移除了本仓的 `apps/neobot-desktop` 与 `src-tauri`）。本仓只保留
`crates/neotrix-neobot` 作为**库** —— 其 `nt_llama` 是 CLI 与桌面端共用的
本地推理唯一实现。

```bash
# 在独立仓里构建
cd ~/Downloads/Neo/neobot
npm --prefix apps/neobot-desktop/frontend install && npm --prefix apps/neobot-desktop/frontend run build
cargo build -p neobot-desktop
```

`cd src-tauri/frontend` 那条命令自 `5c02e738`（2026-09-28 归档 src-tauri）起
就已失效 —— 留着比删掉更糟：新人照做只会得到一个看不懂的
`cd: no such file or directory`。

## Code Conventions

- **Unsafe**: `#![forbid(unsafe_code)]` — zero unsafe code in core crates
- **Warnings**: `#![deny(warnings)]`, `#![deny(dead_code)]`
- **Imports**: Group by std → external → crate, sorted alphabetically
- **Error handling**: Use `?` operator, avoid `.unwrap()` in production code
- **Naming**: Snake case for functions/variables, CamelCase for types, SCREAMING_SNAKE for constants

## Pull Request Process

1. Fork the repo and create a branch from `main`
2. Run `cargo check --lib -p neotrix && cargo test --lib -p neotrix && cargo clippy --lib -p neotrix -- -D warnings`
3. Update documentation if adding/changing features
4. Add tests for new functionality
5. Use conventional commits for your PR title

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add web search integration
fix: resolve panic on empty session list
docs: update CLI command reference
refactor: extract reasoning engine into separate module
test: add ErrorBoundary component tests
chore: update dependencies
ci: add Windows runner to CI matrix
```

## Testing Guidelines

- Unit tests inline with `#[cfg(test)] mod tests { use super::*; }`
- Integration tests in `neotrix-core/tests/`
- Frontend tests live with the frontend, in the standalone desktop repo
  (`~/Downloads/Neo/neobot/apps/neobot-desktop/tests/`) — that repo has no
  `__tests__/` dir; its suites are plain `nt_smoke_*.rs` / `nt_live_*.rs`
  integration tests plus the `registration_tests` module in `src/nt_commands.rs`
- Aim for >80% coverage on new code
- Tests must pass before merging

## Reporting Bugs

Open a [GitHub Issue](https://github.com/neotrix/neotrix/issues/new) with:
- NeoTrix version (`neotrix --version`)
- Platform (OS, architecture)
- Steps to reproduce
- Expected vs actual behavior
- Logs or error output

## Feature Requests

Open a [GitHub Issue](https://github.com/neotrix/neotrix/issues/new) with the `enhancement` label. Describe the problem you're solving and any prior art or references.

## Code of Conduct

This project adheres to the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you agree to uphold this code.
