# NeoTrix External Technology Fusion Architecture Plan

**Generated**: 2026-09-15 | **Cycle**: Comprehensive Multi-Source Absorption
**URLs Processed**: ~200+ external sources across 7 domains

---

## PART 1: ANALYSIS SUMMARY

### Source Categories Analyzed
| Category | Count | Primary NT Domain |
|----------|-------|-------------------|
| AI Agent Frameworks | 43 | NT-ACT |
| AI Research Papers | 25 | NT-CORE, NT-MIND |
| Code/Dev Tools | 30 | NT-ACT, NT-MIND |
| Browser/Security | 24 | NT-SHIELD, NT-WORLD |
| Model Routing/Proxy | 15 | NT-IO |
| Document/File | 14 | nt_file_ability |
| Design/System | 24 | des-architect, des-ui |
| **NEW BATCH**: Skills/OSS | 12+ | NT-ACT, NT-MEMORY |

### NEW URLs Key Additions
| URL | Category | NT Mapping | Action |
|-----|----------|------------|--------|
| PaddlePaddle/PaddleOCR | OCR | NT-WORLD | **P0: Production OCR** |
| addyosmani/agent-skills | Skills | NT-ACT | **P1: SKILL-SPEC contract** |
| NVIDIA/SkillSpector | Skill Verification | NT-MIND | **P2: Skill validation** |
| LMCache/LMCache | Cache | NT-MEMORY | **P1: Memory multitier** |
| DeusData/codebase-memory-mcp | Memory MCP | NT-MEMORY | **P1: Code-grounded memory** |
| TencentCloud/CubeSandbox | Sandbox | NT-WORLD | **P2: Secure sandbox** |
| mattpocock/skills | Skills | NT-ACT | **P1: Progressive disclosure** |
| vercel-labs/agent-skills | Skills | NT-ACT | **P1: Agent skill patterns** |
| supabase/agent-skills | Skills | NT-ACT | **P1: Agent skill patterns** |
| browser-use/browser-use | Browser | NT-WORLD | **P1: Browser automation** |
| jivoi/awesome-osint | OSINT | NT-SHIELD | **P2: OSINT collection** |
| PrimeIntellect/prime-agent | Agent | NT-ACT | **P1: Multi-agent** |
| headroomlabs-ai/headroom | Agent | NT-MIND | **P1: Context management** |
| Krafton.ai/whale | Optimization | NT-META | **P1: WHALE cycle** |
| Microsoft/Orchard | Agent Framework | NT-CORE | **P1: Harness-agnostic env** |
| future-agi/future-agi | Research | NT-MIND | **P2: AGI research** |
| openobserve/openobserve | Observability | NT-META | **P2: Telemetry** |
| onyx-dot-app/onyx | Observability | NT-META | **P2: Tracing** |
| msitarzewski/agency-agents | Multi-Agent | NT-ACT | **P1: Agent coordination** |

---

## PART 2: FUSION ARCHITECTURE — Reverse-Engineered from Foundational Models

### Core Axioms (Updated)

| Axiom | Source | NeoTrix Implementation |
|-------|--------|------------------------|
| Cost-Aware Routing | Spotify Shunt, FreeRouter | GWT salience + cost weight (A1) |
| Context as Scarce Resource | KVMem | Paged KV + adaptive compaction |
| Skill as Production Template | Easel, addyosmani | SKILL-SPEC.md contract (<200 lines) |
| Harness-Weight Co-Optimization | KRAFTON WHALE | NT-META adaptive cycle |
| Speculative Decoding | vLLM N-Gram | Fast/slow path reasoning |
| Semantic Pattern Routing | diagram-design | Primitive→Semantic token model |
| First-Principles Deconstruction | Build-Your-Own-X, FPAM | 6-layer architecture |
| Multi-Agent Graph Orchestration | Google ARTEMIS | Planner/Operator/Checker |
| Hierarchical Memory Placement | Colibri, LMCache | VRAM/RAM/NVMe memory tiers |
| Secure Progressive Disclosure | addyosmani, tech-leads | Skill loading protocol |
| Harness-Agnostic Environment | Microsoft Orchard | NT-WORLD sandbox abstraction |

### Universal Model Abstraction Layer

