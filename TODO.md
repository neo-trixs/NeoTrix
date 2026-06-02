# NeoTrix — 完整后续 TODO

> 最后更新: 2026-06-02 (S59.8: Harness Evolution 论文分析 + TODO 全同步)
> 编译: `--lib` ✅ (pre-existing wasm unused var, E0592 `from_env`)
> 测试: **proxy_daemon_wrapper 4/4 passed · 意识进化 77/77 passed** (pre-existing 31 runtime flaky)
> 当前阶段: **S59.8 — 论文驱动架构决策：Agent-Evolver 分离规划**

---

## 🏗 进化迭代时间线 (Sessions 43-59)

| 批次 | 交付项 | 数量 | 状态 |
|------|--------|------|------|
| 大文件拆分 | 5 文件→42 新文件 | 42 | ✅ |
| EWC Fisher 矩阵 | `ReasoningBrain::new()` 默认初始化 | 1 | ✅ |
| Agentic Code Reasoning | SemiFormal 三态 + 6步循环 + 22 tests | 1 | ✅ |
| Real Sensors | ScreenCapture + MicCapture | 2 | ✅ |
| PilotDeck 吸收 | WorkSpace + WhiteBoxMemory + SmartRouter + AlwaysOn | 4 | ✅ |
| 对标差距 P1 | /review + /budget + session fork/export/import | 5 | ✅ |
| Desktop 增强 | CodeEditor + ContextMenu | 2 | ✅ |
| P2 批量 | MCP Server + Vim + Git worktrees + /schedule + Connectors + Tauri updater | 6 | ✅ |
| P3 六项并行 | Web UI + Plugin + Remote + Voice + Sandbox + LSP | 6 | ✅ |
| S52 发布工程化 | CI/文档/测试/搜索/编辑器/快捷键 等 12 项 | 12 | ✅ |
| S53 编译修复 | 33 编译错误修复 + Sentry + 错误上报 | 35 | ✅ |
| S54 P1 特性 | features CLI/SBOM/gitleaks/16 lib errors/6 test errors | 24 | ✅ |
| S55 全量 clean-build | 30 API 不一致 + Cargo.lock + 3 test 编译 + 2 binary | 35 | ✅ |
| S56 发布差距分析 | 对标 Codex/OpenCode/Pi/Claude/Aider，产出 RELEASE_CHECKLIST.md | 1 | ✅ |
| S56 同期实现 | `/doctor` `/model` `/notification` `/mention` + 5 语言高亮 + 编译修复 | 7 | ✅ |
| S57 h4cker+Mole+意识研究 | h4cker 460 md KB seed / SecretScanner +7 / AISecurity GWT #12 / Mole / 18论文 / Roadmap | 2 文档 | ✅ |
| S57.5 发布就绪 | 52项差距闭环：crypto_agent/clippy/sandbox/JSONL/Docker/Tauri/E2E/GHA/session/ tab/i18n/OpenAPI/theme/@mention/web/KB LRU/全零错误 3506测试 | 30+ | ✅ |
| **S58 蒸馏对话** | Session→ConversationRecord KB 桥接 + auto-save 蒸馏 | 3 | ✅ |
| **S59.0 四项同步** | CryptoAgent 真实链 RPC + E2E 0.17ms Benchmark + macOS 自动同步 + gitleaks 审计修复 | 4 | ✅ |
| **S59.5 意识进化 8/8** | EntropyMonitor(22t) + GoalRegister(11t) + LatentPredictor(16t) + DeadlockAwareRollback + CuriosityBonus + StagnationSignal + GaussianThoughtSampler + PhysicsAttention(9t) | 8 模块 | ✅ |
| **S59.6 CLS+DepthReward** | CLS_Buffer(10t) + RecursiveDepthReward + reasoning_distiller 修复 + Roadmap 更新 | 4 | ✅ |
| **S59.7 代理后端重构** | resolve_daemon_path(3-tier) + spawn_detached 修正 + proxy_cmds/cmd 去硬编码 + ProxyPanel 简化 + CSS + KB 蒸馏 | 6 | ✅ |
| **S59.8 论文驱动分析** | arXiv:2605.30621 Harness Evolution 深度分析 + Agent-Evolver 分离决策 + 论文引用入库 | 3 | ✅ |

---

## 📊 进化发现汇总 (Harness Evolution - arXiv:2605.30621)

