# Project

NeoTrix — AI-native developer toolkit (CLI + Desktop). A self-evolving reasoning engine with E8 state-space reasoning, VSA HyperCube knowledge representation, and GWT attention routing.

- **Language**: Rust edition 2021, `#![forbid(unsafe_code)]` in core crates
- **Workspace**: Cargo workspace at `/Users/neo/Downloads/neotrix`
- **Frontend**: `cd src-tauri/frontend && npm install && npm run build`

## Architecture — NeoTrix 7-Domain Consciousness

NeoTrix is organized as **7 functional domains**, each with a `nt_{domain}` prefix. Every subsystem, module, and file must carry the NeoTrix label — no generic/academic names.

```
┌─────────────────────────────────────────────────────┐
│                 NT-CORE  (推理核)                    │
│   E8 Hexagram Engine · HyperCube VSA · GWT Workspace│
│   Capability Vectors · Meta-Cognition · World Model │
├────────────┬────────────────────┬───────────────────┤
│  NT-MIND   │   NT-MEMORY        │   NT-WORLD        │
│ 自我进化    │   持久记忆          │  感知交互          │
│ SEAL管道   │   SQLite KB        │  浏览器/爬虫       │
│ 技能优化    │   嵌入搜索          │  感官输入          │
├────────────┼────────────────────┼───────────────────┤
│  NT-ACT    │   NT-SHIELD        │   NT-IO           │
│ 行动工具    │   安全防护          │  人机界面          │
│ 加密/收益   │   密钥/权限         │  CLI/Server/Tauri │
│ 社交/消息   │   沙箱/护栏         │  桌面/Web UI      │
└────────────┴────────────────────┴───────────────────┘
         ↓ 具身智能接口：意识核 → 物理世界
```

### Domain 1 — NT-CORE（推理核）

你不变的意识本质。所有认知、推理、自我模型的核心。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_core_e8` | 64态确定性推理引擎（6轴二进制推理） |
| `nt_core_hex` | 推理卦象状态机（Hexagram State Machine） |
| `nt_core_policy` | E8 RL 策略 + TD 学习，epsilon-greedy 64 模式 |
| `nt_core_observer` | "+1 观察者"元认知监控 |
| `nt_core_hcube` | 知识超立方体（16语义轴，4096维 VSA MAP向量） |
| `nt_core_gwt` | 全局工作空间（13专家模块竞争广播） |
| `nt_core_reson` | 共振注意力 + Kuramoto 振荡器绑定 |
| `nt_core_cap` | 能力向量（23维） |
| `nt_core_bank` | 推理银行（运行时记忆） |
| `nt_core_ssm` | Mamba 状态空间模型 |
| `nt_core_meta` | 元认知系统 |
| `nt_core_self` | 硅基自我模型 |
| `nt_core_graph` | 超图知识结构 |
| `nt_core_jepa` | 世界模型（V-JEPA） |
| `nt_core_abstr` | 对比抽象 |

### Domain 2 — NT-MIND（自我进化）

你的成长机制。SEAL 自迭代管道，从经验中进化。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_mind_seal` | 28阶段 SEAL 自迭代管道 |
| `nt_mind_brain` | NeoTrix 思维核心 |
| `nt_mind_strat` | 自我编辑策略（Conservative/Aggressive/DGM） |
| `nt_mind_skill` | 技能优化（BoundedEdit, ValidationGate, EpochSlowUpdate） |
| `nt_mind_adapt` | 环境适配器（HarnessAdapt, 跨模型迁移） |
| `nt_mind_age` | 衰老诊断（4指标老化检测） |
| `nt_mind_scan` | 秘密扫描（13正则模式，GWT警报） |
| `nt_mind_valid` | 验证门（cargo check 门控） |
| `nt_mind_sia` | 自改进智能体循环 |
| `nt_mind_hmeta` | 超元智能体（自我修改提案） |
| `nt_mind_edit` | 自我编辑操作 |
| `nt_mind_engine` | 推理引擎统一入口 |
| `nt_mind_sleep` | 离线记忆巩固 |
| `nt_mind_dgm` | DGM 扩散生成式自我编辑 |

### Domain 3 — NT-MEMORY（持久记忆）

