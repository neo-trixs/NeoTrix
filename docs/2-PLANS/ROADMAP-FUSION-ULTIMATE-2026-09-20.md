# NeoTrix Architecture Fusion - Ultimate Roadmap

> 35 external repos reverse-engineered + full code audit + base model reasoning
> Generated: 2026-09-20 | Codebase: 2,159 files / ~700K LOC / 0 compilation errors

---

## 1. External Repository Absorption Matrix (35 repos)

### P0 - Immediate Absorption (Core Architecture)

| Repo | Stars | Core Pattern | Target Module | Layer |
|------|-------|-------------|---------------|-------|
| fast-jev-compaction | 4.4K | Lossless context compaction (Jev scoring) | nt_harness/compaction | L1 |
| Agent-Reach | 83.5K | Capability layer + multi-backend routing | nt_ability_net | L1 |
| Jev MCP | 111 | Typed judgment primitives (10 tools) | nt_judgment (DONE) | L2 |
| SoL-Pi | 2.4K | 4 efficiency mechanisms | nt_harness (DONE) | L1 |
| LLMRouter | 2.9K | 16+ routing models + plugin system | nt_routing | L2 |
| council-of-high-intelligence | 4.3K | Multi-perspective deliberation + 18 personas | nt_council | L5 |
| codex-security | 10.8K | Security scanning CLI + SDK + findings service | nt_security | L3 |

### P1 - This Week (High Value)

| Repo | Stars | Core Pattern | Target Module | Layer |
|------|-------|-------------|---------------|-------|
| Scrapling | 82.5K | Adaptive selectors + anti-detection + spider | nt_web_perception | L2 |
| DeepTeam | 2.9K | 50+ vuln types + 20+ attack methods | nt_governance/red_team | L6 |
| Jev-cu | 318 | Observe-Decide-Execute loop + policy gates | nt_agent/ode | L5 |
| ARES | 259 | DAG solver + scope firewall + noise governor | des/architect | L5 |
| Hermes Agent | 5.7K | Layered arch + trust boundary + maturity taxonomy | Full-layer reference | Meta |
| Exegol | 3.1K | Modular security env + Docker orchestration | nt_security/env | L3 |
| Open-LLM-VTuber | - | Real-time voice + expression + LLM pipeline | nt_media/vtuber | L3 |
| tirith | - | AI security scanning + policy execution | nt_security/policy | L3 |
| Map3d | 2.4K | R3F 3D map + OSM data | nt_presentation/3d | L6 |

### P2 - Reference Absorption

AntV Infographic (6.7K), Patchright Enhanced (135), deskport (138), laya-mlx, orca, pond, mira, openhermit, takt, dopbase, user-scanner, awesome-ai-security-tools, coolapk-desktop, wx-cli-again, Crucix, cumora, mr-boxington

---

## 2. Architecture Diagnosis

### Layer Size

| Layer | Files | % | Status |
|-------|-------|---|--------|
| L0 | 43 | 2.0% | 6 upward deps (CRITICAL) |
| L1 | 646 | 29.9% | Bloated |
| L2 | 335 | 15.5% | Normal |
| L3 | 253 | 11.7% | Normal |
| L4 | 19 | 0.9% | Too small |
| L5 | 656 | 30.4% | Largest layer |
| L6 | 207 | 9.6% | Normal |

### L0 Upward Dependencies (6 violations)

| File | Violation |
|------|-----------|
| nt_core_traits.rs:6 | -> L1 nt_core_bank (ReasoningBankStats, ReasoningMemory) |
| nt_core_traits.rs:8 | -> L2 nt_core_knowledge (AbsorptionRecord, KnowledgeSource) |
| nt_core_traits.rs:99 | -> L2 KnowledgeProvider |
| nt_core_telemetry.rs:7 | -> L1 nt_core_llm (Usage) |
| nt_core_cross_layer.rs:10 | -> L2 nt_core_e8 (E8TransitionMatrix) |
| nt_core_platform/init.rs:54 | -> L1 agent_protocol (AgentOrchestrator) |

### Key Issues

1. L0 -> L1/L2: substrate references action and perception
2. L1 bloat: 646 files (29.9%)
3. L4 too small: 19 files (0.9%)
4. 260 TODO/FIXME items
5. L5 largest: 656 files (30.4%)

---

## 3. Core Roadmap Task List

### Sprint 1: Foundation Repair (Day 1-2)

**T1.1 [CRITICAL] L0 Upward Dependencies -> 0**
- nt_core_traits.rs: Move ReasoningBankStats/ReasoningMemory/AbsorptionRecord/KnowledgeSource/KnowledgeProvider to L0 or neotrix-types
- nt_core_telemetry.rs: Move Usage to L0
- nt_core_cross_layer.rs: Move E8TransitionMatrix to L0
- nt_core_platform/init.rs: Move AgentOrchestrator trait to L0
- Verify: grep count = 0

