# Phase 27: RSI 生态全景观分析 (Landscape Analysis)

> **Build**: 2026-06-24 | **Based on**: 30+ project deep-dive, 40+ paper search
> **Coverage**: Self-evolving coding agents, RSI frameworks, consciousness architectures, VSA/HDC, Gödel machines

---

## 1. 生态全景图 (30+ Projects, 7 Categories)

### Category A — Self-Evolving Coding Agents (7)
| Project | Stars | Lang | Key Innovation | NeoTrix Gap |
|---------|:-----:|:----:|----------------|-------------|
| yoyo-evolve | 1,827 | Rust | 200 LOC seed → 100K+ LOC/107d, autonomous GH Actions evolution every ~8h | ❌ No autonomous source-modify loop |
| MUE-X | 61 | Python | Reads/rewrites own .py files via AST, continuous mutate→validate→apply | ❌ No AST-level self-mutation |
| Ouroboros (razzant) | 654 | Python | Self-modifying agent with constitution, background consciousness, identity persistence | ❌ No background consciousness outside ticks |
| Ouroboros (secondorderai) | 2 | TS | Electron desktop RSI agent, dream consolidation, crystallization | ❌ Dream consolidation only in SleepCycle |
| iterate (GrayCodeAI) | 6 | Go | GitHub Actions every 12h, reads own source, plans, builds, tests, commits | ❌ No cron-driven autonomous evolution |
| Loom | — | ClojureScript | 2-stage gate (tests + LLM review), 1.4% promotion rate | ❌ Two-stage promotion gate |
| GitPup | — | Python | 24/7 VPS cron, GitHub exploration → PR creation, skill extraction | ❌ No autonomous PR creation |

### Category B — RSI Frameworks (6)
| Project | Stars | Lang | Key Innovation | NeoTrix Gap |
|---------|:-----:|:----:|----------------|-------------|
| GBase | — | Python | Recursive self-improvement: stability audit → performance eval → rollback | ❌ Stability audit before every evolution |
| Autogenesis/SEPL | 61 | Python | 5-operator formal algebra (ρσικε) + RSPL protocol registration | ❌ SEPL ops not as formal traits |
| EvoAgentX | 3,087 | Python | Self-evolving workflow optimizer, survey on self-evolving agents | ❌ Workflow-level evolution |
| AgentEvolver | 1,452 | Python | Self-questioning/navigating/attributing, RL-based evolution | ❌ RL-based evolution loop |
| Yunjue-Agent | 502 | Python | In-situ self-evolving: tool synthesis from interaction traces | ❌ Tool synthesis from traces |
| OmniAgent | 2,040 | Python | 4-layer security scanning, full-dimensional self-evolution | ❌ 4-layer dynamic security scanning |

### Category C — Evolution Engines (3)
| Project | Stars | Key Innovation | NeoTrix Gap |
|---------|:-----:|----------------|-------------|
| EvoMap/Evolver | 8,716 | GEP-powered evolution: Genes → Capsules → Events, auditable lineage | ❌ No GEP integration |
| DGM-HyperAgents (Meta) | 2,600 | Archive tree + meta-agent self-modification | ❌ No archive tree (linear Vec) |
| Escher-Loop (arXiv) | — | Dual-population co-evolution with exploration bonus | ✅ Built as EscherLoopEngine |

### Category D — Rust Cognitive Architectures (7)
| Project | Lang | Key Innovation | NeoTrix Gap |
|---------|:----:|----------------|-------------|
| pulse-null | Rust | Prediction-error loop, Beta-Binomial confidence, 4-layer memory | ✅ Similar architecture |
| Life (broomva) | Rust | 13 modules, 76 crates, event-sourced persistence (Lago) | ❌ No event-sourced persistence |
| HIVE | Rust | Anti-spiral recovery, 5-tier memory, NeuroLease mesh | ❌ Anti-spiral recovery |
| Syntra Kernel | Rust | World-model cognition, evolution engine, WASM sandbox | ❌ WASM cartridge runtime |
| nano-consciousness | Rust | IIT Φ calculation, nanosecond scheduler, STDP | ✅ Similar IIT approach |
| xagent | Rust | 7-stage predictive processing on GPU, free energy principle | ❌ No GPU-resident cognitive runtime |
| consciousness-kernel | Rust | 32-stage Buddhist-epistemology pipeline, governance-ledger | ❌ Missing governance-ledger |

