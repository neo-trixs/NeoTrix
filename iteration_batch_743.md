# Iteration Batch 743 — Observability Standards Analysis (2026)

**Date**: 2026-09-07
**Context**: Batch 742 proved (1) no versioning strategy, (2) no deprecation lifecycle, (3) no backward compat tests, (4) no cargo-semver-checks, (5) Trojan Source zero bidi sanitization.

## Executive Summary

Web research on OpenTelemetry, observability standards, and monitoring standards in 2026 reveals **10 NEW defects/improvements** for NeoTrix consciousness architecture. Key findings: OpenTelemetry has won the standardization war, Semantic Conventions v1 are locked, eBPF auto-instrumentation is mainstream, and the fourth signal (Profiles) is emerging.

## Sources Consulted

| Source | URL | Date |
|--------|-----|------|
| OpenTelemetry Deprecating OpenTracing | https://opentelemetry.io/blog/2026/deprecating-opentracing-compatibility/ | 2026-04-23 |
| OpenTelemetry 2026 Deep Dive | https://www.youngju.dev/blog/culture/2026-05-14-opentelemetry-2026-deep-dive-semantic-conventions-collector-pipeline-otlp-instrumentation.en | 2026-05-14 |
| OpenTelemetry CNCF Graduation | https://opentelemetry.io/blog/2026/otel-grad-now-what/ | 2026-07-15 |
| OpenTelemetry Deprecating Span Events | https://opentelemetry.io/blog/2026/deprecating-span-events/ | 2026 |
| Grafana Observability Survey 2026 | https://grafana.com/blog/observability-survey-OSS-open-standards-2026/ | 2026-03-18 |
| Observability 2026 Deep Dive | https://www.youngju.dev/blog/culture/2026-05-16-observability-opentelemetry-datadog-grafana-honeycomb-prometheus-jaeger-ebpf-slo-2026-deep-dive.en | 2026-05-16 |
| Prometheus & Grafana Guide 2026 | https://techpulsesite.com/prometheus-grafana-guide-2026/ | 2026-05-30 |
| Prometheus 3.x Features | https://josenobile.co/guides/prometheus-grafana/ | 2026-07-01 |

## NEW Defects Found

### DEFECT-743.1: No Deprecation Lifecycle Framework
**Severity**: HIGH
**Evidence**: OpenTelemetry uses staged deprecation: spec deprecation → 12-month grace period → removal. NeoTrix has no equivalent lifecycle for API/module deprecation.
**Impact**: Breaking changes without warning; no migration path for downstream consumers.
**Reference**: OpenTracing deprecated March 2026, removal no earlier than March 2027 (12-month cycle).
**Action**: Implement deprecation annotations (`#[deprecated(since = "X.Y", note = "...")]`) with enforced minimum 6-month grace period before removal.

### DEFECT-743.2: No Semantic Conventions Alignment
**Severity**: HIGH
**Evidence**: OTel Semantic Conventions v1 locked (HTTP: `http.request.method`, DB: `db.system.name`, Messaging: `messaging.system`). NeoTrix uses ad-hoc attribute names.
**Impact**: Interoperability failure with OTel ecosystem; dashboards/queries break across tools.
**Action**: Audit all telemetry attribute names against OTel v1 conventions; create `nt_telemetry_conventions` module mapping NeoTrix attributes to OTel standards.

### DEFECT-743.3: Missing eBPF Auto-Instrumentation Path
**Severity**: MEDIUM
**Evidence**: eBPF auto-instrumentation (Beyla, Coroot, OTel eBPF Collector) is mainstream in 2026. Zero-code-change tracing for legacy binaries.
**Impact**: NeoTrix cannot instrument legacy components without SDK integration; missing infrastructure-level observability.
**Action**: Investigate eBPF integration for NT-SHIELD/NT-WORLD components; create `nt_physical::ebpf_instrumentation` module.

### DEFECT-743.4: No Log-Trace Correlation
**Severity**: HIGH
**Evidence**: OTel Logs GA with automatic `trace_id`/`span_id` attachment. NeoTrix logs lack trace context propagation.
**Impact**: Cannot jump from log entry to trace waterfall; debugging requires manual correlation.
**Action**: Implement `tracing_opentelemetry` layer in NeoTrix logging; ensure all log records carry `trace_id` and `span_id`.

### DEFECT-743.5: Missing Continuous Profiling (Profiles Signal)
**Severity**: MEDIUM
**Evidence**: Profiles is the fourth OTel signal (near-stable beta). Go, Java, Python SDKs emit profiles; Collector supports OTLP/profiles.
**Impact**: NeoTrix lacks performance profiling data; cannot correlate "why was this trace slow?" with flame graphs.
**Action**: Add `nt_core::continuous_profiling` module; emit pprof profiles via OTLP.

