# NeoTrix Architecture Diagrams

ASCII diagrams of key NeoTrix subsystems. Source-of-truth: `neotrix-core/src/`.

---

## 1. 6-Layer Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  L6  META-COGNITION (nt_meta · nt_repair · nt_nexus)                   │
│      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│      │ UnifiedOrch   │ │ RepairHealer │ │ NexusWeaver  │                │
│      │ 13 dimensions │ │ MAPE-K loop  │ │ cross-session│                │
│      └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                │
├─────────────┼────────────────┼────────────────┼────────────────────────┤
│  L5  COGNITION (nt_core · nt_mind)                                     │
│      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│      │ Consciousness │ │ SEAL Pipeline│ │ Reasoning    │                │
│      │ Core (E8+GWT) │ │ 4-stage train│ │ + Knowledge  │                │
│      └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                │
├─────────────┼────────────────┼────────────────┼────────────────────────┤
│  L4  EMOTION (nt_feel)                                                 │
│      ┌──────────────────────────────────────────────────┐              │
│      │ EmotionEngine · FEP-IIT Bridge · EmotionLabel    │              │
│      │ 11 variants · Affective reward context           │              │
│      └──────────────────────┬───────────────────────────┘              │
├─────────────────────────────┼──────────────────────────────────────────┤
│  L3  EMBODIMENT (nt_physical · nt_shield · nt_feel)                    │
│      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│      │ Physical     │ │ SHIELD       │ │ Feel Body    │                │
│      │ sensors/motor│ │ 12+ impls    │ │ embodiment   │                │
│      └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                │
├─────────────┼────────────────┼────────────────┼────────────────────────┤
│  L2  PERCEPTION (nt_world · nt_sense)                                  │
│      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│      │ World Crawl  │ │ SensoryHub   │ │ Perception   │                │
│      │ OSINT/Router │ │ GWT gating   │ │ Bridge       │                │
│      └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                │
├─────────────┼────────────────┼────────────────┼────────────────────────┤
│  L1  ACTION (nt_act · nt_io · nt_memory)                               │
│      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│      │ Tools/Actions│ │ IO/Interface │ │ Memory KB    │                │
│      │ CapabilityReg│ │ CLI/FFI/UI   │ │ TieredStore  │                │
│      └──────────────┘ └──────────────┘ └──────────────┘                │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Consciousness Core Flow

```
  User Instruction
        │
        ▼
┌───────────────────┐
│ 1. DECOMPOSE      │  ConsciousnessCore::process_instruction
│   ├─ Intent Parse │  splits into ConsciousTask[]
│   └─ SubTask DAG  │  with dependency graph
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 2. CAPABILITY     │  CAPABILITY_ROUTES keyword table
│    ROUTE MATCH    │  (keyword → domain → agent routing)
│   ┌─────────────┐ │
│   │ "excel"     │→│ → NT-ACT   · CodeAnalyzer
│   │ "审查"      │→│ → NT-SHIELD· RiskAssessor
│   │ "架构"      │→│ → NT-CORE  · Planner
│   │ "元认知"    │→│ → NT-META  · MetaCognition
│   └─────────────┘ │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 3. DISPATCH &     │  dispatch_internal_capability
│    EXECUTE        │  or LLM SubagentDispatch
│   ┌─ Internal ──┐ │  for complex tasks
│   │ built-in    │ │
│   └─ External ──┘ │  external_closure: knowledge
│   │ LLM + KB   │ │  acquisition + trial-and-error
│   └────────────┘  │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 4. REFLECT &      │ 反思补齐: fill gaps, re-dispatch
│    ABSORB         │  absorb results into KB
│   ├─ Gap Detect  │  experience-tree 5-stage
│   └─ KB Write    │  snapshot→distill→classify→persist→feedback
└───────────────────┘
```

---

