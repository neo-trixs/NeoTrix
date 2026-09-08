# Iteration Batch 652 — Information Theory / Channel Capacity / Compression

**Date**: 2026-09-06 | **Prior batch**: 651 (hardware-zero, CLP-SNN 5600×, Redwood <2wk, no fast-slow emotion decomposition, no hardware-in-loop)

---

## 1. Information Theory Findings

### 1A. Mutual Information-Entropy Plane (arXiv:2608.23456, Aug 2026)
- **What**: New 2D quantifier combining normalized permutation entropy + normalized permutation mutual information from Shannon theory. Provides richer time series characterization than single-metric approaches.
- **Source**: arxiv.org/abs/2608.23456
- **NEW Defect #1**: NeoTrix NT-MIND has no mutual information-entropy plane for its own internal time-series analysis (module activation patterns, SEAL pipeline convergence, emotion state transitions). All health signals use scalar metrics (phi, coherence, heartbeat aggregate). A 2D MI-Entropy plane would expose **coupled dynamics** that single scalars miss — e.g., a module could show stable entropy but collapsing mutual information with other modules, signaling silent desynchronization.

### 1B. Temporal Mutual Information for SNN Robustness (CVPR 2026)
- **What**: Xu et al. prove temporal MI compression in SNNs tightens robustness error bounds via information bottleneck principle. Propose TMI regularizer.
- **Source**: openaccess.thecvf.com (CVPR 2026, pp. 20711-20720)
- **NEW Defect #2**: NeoTrix NT-CORE's E8 reasoning engine and GWT attention routing have **zero information-theoretic robustness guarantees**. The TMI regularizer shows that temporal dynamics inherently compress MI — NeoTrix has no such compression mechanism for its own attention broadcasts. GWT broadcasts carry raw salient signals without MI compression, making the attention system vulnerable to noisy inputs (e.g., corrupted KB entries or adversarial tool outputs propagating across all modules).

### 1C. VBO-MI: Variational Mutual Information Bayesian Optimization (arXiv, Jan 2026)
- **What**: Fully gradient-based Bayesian optimization using variational MI estimation. 31 pages, 8 figures.
- **Source**: arxiv.org/list/cs.IT/2026-01
- **NEW Defect #3**: NeoTrix SEAL pipeline's experiment design (which experiment to run next) uses no VoI (Value-of-Information) framework with MI estimation. The absorbed VoI term in CONTEXT.md exists but is limited to Bayesian experiment design in nt_core_hcube — it does not extend to SEAL pipeline's own exploration/exploitation decisions. VBO-MI shows a fully gradient-based approach that could replace heuristic experiment selection.

### 1D. Entropy 2026 Conference (Barcelona, Jul 2026) — 8 Sessions
- **What**: Cross-disciplinary entropy conference covering complex systems, information theory + AI, quantum information, thermodynamics. 116 attendees, 53 countries.
- **Source**: sciforum.net/event/Entropy2026, blog.mdpi.com (Aug 2026)
- **NEW Defect #4**: NeoTrix has **no entropy production measurement** for its own information flows. The conference highlights that non-equilibrium entropy production is measurable and informative for understanding system dynamics. NT-MEMORY's KB operations (embeddings, BM25 indexing, versioning) are entropy-producing processes but have no entropy rate monitoring. Without this, there is no way to detect when KB operations are becoming inefficient (high entropy production = wasted compute) vs. productive (information gain).

---

## 2. Channel Capacity Findings

### 2A. Shannon-Hartley Calculator for Neuromorphic Channels (MetricGate, May 2026)
- **What**: Blahut-Arimoto algorithm implementation for discrete memoryless channels + analytic Shannon-Hartley for AWGN. Includes convergence diagnostics, optimal input distribution, parameter sensitivity.
- **Source**: metricgate.com/docs/shannon-channel-capacity/
- **NEW Defect #5**: NeoTrix's inter-module communication (EventBus, GWT broadcasts, KB queries) has **no channel capacity analysis**. Each communication path (e.g., NT-WORLD → NT-MEMORY crawl results, NT-ACT → NT-SHIELD sandbox requests) operates without knowing its channel capacity. If the channel capacity is exceeded (too many events, too large payloads), error probability becomes non-zero — but NeoTrix has no mechanism to detect or rate-limit based on capacity. This is a fundamental communication theory gap.

