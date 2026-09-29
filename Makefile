# Makefile for NeoTrix TODO 动态同步

# 启动后台守护进程（带 tracing 日志追踪）
run:
	@echo "[MAKEFILE] 重启 NeoTrix 后台守护进程 (tracing enabled)..."
	@skills/dev-tools/daemon-monitor/daemon-monitor.sh stop 2>/dev/null; sleep 1
	@RUST_LOG=info,neotrix=debug,tokio=warn cargo run --bin daemon 2>&1 &
	@sleep 3
	@pgrep -f "target/debug/daemon" | head -1 > /tmp/neotrix_daemon.pid
	@echo "✅ 守护进程已启动 (PID: $$(cat /tmp/neotrix_daemon.pid))"
	@echo "📋 实时日志追踪: tail -f /tmp/neotrix/daemon.log"
	@echo "📋 健康状态: cat /tmp/neotrix_daemon.health"
	@skills/dev-tools/daemon-monitor/daemon-monitor.sh status

# 同步 TODO（单次）
sync-todo:
	@echo "[MAKEFILE] 运行 TODO 同步..."
	@neotrix todo sync
	@echo "[MAKEFILE] 同步完成"

# 监控模式（由既有 hotreload/daemon self-tick 覆盖；此目标仅提示）
watch-todo:
	@echo "[MAKEFILE] 文件监控已由 nt_io_hotreload / daemon self-tick 覆盖（--watch 已废弃）"

# 守护进程模式（后台运行，自 tick 由 neotrix daemon 提供）
daemon-todo:
	@echo "[MAKEFILE] TODO 守护已并入 neotrix daemon self-tick（sync_todos.py --daemon 废弃）"
	@neotrix daemon --evolve &

# 安装 Git hook
install-hook:
	@echo "[MAKEFILE] 安装 Git post-commit hook..."
	@chmod +x skills/dev-tools/git-hook/git-hook.sh
	@ln -sf ../../skills/dev-tools/git-hook/git-hook.sh .git/hooks/post-commit
	@echo "[MAKEFILE] Hook 安装完成"

# 安装 launchd 服务（macOS）
install-launchd:
	@echo "[MAKEFILE] 安装 launchd 服务..."
	@cp skills/dev-tools/launchd-service/com.neotrix.todo-sync.plist ~/Library/LaunchAgents/
	@launchctl load ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@echo "[MAKEFILE] 服务已启动（间隔300秒）"
	@echo "[MAKEFILE] 查看日志: tail -f /tmp/neotrix-todo-sync.log"

# 卸载 launchd 服务
uninstall-launchd:
	@echo "[MAKEFILE] 卸载 launchd 服务..."
	@launchctl unload ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@rm ~/Library/LaunchAgents/com.neotrix.todo-sync.plist
	@echo "[MAKEFILE] 服务已卸载"

# 检查 TODO 冲突（smart_analyze 冲突报告；冲突时退出码非0）
check-conflicts:
	@neotrix todo status
	@echo "[MAKEFILE] 冲突检查见上方报告（Rust 吸收 sync_todos.py）"

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

# ═══════════════════════════════════════════════════════════
# 项目结构全景视图 (Project Structure Map)
# ═══════════════════════════════════════════════════════════

# 生成项目结构映射
project-map:
	@echo "[MAKEFILE] 生成项目结构全景视图..."
	@bash skills/dev-tools/project-map/map-project.sh
	@echo "✅ 项目映射已生成: .project-map/"

# 查看项目结构
project-view:
	@echo "[MAKEFILE] 项目结构概览:"
	@cat .project-map/structure.txt 2>/dev/null | head -50 || echo "请先运行 make project-map"

# 查看模块映射
project-modules:
	@cat .project-map/modules.txt 2>/dev/null | head -100 || echo "请先运行 make project-map"

# 查看函数索引
project-functions:
	@cat .project-map/functions.txt 2>/dev/null | head -100 || echo "请先运行 make project-map"

# 搜索功能
project-search:
	@if [ -z "$(QUERY)" ]; then echo "用法: make project-search QUERY=<关键词>"; exit 1; fi
	@echo "[MAKEFILE] 搜索功能: $(QUERY)"
	@grep -i "$(QUERY)" .project-map/functions.txt 2>/dev/null || echo "未找到匹配项"

# 精细定位（索引优先，毫秒级）：make project-locate COMPONENT=Foo / FILE=bar.rs / QUERY=xxx
project-locate:
	@if [ -n "$(COMPONENT)" ]; then python3 scripts/ops/nt_locate.py --component "$(COMPONENT)" --root .; \
	elif [ -n "$(FILE)" ]; then python3 scripts/ops/nt_locate.py --source-file "$(FILE)" --root .; \
	elif [ -n "$(QUERY)" ]; then python3 scripts/ops/nt_locate.py --selector "$(QUERY)" --root .; \
	else echo "用法: make project-locate COMPONENT=Foo | FILE=bar.rs | QUERY=xxx"; exit 1; fi

# 索引完整性审计（CI 用）：missing 必须为 0
project-audit-map:
	@python3 scripts/ops/nt_locate.py --audit --root .

