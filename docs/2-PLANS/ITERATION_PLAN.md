# NeoTrix 差距迭代计划

> 目标: 从 v0.18.0 → v0.19.0-rc1 (最小安全发布) → v0.20.0 (生产可用)
> 基线: 3582 tests ✅, 3 compile modes clean ✅, 0 releases ⚠️

---

## 如何阅读

每条任务格式: `[区域] 动作 — 修改文件:说明`

- **Effort**: 🟢 <1h | 🟡 1-4h | 🟠 4-8h | 🔴 1-3d | ⚫ 1w+
- **Deps**: 前置任务编号
- **Risk**: 低/中/高 (编译冲突风险)

---

## 阶段 1: Quick Wins (可一次性完成)

这些任务互不依赖，可并行推进。

### S60.1 — CLI 顽疾修复

| # | 任务 | Effort | 风险 |
|---|------|--------|------|
| 1.1 | **别名冲突修复** — `neotrix-core/src/cli/commands/core_cmds.rs` | 🟢 | 低 |
| | `/stats` alias `/s` → `/st`；/search 保留 `/s` | | |
| | 修改: `core_cmds.rs:99` `vec!["/s"]` → `vec!["/st"]` | | |
| 1.2 | `/help` 英文化 + 分层帮助 — `core_cmds.rs` | 🟡 | 低 |
| | 替换 30 条中文描述为英文；新增 `/help <cmd>` 显示单命令帮助 | | |
| | 添加 `help_detail()` 方法到 CliCommand trait (可选实现) | | |
| 1.3 | `/stats` 中文→英文 — `core_cmds.rs:93-160` | 🟢 | 低 |
| | 替换所有中文 UI 字符串如 "状态"/"费用"/"进化" → "Status"/"Cost"/"Evolve" | | |
| 1.4 | `/config` 真实读写 — `core_cmds.rs`, 新建 `cli/config_manager.rs` | 🟡 | 低 |
| | 读写 `~/.neotrix/config.toml`；当前硬编码 fake 数据 | | |
| 1.5 | `/model` 真实持久化 — `model_cmds.rs` | 🟡 | 中 |
| | 读写 `NEOTRIX_MODEL` + `~/.neotrix/config.toml` 的 `[model]` section | | |

### S60.2 — 安全补漏

| # | 任务 | Effort | 风险 |
|---|------|--------|------|
| 2.1 | **Read-only sandbox 通用化** — `cli/sandbox.rs` + `commands/types.rs` | 🟡 | 低 |
| | 删除 `WRITE_COMMANDS` 硬编码列表；改为在 ApprovalEngine 层通用门控 | | |
| | 方案: `sandbox.rs` 新增 `is_write_operation(action_key)` 用 ActionType 判断 | | |
| 2.2 | **网络隔离策略** — `security/permissions.rs` + `tool_permissions.rs` | 🟠 | 中 |
| | 默认 `PolicyRule::Deny` for NetworkAccess；用户通过 profile 白名单 | | |
| | 集成 LLM API 域名自动放行 | | |
| 2.3 | **SECURITY.md PGP 修复** — `SECURITY.md` | 🟢 | 低 |
| | 替换 `0xDEADBEEF` 为真实 PGP 公钥指纹 | | |

### S60.3 — 桌面端 Quick Wins

| # | 任务 | Effort | 风险 |
|---|------|--------|------|
| 3.1 | **代码分割 + 懒加载** — `App.tsx` + 各组件 | 🟡 | 中 |
| | 用 `React.lazy(() => import('./components/X'))` + `<Suspense>` | | |
| | 拆分 28 组件为按需加载 (核心 3-4 个首屏，其余懒加载) | | |
| 3.2 | **自动更新下载 UX** — `App.tsx` + updater 集成 | 🟡 | 中 |
| | `useStore` 添加 `updateStatus` + `updateProgress` | | |
| | 检测可用更新 → 显示进度条 → 重启确认 | | |

### S60.4 — 代码清理

| # | 任务 | Effort | 风险 |
|---|------|--------|------|
| 4.1 | **修复 18 clippy 警告** — `crates/neotrix-types/` | 🟠 | 中 |
| | 逐个修复 `cargo clippy -p neotrix-types` 的输出 | | |
| 4.2 | **修复 2 个 test 编译错误** — `crypto_agent/gas.rs` + `dex.rs` | 🟡 | 低 |
| | 更新 API 调用以匹配最新接口 | | |
| 4.3 | **README 同步** — `README.md` | 🟢 | 低 |
| | 测试 1715→3582; SEAL 16→27; LOC ~94K→236K | | |
| 4.4 | **删除死代码 binary targets** — `Cargo.toml` | 🟡 | 中 |
| | 审查 19 个 binary target，移除仅内部使用的 | | |