### Category E — VSA/HDC Research (6)
| Project | Key Innovation | NeoTrix Gap |
|---------|----------------|-------------|
| HyperSpace | Modular VSA framework: abstract modules for encoding/binding/cleanup | ❌ No VSA abstraction layer |
| hdlib 2.0 | Quantum HDC, regression, clustering, graph-based VSA learning | ✅ Already using VSA core |
| SynapseHD | SNN + HDC hybrid, Atkinson-Shiffrin memory model | ❌ No SNN integration |
| SRMU | Relevance-gated VSA memory updates, temporal decay | ❌ No relevance-gated memory |
| VaCoAl | Galois-field algebra, STDP-equivalent emergent selection | ❌ No Galois-field VSA |
| Frontiers '26 | Kernel-width adapts to learning vs cognition tasks | ✅ Already adaptive |

### Category F — Gödel Machines (4)
| Project | Key Innovation | NeoTrix Gap |
|---------|----------------|-------------|
| Gödel Agent (ACL 2025) | Self-referential agent framework, LLM-driven self-modification | ❌ No self-referential meta-agent |
| DGM (Sakana) | Open-ended archive tree, SWE-bench 20%→50% | ❌ No archive tree |
| DGM-H (Meta) | Meta-agent rewrites own improvement strategy | ❌ Meta-layer hardcoded |
| Huxley-Gödel | CMP (Clade-level Meta-Productivity) for expansion selection | ❌ No clade-level selection |

### Category G — Startup Landscape (6)
| Company | Raised | Valuation | Focus |
|---------|:------:|:---------:|-------|
| Recursive Superintelligence | $650M | $4.65B | Open-ended RSI for AI research automation |
| Cognition (Devin) | $1B+ | $26B | Autonomous software engineer |
| Blitzy | $200M | $1.4B | Enterprise autonomous development (66.5% SWE-Bench Pro) |
| Anthropic | — | — | 80%+ code written by Claude |
| Recursive (China) | $1.2B | $4.6B | Self-improving models for code + science |
| RELAI | $6.9M | — | Verifiable continual learning with regression control |

---

## 2. NeoTrix 竞争差距矩阵 (Critical Findings)

### 🟢 Areas Where NeoTrix Leads
| Capability | NeoTrix | Best External | Edge |
|-----------|---------|---------------|------|
| Consciousness cycle (12-step) | ✅ 23/25 subsystems wired | pulse-null: 8-stage pipeline | +++ |
| VSA hyperdimensional core | ✅ 4096-bit binary VSA, FWHT cleanup | HyperSpace: modular but no binary | ++ |
| Three-loop self-evolution | ✅ Small/Big/Meta | GBase: single evolution_cycle() | +++ |
| Self-model synthesis | ✅ SelfModelGenerator | Ouroboros: BIBLE.md static | +++ |
| Web perception pipeline | ✅ WebContentExtractor | — | ++ |
| Behavioral personality | ✅ OCEAN+PAD+Fingerprint | — | +++ |

### 🔴 Critical Gaps Not in v22 Roadmap
| Gap | Source | Impact | Fix Estimate |
|-----|--------|--------|:------------:|
| **No self-modifying source code loop** | yoyo, MUE-X, Ouroboros, iterate | NeoTrix can't autonomously improve its own Rust source | ~400 LOC (CronEvolutionAgent) |
| **No autonomous background evolution cron** | yoyo (8h), iterate (12h), GitPup (24/7) | Evolution only runs inside consciousness ticks | ~150 LOC (BackgroundEvolutionScheduler) |
| **No anti-spiral recovery** | HIVE, OmniAgent | Reasoning loops can consume infinite cycles | ~300 LOC (AntiSpiralMonitor) |
| **No WASM cartridge/hot-swap** | HIVE, tempo-x402, Syntra Kernel | Self-modification requires recompile, no runtime safe swap | ~500 LOC + wasm crate |
| **No GEP integration** | EvoMap/Evolver (8.7k★) | Evolution not auditable as genes/capsules/events | ~350 LOC (GepAdapter) |
| **No event-sourced persistence** | Life/Lago (76 crates) | NTSSEG append-only but no event stream replay | ~400 LOC (EventStore) |
| **No governance-ledger** | consciousness-kernel (838 tests) | No ethical/psychological governance layer | ~250 LOC |
| **No clade-level meta-productivity** | Huxley-Gödel Machine | Selection ignores descendant potential | ~300 LOC (CmpEstimator) |

