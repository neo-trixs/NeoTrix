# Daemon Monitor

## Purpose
守护进程状态监控与管理

## Trigger Words
- daemon status
- daemon monitor
- 守护进程状态
- 守护进程监控

## Workflow

### Status Check
```bash
scripts/daemon-monitor.sh status
```
检查守护进程运行状态

### Stop Daemon
```bash
scripts/daemon-monitor.sh stop
```
停止守护进程

### Start Daemon
```bash
cargo run --bin daemon
```
启动守护进程（带 tracing）

## Health File
- PID: `/tmp/neotrix_daemon.pid`
- Health: `/tmp/neotrix_daemon.health`
- Log: `/tmp/neotrix/daemon.log`

## Makefile Integration
```makefile
run:
	@scripts/daemon-monitor.sh stop 2>/dev/null; sleep 1
	@RUST_LOG=info,neotrix=debug,tokio=warn cargo run --bin daemon 2>&1 &
	@scripts/daemon-monitor.sh status
```

## Rust Implementation
`nt_act_dev_tools/daemon_monitor.rs` 提供等效功能
