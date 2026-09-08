# Iteration Batch 445 — Research Loop

**Date**: 2026-09-06  
**Domains**: Additive Manufacturing · CNC Machining · Manufacturing Quality Control

---

## Sources Cited

### Additive Manufacturing (2026)
1. **Nature Communications** — "Knowledge-informed graph attention networks enable defect-free alloy design for laser additive manufacturing" (2026-08-26). DOI: 10.1038/s41467-026-77119-6
2. **AAAI 2026** — "Discovery of Feasible 3D Printing Configurations for Metal Alloys via AI-Driven Adaptive Experimental Design" (2026-03-14). DOI: 10.1609/aaai.v40i47.41428
3. **npj Advanced Manufacturing** — "Active learning for accelerated discovery of complex concentrated NiCoCr alloys in AM" (2026-06-23). DOI: 10.1038/s44334-026-00098-5
4. **MDPI Materials 19(7):1301** — "AI in Metal AM: Design, Process Modeling, Monitoring, Quality Optimization" (2026-03-25). DOI: 10.3390/ma19071301
5. **U of T Engineering** — "Self-driving lab leverages AI to develop tough new 3D-printable metal alloys" (2026-07-13)
6. **CATEC METALIA Project** — "Driving optimisation of metal AM through AI" (2026-06-25)

### CNC Machining (2026)
7. **J-STAGE / Trans. JSME** — "Digital Twin-Based Decision-Support Framework for Milling Processes" (2026-07-01)
8. **Springer Professional** — "AI-driven digital twin self-calibration and NC data optimization for five-axis CNC" (2026-07-30)
9. **Hexagon NCSIMUL 2026.2** — Selective Simulation, GPU-accelerated Rest Stock Previews (2026-05-22)
10. **arXiv:2608.29955** — "Cyber-Physical Machine Tool Framework with Real-Time Machining Process Digital Twin" (2026-08-30)
11. **Vericut 9.7** — AI-powered CNC simulation, regression testing, SmartPATH (2026-06-24)
12. **MDPI Machinery 7(3):66** — "CNC Machining Simulation in Brownfield Environments: DT, AI Optimisation" (2026-04-26)
13. **Springer Nature** — "Controller-Neutral CNC Representation (CNCR) for deterministic translation across heterogeneous controllers" (2026-07-30)
14. **Nidec NC Twin** — Digital twin platform for MVR Series double-column machining centers (2026-07-01)
15. **China Mechanical Engineering** — "DT-based dynamic multi-objective optimization for precision milling" (2026-04-25)

### Manufacturing Quality Control (2026)
16. **VDA/AIAG** — Joint SPC Handbook (AIAG-VDA SPC Manual) published June 30, 2026
17. **NexSPC Analysis** — "2026 AIAG-VDA SPC Manual: Key Changes in Cpk, Ppk, Control Charts" (2026-03-08, updated 2026-08-19)
18. **Quality Digest** — "Beyond the Statistics: Implementing the New VDA AIAG SPC Manual" (2026-08-25)
19. **iFactory** — "Quality Control in Manufacturing: The Full 2026 Playbook" (2026-05-12)
20. **MiDFUN** — "AIAG-VDA SPC 2026 Chapters 6–9: Capability, Performance, and Control Charts" (2026-08-19)
21. **MiDFUN** — "VDA SPC 2026 Chapter 9: ARL, OC Curves, and Control-Chart Detection Power" (2026-08-19)
22. **SS-SPC / Spectral Contrastive Learning** — Self-Supervised Adaptive SPC with Causal Graph Intelligence (2026-06-23)
23. **arXiv:2608.13937** — "MODERN Framework: Deep Vision for Intelligent Quality Monitoring" (2026-08)

---

## Defects Found in NeoTrix Design

