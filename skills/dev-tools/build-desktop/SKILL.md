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