```
┌──────────────────────────────────────────────────────────────────┐
│                     NT-IO: Unified Model Gateway                   │
├──────────────────────────────────────────────────────────────────┤
│  Provider Abstraction:                                              │
│  ├── Local (Ollama) → Trusted                                     │
│  ├── Cloud (OpenAI/Anthropic) → Contracted                       │
│  ├── Proxy (FreeRouter/openfreerouter) → Fallback               │
│  ├── MoE (Colibri/DeepSeek-V4.1) → Tier-aware                  │
│  └── Cache (LMCache) → Hot-store accelerator                    │
├──────────────────────────────────────────────────────────────────┤
│  Routing Signals:                                                   │
│  ├── Cost weight (A1)                                              │
│  ├── Complexity classifier (14-dimension)                         │
│  ├── Speculative decoding confidence                              │
│  └── Context window utilization                                   │
├──────────────────────────────────────────────────────────────────┤
│  Inference Optimization:                                            │
│  ├── Speculative decoding (draft→verify)                          │
│  ├── Paged KV (GPU→Host→NVMe tiered)                             │
│  ├── N-gram context overlap                                       │
│  └── Adaptive compaction (<256K vs >256K)                      │
└──────────────────────────────────────────────────────────────────┘
```

### 6-Layer Architecture with External Mappings

```
┌─────────────────────────────────────────────────────────────┐
│ L6 Meta-Cognition: nt_meta + nt_repair + nt_nexus         │
│  WHALE cycle adaptation | Observability (onyx/openobserve) │
├─────────────────────────────────────────────────────────────┤
│ L5 Cognition: nt_core + nt_mind + E8 + GWT               │
│  E8 reasoning | Speculative decoding | Complexity classifier │
├─────────────────────────────────────────────────────────────┤
│ L4 Emotion: nt_feel core engine                            │
├─────────────────────────────────────────────────────────────┤
│ L3 Embodiment: nt_physical + nt_shield + nt_feel          │
│  HTTP interception | OSINT | Ghidra | Maigret | PenList │
├─────────────────────────────────────────────────────────────┤
│ L2 Perception: nt_world + nt_sense + OCR                  │
│  PaddleOCR | Browser-use | UnifiedCrawler | File Ability  │
├─────────────────────────────────────────────────────────────┤
│ L1 Action: nt_act + nt_io + nt_memory                     │
│  AgentLoop (Planner/Op/Checker) | Skill Registry | Typed Memory │
└─────────────────────────────────────────────────────────────┘
```

---

## PART 3: REDUNDANCY, FLAT DEFECTS & CROSS-DOMAIN MISALIGNMENT

### 🔴 Redundancy (Must Clean Up)

| Redundancy | Sources | Action |
|-----------|---------|--------|
| NT-ACT agent loop duplication | crewAI, OpenHands, claude-task-master, aider | Consolidate into single `nt_act::agent_loop` |
| Aider vs oh-my-openagent patterns | Both do code modification | Absorb hash-anchored verification |
| Memory systems overlap | nt_memory, memanto, agent-memory-atlas, codebase-memory-mcp | Unify under typed memory estates |
| Model routing duplication | FreeRouter, openfreerouter, eve, cmux | Single NT-IO provider gateway |
| Skill loading duplication | addyosmani, tech-leads, vercel-labs, supabase | Single SKILL-SPEC contract |
| Browser automation overlap | browser-use, camofox, nanobrowser | NT-WORLD perception + NT-SHIELD stealth |
| Knowledge base overlap | nt_memory (SQLite KB) + codebase-memory-mcp | Merge codebase memory into KB |

### 🟡 Flat Defects (Structural Issues)

| Defect | Description | Fix |
|--------|-------------|-----|
| No complexity dimension in GWT | Routes by cost only | Add 14-dimension classifier |
| No speculative decoding | No fast/slow path | Add confidence-based decoding |
| No OCR capability | PDF→text pipeline incomplete | Integrate PaddleOCR |
| No typed memory | SQLite KB lacks semantic categorization | Add memory estates |
| No HTTP interception | NT-SHIELD lacks request/response inspection | Add HTTP proxy |
| No harness-weight optimization | NT-META lacks adaptive cycles | Implement WHALE |
| No LMCache integration | No hot-store cache acceleration | Add LMCache adapter |
| No skill verification | No SkillSpector equivalent | Add skill validation |

### 🔵 Cross-Domain Misalignment

| Misalignment | Issue | Resolution |
|-------------|-------|------------|
| File ability ↔ Memory | Both handle file ingestion | `nt_file_ability` parses; `nt_memory` stores extracted knowledge |
| NT-WORLD ↔ NT-SHIELD | Browser automation spans both | NT-WORLD = perception; NT-SHIELD = stealth/fingerprint |
| NT-CORE ↔ NT-MIND | E8 reasoning + SEAL evolution overlap | NT-CORE = reasoning; NT-MIND = evolution |
| NT-ACT ↔ NT-IO | Agent execution and model routing overlap | NT-ACT executes; NT-IO manages model comms |
| NT-MEMORY ↔ NT-NEXUS | Memory and cross-session memory overlap | NT-MEMORY = current KB; NT-NEXUS = cross-session weaving |

---

