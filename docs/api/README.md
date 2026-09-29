# NeoTrix API Documentation

> NeoTrix — Selective State-Space Agent Architecture
>
> Core formula: `Ψ(t+1) = Select(Ô, x) · Select(M, x) · Ψ(t)`

---

## Architecture Overview

NeoTrix implements a six-layer consciousness-embodiment-capability architecture:

```text
┌─────────────────────────────────────────────────────────┐
│  L6 Meta-Cognition                                       │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐   │
│  │ nt_meta     │ │ nt_governance│ │ healing          │   │
│  │ (self-model,│ │ (quality,    │ │ (self-heal,     │   │
│  │  planning,  │ │  audit,      │ │  diagnostic,    │   │
│  │  evolution) │ │  compliance) │ │  predictive)    │   │
│  └─────────────┘ └──────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L5 Cognition                                            │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐   │
│  │ nt_mind     │ │ nt_core      │ │ nt_consciousness │   │
│  │ (reasoning, │ │ (reasoning,  │ │ (GWT, IIT-phi,  │   │
│  │  knowledge, │ │  context,    │ │  crystal, tree) │   │
│  │  evolution) │ │  dispatch)   │ │                 │   │
│  └─────────────┘ └──────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L4 Emotion                                              │
│  ┌───────────────────────────────────────────────────┐   │
│  │ Emotional state tracking, affect regulation       │   │
│  └───────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L3 Embodiment                                           │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐   │
│  │ nt_shield   │ │ nt_guard     │ │ nt_computer     │   │
│  │ (security,  │ │ (chain,      │ │ (browser,       │   │
│  │  defense,   │ │  compliance) │ │  automation)    │   │
│  │  sandbox)   │ │              │ │                 │   │
│  └─────────────┘ └──────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L2 Perception                                           │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐   │
│  │ nt_world    │ │ nt_sense     │ │ nt_knowledge    │   │
│  │ (osint,     │ │ (E8, VSA,    │ │ (vector store,  │   │
│  │  crawl,     │ │  hypercube)  │ │  search)        │   │
│  │  explore)   │ │              │ │                 │   │
│  └─────────────┘ └──────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L1 Action                                               │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐   │
│  │ nt_act      │ │ nt_io        │ │ nt_mention      │   │
│  │ (orchestr.) │ │ (session,    │ │ (user mention)  │   │
│  │             │ │  logging)    │ │                 │   │
│  └─────────────┘ └──────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  L0 Substrate                                            │
│  ┌───────────────────────────────────────────────────┐   │
│  │ Math (E8, HyperCube, Walsh, Kronecker)             │   │
│  └───────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## Module Dependency Graph

```text
nt_meta (L6)
  ├── nt_governance (L6)
  ├── healing (L6)
  ├── nt_mind (L5)
  ├── nt_shield (L3)
  └── nt_world (L2)

nt_mind (L5)
  ├── nt_core (L5)
  ├── nt_consciousness (L5)
  ├── nt_shield (L3)
  ├── nt_world (L2)
  └── l0_substrate (L0)

nt_shield (L3)
  ├── nt_world (L2)
  └── l0_substrate (L0)

nt_world (L2)
  └── l0_substrate (L0)
```

---

## Quick Start Guide

### 1. Import Core Types

```rust
use neotrix_core::l5_cognition::nt_mind::{
    ReasoningBrain, SelfIteratingBrain, SelfEvolver,
    KnowledgeEngine, CapabilityVector,
};
use neotrix_core::l6_meta::nt_meta::{
    SelfModel, MetaCognitiveLoop, EvolutionPlanner,
};
```

### 2. Initialize Reasoning Brain

```rust
let brain = ReasoningBrain::new();
let mut iter_brain = SelfIteratingBrain::new(brain);
iter_brain.iterate(/* input */);
```

### 3. Run Meta-Cognitive Loop

```rust
let mut meta_loop = MetaCognitiveLoop::new();
let result = meta_loop.run_cycle();
```

### 4. Security Sandbox

```rust
use neotrix_core::l3_embodiment::nt_shield::nt_shield_sentry::SentryGuard;

let sentry = SentryGuard::new();
let approval = sentry.validate_action(&action)?;
```

### 5. World Perception

```rust
use neotrix_core::l2_perception::nt_world::osint::OsimpEngine;

let engine = OsintEngine::new();
let intel = engine.collect_intel(&target)?;
```

---

## Module Reference

| Layer | Module | Description |
|-------|--------|-------------|
| L0 | `nt_core_math` | E8 lattice, HyperCube, Walsh/Hadamard transforms |
| L1 | `nt_act` | Action orchestration, I/O, session recovery |
| L2 | `nt_world` | OSINT, crawling, exploration, media sources |
| L3 | `nt_shield` | Security sandbox, defense, compliance |
| L4 | — | Emotion tracking (planned) |
| L5 | `nt_mind` | Reasoning brain, knowledge engine, evolution |
| L5 | `nt_core` | Context engine, dispatch, model gateway |
| L5 | `nt_consciousness` | GWT, IIT-phi, consciousness tree |
| L6 | `nt_meta` | Self-model, planning, meta-cognition |
| L6 | `nt_governance` | Quality control, audit, compliance |
| L6 | `healing` | Self-healing, diagnostic, repair |

---

## Crate Structure

```text
neotrix/
├── neotrix-core/          # Main runtime crate
│   └── src/
│       ├── l0_substrate/  # Math foundations
│       ├── l1_action/     # Action execution
│       ├── l2_perception/ # World models, OSINT
│       ├── l3_embodiment/ # Security, sandbox
│       ├── l4_emotion/    # Emotion state
│       ├── l5_cognition/  # Reasoning, mind
│       └── l6_meta/       # Meta-cognition
├── crates/
│   ├── neotrix-consciousness/  # Consciousness systems
│   ├── neotrix-gateway/        # Model gateway
│   ├── neotrix-multi-agent/    # Multi-agent coordination
│   ├── neotrix-reasoning/      # Reasoning engines
│   ├── neotrix-sysctl/         # System control
│   ├── neotrix-types/          # Shared types
│   ├── neotrix-audit/               # Audit tooling
│   ├── neotrix-neobot/              # 本地推理（nt_llama）—— 桌面端复用的库
│   └── nt-core-capability-tree/     # 能力树
```

**桌面端不在本仓。** `src-tauri` 于 `5c02e738`（2026-09-28）归档、
`apps/neobot-desktop` 于 `d5413335`（同日）移除，桌面 App 统一到独立仓
`~/Downloads/Neo/neobot`（独立 2 成员 workspace，零 `neotrix-core` 依赖）。
本仓保留 `crates/neotrix-neobot` 作库：它的 `nt_llama` 是 CLI 与桌面端共用的
**唯一**本地推理实现 —— 复制实现正是此前「装完开不了话」的根因。

`nt-lang/`（DSL 编译器）亦已不在成员列表。当前 workspace 共 **10 个包**，
以 `cargo metadata --no-deps` 为准，不要照抄本文的列表。

---

## Safety Guarantees

- **R-P1**: Zero `unsafe` code in core modules (`#![forbid(unsafe_code)]`)
- **R-P2**: Production code: 0 warnings; test code exempt
- **R-SEC04**: Multi-turn security scanning
- **R-SEC10**: Defense profile enforcement

---

## Version

- Unified version: `0.18.0`
- Reasoning kernel: 18 stages
