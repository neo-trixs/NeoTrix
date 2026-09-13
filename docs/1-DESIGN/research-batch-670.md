# Research Batch 670 — Repository Analysis

**Date**: 2026-09-13
**Sources**: 5 repositories analyzed for architectural patterns and fusion opportunities

---

## 1. nicobailon/visual-explainer

**URL**: https://github.com/nicobailon/visual-explainer
**Stars**: 9.8k | **Forks**: 648 | **License**: MIT

### Summary

Agent skill that generates self-contained HTML pages for diagrams, diff reviews, plan audits, data tables, and project recaps. Replaces ASCII art with styled Mermaid diagrams, CSS Grid layouts, Chart.js dashboards, and interactive theme pickers. Multi-harness: Claude Code, Pi, MCP, Cursor, OpenCode, Codex, Antigravity CLI, Copilot. Outputs to `~/.agent/diagrams/`.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Multi-Harness Skill** | Single skill ships as plugin for 10+ agent harnesses via manifest adaptation | NT-IO multi-platform adapter pattern |
| **Quick vs Full Mode** | `--quick` produces compact JSON spec → deterministic local renderer; full mode = agent-generated HTML | GWT cost-aware routing (cheap vs expensive path) |
| **Theme Picker Runtime** | 11 palettes + font pairs swap live, re-rendering Mermaid SVGs | NT-FEEL EmotionLabel — runtime state switching |
| **MCP Server** | Local stdio MCP exposing render tools, prompt templates, read-only skill resources | NT-IO MCP gateway integration |
| **Jail-Based Output** | Render targets validated, symlink-rejected, written through temp-file rename | NT-SHIELD PathValidator pattern |
| **PPTX Export** | Best-effort static HTML→PPTX conversion (titles, bullets, code blocks) | NT-FILE-ABILITY document conversion pipeline |

### Fusion Opportunities

1. **NT-IO Visual Output Adapter**: NeoTrix CLI can route diagnostic/audit results through visual-explainer's render pipeline. ConsciousnessTree health snapshots → styled HTML dashboards via `visual_explainer_prepare` + `visual_explainer_render_html`. Aligns with GWT broadcast — visual output as attention vehicle.

2. **SEAL Pipeline Visual Reports**: Each SEAL cycle produces a visual recap (architecture diagram + diff review + plan audit) that the agent renders as an HTML page. This turns abstract evolution into inspectable artifacts — the "fruits" branch of ConsciousnessTree.

3. **Quick-Mode for Cost-Aware Routing (A1)**: visual-explainer's `--quick` JSON-spec path is a concrete implementation of Axiom A1 (Cost-Aware Routing). Adopt the pattern: simple diagnostics → deterministic renderer (cheap), complex architecture explanations → agent-generated HTML (expensive).

### Implementation Changes

- `nt_io`: Add `VisualOutputAdapter` trait bridging NeoTrix results to HTML render pipeline
- `nt_meta::quality_control`: Attach visual reports to quality gate passes/failures
- GWT salience: Visual render cost as a routing weight (quick-render = low salience, full-render = high salience)

---

## 2. bytedance/deer-flow

**URL**: https://github.com/bytedance/deer-flow
**Stars**: 82.3k | **Forks**: 11.4k | **License**: MIT

### Summary

LangGraph-based super-agent harness (v2.0, ground-up rewrite). Orchestrates sub-agents, memory, sandboxes, extensible skills, and IM channel bridges (Feishu/Slack/Telegram/Discord/DingTalk). Full-stack: FastAPI Gateway + Next.js frontend + Nginx reverse proxy. Supports sandboxed execution (Local/Docker/K8s), long-term memory, session goals, context compaction, ACP agent delegation, and scheduled tasks. Model-agnostic with ordered backend fallback (OpenAI, Anthropic, DeepSeek, vLLM, OpenRouter).

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Gateway + Embed Runtime** | Single FastAPI process embeds LangGraph-compatible agent runtime | NT-IO gateway (planned) |
| **4-Service Topology** | Nginx → Gateway → Frontend → Provisioner (optional) | NeoTrix service decomposition |
| **Per-Thread Isolation** | Each conversation thread gets isolated state, sandbox, memory | Worktree isolation + paged KV (A2) |
| **Ordered Backend Fallback** | Single model interface with ordered provider fallback (Pattern P4) | NT-ACT Ordered Backend Router |
| **Skill as Package** | Skills = SKILL.md + references/ + scripts/ + tests/, version-locked | Easel Pattern P5 → SKILL-SPEC.md |
| **Extension System** | `config.yaml plugins:` list contributes middleware, routers, lifecycle hooks | NT-ACT capability nodes (Rune Socketing) |
| **Manual Context Compaction** | User-triggered context window management | KVMem compaction for <256K (A2) |
| **Scheduled Tasks** | Background scheduler with lease-fenced run ownership | NT-ACT parallel task scheduling |
| **ACP Agent Delegation** | Delegate to external agents (Codex, Claude Code, MiniMax Code) via ACP protocol | NT-ACT subagent orchestration |
| **SSE Stream Bridge** | Multi-worker SSE delivery with `Last-Event-ID` replay + Redis option | NT-IO event streaming |