### 🟡 Medium Gaps
| Gap | Source | Fix Estimate |
|-----|--------|:------------:|
| SNN+HDC hybrid (SynapseHD) | SynapseHD | ~600 LOC (SNN feature extractor) |
| 2-stage promotion gate (Loom) | Loom (1.4%→higher promotion) | ~200 LOC |
| GPU-resident cognitive runtime | xagent (7-stage GPU kernel) | ~800 LOC |
| Relevance-gated memory (SRMU) | SRMU (VSA-based) | ~200 LOC |
| RL-based evolution (AgentEvolver) | AgentEvolver | ~350 LOC |

---

## 3. Phase 27 进化路线图 (修正版 v23)

### Wave 0 — Self-Modifying Source Loop (NEW, ~950 LOC)
> **源自**: yoyo-evolve, MUE-X, Ouroboros — 最大架构缺口

| Module | LOC | Tests | Priority | Description |
|--------|:---:|:-----:|:--------:|-------------|
| CronEvolutionScheduler | ~150 | 6 | P0 | Background tokio task every N hours, reads own source, creates tasks in EvolutionTaskSystem |
| SelfSourceReader | ~250 | 10 | P0 | Parse Rust source files, extract function signatures, identify improvement targets |
| AstMutationEngine | ~300 | 12 | P0 | AST-level mutation generation (not string search), `syn`-based |
| AutoCommitGate | ~250 | 8 | P0 | Compile→test→clippy verify before commit, rollback on failure |
| **Total** | **~950** | **36** | | |

### Wave A — SEPL Formalization (~960 LOC)
*(unchanged from v22)*

### Wave B — GEPA Structured Diagnosis (~1,400 LOC)
*Adds:* ArchiveManager (DGM-H tree), TraceEncoder, ReflectiveAnalyzer

### Wave C — DGM-H Self-Reference + Safety (~2,200 LOC)
*Adds:* AntiSpiralMonitor, WASM cartridge init, TasteGovernor, SandboxEvaluator
| Module | LOC | Tests | Priority | Description |
|--------|:---:|:-----:|:--------:|-------------|
| AntiSpiralMonitor | ~300 | 12 | P0 | Detect repeated failures, identical proposals, oscillation patterns |
| WasmCartridgeInit | ~500 | 16 | P1 | WASM runtime for safe self-modification cartridge hot-swap |
| SteppingStoneArchive | ~300 | 14 | P0 | Tree archive with fork/merge |
| TasteGovernor | ~300 | 14 | P0 | 8-gate immutable amendment policy |
| MetaSelfModifier | ~350 | 16 | P0 | Agent rewrites SEAL own code |
| DgmBridge | ~200 | 10 | P0 | Archive ↔ evolution loop |
| CrossDomainTransfer | ~250 | 10 | P1 | Multi-domain mutation validation |
| SandboxEvaluator | ~250 | 10 | P1 | Runtime sandbox |
| **Total Wave C** | **~2,200** | **72** | | |

---

## 4. 立即行动清单 (Priority TODOs)

### 🔴 Today
1. **Build CronEvolutionScheduler** — Background loop for autonomous source evolution
2. **Build AntiSpiralMonitor** — Prevent reasoning loop explosion
3. **Install Rust `syn` + `wasmtime` crates** — Foundation for Wave 0 and Wave C

### 🟡 This Week
4. SEPL formal traits (5 operators as independent trait definitions)
5. CommitGate with full rollback
6. AutoCommitGate (compile→test→clippy)

### 🟢 Next Week
7. ArchiveManager (DGM-H tree)
8. TraceEncoder (structured diagnosis)
9. ReflectiveAnalyzer

---

## 5. Market Context

| Signal | Source | Implication |
|--------|--------|-------------|
| $2B+ into RSI startups in 2026 | RS, Cognition, Blitzy | Market validates RSI as THE direction |
| Anthropic: 80%+ code auto-written | Claude internal report | RSI is already happening at frontier labs |
| ICLR 2026 RSI Workshop (100+ papers) | ICLR | Academic legitimacy achieved |
| OpenClaw → OpenAI acquisition | 195k★ → OpenAI | Self-modifying code agents are tier-1 talent target |
| 30+ open-source RSI projects | This survey | Ecosystem is real, not theoretical |
