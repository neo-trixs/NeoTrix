# Crate Split Design — monolith → 5 crates

> R4-01 | 2026-05-24 | Status: Design Phase

## 1. Current Dependency Graph

Current `neotrix-core` module structure and inter-dependency:

```
lib.rs (re-exports)
├── core/        (zero external deps — data models, traits, enums)
│   ├── capability.rs    ← knowledge.rs, edit.rs, signal/
│   ├── knowledge.rs     ← traits.rs (KnowledgeProvider)
│   ├── edit.rs          ← standalone
│   ├── memory/           ← standalone (ReasoningBank)
│   ├── event.rs         ← standalone
│   ├── signal/          ← standalone (SelectiveState, SelectableOperator)
│   ├── traits.rs        ← standalone (MemoryProvider, ToolProvider etc.)
│   ├── hypercube/       ← standalone (KnowledgeHyperCube)
│   ├── consciousness/   ← standalone (GlobalWorkspace)
│   ├── metacognition/   ← standalone
│   ├── rkyv_store.rs   ← standalone
│   └── iteration*.rs    ← standalone
│
├── agent/       (agent runtime)
│   ├── provider/   → outside (reqwest)
│   ├── team.rs     → reasoning_brain, orchestrator
│   ├── tools/      → reasoning_brain, mcp_tools
│   └── executor/   → standalone
│
├── neotrix/     (everything else)
│   ├── provider/*      → core (CapabilityVector), reqwest
│   ├── reasoning_brain/*  → core/*, agent/*, provider/*
│   ├── stealth_net/*   → reqwest, tokio, chromiumoxide
│   ├── orchestrator/   → reasoning_brain, world_model, provider
│   ├── agent_protocol/ → tokio
│   ├── crawler/        → reqwest, reasoning_brain
│   ├── web_scraper/    → reqwest (external)
│   ├── ..., search, mcp_tools, etc.
│
├── cli/         (TUI) → neotrix/* (ratatui, crossterm)
└── server/      (HTTP) → neotrix/* (axum)
```

**Key issue**: Every module can import every other. The monolith has NO enforced boundaries.

## 2. Proposed Crate Boundaries

### Crate 1: `neotrix-core` (Foundation)
- **Files**: `core/*` (all data models, traits, enums)
- **Deps**: serde, chrono (minimal)
- **API**: CapabilityVector, KnowledgeSource, ReasoningBank, MemoryProvider, etc.
- **Risk**: None — already zero external deps

### Crate 2: `neotrix-llm` (LLM Providers)
- **Files**: `provider/` (openai.rs, anthropic.rs, ollama.rs, gemini.rs, factory.rs, types.rs)
- **Deps**: reqwest, serde, serde_json, async-trait
- **Dep on**: neotrix-core (for trait LlmProvider, types)
- **Risk**: Low — clear boundary, no circular deps

### Crate 3: `neotrix-net` (Networking)
- **Files**: `stealth_net/`, `http_factory.rs`, `crawler/`, `scraper.rs`, `search/`, `duckduckgo.rs`
- **Deps**: reqwest, tokio, url, rand, chrono
- **Dep on**: neotrix-core (for error types)
- **Risk**: Medium — stealth_net has complex feature gates

### Crate 4: `neotrix-agent` (Agent Runtime)
- **Files**: `reasoning_brain/`, `orchestrator/`, `world_model.rs`, `agent_protocol/`, `mcp_tools.rs`, `subagent/`
- **Deps**: tokio, serde, serde_json, async-trait
- **Dep on**: neotrix-core, neotrix-llm, neotrix-net
- **Risk**: High — central integration hub, most circular deps

### Crate 5: `neotrix-ui` (CLI/Desktop)
- **Files**: `cli/`, `server/`
- **Deps**: ratatui, crossterm, axum, tokio
- **Dep on**: neotrix-core, neotrix-agent
- **Risk**: Low — thin layer on top

## 3. Dependency Resolution Strategy

### Circular Dependency Risks

```
neotrix-agent → neotrix-llm  (calls provider.complete())
neotrix-agent → neotrix-net  (calls scraper, crawler)
```

No circular deps if boundaries are clean. The only risk is if `neotrix-llm` needs types from `neotrix-agent` — it doesn't. LlmProvider trait lives in `neotrix-core`.

### Trait Placement

| Trait | Location (crate) |
|-------|-----------------|
| `LlmProvider` | neotrix-core (types.rs) |
| `KnowledgeProvider` | neotrix-core (knowledge.rs) |
| `MemoryProvider` | neotrix-core (traits.rs) |
| `AgentExecutor` | neotrix-core (traits.rs) |
| `Accessor` (R4-05) | neotrix-core (accessor.rs) |

### Cross-crate Types

Types that cross boundaries:
- `CapabilityVector` → neotrix-core, used everywhere
- `KnowledgeSource` → neotrix-core, used by agent
- `NeoTrixError` → neotrix-core, used everywhere
- `LlmRequest`/`LlmResponse` → defined in neotrix-core, used by agent+llm

All shared types stay in `neotrix-core`.

## 4. Migration Phases

### Phase 1: Extract `neotrix-core`
- Move `core/` to `crates/neotrix-core/`
- Create workspace Cargo.toml
- Update all imports `crate::core::` → `neotrix_core::`
- **Effort**: S (2-3h)
- **Risk**: Very low

### Phase 2: Extract `neotrix-llm`
- Move `neotrix/provider/` to `crates/neotrix-llm/`
- Move trait definitions to `neotrix-core`
- **Effort**: M (4-6h)
- **Risk**: Low

### Phase 3: Extract `neotrix-net`
- Move `stealth_net/`, `crawler/`, `scraper.rs`, `search/` to `crates/neotrix-net/`
- Feature gate `stealth-net` in this crate only
- **Effort**: L (8-12h)
- **Risk**: Medium — stealth_net has 22+ files

### Phase 4: Extract `neotrix-agent`
- Move `reasoning_brain/`, `orchestrator/`, `world_model.rs`, etc.
- This is the largest chunk (~40% of codebase)
- **Effort**: XL (16-24h)
- **Risk**: High — most integration points

### Phase 5: Extract `neotrix-ui`
- Move `cli/` and `server/`
- **Effort**: S (2-3h)
- **Risk**: Low

## 5. Impact Analysis

### Compile Time
| Phase | Before | After | Improvement |
|-------|--------|-------|-------------|
| Phase 1 | 285 files | ~30+255 | 10% (core cached) |
| Phase 2 | 285 files | ~30+15+240 | 20% (llm isolated) |
| Phase 3 | 285 files | ~30+15+40+200 | 35% (net isolated) |
| Phase 4 | 285 files | ~30+15+40+120+30 | 60%+ (agent cached) |
| Phase 5 | 285 files | ~30+15+40+120+30+10 | 65%+ (full split) |

### Build Order
```
neotrix-core → neotrix-llm → neotrix-net → neotrix-agent → neotrix-ui
                                                              → src-tauri
```

### Test Impact
- Unit tests migrate with their modules
- Integration tests need crate-prefixed imports
- CI matrix: 5 crates × 3 features = 15 check configurations

## 6. Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Circular deps discovered mid-migration | Medium | Phase 1 first (define all shared types) |
| Feature gate complexity (stealth-net, sandbox, telemetry) | High | Gate features at crate level, not workspace |
| Public API breakage for downstream consumers | Low | No downstream consumers yet |
| Cargo workspace learning curve | Low | Standard Rust practice |
| 3rd-party crate split needs (chromiumoxide, rkyv) | Low | Feature-gate optional deps per crate |