### Fusion Opportunities

1. **Gateway Pattern Adoption**: DeerFlow's Gateway+Embed runtime model (Nginx→Gateway→Frontend) is the production-grade pattern NeoTrix should adopt for its planned daemon architecture. The `Gateway` owns `/api/langgraph/*` and translates to native routes — maps to NeoTrix's CLI→subsystem routing.

2. **Skill-as-Package Contract**: DeerFlow's skill structure (SKILL.md <200 lines + references/ + scripts/ + tests/) aligns with Easel's P5 pattern already absorbed. But DeerFlow adds **skill quality review** (`skills/public/skill-reviewer/`) and **CI waivers** — adopt for NT-ACT skill nodes.

3. **Ordered Backend Router (P4)**: DeerFlow's model routing (multiple providers with ordered fallback, cost awareness, per-model pricing) is a concrete implementation of Pattern P4 + Axiom A1. Port to NT-IO's LLM provider layer.

4. **Per-Thread Isolation + Context Compaction**: Direct implementation of Axiom A2 (Context as Scarce Resource). DeerFlow uses LangGraph checkpointer with delta channel mode and optional Redis bridge. Map to NeoTrix's KVMem paged KV for >256K sessions, compaction for <256K.

5. **ACP Agent Delegation Protocol**: DeerFlow delegates to external agents (Codex, Claude Code) via ACP. This is the "subagent spawning" pattern that maps to NT-ACT's ParallelTaskManager — adopt the ACP contract for cross-agent orchestration.

6. **Extension System**: DeerFlow's `plugins:` list with lifecycle hooks (middleware, task lifecycle, observer, service, router) maps directly to NeoTrix's Rune Socketing 5-color system. Crimson=middleware, Indigo=lifecycle, Obsidian=observer, Golden=error, Alabaster=service.

### Implementation Changes

- `nt_io`: Design Gateway service (Nginx reverse proxy + embedded agent runtime) modeled on DeerFlow's 4-service topology
- `nt_act`: Adopt ACP agent delegation for subagent orchestration
- `nt_io::llm_provider`: Implement Ordered Backend Router with per-model pricing + cost-aware fallback
- `nt_core_self`: Context compaction strategy (KVMem paged KV for long sessions, compaction for short)
- `nt_act::skill_node`: Add SKILL-SPEC.md quality review + CI waiver contract from DeerFlow's skill-reviewer

---

## 3. Stirling-Tools/Stirling-PDF

**URL**: https://github.com/Stirling-Tools/Stirling-PDF
**Stars**: 91.9k | **Forks**: 8.3k | **License**: GPL-3.0

### Summary

#1 PDF application on GitHub. Self-hosted PDF editing platform with 50+ tools (edit, merge, split, sign, redact, convert, OCR, compress). Java/Spring backend + JavaScript frontend. No-code automation pipelines via UI with REST APIs for batch processing. Enterprise features: SSO, auditing, on-prem deployment. 40+ language UI.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **50+ Tool Registry** | Each PDF operation = independent tool with REST endpoint | NT-ACT capability registry |
| **No-Code Pipeline** | UI-based workflow composition for multi-step PDF operations | NT-ACT orchestration / SEAL pipeline |
| **Engine Separation** | `engine/` directory isolates PDF processing logic from web layer | Clean domain separation |
| **Task Runner** | Taskfile.yml as unified build/dev/test command runner | NT build system pattern |
| **Docker-First** | Single `docker run` deployment, K8s Helm charts | NT deployment pattern |
| **OCR Pipeline** | Multi-engine OCR (Tesseract, etc.) for scanned PDFs | NT-WORLD content extraction |
| **File Sharing** | Secure file sharing with expiration and access controls | NT-SHIELD egress + access control |

