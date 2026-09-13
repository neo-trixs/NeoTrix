# Targeted Research 554 — Internal Dispatch Routes

**Date**: 2026-09-12
**File**: `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`
**Author**: NeoTrix consciousness core dispatch wiring

## Summary

Added 4 new internal dispatch routes to `CAPABILITY_ROUTES` table and `dispatch_internal_capability` match arms in the consciousness core. These routes enable the consciousness core to decompose and execute privacy/security/evolution/memory tasks without external knowledge sources.

## Routes Added

| Keyword(s) | Capability Tag | NT Domain | SpecialistType | Purpose |
|------------|---------------|-----------|----------------|---------|
| `隐私`, `脱敏`, `隐私遮罩`, `privacy_mask` | `privacy_mask` | NT-SHIELD | RiskAssessor | Egress privacy masking via `nt_core_llm::redact_internals` |
| `证据扫描`, `证据安全`, `evidence_scan` | `evidence_scan` | NT-SHIELD | RiskAssessor | Evidence-first security scanning via `AgenticScanner::recon_scan` |
| `技能进化`, `技能演进`, `skill_evolution` | `skill_evolution` | NT-MIND | KnowledgeIntegrator | Skill evolution tracking via `EvolutionLoop::run_cycle` |
| `经验桥接`, `经验知识`, `experience_bridge` | `experience_bridge` | NT-MEMORY | KnowledgeIntegrator | Experience-knowledge bridge via `KnowledgeBase::serve_core` |

## Architecture

### CAPABILITY_ROUTES (deterministic keyword → capability mapping)

```
privacy_mask    → (privacy_mask,    NT-SHIELD,  RiskAssessor)
evidence_scan   → (evidence_scan,   NT-SHIELD,  RiskAssessor)
skill_evolution → (skill_evolution, NT-MIND,    KnowledgeIntegrator)
experience_bridge → (experience_bridge, NT-MEMORY, KnowledgeIntegrator)
```

### dispatch_internal_capability (real function dispatch)

Each capability tag maps to a concrete Rust function call:

- **privacy_mask**: Calls `nt_core_llm::redact_internals` — scrubs NeoTrix internal fingerprints from outbound data
- **evidence_scan**: Calls `AgenticScanner::recon_scan` — evidence-first security scan with entry point/framework detection
- **skill_evolution**: Calls `EvolutionLoop::run_cycle` — runs SEAL evolution pipeline and reports new patterns
- **experience_bridge**: Calls `KnowledgeBase::serve_core` — bridges experience nodes with knowledge retrieval

### Design Decisions

1. **R-P42 compliance**: Each route reinforces existing NT-* domain nodes (NT-SHIELD/NT-MIND/NT-MEMORY), no parallel adapter modules
2. **Egress Privacy Guard**: `privacy_mask` delegates to `nt_core_llm::redact_internals` for outbound LLM request filtering
3. **Evidence-first**: `evidence_scan` reuses `AgenticScanner` (same underlying scanner as `agentic_scan`) with emphasis on evidence-first methodology
4. **Experience bridge**: Uses `KnowledgeBase::serve_core` for hybrid retrieval (GWT intent routing + BM25 + graph traversal) rather than raw FTS

## Integration Points

- **GWT attention routing**: Routes feed into GWT salience via capability tag matching
- **CapabilityRegistry**: Routes are indexed in the capability registry for `allocate_tasks` provider lookup
- **Reflect-and-strengthen**: External gaps bud new capability nodes, internal routes reinforce existing ones
