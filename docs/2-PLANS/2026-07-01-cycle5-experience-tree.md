
# 经验树 — 2026-07-01 Cycle 4→5 Blind Spot Synthesis

> **Cycle 4 (2026-06-30 → 2026-07-01)** delivered 31/31 blind spot fixes (8 P0 + 12 P1 + 11 P2), compiled clean (`cargo check --lib` ✅, `cargo clippy -p neotrix-types` ✅). GatewayV2 integration, aggressive fallback strategy, proxy pool L7 probing, and PRM/SAE/GRPO/WTA/5-layer compression/E8→VSA/ProceduralMemory/PER separation all landed.
>
> **Cycle 5** targets the next wave: deeper alignment (GRPO + PRM-Lite), agentic evolution loops, cognitive forgetting, WASM sandboxing, formal verification, runtime tool forging, and hardware attestation. This document synthesizes research across 10 domains, maps 7 new blind spots, and proposes a prioritized roadmap.

---

## 研究来源

Ten domains were explored via GitHub topic search (stars as of 2026-06-30). Key repositories discovered per domain:

### Domain 1: GRPO + Process Reward Models (PRM) + Long-Horizon RL

| Repository | Stars | Relevance |
|---|---|---|
| `agentic-grpo-longhorizon` | 85★ | Core: GRPO for agentic trajectories with LATA normalization, τ-bench +37% |
| `PRM800K` | — | OpenAI PRM dataset, 800K step-level labels |
| `math-shepherd` | 1.2k★ | Process reward model for math reasoning, MCTS-based PRM data |
| `openrlhf` | 14k★ | RLHF framework with GRPO, PPO, PRM support |
| `verl` | 3.5k★ | Volcano Engine RL framework, hybrid engine PRM |
| `grokfast` | 2.1k★ | Gradient-accelerated PRM training, 8× speedup |
| `easyr1` | 4.2k★ | GRPO-centric RLHF for VLMs, including R1-Zero recipes |

### Domain 2: Agentic Self-Evolution (A-Evolve / Reflexio)

| Repository | Stars | Relevance |
|---|---|---|
| `A-Evolve` | 664★ | Solve→Observe→Evolve→Gate→Reload, benchmark-gated self-evolution |
| `Reflexio` | 315★ | Interaction-driven profile/playbook injection (no code changes) |
| `Self-Refine` | 6.8k★ | Iterative self-improvement via feedback loops |
| `AgentEvolve` | 180★ | Multi-agent evolutionary framework with selection/crossover |
| `ADAS` | 45★ | Automatic Domain Adaptation System, environment-aware skill transfer |

### Domain 3: Cognitive Memory Architectures (Forgetting + Compression)

| Repository | Stars | Relevance |
|---|---|---|
| `engram` | 15★ | Rust+MCP, 8192D holographic memory, NVMe-backed cold store |
| `ECHOFORM` | — | FHRR D=8192, cryptographic forgetting certificate (Ed25519 JWS), zero-knowlege forgetting proofs |
| `memoir` | 890★ | Write-it-down → Recall → Forgetting curve, SQLite-backed |
| `mem0` | 26k★ | Memory layer for LLMs, forgetting mechanism based on recency/importance |
| `hippocampus` | 320★ | Biologically-plausible hippocampal memory with pattern separation |
| `cognitive-forgetting` | 12★ | Implementation of human-like forgetting curves (Ebbinghaus + Bayesian) |

### Domain 4: WASM Sandboxing

| Repository | Stars | Relevance |
|---|---|---|
| `fastmcp` | 25.9k★ | MCP server with WASM-based sandbox plugins |
| `extism` | 5.6k★ | Universal plugin system, WASM-based, polyglot |
| `wasmtime` | 16k★ | Bytecode Alliance runtime, Cranelift JIT, fine-grained sandbox |
| `wasm3` | 7.2k★ | Ultra-fast WASM interpreter, 1µs startup |
| `lunatic` | 4.8k★ | Erlang-style WASM runtime with process isolation |
| `wazero` | 5.4k★ | Zero-dependency WASM runtime, no CGO, instant start |
| `containers-than-virtual-machines` | — | WASM vs container performance benchmarks (1ms vs 500ms startup) |

### Domain 5: Formal Verification for Rust (Creusot)

| Repository | Stars | Relevance |
|---|---|---|
| `creusot` | 1.8k★ | MIR→Why3→SMT, verified Rust, contracts-as-attributes |
| `prusti` | 1.2k★ | Viper-based Rust verification, SIL intermediate language |
| `verus` | 1.8k★ | Verified Rust using Z3, linear types support |
| `kani` | 2.3k★ | AWS Rust model checker, CBMC-based, bounded verification |
| `hax` | 470★ | Formal verification for Rust via F*, crytographic protocol focus |
| `aeneas` | 150★ | Coq extraction from Rust, CCPROOF compatible |
| `why3` | 700★ | Deductive verification platform (Creusot backend) |

### Domain 6: Runtime Tool Forging (AgentOS Pattern)

| Repository | Stars | Relevance |
|---|---|---|
| `AgentOS` | 340★ | Agent operating system with dynamic tool loading |
| `open-interpreter` | 58k★ | Natural language → generated → executed code |
| `gpt-engineer` | 53k★ | Code generation agent with self-repair |
| `tool-anything` | 180★ | Dynamic API tool creation from OpenAPI specs |
| `functionless` | 65★ | Serverless function generation at runtime |
| `task-weaver` | 5.6k★ | Code-first agent framework, runtime function definition |
| `auto-tool-creator` | 42★ | LLM creates tools → verifies → registers |

### Domain 7: Hardware Attestation + TEE

| Repository | Stars | Relevance |
|---|---|---|
| `nitro-enclaves` | 2.4k★ | AWS Nitro TEE, cryptographic attestation |
| `sev-snp` | 1.5k★ | AMD SEV-SNP, hardware memory encryption |
| `tdx` | 890★ | Intel Trust Domain Extensions, confidential computing |
| `gramine` | 4.2k★ | LibOS for TEEs, runs unmodified apps in SGX/TDX |
| `enarx` | 1.3k★ | TEE-agnostic runtime, WASM on SEV/SGX/TDX |
| `veracruz` | 210★ | TEE-based multi-party computation framework |
| `oak` | 4.1k★ | Google's confidential computing platform, attestation SDK |

### Domain 8: Constitutional AI + RL Alignment

| Repository | Stars | Relevance |
|---|---|---|
| `constitutional-ai` | 890★ | Anthropic CAI, SL-CAI + RL-CAI, self-critique training |
| `DPO` | 7.2k★ | Direct Preference Optimization, no reward model needed |
| `KTO` | 420★ | Kahneman-Tversky Optimization, preference from binary feedback |
| `GRPO` | 3.1k★ | Group Relative Policy Optimization (DeepSeek-R1) |
| `ORPO` | 580★ | Odds Ratio Preference Optimization, monolithic alignment |
| `safety-tuned-llama` | 340★ | Safety alignment via constrained RL |
| `beam-search-rl` | 180★ | Beam search + RL for multi-step reasoning paths |

### Domain 9: Neurosymbolic + VSA Advances

| Repository | Stars | Relevance |
|---|---|---|
| `vsa-for-ml` | 230★ | VSA for machine learning, FHRR/BSC/HRR implementations |
| `hd-computing` | 870★ | Hyperdimensional computing library, MAP/BSC/RandVec |
| `torch-hd` | 120★ | PyTorch HD computing, differentiable bundling/binding |
| `neurosymbolic-vsa` | 95★ | Neurosymbolic reasoning with VSA graph structures |
| `resonator-networks` | 65★ | Resonator network for FHRR factorization |
| `sdr-vsa` | 30★ | Sparse Distributed Representations with VSA |

### Domain 10: MCP Ecosystem + Protocol Evolution

| Repository | Stars | Relevance |
|---|---|---|
| `fastmcp` | 25.9k★ | MCP server framework with WASM, streaming, auth |
| `mcp-servers` | 40k★ | Reference MCP server implementations by Anthropic |
| `mcp-client` | 2.1k★ | MCP client library with SSE/stdio/WebSocket support |
| `mcp-proxy` | 890★ | MCP proxy with OAuth 2.1, rate limiting, caching |
| `mcp-spec` | 12k★ | MCP specification v2025-03-26 |
| `mcp-k8s` | 45★ | MCP server for Kubernetes cluster management |
| `mcp-oauth` | 120★ | OAuth 2.1 authorization for MCP servers |

---

## 7大盲点详细分析

### 1. GRPO + PRM-Lite + LATA (agentic-grpo-longhorizon)

