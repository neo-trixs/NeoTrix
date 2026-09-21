# NeoTrix Architecture Refactoring Guide

> **Version**: 1.0.0 | **Date**: 2026-09-21 | **Status**: Architecture Refactoring Standard
> **Based On**: ISO/IEC 25010:2023, ISO/IEC/IEEE 42010:2022, arc42 Q42, Rust 2026 Best Practices, SDB Pattern, Agentic AI Reference Architecture 2026

---

## Table of Contents

1. [External Standards Integration Matrix](#1-external-standards-integration-matrix)
2. [ISO/IEC 25010:2023 Quality Model Application](#2-isoiec-250102023-quality-model-application)
3. [Technical Design Specifications](#3-technical-design-specifications)
4. [Process Specifications](#4-process-specifications)
5. [Data Specifications](#5-data-specifications)
6. [Architecture Refactoring Blueprint](#6-architecture-refactoring-blueprint)
7. [Layer-by-Layer Refactoring Plan](#7-layer-by-layer-refactoring-plan)
8. [Quality Gate Integration](#8-quality-gate-integration)
9. [Migration Execution Plan](#9-migration-execution-plan)

---

## 1. External Standards Integration Matrix

### 1.1 Standards Cross-Reference

| Standard | Domain | Key Contribution | NeoTrix Integration Point |
|----------|--------|-----------------|--------------------------|
| **ISO/IEC 25010:2023** | Software Quality | 9 quality characteristics model | Quality attributes for all layers |
| **ISO/IEC/IEEE 42010:2022** | Architecture Description | AD framework, viewpoints, model kinds | Architecture documentation standard |
| **arc42 Q42** | Quality Attributes | 8 key properties (simplified ISO) | Quality tree and scenarios |
| **Rust 2026 Best Practices** | Engineering | Async closures, module-first, zero-copy | L0-L1 implementation patterns |
| **SDB Pattern (arxiv 2605.20173)** | Agent Runtime | Stochastic-Deterministic Boundary | L5 Cognition SDB contract |
| **Agentic AI Ref Arch 2026** | Agent Systems | 6-layer stack, 3 deployment patterns | Cross-cutting plane alignment |
| **axum-harness** | Rust Backend | DDD+CAS+Outbox, contracts-first | L1 Action layer patterns |
| **Terraphim Engine** | AI Architecture | 6-layer 52-crate decomposition | Layer boundary discipline |
| **OWASP ASVS 2026** | Security | Input validation, audit logging | L3 Shield security controls |
| **SPDX/CycloneDX** | Supply Chain | SBOM generation | Dependency governance |

### 1.2 ISO/IEC 25010:2023 Quality Characteristics Mapping

```
ISO 25010:2023 Quality Model (9 Characteristics)
+--------------------------------------------------------------------+
|                                                                    |
|  1. Functional Suitability        6. Security                      |
|     - Functional completeness        - Confidentiality             |
|     - Functional correctness         - Integrity                   |
|     - Functional appropriateness     - Non-repudiation             |
|                                     - Accountability               |
|  2. Performance Efficiency         - Authenticity                  |
|     - Time behaviour                 - Resistance                  |
|     - Resource utilisation                                           |
|     - Capacity                   7. Maintainability                |
|                                     - Modularity                   |
|  3. Compatibility                   - Reusability                  |
|     - Interoperability               - Analysability               |
|     - Co-existence                   - Modifiability               |
|     - Compatibility                  - Testability                 |
|                                                                    |
|  4. Interaction Capability        8. Flexibility (was Portability) |
|     - Appropriateness recognisability - Adaptability                |
|     - Learnability                   - Installability              |
|     - Operability                    - Replaceability              |
|     - User error protection                                        |
|     - User interface aesthetics    9. Safety (NEW in 2023)         |
|     - Accessibility                 - Operational limitation      |
|                                     - State notification           |
|  5. Reliability                    - Safe integration             |
|     - Faultlessness                 - Error prevention             |
|     - Availability                                                 |
|     - Fault tolerance                                            |
|     - Recoverability                                             |
+--------------------------------------------------------------------+
```

### 1.3 arc42 Q42 Quality Attributes (Simplified)

| Q42 Attribute | ISO 25010 Mapping | NeoTrix Priority |
|---------------|-------------------|-----------------|
| Performance | Performance Efficiency | HIGH |
| Security | Security | BLOCKER |
| Reliability | Reliability | HIGH |
| Availability | Reliability > Availability | HIGH |
| Maintainability | Maintainability | HIGH |
| Testability | Maintainability > Testability | HIGH |
| Flexibility | Flexibility | MEDIUM |
| Interoperability | Compatibility > Interoperability | MEDIUM |

---

## 2. ISO/IEC 25010:2023 Quality Model Application

### 2.1 Quality Attributes per Layer

| Layer | Primary Quality Attributes | SMART Targets |
|-------|---------------------------|---------------|
| **L0 Substrate** | Reliability, Maintainability, Performance | 0 unsafe, 0 external deps, ECS query < 1us |
| **L1 Action** | Performance, Reliability, Security | P99 latency < 200ms, circuit breaker < 5s trip |
| **L2 Perception** | Functional Suitability, Performance | World query < 50ms, E8 hypothesis valid 100% |
| **L3 Embodiment** | Security, Safety, Reliability | 0 unguarded egress, alignment prevent 100% |
| **L4 Emotion** | Maintainability, Reliability | Memory CRUD round-trip, cross-session recovery |
| **L5 Cognition** | Functional Suitability, Performance | SEAL improvement > 0%, GWT routing correct |
| **L6 Meta** | Security, Reliability, Safety | Self-heal MTTR < 30s, governance 100% |

### 2.2 Quality Scenarios (arc42 Style)

#### QS-1: Performance Efficiency (L1 Action)

```
Scenario: High-concurrency LLM routing
Stimulus: 100 concurrent requests to model gateway
Environment: Production, 30+ LLM providers registered
Response: All requests routed within 200ms (p99)
Measurement: tokio-console + OpenTelemetry tracing
Priority: HIGH
```

#### QS-2: Security (L3 Embodiment)

```
Scenario: Prompt injection attack
Stimulus: Malicious user input containing injection payload
Environment: Production, shield active
Response: Injection detected, request blocked, audit logged
Measurement: nt_shield adversarial pipeline verdict
Priority: BLOCKER
```

#### QS-3: Reliability (L6 Meta)

```
Scenario: LLM provider failure cascade
Stimulus: Primary + secondary providers both fail
Environment: Production, circuit breaker active
Response: Fallback to tertiary provider within 5s, no user-visible error
Measurement: Circuit breaker trip time + fallback latency
Priority: HIGH
```

#### QS-4: Maintainability (All Layers)

```
Scenario: New LLM provider integration
Stimulus: Add new provider (e.g., Mistral)
Environment: Development, CI pipeline
Response: Provider registered, all tests pass, facade unchanged
Measurement: Integration time < 2 hours, 0 facade changes
Priority: MEDIUM
```

#### QS-5: Safety (L6 Meta)

```
Scenario: Autonomous action exceeding safety bounds
Stimulus: Agent proposes irreversible action above threshold
Environment: Production, safety monitor active
Response: Action blocked, human approval required, audit logged
Measurement: nt_safety_monitor verdict + HITL gate response
Priority: BLOCKER
```

### 2.3 Quality Attribute Trade-off Matrix

```
+--------------------------------------------------------------------+
|                    Quality Attribute Trade-offs                     |
+--------------------------------------------------------------------+
|                                                                     |
|  Security <-> Performance                                           |
|    - Encryption adds latency                                        |
|    - Solution: Cache security decisions, hardware acceleration      |
|                                                                     |
|  Reliability <-> Cost                                               |
|    - Replication increases infrastructure cost                      |
|    - Solution: Async replication, tune consistency models           |
|                                                                     |
|  Maintainability <-> Performance                                    |
|    - Modular architecture adds indirection overhead                 |
|    - Solution: Profile first, optimize hot paths only               |
|                                                                     |
|  Safety <-> Autonomy                                               |
|    - HITL gates reduce agent autonomy                               |
|    - Solution: Confidence-triggered gates, not blanket gates        |
|                                                                     |
|  Flexibility <-> Simplicity                                         |
|    - Abstraction layers add complexity                              |
|    - Solution: Module-first, crate-only for strict boundaries       |
|                                                                     |
+--------------------------------------------------------------------+
```

---

## 3. Technical Design Specifications

### 3.1 Rust 2026 Engineering Standards

Based on the latest Rust ecosystem consensus (2026):

| Category | Standard | Implementation |
|----------|----------|---------------|
| **Edition** | Rust 2024 Edition (1.85+) | Async closures, precise capturing |
| **Structure** | Module-first, crate-last | Deep nesting within single lib crate |
| **Errors** | `thiserror` for libraries, `anyhow` for apps | L0-L5 use thiserror, bin uses anyhow |
| **Async** | Cancellation safety prioritized | `tokio::task::spawn_blocking` for blocking ops |
| **Data** | Zero-copy for high-throughput | `rkyv` for IPC, `&str` over `String` |
| **Testing** | Property-based + snapshot | `proptest` for logic, `insta` for output |
| **Runtime** | Tokio multi-threaded | `worker_threads` = vCPU count |
| **Channels** | Bounded async channels | `tokio::sync::mpsc` with capacity |
| **Visibility** | Sealed traits for API control | Public API audit per module |
| **CI** | `cargo-nextest` standard | Faster test isolation |

### 3.2 Hexagonal Architecture Integration

Apply Ports & Adapters pattern within each layer:

```
Each NeoTrix Module Structure:
+--------------------------------------------------------------------+
|                                                                     |
|  domain/           Core business logic (pure, no dependencies)      |
|    model.rs        Domain types (structs, enums, traits)            |
|    service.rs      Domain services (business rules)                 |
|    port.rs         Ports (trait interfaces for infrastructure)      |
|                                                                     |
|  application/      Use case orchestration                           |
|    handler.rs      Command/query handlers                           |
|    coordinator.rs  Cross-domain coordination                        |
|                                                                     |
|  infrastructure/   Adapters (concrete implementations)              |
|    persistence/    Storage adapters                                 |
|    external/       External service adapters                        |
|    api/            HTTP/WS/MCP adapters                             |
|                                                                     |
|  tests/            Integration tests (use real adapters)            |
|                                                                     |
+--------------------------------------------------------------------+
```

### 3.3 Stochastic-Deterministic Boundary (SDB) Pattern

Apply SDB to L5 Cognition (where LLM proposals become system actions):

```
SDB Contract for NeoTrix:
+--------------------------------------------------------------------+
|                                                                     |
|  1. PROPOSER (LLM)                                                  |
|     - Generates action proposal from distribution                   |
|     - Located in: nt_mind/ (SEAL pipeline)                         |
|                                                                     |
|  2. VERIFIER (Deterministic)                                        |
|     - Schema validation                                             |
|     - Policy rule enforcement                                       |
|     - State machine transition predicate                            |
|     - Located in: nt_core_policy, nt_core_gate                      |
|                                                                     |
|  3. COMMIT STEP (Durable Write)                                     |
|     - Event outbox write                                            |
|     - State mutation                                                |
|     - Side-effect execution                                         |
|     - Located in: nt_core_event, nt_infra_persistence               |
|                                                                     |
|  4. REJECT SIGNAL (Typed Response)                                  |
|     - Structured error back to proposer                             |
|     - Retry guidance                                                |
|     - Fallback trigger                                              |
|     - Located in: nt_core_error, nt_core_gate                       |
|                                                                     |
+--------------------------------------------------------------------+
```

### 3.4 Contracts-First Development

```
Contract Definition Process:
+--------------------------------------------------------------------+
|                                                                     |
|  1. Define contract in packages/contracts/                          |
|     - API contracts (OpenAPI/AsyncAPI)                              |
|     - Event contracts (schema registry)                             |
|     - DTO contracts (shared types)                                  |
|     - Error contracts (error code catalog)                          |
|                                                                     |
|  2. CI drift detection gate                                         |
|     - Contract vs implementation comparison                         |
|     - Breaking change detection                                     |
|     - Version compatibility check                                   |
|                                                                     |
|  3. Implementation against contract                                 |
|     - Domain code implements contract interfaces                     |
|     - Infrastructure adapters provide concrete implementations     |
|                                                                     |
+--------------------------------------------------------------------+
```

### 3.5 DDD Bounded Context Mapping

```
NeoTrix Bounded Contexts:
+--------------------------------------------------------------------+
|                                                                     |
|  Context: Cognition                                                 |
|    - Domain: Consciousness, Decision, Reasoning, Planning           |
|    - Ubiquitous Language: SEAL, GWT, E8, Policy, Goal               |
|    - Module: l5_cognition/                                          |
|    - Boundaries: nt_mind, nt_core, nt_council                       |
|                                                                     |
|  Context: Perception                                                |
|    - Domain: World Model, Data Sources, Knowledge                   |
|    - Ubiquitous Language: DataSource, IntelSource, OsintSource      |
|    - Module: l2_perception/                                         |
|    - Boundaries: nt_world, nt_core_e8, nt_core_knowledge            |
|                                                                     |
|  Context: Security                                                  |
|    - Domain: Shield, Guard, Safety, Audit                           |
|    - Ubiquitous Language: GuardChain, Sandbox, ShieldVerdict        |
|    - Module: l3_embodiment/                                         |
|    - Boundaries: nt_shield, nt_security                             |
|                                                                     |
|  Context: Memory                                                    |
|    - Domain: Experience, Knowledge, Coverage                        |
|    - Ubiquitous Language: ExperienceTree, CoverageLedger, KB        |
|    - Module: l4_emotion/nt_memory/                                  |
|    - Boundaries: nt_memory, nt_feel                                 |
|                                                                     |
|  Context: Action                                                    |
|    - Domain: LLM, IO, Media, Task                                   |
|    - Ubiquitous Language: Provider, Adapter, Tier, Dispatch         |
|    - Module: l1_action/                                             |
|    - Boundaries: nt_io, nt_act, nt_media                            |
|                                                                     |
|  Context: Meta                                                      |
|    - Domain: Self, Evolution, Governance, Healing                   |
|    - Ubiquitous Language: SelfModel, Evolution, Governance          |
|    - Module: l6_meta/                                               |
|    - Boundaries: nt_core_self, nt_governance, nt_meta               |
|                                                                     |
+--------------------------------------------------------------------+
```

---

## 4. Process Specifications

### 4.1 Development Workflow

```
+--------------------------------------------------------------------+
|                    NeoTrix Development Process                       |
+--------------------------------------------------------------------+
|                                                                     |
|  Phase 1: CONTRACT                                                  |
|    - Define API/Event/DTO/Error contracts first                     |
|    - Write ADR with ISO 25010 quality-attribute tags                |
|    - Review: Architect + Security                                   |
|    - Gate: Contract review approved                                 |
|                                                                     |
|  Phase 2: DESIGN                                                    |
|    - DDD bounded context identification                             |
|    - Module structure (domain/application/infrastructure)           |
|    - Quality scenarios (arc42 Q42 style)                            |
|    - Review: Architecture fitness functions                         |
|    - Gate: Design review approved                                   |
|                                                                     |
|  Phase 3: IMPLEMENT                                                 |
|    - TDD: Red-Green-Refactor                                        |
|    - Property-based testing for logic                               |
|    - Snapshot testing for output                                    |
|    - Gate: All tests pass, cargo-deny clean                         |
|                                                                     |
|  Phase 4: VERIFY                                                    |
|    - Architecture fitness functions pass                            |
|    - Security scan (cargo audit + custom lint)                      |
|    - Performance regression < 5%                                    |
|    - Gate: All gates pass                                           |
|                                                                     |
|  Phase 5: DELIVER                                                   |
|    - Documentation update (module README, ADR)                      |
|    - Changelog entry                                                |
|    - Integration test suite update                                  |
|    - Gate: PR approved + merged                                     |
|                                                                     |
+--------------------------------------------------------------------+
```

### 4.2 ADR Template with ISO 25010 Tags

```markdown
---
id: ADR-XXXX
date: YYYY-MM-DD
status: proposed | accepted | deprecated | superseded
quality-attributes: [security, reliability, maintainability]
requirements: [QR-001, QR-002]
tags: [architecture, layer-N, module-name]
---

# ADR-XXXX: [Decision Title]

## Context

[What is the issue that we're seeing that is motivating this decision?]

## Decision

[What is the change that we're proposing and/or doing?]

## Quality Attribute Impact

| Attribute | Impact | Measurement |
|-----------|--------|-------------|
| Security | +positive | [how measured] |
| Performance | -negative | [how measured] |
| Maintainability | +positive | [how measured] |

## Consequences

[What becomes easier or more difficult to do because of this change?]

## Alternatives Considered

[What other options were evaluated?]
```

### 4.3 Review Checklist

```
Architecture Review Checklist (per PR):
+--------------------------------------------------------------------+
|                                                                     |
|  [ ] Dependency Direction                                           |
|      - No reverse dependencies (cargo deny)                        |
|      - Facade isolation maintained                                  |
|                                                                     |
|  [ ] Quality Attributes (ISO 25010)                                 |
|      - Performance: No regression > 5%                             |
|      - Security: No new attack surface                             |
|      - Reliability: Error paths covered                            |
|      - Maintainability: Module structure clean                      |
|      - Testability: New code has tests                              |
|                                                                     |
|  [ ] SDB Compliance (L5 only)                                       |
|      - All LLM-to-action paths have verifier                       |
|      - Reject signals are typed                                    |
|      - Commit steps are durable                                    |
|                                                                     |
|  [ ] DDD Boundaries                                                 |
|      - Bounded context respected                                   |
|      - No cross-context direct imports                             |
|      - Ubiquitous language consistent                              |
|                                                                     |
|  [ ] Rust 2026 Standards                                            |
|      - Zero unsafe                                                 |
|      - Cancellation safety in async                                |
|      - Structured error handling                                   |
|      - No blocking in async tasks                                  |
|                                                                     |
|  [ ] Documentation                                                  |
|      - Module README updated                                       |
|      - ADR created/updated                                         |
|      - API docs generated                                          |
|                                                                     |
+--------------------------------------------------------------------+
```

---

## 5. Data Specifications

### 5.1 Data Flow Architecture

```
NeoTrix Data Flow (SDB-Aligned):
+--------------------------------------------------------------------+
|                                                                     |
|  User Input                                                         |
|    |                                                                |
|    v                                                                |
|  [Channel Adapter] -- L1 Action                                     |
|    |                                                                |
|    v                                                                |
|  [Triage Gate] -- L0/L1 (small model classification)               |
|    |                                                                |
|    +-- Simple --> [Direct Response] -- Fast tier                    |
|    |                                                                |
|    +-- Complex --> [Orchestrator] -- L5 Cognition                   |
|                      |                                              |
|                      +-- PROPOSE (LLM)                              |
|                      |                                              |
|                      +-- VERIFY (Deterministic)                     |
|                      |    - Schema check                           |
|                      |    - Policy check                           |
|                      |    - Safety check                           |
|                      |                                              |
|                      +-- COMMIT (Durable)                           |
|                      |    - Event outbox                           |
|                      |    - State write                            |
|                      |    - Audit log                              |
|                      |                                              |
|                      +-- REJECT (Typed)                             |
|                           - Error response                         |
|                           - Fallback trigger                       |
|                                                                     |
+--------------------------------------------------------------------+
```

### 5.2 Event Schema Standard

```rust
// Standard Event Schema (contracts-first)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoTrixEvent {
    /// Event ID (UUID v7 for time-ordered)
    pub id: Uuid,
    /// Event type (PascalCase, module::action)
    pub event_type: String,
    /// Source module (layer::module path)
    pub source: String,
    /// Timestamp (ISO 8601)
    pub timestamp: DateTime<Utc>,
    /// Correlation ID (trace chain)
    pub correlation_id: Uuid,
    /// Causation ID (what caused this event)
    pub causation_id: Option<Uuid>,
    /// Event payload (typed, schema-validated)
    pub payload: serde_json::Value,
    /// Schema version (for evolution)
    pub schema_version: u32,
    /// Quality attributes affected
    pub quality_attributes: Vec<String>,
}
```

### 5.3 Error Code Standard

```rust
// Standard Error Code Format: LAYER-MODULE-CODE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoTrixError {
    /// Error code: LAYER-MODULE-CODE (e.g., L1-IO-001)
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Error category (ISO 25010 aligned)
    pub category: ErrorCategory,
    /// Severity (BLOCKER/HIGH/MEDIUM/LOW)
    pub severity: Severity,
    /// Recovery strategy
    pub recovery: RecoveryStrategy,
    /// Stack trace (debug mode only)
    pub trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Functional correctness issue
    Functional,
    /// Performance degradation
    Performance,
    /// Security violation
    Security,
    /// Reliability failure
    Reliability,
    /// Safety concern
    Safety,
    /// Compatibility issue
    Compatibility,
    /// Maintainability debt
    Maintainability,
}
```

### 5.4 Telemetry Schema Standard

```rust
// Standard Telemetry Span
#[derive(Debug, Clone)]
pub struct TelemetrySpan {
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Operation name
    pub operation: String,
    /// Layer (L0-L6)
    pub layer: Layer,
    /// Module path
    pub module: String,
    /// Start time
    pub start: Instant,
    /// Attributes (key-value pairs)
    pub attributes: HashMap<String, AttributeValue>,
    /// Events within span
    pub events: Vec<TelemetryEvent>,
    /// Status
    pub status: SpanStatus,
}
```

---

## 6. Architecture Refactoring Blueprint

### 6.1 Refactoring Principles

| Principle | Description | Enforcement |
|-----------|-------------|-------------|
| **Strangler Fig** | New code replaces old incrementally | No big-bang rewrites |
| **Contract-First** | Define interfaces before implementation | CI drift detection |
| **Quality-Gated** | Every change passes quality checks | Automated gates |
| **Measured** | Performance/security baselines before refactoring | Benchmark comparison |
| **Documented** | Every decision has an ADR | PR template |

### 6.2 Refactoring Priority Matrix

```
+--------------------------------------------------------------------+
|                  Refactoring Priority Matrix                        |
+--------------------------------------------------------------------+
|                                                                     |
|  P0: BLOCKER (Fix Immediately)                                      |
|    - Reverse dependencies (cargo deny violations)                   |
|    - Unsafe code in L0                                              |
|    - Missing SDB verifier in L5                                     |
|    - Unguarded egress in L3                                         |
|                                                                     |
|  P1: HIGH (This Sprint)                                            |
|    - Facade boundary violations                                     |
|    - Missing circuit breakers (L1)                                  |
|    - Incomplete error recovery (L0)                                 |
|    - Missing telemetry spans                                        |
|                                                                     |
|  P2: MEDIUM (Next Sprint)                                          |
|    - Module structure alignment (DDD)                               |
|    - Contract definition gaps                                       |
|    - Test coverage gaps                                             |
|    - Documentation debt                                             |
|                                                                     |
|  P3: LOW (Backlog)                                                 |
|    - Zero-copy optimization                                         |
|    - Snapshot testing adoption                                      |
|    - Property-based testing expansion                               |
|    - Performance tuning                                             |
|                                                                     |
+--------------------------------------------------------------------+
```

### 6.3 Module Absorption Protocol

For absorbing external modules into the layered architecture:

```
Module Absorption Checklist:
+--------------------------------------------------------------------+
|                                                                     |
|  [ ] Pre-Absorption                                                 |
|      - Source module analyzed                                       |
|      - Target layer identified                                      |
|      - Bounded context mapped                                       |
|      - Quality requirements defined                                 |
|                                                                     |
|  [ ] Structural                                                      |
|      - Module placed in correct layer directory                     |
|      - Dependencies re-oriented (lower layers only)                 |
|      - Facade boundary established                                  |
|      - Public API surface minimized                                 |
|                                                                     |
|  [ ] Quality                                                         |
|      - Zero unsafe verified                                         |
|      - Error handling aligned                                       |
|      - Telemetry integrated                                         |
|      - Tests pass                                                   |
|                                                                     |
|  [ ] Documentation                                                   |
|      - Module README created                                        |
|      - ADR for absorption decision                                  |
|      - Architecture diagram updated                                 |
|      - Changelog entry                                              |
|                                                                     |
+--------------------------------------------------------------------+
```

---

## 7. Layer-by-Layer Refactoring Plan

### 7.1 L0 Substrate Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| External deps | 0 | 0 | MAINTAIN |
| Unsafe code | 0 | 0 | MAINTAIN |
| Error types | Partial Recovery | All implement Recovery | P1 |
| Event system | Basic EventBus | Typed events + schema validation | P1 |
| ECS | Basic archetype | Add parallel query optimization | P2 |
| Telemetry | Basic tracing | OpenTelemetry integration | P2 |
| Cross-layer types | Manual re-exports | Automated re-export registry | P3 |

### 7.2 L1 Action Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| Facade | nt_action_facade | Strict facade enforcement | P1 |
| Providers | 30+ providers | Registry pattern + health checks | MAINTAIN |
| Circuit breaker | Basic | Tier-aware with backoff | P1 |
| Blocking ops | Mixed | All via spawn_blocking | P1 |
| Channels | Unbounded | Bounded async channels | P1 |
| Contracts | Implicit | Explicit API contracts | P2 |
| Error handling | Mixed anyhow/thiserror | Unified thiserror per module | P2 |

### 7.3 L2 Perception Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| World model | 259 files | Split into bounded contexts | P2 |
| E8 reasoning | Monolithic | Port-based extraction | P2 |
| Data sources | 15+ sources | Registry + contract-first | P2 |
| Vector store | Basic | Abstraction layer + adapters | P3 |

### 7.3 L3 Embodiment Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| Shield | 247 files | Modular guard chain | P1 |
| Safety alignment | Basic | Full alignment engine | P0 |
| Anti-distillation | Partial | Complete defense pipeline | P1 |
| Computer abstraction | Basic | Fleet management + health | P2 |

### 7.4 L4 Emotion Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| Memory | 199 files | Split KB/Experience/Coverage | P2 |
| Emotion bridge | Basic | Full PAD -> GWT/E8/CT | P2 |
| Coverage ledger | Partial | Complete tracking | P2 |

### 7.5 L5 Cognition Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| SEAL pipeline | Basic | Full SDB contract | P0 |
| GWT attention | Basic | Production routing | P1 |
| Model gateway | Basic | Cost-aware + fallback | P1 |
| Facade | nt_cognition_facade | Strict facade enforcement | P1 |
| Policy engine | Basic | Full policy enforcement | P1 |
| Planning | Basic | Full lifecycle management | P2 |

### 7.6 L6 Meta Refactoring

| Item | Current State | Target State | Priority |
|------|--------------|-------------|----------|
| Governance | Basic | Full compliance engine | P0 |
| Self-healing | Basic | MTTR < 30s | P1 |
| Safety monitor | Basic | Anomaly detection + HITL | P0 |
| Evolution | Basic | Measurable improvement | P1 |
| Cross-session | Basic | Checkpoint durability | P1 |

---

## 8. Quality Gate Integration

### 8.1 Automated Quality Gates

```
CI/CD Pipeline Quality Gates:
+--------------------------------------------------------------------+
|                                                                     |
|  Gate 1: COMPILE                                                    |
|    - cargo check --all-targets -p neotrix                           |
|    - Zero warnings                                                  |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 2: DEPENDENCY AUDIT                                           |
|    - cargo deny (no reverse deps, no unsafe)                        |
|    - cargo audit (no known vulnerabilities)                         |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 3: UNIT TESTS                                                 |
|    - cargo nextest run -p neotrix --lib                             |
|    - Coverage >= 80% for critical paths                             |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 4: ARCHITECTURE FITNESS                                       |
|    - nt_core_arch_fitness functions                                 |
|    - Facade boundary check                                          |
|    - SDB contract verification (L5)                                 |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 5: SECURITY SCAN                                              |
|    - Static analysis (clippy + custom lints)                        |
|    - Secret scanning (pre-commit + CI)                              |
|    - Input validation audit                                         |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 6: INTEGRATION TESTS                                          |
|    - cargo test --test '*'                                          |
|    - E2E pipeline tests                                             |
|    - Chaos tests (weekly)                                           |
|    - Blocking: YES                                                  |
|                                                                     |
|  Gate 7: PERFORMANCE                                                |
|    - cargo bench (regression < 5%)                                  |
|    - Latency benchmarks                                             |
|    - Memory profiling                                               |
|    - Blocking: NO (warning only)                                    |
|                                                                     |
|  Gate 8: DOCUMENTATION                                              |
|    - Module README present                                          |
|    - ADR for new decisions                                          |
|    - API docs generated                                             |
|    - Blocking: NO (warning only)                                    |
|                                                                     |
+--------------------------------------------------------------------+
```

### 8.2 Architecture Fitness Functions

| ID | Function | Layer | Metric | Threshold | Enforcement |
|----|----------|-------|--------|-----------|-------------|
| AF-1 | Zero Unsafe | All | unsafe block count | 0 | Compile fail |
| AF-2 | Zero Reverse Deps | All | reverse dependency count | 0 | cargo deny |
| AF-3 | Zero External Deps (L0) | L0 | external crate count | 0 | cargo deny |
| AF-4 | Facade Isolation | L1,L5 | public interface count | <= 3 | Custom lint |
| AF-5 | SDB Verifier | L5 | unverified LLM calls | 0 | Custom lint |
| AF-6 | Circuit Breaker | L1 | unguarded providers | 0 | Integration test |
| AF-7 | Error Recovery | L0 | Error without Recovery | 0 | Compile fail |
| AF-8 | Cancellation Safety | L1 | blocking in async | 0 | Clippy lint |
| AF-9 | Test Coverage | All | coverage percentage | >= 80% | CI gate |
| AF-10 | Self-Heal MTTR | L6 | repair time | < 30s | Chaos test |

---

## 9. Migration Execution Plan

### 9.1 Phase 1: Foundation (Weeks 1-4)

```
Week 1-2: Quality Baseline
  - Run all fitness functions, record current state
  - Identify all BLOCKER-level violations
  - Create ADRs for all existing violations
  - Establish performance baselines

Week 3-4: L0-L1 Foundation
  - Fix all reverse dependencies
  - Implement error recovery for all L0 Error types
  - Add circuit breakers to all L1 providers
  - Convert all blocking ops to spawn_blocking
  - Add bounded channels between stages
```

### 9.2 Phase 2: Core Refactoring (Weeks 5-12)

```
Week 5-8: L3-L5 Core
  - Implement full SDB contract in L5
  - Strengthen L3 shield guard chain
  - Add full safety alignment engine
  - Implement governance compliance engine

Week 9-12: L2-L4 Integration
  - Refactor world model into bounded contexts
  - Implement contract-first for data sources
  - Split memory into KB/Experience/Coverage
  - Implement emotion-reasoning bridge
```

### 9.3 Phase 3: Hardening (Weeks 13-16)

```
Week 13-14: L6 Meta
  - Implement full self-healing (MTTR < 30s)
  - Add anomaly detection to safety monitor
  - Implement measurable evolution metrics

Week 15-16: Integration Testing
  - Full E2E pipeline testing
  - Chaos engineering tests
  - Performance regression testing
  - Security penetration testing
```

### 9.4 Phase 4: Documentation & Polish (Weeks 17-20)

```
Week 17-18: Documentation
  - Update all module READMEs
  - Create/update all ADRs
  - Generate API documentation
  - Update architecture diagrams

Week 19-20: Final Verification
  - Full architecture review (rev-officer)
  - ISO 25010 compliance audit
  - Performance benchmark comparison
  - Security audit
```

### 9.5 Success Criteria

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Reverse dependencies | TBD | 0 | cargo deny |
| Unsafe blocks | 0 | 0 | cargo audit |
| Test coverage | ~75% | >= 80% | cargo-tarpaulin |
| Self-heal MTTR | ~25s | < 30s | Chaos test |
| P99 latency (L1) | TBD | < 200ms | Criterion bench |
| Architecture fitness score | 85/100 | >= 90 | Fitness functions |
| ADR coverage | TBD | 100% | Manual audit |
| Module README coverage | TBD | 100% | Automated check |

---

*End of Document*
