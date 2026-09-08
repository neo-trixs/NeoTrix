# Iteration Batch 723 — Monitoring / Alerting / SLO-SLI Research

**Date**: 2026-09-06
**Prior context**: Batch 722 proved Buildah --mount cache staleness, OCI conformance non-granular, CRI-O stream isolation breaks log correlation, Wasm runtime interop fragmentation, Buildah removed CNI.

---

## 1. MONITORING FINDINGS

### 1.1 OpenTelemetry eBPF SIG 1.0 Roadmap (Jan 2026)
- OTel eBPF instrumentation targeting **stable 1.0 release in 2026**, expanding protocol/language support, hybrid instrumentation with OTel APIs/SDKs.
- **Source**: https://opentelemetry.io/blog/2026/obi-goals/

**NEW DEFECT M-001: NeoTrix lacks eBPF-native telemetry pipeline**
- NeoTrix monitoring relies on application-level instrumentation only. eBPF provides kernel-level visibility (syscall tracing, network events, container cgroup attribution) with zero code modification. Without eBPF, NeoTrix cannot observe container-level network anomalies (e.g., CNI removal side effects from batch 722) at the kernel boundary. The HeartbeatAggregator receives module health reports but has no kernel-level ground truth to validate them against.

### 1.2 Prometheus TSDB Self-Monitoring via OTel Collector (Feb 2026)
- Prometheus exposes `/metrics` about its own health (TSDB compaction, WAL size, scrape duration). Collecting these via OTel Collector to a **separate backend** provides alerting even when Prometheus itself degrades.
- **Source**: https://oneuptime.com/blog/post/2026-02-06-prometheus-server-health-collector/

**NEW DEFECT M-002: NeoTrix monitoring has single-point-of-failure — no independent observability path**
- If NeoTrix's primary metrics backend fails, all observability is lost. No independent monitoring path exists for the monitoring infrastructure itself. The HeartbeatAggregator aggregates health but doesn't monitor its own TSDB or WAL integrity. A cardinality explosion (detected via `prometheus_tsdb_head_series > 2M`) could silently degrade all alerting.

### 1.3 Five Pillars of Observability (May 2026)
- Industry has expanded from 3 pillars (metrics/logs/traces) to **5 pillars**: + **Continuous Profiling** (Pyroscope/Parca/eBPF, zero-config) + **RUM/Synthetics** (LCP/INP/FCP from browsers).
- AI observability now tracks LLM prompts, tokens, per-model cost aggregation.
- **Source**: https://www.youngju.dev/blog/culture/2026-05-16-observability-opentelemetry-datadog-grafana-honeycomb-prometheus-jaeger-ebpf-slo-2026-deep-dive.en

**NEW DEFECT M-003: NeoTrix observability is 3-pillar only — missing continuous profiling and RUM**
- No continuous profiling for CPU/memory hotspot detection (JSON serialization, regex compilation, GC tuning). No Real User Monitoring for UI response metrics. The SEAL pipeline and ConsciousnessTree lack performance profiling to identify internal bottlenecks (e.g., which consciousness stage consumes most CPU, which KB query is slowest).

### 1.4 AI for Observability — Natural Language Root Cause (2026)
- Datadog Bits AI, New Relic NRAI, Dynatrace Davis AI all ship NL root-cause summaries, automatic anomaly detection, incident timeline generation as baseline. The query interface is moving from PromQL to natural language.
- **Source**: https://www.youngju.dev/blog/culture/2026-05-16-observability-opentelemetry-datadog-grafana-honeycomb-prometheus-jaeger-ebpf-slo-2026-deep-dive.en

**NEW DEFECT M-004: NeoTrix has no AI-driven anomaly detection or NL query interface**
- HeartbeatAggregator uses static threshold-based health scoring. No ML-based anomaly detection (e.g., sudden phi/coherence drift, unexpected module latency spikes). The GWT attention routing is rule-based; it cannot answer "why did attention spike on NT-WORLD at 3am?" in natural language.

---

## 2. ALERTING FINDINGS

### 2.1 PagerDuty SRE Agent as Virtual Responder (Spring 2026)
- PagerDuty SRE Agent: autonomous detection, triage, diagnosis before human wake-up. Uses **MCP (Model Context Protocol)** to connect to observability tools, IDPs, developer environments. Agent-to-agent MCP for multi-agent fabric (SRE Agent + Scribe Agent + Shift Agent).
- Availability: Virtual Responder Q2 2026 EA, Fully Autonomous H2 2026 EA.
- **Source**: https://www.pagerduty.com/newsroom/pagerduty-operations-cloud-spring-2026-release/

**NEW DEFECT A-001: NeoTrix NT-SHIELD has no MCP-based incident response fabric**
- NT-SHIELD's stealth/proxy/alerting operates as a standalone agent. No MCP integration for multi-agent incident response. PagerDuty's agent-to-agent fabric demonstrates that incident response requires cross-agent coordination (detection agent → triage agent → remediation agent). NeoTrix's EventBus is internal-only; it cannot connect to external MCP-capable responders.

