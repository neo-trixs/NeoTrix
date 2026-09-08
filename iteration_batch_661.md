# Iteration Batch 661 — Data Visualization, Chart Libraries & Observability

**Date**: 2026-09-06  
**Cycle**: 661 of 10000  
**Context**: Batch 660 proved no edge-native inference dispatch, no MQTT/QUIC transport, no RTOS dispatch latency modeling, no WASM target, Egress Privacy Guard blind to MQTT. Now adding data visualization and observability layers.

---

## 1. Data Visualization (2026 Trends)

### Sources
1. Smashing Magazine (2026-08-26) — "Rethinking Data Visualisation: A UX Approach"
2. Infogram (2026-02-26) — "10 Trends in Data Visualization 2026"
3. Luzmo (2026-04-12) — "Data Visualization Trends 2026"
4. Fuselab Creative (2026-08-25) — "Top Data Visualization Trends 2026"
5. Visme (2026-02-19) — "30+ Best Data Visualizations 2026"
6. CallSphere (2026) — "AI Data Visualization for Sales & CX Teams"

### Key Findings

**F1: Prescriptive dashboards replace descriptive ones** — Dashboards now recommend actions via trend data, not just display stats. AI auto-generates narrative annotations, anomaly flags, and confidence rendering for non-technical stakeholders. *(Fuselab, CallSphere)*

**F2: Composable analytics replaces monolithic dashboards** — Charts placed inline in workflows, side panels, notifications — not on a dedicated reporting page. Single chart components embedded anywhere in the product. *(Luzmo)*

**F3: Natural language querying is now baseline expectation** — Users describe what they want in plain language, receive charts in response. Teams shipping dashboards without NLQ feel "dated within months." *(Luzmo, Fuselab)*

**F4: Mobile-first with separate interaction models** — Not responsive breakpoints; genuinely rethought interaction patterns (filter modals, collapsible chips, horizontal heatmaps on narrow viewports). *(VGIH case study, Infogram)*

**F5: LLM-narrated KPIs** — Every chart has a 1-sentence AI explanation of what changed and why. Dashboards explain themselves. *(CallSphere)*

**F6: Spatial computing + 3D data navigation** — Analysts physically navigate datasets in 3D environments. Gesture-based filtering, voice-triggered drill-downs becoming baseline. *(Fuselab)*

**F7: Data minimalism** — Cluttered dashboards replaced by clean, focused layouts with whitespace and reduced color palettes. Clarity over decoration. *(Infogram)*

**F8: Persona-driven granularity** — Same chart but individual view vs. aggregated team view based on user role. Single metric carries different weight per stakeholder. *(Smashing)*

---

## 2. Chart Libraries (2026 Landscape)

### Sources
1. TanStack Charts comparison (2026-08-26) — comprehensive benchmark
2. Ridhwaan Mayet (2026-05-29) — "Choosing a JavaScript Charting Library 2026"
3. npm: @observablehq/plot v0.6.17, echarts v6.1.0
4. Apache ECharts 6.0 release notes

### Key Findings

**F9: ECharts 6.0 major release** — 12 upgrades: new default theme, dynamic theme switching, dark mode, chord chart, beeswarm chart, scatter jittering, broken axis, matrix coordinate system, custom series npm publishing, 6 new custom chart types (violin, contour, stage, bar range, line range). *(ECharts 6.0 release)*

**F10: TanStack Charts — typed, framework-neutral core** — Single chart definition grows from standard to application-specific composition while keeping D3 and state ownership explicit. 39-45 KiB bundle. *(TanStack comparison)*

**F11: Bundle size tiers stable** — Chart.js: 45-58 KiB; ECharts: 153-173 KiB; Recharts: 153-168 KiB (95-110 KiB externalized React); Observable Plot: 83-92 KiB; D3: 90 KiB gzip; uPlot: 22 KiB gzip (fastest for time series). *(TanStack benchmark, Ridhwaan)*

**F12: Canvas vs SVG decision point** — SVG degrades at thousands of nodes with animation. ECharts defaults canvas (incremental/progressive rendering for large data). For 10K+ points: canvas mandatory. For 100K+: WebGL. *(Ridhwaan)*