| 发现 | 核心数据 | 对 NeoTrix 的影响 |
|------|---------|------------------|
| **Harness-Updating 与能力无关** | 最佳/最差 evolver 差距 ≤ 3.1pp; Qwen3.5-9B ≈ Opus 4.6 | SEAL evolver 可用小模型独立进程 |
| **Harness-Benefit 非单调** | 弱: +4.4pp / 中: **+19.3pp** / 强: +2.6pp | 算力集中在 E8 agent 侧 |
| **激活失败** | Qwen3-32B SLR = 25.1%（Opus 95.7%） | 需 AuditStage 追踪 SLR |
| **跟随失败** | Qwen3-32B HFR = 0.142（Opus 0.757） | 需 GWT InstructionFollowSpecialist |
| **长程衰减** | 0.52→0.13 (弱) vs 0.89→0.80 (强) | GWT broadcast 防 drift |

---

## 🔴 发布前必做 (Release-blocking)

### 外部依赖 (Blocked — 非工程能解决)
- [ ] **macOS 代码签名 + notarization** — 需要 Apple Developer Program 账号 ($99/yr)
- [ ] **Windows EV Code Signing** — 需要 EV 证书 ($300+/yr)
- [ ] **CDN hosting for auto-updater** — 需要 S3/CloudFront 或类似

### 🚨 P0 — 发布拦截器 (工程可做，2周)
- [ ] **GitHub Release v0.19.0-rc1** — 首次发布，标记 pre-release (无签名)
- [ ] **OS 内核沙箱** — macOS Seatbelt / Linux Landlock / seccomp (sandbox 模块现为空壳)
- [ ] **网络隔离策略** — 默认只允许 LLM API, 其他需要显式配置
- [ ] **Read-only sandbox 通用化** — 非硬编码 WRITE_COMMANDS 列表
- [ ] **插件系统基础框架** — plugin discovery + 简单生命周期加载
- [ ] **安装脚本统一** — `curl -fsSL https://neotrix.ai/install | bash` 入口 (参考 OpenCode)
- [ ] **Agent-Evolver 分离** — SEAL HarnessAdapt 用独立小模型进程 (S59.9) | 2d

### 🟠 P1 — 高优先级 (3-4周)
#### CLI 体验
- [ ] `/help` 全英文 + 分层帮助 (`/help <cmd>` 子命令级)
- [ ] 修复别名冲突: `/search` → `/sr` (与 `/stats` 的 `/s` 冲突)
- [ ] `/config` 真实读写 `~/.neotrix/config.toml` (现在硬编码 fake 数据)
- [ ] `/model` 真实持久化到 config.toml (现在 stub)
- [ ] 进度指示器 — 长操作 spinner + progress bar
- [ ] Shell 补全增强 — 上下文感知 (文件/命令/会话)
- [ ] 颜色方案可配置

#### 桌面端
- [ ] 代码分割 + 懒加载 — React.lazy + Suspense 拆分 28 组件
- [ ] 自动更新下载 UX — 进度条 + 重启确认对话框
- [ ] 桌面端 E2E 测试 — Playwright + Tauri driver
- [ ] Sentry/错误监控接入 (或替代品)
- [ ] 原生 OS 通知 (tauri-plugin-notification)

#### 测试/QA
- [ ] 覆盖率测量接入 (tarpaulin/llvm-cov) + CI
- [ ] 前端测试接入 CI (vitest + testing-library)
- [ ] 桌面 E2E 覆盖核心流程

#### API/集成
- [ ] OpenAPI 3.0 规范 (至少 `--serve` HTTP API)
- [ ] REST API token 认证

#### 文档
- [ ] 文档站部署 (VitePress → GitHub Pages / Cloudflare Pages)
- [ ] Rustdoc CI 构建 + 发布

#### 代码清理
- [ ] 修复 neotrix-types 18 个 clippy 警告
- [ ] 修复 2 个 test 编译错误 (crypto_agent)
- [ ] README 同步: 测试 1715→3582, SEAL 16→26-27

#### 🧠 论文驱动进化 (S59.9)
- [ ] **P0: SEAL HarnessAdaptStage → 独立小模型 evolver** — 参考论文发现① | 2d
- [ ] **P1: 新增 ActivationAuditStage** — SLR + HFR + phase adherence 追踪 | 3d
- [ ] **P1: GWT InstructionFollowSpecialist** — 实时监控 harness 跟随率 | 3d
- [ ] **P2: E8 mode 分离** — agent vs evolver 分配不同 E8 模式 | 1d
- [ ] **P2: 论文入库** — 引用加入 consciousness-evolution-roadmap.md | 0.5d

### 🟢 P2 — 锦上添花 (6-8周)
- [ ] **`neotrix cloud submit`** — 异步任务提交
- [ ] **Multi-agent `--agents N`** — 8 并发 Workers
- [ ] **MCP 注册中心** — 社区共享 MCP 服务器
- [ ] **GitHub Action** — `neotrix-action` CI 集成
- [ ] **Benchmark CI 集成** — Criterion 基准 + 回归检测
- [ ] **Fuzzing** — cargo-fuzz 对关键路径
- [ ] **SECURITY.md PGP 键更新** — 真实密钥
- [ ] **Homebrew cask (desktop)** — `brew install --cask neotrix`
- [ ] **Linux AppImage/deb** — 构建配置
- [ ] **VS Code / JetBrains 扩展** — 对标竞品
- [ ] **Computer use** — 沙箱内浏览器控制
- [ ] **ADR 文档** — 架构决策记录