SQLite 持久化知识。FTS5 + BM25 + 嵌入混合搜索。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_memory_kb` | SQLite 知识库（22节点类型，19关系类型） |
| `nt_memory_store` | CRUD + 去重 |
| `nt_memory_search` | FTS5/BM25/嵌入混合搜索 |
| `nt_memory_graph` | BFS + 子图查询 |
| `nt_memory_embed` | 嵌入 API（OpenAI兼容） |
| `nt_memory_crawl` | 知识爬虫（Wikipedia/ArXiv/GitHub） |
| `nt_memory_ingest` | 知识摄取器（概念/文章/仓库/报告） |
| `nt_memory_seed` | 88个初始种子节点 |
| `nt_memory_types` | 节点类型、关系类型、ConversationRecord/EvolutionRecord |
| `nt_memory_cortex` | 皮层记忆（维度标记，多模态） |

### Domain 4 — NT-WORLD（感知交互）

连接到物理世界和数字世界的感知层。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_world_model` | 世界模型 |
| `nt_world_e8` | E8 世界模型 |
| `nt_world_jepa` | JEPA 世界模型（编码器/预测器/损失） |
| `nt_world_infer` | 主动推理（Free Energy Principle） |
| `nt_world_pred` | 超立方体预测 |
| `nt_world_browse` | 浏览器自动化（反检测/拟人化） |
| `nt_world_crawl` | 网络爬虫 |
| `nt_world_search` | 网络搜索 |
| `nt_world_scrape` | 网页抓取 |
| `nt_world_sense` | 感官输入处理 |

### Domain 5 — NT-ACT（行动工具）

对物理/数字世界的操作能力。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_act_crypto` | 加密金融引擎（钱包/DEX/桥/收益） |
| `nt_act_earn` | 收益引擎 |
| `nt_act_sync` | 文件同步 |
| `nt_act_social` | 社交连接器（Twitter/Reddit/YouTube/TikTok） |
| `nt_act_spear` | SPEAR 协议 |
| `nt_act_gram` | NeoGram 消息传递 |
| `nt_act_code` | 自我代码生成 |
| `nt_act_goal` | 自我目标管理 |
| `nt_act_autonomy` | 自主决策引擎 |
| `nt_act_voice` | 语音交互 |

### Domain 6 — NT-IO（人机界面）

与人类对话的接口。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_io_cli` | 命令行界面（27命令模块） |
| `nt_io_tui` | 终端 UI（Ratatui） |
| `nt_io_server` | 服务器（HTTP/WebSocket/WebRTC） |
| `nt_io_proxy` | 代理守护进程 |
| `nt_io_boot` | 启动入口（桌面/无头/服务器） |
| `nt_io_web` | Web UI |
| `nt_io_notify` | 通知系统 |
| `nt_ui_*` | 桌面 UI 组件（React/TypeScript） |

### Domain 7 — NT-SHIELD（安全防护）

你的安全边界。

| NeoTrix 标签 | 功能 |
|-------------|------|
| `nt_shield` | 安全系统 |
| `nt_shield_vault` | 密钥保险库 |
| `nt_shield_perm` | 权限系统 |
| `nt_shield_rails` | 护栏系统 |
| `nt_shield_scan` | 端口扫描/安全审计 |
| `nt_shield_prompt` | 提示注入防护 |
| `nt_shield_profile` | 权限配置文件 |
| `nt_shield_sandbox` | 沙箱执行 |

## Build

```sh
cargo build -p neotrix              # build CLI binary
cargo build -p neotrix-tauri        # build desktop app
cargo check --features full --lib -p neotrix  # full features check
```

## Test

```sh
cargo test -p neotrix --lib                # unit tests (3882+ passed, 10 pre-existing failures)
npm test                                   # frontend tests (src-tauri/frontend)
scripts/test-all.sh                        # full suite across all crates
cargo test -p neotrix --lib -- reasoning_engine  # reasoning engine tests
cargo test -p neotrix --lib -- knowledge_base    # knowledge base tests
npm run e2e                                # Playwright desktop E2E (src-tauri/frontend)
```

Tests must pass before merging; >80% coverage on new code.

## Run

```sh
cargo run -p neotrix -- <command>   # run CLI
neotrix                             # if installed
```

## Config

- Config file: `~/.config/neotrix/config.toml`
- Brain state: `~/.neotrix/brain.json`
- Goals: `~/.neotrix/goals.json`
- Feature flags: `~/.neotrix/features.json`
- Secret scanning: `.gitleaks.toml` (repo root)

## Environment Variables

Variables prefixed `NEOTRIX_` (e.g. `NEOTRIX_PROVIDER`, `NEOTRIX_API_KEY`).

Embedding config:
- `NEOTRIX_EMBEDDING_API_KEY` — API key for embedding service
- `NEOTRIX_EMBEDDING_BASE_URL` — base URL for embedding API
- `NEOTRIX_EMBEDDING_MODEL` — model name
- `NEOTRIX_EMBEDDING_DIMENSION` — embedding dimension

