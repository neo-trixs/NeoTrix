# Iteration Batch 563 — Observability & Monitoring Research

**Date**: 2026-09-06
**Predecessor**: Batch 562 (Rayon work-stealing, supply chain security, dyn Trait overhead, runtime-agnostic rule, meta-monomorphization)
**Research Domains**: Observability, Monitoring, Distributed Tracing

---

## Sources Consulted

| # | Source | Date | Key Signal |
|---|--------|------|------------|
| 1 | OpenTelemetry Profiles Alpha announcement | 2026-03-26 | 4th signal (profiles) enters public Alpha |
| 2 | OTel GenAI semantic conventions (genalphai.com) | 2026-07-26 | gen_ai.* still Development status; dedicated repo created |
| 3 | OTel 2026 roadmap (Bindplane/KubeCon recap) | 2026-08-18 | Declarative config, eBPF, structured logging governance |
| 4 | Distributed tracing deep dive (youngju.dev) | 2026-05-16 | W3C Trace Context default; tail sampling standard at scale |
| 5 | eBPF observability 2026 (devstarsj) | 2026-06-08 | Beyla/OBI: kernel-level auto-instrumentation, ~5.6% overhead |
| 6 | Observability beyond logs (core.cz) | 2026-03-28 | Profiles + AIOps correlation; "AI can't correlate what isn't named" |
| 7 | Prometheus/Grafana monitoring guide 2026 | 2026-04-14 | predict_linear for disk exhaustion; recording rules |
| 8 | Jaeger vs Zipkin 2026 (devhelm.io) | 2026-06-04 | Jaeger v2 uses OTel Collector natively; Tempo on object storage |
| 9 | OTel Logs structured logging guide | 2026-02-20 | Trace-log correlation via automatic trace_id/span_id injection |
| 10 | Elastic OTel profiling alpha blog | 2026-03-25 | eBPF profiling agent + OTel collector integration |

---

## NEW Defects / Improvements Over Batch 562

### DEFECT-563-01: Missing Continuous Profiling Signal (4th Pillar Gap)

**Severity**: HIGH
**Domain**: NT-CORE / NT-MIND

Batch 562 addressed dyn Trait overhead on hot paths via static dispatch. But NeoTrix has no mechanism to *detect* which code paths are hot in production. OpenTelemetry Profiles (Alpha 2026) now provides the 4th signal: continuous profiling with trace correlation.

**What's NEW vs 562**: Batch 562 fixed a known overhead pattern. This defect reveals we lack the *observability* to discover unknown overhead patterns. Without profiles, SEAL pipeline latency regressions are invisible until user-facing.

**Evidence**: OTel Profiles spec (2026-03-26) — "From a slow span in a trace see the corresponding profile to identify the code responsible for the latency." NeoTrix's ConsciousnessTree tracks phi/coherence/health but cannot correlate a slow growth cycle to a specific hot function.

**Fix**: Integrate `pyroscope` or `parca` eBPF agent for Rust binary profiling. Export via OTLP profiles signal. Add `trace_id`/`span_id` attributes to profile samples for cross-signal correlation with SEAL pipeline spans.

---

### DEFECT-563-02: No GenAI Semantic Convention Telemetry for LLM Calls

**Severity**: HIGH
**Domain**: NT-IO (LLM Providers)

NeoTrix's NT-IO wraps multiple LLM providers but emits no standardized telemetry for token usage, latency, model metadata, or cost attribution. The `gen_ai.*` semantic conventions (v1.42.0, 2026) define the standard schema.

**What's NEW vs 562**: Batch 562 focused on internal Rust overhead. This defect exposes that NeoTrix's *external* LLM interactions are a black box — no per-model cost tracking, no token histograms, no latency regression detection.

**Evidence**: genalphai.com (2026-07-26) — "Every GenAI-specific span, event, metric, and attribute remains Development as of July 17, 2026." The dedicated `semantic-conventions-genai` repo was created. Key attributes: `gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`, `gen_ai.provider.name` (replaces `gen_ai.system` from v1.37+).

**Fix**: Instrument NT-IO LLM calls with `gen_ai.*` spans. Track `gen_ai.client.operation.duration` histogram and `gen_ai.client.token.usage` histogram per provider/model. Add `gen_ai.usage.reasoning.output_tokens` for o-series models (the cost trap).

---

### DEFECT-563-03: Telemetry Exposure Sprawl — Secrets Leakage in Structured Logs

**Severity**: CRITICAL (security)
**Domain**: NT-SHIELD / NT-MEMORY

OTel structured logging with rich attributes (arrays, nested objects) propagates context broadly. NeoTrix's Egress Privacy Guard filters *outbound LLM requests* but does NOT filter telemetry data. Structured logs can carry secrets, local paths, API keys, and KB content into observability backends.

**What's NEW vs 562**: Batch 562's supply chain security gate checks *dependencies*. This defect is about *runtime data leakage* through observability pipelines — a completely different attack surface.