#### Problem

SEAL's reward pipeline in Cycle 4 has a `PrmHead` and `PrmObserver` that score intermediate reasoning steps, but there is **no GRPO-style per-token advantage computation** integrated into the policy update loop. The `E8Policy` still uses `epsilon-greedy` for mode selection, not learned advantages.

Additionally, vanilla GRPO collapses on long-horizon agentic trajectories (τ > 50 steps) due to vanishing advantage signals. The `agentic-grpo-longhorizon` (85★) paper demonstrates that GRPO without length normalization degrades by 37% on τ-bench long-horizon tasks.

#### Source

- `agentic-grpo-longhorizon`: GRPO for agentic trajectories; introduces **LATA (Length-Adjusted Token Advantage)** normalization: `A_lata = A_raw / √(τ)` where τ is trajectory length. Achieved +37% on τ-bench.
- `PRM800K` / `math-shepherd`: Established that PRM step-level scoring significantly improves over outcome-only rewards for multi-step reasoning.
- `verl` / `openrlhf`: Production GRPO implementations with hybrid engine support (actor + reference + reward + PRM).

#### Fix Applied in Cycle 4

- `PrmHead` scores each E8 transition step (u8 → f64 logit).
- `PrmObserver` aggregates per-step PRM scores into trajectory-level metadata.
- LATA `(1/√L)` normalization applied to PRM scores in `process_trajectory()`.

#### Remaining Gaps

| # | Gap | Severity | Fix |
|---|---|---|---|
| 1.1 | No `learn_from_scores()` integration — PRM scores are observed but never consumed by `E8Policy` for advantage estimation | **High** | Add `E8Policy.learn_from_advantages(advantages: &[f64])` that updates E8 transition matrix via gradient approximation |
| 1.2 | No beam search over trajectory steps during PRM scoring — only greedy mode selection | **Medium** | Add `PrmBeamSearch` that maintains top-K partial trajectories, expands via E8, prunes via PRM threshold |
| 1.3 | No group sampling — GRPO requires G=4..8 trajectories per prompt for advantage baseline | **High** | Add `E8GroupSampler` that runs E8 with different temperature seeds, collects G trajectories, computes group-normalized advantages |
| 1.4 | No KL penalty against reference policy — GRPO clips policy ratio to prevent collapse | **Medium** | Add reference E8 transition matrix snapshot, compute KL divergence per step |
| 1.5 | LATA applied to PRM scores but not validated against held-out τ-bench tasks | **Low** | Add `τ_bench_task` test harness with len-8, len-32, len-128 variants |

#### Proposed Integration

```
E8Engine
  ├── run_trajectory(task, temperature) → Trajectory { steps: Vec<E8State>, scores: Vec<f64> }
  ├── E8GroupSampler
  │     ├── sample_group(task, G=4) → Vec<Trajectory>
  │     └── compute_group_advantages(group) → Vec<f64>  // group-normalized + LATA
  ├── PrmHead
  │     └── score_step(state) → f64
  ├── PrmBeamSearch
  │     ├── expand(state, beam_width=5) → Vec<E8State>
  │     └── prune(states, threshold=0.3) → Vec<E8State>
  └── E8Policy
        ├── learn_from_advantages(advantages)  // update transition matrix
        ├── kl_divergence(reference) → f64
        └── update_with_clipping(ratio, kl_threshold=0.1)
```

**Files affected**: `nt_core_e8.rs`, `nt_core_policy.rs`, `nt_core_prm.rs`, `nt_core_observer.rs`

---

### 2. Agentic Evolution Loop (A-Evolve / Reflexio)

#### Problem

SEAL is a 27-stage pipeline with strong self-editing capabilities, but it has two structural weaknesses compared to state-of-the-art agentic evolution:

1. **No benchmark gate before applying edits**: SEAL's ValidationGate runs `cargo check` (syntactic correctness) but does not benchmark the *current* system on a task suite before deciding whether an edit is beneficial. A-Evolve (664★) runs Solve→Observe→Evolve→Gate→Reload where the Gate requires quantifiable improvement on a held-out benchmark.

2. **No profile/playbook injection**: Reflexio (315★) injects interaction-derived profiles and playbooks into the agent's context *without modifying source code*. This is complementary to SEAL's code-modification approach — some behaviors are better shaped via context than code.

#### Source

- `A-Evolve`: 5-stage loop with benchmark gating. "Solve" generates candidate edits, "Observe" collects execution traces, "Evolve" produces modifications, "Gate" runs benchmark comparison (pre-edit vs post-edit), "Reload" loads champion. 664★, PyTorch-based.
- `Reflexio`: Interaction-driven profile/playbook injection. Maintains a dynamic profile (user preferences, environment constraints) and playbook library (reusable solution patterns). Injected into system prompt as compressed summaries. 315★, no code modification.
- `Self-Refine` (6.8k★): Iterative feedback→refine loop, established that iterative self-improvement converges in 3-5 rounds.
- `ADAS` (45★): Environment-aware domain adaptation, context-specific skill selection.

#### Gap vs SEAL

| Dimension | SEAL (Current) | A-Evolve | Reflexio | Gap |
|---|---|---|---|---|
| Edit approach | Source code modification | Source code modification | Context injection | Both are complementary |
| Pre-edit validation | `cargo check` (syntax) | Benchmark suite (performance) | N/A (no code changes) | SEAL lacks benchmark gate |
| Post-edit validation | `cargo test` | Benchmark comparison | None | SEAL runs tests but no delta comparison |
| Profile injection | Static config.toml | N/A | Dynamic interaction-derived | SEAL has no equivalent |
| Skill library | `nt_mind_skill` (compiled) | N/A | Playbook library (context) | SEAL skills are code-only |
| Rollback criteria | Test failure | Benchmark regression | N/A | SEAL has coarser rollback |

#### Integration

**Phase 1 — BenchmarkGate (P0)**

Add a `BenchmarkGateStage` to SEAL (insert after `ValidationGate`, before `GwtAbsorb`):

```
ValidationGate (cargo check) → BenchmarkGate (benchmark delta) → GwtAbsorb
```

`BenchmarkGate` maintains a suite of representative tasks (~5-10), runs them pre- and post-edit, computes win rate. If win rate drops >5%, rollback.

```
pub struct BenchmarkGateStage {
    suite: Vec<BenchmarkTask>,       // task + expected output pattern
    threshold: f64,                  // min acceptable improvement (default: -0.05)
    pre_scores: HashMap<String, f64>, // cached from last benchmark run
}

impl BenchmarkGateStage {
    fn run_suite(engine: &mut ReasoningEngine) → HashMap<String, f64>;
    fn compute_delta(pre: &HashMap<String, f64>, post: &HashMap<String, f64>) → f64;
    fn gate(engine: &mut ReasoningEngine) → GateDecision { Accept | Rollback | Retry }
}
```

**Phase 2 — Profile Injection (P1)**

Add a `ProfileInjectionStage` to SEAL (insert after `ConversationDistillStage`):

```
ConversationDistillStage → ProfileInjectionStage → AgingDiagnosis
```

`ProfileInjectionStage` extracts user interaction patterns, environment constraints, and recurring preferences from `ConversationRecord`s, compresses into a ~500-token profile, and stores in GWT workspace for downstream stages.

```
pub struct ProfileInjectionStage {
    max_profile_tokens: usize,       // default: 500
    profile_cache: LruCache<String, Profile>,
}

pub struct Profile {
    user_preferences: Vec<String>,    // e.g., ["prefers concise output", "dislikes panics"]
    environment: Vec<String>,          // e.g., ["macOS arm64", "limited GPU memory"]
    recurring_patterns: Vec<String>,   // e.g., ["user corrects import order", "forgets error handling"]
}
```

**Phase 3 — Playbook Library (P2)**

Add a field to `nt_memory_kb` for playbook storage (new node type `Playbook` with relation `APPLIES_TO`), and a `PlaybookRetrievalStage` that fetches relevant playbooks before `SelfEditGenStage`.

```
NodeType::Playbook = 23,
Relation::APPLIES_TO = 20,

Playbook {
    id: Uuid,
    pattern: String,         // problem signature
    solution: String,        // reusable approach
    source: String,          // Reflexio / manual / SEAL-derived
    success_rate: f64,       // tracked over time
}
```

**Files affected**: `nt_mind_seal/benchmark_gate.rs` (NEW), `nt_mind_seal/profile_injection.rs` (NEW), `nt_memory_types.rs`, `nt_memory_store.rs`, `nt_mind_seal/self_edit_gen.rs`

---

### 3. Cognitive Memory with Forgetting (Engram / ECHOFORM)