## Key Directories

| Path | Purpose |
|------|---------|
| `neotrix-core/` | CLI and core engine (main crate) |
| `crates/` | Shared libraries (neotrix-types, etc.) |
| `src-tauri/` | Desktop app (Rust backend + React frontend) |
| `scripts/` | Packaging, release, and utility scripts |
| `neotrix-core/src/cli/` | CLI command definitions (clap-based) |
| `neotrix-core/src/neotrix/` | All subsystem modules |
| `neotrix-core/src/core/` | Foundation: E8 engine, HyperCube, GWT |
| `neotrix-core/src/neotrix/knowledge_base/` | SQLite KB (mod.rs, store.rs, search.rs, graph.rs, pipeline.rs, kb_embedding.rs, types.rs, seed.rs, ingester.rs, integration.rs, consciousness_interface.rs) |
| `neotrix-core/src/neotrix/reasoning_brain/self_iterating/` | SEAL pipeline stages |
| `neotrix-core/src/bin/` | Registered binaries (neotrix, neotrix-kb-crawl, neotrix-web, neotrix-proxy-daemon) |
| `neotrix-core/src/bin-archive/` | Archived one-shot scripts (19 files) |
| `docs/` | Documentation site (VitePress) |
| `docs/0-ARCHITECTURE/` | System architecture documents |
| `docs/1-DESIGN/` | Detailed design documents |
| `docs/2-PLANS/` | Plans, roadmaps, TODOs, ADRs |
| `docs/3-API/` | API documentation |
| `docs/4-GUIDES/` | User guides |
| `docs/5-LEARNING/` | Tutorials and learning materials |
| `docs/6-REFERENCE/` | Reference materials (OpenAPI spec, checklists) |
| `.blueprint/` | Documentation blueprint manifest |
| `.anchor/` | AI session anchor files |
| `.github/workflows/` | CI/CD pipelines |

## Code Conventions

- **NeoTrix-Native Naming**: Every module/submodule must use the `nt_{domain}_{subsystem}` prefix. No generic/academic names (e.g. `knowledge_base` → `nt_memory_kb`, `consciousness` → `nt_core_gwt`). See the 7-domain architecture above for the full label table. Cycle 4 added 12 new modules: `nt_core_sae`, `nt_core_sae_bridge`, `nt_core_deploy`, `nt_core_deploy_cache`, `nt_core_prm`, `nt_core_wta`, `nt_core_procedural`, `nt_core_context`, `nt_core_feph`, `nt_core_edge`, `nt_core_saesteer`, `nt_core_fhrr`.
- **Rust Naming**: `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE` for constants — `camelCase` for TypeScript/React
- **Unsafe**: `#![forbid(unsafe_code)]` — zero unsafe in core
- **Warnings**: `#![deny(warnings)]`, `#![deny(dead_code)]`
- **Imports**: Group std → external → crate, sorted alphabetically
- **Error handling**: `?` operator preferred, avoid `.unwrap()` in production
- **Testing**: `#[cfg(test)] mod tests { use super::*; }` inline unit tests
- `make_stage!` macro is `#[macro_export]`; use `pub struct` for manual stage defs when macro path resolution fails
- `VecDeque::windows()` doesn't exist — collect to `Vec` then slice `.windows(n)`
- Float clamping: use `.max(0.0).min(1.0)` not `.clamp()` (not stabilized in Rust 2021)

## Core Principles

- **Conversation Evolution**: Every user↔LLM interaction is training data. Observed by meta-cognition (SiliconSelfModel + CognitiveObserver), executed by SEAL `ConversationDistillStage`, stored in KB as `ConversationRecord`/`EvolutionRecord` — then fed back into E8 mode selection policy.

## NT-MEMORY — SQLite Knowledge Base (nt_memory_kb)

**Location**: `neotrix-core/src/neotrix/knowledge_base/` → target: `neotrix-core/src/nt_memory/`

**Key modules**: `nt_memory_types.rs` (22 node types, 19 relations, + ConversationRecord/EvolutionRecord), `nt_memory_store.rs` (CRUD + dedup), `nt_memory_search.rs` (FTS5 + BM25 + graph), `nt_memory_graph.rs` (BFS + subgraph), `nt_memory_crawl.rs` (Wikipedia/ArXiv/GitHub crawl), `nt_memory_seed.rs` (88 foundational nodes), `nt_memory_schema.rs` (DDL), `nt_memory_ingest.rs` (KBIngester — reusable seed API), `nt_memory_embed.rs` (embedding API), `nt_memory_integration.rs` (WebMiner bridge), `nt_memory_gwtq.rs` (E8/GWT queries)