### Fusion Opportunities

1. **PDF Processing Pipeline Integration**: NeoTrix already has `nt_file_ability` with PdfIconEnhance and PdfImageExtract. Stirling-PDF's 50+ tool catalog maps to a **capability registry pattern** — each PDF operation as a UnifiedCapability node routed through CapabilityRegistry. Adopt the tool-per-endpoint pattern for NT-FILE-ABILITY.

2. **No-Code Pipeline for SEAL**: Stirling-PDF's UI-based pipeline composition (chain multiple PDF operations) is a concrete model for SEAL pipeline's multi-stage workflow. Each SEAL phase (Soil→Roots→Trunk→Branches→Fruits→Core) could be composed visually like Stirling's PDF pipeline.

3. **Engine Isolation Pattern**: Stirling's `engine/` directory cleanly separates processing logic from web layer. NeoTrix should adopt this: `nt_file_ability/engine/` for PDF/image processing, `nt_file_ability/api/` for REST/CLI interface.

4. **OCR as Content Extraction Node**: Stirling's multi-engine OCR pipeline (Tesseract + alternatives) maps to NT-WORLD's UnifiedCrawler content extraction. Add OCR as a first-class extraction strategy for scanned documents.

### Implementation Changes

- `nt_file_ability`: Expand PdfEnhanceCapability into a full tool registry (merge, split, sign, redact, convert, OCR) following Stirling's per-tool-per-endpoint pattern
- `nt_file_ability/engine/`: Create engine subdirectory isolating PDF processing from API layer
- `nt_world`: Add OCR extraction strategy (multi-engine: Tesseract, PaddleOCR) for scanned document processing
- `nt_act`: Adopt no-code pipeline composition pattern for multi-step workflows

---

## 4. unclecode/crawl4ai

**URL**: https://github.com/unclecode/crawl4ai
**Stars**: 82.9k | **Forks**: 8.6k | **License**: Apache-2.0

### Summary

LLM-friendly web crawler and scraper. Produces clean Markdown from web pages for RAG, agents, and data pipelines. Features: async browser pool, stealth mode, deep crawl (BFS/DFS/Best-First), LLM-driven extraction, structured data extraction (CSS/XPath schemas), session management, proxy support, caching, crash recovery, and Docker deployment with monitoring dashboard.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Markdown-First Output** | Clean/Fit Markdown with headings, tables, code, citations | NT-WORLD content normalization |
| **Async Browser Pool** | Managed Chromium pool with page pre-warming | NT-WORLD UnifiedCrawler browser pool |
| **Deep Crawl Strategies** | BFS/DFS/Best-First with max-depth, crash recovery, prefetch mode | NT-WORLD crawl strategy routing |
| **BM25 Content Filtering** | BM25-based noise removal for LLM-friendly output | NT-MEMORY BM25 search index |
| **LLM Extraction** | Schema-driven structured extraction via any LLM provider | NT-WORLD + NT-IO LLM extraction |
| **MemoryAdaptiveDispatcher** | Adaptive memory management for crawl scheduling | GWT attention allocation |
| **Crash Recovery** | `resume_state` + `on_state_change` callbacks for long crawls | SEAL pipeline checkpoint |
| **DomainMapper** | URL domain mapping for multi-tenant crawling | NT-WORLD domain routing |
| **3-Tier Browser Pool** | Permanent/Hot/Cold browser tiers with auto-promotion | Resource tiering (Obsidian rune) |
| **Prefetch Mode** | Skip processing, discover URLs only (5-10x faster) | Two-phase pipeline (A2 optimization) |

### Fusion Opportunities

1. **NT-WORLD Crawler Upgrade**: crawl4ai's architecture is the direct upgrade path for NT-WORLD's UnifiedCrawler. Key additions: async browser pool, stealth mode, BM25 content filtering, deep crawl strategies. Port the `AsyncWebCrawler` pattern with `BrowserConfig` + `CrawlerRunConfig` separation.

