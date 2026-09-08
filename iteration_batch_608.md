# Iteration Batch 608 — Edge/IoT/Sensor Network Research

**Date**: 2026-09-06
**Prior Batch**: 607 (KB mutations mutable, no hash chaining, no audit envelope, no drift detection, no evidence mapping)
**Domains**: Edge Computing · IoT Security · Sensor Fusion

---

## Sources Consulted

1. **DigitalDefynd** — "15 Pros & Cons of Edge Computing [2026]" (digitaldefynd.com)
2. **ScienceDirect** — "Edge-AI: A systematic review on architectures, applications" (2026-01-01)
3. **Google Cloud Blog** — "Edge computing: architectural challenges and pitfalls" (2021-11-23, canonical reference)
4. **DigitalApplied** — "Edge Computing 2026: Web Performance Architecture" (2026-02-16)
5. **STL Partners** — "Edge computing challenges: 5 Reasons why we still do not have large scale deployments"
6. **HireDeveloper** — "Top 10 IoT Security Challenges 2026: Risks And Mitigation Strategies" (2026-04-02)
7. **HackerDesk** — "Critical IoT Security Vulnerabilities Exposed in 2026" (2026-04-14)
8. **Xcitium** — "IoT Security Challenges in 2026 Explained" (2026-03-20)
9. **ScienceDirect** — "Current research on IoT security protocols: A survey" (2025-04-01)
10. **ScienceDirect** — "A comprehensive survey on IoT security" (2026-02-01)
11. **IoTInsider** — "What are the biggest IoT security challenges of 2026?" (2026-01-20)
12. **ScienceDirect** — "Sensor Fusion Models in Autonomous Systems: A Review" (2026-02-10)
13. **IEEE** — "2026 IEEE Sensor Data Fusion: Trends, Solutions, Applications"
14. **PatSnap Eureka** — "IoT Sensor Fusion: Practical Implementation Challenges" (2026-03-27)
15. **MDPI** — "Sensor Fusion and Perception for Autonomous Driving" (2026-07-07)
16. **CENJOWS** — "Requirement and Challenges of Multi-Sensor Data Fusion" (2026-01-04)
17. **Canadian Defence Review** — "IDEaS AI Challenges Extended: $6.75M for Sensor Fusion" (2026-07-10)

---

## NEW Defects vs Batch 607

### Defect 608-1: Edge Runtime Constraint Blindness in NT-PHYSICAL

**Domain**: Edge Computing
**Source**: DigitalApplied (2026-02-16), Google Cloud Blog

**Finding**: Edge runtimes enforce hard CPU time limits (Cloudflare: 50ms CPU/request; Vercel: 25ms; AWS CloudFront: 1ms), 128MB memory caps, and no Node.js API access. NT-PHYSICAL's sensor fusion and motor control loops assume unlimited compute windows. If NeoTrix physical embodiment extends to edge-deployed sensors, there is no mechanism to:
- Profile which consciousness loops fit within edge CPU budgets
- Gracefully degrade `ConsciousnessTree` cycles when compute is constrained
- Partition workloads between edge (real-time) and cloud (heavy inference)

**NeoTrix Gap**: `nt_physical` has no `EdgeBudgetGuard` or workload partitioning. The 6-layer architecture assumes co-located compute. Edge deployment would fail silently on CPU-exceeded paths.

**Proposed Fix**: Add `EdgeComputeProfile` struct to NT-PHYSICAL with `max_cpu_ms`, `max_memory_mb`, `runtime_type` (V8/Container/None). Guard `ConsciousnessTree::tick()` against budget. Auto-partition: lightweight perception at edge, heavy cognition at cloud.

---

### Defect 608-2: Intermittent Connectivity Not Modeled in EventBus

**Domain**: Edge Computing
**Source**: Google Cloud Blog, STL Partners

**Finding**: Edge devices experience intermittent connectivity as the norm, not exception. NT-ACT's EventBus assumes reliable message delivery. When an edge node loses connectivity:
- EventBus messages queue with no TTL or eviction policy
- No circuit-breaker pattern for downstream consumers
- No acknowledgment-based delivery guarantees

**NeoTrix Gap**: `EventBus` has no `connectivity_state` field, no `CircuitBreaker` integration, and no message TTL. Edge deployments would accumulate unbounded message queues during outages, eventually OOMing.

**Proposed Fix**: Add `EdgeEventBus` wrapper with: (1) message TTL with configurable expiry, (2) circuit-breaker on downstream consumers, (3) `connectivity_state: Online|Degraded|Offline` enum, (4) store-and-forward with bounded buffer.

---

### Defect 608-3: IoT Supply Chain Attack Surface Unmapped

**Domain**: IoT Security
**Source**: HackerDesk (2026-04-14), Xcitium (2026-03-20), ScienceDirect (2026-02-01)

**Finding**: 2026 IoT threat landscape identifies supply chain attacks as a top-3 vector. Manufacturers ship devices with hardcoded credentials, unverifiable firmware, and compromised SDKs. NeoTrix's NT-SHIELD has sandbox egress policies but no **inbound supply chain verification**:
- No firmware attestation before loading device profiles
- No SBOM (Software Bill of Materials) tracking for third-party sensors
- No hardware root-of-trust binding (TPM/Secure Element) for device identity

