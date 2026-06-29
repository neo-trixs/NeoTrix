# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.18.0] - 2026-06-01

### Added

- E8 reasoning engine: 64-state deterministic state machine over 6 binary reasoning axes
- SEAL self-iteration: 26-stage pipeline for autonomous self-improvement
- HyperCube VSA: 4096-dim MAP vector-symbolic knowledge representation over 16 semantic axes
- GWT attention routing: 11 specialist modules competing for global workspace broadcast
- KnowledgeBase: SQLite-persistent knowledge layer with FTS5 search, embedding hybrid rerank, and graph traversal
- Embedding-enhanced KB search: Gemini Embedding 2 / OpenAI-compatible batch embedding API
- Core-First Reasoning engine: structured reasoning plans via GWT resonance + SiliconSelfModel + ThinkingBridge
- Knowledge distillation pipeline (DistillationStage)
- Harness adapter profiles persisted to KnowledgeBase
- HyperCube query_by_task_type filtering
- Debian packaging with openssl/ca-certificates dependencies
- Tauri desktop build configuration (macOS dmg, Linux AppImage, Windows msi)
- Configurable budget tracking with per-session/daily/monthly limits
- Secret scanner with 13 regex patterns for credential detection
- Aging diagnosis with 4-indicator monitoring
- E8 epsilon-greedy RL policy with TD-style updates

### Changed

- HyperCube query() now uses real nearest-neighbor (Euclidean distance) instead of stub
- recommend_for_e8_mode() upgraded to hybrid_rerank_search (0.3 FTS + 0.7 cosine rerank)
- prompt templates now display explicit StrategyKind and Mode description
- GWT auto-registers all 11 default specialists on startup
- All reasoning entry points (reason, reason_stream, reason_through_path, reason_complement, reflect_on_trajectory) unified under CoreReasoningPlan
- E8Policy epsilon now decays post-reasoning via core_review()
- State trajectory bounded at 200 entries
- MSRV set to Rust 1.81

### Fixed

- GWT starting with 0 specialists (auto-register defaults)
- reason_through_path using legacy reason_with_hexagram instead of CoreReasoningPlan
- SiliconSelfModel and ThinkingBridge not wired into reasoning pipeline
- E8Policy.previous_mode never set (added set_previous(), synced in plan_reasoning and all path methods)
- core_review else-if blocking independent silicon_self observation
- Prompt template showing mode_desc for Strategy field
- Embedding DB lock held across API calls (three-phase: short-lock-collect → no-lock-API → short-lock-write)
- Embedding batch API per-node inefficiency (switched to batch endpoint)
- cosine_similarity missing dimension guard
- Non-proxy-safe HTTP calls in pipeline (migrated to http_client())
- Dead code cleanup (highlight_python_line stub, WHALE_THRESHOLD_USD, suspicious_patterns)
- 3 pre-existing compilation errors (file_sync/transfer.rs unclosed delimiter, cli/jsonl_stream.rs StdoutLock Send, entry/mod.rs reason() &str/String)
- All production .unwrap() calls replaced with .expect() or proper error handling

### Security

- #![forbid(unsafe_code)] across all core crates
- overflow-checks = true in all profiles
- Secret scanner integrated into SEAL pipeline (freq=1)
- cargo-deny and cargo-audit in CI pipeline