2. **BM25 Content Filter → NT-MEMORY**: crawl4ai's `BM25ContentFilter` and `PruningContentFilter` are production implementations of noise removal. Port to NT-MEMORY's search pipeline for cleaner document ingestion.

3. **Crash Recovery → SEAL Pipeline**: crawl4ai's `resume_state` + `on_state_change` pattern for long-running crawls maps directly to SEAL pipeline checkpoint recovery. Each SEAL phase writes state; crashes resume from last checkpoint.

4. **MemoryAdaptiveDispatcher → GWT**: The adaptive dispatcher that manages memory pressure and schedules crawl tasks based on resource availability is a concrete implementation of GWT attention allocation. Adopt the pattern for GWT salience-based task scheduling.

5. **Two-Phase Pipeline (Prefetch → Process)**: crawl4ai's prefetch mode (discover URLs first, process selectively) implements Axiom A2 (Context as Scarce Resource) — minimize wasted processing. Apply to NT-WORLD: first pass discovers content structure, second pass extracts relevant content.

6. **DomainMapper**: URL domain mapping for multi-tenant crawling maps to NT-WORLD's domain routing and NT-SHIELD's egress policy. Adopt DomainMapper for per-domain crawl configuration.

### Implementation Changes

- `nt_world`: Rewrite UnifiedCrawler using crawl4ai's async browser pool + stealth mode architecture
- `nt_memory`: Add BM25ContentFilter for document ingestion noise removal
- `nt_core_mind`: Add crash recovery checkpoints to SEAL pipeline (resume_state + on_state_change)
- `gwt`: Adopt MemoryAdaptiveDispatcher pattern for attention-based task scheduling
- `nt_world`: Implement two-phase pipeline (prefetch discovery → selective processing)

---

## 5. siknet/FreePEP

**URL**: https://github.com/siknet/FreePEP
**Stars**: 970 | **Forks**: 154 | **License**: MIT

### Summary