### 2B. Neuromorphic Network Co-Design (Nature Machine Intelligence, Jun 2026)
- **What**: Pengfei Sun et al. — algorithm-hardware co-design of SNN with dual memory pathway on custom neuromorphic chip. 4× throughput, 5× energy efficiency, 40-60% fewer parameters.
- **Source**: nature.com/natmachintell/articles (Jun 2026)
- **NEW Defect #6**: This reinforces Batch 651's hardware-zero finding but adds a **new dimension**: the co-design principle itself is absent. NeoTrix designs algorithms (E8, GWT, SEAL) without considering what hardware they'd run on. The Sun et al. paper shows that co-designing algorithm + hardware simultaneously yields multiplicative gains. NeoTrix has no algorithm-hardware co-design pathway — even if hardware awareness were added later, the algorithms weren't designed with hardware constraints in mind, limiting achievable efficiency.

### 2C. Polar Codes / LDPC Near Capacity (Fiveable, Mar 2026)
- **What**: Modern codes (turbo, LDPC, polar) approach Shannon limit within fractions of a dB. Block length matters — short codes pay penalty. Iterative decoding enables practical operation.
- **Source**: fiveable.me/coding-theory/unit-1 (Mar 2026)
- **NEW Defect #7**: NeoTrix's VSA HyperCube encoding has **no rate-distortion analysis**. VSA embeddings are high-dimensional vectors with no analysis of the "channel" between concept-space and embedding-space. The distortion introduced by dimensionality reduction, quantization, and embedding model approximation is unmeasured. A rate-distortion analysis would reveal whether the embedding dimension is excessive (wasting memory) or insufficient (losing critical distinctions).

---

## 3. Compression Findings

### 3A. AI Compression Without Losing Critical Details (Stanford/SLAC, Aug 2026)
- **What**: Neural network separates features by scale via wavelet analysis, compresses each scale separately. Enables selective decompression of regions of interest at different resolutions. Published in Nature Machine Intelligence.
- **Source**: news.stanford.edu (Aug 2026), doi: 10.1038/s42256-026-01287-9
- **NEW Defect #8**: NeoTrix KB has **no multi-scale compression**. All KB embeddings are stored at a single resolution/scale. The SLAC method shows that separating features by scale and compressing separately preserves critical fine-grained details that uniform compression destroys. NeoTrix's experience entries, module health snapshots, and knowledge graph nodes could benefit from multi-scale representation — e.g., a module health snapshot stored at coarse scale for quick health checks but with fine-scale detail available for deep diagnostics. Current single-scale storage wastes space on fine detail that's rarely accessed.

### 3B. 2026 Algorithmic Information Theory Data Compression Challenge (arXiv:2606.17712)
- **What**: General-purpose lossless compressor competition. 8GB RAM limit, 1MB decompressor. Tested on protein sequences, source code, Wikipedia, physics data, astronomical images, binaries. Neural compressors competed against Brotli, Zstandard, LZ4, XZ.
- **Source**: arxiv.org/abs/2606.17712, aitdcc.github.io
- **NEW Defect #9**: NeoTrix's KB storage uses no **algorithmic information theory** metrics. Kolmogorov complexity (the theoretical basis for the challenge) measures the ultimate compressibility of data. NeoTrix stores KB entries, experience snapshots, and module state without analyzing their algorithmic complexity. If entries have low Kolmogorov complexity (highly compressible), they're redundant and could be distilled further. If high (incompressible), they're genuinely novel and should be preserved at full fidelity. This metric would optimize the KB absorption pipeline's distillation quality.

