---
name: build-desktop
description: 构建桌面端（launchd 服务 + Rust bin 打包）
when_to_use: 需要构建或更新 macOS 桌面服务产物时
tools:
  - dev-tools/build-desktop/build-desktop.sh
# ⚠️ 已知失效（2026-09-28 实测）：脚本调 `cargo build -p neotrix-tauri`，
# 该包已随 5c02e738 移出 workspace 且 src-tauri/ 已删 ⇒ **必失败**。
# 脚本自身第 20/28 行已标注此事。用前先读那两行。
---

# Desktop Build Pipeline

## Purpose
Tauri 桌面端构建全流程：check → build → package:dir → package

## Trigger Words
- build desktop
- tauri build
- 桌面端构建
- desktop package

## Workflow

### 1. Check (最快验证门)
```bash
scripts/build-desktop.sh check
```
验证前端 tsc + tests + cargo check

### 2. Build (前端构建 + cargo build)
```bash
scripts/build-desktop.sh build [--release]
```
构建前端 + 桌面二进制

### 3. Package:Dir (未打包 .app)
```bash
scripts/build-desktop.sh package:dir
```
完整 tauri build --no-bundle → 本地验证

### 4. Package (原生安装包)
```bash
scripts/build-desktop.sh package
```
完整 tauri build → dmg/appimage/msi + updater 签名

## Makefile Integration
```makefile
desktop-check:
	@scripts/build-desktop.sh check

desktop-build:
	@scripts/build-desktop.sh build $(if $(RELEASE),--release)

desktop-package-dir:
	@scripts/build-desktop.sh package:dir

desktop-package:
	@scripts/build-desktop.sh package
```

## Rust Implementation
`nt_act_dev_tools/build_desktop.rs` 提供等效功能