# 显示项目统计
project-stats:
	@echo "[MAKEFILE] 项目统计:"
	@echo "  Rust 文件: $$(find neotrix-core/src -name '*.rs' | wc -l | tr -d ' ')"
	@echo "  公开函数: $$(grep -r 'pub fn\|pub async fn' neotrix-core/src --include='*.rs' | wc -l | tr -d ' ')"
	@echo "  模块数量: $$(find neotrix-core/src -type d | wc -l | tr -d ' ')"
	@echo "  代码行数: $$(find neotrix-core/src -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"

.PHONY: find find-audit
# 按任务意图查工具（替代「读 AGENTS.md 全文再人肉匹配」）
# 用法: make find QUERY="worktree 磁盘"
find:
	@python3 scripts/ops/nt_find.py $(QUERY)
find-audit:
	@python3 scripts/ops/nt_find.py --audit

.PHONY: project-map project-view project-modules project-functions project-search project-stats

# ═══════════════════════════════════════════════════════════
# 构建版本控制 (Build Version Control)
# ═══════════════════════════════════════════════════════════

BUILD_VERSION ?= $(shell date +%Y%m%d-%H%M%S)
BUILD_DIR ?= target/builds

# 清理所有构建产物
clean:
	@echo "[MAKEFILE] 清理所有构建产物..."
	@cargo clean
	@rm -rf $(BUILD_DIR)
	@echo "✅ 清理完成"

