# Iteration Batch 727 — Autoscaling / Capacity Planning / FinOps

**Date**: 2026-09-06
**Defects from 726**: (1) no per-value encryption for KB state, (2) zero config management for 9 subsystems, (3) no GitOps/single source of truth, (4) no drift detection, (5) silent reconciler failures

---

## 1. AUTOSCALING FINDINGS

### 1.1 KEDA Scale-to-Zero Now Native (Kubernetes v1.37 Beta)
**Source**: https://kubernetes.io/blog/2026/09/02/kubernetes-v1-37-hpa-scale-to-zero-beta/

Kubernetes v1.37 (Sept 2026) enables HPA `minReplicas: 0` by default for external/object metrics. The `ScaledToZero` condition distinguishes auto scale-down from manual pause. Requires a `ScaledToZero=True` condition on the controller for reconciliation to own the zero state.

**NEW DEFECT — NEO-727-01: NT-IO Batch Jobs Have No Scale-to-Zero Policy**
- NeoTrix batch ingestion (NT-WORLD crawl jobs, NT-MIND distillation cycles) runs as fixed-replica deployments
- No KEDA ScaledObjects defined for event-driven workloads
- No HPA with `minReplicas: 0` for idle queue consumers
- **Impact**: 100% resource waste during idle periods (estimated 40-60% of runtime)
- **Fix**: Define KEDA ScaledObjects for crawl/distillation pipelines with queue-depth triggers + scale-to-zero cooldown

### 1.2 VPA Auto + HPA Conflict Creates Oscillation Loop
**Source**: https://dev.to/npayyappilly/hpa-vs-keda-vs-vpa-a-quantitative-framework-for-autoscaling-strategy-selection-in-kubernetes-5fch (2026-08-24)

VPA Auto mode changes resource requests → pod restart → HPA sees capacity drop → scales out → VPA adjusts again → infinite loop. Safe: VPA in Recommendation mode only, HPA on custom metrics.

**NEW DEFECT — NEO-727-02: NT-CORE Resource Requests Never Right-Sized**
- NeoTrix subsystems have static `resources.requests` in manifests (if any exist)
- No VPA in Recommendation mode deployed for any domain
- No Goldilocks dashboard for cluster-wide recommendations
- **Impact**: OOM kills on under-provisioned pods; 30-70% over-provisioning on underutilized ones
- **Fix**: Deploy VPA in `Off` mode per namespace, run Goldilocks for 2 weeks, apply recommendations

### 1.3 KEDA Multi-Trigger + Fallback Safety
**Source**: https://blog.codercops.com/blog/kubernetes-autoscaling-keda-vpa-2026

KEDA's `fallback` property maintains replica count when metric collection fails. Without it, metric unavailability causes scale-to-zero during active workloads. KEDA 2.19+ has 70+ built-in scalers.

**NEW DEFECT — NEO-727-03: No Fallback Policy for Metric-Dependent Scaling**
- NeoTrix EventBus consumers (NT-ACT orchestration, NT-IO LLM gateway) depend on internal metrics
- No `fallback` replicas configured if metrics API becomes unavailable
- **Impact**: Silent workload starvation during metric pipeline degradation
- **Fix**: Configure `fallback.replicas` for all KEDA ScaledObjects

### 1.4 Karpenter Consolidation vs Static Node Groups
**Source**: https://cloud.servermall.com/blog/kubernetes-autoscaling-in-2026-hpa-vpa-keda-cluster-autoscaler-and-karpenter/ (2026-06-06)

Karpenter provisions nodes directly via cloud APIs (no node groups), picks optimal instance type, consolidates underutilized nodes aggressively. 40-60% better node utilization vs Cluster Autoscaler.

**NEW DEFECT — NEO-727-04: NT-PHYSICAL Node Provisioning Not Optimized**
- NeoTrix deployment targets static node groups (if any)
- No Karpenter NodePool definitions with consolidation policies
- No spot/on-demand mixing for cost optimization
- **Impact**: 40-60% higher node costs than necessary
- **Fix**: Define Karpenter NodePool with spot priority + consolidation enabled