### DEFECT-445-01: Missing Closed-Loop Alloy Discovery Pipeline
**Severity**: HIGH  
**Source**: Sources 1, 2, 3, 5  
**Gap**: 2026 research demonstrates AI-driven *closed-loop* alloy discovery: graph attention networks + Bayesian experiment design → print → characterize → feed back → iterate. NeoTrix has no equivalent pipeline for material-science-guided generative design. The SEAL pipeline evolves *skills*, not *physical material parameters*.  
**Evidence**: Nature Communications shows knowledge-informed GATs with uncertainty quantification can design defect-free Ni superalloys from small datasets (Source 1). AAAI 2026 shows 40-experiment budget discovering 6 feasible configs for GRCop-42 from 100M+ options (Source 2).  
**Suggestion**: Add a `nt_world_material::AlloyDiscoveryPipeline` module under NT-WORLD that wraps active-learning + Bayesian experiment design. Interface: `propose_candidates(alloy_space, objectives) → [config]`, `ingest_results(config, measurements) → updated_model`. Integrate with SEAL as a new Phase-5 "material evolution" stage.

### DEFECT-445-02: No Digital Twin for Process-Level (Not Just Machine-Level) Simulation
**Severity**: HIGH  
**Source**: Sources 10, 12, 14  
**Gap**: NeoTrix's DT concept is machine-level (heartbeat, health). 2026 CNC research (arXiv:2608.29955) demonstrates hierarchical DTs that maintain *simultaneous* DTs of machine tool AND machining process — voxel-based workpiece, synchronized vibration, persistent part DT repository. Nidec NC Twin (Source 14) adds surface-quality prediction (Sa/Sz) and cycle-time within 1% accuracy.  
**Evidence**: The arXiv framework achieves 20 Hz machining-state update, 100+ fps visualization, 0.16mm mean depth error. NeoTrix has no equivalent process-level simulation layer.  
**Suggestion**: Extend the NT-PHYSICAL domain with `nt_physical::process_digital_twin` providing: voxel workpiece model, cutting-force simulation, surface roughness prediction (Sa/Sz), and cycle-time estimation. Interface should conform to ISO 23247 reference architecture (Source 12).

### DEFECT-445-03: G-Code Translation Gap — No Controller-Neutral Representation
**Severity**: MEDIUM  
**Source**: Source 13  
**Gap**: The CNCR framework (2026) proves CNC programs can be semantically reconstructed into a controller-neutral intermediate form, enabling deterministic translation across Fanuc/Siemens/Heidenhain/Haas. NeoTrix has no CNC code abstraction layer. If NeoTrix ever controls CNC equipment, it will face the same dialect-lock problem.  
**Suggestion**: Add `nt_act::cnc_controller_neutral` with CNCR-style semantic reconstruction. Even as a stub, the data model (modal states + machining actions + normalized toolpath primitives) should be defined now to prevent vendor lock-in.

### DEFECT-445-04: No EWMA/CUSUM Adaptive Control Charts
**Severity**: HIGH  
**Source**: Sources 16, 17, 18, 20, 21  
**Gap**: The 2026 AIAG-VDA SPC Manual (published June 2026) fundamentally restructures SPC: (1) EWMA/CUSUM for small sustained shifts now standard practice, (2) confidence-level flexibility for control limits, (3) ARL-based detection-power engineering (design chart → compute ARL₁ → verify → deploy), (4) strict separation of retrospective analysis vs. real-time SPC control, (5) attribute charts must use exact binomial/Poisson CDF, not normal approximation. NeoTrix has no SPC module at all.  
**Evidence**: VDA SPC 2026 Chapter 9 (Source 21) states: "A control chart is not a form for recording defects. It is an engineering design." The ARL-based approach means you design for detection of a specific shift size, not just "draw 3σ limits."  
**Suggestion**: Add `nt_meta::spc_engine` implementing: Shewhart (X̄-R, X̄-s, I-MR), EWMA, CUSUM, attribute charts (p, np, u, c) with exact CDF computation, ARL/OC-curve computation, and the new confidence-level flexibility. Must support both retrospective (analysis) and real-time (SPC) chart modes per 2026 standard.

### DEFECT-445-05: Missing Alarm Management / False-Alarm Suppression
**Severity**: MEDIUM  
**Source**: Sources 17, 18, 21  
**Gap**: The 2026 SPC manual explicitly warns: "each added rule increases sensitivity but also raises Type I error." Modern factories with thousands of charts face alarm fatigue. NeoTrix's event system has no tiered-response mechanism. Every signal gets equal weight.  
**Evidence**: Quality Digest (Source 18) shows the real challenge isn't calculation — it's alarm management. Ppk-based alert suppression (suppress alarms when Ppk >> 1.33) is recommended practice.  
**Suggestion**: Add alarm severity tiers to NT-META's cross-module audit: (a) critical — immediate stop, (b) warning — investigate within N subgroups, (c) informational — log only. Implement Ppk-threshold gating: if Ppk > configurable_threshold, auto-suppress non-critical alerts. Tie into EventBus with priority channels.