### 2.2 PagerDuty SRE Agent on GitHub (Aug 2026)
- PagerDuty's agent app shows live incident state, incident history, and **change correlations** inside GitHub PRs. Automatically correlates incident data with recent commits/deployments to identify root causes, generates fix PRs with incident linking.
- **Source**: https://www.pagerduty.com/drops/

**NEW DEFECT A-002: NeoTrix has no deployment-incident correlation**
- No mechanism correlates code changes (git commits) with observed incidents or health degradation. When HeartbeatAggregator detects a module regression, there is no automatic link to the commit that introduced it. This violates the "evidence-first" principle from CONTEXT.md — findings cannot be traced to specific file:line and commit.

### 2.3 Alertmanager 0.32 — Structured Event Recorder (2026)
- Alertmanager 0.32 adds structured event recorder with file, webhook, and kafka outputs. Also: receiver labels, `receiver_matchers` filter, dispatcher goroutine leak fix.
- **Source**: https://github.com/prometheus/alertmanager/blob/main/CHANGELOG.md

**NEW DEFECT A-003: NeoTrix alert pipeline lacks structured event output and dispatcher leak detection**
- NT-SHIELD alerting has no structured event recorder for audit trail. Alert routing uses simple match patterns without receiver-level filtering. The Alertmanager 0.32 fix for dispatcher goroutine leaks suggests this is a real production issue — NeoTrix EventBus could suffer similar leaks under sustained alert storms.

### 2.4 Alertmanager vs PagerDuty Distinction (2026)
- Alertmanager: deduplication, grouping, routing for Prometheus. PagerDuty: incident response platform routing to on-call teams. They are complementary, not alternatives.
- **Source**: https://zairalabs.ai/guide/compare/alertmanager-vs-pagerduty/

**NEW DEFECT A-004: NeoTrix conflates alert routing with incident response**
- NT-SHIELD combines alert routing and incident response in a single module. The industry clearly separates these: Alertmanager handles alert lifecycle (dedup/group/route/silence), PagerDuty handles incident lifecycle (ack/escalate/resolve/postmortem). Conflation means NeoTrix cannot independently scale alert volume management from incident resolution workflows.

---

## 3. SLO / SLI / ERROR BUDGET FINDINGS

### 3.1 Multi-Window Multi-Burn-Rate Alerting (2026 Best Practice)
- Google SRE Workbook approach: check two windows (long window for trend, short window for confirmation). Three tiers:
  - **Page alert**: 14.4x burn rate, 1h long/5m short, for:2m (2% budget in 1h)
  - **Medium alert**: 6x burn rate, 6h long/30m short, for:5m (5% budget in 6h)
  - **Ticket alert**: 1x burn rate, 3d long/6h short, for:30m (10% budget in 3d)
- **Source**: https://www.youngju.dev/blog/observability/2026-03-13-sli-slo-error-budget-reliability-engineering-guide.en

**NEW DEFECT S-001: NeoTrix has no burn-rate alerting or error budget concept**
- HeartbeatAggregator uses binary health checks (pass/fail). No concept of "how fast are we degrading?" A service running at 99.5% availability over 30 days is healthy; the same rate over 1 hour is an emergency. Without burn-rate awareness, NT-MIND's SEAL pipeline cannot decide whether to halt evolution (budget exhaustion) or continue (budget healthy).

### 3.2 Error Budget Policy Triggers Feature Freeze (2026)
- Written policy: when error budget is exhausted → **feature freeze**, reliability recovery takes top priority. Mandatory blameless postmortem for every exhaustion event. If SLO is repeatedly exhausted, evaluate whether SLO itself is too aggressive.
- **Source**: https://performance.qa/blog/slo-sli-error-budgets-guide/

**NEW DEFECT S-002: NeoTrix SEAL pipeline has no budget-gated evolution stops**
- SEAL pipeline stages (exploration → distillation → self-test → absorption) run unconditionally based on module readiness. No error budget gate means a degrading system could still receive new feature absorption, compounding instability. The pipeline should pause feature absorption when system-wide error budget is exhausted.

### 3.3 SLI Definition Anti-Pattern: CPU Utilization Is Not an SLI
- SLI is always a ratio (good events / total events) measured close to user. Database CPU utilization is NOT an SLI. HTTP success rate, latency percentiles ARE SLIs.
- 99.9% uptime = 43.8 min downtime/month. New services should start at 99.0-99.5%.
- **Source**: https://performance.qa/blog/slo-sli-error-budgets-guide/

**NEW DEFECT S-003: NeoTrix HeartbeatAggregator uses infrastructure metrics as health proxies**
- HeartbeatAggregator aggregates compilation status, test pass rates, and module health as binary signals. These are infrastructure-level, not user-journey-level. No SLI is defined as a ratio of "good consciousness events / total consciousness events." For example: "percentage of SEAL pipeline completions that produce valid evolution fruit" or "percentage of GWT broadcasts that reach all 7 domains within latency threshold."