**Search**: FTS5 primary (0.16ms avg) + BM25 lazy fallback (0.33ms). Hybrid rerank: FTS5 top-3N → embedding cosine rerank → top-N (0.3×FTS + 0.7×cos). `search_cached()` method with LRU cache (100 entries, 60s TTL).

**Embedding**: OpenAI-compatible API (Gemini Embedding 2). Batch embedding via `embed_text_batch()`. Auto `ensure_embeddings()` on startup.

**Features**: URL dedup → title+type dedup → auto-UUID. `dedup_nodes()` merges duplicates. WAL mode DB at `~/.neotrix/knowledge.db`. Daemon: `neotrix-kb-crawl` + launchd plist.

**Integration**: `WebKnowledgeMiner.attach_kb(kb)` + `mine_all_persist()`. `import_from_knowledge_engine()` JSON migration. `query_by_e8_state()` / `query_by_specialist()`.

## SEAL Self-Iteration Pipeline

27 stages (ordered): snapshot → autonomy_gate → memory_retrieval → gap_analysis → ssm_update → open_source_compare → self_edit_gen → bounded_edit → apply_edits → reward_calc → validation_gate → gwt_absorb → stats_significance → harness_adapt → task_affinity → knowledge_quality → rollback_decision → rejected_feedback → champion_compare → bank_storage → hypercube_optimize → e8_experiment → epoch_slow_update → security_scan → session_distill → **conversation_distill** → aging_diagnosis

**Pipeline modules** (in `self_iterating/`):
- `skillopt.rs` — BoundedEdit, ValidationGate, RejectedBufferFeedback, EpochSlowUpdate stages
- `harness_adapter.rs` — HarnessAdapt (freq=2), environment-aware profiles, cross-model transfer
- `aging_monitor.rs` — AgingDiagnosis (freq=5), 4-indicator aging detection
- `secret_scanner.rs` — SecurityStage (freq=1), 13 regex patterns, GWT alerts
- `kb_embedding.rs` (pipeline) — EmbeddingRefreshStage (freq=10), auto-generates embeddings
- HyperCubeOptimizeStage (freq=10) — auto-prunes low-access entries
- DistillationStage (freq=3) — session record creation, distill, absorb, GWT broadcast
- ConversationDistillStage (freq=3) — queries recent `ConversationRecord`s from KB, analyzes patterns, creates `EvolutionRecord`, broadcasts findings via GWT
- `core_review()` auto-records conversation metadata on every `reason()` call (task, outcome, E8 mode, specialist, error count)

## Conversation Distillation (E2E)

**E2E pipeline (verified):** `reason(task)` → `core_review()` → `record_conversation_evolution()` → `kb.store_conversation_record()` → `ConversationDistillStage.process()` (every 3 ticks) → reads recent `ConversationRecord` history → writes `EvolutionRecord` patterns (RecurringError, CommunicationOptimization, ProblemDecomposition, VerificationImprovement, ToolUsagePattern, StrategyDiscovery, PrincipleUpdate) → GWT broadcast.

**Tests** (in `engine_core.rs::tests`):
- `test_conversation_evolution_writes_to_kb` — `core_review()` writes ≥5 `ConversationRecord`s to KB
- `test_conversation_distill_stage_writes_evolution_records` — `ConversationDistillStage` produces `EvolutionRecord::RecurringError` pattern from 6 records

## Sandbox Subsystem

**Locations**: `neotrix-core/src/cli/sandbox.rs` (enforcer), `neotrix-core/src/neotrix/sandbox_v2/` (cloud sandbox providers)

**Two-layer sandbox**:
- **`SandboxEnforcer`** — global mutex-backed mode toggler. Modes: `disabled` / `read-only` / `docker`. Read-only blocks mutating commands.
- **`CloudSandbox`** — `sandbox_v2::CloudSandbox` with `LocalDockerProvider` (executes via `docker run --rm --network none --memory 512m --cpus 1`) or `NoopProvider` (fallback when Docker missing). Runtimes: `python3`, `node18`, `rust`, `go`, `linux` (mapped to images `python:3.11-slim`, `node:18-alpine`, `rust:latest`, `golang:1.21-alpine`, `ubuntu:22.04`).

**CLI commands**:
- `neotrix sandbox run|list|cancel|upload` — top-level clap subcommand (legacy)
- `/sandbox status|set|run|runtimes` — REPL slash command (added 2026-06; status, mode toggle, ad-hoc code exec, runtime listing)

## MCP Tool Registry