### S60.5 — API/文档 Quick Wins

| # | 任务 | Effort | 风险 |
|---|------|--------|------|
| 5.1 | **OpenAPI 3.0 规范** — 新建 `docs/openapi.yaml` | 🟠 | 低 |
| | 基于 `server.rs` 路由，手写 OpenAPI 3.0 spec 覆盖所有 API | | |
| 5.2 | **REST API Token 认证** — `web_ui/server.rs` + `api.rs` | 🟡 | 中 |
| | 从 `~/.neotrix/config.toml` 读取 `api_token`；所有 /api/* 请求验证 | | |
| 5.3 | **文档站部署** — `.vitepress/` + CI | 🟡 | 低 |
| | 完善 VitePress 内容；CI 中构建 + 部署到 GitHub Pages | | |

---

## 阶段 2: Medium Effort (需要 1-3 天)

这些通常是新模块或重构。

### S61.1 — 插件系统增强

| # | 任务 | Effort | Deps | 风险 |
|---|------|--------|------|------|
| 6.1 | **WASM 插件加载** — `plugin/registry.rs` | 🔴 | — | 高 |
| | 用 `wasmtime` 加载 `.wasm` 文件(已有 feature gate) | | | |
| | 实现 Plugin trait 的 WASM 桥接层 | | | |
| 6.2 | **插件 CLI 命令** — 新建 `cli/commands/plugin_cmds.rs` | 🟡 | — | 低 |
| | `/plugin list | load <path> | unload <name> | info <name>` | | |
| 6.3 | **插件沙箱** — `plugin/sandbox.rs` | 🔴 | 6.1 | 高 |
| | WASM 插件运行在 sandbox 中，限制资源 (内存/文件/网络) | | | |

### S61.2 — 网络/沙箱增强

| # | 任务 | Effort | Deps | 风险 |
|---|------|--------|------|------|
| 7.1 | **OS 内核沙箱框架** — `cli/sandbox.rs` + `security/sandbox_v2/` | 🔴 | — | 高 |
| | macOS: `Seatbelt` sandbox profile 生成 + 应用 | | | |
| | Linux: `Landlock` + `seccomp-bpf` 规则 | | | |
| | 方案: sandbox 按 `--sandbox seatbelt` / `--sandbox landlock` 切换 | | | |
| 7.2 | **Docker sandbox 实现** — `cli/sandbox.rs` Docker 模式 | 🟠 | — | 中 |
| | 自动构建 Docker sandbox image；文件挂载 + 命令代理 | | | |
| 7.3 | **MCP 注册中心** — 新建 `cli/commands/mcp_cmds.rs` 增强 | 🟠 | — | 中 |
| | `/mcp discover | publish | search` — 社区分享 MCP 服务器 | | | |

### S61.3 — 测试基础设施

| # | 任务 | Effort | Deps | 风险 |
|---|------|--------|------|------|
| 8.1 | **覆盖率测量** — CI + `cargo-llvm-cov` | 🟡 | — | 低 |
| | 在 CI 中运行 `cargo llvm-cov --lcov --output-path lcov.info` | | | |
| | 生成覆盖率 badge + PR comment | | | |
| 8.2 | **前端测试接入 CI** — `.github/workflows/ci.yml` | 🟢 | — | 低 |
| | 添加 `cd src-tauri/frontend && npx vitest --run` | | | |
| 8.3 | **桌面端 E2E 测试** — `e2e/desktop-smoke.spec.ts` | 🟠 | — | 中 |
| | Playwright + Tauri driver 测试：启动→加载→会话→设置 | | | |
| 8.4 | **Benchmark CI 集成** — `.github/workflows/bench.yml` | 🟡 | — | 低 |
| | 运行 Criterion bench + 对比 baseline | | | |
| 8.5 | **Flaky 测试标记** — 添加 retry/标记 | 🟢 | — | 低 |
| | `#[serial_test]` 或 `#[cfg_attr]` 标记 flaky | | | |

### S61.4 — 生态系统基础

| # | 任务 | Effort | Deps | 风险 |
|---|------|--------|------|------|
| 9.1 | **多 Provider 支持** — 新建 `cli/provider_manager.rs` | 🔴 | — | 中 |
| | 支持 Claude/Gemini/DeepSeek/Ollama provider 配置 | | | |
| | 参考 OpenCode 的 provider 抽象 | | | |
| 9.2 | **GitHub Action** — `.github/actions/neotrix-action/` | 🟡 | — | 低 |
| | `neotrix exec` 的 Docker action 封装 | | | |
| 9.3 | **安装脚本统一** — `scripts/install.sh` | 🟡 | — | 低 |
| | 统一 curl→bash 入口 (参考 OpenCode 安装脚本) | | | |

---

## 阶段 3: High Effort (需要 1-2 周以上)

### S62.x — 高级特性

| # | 任务 | Effort | Deps | 风险 |
|---|------|--------|------|------|
| 10.1 | **并行 Agent** `--agents N` — `entry/mod.rs` + `agent/` | ⚫ | 9.1 | 高 |
| | 多并发 Worker/Explorer/Monitor | | | |
| 10.2 | **背景 Agent** `--daemon` 持久化 — `background_loop/` | ⚫ | — | 高 |
| | 类似 Claude Code `/bg`，终端关闭后 Agent 继续运行 | | | |
| 10.3 | **Cloud submit** — `cli/commands/cloud_cmds.rs` | ⚫ | 9.2 | 高 |
| | 异步任务提交 + 结果拉取 | | | |
| 10.4 | **VS Code 扩展** — `src/vscode-extension/` | ⚫ | — | 高 |
| | LSP-based 内联建议 + 对话面板 | | | |

---

## 依赖图

```
S60 Quick Wins (并行)
├── S60.1 CLI 顽疾
│   ├── 1.1 别名冲突 🟢
│   ├── 1.2 /help 英语 🟡
│   ├── 1.3 /stats 英语 🟢
│   ├── 1.4 /config 🟡
│   └── 1.5 /model 🟡
├── S60.2 安全
│   ├── 2.1 Read-only 🟡
│   ├── 2.2 网络隔离 🟠
│   └── 2.3 PGP 🟢
├── S60.3 桌面
│   ├── 3.1 代码分割 🟡
│   └── 3.2 更新 UX 🟡
├── S60.4 清理
│   ├── 4.1 clippy 🟠
│   ├── 4.2 test编译 🟡
│   ├── 4.3 README 🟢
│   └── 4.4 binary 🟡
└── S60.5 API/文档
    ├── 5.1 OpenAPI 🟠
    ├── 5.2 认证 🟡
    └── 5.3 文档站 🟡

S61 Medium (依赖部分 S60)
├── S61.1 插件
│   ├── 6.1 WASM (需要 wasmtime) 🔴
│   ├── 6.2 CLI命令 (独立) 🟡
│   └── 6.3 沙箱 (依赖 6.1) 🔴
├── S61.2 网络/沙箱
│   ├── 7.1 内核沙箱 (长任务) 🔴
│   ├── 7.2 Docker 🟠
│   └── 7.3 MCP注册 (独立) 🟠
├── S61.3 测试
│   ├── 8.1 覆盖率 🟡
│   ├── 8.2 前端CI 🟢
│   ├── 8.3 桌面E2E 🟠
│   ├── 8.4 Benchmark 🟡
│   └── 8.5 flaky 🟢
└── S61.4 生态
    ├── 9.1 多Provider (大) 🔴
    ├── 9.2 GitHub Action 🟡
    └── 9.3 安装脚本 🟡

S62 高级特性 (依赖 S61)
├── 10.1 并行Agent ⚫
├── 10.2 背景Agent ⚫
├── 10.3 Cloud submit ⚫
└── 10.4 VS Code扩展 ⚫
```

---

## 发布门控矩阵

| 里程碑 | 必须完成 | 可选跳过 | 时间估计 |
|--------|---------|---------|---------|
| **v0.19.0-rc1** | 1.1, 1.2, 1.3, 2.1, 3.1, 4.1, 4.2, 4.3, 5.3 | 1.4, 1.5, 2.2, 3.2, 5.1, 5.2 | 1-2 周 |
| **v0.19.0** | rc1 + 6.2, 8.1, 8.2, 9.3 | 2.2, 3.2, 5.1, 6.1 | rc1 + 1 周 |
| **v0.20.0** | 2.2, 7.1, 7.2, 8.3, 8.4, 9.1, 9.2 | 6.1, 6.3, 7.3 | v0.19 + 3-4 周 |
| **v0.21.0** | 6.1, 6.3, 10.1, 10.2 | 7.3, 10.3, 10.4 | v0.20 + 6-8 周 |

---

## 快速启动 (Next Actions)

立即可以从 S60.1 和 S60.4 开始任何任务，不需要等待其他任务完成。

推荐顺序:
1. 🟢 2.1 Read-only sandbox 通用化 (15 分钟)
2. 🟢 1.1 别名冲突修复 (5 分钟)
3. 🟢 1.3 /stats 英文化 (10 分钟)
4. 🟢 4.3 README 同步 (10 分钟)
5. 🟡 1.2 /help 英文化 + 分层 (1 小时)
6. 🟡 3.1 代码分割 (2 小时)
7. 🟡 1.4 /config 真实读写 (2 小时)
8. 🟡 1.5 /model 真实持久化 (1 小时)
9. 🟡 2.2 网络隔离策略 (3 小时)
10. 🟠 4.1 clippy 18 warnings (3-4 小时)