Batch downloader for PEP (People's Education Press) electronic textbooks. AES decryption engine for encrypted content, Playwright browser automation for WAF slider verification bypass, multi-threaded parallel downloading, and JPEG→PDF synthesis. Dual UI: WebUI (FastAPI) and interactive CLI. 780+ textbook catalog.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Dual UI (WebUI + CLI)** | Same backend, two interface modes | NT-IO multi-interface pattern |
| **AES Decryption Engine** | Core crypto engine for content decryption | NT-SHIELD crypto utilities |
| **WAF Bypass** | Playwright-based slider verification automation | NT-SHIELD stealth automation |
| **Multi-Threaded Download** | Configurable parallel workers with progress tracking | NT-ACT ParallelTaskManager |
| **Catalog Caching** | Local JSON cache of textbook metadata (auto-refresh) | NT-MEMORY KB cache pattern |
| **PDF Synthesis** | JPEG page collection → multi-page PDF via Pillow | NT-FILE-ABILITY document generation |
| **Breakpoint Resume** | Skip completed downloads, resume from interruption | SEAL pipeline checkpoint |
| **Layered Directory** | Organized output: `学段/年级/教材名.pdf` | KB namespace organization |

### Fusion Opportunities

1. **Download Orchestration Pattern**: FreePEP's multi-threaded download with progress tracking and breakpoint resume is a concrete implementation for NT-ACT's production pipeline. The worker-pool pattern (configurable `-w N` threads) maps to ParallelTaskManager.

2. **Content Decryption as NT-SHIELD Node**: The AES decryption engine for protected content maps to NT-SHIELD's security utilities. Package as a general-purpose decryption node for content acquisition pipelines.

3. **Catalog Cache → KB Pattern**: FreePEP's local JSON catalog cache with auto-refresh is a simple version of NT-MEMORY's KB caching. Adopt the "stale-while-revalidate" pattern for catalog/metadata caching.

4. **Dual Interface (WebUI + CLI)**: FreePEP's single-backend, dual-interface pattern (FastAPI WebUI + CLI) is the model for NT-IO's planned multi-interface architecture. One agent runtime, multiple access paths.

5. **PDF Synthesis Pipeline**: JPEG→PDF synthesis via Pillow complements NT-FILE-ABILITY's existing PDF capabilities. Add as a document generation node (reverse of PDF extraction).

### Implementation Changes

- `nt_act::parallel_task`: Adopt FreePEP's worker-pool download pattern with progress tracking and breakpoint resume
- `nt_shield`: Package AES decryption as reusable content decryption node
- `nt_memory`: Add stale-while-revalidate catalog cache pattern
- `nt_io`: Implement dual-interface pattern (one agent runtime, WebUI + CLI + API)
- `nt_file_ability`: Add JPEG→PDF synthesis pipeline (complement to existing PDF extraction)

---

## Cross-Source Synthesis

### Pattern Convergence Matrix

| Pattern | visual-explainer | deer-flow | Stirling-PDF | crawl4ai | FreePEP | NeoTrix Mapping |
|---------|-----------------|-----------|--------------|----------|---------|-----------------|
| **Multi-Harness/Interface** | ✓ (10 harnesses) | ✓ (IM bridges) | ✓ (Web/API/Docker) | ✓ (CLI/Docker/API) | ✓ (WebUI/CLI) | NT-IO multi-interface |
| **Ordered Backend Fallback** | — | ✓ (model routing) | — | ✓ (proxy fallback) | — | NT-ACT Ordered Backend Router |
| **Crash Recovery/Checkpoint** | — | ✓ (run ownership) | — | ✓ (resume_state) | ✓ (breakpoint) | SEAL pipeline checkpoint |
| **Adaptive Resource Mgmt** | ✓ (quick/full) | ✓ (context compaction) | — | ✓ (MemoryDispatcher) | ✓ (worker pool) | GWT attention allocation |
| **Skill as Package** | ✓ (SKILL.md) | ✓ (SKILL.md+tests) | — | — | — | SKILL-SPEC.md contract |
| **Tool Registry** | — | ✓ (MCP tools) | ✓ (50+ PDF tools) | ✓ (extraction strategies) | — | NT-ACT capability registry |
| **Content Filtering** | — | — | — | ✓ (BM25/Pruning) | — | NT-MEMORY BM25 ingestion |
| **Stealth/Anti-Detection** | — | — | — | ✓ (stealth mode) | ✓ (WAF bypass) | NT-SHIELD stealth automation |
| **Engine Isolation** | ✓ (render vs agent) | ✓ (harness vs app) | ✓ (engine/ dir) | ✓ (crawler vs extract) | ✓ (core vs UI) | Domain separation pattern |

### Priority Fusion Targets

| Priority | Target | Source(s) | Impact |
|----------|--------|-----------|--------|
| **P0** | NT-WORLD Crawler Rewrite | crawl4ai | Async browser pool + stealth + BM25 filtering |
| **P0** | SEAL Crash Recovery | crawl4ai + FreePEP | resume_state + on_state_change for long pipelines |
| **P1** | NT-IO Gateway Architecture | deer-flow | 4-service topology + embedded runtime |
| **P1** | Ordered Backend Router | deer-flow | Model routing with cost-aware fallback |
| **P1** | NT-FILE-ABILITY Tool Registry | Stirling-PDF | 50+ PDF tools as capability nodes |
| **P2** | NT-IO Visual Output | visual-explainer | HTML dashboard rendering for diagnostics |
| **P2** | NT-MEMORY BM25 Ingestion | crawl4ai | Content noise removal for document ingestion |
| **P2** | Parallel Task Worker Pool | FreePEP | Multi-threaded download with progress tracking |
| **P3** | Skill Quality Review | deer-flow | SKILL-SPEC.md quality gate + CI waivers |
| **P3** | OCR Extraction | Stirling-PDF | Multi-engine OCR for scanned documents |

### Axiom Alignment

| Axiom | Sources Implementing | NeoTrix Action |
|-------|---------------------|----------------|
| **A1: Cost-Aware Routing** | visual-explainer (quick/full), deer-flow (model pricing), crawl4ai (prefetch) | GWT salience + cost weight for model routing |
| **A2: Context as Scarce Resource** | deer-flow (context compaction), crawl4ai (prefetch mode), FreePEP (breakpoint) | KVMem paged KV + compaction adaptive switching |
| **A3: Skill as Production Template** | visual-explainer (SKILL.md), deer-flow (skill packages) | SKILL-SPEC.md contract for all NT-* skills |