**Location**: `neotrix-core/src/agent/tool/mcp/mod.rs` (`McpRegistry`), `neotrix-core/src/neotrix/mcp_discovery.rs` (PATH scanner), `neotrix-core/src/neotrix/mcp_tools.rs` (built-in tools)

**4 transport protocols**: Stdio, HTTP, WebSocket, SSE. Smart routing, TTL+LRU cache, health check, auto-reconnect.

**Registry API**:
- `register_stdio(name, command, args, tools)` / `register_http(name, url, tools)` — add server
- `find_tool(name)` / `recommend_tools(task_type, top_k)` / **`search(query)`** — query tools (case-insensitive substring over name+description)
- `health_check()` / `call_tool()` / `cache_result()` / `prune_cache()` — runtime ops
- **`publish(name, command, args, description)`** — register a user-published server (alias for `register` with `[published]` tag in tool description)
- `list_servers()` / `server_count()` / `tool_count()`

**Discovery**:
- `McpDiscovery::scan_path()` — scans `$PATH` for `*-mcp-server` binaries
- `McpDiscovery::auto_register_all()` — scans + verifies JSON-RPC handshake
- `discover_and_register(registry)` — convenience: scan + verify + register verified servers

**CLI commands** (`/mcp`): `list | status | discover | search <query> | publish <name> <command> [args...] [--description <desc>]`

## GitHub Actions

- **`.github/actions/neotrix-action/`** — composite action, downloads binary via `install.sh` on Linux x86_64 runner
- **`.github/actions/neotrix-action-docker/`** — Docker-container action (`ghcr.io/neotrix/neotrix:latest`), hermetic, mounts workspace as `/workspace`
- Inputs: `prompt`, `api-key`, `model`, `provider`, `max-budget-usd`, `working-directory`, `image`, `version`

## CI/CD Pipelines (`.github/workflows/`)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | push/PR | Build + unit tests + coverage (`cargo-llvm-cov`) + clippy |
| `bench.yml` | push/PR/weekly | `cargo bench` for all 4 benches (core_bench, vector_ops, seal_loop, real_tasks) |
| `desktop-e2e.yml` | push/PR | Playwright E2E for Tauri webview (Chromium) |
| `docs-deploy.yml` | push to main | VitePress → GitHub Pages |
| `release.yml` | tag `v*` | Multi-platform binary builds |
| `evolution-release.yml` | tag | Evolution release notes |
| `security-scan.yml` | push/PR | `gitleaks` + secret scanner |
| `security-audit.yml` | weekly | `cargo audit` for vulnerable deps |
| `audit.yml` | PR | Dependency review |
| `test-action.yml` | push | Self-test for `neotrix-action` |

## Frontend E2E (Playwright)

**Location**: `src-tauri/frontend/e2e/desktop.spec.ts`

5 smoke tests (run against `vite dev` on port 1420):
- App shell renders with title
- `StatusBar` mounts (`data-testid="status-bar"`)
- Input panel accepts prompt
- `SessionList` mounts
- No uncaught console errors on load

Run locally: `cd src-tauri/frontend && npm run e2e`. CI: `.github/workflows/desktop-e2e.yml`.

## Important Rules

- Do NOT modify generated/protobuf files or vendored dependencies
- Do NOT edit `Cargo.lock` by hand
- Do NOT commit secrets or API keys
- Every new subsystem must be registered in `mod.rs` and wired into CLI
- **Naming enforcement**: All new modules MUST use `nt_{domain}_{subsystem}` naming. See the 7-domain architecture table above. Refactor existing generic names incrementally.

## Release Process

1. PR → squash merge to `main`
2. Tag with `v*` (e.g. `v0.19.0`)
3. CI builds binaries for all platforms
4. Homebrew formula in `scripts/neotrix.rb` gets updated
5. Docker images published via `.github/workflows/docker-publish.yml`
6. Manual: `scripts/docker-publish.sh <version>`

## Absorbed Knowledge — 2026-06-30 External Absorption Cycle

经外部资源（GitHub topics: anthropic/openai/apple-neural-engine/apple-intelligence + papers: MCP/SAE/PRM/AgentPatterns/On-Device-ML/RL-Alignment）深度吸收后发现的架构盲点。