---

## 2. CAPACITY PLANNING FINDINGS

### 2.1 Saturation as SLI — Closed-Loop SLO Integration
**Source**: https://novaaiops.com/capacity-planning (2026-05-29)

Capacity should be wired into SLO framework: pick a utilization threshold that predicts SLO breach, alert on crossing it, trigger capacity action BEFORE error budget burns. Treat saturation as an SLI.

**NEW DEFECT — NEO-727-05: NT-MEMORY KB Has No Saturation SLI**
- NeoTrix KB (SQLite-backed) has no defined saturation threshold
- No alert when KB query latency exceeds capacity planning bounds
- No feedback loop between KB utilization and SLO/error budget
- **Impact**: KB becomes silent bottleneck; SLO violations without root cause visibility
- **Fix**: Define KB saturation SLI (query latency P95, index fill ratio), alert at 70% threshold

### 2.2 Headroom Paradox — High Utilization Removes Error Margin
**Source**: https://novaaiops.com/capacity-planning (2026-05-29)

Fleet at 90% steady-state utilization has almost no headroom: one failed node, one traffic spike tips into saturation. Target 50-70% utilization, leave 30-50% headroom.

**NEW DEFECT — NEO-727-06: NT-WORLD Crawl Pipeline Has No Headroom Budget**
- Crawl workers (UnifiedCrawler, fetchers) run at near-100% utilization during active crawl
- No headroom for burst parsing, retry storms, or upstream rate limit backoff
- **Impact**: Crawl pipeline saturation → cascading EventBus delays → NT-MIND distillation stalls
- **Fix**: Define headroom policy: max 70% crawl worker utilization, autoscale buffer for retries

### 2.3 Capacity Planning ≠ Autoscaling
**Source**: https://novaaiops.com/capacity-planning (2026-05-29)

"Autoscaling is excellent at absorbing organic demand swings inside limits you have already set. It is helpless against: regional quota ceiling, downstream database that cannot add replica in seconds, stateful service needing careful rebalance, runaway scale-up that triples bill."

**NEW DEFECT — NEO-727-07: No Capacity Envelope Definition for Any Subsystem**
- No documented max/min scaling bounds for 9 NT-* domains
- No quota ceiling tracking (DB connections, API rate limits, GPU memory)
- No cost ceiling enforcement on scale-up events
- **Impact**: Unbounded scaling can triple costs; quota exhaustion causes silent failures
- **Fix**: Define capacity envelope per domain: min/max replicas, quota limits, cost ceiling

### 2.4 Match Strategy Requires Active Management
**Source**: https://www.clicktime.com/resources/resource-capacity-planning-guide (2026-03-12)

Match strategy (aligning capacity to demand through small frequent adjustments) requires active management — it's not "set and forget."

**NEW DEFECT — NEO-727-08: No Capacity Review Cadence Defined**
- No recurring capacity review for any NeoTrix subsystem
- No forecast-vs-actual comparison
- No feedback loop from incidents into capacity model
- **Impact**: Capacity drift goes undetected until SLO breach
- **Fix**: Define weekly capacity review per domain, monthly cross-domain capacity sync

---

## 3. FinOps / RESOURCE MANAGEMENT FINDINGS

### 3.1 Kubernetes Cost Allocation = Highest ROI FinOps Activity
**Source**: https://www.halkwinds.com/research/finops-benchmark-report-2026 (2026-04-30)

Kubernetes cost optimization delivers median 34% spend reduction — the highest ROI FinOps activity in 2025. Purpose-built tooling mapping namespace/pod/label to cost is central.

**NEW DEFECT — NEO-727-09: Zero K8s Cost Allocation for NT-* Domains**
- No namespace-level cost attribution for NT-CORE, NT-MIND, NT-MEMORY, etc.
- No Kubecost or equivalent deployed
- No per-domain cost dashboards
- **Impact**: Cannot identify which domain drives cost; optimization is guesswork
- **Fix**: Deploy Kubecost, label all pods by domain, establish per-domain cost baseline