## 3. SEAL Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    SEAL Self-Iterating Loop                     │
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │ EXPLORE  │───▶│ DISTILL  │───▶│   TEST   │───▶│  ABSORB  │  │
│  │          │    │          │    │          │    │          │  │
│  │ Acquire  │    │ Extract  │    │ SelfTest │    │ KB +     │  │
│  │ papers/  │    │ patterns │    │ T1/T2/T3 │    │ Skill    │  │
│  │ repos/   │    │ compress │    │ regress  │    │ Registry │  │
│  │ docs     │    │ crystallize│   │ converge │    │          │  │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘  │
│       │               │               │               │         │
│       ▼               ▼               ▼               ▼         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │           BrainStage Pipeline (24+ stages)              │    │
│  │                                                         │    │
│  │  AntiDistillation · ConstitutionalSelfCritique ·        │    │
│  │  MetaRsi · ConceptEmergence · SafetyCheck ·             │    │
│  │  BenchmarkGate · ProceduralMemory · DpSgd ·            │    │
│  │  HyperArchive · HyperMetaAgent · DGMMetaEvolve ·       │    │
│  │  OpenSpaceEvolve · ProcessStage · SecretScanner ·      │    │
│  │  ValidationGate · BoundedEdit · EpochSlowUpdate ·       │    │
│  │  GoalTerminator · NarrowRecovery · SemanticRecall ·     │    │
│  │  EvidenceCapture · ExternalVerifier · FinalVerification │    │
│  └─────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│  ┌─────────────────────────▼───────────────────────────────┐    │
│  │ SKILL.state Integration (arXiv:2608.26263)              │    │
│  │  ├─ ContractAwareStage wraps Distill                    │    │
│  │  ├─ DeliveryPromiseContract prevents silent degradation │    │
│  │  └─ StageInsight → Cyclic/DeadEnd/Stalling/Escalate    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ◀──── Reward Feedback (GRPO / ExternalReward) ──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Knowledge Base Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      KnowledgeBase                              │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  TieredStore                            │    │
│  │                                                         │    │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │    │
│  │   │   HOT (RAM) │  │ WARM (SQLite)│  │ COLD (JSONL)│   │    │
│  │   │             │  │              │  │             │   │    │
│  │   │ LRU 10K     │  │ Indexed     │  │ Archive     │   │    │
│  │   │ ms read     │  │ ms query    │  │ sec read    │   │    │
│  │   │             │  │ access_count│  │             │   │    │
│  │   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘   │    │
│  │          │                │                │           │    │
│  │     demote ───────▶     │    ◀────── promote          │    │
│  │     (stale >1h)         │      (access >= 5)          │    │
│  │                         │                             │    │
│  │                    archive                            │    │
│  │                   (>24h + low freq)                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────┐   │
│  │ Graph Store   │  │ Vector Store  │  │ Experience Index  │   │
│  │ (Rust SQLite) │  │ (embeddings)  │  │ (KB namespace)    │   │
│  │ Nodes + Edges │  │ Hybrid RAG    │  │ experience-tree   │   │
│  └───────┬───────┘  └───────┬───────┘  └─────────┬─────────┘   │
│          └──────────────────┼────────────────────┘             │
│                             │                                  │
│  ┌──────────────────────────▼──────────────────────────────┐   │
│  │            MemoryOrchestrator                           │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │ Working  │ │ Episodic │ │ Semantic │ │Procedural│  │   │
│  │  │ (short)  │ │ (events) │ │ (facts)  │ │ (skills) │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  │  TTL-based decay · half-life · promotion thresholds     │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Security Architecture (NT-SHIELD)