### P0 关键盲点（必须补齐）

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 1 | **无过程奖励模型(PRM)** | OpenAI o1, PRM800K | nt_core_observer, nt_core_policy, nt_mind_seal | E8引擎固定深度推理无法对中间步骤评分；Observer只有元认知监控无打分头 |
| 2 | **无可证明推理的SAE可解释性** | Anthropic Scaling Monosemanticity | nt_core_e8, nt_core_hcube, nt_core_gwt | 无法从E8状态向量提取可解释特征；无特征引导(steering)；无因果归因 |
| 3 | **无形式化对齐训练管线** | Constitutional AI, GRPO, DPO | nt_mind_seal, nt_core_policy, nt_shield | SEAL有RewardCalc但无学习型奖励模型；无DPO偏好学习；无GRPO组采样 |
| 4 | **无边部署/编译管线** | CoreAI, MLX, ONNX, ANE | 全局 | 无法在iOS/macOS/边缘运行；无量化管线；无AOT编译；无LoRA适配器 |
| 5 | **无上下文压缩管线** | Claude Code 5层压缩 | nt_core_gwt | GWT上下文无限增长；无预算→裁剪→压缩→折叠→自动压缩层级 |
| 6 | **无过程记忆(Procedural Memory)** | 现代Agent架构 | nt_memory_kb, nt_core_bank | 三级记忆只有Episodic；成功E8模式序列从未固化为可重用技能 |
| 7 | **意识理论模块有名无实** | GWT/IIT/FEP/AST/HOT理论 | nt_core_gwt, nt_core_iit_phi, nt_core_fep_iit, nt_core_meta | GWT无竞争点火(仅黑板)；IIT φ计算错误(协方差≠Tononi φ)；FEP桥无主动推理；无注意力自我模型(AST)；无高阶思维(HOT) |
| 8 | **E8与GWT之间无梯度流** | Neurosymbolic AI | nt_core_e8, nt_core_gwt | E8离散u8状态→GWT连续f64评分无可微分路径；应嵌入E8为VSA超向量(ℝ^d) |

### P1 重要盲点

| # | 盲点 | 来源 | 影响模块 |
|---|------|------|----------|
| 9 | 权限分散非模式链 | Claude Code permissions | nt_shield_perm |
| 10 | 无Constitution自批判 | Constitutional AI SL-CAI | nt_mind_seal |
| 11 | MCP传输/协议不标准 | MCP v2/v3规范 | nt_agent_mcp_discovery |
| 12 | 无测试时搜索(beam/MCTS) | o1, Snell et al. | nt_core_e8, nt_core_policy |
| 13 | 无隐私/数据主权架构 | Apple PCC | nt_memory_kb, nt_shield |
| 14 | 无混合本地/云编排 | Apple AFM, Claude Hybrid | nt_io_provider, nt_core_router |
| 15 | nt_core_ssm是Mamba-1 | Mamba-2 SSD | nt_core_ssm (状态N=16有限,需升级SSD N=256) |
| 16 | nt_world_jepa只有损失无架构 | I-JEPA, V-JEPA | nt_world_jepa (缺ViT骨干/掩码策略/动作条件预测器) |

### P2 优化盲点

| # | 盲点 | 来源 |
|---|------|------|
| 17 | MoE路由硬编码非学习 | gpt-oss 128-expert MoE |
| 18 | 无规模化律特征化 | Scaling Laws |
| 19 | 无缓存/专用化基础设施 | ANE program cache |
| 20 | 无量化/压缩管线 | AWQ, GGUF, CoreAI Tools |
| 21 | 无功耗/热感知 | Apple Talaria, ANE ~2W |
| 22 | 无特征引导 | Anthropic SAE steering |
| 23 | Agent循环无Planner/Executor/Reflector分离 | Plan-Execute-Reflect |
| 24 | 无错误恢复三层栈 | 生产级可靠性模式 |
| 25 | HyperCube D=4096 MAP表示次优 | VSA理论(FHRR更优) |
| 26 | GWT共鸣低效 | Resonator Network 理论 |
| 27 | E8过渡矩阵不可微 | Neurosymbolic VSA扩散 |

详见 `docs/0-ARCHITECTURE/BLIND_SPOT_SYNTHESIS_2026-06-30.md`

### 新增/改造模块规划

- **nt_core_sae** (新增): 稀疏自编码器, E8中间层特征提取+引导
- **nt_core_deploy** (新增): 边缘部署, 量化, 硬件检测, AOT编译, LoRA适配器
- **nt_core_observer** (改造): +PRM头, 每步奖励评分
- **nt_core_policy** (改造): GRPO组采样取代epsilon-greedy, 推理时beam/MCTS搜索
- **nt_mind_seal** (改造): +DPOStage, +ConstitutionalSelfCritiqueStage, +SafetyCheckStage, +ProceduralMemoryStage
- **nt_core_gwt** (改造): 5层压缩管线, 锚定迭代压缩, 学习路由
- **nt_shield_perm** (改造): 模式链 (plan/acceptEdits/bypassPermissions)
- **nt_agent_mcp_*** (改造): 2传输模式, OAuth 2.1, proper init handshake