### DEFECT-445-06: No Physics-Informed Hybrid ML for Manufacturing
**Severity**: HIGH  
**Source**: Sources 4, 15  
**Gap**: MDPI Materials (Source 4) shows hybrid physics-ML models are "the most promising near-term pathway" for industrial AM. China Mechanical Engineering (Source 15) demonstrates Optuna-GBR + IMORIME for dynamic multi-objective optimization achieving 19.99% energy reduction. NeoTrix uses pure data-driven ML (VSA embeddings) with no physics-informed constraints.  
**Suggestion**: Introduce `nt_core::physics_informed_wrapper` that enforces conservation laws, thermodynamic constraints, and process physics as loss-function regularizers on top of existing ML predictions. Target: at minimum, thermal-history-aware quality prediction for manufacturing processes.

### DEFECT-445-07: No Multi-Sensor Fusion for In-Situ Monitoring
**Severity**: MEDIUM  
**Source**: Sources 4, 6  
**Gap**: CATEC's METALIA project (Source 6) demonstrates multi-sensor fusion (thermal + acoustic + visual) for real-time defect detection in PBF and DED processes. MDPI review (Source 4) identifies multi-sensor monitoring as key to bridging research→industry. NeoTrix has no sensor fusion abstraction.  
**Suggestion**: Add `nt_physical::sensor_fusion_hub` supporting: thermal camera, acoustic emission, visual inspection, force/torque sensor inputs. Provide `fusion_pipeline(sensors) → anomaly_score` interface compatible with GWT attention routing.

### DEFECT-445-08: Missing Brownfield Legacy Integration Framework
**Severity**: LOW  
**Source**: Source 12  
**Gap**: MDPI Machinery (Source 12) presents a 7-layer ISO 23247-aligned architecture specifically for brownfield CNC environments — legacy controllers, heterogeneous data semantics, edge-cloud compute placement under latency constraints, OPC UA v1.05 with PFS. NeoTrix assumes greenfield modern hardware.  
**Suggestion**: Document a `nt_io::brownfield_adapter` specification for connecting legacy CNC equipment via OPC UA / MTConnect middleware. Include edge-cloud split rules and latency-aware compute placement.

---

## Design Improvement Suggestions (Ranked by Priority)

| Rank | Defect | Suggestion | Effort |
|------|--------|-----------|--------|
| 1 | 445-04 | Build `nt_meta::spc_engine` with EWMA/CUSUM/ARL per AIAG-VDA 2026 | 2-3 weeks |
| 2 | 445-01 | Add `nt_world_material::AlloyDiscoveryPipeline` for closed-loop material evolution | 3-4 weeks |
| 3 | 445-02 | Extend NT-PHYSICAL with process-level DT (voxel + cutting force + surface quality) | 4-6 weeks |
| 4 | 445-06 | Add physics-informed wrapper to existing ML pipeline | 1-2 weeks |
| 5 | 445-05 | Implement alarm tiering + Ppk-gated suppression in NT-META | 1 week |
| 6 | 445-07 | Add sensor fusion hub to NT-PHYSICAL | 2-3 weeks |
| 7 | 445-03 | Define CNCR data model stub for CNC interop | 0.5 week |
| 8 | 445-08 | Document brownfield adapter specification | 0.5 week |

---

## Cross-Domain Synthesis

The 2026 research landscape reveals a clear convergence: **physics-informed closed-loop AI** is replacing pure data-driven approaches across all three domains. The key architectural insight for NeoTrix:

1. **Alloy discovery** (AM) and **quality control** (SPC) share the same pattern: Bayesian experiment design → measure → feed back → update model. NeoTrix should unify these under a single `BayesianExperimentDesign` trait.

2. **Digital twins** are splitting into two layers: machine-level (health/status) and process-level (simulation/prediction). NeoTrix currently only has machine-level. Adding process-level closes the loop with SPC.

3. **Alarm fatigue** is the #1 adoption barrier for AI-driven quality systems. NeoTrix's EventBus should adopt tiered severity from day one, not bolt it on later.