#### Problem

NeoTrix's HyperCube VSA uses FHRR D=2048, which is the minimum viable dimension for FHRR binding. State-of-the-art cognitive memory systems use:

- **engram** (15★): Rust + MCP, 8192D holographic memory, NVMe-backed cold store, LRU eviction with frequency weighting.
- **ECHOFORM**: FHRR D=8192 with cryptographic forgetting certificates (Ed25519 JWS) that prove a memory was intentionally forgotten (not lost). Enables verifiable right-to-be-forgotten compliance.

Additionally, NeoTrix has **no forgetting mechanism** — HyperCube entries accumulate indefinitely. The `HyperCubeOptimizeStage` (freq=10) prunes low-access entries, but this is access-count-based, not cognition-inspired (recency × importance × interference).

#### Source

- `engram` (15★): Holonomic memory model. `D=8192` via FHRR. Cold tier backed by NVMe SSD (memory-mapped files). Warm tier in RAM. Cache coherence via MCP notifications.
- `ECHOFORM`: Forgetting certificates as Ed25519 JWS payloads. Each certificate contains: hash of forgotten data, timestamp, signature. Enables audit trails for data deletion requests (GDPR/CCPA).
- `memoir` (890★): Ebbinghaus forgetting curve implementation: `retention = exp(-t / S)` where S is stability. Spaced repetition-based recall optimization.
- `cognitive-forgetting` (12★): Bayesian forgetting with uncertainty estimation. `P(recall) = Bernoulli(σ(α - β·log(t + 1)))`.

#### Gap vs HyperCube

| Dimension | NeoTrix (Current) | Engram | ECHOFORM | Gap |
|---|---|---|---|---|
| FHRR dimension | 2048 | 8192 | 8192 | +4× capacity needed |
| Cold storage | None | NVMe-backed, memory-mapped | None specified | Need cold tier |
| Forgetting policy | Access-count pruning | LRU + frequency weighting | Cryptographic forgetting | No recency/importance model |
| Forgetting proofs | None | None | Ed25519 JWS certificates | No compliance trail |
| Recall curve | Flat (all entries equally recallable) | Recency-weighted | Importance-weighted | No Ebbinghaus curve |
| Interference model | None | Holographic (inherent) | Holographic (inherent) | VSA binding gives interference for free |

#### Fix

**Step 1 — FHRR D=8192 Upgrade**

Double dimension from 2048 to 8192. This requires updating phasor storage from `[f64; 2048]` to `[f64; 8192]`. Benchmark impact: ~4× memory, ~2× compute (SIMD benefits from wider registers).

```
// Current (D=2048)
const FHRR_DIMENSION: usize = 2048;

// Target (D=8192)
const FHRR_DIMENSION: usize = 8192;
```

**Step 2 — Cold Storage Tier**

Add `ColdStorageBackend` trait with `NvmeBackend` (memory-mapped files on SSD) and `SqliteBackend` (SQLite blob storage for portability). LRU eviction from warm → cold at `HyperCubeOptimizeStage`.

```
pub enum StorageTier {
    Warm(VsaVector),           // in-memory FHRR D=8192
    Cold(ColdStorageHandle),   // NVMe/SQLite reference
}

pub trait ColdStorageBackend {
    fn store(&mut self, key: &[u8], data: &[u8]) → Result<()>;
    fn load(&mut self, key: &[u8]) → Result<Option<Vec<u8>>>;
    fn delete(&mut self, key: &[u8]) → Result<()>;
    fn evict(&mut self, strategy: EvictionStrategy) → Result<Vec<Vec<u8>>>;
}
```

**Step 3 — Forgetting Certificate**

Add `ForgettingCertificate` struct and integrate with `nt_shield_vault` for key management:

```
pub struct ForgettingCertificate {
    pub data_hash: [u8; 32],        // SHA-256 of forgotten data
    pub timestamp: i64,              // UNIX epoch
    pub signature: Vec<u8>,          // Ed25519 JWS
    pub key_id: String,              // signing key identifier
    pub reason: ForgettingReason,    // UserRequest | TTLExpired | SpacePressure | LegalCompliance
}
```

**Step 4 — Ebbinghaus Recall Curve**

Modify `HyperCube::recall(key)` to compute recall probability:

```
fn recall_probability(&self, key: &VsaVector) → f64 {
    let age = self.current_time() - self.last_access(key);
    let stability = self.stability(key);  // based on access frequency + importance
    (-age as f64 / stability).exp()       // Ebbinghaus: R = exp(-t/S)
}
```

Only entries with `recall_probability > threshold` (default: 0.05) are returned from warm storage. Others are demoted to cold.

**Files affected**: `nt_core_hcube.rs`, `nt_memory_cortex.rs`, `nt_mind_seal/hypercube_optimize.rs`, `nt_shield_vault.rs`, `Cargo.toml` (add `ed25519-dalek`, `sha2`)

---

### 4. WASM Sandbox vs Container Sandbox

#### Problem

NeoTrix's current sandbox uses Docker containers (`docker run --rm --network none --memory 512m --cpus 1`). This provides strong isolation but has three critical drawbacks:

1. **Startup latency**: Docker `create + start + attach` takes 300-800ms per invocation. For the SEAL pipeline (which calls code execution 10-50× per cycle), this adds 3-40 seconds of overhead.
2. **Resource overhead**: Each container allocates a full OS/userspace — ~50-100MB RSS even for a trivial Python script.
3. **Portability**: Requires Docker daemon, which is unavailable in Tauri webview contexts, CI runners without Docker, and edge/mobile deployments.

State-of-the-art: WASM in-process sandboxing achieves ~1ms startup, ~1-5MB overhead, and runs anywhere with a runtime.

#### Source

- `fastmcp` (25.9k★): WASM-based plugin system for MCP servers. Sub-millisecond code execution. Polyglot (Python, JS, Rust compiled to WASM).
- `extism` (5.6k★): Universal plugin system, WASM host SDK for 15+ languages. Portable plugins (same `.wasm` runs in any host).
- `wasmtime` (16k★): Bytecode Alliance runtime. Cranelift JIT for near-native speed. Fine-grained sandbox with WASI preview 2.
- `lunatic` (4.8k★): Erlang-style process isolation on WASM. Each process has isolated heap, GC, and scheduler.
- `containers-than-virtual-machines`: Benchmarks showing WASM startup at 1-5ms vs containers at 300-800ms.

#### Integration

**Phase 1 — WasmCodeExecutor (P0)**

Add a new `CodeExecutor` trait implementation using `wasmtime`:

```
pub enum SandboxProvider {
    Docker(DockerProvider),  // existing, for heavy workloads
    Wasm(WasmProvider),      // new, for quick code execution
    Noop(NoopProvider),      // existing fallback
}

pub struct WasmProvider {
    engine: wasmtime::Engine,
    store: wasmtime::Store<WasmContext>,
    linker: wasmtime::Linker<WasmContext>,
    memory_limit: usize,       // default: 64MB
    fuel_per_exec: u64,        // default: 1_000_000 (instruction limit)
}
```

WASI Preview 2 sandbox with:
- `--dir /sandbox` (read-write, tmpfs)
- `--env NO_NETWORK=1` (no networking)
- Fuel metering for CPU limit
- Wall-clock timeout (default: 5s)

**Phase 2 — Automatic Selection (P1)**

Add a `SandboxRouter` that selects provider based on task profile:

```
pub fn select_provider(task: &CodeExecTask) → SandboxProvider {
    match task.runtime {
        Runtime::Python => {
            if task.expected_duration < Duration::from_millis(100) && task.memory_mb < 64 {
                SandboxProvider::Wasm(WasmProvider::for_python())
            } else {
                SandboxProvider::Docker(DockerProvider::python())
            }
        }
        Runtime::NodeJs => {
            if task.expected_duration < Duration::from_millis(50) {
                SandboxProvider::Wasm(WasmProvider::for_javascript())
            } else {
                SandboxProvider::Docker(DockerProvider::node())
            }
        }
        Runtime::Rust | Runtime::Go => SandboxProvider::Docker(...),
        Runtime::Linux => SandboxProvider::Docker(...),
    }
}
```

**Phase 3 — Python-to-WASM Compilation (P2)**

For Python workloads, compile to WASM via `pyodide` or a lightweight Python→WASM toolchain. Alternatively, embed a WASM-compiled Python interpreter (like `python-wasm`).

#### Files affected

`neotrix-core/src/neotrix/sandbox_v2/wasm_provider.rs` (NEW), `neotrix-core/src/neotrix/sandbox_v2/mod.rs`, `neotrix-core/src/neotrix/sandbox_v2/sandbox_router.rs` (NEW), `Cargo.toml` (add `wasmtime`, `wasmtime-wasi`)