**F13: Observable Plot v0.6.17** — Concise marks+transforms API, grammar of graphics, D3-powered but simpler. No first-party canvas/WebGL renderer — SVG only. *(npm)*

**F14: Config-driven vs component-driven split** — ECharts/Chart.js/Plotly: config objects (serializable, templateable from backend). Recharts: JSX components (React-native). D3: primitives (build anything, steep learning curve). *(Ridhwaan)*

---

## 3. Observability (2026 Stack)

### Sources
1. Grafana Labs — Lightweight APM for OpenTelemetry
2. OneUptime (2026-02-06) — Real-Time Service Dependency Map from OTel
3. OneUptime (2026-03-02) — OTel + Grafana Stack on Ubuntu
4. DEV Community (2026-04-14) — "Building Production-Grade Observability"
5. GrafanaCON 2026 — OTel instrumentation workshop

### Key Findings

**F15: OTel Collector + LGTM stack is production default** — Collector (gateway mode) → Tempo (traces) + Mimir (metrics) + Loki (logs) → Grafana unified visualization. 8GB RAM minimum. *(OneUptime, DEV Community)*

**F16: Service dependency maps from traces** — OTel Collector `servicegraph` connector emits `traces_service_graph_request_total`, `traces_service_graph_request_failed_total`, latency histograms. Grafana Node Graph panel renders directed graphs with health-colored edges. *(OneUptime service dependency)*

**F17: Trace-to-log correlation** — Click trace in Tempo → jump to Loki logs with matching trace_id. Click log → jump to Tempo trace. Derived fields in Loki data source config. *(OneUptime Ubuntu guide)*

**F18: SLO dashboards as first-class** — Error budget remaining, burn rate (1h/6h/24h windows), multi-burn alert status. RED method: Rate, Errors, Duration, Saturation. *(DEV Community)*

**F19: AlertManager on symptoms not causes** — Bad: "CPU 92%". Good: "Payment service error budget exhausted". Linked runbooks. *(DEV Community)*

**F20: Tempo stores traces in object storage only** — No indexing costs, indexed only by trace ID. Horizontally scalable. *(DEV Community)*

---

## 4. NEW DEFECTS IDENTIFIED

### DEFECT-661-1: No insight-to-action pipeline (NEOTRIX-NEW)
**Severity**: HIGH  
**Domain**: NT-IO + NT-MIND  
**Description**: 2026 dashboards are prescriptive — they recommend actions via trend analysis, not just display stats. NeoTrix's `nt_io_multimodal_transform` generates Mermaid diagrams (flowcharts, pie charts, bar charts) but has no pathway from chart observation → action recommendation. The `generative-dashboard` agent pattern in `nt_core_agent_patterns:231` is registered but has no backing implementation for insight extraction from visualization output.  
**Evidence**: `nt_io_multimodal_transform.rs` generates VisualType variants (Pie, Radar, BarChart, LineChart, Flowchart) but none produce actionable next-steps. `nt_core_agent_patterns.rs:231-237` registers pattern without impl.  
**Impact**: NeoTrix can render charts but cannot tell the operator what to do about them. Dashboards remain passive.  
**Fix**: Add `InsightExtractor` trait to `nt_io` that wraps chart output with LLM-generated narrative annotations and action recommendations. Wire into `generative-dashboard` pattern.

### DEFECT-661-2: No composable chart embedding (NEOTRIX-NEW)
**Severity**: MEDIUM  
**Domain**: NT-IO  
**Description**: 2026 composable analytics places individual charts inline in workflows, side panels, notifications — not on a dedicated page. NeoTrix's `nt_io_multimodal_transform` produces standalone Mermaid blocks but has no mechanism to embed a single chart component into arbitrary UI surfaces (TUI panels, web components, CLI output). The Mermaid output is monolithic.  
**Evidence**: `VisualType` enum variants produce text blocks; no chart-as-component abstraction exists. TUI config (`nt_cli_tui/config.rs:57,146`) has telemetry toggle but no chart embedding mode.  
**Impact**: Charts只能全页展示,无法作为内联上下文片段注入到其他界面.  
**Fix**: Define `EmbeddedChart` trait with render_to(surface) that supports TUI panel, web fragment, and CLI inline modes. ECharts Canvas/SVG dual-render approach is the reference.