### 3.2 Chargeback Eliminates 67% Idle Resources
**Source**: https://www.halkwinds.com/research/finops-benchmark-report-2026 (2026-04-30)

Organizations implementing chargeback report 67% reduction in idle cloud resources because engineering teams facing a real cost signal make different provisioning decisions.

**NEW DEFECT — NEO-727-10: No Chargeback/Showback for NeoTrix Subsystems**
- No cost attribution to specific domains
- No "who owns this cost" visibility
- Engineers operate without cost feedback loop
- **Impact**: Perpetual over-provisioning; no incentive to optimize
- **Fix**: Implement showback per domain (label-based), graduate to chargeback

### 3.3 AI Cost Management Is #1 Desired Skillset
**Source**: https://data.finops.org/ (State of FinOps 2026)

98% of FinOps practitioners now manage AI spend (up from 31% in 2024). AI cost management is the single most desired skillset. AI workloads have less transparent, more variable pricing.

**NEW DEFECT — NEO-727-11: NT-IO LLM Gateway Has No Token/Cost Metering**
- NeoTrix proxies LLM calls via NT-IO gateway
- No per-request token counting or cost attribution
- No budget enforcement on LLM spend
- No anomaly detection on token usage patterns
- **Impact**: Unbounded LLM costs; no visibility into which domain/feature drives spend
- **Fix**: Implement token counter in LLM gateway, per-domain cost attribution, budget alerts

### 3.4 Real-Time Alerting Reduces Budget Overruns by 71%
**Source**: https://www.halkwinds.com/research/finops-benchmark-report-2026 (2026-04-30)

Real-time cost alerting reduces cloud budget overruns by 71% compared to monthly billing review.

**NEW DEFECT — NEO-727-12: No Real-Time Cost Anomaly Detection**
- NeoTrix has no cost anomaly detection for any infrastructure component
- No automated alerts on spend deviations
- Monthly-at-best cost review cadence
- **Impact**: Cost overruns detected only after bill arrives; 71% higher overrun risk
- **Fix**: Deploy real-time cost alerting (CloudTrail correlation, EventBus-driven anomaly detection)

### 3.5 Unified Multi-Cloud Tooling Saves 29% More
**Source**: https://www.halkwinds.com/research/finops-benchmark-report-2026 (2026-04-30)

Multi-cloud organizations running unified FinOps tooling save 29% more than those managing each provider separately.

**NEW DEFECT — NEO-727-13: No Unified Cost View Across NT-SHIELD Proxy Pools**
- NT-SHIELD manages multiple proxy providers (residential, datacenter, Tor)
- No unified cost view across proxy providers
- No cost-per-request attribution per proxy tier
- **Impact**: Cannot optimize proxy spend; cheapest provider invisible
- **Fix**: Unified cost dashboard across proxy providers, per-request cost attribution

### 3.6 FinOps Maturity Gap: 73% Have Function, Few Are Mature
**Source**: https://www.halkwinds.com/research/finops-benchmark-report-2026 (2026-04-30)

73% of enterprises with >$1M cloud spend now operate a dedicated FinOps function (up from 47% in 2024). Top-quartile achieves 3.8x better cost efficiency than bottom-quartile.

**NEW DEFECT — NEO-727-14: No FinOps Practice for NeoTrix Infrastructure**
- No FinOps function or designated practitioner
- No cost efficiency benchmarks
- No maturity model assessment
- **Impact**: NeoTrix operates at bottom-quartile FinOps maturity
- **Fix**: Establish minimum FinOps practice: tagged resources, weekly cost review, quarterly optimization cycle

---

## 4. CROSS-DOMAIN SYNTHESIS

### Defect Summary Table