---

## 对标分析 (v0.136 Codex CLI / OpenCode 165k / Pi 54k / Claude Code)

### 市场定位
NeoTrix 的核心差异化不在对标, 而在 **E8 推理引擎 + SEAL 自迭代 + VSA HyperCube + GWT 意识架构** — 这是所有 80+ 竞品中独有的。

| 维度 | NeoTrix | Codex CLI v0.136 | OpenCode | 差距等级 |
|------|---------|-----------------|----------|----------|
| GitHub Stars | ~private | 87k | 165k | 预发布 |
| License | Apache 2.0 | Apache 2.0 | MIT | ✅ |
| 发布次数 | ~0 | 805 次 | ~200+ | ⚠️ 未发布 |
| 生态系统 | 无 | 插件市场/技能/skill hub | 75+ providers | ⚠️ 需追 |
| 安装量 | 手动脚本 | Homebrew/npm/curl | npm/curl | ⚠️ 需追 |

### 功能对标

#### 已领先
| 特性 | NeoTrix | 说明 |
|------|---------|------|
| E8 状态空间推理 | ✅ 独有 | 64-态六轴推理机 |
| SEAL 自迭代 | ✅ 独有 | 28 阶段自我进化 |
| VSA HyperCube | ✅ 独有 | 4096-dim MAP 向量知识 |
| GWT 意识架构 | ✅ 独有 | 11 专家模块竞争架构 |
| KnowledgeBase SQLite | ✅ | FTS5 + BM25 + LRU cache |
| LSP 集成 | ✅ | Codex/OpenCode/Pi 均无 |
| Vim 模式 | ✅ | Codex v0.135 刚加 |
| MCP 服务器 | ✅ | 可被其他 agent 调用 |
| Voice 输入 | ✅ | Codex/OpenCode 均无 |
| 会话搜索 (Ctrl+R) | ✅ | Codex v0.134 才加 |

#### 持平
| 特性 | NeoTrix | 对标 | 说明 |
|------|---------|------|------|
| CLI 基本操作 | ✅ | ✅ Codex/OpenCode | 完整 clap CLI |
| `/help` `/doctor` | ✅ | ✅ | 诊断命令 |
| Tab 补全 | ✅ | ✅ | 上下文感知 |
| JSONL 日志 | ✅ | ✅ OpenCode | 6 事件类型 |
| Docker 部署 | ✅ | ✅ | Compose + 双服务 |
| E2E 测试 | ✅ | ✅ Codex | Playwright 冒烟 |

#### 需追赶
| 特性 | 对标 | NeoTrix | 计划 |
|------|------|---------|------|
| 插件生态系统 | Codex/OpenCode | ❌ | v0.20 |
| MCP 服务端 | Claude Code | ✅ (已有) | 已支持 |
| Web UI | OpenCode | ✅ (已有) | 已支持 |
| 多 Provider 开箱 | OpenCode 75+ | ⚠️ 基础 | v0.20 |
| 一键安装 | Codex Homebrew | ⚠️ 脚本 | v0.19.0-rc1 |
| 云端异步任务 | OpenCode cloud | ❌ | v0.21 |

---

## 当前聚焦: S59.8 → S59.9 论文驱动进化

```
S59.8 ── Harness Evolution 论文分析 ──────────────
  ├─  arXiv:2605.30621 全文分析          ✅ 3 核心发现
  ├─  Agent-Evolver 分离决策              ✅ P0 立项
  ├─  ActivationAudit 架构设计            ✅ P1 立项
  ├─  InstructionFollowSpecialist 设计     ✅ P1 立项
  ├─  KB 论文引用入库                     ✅ ConversationRecord 写入
  └─  TODO 全同步                         ✅ 本文件
        ↓
S59.9 ── 论文驱动进化实施 ──────────────
  ├─  SEAL HarnessAdapt → 独立 evolver    ⬜ P0
  ├─  ActivationAuditStage (SLR+HFR)      ⬜ P1
  ├─  GWT InstructionFollowSpecialist     ⬜ P1
  ├─  E8 mode agent/evolver 分离          ⬜ P2
  └─  Roadmap 论文引用更新                ⬜ P2
        ↓
v0.19.0-rc1 ── 最小安全发布 (待签名)
       ↓
v0.20.0    ── CI/CD 就绪 + 文档站 + 插件基础
       ↓
v0.21.0    ── 生产可用 + 多 provider + 异步任务
```
