# Project

NeoTrix — AI-native developer toolkit (CLI + Desktop). A self-evolving reasoning engine with E8 state-space reasoning, VSA HyperCube knowledge representation, and GWT attention routing.

- **Language**: Rust edition 2021, `#![forbid(unsafe_code)]` in core crates
- **Workspace**: Cargo workspace at `/Volumes/neotrix/neotrix`
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
| `.github/workflows/` | CI/CD pipelines |

## Code Conventions

- **NeoTrix-Native Naming**: Every module/submodule must use the `nt_{domain}_{subsystem}` prefix. No generic/academic names (e.g. `knowledge_base` → `nt_memory_kb`, `consciousness` → `nt_core_gwt`). See the 7-domain architecture above for the full label table.
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

**Location**: `neotrix-core/src/agent/tools/mod.rs` (`McpRegistry`), `neotrix-core/src/neotrix/mcp_discovery.rs` (PATH scanner), `neotrix-core/src/neotrix/mcp_tools.rs` (built-in tools)

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

## Known Issues

- neotrix-types clippy: 18 remaining (complex type refactors)
- Examples: 4 warnings (unused variables/fields)
- 10 lib test runtime failures (pre-existing: mcp_discovery `TempDir.join()` API, project_manager git setup, parallel executor shell, etc.)
- `file_sync/` module: `static_server` uses `filetime` crate, unused `modified` variable in `transfer.rs` (pre-existing)
- `cli/jsonl_stream.rs`: `StdoutLock` Send issue fixed (`.lock()` → `io::stdout()`)
- `crypto_agent/gas.rs` + `dex.rs`: test-compile errors resolved; now test-runtime failures (unrelated)
- `daemon` binary: 2 unused variable warnings
- macOS code signing + notarization — requires Apple Developer Program account (blocked)
- Windows EV 签名 — requires EV Code Signing certificate (blocked)
- CDN 部署 + updater binary hosting — requires hosting infra (blocked)