---

### 5. Creusot Formal Verification

#### Problem

NeoTrix core has `#![forbid(unsafe_code)]`, which eliminates undefined behavior but does not eliminate logic errors. Critical subsystems (E8 transition logic, VSA binding, PRM scoring) have no formal correctness guarantees.

Creusot (1.8k★) is a mature Rust verification tool that translates MIR → Why3 → SMT solver. It annotates functions with pre/post-conditions and invariants using Rust attributes, then proves them automatically.

#### Source

- `creusot` (1.8k★): `#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]` attributes on Rust functions. Contracts checked by Z3/Alt-Ergo. Verified Rust standard library subset.
- `prusti` (1.2k★): Viper-based verification. Stronger for complex heap structures. Supports permissions and lifetimes.
- `kani` (2.3k★): AWS model checker. Bounded verification (unrolls loops to a fixed depth). Good for finding bugs faster than full verification.
- `verus` (1.8k★): Uses Z3 natively. Best ergonomics for Rust. Google-internal deployment at scale.

#### Integration

**Phase 1 — CreusotProofStage (P0)**

Add a new SEAL stage that runs Creusot verification on critical code paths after edits:

```
pub struct CreusotProofStage {
    targets: Vec<ProofTarget>,    // files/functions to verify
    solver: SolverBackend,        // Z3 | Alt-Ergo | CVC5
    timeout_secs: u64,            // default: 60
    cost_threshold: f64,          // max cost before skipping (CPU-minutes)
}

pub struct ProofTarget {
    file: PathBuf,
    functions: Vec<String>,       // empty = verify all annotated
    cost_estimate: f64,           // estimated CPU-minutes
}
```

**Phase 2 — Cost-Aware Gating (P1)**

Full Creusot verification of the entire codebase would take hours. Use cost-aware gating:

- Only verify functions with `#[proof]` attribute (opt-in)
- Estimate verification cost from AST size + loop count
- Skip if cost > `cost_threshold` (default: 2 CPU-minutes)
- Run in CI, not during interactive development

```
pub fn should_verify(target: &ProofTarget) → bool {
    let available = self.remaining_budget();
    let cost = target.cost_estimate;
    cost < available && cost < self.cost_threshold
}
```

**Phase 3 — Critical Path Identification (P2)**

Auto-identify critical functions via coverage analysis + dependency graph:

- Functions called from both `E8Engine::reason()` and `GWT::broadcast()` are high-priority
- Functions with >3 callers are medium-priority
- Leaf utility functions are low-priority

#### Limitation

Creusot requires `opam` (OCaml package manager) and OCaml toolchain. This is not available in all environments. Strategy:

- **CI-only**: Run Creusot in CI via a Docker image with OCaml + Creusot pre-installed
- **Optional gating**: `CreusotProofStage` is a no-op if `creusot` binary not found (check `which creusot`)
- **Caching**: Proof results cached in `~/.neotrix/proof_cache/`, invalidated on source change

#### Files affected

`neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/creusot_proof_gate.rs` (NEW), `neotrix-core/src/neotrix/safety/verification.rs` (NEW), `Dockerfile.ci` (add OCaml + Creusot), `.github/workflows/ci.yml`

---

### 6. Runtime Tool Forging (AgentOS Pattern)

#### Problem

Every tool in NeoTrix is compiled at build time — MCP servers are discovered (via `*-mcp-server` binaries on PATH) or configured statically. There is **no mechanism for the agent to create, compile, register, and use a tool at runtime**.

This is a fundamental limitation for a self-evolving system. If the agent encounters a novel task (e.g., "compute the fractal dimension of this terrain image"), it cannot create a new tool — it must either prompt-engineer its way through or fail.

#### Source

- `AgentOS` (340★): Dynamic tool loading. Agents write tool code → receive tool ID → invoke by ID. Code stored in SQLite-backed registry.
- `tool-anything` (180★): OpenAPI specs → dynamic tool creation. Agents describe APIs, tools are generated at runtime.
- `auto-tool-creator` (42★): LLM generates tool code → sandbox validation → registration. 78% first-attempt success rate.
- `open-interpreter` (58k★): Code generation + execution in one step. No persistent tools, but demonstrates the feasibility of runtime code creation.

#### Gap

| Dimension | NeoTrix (Current) | AgentOS | Gap |
|---|---|---|---|
| Tool creation | Compile-time | Runtime | No `mcp create` flow |
| Tool code storage | Filesystem | SQLite | No structured registry for user-created tools |
| Tool verification | Cargo check | Sandbox execution + json-schema | Need both syntax + behavioral verification |
| Tool discovery | PATH scanning | Registry query | Need merged discovery (built-in + user-created) |
| Tool lifecycle | Static | Create→verify→publish→deprecate | No lifecycle management |

#### Integration

**Phase 1 — `mcp publish` Accept Code (P0)**

Extend the existing `McpRegistry::publish()` to accept Python source code (not just command paths):

```
// Current (Cycle 4)
McpRegistry::publish(name: &str, command: &str, args: &[String], description: &str)

// Target (Cycle 5)
McpRegistry::publish(PublishRequest {
    name: String,
    source: ToolSource,           // CommandPath | InlineCode { code: String, runtime: Runtime }
    description: String,
    input_schema: Option<Value>,  // JSON Schema for tool inputs
})
```

When `ToolSource::InlineCode` is provided:
1. Write code to `~/.neotrix/tools/{name}/main.py`
2. Create a wrapper MCP server script (`~/.neotrix/tools/{name}/server.py`)
3. Register via stdio transport pointing to the wrapper

**Phase 2 — Python Tool Sandbox (P1)**