### DEFECT-661-3: No natural language query interface (NEOTRIX-NEW)
**Severity**: HIGH  
**Domain**: NT-IO + NT-CORE  
**Description**: NLQ is 2026 baseline expectation. Users describe what they want in plain language and receive charts. NeoTrix has no NLQ→chart pipeline. The `state_dashboard` VSA dimension (0.85 activation in `cli_agent_tools.rs:121`) exists as a concept but has no query parser or chart generation from natural language.  
**Evidence**: `nt_core_knowledge/vectors_group_b/cli_agent_tools.rs:121` — `state_dashboard` vector exists. `nt_io_multimodal_transform` only classifies from visual description strings, not from NL queries.  
**Impact**: Operators must manually describe charts instead of asking "show me p99 latency over last hour."  
**Fix**: Add `NLQueryParser` that decomposes natural language into (metric, time_range, filter, chart_type) and pipes to `nt_io_multimodal_transform`.

### DEFECT-661-4: No real-time streaming visualization (NEOTRIX-NEW)
**Severity**: MEDIUM  
**Domain**: NT-IO + NT-WORLD  
**Description**: 2026 real-time dashboards update automatically as new data arrives. NeoTrix's telemetry store (`nt_core_telemetry.rs`) records events but has no streaming push to visualization layer. The `TelemetryStore` is pull-based (read on demand), not push-based (stream to subscribers).  
**Evidence**: `nt_core_telemetry.rs:828` — records events into `provider_usage_ledger`. No WebSocket/SSE/streaming subscription mechanism. `nt_io_telemetry.rs` only sets up OTel span export, not visualization streaming.  
**Impact**: Dashboard state is stale until explicitly queried. Cannot show live p99 drift, live error rate, or live consciousness phi.  
**Fix**: Add `TelemetryStream` that wraps `TelemetryStore` with tokio::broadcast channels. Visualization layer subscribes and re-renders on delta.

### DEFECT-661-5: ECharts 6.0 chord/beeswarm/matrix not available (NEOTRIX-NEW)
**Severity**: LOW  
**Domain**: NT-IO  
**Description**: ECharts 6.0 ships new chart types (chord for relationship visualization, beeswarm for overlapping data, matrix coordinate system, violin/contour) that are ideal for NeoTrix's domain: chord for module dependency relationships, beeswarm for scatter jittering of p99 latency clusters, matrix for cross-domain health grids. None are accessible through current Mermaid-only output.  
**Evidence**: ECharts 6.0 release notes — 12 major upgrades. NeoTrix only uses Mermaid text generation (`nt_io_multimodal_transform.rs`).  
**Impact**: Cannot render relationship diagrams (chord), dense scatter plots (beeswarm), or grid comparisons (matrix) that would serve ConsciousnessTree visualization.  
**Fix**: Add ECharts as alternative renderer to `VisualType` system. Define `ChartRenderer` trait with Mermaid and ECharts backends.

### DEFECT-661-6: No service dependency graph from traces (NEOTRIX-NEW)
**Severity**: HIGH  
**Domain**: NT-MEMORY + NT-WORLD  
**Description**: OTel service dependency maps (Node Graph panels) visualize inter-service communication extracted from distributed traces. NeoTrix has 11 ConsciousnessTree branches and 7 faction domains that communicate — but no visualization of actual cross-domain call patterns. The `HeartbeatAggregator` collects health signals but does not emit edge metrics (request rate, error rate, latency per inter-module call).  
**Evidence**: `HeartbeatAggregator` in `core/nt_core_heartbeat.rs` produces `SystemHealthSnapshot` but no per-edge metrics. `cognitive_hub.rs:37` has "collaboration frequency matrix" but it's internal to REINFORCE reward, not visualization.  
**Impact**: Cannot see which NT domains are calling which, where bottlenecks are, or which cross-domain paths are degraded. Blind to topology.  
**Fix**: Add `ServiceGraphMetrics` that instruments cross-domain calls with OTel spans. Emit `traces_service_graph_*` equivalent. Visualize with Node Graph panel in Tauri desktop.