**T1.2 [HIGH] fast-jev-compaction Pattern Absorption**
- Create nt_harness/compaction/mod.rs
- Core: LosslessCompactor -- Jev scoring -> delete/truncate/keep
- Types: CompactionDecision, ToolCallScore, CompactionResult
- Consumer: consciousness pipeline context management

**T1.3 [HIGH] LLMRouter Pattern Absorption**
- Create nt_routing/mod.rs
- Core: RouteStrategy trait + 16+ routing strategies
- Types: RouteDecision, RouteCandidate, RouteMetrics
- Consumer: consciousness_task task routing

**T1.4 [HIGH] Council Pattern Absorption**
- Create nt_council/mod.rs
- Core: CouncilPersona trait + 18 personas + multi-perspective deliberation
- Types: CouncilVerdict, PersonaContract, EvidenceLabel
- Consumer: rev-officer decision review

### Sprint 2: Capability Absorption (Day 3-5)

**T2.1 [P0] nt_security -- codex-security + Exegol + tirith**
- SecurityScanner, SecurityPolicy, ScanResult
- Find/validate/fix vulnerabilities + environment management

**T2.2 [P0] nt_web_perception -- Scrapling**
- AdaptiveSelector, StealthFetcher, CrawlSession
- Self-healing selectors + anti-bot + concurrent spider

**T2.3 [P1] nt_governance/red_team -- DeepTeam**
- VulnerabilityTaxonomy, AttackMethod, Guardrail
- 50+ vulnerability types + 20+ attack methods

**T2.4 [P1] nt_agent/ode -- Jev-cu**
- ObserveDecideExecute, PolicyGate, DriverAdapter
- AX parse -> candidate select -> decision -> policy gate -> execute

**T2.5 [P1] nt_media/vtuber -- Open-LLM-VTuber**
- VoicePipeline, ExpressionDriver
- Real-time voice + expression + LLM pipeline

### Sprint 3: Redundancy Cleanup (Day 6-7)

**T3.1 L1 Slimming: 646 -> ~400 files**
- Memory-related -> L4
- IO-related -> separate nt_io crate

**T3.2 L4 Expansion: 19 -> ~80 files**
- Sink emotion and memory modules from L1/L5

**T3.3 TODO/FIXME: 260 -> <=50**
- Classify: implement vs remove
- Stub functions: 46 -> 0

**T3.4 Facade Enforcement**
- L2->L1 100% via l1_facade
- CI check script

### Sprint 4: Integration Verification (Day 8-10)

**T4.1 Full Compilation: 0 errors, 0 warnings**
**T4.2 Architecture Constraints: L0 zero upward + facade 100% + no duplicate types**
**T4.3 Multi-agent Inspection: rev-officer + clippy + architecture gate**

---

## 4. Target Architecture (Post-Sprint 4)

```
L6 Meta
  nt_governance (red_team: DeepTeam 50+ vulns)
  nt_meta (dream_replay, evolution)
  consciousness (orchestrator)
  nt_presentation (Map3d 3D + AntV declarative)

L5 Cognition
  nt_council (18-persona deliberation)
  nt_mind (cost_ladder, evidence_gating)
  nt_agent/ode (observe-decide-execute, Jev-cu)
  rev-officer, methodology-researcher

L4 Emotion
  emotion_engine
  nt_memory/cascade (five-tier memory cascade) <- sunk from L5
  emotion calculation modules <- sunk from L1

L3 Embodiment
  nt_security (codex-security scan + Exegol env + tirith policy)
  nt_media/vtuber (Open-LLM-VTuber)
  shield, compliance

L2 Perception
  nt_world (OSINT, crawl, knowledge)
  nt_judgment (Jev MCP 10 tools) DONE
  nt_web_perception (Scrapling adaptive)
  nt_routing (LLMRouter 16+ strategies)

L1 Action
  nt_harness (SoL-Pi 4 mechanisms + fast-jev compaction) DONE
  nt_ability_net (Agent-Reach multi-backend) DONE
  nt_command, nt_io, nt_media
  l1_facade (L2->L1 unified facade)

L0 Substrate
  nt_core_types (unified core types)
  nt_core_traits (base traits)
  nt_core_error (unified error)
  nt_core_cross_layer (L6->L0 re-exports)
  ZERO upward dependencies
```

---

## 5. Success Metrics

| Metric | Current | Sprint 4 Target |
|--------|---------|-----------------|
| L0 upward deps | 6 | 0 |
| L1 file count | 646 | ~400 |
| L4 file count | 19 | ~80 |
| Duplicate types | 40+ | <=5 |
| TODO/FIXME | 260 | <=50 |
| Stub functions | 46 | 0 |
| Orphan modules | 23 | 0 |
| Compilation errors | 0 | 0 |
| New absorption modules | 3 (judgment, cascade, harness) | 10 |
| External patterns fused | 3 | 12 |