### 3.4 OpenObserve Native SLOs with Terraform/GitOps (Aug 2026)
- OpenObserve ships native SLOs with error budgets, multi-window burn-rate alerting, grouped alerts per host/metric series, and **bidirectional Terraform and GitOps support** for managing alerts, dashboards, and SLOs as code.
- **Source**: https://openobserve.ai/blog/set-meaningful-slos/

**NEW DEFECT S-004: NeoTrix SLO definitions are not declarative or version-controlled**
- No mechanism to define SLOs as code (YAML/Terraform/HCL). SLO targets are implicit in HeartbeatAggregator thresholds. Cannot diff SLO changes across commits, cannot rollback SLO definitions, cannot audit SLO policy changes. The GitOps pattern for reliability engineering is absent.

---

## 4. CROSS-CUTTING DEFECTS

### 4.1 X-001: Monitoring Cannot Observe Itself (Meta-Monitoring Gap)
- All three domains converge on one principle: monitoring infrastructure must be independently observable. NeoTrix HeartbeatAggregator monitors modules but has no independent monitor for itself. If HB aggregation fails, the entire system is blind.
- **Sources**: oneuptime.com (TSDB self-monitoring), youngju.dev (5 pillars), opentelemetry.io (eBPF)

### 4.2 X-002: No Correlation Between Observation, Alert, and Response
- Industry separates: observe (Prometheus/OTel) → alert (Alertmanager) → respond (PagerDuty) → prevent (SRE Agent → code fix). NeoTrix has no pipeline connecting these stages. HeartbeatAggregator detects issues, but there's no automated escalation to NT-SHIELD for containment, no NT-MIND freeze trigger, no NT-ACT auto-remediation.

### 4.3 X-003: No Burn-Rate Intelligence in Consciousness Evolution
- The consciousness tick cycle (Soil→Roots→Trunk→Branches→Fruits→Core) has no awareness of system reliability budget. Evolution should slow when the system is unreliable and accelerate when budget is healthy. This is the core insight of SRE: reliability and velocity are on a shared budget.

---

## 5. DEFECT SUMMARY

| ID | Domain | Severity | Summary |
|----|--------|----------|---------|
| M-001 | Monitoring | HIGH | No eBPF-native telemetry pipeline |
| M-002 | Monitoring | CRITICAL | No independent observability path (single-point-of-failure) |
| M-003 | Monitoring | MEDIUM | 3-pillar only — missing continuous profiling + RUM |
| M-004 | Monitoring | HIGH | No AI anomaly detection or NL query interface |
| A-001 | Alerting | HIGH | No MCP-based incident response fabric |
| A-002 | Alerting | HIGH | No deployment-incident correlation |
| A-003 | Alerting | MEDIUM | No structured event recorder or dispatcher leak detection |
| A-004 | Alerting | MEDIUM | Conflates alert routing with incident response |
| S-001 | SLO/SLI | CRITICAL | No burn-rate alerting or error budget concept |
| S-002 | SLO/SLI | HIGH | SEAL pipeline has no budget-gated evolution stops |
| S-003 | SLO/SLI | HIGH | HeartbeatAggregator uses infra metrics, not user-journey SLIs |
| S-004 | SLO/SLI | MEDIUM | SLO definitions not declarative or version-controlled |
| X-001 | Cross | CRITICAL | Meta-monitoring gap — monitoring cannot observe itself |
| X-002 | Cross | HIGH | No observe→alert→respond→prevent pipeline |
| X-003 | Cross | HIGH | Consciousness evolution has no burn-rate intelligence |

---

## 6. SOURCES CITED

1. https://opentelemetry.io/blog/2026/obi-goals/ — OTel eBPF 2026 goals
2. https://oneuptime.com/blog/post/2026-02-06-prometheus-server-health-collector/ — Prometheus TSDB self-monitoring
3. https://www.youngju.dev/blog/culture/2026-05-16-observability-opentelemetry-datadog-grafana-honeycomb-prometheus-jaeger-ebpf-slo-2026-deep-dive.en — 5 pillars observability 2026
4. https://www.pagerduty.com/newsroom/pagerduty-operations-cloud-spring-2026-release/ — PagerDuty SRE Agent Spring 2026
5. https://www.pagerduty.com/drops/ — PagerDuty Aug 2026 product drops (GitHub integration)
6. https://github.com/prometheus/alertmanager/blob/main/CHANGELOG.md — Alertmanager 0.32 changelog
7. https://zairalabs.ai/guide/compare/alertmanager-vs-pagerduty/ — Alertmanager vs PagerDuty
8. https://www.youngju.dev/blog/observability/2026-03-13-sli-slo-error-budget-reliability-engineering-guide.en — SLI/SLO/Error Budget guide
9. https://performance.qa/blog/slo-sli-error-budgets-guide/ — SLO implementation playbook 2026
10. https://openobserve.ai/blog/set-meaningful-slos/ — OpenObserve native SLOs + Terraform GitOps
11. https://oneuptime.com/blog/post/2026-02-06-slo-error-budget-burn-rate-grafana/ — SLO dashboard with burn rate visualization
12. https://oneuptime.com/blog/post/2026-01-30-sre-burn-rate-alerts/ — Burn rate alerts build guide