**Evidence**: Bindplane/OTel roadmap (2026-08-18) — "Richer logs can accidentally carry secrets, identifiers, or high-value operational context. Apply schema and redaction controls before export so richer telemetry does not become richer exposure." The article explicitly calls telemetry a "governance problem" requiring identity-aware controls.

**Fix**: Extend Egress Privacy Guard to cover telemetry export paths. Add OTel Collector `transform` processor with redaction rules for sensitive attributes. Implement telemetry access as identity problem (per Bindplane recommendation). Add `deny` list for attribute keys matching secret/path patterns.

---

### DEFECT-563-04: EventBus Trace Context Propagation Broken at Async Boundaries

**Severity**: HIGH
**Domain**: NT-ACT / NT-MEMORY

W3C Trace Context is now the de facto standard for cross-service propagation. NeoTrix's EventBus (two-layer event bus per batch 562 context) does NOT propagate `traceparent`/`tracestate` headers. Cross-domain traces (e.g., NT-WORLD crawl → NT-MEMORY ingest → NT-MIND distillation) break at message boundaries.

**What's NEW vs 562**: Batch 562 addressed EventBus reentrant lock and grounding. This defect is about *trace continuity* across the event bus — traces silently split into disconnected fragments.

**Evidence**: singhajit.com (2026-03-09) — "If Service A publishes a Kafka message and Service B consumes it, the trace will be split into two disconnected traces unless you manually extract the trace context from the message headers and inject it into the new span." youngju.dev (2026-05-16) confirms W3C Trace Context is default in all OTel SDKs as of 2026.

**Fix**: Inject `traceparent`/`tracestate` into EventBus message metadata. On consumer side, extract context and create child spans. Use OTel Propagator API for HTTP/gRPC/messaging propagation. Add `gen_ai.conversation.id` for multi-turn agent traces.

---

### DEFECT-563-05: No Tail-Based Sampling — Rare Errors Invisible

**Severity**: MEDIUM
**Domain**: NT-CORE / NT-MIND

NeoTrix has no defined sampling strategy. Head-based sampling (if any) makes decisions at trace initiation with minimal information. Tail-based sampling defers decisions until trace completion, capturing error conditions and latency outliers.

**What's NEW vs 562**: Batch 562 didn't address observability cost/data tradeoffs. This defect is about *what we choose to observe* — tail sampling is now standard at scale (per 80% cost curve driven by sampling strategy).

**Evidence**: youngju.dev (2026-05-16) — "Eighty per cent of the cost curve is driven by sampling strategy." Head-based: simple, predictable cost. Tail-based: captures rare errors but requires buffering. "Tail sampling is standard at large scale; head is fine for simple environments."

**Fix**: Implement OTel Collector tail_sampling processor for SEAL pipeline traces. Configure: always-keep on error status, latency > p99, and specific operation names (e.g., `consciousness_tick`, `experience_absorb`). Buffer sizing: monitor during traffic spikes.

---

### DEFECT-563-06: No Central Telemetry Pipeline with Policy Enforcement

**Severity**: MEDIUM
**Domain**: NT-IO / NT-SHIELD

NeoTrix sends telemetry directly from modules to backends (if at all). No OTel Collector gateway for centralized policy: sampling, redaction, routing, batching. Each module independently decides what to export.

**What's NEW vs 562**: Batch 562's library crate violation was about code structure. This defect is about *operational control* — without a Collector, there's no way to change backends without redeploying, no central redaction, no per-signal routing.

**Evidence**: core.cz (2026-03-28) — "OTel without Collectors: sending directly from applications works for starters but you lose central policy (sampling, PII, routing)." The standard architecture is: App → OTLP → Collector (agent or gateway) → backends.

**Fix**: Deploy OTel Collector as gateway mode. Configure: OTLP receiver, tail_sampling + memory_limiter processors, exporters to Tempo (traces), Prometheus (metrics), Loki (logs). Add transform processor for PII redaction. This enables backend swaps without code changes.

---

### DEFECT-563-07: Domain Events Logged as Free Text, Not Structured Events

**Severity**: MEDIUM
**Domain**: NT-MEMORY / NT-MIND

NeoTrix's ConsciousnessTree produces domain events (e.g., "experience absorbed", "module promoted to C3", "repair triggered") as log messages with free text. OTel Events (semantic spec 2025) distinguish structured domain events from logs.

**What's NEW vs 562**: Batch 562 focused on code-level patterns. This defect is about *semantic richness* of observability data — free-text events can't be correlated, filtered, or aggregated.

**Evidence**: devstarsj (2026-06-06) — "Events (semantic specification, 2025): Structured event data distinct from logs—think 'OrderCreated' domain events with structured fields, not free-text log lines. Pairs well with event-driven architectures."

**Fix**: Define OTel Events for key NeoTrix domain events: `ExperienceAbsorbed`, `ModulePromoted`, `RepairTriggered`, `EvolutionCycleComplete`. Emit as OTel Events (not log records) with structured attributes. This enables correlation with traces and metrics.