# 清理旧构建（保留最近2个版本）
clean-old:
	@echo "[MAKEFILE] 清理旧构建版本..."
	@ls -dt $(BUILD_DIR)/*/ 2>/dev/null | tail -n +3 | xargs rm -rf 2>/dev/null || true
	@echo "✅ 旧构建已清理（保留最近2个版本）"

# 版本化构建（debug）
build-versioned:
	@echo "[MAKEFILE] 版本化构建: $(BUILD_VERSION)"
	@mkdir -p $(BUILD_DIR)/$(BUILD_VERSION)
	@cargo build 2>&1 | tee $(BUILD_VERSION).log
	@cp -r target/debug $(BUILD_VERSION)-debug
	@ln -sfn $(BUILD_VERSION) $(BUILD_DIR)/latest
	@echo "✅ 构建完成: $(BUILD_DIR)/$(BUILD_VERSION)"

# 版本化构建（release）
build-versioned-release:
	@echo "[MAKEFILE] 版本化构建 (release): $(BUILD_VERSION)"
	@mkdir -p $(BUILD_DIR)/$(BUILD_VERSION)
	@cargo build --release 2>&1 | tee $(BUILD_VERSION)-release.log
	@cp -r target/release $(BUILD_VERSION)-release
	@ln -sfn $(BUILD_VERSION) $(BUILD_DIR)/latest
	@echo "✅ 构建完成: $(BUILD_DIR)/$(BUILD_VERSION)"

# 列出所有构建版本
list-versions:
	@echo "[MAKEFILE] 可用构建版本:"
	@ls -lt $(BUILD_DIR)/ 2>/dev/null | head -10 || echo "无构建版本"

# 切换到指定版本
use-version:
	@if [ -z "$(VERSION)" ]; then echo "用法: make use-version VERSION=<版本号>"; exit 1; fi
	@ln -sfn $(VERSION) $(BUILD_DIR)/current
	@echo "✅ 当前版本: $(VERSION)"

# 显示当前版本
current-version:
	@if [ -L "$(BUILD_DIR)/current" ]; then echo "当前版本: $$(readlink $(BUILD_DIR)/current)"; else echo "未设置版本"; fi

.PHONY: clean clean-old build-versioned build-versioned-release list-versions use-version current-version

# ═══════════════════════════════════════════════════════════
# 桌面端构建阶梯 (iPolloWork 式: check → build → package:dir → package)
# ═══════════════════════════════════════════════════════════

# 最快验证门: 前端 tsc+tests + cargo check
desktop-check:
	@skills/dev-tools/build-desktop/build-desktop.sh check

# 前端构建 + cargo build 桌面二进制 (--release 传 RELEASE=1)
desktop-build:
	@skills/dev-tools/build-desktop/build-desktop.sh build $(if $(RELEASE),--release)

# 完整 tauri build --no-bundle → 未打包 .app 本地验证
desktop-package-dir:
	@skills/dev-tools/build-desktop/build-desktop.sh package:dir

# 完整 tauri build → 原生安装包 (dmg/appimage/msi) + updater 签名
desktop-package:
	@skills/dev-tools/build-desktop/build-desktop.sh package

# ═══════════════════════════════════════════════════════════
# 开发快捷命令 (Dev Commands)
# ═══════════════════════════════════════════════════════════

lint:
	@echo "Running cargo fmt check..."
	@cargo fmt -- --check
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Lint passed"

test:
	@cargo test --all

check:
	@cargo check --lib -p neotrix

build:
	@cargo build -p neotrix

# ═══════════════════════════════════════════════════════════
# 架构门禁快捷命令 (Arch Gates, 蓝图 D-05 / R-P230-R-P240)
# ═══════════════════════════════════════════════════════════

# L0-L6 单向依赖快检 (镜像 architecture_constraints.rs)
layer-deps:
	bash scripts/check-layer-deps.sh

# 模块文档漂移 (R-P232, 基线 111, 目标 0 后切 --strict 进 CI)
doc-drift:
	bash scripts/check-doc-drift.sh

doc-drift-strict:
	bash scripts/check-doc-drift.sh --strict

# 真值面门禁 (audit 2026-09-27; 零编译成本, 纯 grep/find)
# 卡三类"代码存在但工具链看不见"的漂移: 0 字节 .rs / tests 下未声明的 .rs / 入库的编译产物
# 棘轮基线 = scripts/truth-surface-baseline.txt (列表而非计数, 防止"删一加一"隐身)
truth-surface:
	bash scripts/check-truth-surface.sh

truth-surface-strict:
	bash scripts/check-truth-surface.sh --strict

truth-surface-baseline:
	bash scripts/check-truth-surface.sh --update-baseline

# 覆盖率地板 (过渡值 70 防腐化; P2 清理后提到 80 卡点)
# 需: cargo install cargo-llvm-cov
coverage-gate:
	cargo llvm-cov --lcov --output-path lcov.info --lib -p neotrix -p neotrix-types --fail-under-lines 70

# 基准基线/对比 (需: cargo install critcmp; P3 接 github-action-benchmark 烘焙门)
bench-baseline:
	cargo bench -- --save-baseline main

bench-compare:
	cargo bench -- --save-baseline pr && critcmp main pr

# 依赖树 unsafe 审计 (快门, 免编译; 需: cargo install cargo-geiger)
geiger:
	cargo geiger --forbid-only

# 未使用依赖检查 (stable 可用; udeps 需 nightly 故不用; 需: cargo install cargo-machete)
machete:
	cargo machete

# 构建面盘点 (build.rs + proc-macro, R-P248, 基线: 2 良性 build.rs + 0 自有 proc-macro)
build-surface:
	bash scripts/check-build-surface.sh

build-surface-strict:
	bash scripts/check-build-surface.sh --strict

# Fuzz 就绪探针 (R-P253 烘焙计划; nightly 仅用于 fuzz 构建, 生产保持 stable)
fuzz:
	bash scripts/check-fuzz-ready.sh

# 未入库真资产门 (R-EXIST-1, RUST-STANDARDS.md 17.8; 零编译成本, 只读)
# 卡"在磁盘上但没进 git"的知识流失: 干净检出会缺文档/测试/脚本。
# 2026-09-29 立: 一次性查出 78 份文档 + 26 个集成测试。
untracked-assets:
	bash scripts/check-untracked-assets.sh

# 生产 unwrap/expect/panic 棘轮 (2026-09-29; 零编译成本, 只读)
# AGENTS/RUST-STANDARDS 明令生产禁三者, 但实测该规则**全仓无门无基线**,
# 存量 712 站点裸奔。本门只卡新增, 不强求归零 (与 layer-deps 同构)。
unwrap:
	bash scripts/check-unwrap.sh

unwrap-strict:
	bash scripts/check-unwrap.sh --strict

unwrap-baseline:
	bash scripts/check-unwrap.sh --update-baseline

# 全门禁聚合 (advisory 全家桶, 逐个返回码保留; SIM-30)
audit-all:
	bash scripts/check-layer-deps.sh; \
	bash scripts/check-doc-drift.sh; \
	bash scripts/check-build-surface.sh; \
	bash scripts/check-fuzz-ready.sh; \
	bash scripts/check-api-surface.sh; \
	bash scripts/check-untracked-assets.sh; \
	bash scripts/check-unwrap.sh; \
	bash scripts/check-truth-surface.sh

# 架构环门 (NTS-B13; 需: cargo install cargo-modules)
arch-acyclic:
	cargo modules dependencies --lib -p neotrix --acyclic --no-fns --no-traits --no-types 2>/dev/null || echo "need: cargo install cargo-modules (NTS-B13 bake plan)"

# 供应链 IOC 巡检 (SIM-37; 零成本 tripwire, P3 前置 Socket 全套)
supply-iocs:
	bash scripts/check-supply-iocs.sh

.PHONY: sync-todo watch-todo daemon-todo install-hook install-launchd uninstall-launchd check-conflicts todo-stats shanhai-pipeline shanhai-stats shanhai-mappings shanhai-evidence shanhai-export shanhai-visualize shanhai-all build-shanhai desktop-check desktop-build desktop-package-dir desktop-package lint test check build layer-deps doc-drift doc-drift-strict coverage-gate bench-baseline bench-compare geiger machete build-surface build-surface-strict fuzz arch-acyclic audit-all supply-iocs

# 进化实验账本活性门（防「造了没人跑」）
evolution-gate:
	@bash scripts/check-evolution-ledger.sh

# 跑一次完整 A/B 进化实验（人工/CI 按需，不进默认 CI）
evolution-exp:
	@cargo run --bin nt-evolution-exp -- --help