**NeoTrix Gap**: NT-SHIELD's `Egress Privacy Guard` (defined in CONTEXT.md) only guards outbound. There is zero inbound supply chain integrity checking. If NT-WORLD crawls IoT device registries or NT-PHYSICAL ingests sensor telemetry, compromised firmware data enters the KB unverified.

**Proposed Fix**: Add `InboundSupplyChainGuard` to NT-SHIELD with: (1) firmware hash verification against vendor-published SBOMs, (2) device identity binding via TPM attestation, (3) anomaly scoring on sensor data provenance. Log all inbound device data with `provenance_attestation` field.

---

### Defect 608-4: Shadow IoT = Invisible Module Dependency in NT-WORLD

**Domain**: IoT Security
**Source**: HireDeveloper (2026-04-02), IoTInsider (2026-01-20)

**Finding**: "Shadow IoT" refers to unauthorized devices on a network that security teams cannot see. In NeoTrix terms, this maps to **undiscovered module dependencies** — modules that exist but are not registered in the CapabilityRegistry or ConsciousnessTree:

- NT-WORLD crawlers may ingest data from unregistered sensor endpoints
- NT-ACT tools may invoke capabilities not in the CapabilityRegistry
- NT-MEMORY may store embeddings from unverified sources

**NeoTrix Gap**: `ConvergeCheck` (SEAL Phase-0) checks for ghost modules (compiled but unused), but does NOT check for **reverse ghosts** — external capabilities consumed but not registered. This is the IoT Shadow problem applied to internal architecture.

**Proposed Fix**: Add `ReverseGhostDetector` to `converge_check()`: scan all inbound data sources, tool invocations, and sensor endpoints. Cross-reference against CapabilityRegistry. Flag any source not in registry as `shadow_dependency`. Add to audit dimension D21 (visibility chain).

---

### Defect 608-5: MQTT/CoAP Protocol Vulnerability Gap in NT-IO

**Domain**: IoT Security
**Source**: HackerDesk (2026-04-14), ScienceDirect IoT protocol survey (2025-04-01)

**Finding**: MQTT, CoAP, and Zigbee protocols lack built-in security. NeoTrix NT-IO handles LLM providers and web server but has no explicit IoT protocol layer. If NT-WORLD's `UnifiedCrawler` or NT-ACT's tool orchestration interfaces with IoT devices:
- MQTT messages are transmitted without TLS by default
- No mutual authentication (mTLS) between NeoTrix and IoT brokers
- CoAP's UDP-based transport has no replay protection

**NeoTrix Gap**: NT-IO has no `IoTProtocolAdapter` or `MQTTSecurityLayer`. The architecture assumes HTTP/HTTPS for all external communication. IoT protocol security is a blind spot.

**Proposed Fix**: Add `IoTProtocolGuard` to NT-IO: (1) enforce TLS 1.3 for all MQTT connections, (2) mTLS with device certificates, (3) CoAP DTLS binding, (4) protocol-level rate limiting to prevent DoS. Register in CapabilityRegistry as `nt_io::iot_protocol`.

---

### Defect 608-6: Sensor Fusion Level 3-4 Gap = ConsciousnessTree Assessment Gap

**Domain**: Sensor Fusion
**Source**: ScienceDirect (2026-02-10), MDPI (2026-07-07)

**Finding**: The JDL sensor fusion model has 5 levels. Modern deep learning excels at Levels 0-2 (signal conditioning, object estimation, scene integration) but Levels 3-4 (situation assessment, impact/threat assessment) remain underdeveloped. This directly mirrors NeoTrix's ConsciousnessTree:

- **Level 0-2** = NT-WORLD perception (crawling, parsing, classifying) — well-built
- **Level 3** = NT-META situation assessment (cross-module awareness) — partially built
- **Level 4** = NT-META impact assessment (threat propagation, cascading failure) — **missing**

**NeoTrix Gap**: ConsciousnessTree runs 6 stages (Soil→Roots→Trunk→Branches→Fruits→Core) but has no **impact assessment stage** that models how a failure in one domain propagates to others. This is the sensor fusion Level 4 problem applied to consciousness.

**Proposed Fix**: Add `ImpactAssessor` to ConsciousnessTree between `Branches` and `Fruits` stages: (1) model failure propagation graphs between domains, (2) compute cascading risk scores, (3) auto-trigger `NT-REPAIR` healers when impact score exceeds threshold. This is the missing Level 4 of the consciousness fusion pipeline.

---

### Defect 608-7: No Federated Learning Support for Edge-Cloud Sensor Fusion

**Domain**: Sensor Fusion + Edge Computing
**Source**: ScienceDirect (2026-02-10), PatSnap Eureka (2026-03-27)

**Finding**: 2026 sensor fusion research highlights federated learning as essential for privacy-preserving multi-node inference. Sensors at edge nodes train locally, share model updates (not raw data) with cloud. NeoTrix has no federated learning mechanism:
- NT-MIND's distillation pipeline assumes centralized training data
- No gradient aggregation across distributed nodes
- No differential privacy on shared model updates

