# Decision Log
> Every architecture decision must be recorded with context, options considered, and rationale.

## D-001: Unbounded Channel Fix (2026-06-19)
**Context**: 3 unbounded mpsc::channel() found — OOM risk in CI pipeline
**Decision**: Replace with sync_channel(256)
**Options considered**: (a) unbounded + drain thread, (b) bounded + backpressure
**Rationale**: 256 capacity with backpressure is zero-overhead and matches the per-cycle message rate
**Affected files**: review/mod.rs:77,129, nt_core_tool/mod.rs:33

## D-002: #[serial] Singleton Protection (2026-06-19)
**Context**: 18 OnceLock<Mutex<>> globals lacked test isolation
**Decision**: Add #[serial_test::serial] to each test module
**Options considered**: (a) DI framework, (b) #[serial] annotation
**Rationale**: #[serial] is zero-overhead outside tests; DI framework is 3+ session investment
**Affected files**: memory_evolution.rs, efe_curiosity_bridge.rs, evidence_inspector.rs, silicon_self.rs, ...

## D-003: Dead Dispatch Arm Marking (2026-06-19)
**Context**: 89 handler dispatch arms registered but no pipeline caller
**Decision**: Mark with `// DEAD` comment — do not delete
**Options considered**: (a) delete arms, (b) add callers, (c) annotate
**Rationale**: Deleting risks forgetting why they exist; adding callers is out of scope; `// DEAD` preserves intent
**Affected files**: modules_core.rs:1881-2135

## D-004: Governance Layer Creation (2026-06-19)
**Context**: No centralized spec registry, decision log, or self-evolving rules existed — architectural decisions were captured only in AGENTS.md session logs
**Decision**: Create `governance/` directory with SPEC_INDEX.md, DECISION_LOG.md, RULES.md, and per-subsystem spec files
**Options considered**: (a) embed in AGENTS.md, (b) create standalone governance directory
**Rationale**: AGENTS.md is already 3000+ lines; separating governance into its own tree keeps the behavior spec focused on identity and rules, while specs/decision-log live in a dedicated structure
**Affected files**: governance/* (new)

## D-005: Production panic elimination (2026-06-19)
**Context**: ~90 unwrap/expect sites in production code (non-test) posed crash risk in handler dispatch, JSON parsing, and crypto init paths
**Decision**: Replace with match + early return / unwrap_or_default / unwrap_or(Value::Null)
**Options considered**: (a) propagate Result, (b) recover with fallback values
**Rationale**: Result propagation would balloon signature changes across ~1800 call sites; fallback values are zero-risk for the derived paths (JSON serialization, regex compilation)
**Affected files**: modules_agent.rs, modules_kb.rs, modules_storm.rs, modules_core.rs, openai_compat.rs, nt_io_proxy.rs, web_navigator.rs, nt_core_agent/*, a2a_grpc.rs

## D-006: VSA Universal Representation Enforcement (2026-06-19)
**Context**: Architecture Principle #2 (all subsystems share VSA 4096-bit vectors) had 8/16 subsystems with zero VSA operations
**Decision**: Add `Option<[u8; 64]>` vsa_fingerprint to all subsystem structs via QuantizedVSA::seeded_random()
**Options considered**: (a) force all data paths through VSA, (b) add fingerprint only for traceability
**Rationale**: Full VSA data path conversion would require pipeline rewrites; fingerprint provides traceability and enables future VSA-native operations without breaking existing consumers
**Affected files**: curiosity_drive.rs, evidence.rs, narrative_self.rs, consciousness_bridge.rs, hypergraph.rs, memory_distill/*, storm_engine.rs, jepa/*

## D-007: Architecture Leak Clearance (2026-06-19)
**Context**: 8 prompt files contained E8/HyperCube/SEAL/GWT internal architecture identifiers sent to third-party LLMs — exposing attack surface
**Decision**: Replace all internal architecture identifiers with generic descriptors (E8→"reasoning core", SEAL→"self-improvement loop", HyperCube→"knowledge representation", GWT→"attention routing")
**Options considered**: (a) strip all technical identifiers, (b) keep as "brand differentiator"
**Rationale**: Architecture identifiers provide no user-facing value but expose internal topology to adversarial prompt engineering
**Affected files**: content.rs, server.rs, brain_cmds.rs, core_cmds.rs, reasoning_distiller.rs, templates.rs, maintenance.rs, benchmark.rs