Wrap runtime-created tools in a WASM sandbox (see Blind Spot #4):

- Python tools run via `wasmtime` + `python-wasm`
- 64MB memory limit, 5s timeout, no network
- `input_schema` validated on every call
- Tool stdout/stderr captured and returned as structured output

**Phase 3 — Verification Gate (P1)**

Before registration, run verification:

1. Syntax check: `python3 -c "import ast; ast.parse(code)"`
2. Schema check: Invoke tool with `{"_test": true}`, expect `{"status": "ready"}`
3. Security scan: Scan code for banned patterns (`import os`, `subprocess`, `eval`)
4. Benchmark: Run tool on sample inputs, record latency

**Phase 4 — MCP Registry Merged Discovery (P2)**

Merge built-in tools (from PATH), user-created tools (from SQLite), and published servers into a unified `ToolRegistry`:

```
pub struct UnifiedToolRegistry {
    builtin: McpRegistry,       // PATH-discovered + built-in
    user_created: UserToolStore, // SQLite-backed
    published: McpRegistry,     // user-published external servers
}
```

`find_tool(name)` queries all three sources, merges results with priority: user_created > published > builtin (user tools can shadow built-ins).

#### Files affected

`neotrix-core/src/agent/tool/mcp/registry.rs` (modify `publish()`), `neotrix-core/src/agent/tool/mcp/user_tool_store.rs` (NEW), `neotrix-core/src/neotrix/sandbox_v2/wasm_provider.rs`, `neotrix-core/src/cli/mcp.rs`, `neotrix-core/src/neotrix/mcp_discovery.rs`

---

### 7. Hardware-backed Attestation + TEE

#### Problem

NeoTrix has no support for Trusted Execution Environments (TEEs). This is a critical gap for:

- **Remote attestation**: Proving to a third party that a specific computation ran without tampering
- **Private data processing**: Running user code on sensitive data without exposing it to the host OS
- **Secure multi-party computation**: Multiple untrusted parties contribute data, computation happens in TEE
- **Compliance**: GDPR/CCPA/HIPAA requirements for data processing audit trails

#### Source

- `nitro-enclaves` (2.4k★): AWS Nitro TEE. CPU + memory isolated from parent EC2 instance. Cryptographic attestation via `nitro-cli`. Supports KMS integration for key derivation.
- `sev-snp` (1.5k★): AMD Secure Encrypted Virtualization with Secure Nested Paging. Memory encryption with hardware-enforced isolation. Page-level integrity protection.
- `tdx` (890★): Intel Trust Domain Extensions. Hardware-enforced confidential VMs. No hypervisor or host OS can access TD memory.
- `gramine` (4.2k★): LibOS for running unmodified applications in TEEs. Supports SGX, TDX, SEV. Highly relevant — allows existing NeoTrix binaries to run in TEE with minimal changes.
- `enarx` (1.3k★): TEE-agnostic runtime. Same application runs on SGX, SEV, TDX without modification. WASM-based workload delivery. Strong alignment with NeoTrix's WASM direction.
- `oak` (4.1k★): Google's confidential computing platform. Attestation SDK, key provisioning, remote execution framework.

#### Integration Strategy

**Phase 1 — Attestation Framework (P0)**

Define the attestation primitives without requiring actual TEE hardware (which is unavailable on macOS development machines):

```
pub trait AttestationProvider {
    fn platform(&self) → TeePlatform;           // Nitro | SEV_SNP | TDX | Simulated
    fn attest(&self, payload: &[u8]) → Result<AttestationReport>;
    fn verify(&self, report: &AttestationReport, expected_payload: &[u8]) → Result<bool>;
    fn get_measurement(&self) → Result<Vec<u8>>;  // runtime measurement hash
}

pub struct AttestationReport {
    pub platform: TeePlatform,
    pub payload_hash: [u8; 32],
    pub measurement: Vec<u8>,
    pub signature: Vec<u8>,
    pub certificate_chain: Vec<Vec<u8>>,
    pub timestamp: i64,
}

// Simulated provider for development
pub struct SimulatedAttestation;

impl AttestationProvider for SimulatedAttestation {
    fn attest(&self, payload: &[u8]) → Result<AttestationReport> {
        let hash = sha256(payload);
        let key = load_dev_key();
        let sig = key.sign(&hash);
        Ok(AttestationReport {
            platform: TeePlatform::Simulated,
            payload_hash: hash,
            measurement: vec![],
            signature: sig.to_bytes().to_vec(),
            certificate_chain: vec![key.public().to_bytes().to_vec()],
            timestamp: unix_now(),
        })
    }
}
```

**Phase 2 — TEE Sandbox (P1)**

Add a `TeeSandboxProvider` that runs code inside a TEE. During development (no TEE hardware), this falls back to simulated attestation:

```
pub enum SandboxProvider {
    Docker(DockerProvider),
    Wasm(WasmProvider),
    Tee(TeeSandboxProvider),  // NEW
    Noop(NoopProvider),
}

pub struct TeeSandboxProvider {
    attestation: Box<dyn AttestationProvider>,
    key_broker: KeyBroker,        // derives keys from attestation
    runtime: TeeRuntime,          // Gramine | Enarx | Custom
}
```

**Phase 3 — Confidential Sandbox Code Execution (P2)**

When executing user code that contains sensitive data (API keys, personal data):

1. Provision a TEE sandbox via `nitro-cli` / `gramine-sgx`
2. Transfer code + data via encrypted channel
3. TEE performs computation, generates attestation report
4. Result + attestation returned to caller
5. TEE destroyed

**Phase 4 — Key Derivation from Attestation (P2)**

Use TEE measurements to derive unique keys (TEE-bound keys that cannot be extracted):

```
pub struct KeyBroker {
    provider: Box<dyn AttestationProvider>,
    kms: Option<KmsClient>,  // AWS KMS integration
}

impl KeyBroker {
    fn derive_key(&self, context: &[u8]) → Result<SymmetricKey> {
        let measurement = self.provider.get_measurement()?;
        let salt = sha256(context);
        // HKDF-extract(measurement, salt) → key bound to this TEE
        Ok(hkdf_extract(&measurement, &salt))
    }
}
```

#### Limitations for Development

| Constraint | Mitigation |
|---|---|
| No TEE hardware on macOS | `SimulatedAttestation` provider for dev; real TEE on AWS/GCP |
| Nitro Enclaves requires EC2 | CI runs in `nitro-enclave`-capable instances |
| SEV-SNP requires AMD EPYC | Document `c6a` / `m6a` instance types for CI |
| TDX requires Intel Xeon | Document `c7i` / `m7i` instance types for CI |
| Gramine/Enarx complex setup | Docker image with pre-installed Gramine, mounted code |

#### Files affected

`neotrix-core/src/neotrix/sandbox_v2/tee_provider.rs` (NEW), `neotrix-core/src/neotrix/sandbox_v2/attestation.rs` (NEW), `neotrix-core/src/neotrix/sandbox_v2/key_broker.rs` (NEW), `neotrix-core/src/neotrix/sandbox_v2/mod.rs`, `Cargo.toml` (add `ed25519-dalek`, `sha2`, `hkdf`)

---

## Priority Matrix

| # | Blind Spot | Impact | Effort | Dependencies | Risk | Strategic Value |
|---|---|---|---|---|---|---|
| 1 | GRPO + PRM-Lite + LATA | **High** | **High** | E8Policy, PrmHead (existing) | Medium — LATA already implemented; GRPO integration is additive | **Foundation for learned policy** — unblocks all downstream RL |
| 2a | BenchmarkGate (A-Evolve) | **High** | Medium | ValidationGate (existing), benchmark suite (NEW) | Low — isolated new stage | **Quantifiable evolution** — moves SEAL from check→measure |
| 2b | Profile Injection (Reflexio) | Medium | Medium | ConversationDistillStage (existing) | Low — context-only, no code changes | **Context-aware behavior** without risking stability |
| 3 | Cognitive Forgetting (Engram/ECHOFORM) | **High** | **High** | HyperCube (existing), FHRR D=8192 upgrade | Medium — D=8192 may break cosine similarity thresholds | **Memory scalability** — prevents unbounded growth |
| 4 | WASM Sandbox | Medium | **High** | SandboxProvider (existing), wasmtime crate | Medium — WASM compilation for Python is complex | **10-100× faster sandbox** — critical for SEAL throughput |
| 5 | Creusot Formal Verification | Medium | **High** | opam/OCaml toolchain (CI only) | Low — optional stage, skipped if unavailable | **Correctness proofs** — highest quality bar |
| 6 | Runtime Tool Forging | **High** | Medium | UserToolStore (NEW), WASM sandbox (#4) | Medium — security surface increases | **Self-evolving tool ecosystem** — core differentiator |
| 7 | TEE + Attestation | Low | **High** | Gramine/Enarx, cloud TEE instances | Low — simulated provider works offline | **Enterprise compliance** — unlocks regulated use cases |

### Impact × Effort Quadrant

```
                    High Impact
                        │
          ┌─────────────┼─────────────┐
          │   #1 GRPO   │   #2a Gate  │
          │   #3 Forget │   #6 Forge  │
          │             │             │
    High  ├─────────────┼─────────────┤  Low
    Effort│   #4 WASM   │   #2b Prof  │  Effort
          │   #5 Creusot│   #7 TEE    │
          │             │             │
          └─────────────┼─────────────┘
                        │
                    Low Impact
```

**Top-right quadrant (High Impact, Low Effort)**: #2a (BenchmarkGate), #6 (Runtime Tool Forging) — highest ROI for Cycle 5.

**Top-left quadrant (High Impact, High Effort)**: #1 (GRPO), #3 (Forgetting) — must schedule early as they have long lead times.

---

## Cycle 5 Suggested Roadmap

### Milestone 1: GRPO-PRM Integration (Week 1-2)
**Dependencies**: `PrmHead`, `PrmObserver`, `E8Policy` all exist from Cycle 4
**Deliverable**: `cargo test -p neotrix --lib -- grpo` passes

| Day | Task | Files |
|---|---|---|
| 1-2 | Implement `E8GroupSampler` — run G=4 trajectories with different temperatures | `nt_core_e8.rs` |
| 3-4 | Implement `group_normalize_advantages()` — subtract mean, divide by std, apply LATA | `nt_core_policy.rs` |
| 5-6 | Implement `learn_from_advantages()` — update E8 transition matrix via soft update | `nt_core_policy.rs` |
| 7-8 | Add KL penalty against reference transition matrix | `nt_core_policy.rs` |
| 9-10 | Implement `PrmBeamSearch` — top-K expansion + PRM pruning | `nt_core_prm.rs` |
| 11-12 | Add τ-bench test harness with len-8/32/128 tasks | `tests/grpo_tau_bench.rs` |
| 13-14 | Integration test: group sampling → advantage → policy update → verify E8 mode distribution shift | `tests/grpo_integration.rs` |

**Tests to pass**: `test_group_sampler_returns_g_trajectories`, `test_group_normalized_advantages_sum_to_zero`, `test_prm_beam_search_improves_vs_greedy`, `test_learn_from_advantages_shifts_mode_distribution`, `test_kl_penalty_prevents_mode_collapse`

---

### Milestone 2: BenchmarkGate + Profile Injection (Week 3)
**Dependencies**: Milestone 1 (can run in parallel)
**Deliverable**: `cargo test -p neotrix --lib -- benchmark_gate` + `profile_injection` passes

| Day | Task | Files |
|---|---|---|
| 1-2 | Design `BenchmarkTask` struct, benchmark suite configuration format (TOML) | `nt_mind_seal/benchmark_gate.rs` |
| 3-4 | Implement `BenchmarkGateStage::run_suite()` — invokes engine, scores outputs | `nt_mind_seal/benchmark_gate.rs` |
| 5 | Implement `compute_delta()` + `gate()` with Accept/Rollback/Retry decisions | `nt_mind_seal/benchmark_gate.rs` |
| 6 | Wire into SEAL pipeline (after ValidationGate, before GwtAbsorb) | `nt_mind_seal/mod.rs` |
| 7 | Implement `ProfileInjectionStage` — parse ConversationRecords, compress to profile | `nt_mind_seal/profile_injection.rs` |
| 8 | Store profile in GWT workspace, make available to downstream stages | `nt_core_gwt.rs` |
| 9-10 | Tests: benchmark gate catches regression, profile injection extracts correct patterns | `tests/seal_benchmark_gate.rs`, `tests/profile_injection.rs` |

**Tests to pass**: `test_benchmark_gate_accepts_improvement`, `test_benchmark_gate_rolls_back_regression`, `test_benchmark_gate_retries_on_flaky`, `test_profile_injection_extracts_preferences`, `test_profile_injection_empty_when_no_history`

---

### Milestone 3: FHRR D=8192 + Cognitive Forgetting (Week 4-5)
**Dependencies**: Milestone 1-2 (independent)
**Deliverable**: `cargo test -p neotrix --lib -- hypercube` passes with D=8192

| Day | Task | Files |
|---|---|---|
| 1-2 | Upgrade `FHRR_DIMENSION` from 2048→8192; update all phasor operations to handle wider SIMD | `nt_core_hcube.rs` |
| 3 | Run HyperCube benchmarks; verify cosine similarity thresholds still valid (adjust if needed) | `benches/hypercube_bench.rs` |
| 4-5 | Implement `ColdStorageBackend` trait + `NvmeBackend` (memory-mapped files) | `nt_core_hcube/cold_storage.rs` |
| 6 | Implement `SqliteBackend` for portable cold storage | `nt_core_hcube/cold_storage.rs` |
| 7-8 | Implement Ebbinghaus recall probability in `HyperCube::recall()` | `nt_core_hcube.rs` |
| 9 | Implement LRU→cold eviction in `HyperCubeOptimizeStage` | `nt_mind_seal/hypercube_optimize.rs` |
| 10-11 | Implement `ForgettingCertificate` + Ed25519 signing + verification | `nt_shield_vault/forgetting.rs` |
| 12 | Integrate forgetting certificates into `nt_shield_vault` key management | `nt_shield_vault.rs` |
| 13-14 | Tests: D=8192 accuracy parity, cold store round-trip, forgetting certificate creation/verification | `tests/hypercube_forgetting.rs` |

**Tests to pass**: `test_d8192_similarity_preserved_vs_d2048`, `test_cold_store_store_load_roundtrip`, `test_ebbinghaus_recall_older_entries_have_lower_probability`, `test_forgetting_certificate_sign_and_verify`, `test_forgetting_certificate_tamper_detection`

---

### Milestone 4: WASM Sandbox + Runtime Tool Forging (Week 6-7)
**Dependencies**: Milestone 3 (for cold storage of tools); can start in parallel
**Deliverable**: `cargo test -p neotrix --lib -- wasm_sandbox` + `cargo test -p neotrix --lib -- tool_forging` passes

| Day | Task | Files |
|---|---|---|
| 1-2 | Add `wasmtime` + `wasmtime-wasi` crates; implement `WasmProvider` with WASI sandbox | `sandbox_v2/wasm_provider.rs` |
| 3 | Implement fuel metering + wall-clock timeout in WASM execution | `sandbox_v2/wasm_provider.rs` |
| 4 | Implement `SandboxRouter` for automatic provider selection (WASM vs Docker) | `sandbox_v2/sandbox_router.rs` |
| 5-6 | Extend `McpRegistry::publish()` to accept `ToolSource::InlineCode` | `agent/tool/mcp/registry.rs` |
| 7 | Implement Python code → MCP server wrapper generation | `agent/tool/mcp/user_tool_store.rs` |
| 8 | Implement verification gate (ast.parse → schema check → security scan → benchmark) | `agent/tool/mcp/user_tool_store.rs` |
| 9-10 | Implement `UnifiedToolRegistry` merging builtin + user_created + published | `agent/tool/mcp/registry.rs` |
| 11-12 | WASM sandbox for Python tools (integrate `python-wasm`) | `sandbox_v2/wasm_provider.rs` |
| 13-14 | Tests: tool forge→verify→publish→invoke E2E, WASM sandbox isolation | `tests/wasm_sandbox.rs`, `tests/tool_forging_e2e.rs` |

**Tests to pass**: `test_wasm_sandbox_executes_python`, `test_wasm_sandbox_enforces_fuel_limit`, `test_wasm_sandbox_no_network`, `test_tool_forge_create_verify_publish_e2e`, `test_tool_forge_security_scan_rejects_banned_imports`, `test_unified_registry_user_tools_shadow_builtins`

---

### Milestone 5: CreusotProofStage + TEE Attestation (Week 8-9)
**Dependencies**: None (independent)
**Deliverable**: `cargo test -p neotrix --lib -- creusot` + attestation tests passes

| Day | Task | Files |
|---|---|---|
| 1-2 | Add `#[proof]` attribute documentation; annotate 10 critical E8 functions with pre/post-conditions | `nt_core_e8.rs` |
| 3-4 | Implement `CreusotProofStage` — shell out to `creusot`, parse results, cache proofs | `safety/creusot_stage.rs` |
| 5 | Implement cost-aware gating + CI-only execution | `safety/creusot_stage.rs` |
| 6 | Add Creusot Docker image to CI workflow | `.github/workflows/ci.yml` |
| 7 | Implement `AttestationProvider` trait + `SimulatedAttestation` | `sandbox_v2/attestation.rs` |
| 8 | Implement `TeeSandboxProvider` (simulated for dev, real for CI) | `sandbox_v2/tee_provider.rs` |
| 9 | Implement `KeyBroker` with HKDF derivation from attestation measurement | `sandbox_v2/key_broker.rs` |
| 10-12 | Bundle Gramine Docker image for CI TEE execution | `Dockerfile.gramine` |
| 13-14 | Tests: Creusot verifies annotated functions, attestation sign+verify, key derivation determinism | `tests/creusot_stage.rs`, `tests/attestation.rs` |

**Tests to pass**: `test_creusot_proof_stage_verifies_annotated_functions`, `test_creusot_proof_stage_skips_missing_toolchain`, `test_attestation_simulated_sign_and_verify`, `test_key_broker_derivation_deterministic`, `test_tee_sandbox_executes_code`

---

### Risk Register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| GRPO integration destabilizes E8 policy | Medium | High | Gate behind feature flag `grpo-policy`; rollback via git |
| D=8192 breaks similarity thresholds | Medium | High | Run benchmark comparison pre/post; auto-tune threshold |
| WASM Python compilation is too slow | High | Medium | Fallback to Docker; optimize Python→WASM with lazy compilation |
| Creusot CI image build fails | Medium | Low | Skip stage if Creusot unavailable; test on Ubuntu only |
| TEE attestation is over-engineered for current use cases | Medium | Low | Simulated attestation is sufficient for development; real TEE only in CI |
| Runtime tool forging creates security hole | Medium | High | WASM sandbox + banned-import scanning + user confirmation dialog |

### Success Criteria

| Metric | Current | Milestone 1 | Milestone 2 | Milestone 3 | Milestone 4 | Milestone 5 |
|---|---|---|---|---|---|---|
| E8 policy learning | None | GRPO advantages | — | — | — | — |
| SEAL edit acceptance | `cargo check` | — | +BenchmarkGate | — | — | — |
| Memory capacity | 2048 FHRR | — | — | 8192 FHRR + cold | — | — |
| Sandbox startup | 300-800ms | — | — | — | 1-5ms (WASM) | — |
| Runtime tools | None | — | — | — | Create→verify→publish E2E | — |
| Verified functions | 0% | — | — | — | — | 10 annotated E8 fns |
| Attestation | None | — | — | — | — | Sign+verify E2E |

---

## Technical Debt & Pre-requisites

### Cargo.toml additions (workspace-wide)

```
# Cycle 5 new dependencies
wasmtime = "14"
wasmtime-wasi = "14"
ed25519-dalek = "2"
sha2 = "0.10"
hkdf = "0.12"
```

### CI Infrastructure

- `.github/workflows/ci.yml`: Add `creusot` verification job (Docker image `ghcr.io/neotrix/creusot:latest`)
- `.github/workflows/tee-ci.yml` (NEW): TEE attestation tests on `nitro-enclave`-capable runner
- Docker images: `Dockerfile.creusot`, `Dockerfile.gramine`

### Config additions (`~/.neotrix/config.toml`)

```toml
[seal.benchmark_gate]
enabled = true
suite_path = "~/.neotrix/benchmark_suite.toml"
threshold = -0.05

[seal.profile_injection]
enabled = false  # opt-in until stable
max_profile_tokens = 500

[hypercube]
dimension = 8192
cold_storage = "sqlite"  # "nvme" | "sqlite" | "none"
cold_storage_path = "~/.neotrix/cold_store/"
forgetting_threshold = 0.05

[sandbox]
wasm_enabled = true
wasm_memory_mb = 64
wasm_timeout_ms = 5000

[attestation]
provider = "simulated"  # "simulated" | "nitro" | "gramine"

[proof]
creusot_enabled = false  # CI-only
creusot_targets = ["nt_core_e8.rs", "nt_core_hcube.rs"]
```

---

## Implementation Notes

### Note 1: LATA is already applied — GRPO integration is the delta

Cycle 4 implemented `PrmHead` with LATA normalization. The remaining work is:
- `E8GroupSampler`: generates G trajectories
- `group_normalize()`: subtracts group mean, divides by group std
- `learn_from_advantages()`: updates E8 transition matrix

The PRM scoring infrastructure (`PrmObserver`, trajectory metadata) is already in place.

### Note 2: BenchmarkGate must be lightweight

Benchmark tasks should run in <30s total. Suite design principles:
- Tasks must be deterministic (or averaged over 3 runs)
- Mix of domains (reasoning, memory, code generation, tool use)
- Outputs scored via regex/match patterns, not LLM-as-judge (too slow)
- Results cached with TTL (re-run only if edit touches affected subsystems)

### Note 3: FHRR D=8192 requires threshold recalibration

All cosine similarity thresholds in the codebase must be re-validated:
- `HyperCube::similarity()` threshold for "match" (may shift from 0.85 to 0.90)
- `nt_memory_search` hybrid rerank threshold
- `nt_core_gwt` specialist-competition similarity
- Benchmark: pre/post threshold distributions on 1000 random vectors

### Note 4: WASM Python compilation uses wasmtime-pyodide

For Python→WASM, use `pyodide` as the WASM-compiled Python interpreter:
- Embed `pyodide.asm.wasm` (~12MB) in binary or download on first use
- Execute Python via `python-wasm` shim
- Fall back to Docker for scripts requiring native extensions (numpy, pandas, etc.)

### Note 5: Runtime tool security model

Tools created at runtime have elevated risk. Security measures:
1. **Code scanning**: Static analysis for `os`, `subprocess`, `eval`, `exec`, `__import__`
2. **Sandbox execution**: WASM + fuel metering + wall-clock timeout
3. **User confirmation**: `mcp publish` requires `--force` to skip confirmation
4. **Rate limiting**: 10 tool creations per hour per user
5. **Audit log**: All created tools logged with SHA-256 hash, creation timestamp, and creator identity

### Note 6: Creusot annotation strategy

Target functions for initial proof annotations:

| Function | Pre-condition | Post-condition | Priority |
|---|---|---|---|
| `E8State::transition(mode)` | `mode < 64` | `new_state < 64` | Critical |
| `FHRR::bind(a, b)` | — | `similarity(bind(a,b), bind(a,b)) ≈ 1.0` | Critical |
| `FHRR::bundle(a, b)` | — | `similarity(bundle(a,b), a) > 0.5` | High |
| `HyperCube::recall(key)` | `key.dim() == FHRR_DIMENSION` | — | High |
| `PrmHead::score(state)` | `state.len() == 64` | `score ∈ [0, 1]` | Medium |

### Note 7: TEE is a future-proofing investment

Real TEE attestation requires cloud infrastructure:
- AWS `nitro-enclaves` CLI + KMS integration
- AMD SEV-SNP on `c6a`/`m6a` instances (available but not default)
- Intel TDX on `c7i`/`m7i` instances (limited availability)

For Cycle 5, the `SimulatedAttestation` provider is sufficient for development. Real TEE integration is deferred to Cycle 6.

---

## Appendix A: File Map for Cycle 5

| File | Status | Purpose |
|---|---|---|
| `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/benchmark_gate.rs` | NEW | A-Evolve BenchmarkGate stage |
| `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/data_synthesis.rs` | NEW | Reflexio-style profile injection stage |
| `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/creusot_proof_gate.rs` | NEW | Creusot formal verification stage |
| `neotrix-core/src/crates/nt_core_e8.rs` | MOD | Add group sampler, beam search |
| `neotrix-core/src/crates/nt_core_policy.rs` | MOD | Add `learn_from_advantages()`, KL penalty, group normalization |
| `neotrix-core/src/crates/nt_core_prm.rs` | MOD | Add `PrmBeamSearch` |
| `neotrix-core/src/crates/nt_core_observer.rs` | MOD | Add PRM→policy integration hooks |
| `neotrix-core/src/crates/nt_core_hcube.rs` | MOD | FHRR D=8192, forgetting, cold storage integration |
| `neotrix-core/src/crates/nt_core_hcube/cold_storage.rs` | NEW | Cold storage backends (NVMe, SQLite) |
| `neotrix-core/src/crates/nt_core_hcube/forgetting.rs` | NEW | Ebbinghaus recall, forgetting certificates |
| `neotrix-core/src/shield/nt_shield_vault/forgetting.rs` | NEW | Ed25519 forgetting certificate management |
| `neotrix-core/src/agent/tool/mcp/registry.rs` | MOD | `publish()` accepts inline code, `UnifiedToolRegistry` |
| `neotrix-core/src/agent/tool/mcp/user_tool_store.rs` | NEW | SQLite-backed user-created tool registry |
| `neotrix-core/src/sandbox_v2/wasm_provider.rs` | NEW | WASM-based code execution provider |
| `neotrix-core/src/sandbox_v2/sandbox_router.rs` | NEW | Automatic provider selection (WASM vs Docker) |
| `neotrix-core/src/sandbox_v2/attestation.rs` | NEW | Attestation framework + SimulatedAttestation |
| `neotrix-core/src/sandbox_v2/tee_provider.rs` | NEW | TEE-based sandbox provider |
| `neotrix-core/src/sandbox_v2/key_broker.rs` | NEW | TEE-attestation key derivation |
| `neotrix-core/src/shield/nt_shield_perm.rs` | MOD | Tool creation permission model |
| `neotrix-core/src/cli/mcp.rs` | MOD | `mcp publish` accept code, `mcp list` show user tools |
| `neotrix-core/src/cli/sandbox.rs` | MOD | `sandbox status` show WASM provider |
| `docs/2-PLANS/2026-07-01-cycle5-experience-tree.md` | THIS | This document |

## Appendix B: Benchmark Suite Template

```toml
# ~/.neotrix/benchmark_suite.toml
version = "1.0"

[[tasks]]
id = "e8_reasoning_basic"
description = "Basic E8 reasoning chain of 8 steps"
type = "Reasoning"
input = "What is 2 + 2?"
expected_output_pattern = "4"
max_score = 1.0
timeout_secs = 5

[[tasks]]
id = "e8_reasoning_multi_step"
description = "Multi-step E8 reasoning (16 steps)"
type = "Reasoning"
input = "If x = 3 and y = 4, what is x^2 + y^2?"
expected_output_pattern = "25"
max_score = 1.0
timeout_secs = 10

[[tasks]]
id = "memory_store_retrieve"
description = "Store and retrieve from KB"
type = "Memory"
input = "Store that Paris is the capital of France, then retrieve it"
expected_output_pattern = "Paris"
max_score = 1.0
timeout_secs = 10

[[tasks]]
id = "code_gen_simple"
description = "Generate a Python fibonacci function"
type = "CodeGeneration"
input = "Write a Python function that computes fibonacci numbers"
expected_output_pattern = "def fib"
max_score = 1.0
timeout_secs = 30

[[tasks]]
id = "tool_use_mcp"
description = "Call an MCP tool to get weather"
type = "ToolUse"
input = "What's the weather in Tokyo?"
expected_output_pattern = "weather|temperature|°[CF]"
max_score = 1.0
timeout_secs = 30
```

---

## Appendix C: Version Compatibility

| Crate | Cycle 4 Version | Cycle 5 Target | Notes |
|---|---|---|---|
| `wasmtime` | — | 14.x | Latest stable with WASI preview 2 |
| `ed25519-dalek` | — | 2.x | For forgetting certificates + attestation |
| `sha2` | — | 0.10.x | Hash operations |
| `hkdf` | — | 0.12.x | Key derivation |
| `pyodide` | — | 0.25.x | WASM-compiled Python (downloaded, not vendored) |
| `creusot` | — | 0.1.x (Rust nightly) | CI-only, optional |
| `gramine` | — | 1.6+ | CI-only, optional TEE LibOS |

---

> **Cycle 5 is the alignment cycle.** It takes the infrastructure built in Cycle 4 (PRM, SAE, GRPO shell, compression, deploy) and makes it *learn* — GRPO for policy improvement, benchmark gates for quantifiable evolution, cognitive forgetting for sustainable memory, WASM for fast iteration, and formal verification for correctness guarantees. The result is an agent that not only self-edits but self-improves with measurable rigor.

---

## 经验树 — 2026-07-01 Deep GitHub Topics Absorption Cycle (2nd Wave)

### 研究范围
4 parallel agents explored 22+ domains across GitHub, producing 70+ blind spots.

| Domain | Projects Analyzed | Blind Spots |
|--------|------------------|-------------|
| Multi-Agent Orchestration | LangGraph, CrewAI, OpenAI SDK, AutoGen/MAF | 8 |
| Agentic RAG / Retrieval | LightRAG, GraphRAG, HippoRAG2, CRAG | 6 |
| Self-Improving | A-Evolve, DSPy, Voyager, Reflexio | 7 |
| Structured Output | XGrammar, Outlines, LM Format Enforcer | 3 |
| LLM Observability | Langfuse, AgentEval | 3 |
| Vector Databases | Qdrant, LanceDB, Milvus, Chroma | 9 |
| Agent Memory | Mem0, Letta, Zep/Graphiti, Cognee | 7 |
| Knowledge Graphs | FalkorDB, Neo4j LLM, Kappa Graph | 5 |
| LLM Caching | GPTCache, vCache, LMCache | 3 |
| Data Sovereignty | Apple PCC | 2 |
| Security/Guardrails | Guardrails AI, NeMo, Lakera | 5 |
| Sandbox | gVisor, Firecracker, wasmtime | 2 |
| Model Deployment | CoreML/MLX, llama.cpp, ExecuTorch | 3 |
| Identity/Auth | OAuth 2.1, OpenFGA, OPA, Sigstore | 3 |
| MCP Ecosystem | fastmcp, MCP spec, server patterns | 3 |
| Formal Verification | Creusot, Kani, Prusti | 2 |
| Reasoning/RL | TRL, PRM800K, OpenR, STaR | 4 |
| Interpretability | SAELens, TransformerLens, Anthropic SAE | 3 |
| VSA/HD Computing | torchhd, engram, Resonator Networks | 3 |
| On-Device ML | Apple Intelligence, Phi, Gemma | 3 |

### 汲取的关键洞见

**1. Multi-Agent Orchestration: LangGraph's superstep model**
- StateGraph with typed reducers makes agent state deterministic and replayable — NeoTrix E8 has no checkpointing
- The Pregel-inspired superstep synchronization barrier prevents race conditions in concurrent agent execution
- Gap: NeoTrix GWT is a blackboard (shared memory), not a typed message-passing graph

**2. CrewAI's dual-layer architecture**
- Crews (autonomous collaboration) inside Flows (deterministic orchestration) — solves autonomy-vs-control tension
- Role/goal/backstory separation creates predictable agent specialization
- Gap: NeoTrix SEAL is monolithic 27-stage; no hierarchical team decomposition

**3. OpenAI Agents SDK's minimal primitives**
- Only 4 primitives (Agent, Tool, Handoff, Guardrail) cover 90% of multi-agent patterns
- Handoff (specialist owns answer) vs agent-as-tool (manager retains control) is the key design axis
- Gap: NeoTrix has no handoff mechanism and no guardrail primitives

**4. GraphRAG's hierarchical community structure**
- Leiden community detection creates entity clusters; LLM summaries at each hierarchy level enable global-to-local retrieval
- Bottom-up summary generation (leaf→parent→root) avoids redundancy
- Gap: NeoTrix KB graph is flat; no community detection, no hierarchical retrieval

**5. HippoRAG2's Personalized PageRank**
- PPR naturally implements multi-hop retrieval — relevance propagates through graph edges, not just embedding space
- Complementary learning systems (hippocampal index + neocortex encoder) mimic human memory
- Gap: NeoTrix has entity→relation→entity graph but only uses flat FTS5/embedding search

**6. vCache's verified error bounds**
- Online learning finds optimal similarity threshold per embedding cluster
- Provides formal guarantees on false positive rate — 50-90% cost savings with provable correctness
- Gap: NeoTrix has NO semantic caching whatsoever

**7. Mem0's ADD-only memory pattern**
- Single-pass extraction, no UPDATE/DELETE — avoids memory corruption
- Multi-signal retrieval fused in parallel → 94.8 on LongMemEval
- Gap: NeoTrix overwrites/updates; now fixed with `search_fused()` (4-signal fusion)

**8. A-Evolve's benchmark-driven evolution**
- Solve→Observe→Evolve→Gate→Reload cycle with graduated mutation scope based on pass rate
- Solver-proposes, evolver-curates (GuidedSynthesis) produces higher-quality interventions
- EGL (Evolutionary Generality Loss) stagnation detection with git rollback
- Gap: SEAL needs benchmark-gating and staged-mutation patterns

**9. XGrammar's adaptive token mask cache**
- Precomputes token acceptance categorization (always-accepted / always-rejected / context-dependent) — most tokens are in the first two categories, O(1) masking
- TagDispatch for mixed structured/unstructured output
- Gap: NeoTrix has no token-level constrained decoding; now has stub in nt_io_constrained

### 执行的修复与新增模块

| 模块 | 新增/改造 | 功能 |
|------|-----------|------|
| `nt_memory_search` | 改造 | `search_fused()` 4信号并行融合 (FTS5+BM25+embedding+graph PPR) + LRU缓存 |
| `nt_core_cache` | 新增 | 语义缓存层: 自适应vCache阈值, LRU淘汰, 余弦相似度, TTL, 10测试 |
| `nt_core_policy` | 改造 | `beam_search_with_prm()` PRM引导+ LATA √L归一化, 3测试 |
| `nt_memory_types` | 改造 | `TemporalValidity`时间窗口, `supersedes`事实取代, `source_episode`来源追踪 |
| `nt_io_constrained` | 新增 | 约束解码存根: XGrammar架构, TagDispatch, 6测试 |
| `nt_memory_kb/mod.rs` | 改造 | KnowledgeBase添加`search_fused()`公开方法 |
| `core/mod.rs` | 注册 | 2新模块注册 + 重导出 |

### 前后状态对比

| 指标 | 之前 | 之后 |
|------|------|------|
| LLM调用缓存 | 无 | `nt_core_cache` (LRU + 自适应阈值 + TTL) |
| 搜索引擎 | FTS5/BM25/embedding级联 | 4信号并行融合 (FTS5+BM25+embedding+graph PPR) |
| E8搜索 | mode_values基 beam_search | PRM引导 beam_search (PRM+mode_values+LATA) |
| 记忆时间戳 | 无 | `TemporalValidity` + `supersedes` + `source_episode` |
| 约束解码 | 无 | `nt_io_constrained`存根 (XGrammar架构) |
| `cargo check --lib` | 0 errors | 0 errors ✅ |
| 新测试 | — | 25 new tests pass ✅ |

### 剩余高优先级盲点 (未执行)

以下P0盲点尚未实现, 列入Cycle 5 TODO:

1. **Graph-based orchestration** (LangGraph模式): `nt_core_graph_orch` 带checkpointing的有穷状态图
2. **OpenTelemetry observability**: `nt_io_telemetry` 全链路追踪 (已设计, 需依赖)
3. **Hierarchical community retrieval** (GraphRAG): Leiden社区检测 + 社区摘要
4. **Benchmark-driven evolution** (A-Evolve): SEAL benchmark-gating + 分步变异
5. **Constrained decoding full impl**: 需XGrammar Rust绑定集成到GatewayV2
6. **Epistemic confidence layer** (Kappa Graph): 置信度感知的KB检索
7. **Agent-driven memory management** (Letta): 智能体自编辑记忆块
8. **KV cache tiering** (LMCache): GPU→DRAM→SSD三级KV缓存
