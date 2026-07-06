# Makefile for NeoTrix TODO 动态同步

# 启动后台守护进程（带 tracing 日志追踪）
run:
	@echo "[MAKEFILE] 重启 NeoTrix 后台守护进程 (tracing enabled)..."
	@scripts/daemon-monitor.sh stop 2>/dev/null; sleep 1
	@RUST_LOG=info,neotrix=debug,tokio=warn cargo run --bin daemon 2>&1 &
	@sleep 3
	@pgrep -f "target/debug/daemon" | head -1 > /tmp/neotrix_daemon.pid
	@echo "✅ 守护进程已启动 (PID: $$(cat /tmp/neotrix_daemon.pid))"
	@echo "📋 实时日志追踪: tail -f /tmp/neotrix/daemon.log"
	@echo "📋 健康状态: cat /tmp/neotrix_daemon.health"
	@scripts/daemon-monitor.sh status

# 同步 TODO（单次）
sync-todo:
	@echo "[MAKEFILE] 运行 TODO 同步..."
	@python3 scripts/sync_todos.py --check-once
	@echo "[MAKEFILE] 同步完成"

# 监控模式（需要 watchdog）
watch-todo:
	@echo "[MAKEFILE] 启动文件监控模式..."
	@python3 scripts/sync_todos.py --watch

# 守护进程模式（后台运行）
daemon-todo:
	@echo "[MAKEFILE] 启动守护进程模式..."
	@python3 scripts/sync_todos.py --daemon 300 &

# 安装 Git hook
install-hook:
	@echo "[MAKEFILE] 安装 Git post-commit hook..."
	@chmod +x scripts/git-hook.sh
	@ln -sf ../../scripts/git-hook.sh .git/hooks/post-commit
	@echo "[MAKEFILE] Hook 安装完成"

# 安装 launchd 服务（macOS）
install-launchd:
	@echo "[MAKEFILE] 安装 launchd 服务..."
	@cp scripts/com.neotrix.todo-sync.plist ~/Library/LaunchAgents/
	@launchctl load ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@echo "[MAKEFILE] 服务已启动（间隔300秒）"
	@echo "[MAKEFILE] 查看日志: tail -f /tmp/neotrix-todo-sync.log"

# 卸载 launchd 服务
uninstall-launchd:
	@echo "[MAKEFILE] 卸载 launchd 服务..."
	@launchctl unload ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@rm ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@echo "[MAKEFILE] 服务已卸载"

# 检查 TODO 冲突
check-conflicts:
	@python3 scripts/sync_todos.py --check-conflicts

# 显示当前 TODO 统计
todo-stats:
	@echo "[MAKEFILE] TODO 统计:"
	@grep -c "###" TODO.md 2>/dev/null || echo "0"
	@echo " 个 TODO 项"

# ═══════════════════════════════════════════════════════════
# 山海经研究管道
# ═══════════════════════════════════════════════════════════

SHANHAI_OUT ?= /tmp/shanhai

# 一步执行: 吸收所有数据 → 导出 GeoJSON → 更新可视化
shanhai-pipeline:
	@mkdir -p $(SHANHAI_OUT)
	@echo "=== 山海经数据管道 ==="
	@echo "阶段1/4: 基础数据吸收..."
	@cargo run -p neotrix --bin neotrix-shanhai-ingest 2>/dev/null || true
	@echo "阶段2/4: 地理坐标吸收..."
	@cargo run -p neotrix --bin neotrix-shanhai-geo 2>/dev/null || true
	@echo "阶段3/4: 证据吸收..."
	@cargo run -p neotrix --bin neotrix-shanhai-evidence 2>/dev/null || true
	@echo "阶段4/5: 关系链接 (证据→山峰/学者)..."
	@cargo run -p neotrix --bin neotrix-shanhai-link 2>/dev/null || true
	@echo "阶段5/5: 导出 GeoJSON..."
	@cargo run -p neotrix --bin neotrix-shanhai-query -- export-geojson $(SHANHAI_OUT)/shanhai-mappings.geojson 2>/dev/null
	@echo "=== ✅ 管道完成 ==="
	@echo "GeoJSON: $(SHANHAI_OUT)/shanhai-mappings.geojson"
	@echo "下一步: make shanhai-visualize 或打开 shanhaijing-world.html"

# 查看 KB 山海数据统计
shanhai-stats:
	@cargo run -p neotrix --bin neotrix-shanhai-query -- stats

# 列出所有映射点
shanhai-mappings:
	@cargo run -p neotrix --bin neotrix-shanhai-query -- mappings

# 列出所有证据
shanhai-evidence:
	@cargo run -p neotrix --bin neotrix-shanhai-query -- evidence

# GeoJSON 导出
shanhai-export:
	@mkdir -p $(SHANHAI_OUT)
	@cargo run -p neotrix --bin neotrix-shanhai-query -- export-geojson $(SHANHAI_OUT)/shanhai-mappings.geojson

# 可视化: GeoJSON → 打开 HTML 地图
shanhai-visualize: shanhai-export
	@echo "GeoJSON 已导出至 $(SHANHAI_OUT)/shanhai-mappings.geojson"
	@echo "请打开 shanhaijing-world.html 查看 (或执行下面命令)"
	@echo "  open shanhaijing-world.html"
	@echo "或打开 kb-viewer HTML 查看 KB 数据点:"
	@echo "  open docs/shanhai-kb-viewer.html"

# 全链路: 清理 → 吸收 → 导出 → 可视化
shanhai-all: build-shanhai shanhai-pipeline
	@echo "=== 全链路完成 ==="
	@ls -la $(SHANHAI_OUT)/shanhai-mappings.geojson

# 编译所有 shanhai 二进制
build-shanhai:
	cargo check -p neotrix --bin neotrix-shanhai-ingest --bin neotrix-shanhai-geo --bin neotrix-shanhai-evidence --bin neotrix-shanhai-all --bin neotrix-shanhai-query

.PHONY: sync-todo watch-todo daemon-todo install-hook install-launchd uninstall-launchd check-conflicts todo-stats shanhai-pipeline shanhai-stats shanhai-mappings shanhai-evidence shanhai-export shanhai-visualize shanhai-all build-shanhai
