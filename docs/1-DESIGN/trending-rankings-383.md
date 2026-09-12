# Trending Rankings — Cycle 383 (2026-09-12)

## 10 New Projects (Not in Cycles 318-382)

| # | Project | Stars | Category | NeoTrix Domain | Key Pattern |
|---|---------|-------|----------|----------------|-------------|
| 1 | **Semantica** | 11.9K | Graph-native context/decision intelligence | NT-MEMORY / NT-CORE | Context Graphs as deterministic infrastructure — no LLM for graph construction, reasoning, or provenance. Decision Intelligence with W3C PROV-O audit trail |
| 2 | **NVIDIA Switchyard** | 2.8K | LLM traffic routing proxy (Rust) | NT-IO / NT-CORE | Protocol translation (OpenAI↔Anthropic), multi-backend routing (classifier/stage/escalation), operational metrics. 74% cost reduction via escalation routing |
| 3 | **OpenViking** | 35K+ | Self-evolving context database | NT-MEMORY / NT-MIND | Filesystem paradigm for agent context (viking:// URI), L0/L1/L2 tiered loading, automatic session→memory extraction, directory recursive retrieval |
| 4 | **Claw Compactor** | — | LLM token compression engine | NT-IO / NT-MEMORY | 14-stage Fusion Pipeline: AST-aware code analysis, semantic dedup, format optimization. 15–82% compression, zero LLM inference cost, reversible |
| 5 | **FreeLLM** | 23.5K | Free LLM API aggregator | NT-IO | 34 providers, 635 endpoints behind one /v1 endpoint, smart routing, auto failover, encrypted keys |
| 6 | **omlx** | 20K | LLM inference server (Apple Silicon) | NT-IO / NT-PHYSICAL | Continuous batching + SSD caching, macOS menu bar management, local-first inference |
| 7 | **ai-memory** | 3.4K | Long-term memory for agent CLIs | NT-MEMORY / NT-NEXUS | Cross-vendor handoff, Rust implementation, session memory persistence for Claude Code/Codex |
| 8 | **Book-to-Skill** | 15.9K | PDF→Claude Code skill converter | NT-MIND / NT-ACT | Transforms technical book PDFs into agent skills — study, reference, and use while working |
| 9 | **ToolRank** | — | AI agent tool optimization platform | NT-ACT / NT-CORE | Tool definition scoring (findability/clarity/precision/efficiency), LLM selection tournaments, runtime reliability testing |
| 10 | **Needle** | 8K | 14MB foundation model for tiny devices | NT-PHYSICAL / NT-IO | Ultra-compact model for phones, wearables, smart home, robots. Edge AI inference |

## Deep Dives

### 1. Semantica — Graph-Native Context & Decision Intelligence
- **Source**: semantica-agi/semantica (GitHub, 11.9K stars)
- **Core Innovation**: Deterministic infrastructure layer (no LLM required) that turns enterprise data into structured Context Graphs and knowledge graphs. Decision Intelligence makes every AI choice a first-class, auditable, queryable graph node
- **Key Features**: Polyglot graph storage (RDF + LPG + vector), bi-temporal facts, SHACL/OWL governance, Rete/Datalog/SPARQL reasoning, W3C PROV-O provenance, multi-agent context sharing
- **NeoTrix Mapping**: Maps to NT-MEMORY (graph-native knowledge representation) + NT-CORE (deterministic reasoning engine). Semantica's "no LLM for graph construction" philosophy aligns with NeoTrix's VSA HyperCube — deterministic symbolic infrastructure, not black-box embeddings

### 2. NVIDIA Switchyard — LLM Traffic Routing Proxy
- **Source**: NVIDIA-NeMo/Switchyard (GitHub, 2.8K stars)
- **Core Innovation**: Rust proxy/library that routes LLM requests across providers with protocol translation (OpenAI↔Anthropic↔Responses API). Composable routing algorithms: LLM classifier, stage router, escalation router
- **Key Features**: Signal-driven routing (tool results/errors select target), 74% cost reduction via escalation routing, Prometheus metrics, drops into existing gateways (NeMo Relay, LiteLLM)
- **NeoTrix Mapping**: Maps to NT-IO (multi-provider routing) + NT-CORE (routing intelligence). Switchyard's "stage router" pattern — where conversation signals (not just content) drive routing — is a direct evolution of NeoTrix's Axiom A1 (Cost-Aware Routing). The escalation router (weak→strong on demand) validates A1 at infrastructure level

### 3. OpenViking — Self-Evolving Context Database
- **Source**: volcengine/OpenViking (GitHub, 35K+ stars)
- **Core Innovation**: Filesystem paradigm for agent context — memories, resources, skills stored as viking:// URIs. L0/L1/L2 tiered loading on demand. Automatic session→memory extraction with trajectory observability
- **Key Features**: Directory recursive retrieval (vector search → drill down), session-to-memory self-evolution, Claude Code/Codex/Trae integration, multi-user Peer Mode, benchmark-proven (LoCoMo 80–83% accuracy, 34–91% token reduction)
- **NeoTrix Mapping**: Maps to NT-MEMORY (context database) + NT-MIND (self-evolution via session extraction). OpenViking's tiered loading (L0→L1→L2) is isomorphic to KVMem's hot/cold tiering. The filesystem-as-interface pattern validates NeoTrix's module-as-directory architecture

### 4. Claw Compactor — 14-Stage Token Compression
- **Source**: open-compress/claw-compactor (GitHub)
- **Core Innovation**: 14-stage Fusion Pipeline where each stage is a specialized compressor (AST-aware code analysis, JSON statistical sampling, simhash dedup). Content-type-aware, not perplexity-based like LLMLingua-2
- **Key Features**: 15–82% compression, zero LLM inference cost (<50ms latency), reversible compression with Rewind markers, 1600+ tests, ROUGE-L @0.5 = 0.723 (26.8% better than LLMLingua-2)
- **NeoTrix Mapping**: Maps to NT-IO (token compression for inference) + NT-MEMORY (context compaction). Claw Compactor's content-type-aware stages solve the problem that perplexity-based methods destroy code identifiers and JSON keys — critical for NeoTrix's developer-focused use cases

### 5. FreeLLM — Free LLM API Aggregator
- **Source**: tashfeenahmed/freellmapi (GitHub, 23.5K stars)
- **Core Innovation**: 34 free LLM providers, 635 model endpoints unified behind a single OpenAI-compatible /v1 endpoint. Smart routing + automatic failover
- **Key Features**: Zero-cost API access for experimentation, encrypted key management, provider health monitoring
- **NeoTrix Mapping**: Maps to NT-IO (multi-provider routing). FreeLLM validates Axiom A1 (Cost-Aware Routing) — the zero-cost tier is the ultimate "cheap model for simple tasks." Combined with Switchyard's escalation routing, this creates a complete cost-tiered architecture

### 6. omlx — LLM Inference Server for Apple Silicon
- **Source**: jundot/omlx (GitHub, 20K stars)
- **Core Innovation**: LLM inference server with continuous batching and SSD caching, managed from macOS menu bar. Local-first, no cloud dependency
- **Key Features**: SSD-backed KV cache for models that don't fit in RAM, continuous batching for throughput, native macOS integration
- **NeoTrix Mapping**: Maps to NT-IO (local inference) + NT-PHYSICAL (edge deployment). omlx's SSD caching for KV overflow directly relates to KVMem's paged KV virtualization — both solve the "model larger than GPU memory" problem via tiered storage

### 7. ai-memory — Long-Term Memory for Agent CLIs
- **Source**: akitaonrails/ai-memory (GitHub, 3.4K stars)
- **Core Innovation**: Persistent long-term memory across agent coding sessions, enabling handoff between different agent vendors (Claude Code → Codex → others)
- **Key Features**: Rust implementation, cross-vendor memory portability, session persistence
- **NeoTrix Mapping**: Maps to NT-MEMORY (cross-session persistence) + NT-NEXUS (cross-agent weaving). ai-memory's vendor-agnostic memory portability addresses the same problem as NT-NEXUS — knowledge that persists across session boundaries and agent instances

### 8. Book-to-Skill — PDF→Claude Code Skill Converter
- **Source**: virgiliojr94/book-to-skill (GitHub, 15.9K stars)
- **Core Innovation**: Transforms any technical book PDF into a Claude Code skill — structured for study, reference, and active use during coding
- **Key Features**: PDF parsing → skill extraction → agent integration, supports reference and study modes
- **NeoTrix Mapping**: Maps to NT-MIND (skill crystallization) + NT-WORLD (document parsing). Book-to-Skill is the inverse of SEAL's distillation: instead of distilling agent experience into skills, it distills external knowledge into skills. This is R-P79 (external technology absorption) in production form

### 9. ToolRank — AI Agent Tool Optimization
- **Source**: ToolRank (toolrank.dev, 2026)
- **Core Innovation**: Scores tool definitions across 4 dimensions (findability, clarity, precision, efficiency) and provides specific fixes. Uses LLM selection tournaments + runtime reliability testing
- **Key Features**: Rule-based scoring, rewrite proposals, category rankings, agent framework SDK, continuous ecosystem scanning
- **NeoTrix Mapping**: Maps to NT-ACT (tool quality) + NT-CORE (tool selection intelligence). ToolRank's 4-dimension scoring could enhance NT-ACT's MCP tool quality — ensuring tools are not just available but optimized for agent consumption

### 10. Needle — 14MB Foundation Model
- **Source**: cactus-compute/needle (GitHub, 8K stars)
- **Core Innovation**: 14MB foundation model for tiny devices — phones, wearables, smart home, robots. Edge AI without cloud dependency
- **Key Features**: Ultra-compact model, runs on-device, no cloud required
- **NeoTrix Mapping**: Maps to NT-PHYSICAL (edge inference) + NT-IO (on-device model). Needle is the physical embodiment of Axiom A1 — the cheapest model for the simplest tasks, running where no cloud is available. Directly relevant to NT-PHYSICAL's sensor/actuator loops

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|---------------------|
| **Graph-Native Context** | Semantica, OpenViking | NT-MEMORY evolution: from flat KB to graph-native context with provenance |
| **Cost-Aware Routing** | Switchyard, FreeLLM, Needle | Axiom A1 validation: tiered routing from zero-cost → escalation → frontier |
| **Filesystem-as-Interface** | OpenViking, Book-to-Skill | Module-as-directory architecture validation (EVE prev, now OpenViking at scale) |
| **Token Compression** | Claw Compactor, OpenViking tiered loading | Context as Scarce Resource (A2): deterministic compression without LLM inference |
| **Session→Memory Self-Evolution** | OpenViking, ai-memory | NT-NEXUS cross-session weaving + experience-tree absorption |
| **Tool Quality Scoring** | ToolRank | NT-ACT MCP tool optimization, A2 implications (better tools = less context waste) |
| **Edge AI** | Needle, omlx | NT-PHYSICAL inference at the edge, no cloud dependency |
| **Reversible Compression** | Claw Compactor | KVMem-compatible: compressed KV can be reconstructed on demand |
| **Cross-Vendor Portability** | ai-memory, FreeLLM | Provider-agnostic memory and routing — vendor independence as architectural principle |
| **Deterministic Infrastructure** | Semantica, Claw Compactor | "No LLM in the loop" for infrastructure — deterministic where possible, LLM where necessary |
