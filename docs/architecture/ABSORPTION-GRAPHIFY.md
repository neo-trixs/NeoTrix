# NeoTrix Absorption: Graphify-Labs/graphify

> **Source**: https://github.com/Graphify-Labs/graphify (v8, 120K+ stars)
> **Absorption Date**: 2026-09-21
> **Absorbed By**: NeoTrix Development Discipline System
> **Status**: Pattern Extraction Complete

---

## Table of Contents

1. [Absorption Summary](#1-absorption-summary)
2. [Pattern Analysis](#2-pattern-analysis)
3. [NeoTrix Integration Plan](#3-neotrix-integration-plan)
4. [Concrete Implementation](#4-concrete-implementation)
5. [Verification Checklist](#5-verification-checklist)

---

## 1. Absorption Summary

### 1.1 What Graphify Does

Graphify is a **knowledge graph builder** that turns codebases, docs, SQL schemas, configs, and PDFs into a queryable knowledge graph. Key features:

- **Local-first AST parsing** via tree-sitter (no LLM for code)
- **Every edge explained** with confidence tags (EXTRACTED/INFERRED/AMBIGUOUS)
- **Pipeline architecture**: detect → extract → build → cluster → analyze → report → export
- **Security-first design** with comprehensive threat model
- **AI agent integration** as a skill for Claude Code, Cursor, Codex, etc.

### 1.2 What NeoTrix Can Absorb

| Pattern | Graphify Implementation | NeoTrix Integration Point |
|---------|------------------------|--------------------------|
| **Confidence Labels** | EXTRACTED/INFERRED/AMBIGUOUS on every edge | L0 cross-layer types, L5 decision traces |
| **Pipeline Architecture** | Each stage independent, plain dicts communication | L1 Action pipeline, L2 Perception pipeline |
| **Anti-Drift Generation** | skillgen --check in pre-commit | Architecture fitness functions |
| **Security Threat Model** | SECURITY.md with per-vector mitigations | L3 Shield threat documentation |
| **Module Responsibility Table** | ARCHITECTURE.md with entry points + I/O | All layer module documentation |
| **Testing Philosophy** | One test per module, pure unit, no side effects | Test infrastructure standard |
| **Schema Validation** | validate.py enforces extraction schema | Event/Contract validation |
| **Knowledge Graph Query** | query/path/explain commands | L2 Perception knowledge graph |

---

## 2. Pattern Analysis

### 2.1 Confidence Labels Pattern

**Graphify**: Every edge in the knowledge graph carries a confidence tag:
- `EXTRACTED` - Explicitly stated in source (import, direct call)
- `INFERRED` - Reasonable deduction (call-graph second pass, co-occurrence)
- `AMBIGUOUS` - Uncertain, flagged for human review

**NeoTrix Adaptation**: Apply confidence labels to all cross-module dependencies and decision traces.

```
NeoTrix Confidence Labels:
+--------------------------------------------------------------------+
|                                                                     |
|  EXTRACTED                                                          |
|    - Direct `use` statements between modules                        |
|    - Trait implementations                                          |
|    - Explicit function calls                                        |
|    - Event subscriptions                                             |
|                                                                     |
|  INFERRED                                                           |
|    - Indirect dependencies via shared types                         |
|    - Runtime behavior patterns                                      |
|    - Performance characteristics                                    |
|    - Security implications                                           |
|                                                                     |
|  AMBIGUOUS                                                          |
|    - Potential circular dependencies                                |
|    - Unclear ownership boundaries                                   |
|    - Platform-specific code paths                                   |
|    - Feature-gated functionality                                     |
|                                                                     |
+--------------------------------------------------------------------+
```

### 2.2 Pipeline Architecture Pattern

**Graphify**: `detect() → extract() → build() → cluster() → analyze → report → export`
- Each stage in its own module
- Communication through plain Python dicts
- No shared state, no side effects outside output directory
- Most stages are single functions

**NeoTrix Adaptation**: Apply to L1-L2-L5 pipelines.

```
NeoTrix Pipeline Standard:
+--------------------------------------------------------------------+
|                                                                     |
|  Stage 1: DETECT                                                    |
|    - Input: Raw data (files, events, requests)                      |
|    - Output: Scan summary dict                                      |
|    - Module: nt_core_detect (L0)                                    |
|                                                                     |
|  Stage 2: EXTRACT                                                   |
|    - Input: List of paths/items                                     |
|    - Output: {nodes, edges} dict                                    |
|    - Module: nt_core_extract (L1)                                   |
|                                                                     |
|  Stage 3: BUILD                                                     |
|    - Input: Extraction dict(s)                                      |
|    - Output: Graph structure                                         |
|    - Module: nt_core_graph (L1)                                     |
|                                                                     |
|  Stage 4: ANALYZE                                                   |
|    - Input: Graph structure                                          |
|    - Output: Analysis results (god nodes, cycles, etc.)             |
|    - Module: nt_core_analyze (L2)                                   |
|                                                                     |
|  Stage 5: REPORT                                                    |
|    - Input: Graph + Analysis                                         |
|    - Output: Human-readable report                                   |
|    - Module: nt_core_report (L5)                                    |
|                                                                     |
|  Stage 6: EXPORT                                                    |
|    - Input: Graph + Report                                           |
|    - Output: Target format (JSON, HTML, etc.)                       |
|    - Module: nt_core_export (L1)                                    |
|                                                                     |
+--------------------------------------------------------------------+
```

### 2.3 Anti-Drift Generation Pattern

**Graphify**: `skillgen --check` in pre-commit ensures generated skill artifacts match their source fragments. If someone hand-edits a generated file, the check fails.

**NeoTrix Adaptation**: Architecture fitness functions as anti-drift guards.

```
NeoTrix Anti-Drift Mechanism:
+--------------------------------------------------------------------+
|                                                                     |
|  Source of Truth:                                                    |
|    - Architecture definitions (ARCHITECTURE.md)                     |
|    - Module responsibility tables                                   |
|    - Layer dependency rules                                         |
|    - Quality attribute targets                                      |
|                                                                     |
|  Generated Artifacts:                                                |
|    - Architecture fitness functions                                 |
|    - Integration test assertions                                    |
|    - CI pipeline gates                                              |
|    - Documentation templates                                        |
|                                                                     |
|  Anti-Drift Check:                                                   |
|    - Pre-commit hook verifies generated artifacts match source      |
|    - CI gate fails if drift detected                                |
|    - Auto-regeneration available via `cargo nt-arch-sync`           |
|                                                                     |
+--------------------------------------------------------------------+
```

### 2.4 Security Threat Model Pattern

**Graphify**: SECURITY.md with comprehensive threat surface analysis:
- SSRF via URL fetch → validate_url() blocks private IPs
- Oversized downloads → size caps + abort
- Path traversal → validate_graph_path() resolves inside output dir
- XSS → sanitize_label() strips control chars, HTML-escapes
- Prompt injection → hash-stamped untrusted_source delimiters
- YAML frontmatter injection → escape backslashes/quotes/newlines
- Encoding crashes → errors="replace" for non-UTF-8
- Symlink traversal → followlinks=False in os.walk

**NeoTrix Adaptation**: Enhance L3 Shield with graphify-style threat documentation.

```
NeoTrix Threat Model Standard:
+--------------------------------------------------------------------+
|                                                                     |
|  For each threat vector:                                            |
|    - Vector name and description                                    |
|    - Attack scenario                                                |
|    - Mitigation implementation (code reference)                     |
|    - Verification method (test reference)                           |
|    - Residual risk assessment                                       |
|                                                                     |
|  Vectors to document:                                                |
|    - Prompt injection (direct + indirect)                           |
|    - SSRF via URL fetch                                              |
|    - Path traversal                                                  |
|    - Oversized payloads                                              |
|    - Secret leakage                                                  |
|    - Supply chain attacks                                            |
|    - Denial of service                                               |
|    - Privilege escalation                                            |
|                                                                     |
+--------------------------------------------------------------------+
```

### 2.5 Module Responsibility Table Pattern

**Graphify**: ARCHITECTURE.md with a table mapping:
- Module name
- Entry point(s) with real signatures
- Input → Output types

Plus a note: "Signatures below are the real ones - tests import every symbol named here, so this table cannot drift from the code."

**NeoTrix Adaptation**: Enforce module documentation with anti-drift.

```
NeoTrix Module Documentation Standard:
+--------------------------------------------------------------------+
|                                                                     |
|  Required for every module:                                          |
|    - Module name and layer                                           |
|    - Primary responsibility (1 sentence)                             |
|    - Public entry points (function signatures)                       |
|    - Input → Output types                                            |
|    - Dependencies (what it imports)                                  |
|    - Dependents (what imports it)                                    |
|    - Quality attributes it supports                                  |
|    - Related ADRs                                                    |
|                                                                     |
|  Enforcement:                                                        |
|    - CI test imports every documented symbol                         |
|    - Pre-commit checks documentation exists                          |
|    - Drift detection between code and docs                           |
|                                                                     |
+--------------------------------------------------------------------+
```

### 2.6 Testing Philosophy Pattern

**Graphify**: 
- One test file per module under `tests/`
- All tests are pure unit tests
- No network calls
- No filesystem side effects outside `tmp_path`
- Tests import documented symbols (anti-drift)

**NeoTrix Adaptation**: Formalize test standards per layer.

```
NeoTrix Test Standard:
+--------------------------------------------------------------------+
|                                                                     |
|  Layer L0 (Substrate):                                               |
|    - Pure unit tests only                                            |
|    - No external dependencies                                        |
|    - Property-based testing for math/logic                           |
|    - Snapshot testing for type definitions                           |
|                                                                     |
|  Layer L1 (Action):                                                  |
|    - Unit tests for each provider adapter                            |
|    - Integration tests for circuit breaker                           |
|    - Mock-based tests for external services                          |
|    - Property-based testing for serialization                        |
|                                                                     |
|  Layer L2 (Perception):                                              |
|    - Unit tests for each data source                                 |
|    - Integration tests for E8 reasoning                              |
|    - Fuzz testing for web perception                                 |
|                                                                     |
|  Layer L3 (Embodiment):                                              |
|    - Security-focused tests (penetration, injection)                 |
|    - Guard chain order tests                                         |
|    - Sandbox escape tests                                            |
|                                                                     |
|  Layer L4 (Emotion):                                                 |
|    - Memory CRUD round-trip tests                                    |
|    - Cross-session recovery tests                                    |
|    - Coverage ledger accuracy tests                                  |
|                                                                     |
|  Layer L5 (Cognition):                                               |
|    - SDB contract tests (verifier + reject)                          |
|    - GWT attention routing tests                                     |
|    - SEAL pipeline evolution tests                                   |
|                                                                     |
|  Layer L6 (Meta):                                                    |
|    - Self-healing MTTR tests                                         |
|    - Governance compliance tests                                     |
|    - Safety monitor anomaly detection tests                          |
|                                                                     |
+--------------------------------------------------------------------+
```

---

## 3. NeoTrix Integration Plan

### 3.1 New Dev Rules (R-P230-R-P245)

Based on graphify patterns, add these new development rules:

#### R-P230: Confidence Labeling for Dependencies

**Rule**: All cross-module dependencies must be labeled with confidence: EXTRACTED (direct import), INFERRED (indirect via shared types), or AMBIGUOUS (potential issue).

**Implementation**: Extend `cargo nt-dep-labels` to analyze and label all dependencies.

#### R-P231: Pipeline Stage Independence

**Rule**: Each pipeline stage must be a separate module with clear input→output contract. No shared mutable state between stages.

**Implementation**: Enforce via architecture fitness function `PipelineIndependenceFitness`.

#### R-P232: Anti-Drift Documentation

**Rule**: Module documentation (entry points, I/O types) must match code. CI test imports every documented symbol.

**Implementation**: Add `doc-drift-check` to pre-commit and CI.

#### R-P233: Security Threat Documentation

**Rule**: Every module handling external input must have a SECURITY.md section documenting threat vectors and mitigations.

**Implementation**: Add to PR template, enforce via review checklist.

#### R-P234: Schema Validation for Events

**Rule**: All events must pass schema validation before processing. Invalid events are rejected with typed error.

**Implementation**: Extend `nt_core_event` with `validate()` method.

#### R-P235: Test Per Module

**Rule**: Every module must have a corresponding test file. Tests must be pure unit tests (no network, no filesystem outside tmp_path).

**Implementation**: Add `test-coverage-check` to CI.

#### R-P236: Entry Point Documentation

**Rule**: Every public function must have documentation showing its signature, input types, and output types.

**Implementation**: Enforce via clippy lint + CI.

#### R-P237: Dependency Confidence Scoring

**Rule**: Cross-layer dependencies must have confidence scores. High-confidence (direct import) dependencies are preferred over low-confidence (inferred).

**Implementation**: Extend `cargo deny` with confidence reporting.

#### R-P238: Pipeline Communication via Plain Types

**Rule**: Pipeline stages must communicate via plain structs/enums, not trait objects or async channels.

**Implementation**: Architecture fitness function `PipelineCommunicationFitness`.

#### R-P239: Security Mitigation Verification

**Rule**: Every security mitigation must have a corresponding test that verifies it works.

**Implementation**: Add to security-scan CI workflow.

#### R-P240: Module Responsibility Enforcement

**Rule**: Every module must have a documented primary responsibility. No module may have more than 3 primary responsibilities.

**Implementation**: Add to architecture fitness functions.

---

## 4. Concrete Implementation

### 4.1 New Architecture Fitness Functions

Add these fitness functions based on graphify patterns:

```rust
// New fitness functions to add to nt_core_arch_fitness.rs

/// Verifies pipeline stage independence
pub struct PipelineIndependenceFitness;

impl SelfTest for PipelineIndependenceFitness {
    fn name(&self) -> &str { "PipelineIndependence" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Check that pipeline stages are separate modules
        // Check that no stage imports another stage's internal types
        // Check that stages communicate via plain structs
        Ok(())
    }
}

/// Verifies documentation matches code
pub struct DocDriftFitness;

impl SelfTest for DocDriftFitness {
    fn name(&self) -> &str { "DocDrift" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Import all documented symbols
        // Verify they exist in code
        // Check signature matches documentation
        Ok(())
    }
}

/// Verifies security mitigations have tests
pub struct SecurityMitigationFitness;

impl SelfTest for SecurityMitigationFitness {
    fn name(&self) -> &str { "SecurityMitigation" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Check that security mitigations have corresponding tests
        // Verify test coverage for threat vectors
        Ok(())
    }
}

/// Verifies confidence labels exist for dependencies
pub struct ConfidenceLabelFitness;

impl SelfTest for ConfidenceLabelFitness {
    fn name(&self) -> &str { "ConfidenceLabel" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Check that cross-module dependencies are labeled
        // Verify no ambiguous dependencies in critical paths
        Ok(())
    }
}
```

### 4.2 Enhanced Pre-Commit Hook

Enhance the existing pre-commit hook with graphify-inspired checks:

```bash
#!/bin/bash
# Enhanced pre-commit hook with graphify-inspired checks

set -e

echo "=== NeoTrix Pre-Commit Gates ==="

# Gate 1: Naming convention (existing)
echo "Gate 1: Naming convention..."
STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)
if [ -n "$STAGED_RS" ]; then
    for file in $STAGED_RS; do
        basename=$(basename "$file")
        if [[ "$basename" != "mod.rs" && "$basename" != "lib.rs" && "$basename" != "main.rs" ]]; then
            if [[ ! "$basename" =~ ^nt_ ]]; then
                echo "ERROR: $file does not follow nt_ prefix convention"
                exit 1
            fi
        fi
    done
fi

# Gate 2: Documentation drift check (NEW - graphify pattern)
echo "Gate 2: Documentation drift check..."
if command -v cargo &> /dev/null; then
    cargo nt-doc-drift-check || {
        echo "ERROR: Documentation drift detected"
        echo "Run: cargo nt-doc-sync to fix"
        exit 1
    }
fi

# Gate 3: Confidence label check (NEW - graphify pattern)
echo "Gate 3: Confidence label check..."
if command -v cargo &> /dev/null; then
    cargo nt-confidence-check || {
        echo "ERROR: Missing confidence labels on cross-module dependencies"
        exit 1
    }
fi

# Gate 4: Security mitigation check (NEW - graphify pattern)
echo "Gate 4: Security mitigation check..."
if command -v cargo &> /dev/null; then
    cargo nt-security-mitigation-check || {
        echo "ERROR: Security mitigations missing tests"
        exit 1
    }
fi

# Gate 5: Build gate (existing)
echo "Gate 5: Build gate..."
cargo check -p neotrix --lib || {
    echo "ERROR: Build failed"
    exit 1
fi

echo "=== All gates passed ==="
```

### 4.3 Enhanced Module Documentation Template

Based on graphify's ARCHITECTURE.md pattern:

```markdown
## Module: nt_[name]

**Layer**: L[0-6]
**Primary Responsibility**: [One sentence]

### Entry Points

| Function | Signature | Input → Output |
|----------|-----------|----------------|
| `function_name` | `fn name(arg: Type) -> Result<Output>` | `InputType → OutputType` |

### Dependencies

**Imports** (what this module uses):
- `l0_substrate::nt_core_types` - Core type definitions
- `l1_action::nt_io::provider` - LLM provider interface

**Dependents** (what uses this module):
- `l5_cognition::nt_mind` - Reasoning brain

### Quality Attributes

| Attribute | This Module's Role |
|-----------|-------------------|
| Security | Input validation, audit logging |
| Performance | Caching, lazy evaluation |
| Reliability | Error recovery, circuit breaker |

### Related ADRs

- ADR-001: [Decision title]
- ADR-002: [Decision title]

### Threat Vectors (if applicable)

| Vector | Mitigation | Test |
|--------|-----------|------|
| Prompt injection | Input sanitization | test_injection.rs |
| Path traversal | Path validation | test_traversal.rs |
```

### 4.4 Confidence Label Implementation

```rust
// Add to nt_core_cross_layer.rs

/// Confidence level for cross-module relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Confidence {
    /// Relationship is explicitly stated in source
    /// (direct import, trait impl, function call)
    Extracted,
    
    /// Relationship is a reasonable deduction
    /// (indirect via shared types, runtime patterns)
    Inferred,
    
    /// Relationship is uncertain, flagged for review
    /// (potential circular deps, unclear ownership)
    Ambiguous,
}

/// A labeled dependency between modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledDependency {
    pub source: ModulePath,
    pub target: ModulePath,
    pub relationship: DependencyRelationship,
    pub confidence: Confidence,
    pub location: SourceLocation,
    pub rationale: Option<String>,
}

/// Analyze and label all cross-module dependencies
pub fn analyze_dependencies() -> Vec<LabeledDependency> {
    // Implementation: parse source, analyze imports, assign confidence
    todo!()
}

/// Check for ambiguous dependencies in critical paths
pub fn check_critical_path_ambiguity() -> Result<(), Vec<LabeledDependency>> {
    // Implementation: find ambiguous deps in hot paths
    todo!()
}
```

---

## 5. Verification Checklist

### 5.1 Absorption Completeness

| Pattern | Absorbed | NeoTrix Integration | Status |
|---------|----------|--------------------|--------| 
| Confidence Labels | Yes | L0 cross-layer types | IMPLEMENT |
| Pipeline Architecture | Yes | L1-L2-L5 pipelines | IMPLEMENT |
| Anti-Drift Generation | Yes | Architecture fitness functions | IMPLEMENT |
| Security Threat Model | Yes | L3 Shield documentation | IMPLEMENT |
| Module Responsibility Table | Yes | All module documentation | IMPLEMENT |
| Testing Philosophy | Yes | Test standards per layer | IMPLEMENT |
| Schema Validation | Yes | Event validation | IMPLEMENT |
| Entry Point Documentation | Yes | Public function docs | IMPLEMENT |

### 5.2 New Dev Rules to Add

| Rule | Description | Priority |
|------|-------------|----------|
| R-P230 | Confidence labeling for dependencies | HIGH |
| R-P231 | Pipeline stage independence | HIGH |
| R-P232 | Anti-drift documentation | HIGH |
| R-P233 | Security threat documentation | HIGH |
| R-P234 | Schema validation for events | MEDIUM |
| R-P235 | Test per module | HIGH |
| R-P236 | Entry point documentation | MEDIUM |
| R-P237 | Dependency confidence scoring | MEDIUM |
| R-P238 | Pipeline communication via plain types | MEDIUM |
| R-P239 | Security mitigation verification | HIGH |
| R-P240 | Module responsibility enforcement | MEDIUM |

### 5.3 New Fitness Functions to Add

| Function | Layer | What It Checks | Priority |
|----------|-------|---------------|----------|
| PipelineIndependenceFitness | L1-L5 | Stages are independent modules | HIGH |
| DocDriftFitness | All | Documentation matches code | HIGH |
| SecurityMitigationFitness | L3 | Security mitigations have tests | HIGH |
| ConfidenceLabelFitness | L0 | Dependencies are labeled | MEDIUM |
| ModuleResponsibilityFitness | All | Modules have clear responsibility | MEDIUM |

### 5.4 CI/CD Enhancements

| Enhancement | What It Does | Priority |
|-------------|-------------|----------|
| doc-drift-check | Verifies documentation matches code | HIGH |
| confidence-check | Verifies dependencies are labeled | MEDIUM |
| security-mitigation-check | Verifies security tests exist | HIGH |
| pipeline-independence-check | Verifies pipeline stages are independent | MEDIUM |

---

*End of Absorption Document*