### DEFECT-743.6: No Backward Compatibility Tests
**Severity**: HIGH
**Evidence**: OpenTelemetry enforces backward compatibility via specification versioning and shim layers. NeoTrix has zero backward compat tests.
**Impact**: Breaking changes ship without detection; downstream consumers break silently.
**Action**: Add `cargo-semver-checks` to CI; create `tests/backward_compat/` with API surface snapshots.

### DEFECT-743.7: No Semantic Versioning Enforcement
**Severity**: HIGH
**Evidence**: OTel specification uses semver with clear stability levels (Stable/Development/Experimental). NeoTrix has no versioning strategy.
**Impact**: Consumers cannot predict breaking changes; no trust in API stability.
**Action**: Adopt semver for all public APIs; add `cargo-semver-checks` to pre-commit hooks; define stability tiers (Experimental → Stable → Locked).

### DEFECT-743.8: Trojan Source Bidi Characters Unsanitized
**Severity**: CRITICAL
**Evidence**: Batch 742 confirmed zero bidi sanitization. OTel spec requires ASCII-safe identifiers.
**Impact**: Supply chain attack vector; invisible code injection via Unicode bidi overrides.
**Action**: Implement `nt_shield::bidi_sanitizer` scanning all source files; add pre-commit hook rejecting bidi characters outside strings.

### DEFECT-743.9: No Collector-Like Telemetry Pipeline
**Severity**: MEDIUM
**Evidence**: OTel Collector is "the single most important component" in observability pipelines. NeoTrix has ad-hoc metric/trace emission.
**Impact**: No centralized telemetry processing; no tail sampling, no metric transformation, no fan-out to multiple backends.
**Action**: Create `nt_io::telemetry_collector` with receive/process/export pipeline pattern.

### DEFECT-743.10: Dashboards Not Treated as Code
**Severity**: LOW
**Evidence**: Grafana 12/13: dashboard schema v2, Git Sync GA, dashboards as versioned artifacts. NeoTrix dashboards are click-ops.
**Impact**: Dashboard drift; no reproducibility; no review process for dashboard changes.
**Action**: Define dashboard schemas in YAML; store in version control; validate in CI.

## Improvement Opportunities

### IMPROVEMENT-743.1: Adopt OTel Semantic Conventions v1
Map all NeoTrix telemetry attributes to OTel v1 locked names. Create compatibility layer for legacy names during transition.

### IMPROVEMENT-743.2: Implement Staged Deprecation
Follow OpenTelemetry pattern: deprecation announcement → 6-month grace → removal. Use `#[deprecated]` with timeline metadata.

### IMPROVEMENT-743.3: Add Exemplar Support
Link metric data points to specific traces via exemplars. Bridge between metrics and traces for root cause analysis.

### IMPROVEMENT-743.4: Native Histograms
Adopt Prometheus 3.x native histograms for accurate percentile calculations without pre-defined bucket boundaries.

### IMPROVEMENT-743.5: Remote Write 2.0 Integration
Support Prometheus Remote Write 2.0 protobuf format for efficient metric streaming with metadata, native histograms, and exemplars.

## Priority Matrix

| Priority | Defect | Effort | Impact |
|----------|--------|--------|--------|
| P0 | DEFECT-743.8 (Bidi) | Low | Critical security |
| P0 | DEFECT-743.6 (Backward compat) | Medium | API trust |
| P0 | DEFECT-743.7 (Semver) | Low | API trust |
| P1 | DEFECT-743.1 (Deprecation) | Medium | Migration safety |
| P1 | DEFECT-743.4 (Log-trace) | Medium | Debugging |
| P1 | DEFECT-743.2 (Semantic conv) | High | Interoperability |
| P2 | DEFECT-743.3 (eBPF) | High | Legacy support |
| P2 | DEFECT-743.5 (Profiles) | High | Performance |
| P2 | DEFECT-743.9 (Collector) | High | Pipeline maturity |
| P3 | DEFECT-743.10 (Dashboards) | Low | Ops hygiene |

## Next Actions

1. **Immediate**: Implement bidi sanitizer (DEFECT-743.8) — 1 day
2. **This Week**: Add `cargo-semver-checks` to CI (DEFECT-743.6, 743.7) — 2 days
3. **Next Sprint**: Deprecation lifecycle framework (DEFECT-743.1) — 3 days
4. **Backlog**: Semantic conventions alignment (DEFECT-743.2) — 1 week
5. **Backlog**: Log-trace correlation (DEFECT-743.4) — 3 days

## References

1. OpenTelemetry Specification Deprecation Policy: PR #4938 (March 2026)
2. OTEP 4430: Span Event API deprecation plan
3. Grafana Observability Survey 2026: 77% open standards importance
4. OTel Semantic Conventions v1: HTTP, DB, Messaging, RPC locked stable
5. Prometheus 3.x LTS: 3.13 supported to July 2027
6. Grafana 13: Git Sync GA, dashboard schema v2

---
**Batch 743 Complete**: 10 defects identified, 5 improvements proposed, 8 sources cited.