| ID | Domain | Defect | Severity | Source |
|----|--------|--------|----------|--------|
| NEO-727-01 | NT-WORLD/NT-MIND | No scale-to-zero for batch jobs | HIGH | K8s v1.37, KEDA docs |
| NEO-727-02 | NT-CORE | Resource requests never right-sized | HIGH | VPA+HPA conflict research |
| NEO-727-03 | NT-ACT/NT-IO | No fallback for metric-dependent scaling | MEDIUM | KEDA safety patterns |
| NEO-727-04 | NT-PHYSICAL | No Karpenter/consolidation policy | HIGH | Karpenter benchmarks |
| NEO-727-05 | NT-MEMORY | KB has no saturation SLI | HIGH | NovaAIOps capacity guide |
| NEO-727-06 | NT-WORLD | Crawl pipeline has no headroom budget | MEDIUM | Headroom paradox research |
| NEO-727-07 | ALL | No capacity envelope definitions | HIGH | Capacity ≠ autoscaling |
| NEO-727-08 | ALL | No capacity review cadence | MEDIUM | Match strategy requires mgmt |
| NEO-727-09 | ALL | Zero K8s cost allocation | CRITICAL | Halkwinds FinOps benchmark |
| NEO-727-10 | ALL | No chargeback/showback | HIGH | Halkwinds idle resource data |
| NEO-727-11 | NT-IO | LLM gateway has no token metering | CRITICAL | State of FinOps 2026 |
| NEO-727-12 | ALL | No real-time cost anomaly detection | HIGH | Halkwinds alerting data |
| NEO-727-13 | NT-SHIELD | No unified proxy cost view | MEDIUM | Multi-cloud tooling research |
| NEO-727-14 | ALL | No FinOps practice | HIGH | FinOps maturity data |

### Relationship to Batch 726 Defects

| Batch 726 Defect | Batch 727 Expansion |
|-------------------|---------------------|
| (1) No per-value encryption | → NEO-727-09/010: Cost data also unencrypted/attribution missing |
| (2) Zero config management | → NEO-727-02/07: No resource request config, no capacity envelopes |
| (3) No GitOps/single source of truth | → NEO-727-01/04: No KEDA/HPA manifests, no Karpenter NodePool specs |
| (4) No drift detection | → NEO-727-05/08/12: No saturation SLI, no capacity review, no cost anomaly |
| (5) Silent reconciler failures | → NEO-727-03/06: No fallback replicas, no headroom for retry storms |

### Sources Cited

1. Kubernetes v1.37 HPA Scale-to-Zero Beta — https://kubernetes.io/blog/2026/09/02/kubernetes-v1-37-hpa-scale-to-zero-beta/
2. HPA vs KEDA vs VPA Quantitative Framework — https://dev.to/npayyappilly/hpa-vs-keda-vs-vpa-a-quantitative-framework-for-autoscaling-strategy-selection-in-kubernetes-5fch
3. KEDA, VPA, Goldilocks 2026 — https://blog.codercops.com/blog/kubernetes-autoscaling-keda-vpa-2026
4. K8s Autoscaling Complete Guide 2026 — https://www.youngju.dev/blog/kubernetes/2026-03-11-kubernetes-hpa-vpa-keda-autoscaling-production.en
5. K8s Autoscaling HPA VPA KEDA Karpenter — https://cloud.servermall.com/blog/kubernetes-autoscaling-in-2026-hpa-vpa-keda-cluster-autoscaler-and-karpenter/
6. Capacity Planning for SRE 2026 — https://novaaiops.com/capacity-planning
7. Resource Capacity Planning Guide 2026 — https://www.clicktime.com/resources/resource-capacity-planning-guide
8. State of FinOps 2026 Report — https://data.finops.org/
9. FinOps Benchmark Report 2026 — https://www.halkwinds.com/research/finops-benchmark-report-2026
10. AWS FinOps Agent Public Preview — https://aws.amazon.com/blogs/aws-cloud-financial-management/aws-finops-agent-is-now-public-preview/
11. FinOps Principles 2026 — https://www.flexera.com/blog/finops/finops-principles/
12. Cloud Cost Optimization 2026 — https://wring.co/blog/cloud-cost-optimization