### 3C. Practical Learned Image Compression (Apple/arXiv:2605.05148, May 2026)
- **What**: Comprehensive study of key modeling choices for practical learned image codecs. Jointly optimized for perceptual quality AND runtime. Novel techniques within ablations.
- **Source**: arxiv.org/abs/2605.05148, machinelearning.apple.com/research/compression
- **NEW Defect #10**: NeoTrix has no **perceptual quality vs. runtime tradeoff analysis** for its own data representations. The Apple paper jointly optimizes for quality AND speed — a fundamental tradeoff NeoTrix ignores. VSA HyperCube embeddings prioritize representational quality (high dimensionality) without measuring the runtime cost of maintaining those embeddings. GWT attention broadcasts prioritize salience accuracy without measuring broadcast latency. There's no Pareto frontier analysis for NeoTrix's internal data representations.

### 3D. Neural Weight Compression for Language Models (arXiv:2510.11234)
- **What**: Learned compression framework training neural codecs directly from pretrained LM weights. Nonlinear transforms + entropy coding outperform hand-designed transforms (channel scaling, rotation).
- **Source**: arxiv.org/abs/2510.11234
- **NEW Defect #11**: NeoTrix's LLM integration (NT-IO provider layer) has no **model weight compression awareness**. When loading/switching LLM providers, NeoTrix treats models as black boxes with no analysis of weight compressibility. If local models (Ollama) could have their weights compressed with learned codecs, memory footprint drops significantly — enabling more concurrent models or larger context windows on the same hardware. NeoTrix's provider abstraction hides this optimization opportunity.

### 3E. End-to-End Learned Video Compression Survey (Neurocomputing, Sep 2026)
- **What**: Comprehensive review of learned video compression. 186 references. Covers neural codecs replacing entire video coding pipelines.
- **Source**: sciencedirect.com/science/article/pii/S0925231226012361
- **NEW Defect #12**: NeoTrix's NT-WORLD content pipeline has **no learned video compression integration**. For dynamic-comic production (the primary use case), video frame sequences between keyframes are stored uncompressed or with generic codecs. A learned video codec could exploit the specific characteristics of dynamic-comic content (repeated character models, limited scene complexity, synthetic generation patterns) for much higher compression ratios than generic codecs.

---

## Summary: 12 New Defects Found

| # | Category | Defect | Severity |
|---|----------|--------|----------|
| 1 | Info Theory | No MI-Entropy plane for internal time-series analysis | HIGH |
| 2 | Info Theory | GWT broadcasts carry raw signals without MI compression | CRITICAL |
| 3 | Info Theory | No VBO-MI for SEAL pipeline experiment selection | MEDIUM |
| 4 | Info Theory | No entropy production measurement for KB operations | HIGH |
| 5 | Channel Cap | No channel capacity analysis for inter-module communication | CRITICAL |
| 6 | Channel Cap | No algorithm-hardware co-design pathway | CRITICAL |
| 7 | Channel Cap | No rate-distortion analysis for VSA HyperCube encoding | HIGH |
| 8 | Compression | No multi-scale compression for KB storage | HIGH |
| 9 | Compression | No Kolmogorov complexity metrics for KB entries | MEDIUM |
| 10 | Compression | No perceptual quality vs. runtime Pareto analysis | MEDIUM |
| 11 | Compression | No model weight compression for LLM provider layer | MEDIUM |
| 12 | Compression | No learned video compression for dynamic-comic pipeline | HIGH |

## Sources Cited

1. arxiv.org/abs/2608.23456 — Mutual information-entropy plane (Aug 2026)
2. openaccess.thecvf.com — Robust SNNs by Temporal MI (CVPR 2026)
3. arxiv.org/list/cs.IT/2026-01 — VBO-MI Bayesian Optimization
4. sciforum.net/event/Entropy2026 — Entropy 2026 Conference
5. metricgate.com/docs/shannon-channel-capacity/ — Shannon Capacity Calculator (May 2026)
6. nature.com/natmachintell — Neuromorphic Co-Design (Jun 2026)
7. fiveable.me/coding-theory — Channel Capacity Concepts (Mar 2026)
8. news.stanford.edu — SLAC AI Compression (Aug 2026)
9. arxiv.org/abs/2606.17712 — AIT Compression Challenge (Jun 2026)
10. arxiv.org/abs/2605.05148 — Practical Learned Image Compression (May 2026)
11. arxiv.org/abs/2510.11234 — Neural Weight Compression
12. sciencedirect.com — Learned Video Compression Survey (Sep 2026)