## PART 4: CORE ROADMAP TASK LIST

### P0 — Foundation (Immediate)

| # | Task | Module | Source |
|---|------|--------|--------|
| P0-1 | **Unified Model Gateway** | NT-IO | FreeRouter, eve, openfreerouter |
| P0-2 | **PaddleOCR Integration** | NT-WORLD | PaddlePaddle/PaddleOCR |
| P0-3 | **Agent Loop Consolidation** | NT-ACT | ARTEMIS, LongHorizon-Harness |
| P0-4 | **Typed Semantic Memory** | NT-MEMORY | Memanto, LMCache |
| P0-5 | **Complexity Classifier** | NT-CORE | openfreerouter, llmrouter |

### P1 — Enhancement (Near-Term)

| # | Task | Module | Source |
|---|------|--------|--------|
| P1-1 | **Speculative Decoding** | NT-CORE | vLLM N-Gram, Outcome School |
| P1-2 | **HTTP Interception Proxy** | NT-SHIELD | Gori, defending-code-harness |
| P1-3 | **WHALE Cycle Adaptation** | NT-META | KRAFTON WHALE |
| P1-4 | **LMCache Integration** | NT-MEMORY | LMCache/LMCache |
| P1-5 | **OSINT Collection** | NT-SHIELD | awesome-osint, maigret |
| P1-6 | **Secure Skill Registry** | NT-ACT | addyosmani, tech-leads |
| P1-7 | **SkillSpector Validation** | NT-MIND | NVIDIA/SkillSpector |
| P1-8 | **Codebase Memory MCP** | NT-MEMORY | DeusData/codebase-memory-mcp |
| P1-9 | **Browser Automation** | NT-WORLD | browser-use, nanobrowser |
| P1-10 | **Harness-Agnostic Sandbox** | NT-WORLD | Microsoft Orchard, CubeSandbox |

### P2 — Expansion (Medium-Term)

| # | Task | Module | Source |
|---|------|--------|--------|
| P2-1 | **Chart/Figure Generation** | NT-IO | vivid-figures-skill |
| P2-2 | **Autonomous Pentest** | NT-SHIELD | Penetration-List |
| P2-3 | **Disaggregated Pipeline** | NT-CORE | vLLM-Omni |
| P2-4 | **Knowledge Graph Integration** | NT-MEMORY | agent-memory-atlas |
| P2-5 | **Observability Stack** | NT-META | onyx, openobserve |
| P2-6 | **Prime-Agent Coordination** | NT-ACT | PrimeIntellect/prime-agent |

---

## PART 5: MULTI-AGENT INSPECTION SQUAD

### Squad Structure
```
NT-CORE (Orchestrator)
├── Audit Agent (rev-officer) — D1-D63 full review, every 5 cycles
├── Build Agent (dev-implementer) — cargo check + test, every commit
├── Memory Agent — KB health + experience, 60s tick
└── Security Agent — NT-SHIELD audit, weekly
├── World Agent — NT-WORLD crawl health, daily
├── IO Agent — NT-IO provider health, daily
├── Mind Agent — SEAL pipeline health, every cycle
└── Design Agent — Architecture review, monthly
```

### Agent Prompt Templates

**Audit Agent**: Full rev-officer D1-D63 review with FPAM methodology
**Build Agent**: cargo check + cargo test + clippy verification
**Memory Agent**: KB integrity, experience-tree hub validation, ghost branch cleanup
**Security Agent**: NT-SHIELD penetration test + HTTP intercept validation
**World Agent**: NT-WORLD crawl pipeline + OCR accuracy validation
**IO Agent**: NT-IO provider health + routing efficiency + cost analysis
**Mind Agent**: SEAL pipeline + WHALE cycle health + skill crystallization
**Design Agent**: Architecture review + six-layer compliance + ADR validation

---

## PART 6: ABSORPTION ENTRIES

This plan will be written to `~/.neotrix/pending-absorb.json` for background absorption cycle processing.

**Key Absorption Domains**:
- `domain_nt_core`: E8 reasoning + GWT routing + complexity classifier + speculative decoding
- `domain_nt_mind`: WHALE cycle + SEAL pipeline + skill crystallization + SkillSpector
- `domain_nt_memory`: Typed memory estates + LMCache + codebase memory + Paged KV
- `domain_nt_world`: PaddleOCR + browser-use + CubeSandbox + UnifiedCrawler
- `domain_nt_act`: AgentLoop + skill registry + secure progressive disclosure
- `domain_nt_io`: Unified Model Gateway + provider abstraction + LMCache adapter
- `domain_nt_shield`: HTTP interception + OSINT + PenList + Ghidra integration
- `domain_nt_meta`: Observability stack + WHALE adaptation + cross-module audit

