---
name: launchd-service
description: 安装/卸载 macOS launchd 常驻服务（todo-sync 等）
when_to_use: 需要让某个任务在 macOS 登录后常驻运行时
tools:
  - dev-tools/launchd-service/com.neotrix.todo-sync.plist
# 无 .sh 工具 —— 本技能交付的是 plist 本身，由 Makefile 的
# install-launchd / uninstall-launchd target 调用。
---

# LaunchD Service

## Purpose
macOS launchd 服务管理：TODO 同步定时任务

## Trigger Words
- launchd
- install service
- 安装服务
- todo sync service
- plist

## Workflow

### Install Service
```bash
cp scripts/com.neotrix.todo-sync.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
```

### Uninstall Service
```bash
launchctl unload ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
rm ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
```

### Service Configuration
- Interval: 300 seconds (5 minutes)
- Log: `/tmp/neotrix-todo-sync.log`
- Command: `neotrix todo sync`

## Plist Files
- `com.neotrix.todo-sync.plist` - TODO 同步服务
- `com.neotrix.daemon.plist` - 守护进程服务
- `com.neotrix.kb-crawl.plist` - KB 爬虫服务

## Makefile Integration
```makefile
install-launchd:
	@cp scripts/com.neotrix.todo-sync.plist ~/Library/LaunchAgents/
	@launchctl load ~/Library/LaunchAgents/com.neotrix.todo-sync.plist

uninstall-launchd:
	@launchctl unload ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@rm ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
```
