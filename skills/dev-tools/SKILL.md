# Dev Tools Skills

## Purpose
开发工具技能集合：构建、监控、钩子、服务管理

## Available Skills

| Skill | Purpose | Script |
|-------|---------|--------|
| [build-desktop](build-desktop/) | Tauri 桌面端构建 | `build-desktop.sh` |
| [daemon-monitor](daemon-monitor/) | 守护进程监控 | `daemon-monitor.sh` |
| [git-hook](git-hook/) | Git 钩子管理 | `git-hook.sh` |
| [launchd-service](launchd-service/) | macOS 服务管理 | `com.neotrix.todo-sync.plist` |
| [pre-build-check](pre-build-check/) | 构建前检查 | `pre-build-check.sh` |

## Quick Reference

### Build Desktop
```bash
make desktop-check      # 验证
make desktop-build      # 构建
make desktop-package    # 打包
```

### Daemon Management
```bash
make run                # 启动守护进程
make daemon-status      # 检查状态
```

### Git Hooks
```bash
make install-hook       # 安装钩子
```

### Services
```bash
make install-launchd    # 安装服务
make uninstall-launchd  # 卸载服务
```

## Rust Implementations
以下功能已在 `nt_act_dev_tools/` 中有 Rust 实现：
- `build_desktop.rs` ← `build-desktop.sh`
- `daemon_monitor.rs` ← `daemon-monitor.sh`
- `git_hook.rs` ← `git-hook.sh`

## Migration Path
1. Shell 脚本保持可用（当前）
2. Rust 实现通过 CLI 暴露（中期）
3. Shell 脚本归档（远期）