### DEFECT-661-7: No persona-driven dashboard adaptation (NEOTRIX-NEW)  
**Severity**: MEDIUM  
**Domain**: NT-IO + NT-FEEL  
**Description**: 2026 dashboards adapt based on user role/preferences/goals. Same data, different views for IC vs. manager. NeoTrix has no user role model for dashboard rendering. `AttentionManager` routes between acquisition/evolution modes but has no persona-driven visualization adaptation.  
**Evidence**: `nt_core_self::AttentionManager` routes Weapon Set I/II but no persona vector for "who is viewing this dashboard." `nt_cli_tui/config.rs` has telemetry bool but no role/preference model.  
**Impact**: All users see same dashboard layout regardless of whether they need individual metrics vs. team aggregates.  
**Fix**: Define `ViewerPersona` enum (Operator, Researcher, SystemArchitect, Executive) with per-persona chart selection and granularity. Feed into `EmbeddedChart` rendering.

### DEFECT-661-8: No error budget / SLO visualization (NEOTRIX-NEW)
**Severity**: MEDIUM  
**Domain**: NT-META + NT-IO  
**Description**: 2026 production observability centers on SLO dashboards with error budget burn rates, multi-window burn alerts, and RED method panels. NeoTrix has `ConsciousnessTree` maturity tracking (C0-C6) but no SLO/error-budget visualization for system health. No burn rate computation, no error budget remaining metric.  
**Evidence**: `ConsciousnessTree` tracks C0-C6 constellation maturity but doesn't compute SLI/SLO ratios. `HeartbeatAggregator` produces snapshots but no burn rate trend lines.  
**Impact**: Cannot answer "are we within error budget for consciousness coherence?" or "how fast are we burning through phi stability budget?"  
**Fix**: Add `SloDashboard` that computes SLI (e.g., phi_coherence > 0.8 ratio) and error budget remaining over 30d window. Render as burndown chart.

---

## 5. SUMMARY

### What's NEW from this batch
- **Prescriptive dashboards** replacing descriptive (AI narrative annotations, action recommendations)
- **Composable analytics** — charts embedded anywhere, not on dedicated pages
- **NLQ as baseline** — natural language → chart pipeline now expected
- **ECharts 6.0** — chord, beeswarm, matrix, violin, contour charts available
- **TanStack Charts** — typed, framework-neutral core at 39-45 KiB
- **OTel service dependency maps** from distributed traces (servicegraph connector)
- **SLO-first observability** with error budget burn rates
- **Trace-to-log-to-metrics correlation** as production requirement

### Sources Cited
1. Smashing Magazine — https://www.smashingmagazine.com/2026/08/rethinking-data-visualisation-ux-approach-dashboards/
2. Infogram — https://infogram.com/blog/10-trends-in-data-visualization-to-watch-in-2026/
3. Luzmo — https://www.luzmo.com/blog/data-visualization-trends
4. Fuselab — https://fuselabcreative.com/top-data-visualization-trends-2026/
5. Visme — https://visme.co/blog/best-data-visualizations/
6. CallSphere — https://callsphere.ai/blog/ai-data-visualization.md
7. TanStack — https://tanstack.com/charts/latest/docs/comparison
8. Ridhwaan — https://www.ridhwaan.xyz/blog/choosing-a-charting-library-echarts-d3-recharts-plotly-chartjs-deckgl/
9. ECharts 6.0 — https://echarts.apache.org/handbook/en/basics/release-note/v6-feature/
10. Grafana OTel — https://grafana.com/grafana/dashboards/24528-lightweight-apm-for-opentelemetry/
11. OneUptime — https://oneuptime.com/blog/post/2026-02-06-service-dependency-map-dashboard-opentelemetry-traces/view
12. OneUptime Ubuntu — https://oneuptime.com/blog/post/2026-03-02-how-to-configure-opentelemetry-with-grafana-stack-on-ubuntu/view
13. DEV Community — https://dev.to/varunvarde/building-production-grade-observability-opentelemetry-grafana-stack-9mc
14. GrafanaCON — https://github.com/grafana/grafanacon2026-opentelemetry-instrumentation

### Defect Count
- **8 new defects** (DEFECT-661-1 through DEFECT-661-8)
- HIGH: 3 (insight pipeline, NLQ, service dependency graph)
- MEDIUM: 4 (composable embedding, streaming viz, persona adaptation, SLO viz)
- LOW: 1 (ECharts 6.0 new chart types)