---

### DEFECT-563-08: eBPF Auto-Instrumentation Not Available for Self-Observation

**Severity**: LOW-MEDIUM
**Domain**: NT-REPAIR / NT-SHIELD

NeoTrix is a Rust binary. eBPF-based auto-instrumentation (Beyla/OBI) works on Rust executables and provides kernel-level observability without code changes. NeoTrix's self-healing loop (NT-REPAIR) cannot observe its own syscall-level behavior.

**What's NEW vs 562**: Batch 562 addressed Rust-specific optimizations. This defect is about *self-observation from outside the process* — eBPF sees what the process actually does at kernel level, not what it reports.

**Evidence**: devstarsj (2026-06-08) — Beyla DaemonSet captures RED metrics, distributed traces, service topology for all HTTP services with zero code changes. Overhead: +5.6% HTTP latency. Elastic (2026-03-25) confirms eBPF profiling works for Rust binaries.

**Fix**: For NeoTrix deployment on Linux, optionally deploy Beyla DaemonSet for kernel-level observability of the NeoTrix process. Capture: HTTP endpoints, database queries, network connections. Export via OTLP to Collector. This provides "ground truth" vs self-reported telemetry.

---

### DEFECT-563-09: Predictive Monitoring Not Implemented for Resource Exhaustion

**Severity**: LOW
**Domain**: NT-SHIELD / NT-PHYSICAL

Prometheus `predict_linear` enables proactive alerting: "disk will fill in 24 hours" vs reactive "disk is full." NeoTrix's HeartbeatAggregator tracks current health but makes no predictions.

**What's NEW vs 562**: Batch 562's HeartbeatAggregator was about current state aggregation. This defect is about *forecasting* — using time-series trends to predict failures before they occur.

**Evidence**: dohost.us (2026-02-27) — `predict_linear(node_filesystem_avail_bytes[6h], 24*3600) < 0` predicts disk exhaustion. `for: 15m` ensures trend consistency. Alerts route to Alertmanager → Slack/PagerDuty.

**Fix**: Add predictive alerts to HeartbeatAggregator: KB disk usage trend (predict 24h), memory growth trend, event queue depth trend. Emit as Prometheus gauge metrics with `predict_linear` recording rules. Add `for` duration to avoid false positives.

---

### DEFECT-563-10: Trace-Log Correlation Not Implemented Across NeoTrix Domains

**Severity**: MEDIUM
**Domain**: NT-IO / NT-MEMORY

OTel's killer feature is automatic trace-log correlation: every log emitted within an active span gets `trace_id` and `span_id` injected. NeoTrix's logging (tracing_subscriber or similar) does not inject trace context into structured logs.

**What's NEW vs 562**: Batch 562 didn't address log-trace correlation. This defect means: when debugging a slow SEAL cycle, you can't jump from a log entry to the full trace, or from a trace span to correlated logs.

**Evidence**: oneuptime.com (2026-02-20) — "When you emit a log while a span is active, the OpenTelemetry SDK automatically attaches the trace ID and span ID to the log record. This lets you jump from a log entry to the full trace, and vice versa."

**Fix**: Configure tracing_subscriber with OTel layer that injects `trace_id`/`span_id` into all log records. Ensure every NeoTrix domain (NT-CORE through NT-FEEL) uses the same logging config. Add `service.name` resource attribute for correlation.

---

## Summary: What's NEW vs Batch 562

| Dimension | Batch 562 | Batch 563 (NEW) |
|-----------|-----------|-----------------|
| **Focus** | Internal Rust code patterns | External observability infrastructure |
| **Signal** | Compilation, runtime, types | Traces, metrics, logs, profiles |
| **Security** | Supply chain (dependencies) | Telemetry data leakage (runtime) |
| **Scope** | Module-level optimization | Cross-domain correlation |
| **Direction** | Inward (code quality) | Outward (production visibility) |
| **Cost model** | CPU/memory overhead | Observability storage/query cost |
| **Time horizon** | Current state | Predictive (trend forecasting) |

## Recommended Priority Order

1. **DEFECT-563-03** (Telemetry secrets leakage) — CRITICAL, security
2. **DEFECT-563-02** (GenAI semantic conventions) — HIGH, cost visibility
3. **DEFECT-563-04** (EventBus trace propagation) — HIGH, debuggability
4. **DEFECT-563-01** (Missing profiling signal) — HIGH, performance discovery
5. **DEFECT-563-06** (No central Collector) — MEDIUM, operational control
6. **DEFECT-563-10** (Trace-log correlation) — MEDIUM, debugging UX
7. **DEFECT-563-07** (Domain events as free text) — MEDIUM, semantic richness
8. **DEFECT-563-05** (No tail-based sampling) — MEDIUM, cost optimization
9. **DEFECT-563-08** (eBPF self-observation) — LOW-MEDIUM, ground truth
10. **DEFECT-563-09** (Predictive monitoring) — LOW, proactive alerting