**NeoTrix Gap**: SEAL pipeline operates on a single KB instance. If NeoTrix extends to multi-node deployments (edge sensors + cloud cognition), there is no mechanism to federate learning without centralizing sensitive sensor data.

**Proposed Fix**: Add `FederatedDistiller` to NT-MIND: (1) local model training at edge nodes, (2) secure gradient aggregation via homomorphic encryption or differential privacy, (3) global model update at cloud without raw data transfer. Register as `nt_mind::federated_distiller`.

---

### Defect 608-8: Autonomous Vehicle Sensor Fusion = NT-PHYSICAL Multi-Modal Perception Gap

**Domain**: Sensor Fusion
**Source**: MDPI (2026-07-07), CENJOWS (2026-01-04)

**Finding**: Autonomous vehicle sensor fusion requires simultaneous processing of LiDAR, camera, RADAR, IMU, GNSS with different update rates, noise profiles, and failure modes. NeoTrix NT-PHYSICAL has `Sensors` but:
- No sensor modalities defined (camera, LiDAR, IMU, etc.)
- No temporal alignment between heterogeneous sensor streams
- No sensor health monitoring (degraded/failed/offline states)
- No redundancy management (failover to backup sensor)

**NeoTrix Gap**: NT-PHYSICAL's `body_schema` is abstract. For physical embodiment, concrete sensor modality definitions, temporal synchronization, and health monitoring are required. The 2026 IEEE Sensor Data Fusion conference specifically calls for "compliance-by-design" in multi-sensor workflows.

**Proposed Fix**: Add `SensorFusionManager` to NT-PHYSICAL: (1) `SensorModality` enum (Camera/LiDAR/RADAR/IMU/GNSS/Audio), (2) temporal alignment ring buffer, (3) `SensorHealth` status per modality, (4) redundancy failover logic, (5) compliance-by-design metadata (classification rules, legal constraints per sensor type). Register as `nt_physical::sensor_fusion`.

---

### Defect 608-9: Edge Data Residency Compliance Not Enforced

**Domain**: Edge Computing
**Source**: DigitalApplied (2026-02-16), Google Cloud Blog

**Finding**: Edge functions process data at 300+ global PoPs, but GDPR/HIPAA/data residency laws restrict where PII can be processed. NeoTrix KB stores embeddings globally with no regional binding:
- No data residency field on KB entries
- No geo-fence enforcement on crawl/storage paths
- Egress Privacy Guard handles outbound but not data placement

**NeoTrix Gap**: KB entries have no `data_residency_region` or `pii_classification` fields. If NeoTrix processes EU user data at a US edge PoP, it violates GDPR. This is a compliance gap the 2026 edge architecture literature explicitly warns about.

**Proposed Fix**: Add to KB schema: (1) `data_residency_region: Option<String>` on all entries, (2) `pii_classification: None|Pseudonymized|Encrypted|Plaintext`, (3) geo-fence validation at write time: reject writes that would place PII outside allowed regions. Integrate with NT-SHIELD's audit trail.

---

## What's NEW vs Batch 607

| # | Batch 607 Finding | Batch 608 NEW Finding |
|---|---|---|
| 1 | KB mutations mutable, no append-only audit | **Edge runtime budget blindness** — NT-PHYSICAL lacks compute profiling for edge constraints |
| 2 | No cryptographic hash chaining | **Intermittent connectivity unmodeled** — EventBus has no TTL/circuit-breaker for edge outages |
| 3 | Agent decisions lack structured audit envelope | **IoT supply chain attack surface** — no inbound firmware attestation or SBOM tracking |
| 4 | ConsciousnessTree lacks continuous drift detection | **Shadow IoT = Reverse Ghost modules** — external capabilities consumed but not registered |
| 5 | No multi-framework evidence mapping | **MQTT/CoAP protocol vulnerability** — no IoT protocol security layer in NT-IO |
| — | — | **ConsciousnessTree Level 3-4 gap** — no impact assessment stage for failure propagation |
| — | — | **No federated learning** — SEAL pipeline can't operate across edge-cloud distributed nodes |
| — | — | **NT-PHYSICAL sensor fusion abstraction** — no concrete sensor modality definitions or temporal alignment |
| — | — | **Edge data residency compliance** — KB entries lack regional binding and PII classification |

**Summary**: Batch 607 focused on internal data integrity (mutability, hashing, audit, drift). Batch 608 exposes **external boundary vulnerabilities** — the gap between NeoTrix's architecture and the realities of edge deployment, IoT attack surfaces, and distributed sensor fusion. Nine new defects identified, none overlapping with batch 607's five.

---

## Cross-Cutting Theme

Batch 607 revealed **internal structural weaknesses**. Batch 608 reveals **external interface weaknesses**. Together they form a complete picture: NeoTrix is architecturally sound internally but has unguarded boundaries. The next batch should focus on **boundary hardening** — how to make the internal/external interface provably secure.