```
┌─────────────────────────────────────────────────────────────────┐
│                      NT-SHIELD                                  │
│                                                                 │
│  ┌─── Input Layer ─────────────────────────────────────────┐   │
│  │  InputGatekeeper ──▶ PromptGuardian ──▶ SlangNorm       │   │
│  │  (injection detect)  (policy check)    (normalize)      │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                  │
│  ┌─── Defense Layer ────────▼──────────────────────────────┐   │
│  │  UnifiedDefenseLayer                                    │   │
│  │  ├─ ReasoningProtectionEngine  (CoT block strip)        │   │
│  │  ├─ RefusalTamperEngine        (refusal integrity)      │   │
│  │  ├─ AntiDistillationEngine     (model protection)       │   │
│  │  ├─ GuardrailTraversalEngine   (boundary enforcement)   │   │
│  │  ├─ DualEvidenceScanner        (dual-channel verify)    │   │
│  │  └─ ProxyDetectionEngine       (proxy/VPN detect)       │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                  │
│  ┌─── Output Layer ─────────▼──────────────────────────────┐   │
│  │  OutputSentinel ──▶ ContextBoundary                     │   │
│  │  (leak scan)         (trust level gate)                 │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                  │
│  ┌─── Impl Modules ─────────▼─────────────────────────────┐   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │ Pentest  │ │ Vuln     │ │ Internal │ │ Web      │  │   │
│  │  │ Agent    │ │ Scanner  │ │ Scan     │ │ Scanner  │  │   │
│  │  │ (AI)     │ │ (nuclei) │ │ (fscan)  │ │ (w3af)   │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │ Pentest  │ │ Reverse  │ │ Mobile   │ │ AI       │  │   │
│  │  │ Swarm    │ │ Engineer │ │ Analyzer │ │ Security │  │   │
│  │  │ (multi)  │ │ (ghidra) │ │(objection│ │ (ART)    │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                  │
│  ┌─── Infrastructure ───────▼─────────────────────────────┐   │
│  │  ShieldCore · Sandbox (Docker/device/remote)           │   │
│  │  StealthNet · ZTNet (zero-trust) · Guard Chain        │   │
│  │  Safety · Evasion · PropagationGuard                   │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                 │
│  OWASP Coverage:                                               │
│  ├─ API Security Top 10 (2023): 8 attack types + payloads     │
│  ├─ Reasoning Trace Guard: strip CoT blocks                    │
│  ├─ CoH Guard: chain-of-thought hygiene                        │
│  └─ CVSS scoring: vulnerability severity classification        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Dispatch Routing Diagram

```
  ┌──────────────────────────────────────────────────────────────┐
  │                   Instruction Input                          │
  └──────────────────────────┬───────────────────────────────────┘
                             │
                             ▼
              ┌──────────────────────────────┐
              │  Complexity Check            │
              │  is_complex_task(prompt)     │
              └──────┬───────────────┬───────┘
                     │               │
            simple   │               │  complex
                     ▼               ▼
           ┌─────────────┐  ┌────────────────────────┐
           │ Direct      │  │ TaskDecomposerDispatcher│
           │ Execution   │  │                         │
           └─────────────┘  └───────────┬────────────┘
                                        │
                    ┌────────────────────┤
                    │                    │
                    ▼                    ▼
         ┌──────────────────┐  ┌──────────────────┐
         │  CAPABILITY      │  │  LLM Subagent    │
         │  ROUTE TABLE     │  │  Dispatch        │
         │                  │  │                  │
         │  keyword → route │  │  Provider → LLM  │
         │  ┌────────────┐  │  │  Coder/Planner/  │
         │  │ "excel"    │──│──│  Reviewer        │
         │  │  →NT-ACT   │  │  │                  │
         │  │ "审查"     │──│──│  External:       │
         │  │  →NT-SHIELD│  │  │  knowledge       │
         │  │ "检索"     │──│──│  acquisition +   │
         │  │  →NT-MEMORY│  │  │  trial-error     │
         │  │ "诊断"     │──│──│                  │
         │  │  →NT-REPAIR│  │  └────────┬─────────┘
         │  │ "架构"     │──│──│         │
         │  │  →NT-CORE  │  │         │
         │  │ "爬虫"     │──│──│         │
         │  │  →NT-WORLD │  │         │
         │  └────────────┘  │         │
         └────────┬─────────┘         │
                  │                   │
                  ▼                   ▼
         ┌─────────────────────────────────────┐
         │         Domain Executor             │
         │                                     │
         │  NT-ACT    ─ tools/actions          │
         │  NT-WORLD  ─ crawl/OSINT            │
         │  NT-MEMORY ─ KB retrieval           │
         │  NT-MIND   ─ skill crystallize      │
         │  NT-CORE   ─ architecture/planning  │
         │  NT-SHIELD ─ security audit         │
         │  NT-META   ─ meta-cognition         │
         │  NT-REPAIR ─ root cause / fix       │
         └──────────────────┬──────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │  Result Aggregation      │
              │  + Reflection + Absorb   │
              │  (experience-tree)       │
              └──────────────────────────┘
```