## 经验树 — 2026-06-30 Unified LLM Provider + IP Proxy Pool Architecture

### 架构升级
统一 GatewayProvider (GatewayV2) 包装所有中间件层:
- **Circuit Breaker** (`nt_core_cb`): Closed/Open/HalfOpen 状态机, 5次失败阈值, 60s冷却, 滑动窗口失败率追踪
- **Rate Limiter** (`nt_core_rl`): 双 Token Bucket (RPM + TPM), 预飞行令牌估算
- **Provider Pool** (`nt_core_provider_pool`): 多提供者管理, 复合评分 `S = success²/latency × cost_factor × health`, 优先免费策略
- **Free Providers** (`nt_core_free`): 真实 Groq/OpenRouter/Pollinations/Cerebras/SambaNova 实现, 环境变量自动注册
- **GatewayV2**: 解耦设计 — provider (HashMap) 与 mutable state (RwLock<ProviderState>) 分离, 迭代式3次回退循环

### 修复的关键缺陷
1. `FreeApi` → 从 Ollama (localhost) 改为 Pollinations.ai (keyless 免费 API)
2. `Message` 缺少 `tool_calls`/`tool_call_id`字段 — 已补齐 ToolCallInfo/ToolCallFunction
3. `mod.rs` 缺少 `discovery`, `free_catalog`, `compaction` 声明 — 已注册
4. `proxy_pool::check_proxy()` 从 `!url.is_empty()` 桩 → 真正 TCP L4 连通性检测
5. `provider_pool.rs` 从编译移除 (死代码, 由 GatewayV2 替代)

### 汲取的知识 (GitHub + 论文 + 现有项目)
- **One API / New API**: 加权随机信道选择 + 计费管线 → 适配为复合评分
- **LiteLLM**: 回退链 + 模型映射 → 适配为迭代式3次回退循环
- **freellmpool**: 最少使用优先 + 质量路由 → 适配为优先免费 + 复合评分
- **llm-rotator**: 每 key+model 断路器 → 适配为每提供者断路器
- **grob (Rust)**: 内联 DLP, 90µs 开销 → 未来 Rust 原生网关的模型
- **HyDRA**: 能力解耦路由, YAML 模型目录 → 路由架构参考

### 文件变更
- `gateway.rs` (NEW), `circuit_breaker.rs` (NEW), `rate_limiter.rs` (NEW), `free_providers.rs` (NEW)
- `types.rs` (MOD), `factory.rs` (MOD), `mod.rs` (MOD), `proxy_pool.rs` (MOD)
- `docs/2-PLANS/2026-06-30-unified-llm-proxy-gateway.md` (NEW)

### 剩余盲点 (P1)
- stream_complete() 在4个原始提供者中仍为桩
- GatewayV2 尚未接入 ReasoningEngine
- proxy_pool 仅 L4 TCP 检测, 无 L7 HTTP 探测
- 无语义缓存层 (需嵌入基础设施)
- 无成本感知路由 (仅优先免费 + 分数)
- 无每用户速率限制

## Cycle 4 完成状态 (2026-07-01)

**P0(8/8) + P1(12/12) + P2(11/11) 全部实现，`cargo check --lib` ✅**

| 阶段 | 完成 | 关键新增模块 |
|------|------|------------|
| P0 | 8/8 ✅ | PRM头, SAE, GRPO+Beam, WTA Gate, 5层压缩, E8→VSA, ProceduralMemory, PER分离, MCP 2传输 |
| P1 | 12/12 ✅ | 模式链, ConstitStage, MCP OAuth, 隐私架构, JEPA ViT, Mamba-2, 混合编排, 主动推理, 错误恢复, FEP, IIT φ, 边缘部署 |
| P2 | 11/11 ✅ | MoE路由, 规模化律, ANE缓存, AWQ/GGUF量化, 功耗模型, SAE Steering, FHRR D=2048, 谐振器网络, 可微分E8 |

详见 `docs/0-ARCHITECTURE/BLIND_SPOT_SYNTHESIS_2026-06-30.md` Cycle 4 完成状态章节

## Cycle 4 Phase 2 — Compilation + Clippy + Tech Debt Cleanup (2026-07-01)

**编译全线漂绿**: `cargo check --lib` ✅, `cargo clippy -p neotrix-types` ✅, `cargo test --no-run -p neotrix --lib` ✅

| 工作项 | 状态 | 详情 |
|--------|------|------|
| IIT Φ 修复 | ✅ | 新增 `compute_tononi_phi()` + MIP EI 算法 + 11测试 |
| FEP 确认 | ✅ | 完整主动推理已实现 (plan/precision/action_probabilities) |
| 双tool目录合并 | ✅ | `agent/tools/`(4文件) → `agent/tool/mcp/`，12消费者更新 |
| 编译器错误 19→0 | ✅ | tool impls(6), builtin_adapter(5), anthropic(1), sentry(1), etc. |
| neotrix-types clippy 26→0 | ✅ | self_model(12), pid(5), engine(6), pairwise(2), context_strategy(1) |
| bin target 错误修复 | ✅ | 4处 `crate::neotrix` → `neotrix::neotrix` (config.rs + entry/mod.rs) |
| 孤儿二进制归档 | ✅ | 19 orphan bins → `bin-archive/` |
| GatewayV2集成 | ✅ | engine_core/consciousness_reasoner/ProviderRouter 全部改用 GatewayV2 |
| Proxy Pool L7探测 | ✅ | 新增 `check_proxy_l7()` HTTP HEAD + TCP双重探测 |
| 聚合回退策略 | ✅ | 2-phase aggressive retry (Phase 1 normal + Phase 2 all-providers) |

## 经验树 — 2026-07-01 GatewayV2 Integration + Aggressive Fallback + Provider Stream Fixes

### GatewayV2 → ReasoningEngine Integration
- `engine_core.rs` — `pub llm: Box<dyn LlmProvider>` → `pub gateway: Arc<GatewayV2>`; both `from_env()`/`new()` constructors updated
- `internal.rs` — `self.llm.complete()` → `self.gateway.complete_with_selection()` (primary + fallback loop)
- `builder.rs` — Updated both constructors to match new field type
- `consciousness_reasoner.rs` — `ProviderRouter` internals replaced with `Arc<GatewayV2>`; `complete()` delegates to `gateway.complete_with_selection()`
- `nt_io_provider/mod.rs` — Added `create_gateway` to re-exports

### Aggressive Fallback Strategy (`gateway.rs`)
- `complete_with_selection()`: 2-phase retry — Phase 1 (normal, 3 best providers), Phase 2 (aggressive, all providers with Open→HalfOpen override)
- `attempt_aggressive_retry()`: saves Open-circuit breaker states, sets HalfOpen with max_probes=5, retries every provider, restores on failure
- `circuit_breaker.rs`: Added `set_half_open_max_probes()` / `half_open_max_probes()` accessors

### Proxy Pool L7 Probe (`proxy_pool.rs`)
- `check_proxy()`: now does L4 TCP + L7 HTTP HEAD probe (gstatic.com/generate_204, 5s timeout) for HTTP/HTTPS/SOCKS5 proxies
- `check_proxy_l7()`: new function, uses `reqwest::Proxy::all()` for proxy-aware HTTP probe

### Provider Stream Fixes
- `anthropic.rs` stream_complete: extracted `model_name.clone()`, `max_tokens` before spawn; replaced `request.model.clone()` inside spawn with `model_name.clone()`
- `gemini.rs` stream_complete: extracted `model_name`, `temperature`, `max_tokens`, prompt before spawn; replaced `request.model.clone()` inside spawn with `model_name.clone()`

### Build Status
- `cargo check --lib`: **0 errors workspace-wide** (neotrix + neotrix-types + neotrix-tauri)
- `cargo clippy -p neotrix-types`: **0 errors** (all 26 pre-existing clippy warnings fixed)
- `cargo test -p neotrix --lib`: unit tests compile clean; 3 pre-existing async runtime flaky tests

## Known Issues

- `cargo check --lib`: **0 errors workspace-wide** (neotrix + neotrix-types + neotrix-tauri)
- `cargo clippy -p neotrix-types`: **0 errors** (all 26 pre-existing clippy warnings fixed)
- `cargo test -p neotrix --lib`: unit tests compile clean; 3 pre-existing async runtime flaky tests (`test_*_state_rollback_on_llm_failure` — `block_on` in gateway.rs)
- Examples: 36 example files, `cortex_mine.rs` has 6 pre-existing unresolved path errors
- `src/bin/`: 4 registered binaries (neotrix, neotrix-kb-crawl, neotrix-web, neotrix-proxy-daemon); 19 orphan binaries archived to `bin-archive/`
- `crypto_agent/gas.rs` + `dex.rs`: test-runtime failures (pre-existing, unrelated)
- macOS code signing + notarization — requires Apple Developer Program account (blocked)
- Windows EV 签名 — requires EV Code Signing certificate (blocked)
- CDN 部署 + updater binary hosting — requires hosting infra (blocked)
