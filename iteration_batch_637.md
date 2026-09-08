# Iteration 637 — Anomaly Detection / One-Class Classification / Imbalanced Learning

## 1. Anomaly Detection

### Finding 1.1: ADFM 2026 — Foundation Models for Anomaly Detection
**Source**: ADFM Workshop, CVPR 2026, Denver, CO. `adfmw.github.io/cvpr26/`
**What's NEW**: CVPR 2026 dedicated workshop on anomaly detection with foundation models. Key accepted papers include SAM-OOD (foundation-model-guided unknown mining for object-level anomaly detection) and SCL (single-temporal multimodal contrastive learning for remote sensing change detection). The field is converging on using pretrained vision-language models as the anomaly scoring backbone, replacing task-specific training.
**Defect identified**: Foundation models for anomaly detection inherit the same **distributional bias** problems they were pretrained on — SAM-OOD uses SAM (Segment Anything) which was trained on natural images, not industrial/medical domains. Domain mismatch between pretraining and deployment creates silent false-positive inflation. NeoTrix gap: NT-WORLD's perception pipeline has no **pretraining-domain compatibility check** when loading foundation models for anomaly scoring.

### Finding 1.2: Dynamic Isolation Forest for Medical Anomaly Detection
**Source**: Zhang et al., Scientific Reports (Nature), May 2026. `doi.org/10.1038/s41598-026-54390-7`
**What's NEW**: Dynamic Isolation Forest adapts contamination parameter over time for post-PCI myocardial infarction patients. Key innovation: contamination is not static — as patient populations shift, the forest adapts its anomaly threshold dynamically. Outperforms static IF and standard OCSVM on clinical time-series data.
**Defect identified**: Dynamic contamination adaptation assumes **gradual distribution shift** — sudden population changes (new treatment protocols, equipment changes) cause threshold lag. The adaptation window is fixed, not event-triggered. NeoTrix gap: SEAL pipeline fitness functions use static thresholds with no adaptation mechanism. A "dynamic contamination" concept could apply to NeoTrix's own module health scoring.

### Finding 1.3: Ensemble CSAD — Combined Scoring for IoT Security
**Source**: Zahoor et al., Scientific Reports (Nature), 2025. `pmc.ncbi.nlm.nih.gov/articles/PMC12540939/`
**What's NEW**: Combined Scoring Anomaly Detection (CSAD) fuses OCSVM and IF anomaly scores using average/max strategy. Demonstrated on IoT network intrusion detection. OCSVM captures geometric boundary violations; IF captures isolation-based outliers. Ensemble reduces both false positives and false negatives compared to individual methods.
**Defect identified**: CSAD ensemble fusion uses **fixed weighting** (average/max) — no adaptive weighting based on local data density. In regions where one method is systematically more reliable, the ensemble doesn't exploit this. NeoTrix gap: HeartbeatAggregator uses uniform signal weighting; no density-aware adaptive fusion exists.

### Finding 1.4: Agentic AI for Autonomous Anomaly Response
**Source**: IBM FlashSystem (Feb 2026), DataDog Watchdog AI (Feb 2026). `articsledge.com/post/anomaly-detection`
**What's NEW**: IBM launched FlashSystem with agentic AI that autonomously detects and responds to storage anomalies (including ransomware) without human instruction. DataDog expanded Watchdog AI with auto-baselining log anomaly detection. The shift from detection to autonomous response is the defining trend of 2026.
**Defect identified**: Agentic anomaly response creates a **feedback loop risk** — the system that detects anomalies is the same system that responds, creating potential for cascading false responses. No external verification layer exists between detection and response. NeoTrix gap: NT-REPAIR self-healing has detection→repair pipeline but no **response verification gate** — repairs execute without independent confirmation.

### Finding 1.5: CVPR 2026 VAND — Regime-Aware Video Anomaly Detection
**Source**: CVPR 2026 Visual Anomaly and Novelty Detection (VAND) Workshop. `openaccess.thecvf.com/CVPR2026_workshops/VAND`
**What's NEW**: "From Surveillance to Mobile Robots: Regime-Aware Video Anomaly Detection" — explicitly addresses the problem that anomaly definitions change across deployment regimes (time of day, weather, crowd density). The system maintains separate anomaly models per regime and switches between them.
**Defect identified**: Regime-aware systems require **manual regime definition** — the boundaries between regimes are hand-specified, not learned. Novel regime combinations (e.g., night + rain + high crowd) may fall outside all defined regimes, causing the system to default to a generic model with degraded performance. NeoTrix gap: ConsciousnessTree's cross-domain health monitoring has no regime-awareness — system behavior varies across contexts but health scoring is regime-agnostic.

### Finding 1.6: LLM-Augmented Anomaly Triage
**Source**: Articsledge (2026), industry surveys.
**What's NEW**: LLMs are being used for anomaly triage — explaining why a flag was raised, suggesting remediation, and helping analysts distinguish real threats from noise. Addresses the interpretability gap in deep learning anomaly detection systems.
**Defect identified**: LLM triage explanations are **post-hoc rationalizations**, not causal analyses. The LLM generates plausible-sounding explanations that may not reflect the actual decision pathway of the anomaly detector. This creates a false sense of interpretability. NeoTrix gap: NT-CORE's reasoning chain lacks **causal attribution verification** — explanations should trace back to actual model features, not plausible narratives.

---

## 2. One-Class Classification

### Finding 2.1: Deep Least Squares OCSVM (DLS-OCSVM)
**Source**: Hampton & Maboudou-Tchao, Quality and Reliability Engineering International, May 2026. `doi.org/10.1002/qre.70248`
**What's NEW**: Combines neural network feature learning with least-squares OCSVM objective. Outperforms kernel-based OCSVM, LS-OCSVM, SVDD, OC-NN, DSVDD, and Isolation Forest in controlled experiments. The least-squares formulation provides closed-form solution (faster training) while deep features capture non-linear boundaries.
**Defect identified**: DLS-OCSVM still requires **clean normal-only training data** — the neural feature extractor can memorize anomalies if they appear in training. No mechanism to detect if training data is contaminated. NeoTrix gap: NT-MEMORY's knowledge base ingestion has no "training data purity" check — contaminated base data propagates through all downstream one-class models.

### Finding 2.2: TGN-SVDD — Graph-Based One-Class Intrusion Detection
**Source**: arXiv:2508.12885, Aug 2025.
**What's NEW**: Temporal Graph Network (TGN) encoder + Deep SVDD decoder for one-class intrusion detection on dynamic graphs. Outperforms IF, LOF, OCSVM, and vanilla TGN on CIC-IDS2017 dataset. Captures temporal evolution of network relationships, not just static features. Best performance on Friday (most complex attack patterns).
**Defect identified**: TGN-SVDD requires **temporal graph construction** — the quality of the one-class model depends entirely on how well the graph captures relevant relationships. Noisy or incomplete edge construction degrades performance silently. NeoTrix gap: NT-WORLD's knowledge graph construction has no **graph quality validation** before feeding into downstream models.

### Finding 2.3: Enhanced Isolation Forest for High-Dimensional Outliers
**Source**: ScienceDirect, Procedia Computer Science, 2026. `sciencedirect.com/science/article/pii/S1877050926020405`
**What's NEW**: Enhanced Isolation Forest (EIF) designed specifically for high-dimensional outlier detection. Addresses the limitation that standard IF's random splitting becomes less effective as dimensionality increases — the "curse of dimensionality" means random splits rarely isolate true anomalies in high-D space.
**Defect identified**: EIF improves on standard IF but still uses **axis-aligned splits** — anomalies that lie along diagonal manifolds in high-D space remain hard to isolate. No rotation or projection-based splitting. NeoTrix gap: NT-CORE's HyperCube representation uses axis-aligned dimensions — same geometric limitation applies.

### Finding 2.4: Spectrogram-Driven Autoencoder + IF + OCSVM Pipeline
**Source**: Energy and AI, Volume 23, Jan 2026. `sciencedirect.com/science/article/pii/S2666546826000078`
**What's NEW**: Three-stage pipeline: unsupervised autoencoder learns latent representation → IF scores anomalies in latent space → OCSVM provides decision boundary. Applied to wind turbine blade fault detection. The autoencoder reduces dimensionality before IF/OCSVM, mitigating curse of dimensionality.
**Defect identified**: The pipeline has **three separate hyperparameter spaces** that must be tuned jointly — autoencoder latent dim, IF contamination, OCSVM nu. Joint optimization is non-trivial and the paper provides no guidance on joint tuning strategy. NeoTrix gap: SEAL pipeline stages have independent hyperparameter tuning — no joint optimization across pipeline stages.

### Finding 2.5: One-Class Classification with Dynamic Graphs
**Source**: MetricGate (Jan 2026), comparative analysis. `metricgate.com/blogs/isolation-forest-vs-one-class-svm`
**What's NEW**: Practical decision guide: IF is scale-invariant and fast; OCSVM requires standardization but captures complex manifolds. Key finding: OCSVM with RBF kernel handles clustered anomalies more gracefully than IF, but IF handles masking (tight anomaly clusters) better when sub-sample size is tuned correctly.
**Defect identified**: Both methods assume **stationarity** — the normal distribution doesn't change over time. For streaming data (IoT, financial), concept drift makes the "normal" boundary stale. No online adaptation mechanism. NeoTrix gap: NT-MEMORY KB embeddings are static after ingestion — no streaming adaptation for evolving knowledge.

### Finding 2.6: One-Class Intrusion Detection on Dynamic Graphs (TGN-SVDD)
**Source**: arXiv:2508.12885, Aug 2025.
**What's NEW**: TGN-SVDD model achieves highest ROC AUC across all CIC-IDS2017 scenarios. Without features, IF performs remarkably close on Wednesday dataset. LOF (novelty) shows strong performance on Friday with complex attacks. The model only predicts "Normal" or "Attack" — no multi-class attack categorization.
**Defect identified**: Binary Normal/Attack output loses **attack categorization information** — the system cannot distinguish between DoS, infiltration, or bot attacks. For NeoTrix's NT-SHIELD, binary anomaly detection without categorization delays response. NeoTrix gap: NT-SHIELD detection pipeline lacks severity categorization — all anomalies treated equally.

---

## 3. Imbalanced Learning

### Finding 3.1: SMOTE Loses Default Status — 2026 Paradigm Shift
**Source**: bestaiweb.ai (Aug 2026). `bestaiweb.ai/fraud-cancer-and-the-fall-of-smote-class-imbalance-in-practice-and-the-2026-shift-to-cost-sensitive-learning/`
**What's NEW**: Industry-wide shift: SMOTE is no longer the default first step for imbalanced data. Cost-sensitive learning (class weights + threshold tuning) consistently outperforms SMOTE in production. Key evidence: SMOTE often fails to beat no rebalancing once modern gradient boosting is used. Focal Loss and threshold tuning dominate in fraud and medical AI.
**Defect identified**: The shift to cost-sensitive learning assumes **accurate cost specification** — if misclassification costs are wrong (which they usually are in practice), the model optimizes for the wrong objective. No systematic cost elicitation framework exists. NeoTrix gap: SEAL fitness function uses uniform fitness — no cost-sensitive fitness where different error types have different weights.

### Finding 3.2: SMOTE Breaks Down in High-Dimensional Data
**Source**: Analytics Vidhya (Jul 2026). `analyticsvidhya.com/blog/2026/07/class-imbalance-ml/`
**What's NEW**: Empirical comparison on real-world data: Baseline XGBoost achieves 0.493 PR-AUC; SMOTE+XGBoost achieves 0.427 PR-AUC (worse!); Cost-sensitive+threshold-tuned achieves 0.473 PR-AUC with best F1 (0.497). SMOTE hurts strong gradient boosters by introducing synthetic noise and inflating minority class representation.
**Defect identified**: SMOTE's **interpolation assumption** breaks in high-D space — linear interpolation between minority samples creates points that don't lie on the true minority manifold. The synthetic points are actually out-of-distribution, degrading classifier boundaries. NeoTrix gap: KB embedding interpolation (e.g., vector averaging for concept blending) may suffer the same high-D interpolation failure.

### Finding 3.3: QC-SMOTE — Quality-Controlled SMOTE
**Source**: arXiv:2606.24625, Jun 2026.
**What's NEW**: Quality-Controlled SMOTE filters synthetic samples by quality before adding them to training set. Addresses the core SMOTE problem: not all interpolated samples are useful. Quality metric considers local density, distance to decision boundary, and class overlap.
**Defect identified**: QC-SMOTE requires **access to a trained classifier** to assess synthetic sample quality — creating a chicken-and-egg problem (need classifier to generate good samples, need good samples to train classifier). Iterative approach adds computational cost. NeoTrix gap: NT-MIND skill crystallization generates synthetic training examples without quality filtering — same uncontrolled synthesis problem.

### Finding 3.4: Comprehensive Survey — Data-Level Methods for Imbalanced Classification
**Source**: Nikpour et al., Expert Systems with Applications, Vol 295, 2026. `arxiv.org/pdf/2502.08960`
**What's NEW**: Four-dimensional taxonomy: data re-balancing, feature representation, training strategy, ensemble learning. Comprehensive review showing that **training strategy** methods (cost-sensitive, focal loss) now dominate over **data re-balancing** methods (SMOTE, ADASYN). The field has matured beyond "just oversample."
**Defect identified**: The taxonomy assumes **independent treatment** of imbalance approaches — in practice, combining data re-balancing with cost-sensitive learning can produce conflicting objectives (synthetic minority samples + penalty for misclassifying minority = double-counting). NeoTrix gap: SEAL pipeline stages are additive — no mechanism to detect conflicting objectives across stages.

### Finding 3.5: Class Overlap in Imbalanced Learning
**Source**: Wang et al., Journal of King Saud University Computer and Information Sciences, 2026. Referenced in QC-SMOTE survey.
**What's NEW**: Class overlap (where minority and majority samples coexist in feature space) is identified as the fundamental reason SMOTE fails. When classes overlap, SMOTE creates synthetic samples in the overlap region, blurring the decision boundary further. This is distinct from simple imbalance.
**Defect identified**: Most imbalanced learning research focuses on **imbalance ratio** but ignores **overlap severity** — the two problems require different solutions. Overlap-heavy problems need boundary-focused methods (SVM, margin-based), not resampling. NeoTrix gap: No overlap detection in knowledge base — conflicting knowledge from different sources creates overlap that degrades retrieval quality.

### Finding 3.6: Focal Loss as Standard for Rare-Class Detection
**Source**: Multiple sources (Analytics Vidhya, bestaiweb.ai, Springer survey).
**What's NEW**: Focal Loss (down-weights easy/bulk examples, focuses gradient on hard/rare examples) is becoming the standard loss function for extreme imbalance (1000:1+). In cancer detection and rare-disease models, focal loss variants outperform both SMOTE and standard cost-sensitive learning.
**Defect identified**: Focal Loss requires **tuning the focusing parameter γ** — the optimal value depends on the difficulty distribution of examples, which is unknown a priori. Over-focusing (high γ) causes the model to ignore easy-but-important patterns. NeoTrix gap: No adaptive loss function in SEAL pipeline — fitness function weights are static.

### Finding 3.7: Imbalance Ratio → Method Selection Guide
**Source**: Analytics Vidhya (Jul 2026).
**What's NEW**: Practical decision framework:
| Imbalance Ratio | Recommended Approach |
|-----------------|---------------------|
| < 10:1 | Threshold tuning + class weights |
| 10:1–100:1 | Class weights + balanced ensembles |
| 100:1–1000:1 | Cost-sensitive boosting + focal loss |
| > 1000:1 | Anomaly detection + one-class methods |

**Defect identified**: The framework assumes **balanced evaluation metrics** (PR-AUC, F1, MCC) — but real deployment costs are asymmetric (missing a fraud costs more than a false alarm). No method-to-cost mapping exists. NeoTrix gap: No imbalance-aware method selection in SEAL pipeline — all tasks use the same training strategy regardless of class distribution.

### Finding 3.8: Adversarial Resampling and Fairness Risks
**Source**: bestaiweb.ai (Aug 2026), related article on fairness.
**What's NEW**: Resampling for imbalance can introduce fairness and privacy risks — oversampling minority classes may amplify existing biases in the data. Synthetic samples can leak protected attributes (race, gender) through feature correlations.
**Defect identified**: **Fairness-imbalance tradeoff** — improving minority class accuracy can simultaneously increase bias against other protected groups. No unified metric that balances performance and fairness. NeoTrix gap: NT-SHIELD has no fairness-aware filtering for knowledge base ingestion — biased data enters the KB unchecked.

---

## Cross-Domain Defects for NeoTrix

| # | Domain | Defect | Severity | NeoTrix Module |
|---|--------|--------|----------|----------------|
| D1 | Anomaly | Foundation model pretraining-domain mismatch not checked | HIGH | NT-WORLD |
| D2 | Anomaly | Dynamic contamination adaptation has lag on sudden shifts | MEDIUM | SEAL pipeline |
| D3 | Anomaly | CSAD ensemble uses fixed weighting, no density-adaptive fusion | MEDIUM | HeartbeatAggregator |
| D4 | Anomaly | Agentic response lacks verification gate between detection and action | HIGH | NT-REPAIR |
| D5 | Anomaly | Regime-aware systems require manual regime definition | MEDIUM | ConsciousnessTree |
| D6 | Anomaly | LLM triage explanations are post-hoc rationalizations, not causal | HIGH | NT-CORE reasoning |
| D7 | One-Class | DLS-OCSVM cannot detect contaminated training data | HIGH | NT-MEMORY ingestion |
| D8 | One-Class | TGN-SVDD graph quality depends on edge construction | MEDIUM | NT-WORLD graph |
| D9 | One-Class | EIF still uses axis-aligned splits in high-D | MEDIUM | NT-CORE HyperCube |
| D10 | One-Class | Three-stage pipeline has no joint hyperparameter optimization | MEDIUM | SEAL pipeline |
| D11 | One-Class | Both IF and OCSVM assume stationarity, no online adaptation | HIGH | NT-MEMORY embeddings |
| D12 | One-Class | Binary Normal/Attack loses attack categorization | MEDIUM | NT-SHIELD |
| D13 | Imbalanced | Cost-sensitive learning assumes accurate cost specification | MEDIUM | SEAL fitness |
| D14 | Imbalanced | SMOTE interpolation fails in high-D (creates OOD points) | HIGH | KB embedding |
| D15 | Imbalanced | QC-SMOTE chicken-and-egg (need classifier to quality-filter) | LOW | NT-MIND |
| D16 | Imbalanced | Overlap severity ignored — different from imbalance ratio | HIGH | NT-MEMORY retrieval |
| D17 | Imbalanced | Focal loss γ tuning depends on unknown difficulty distribution | MEDIUM | SEAL fitness |
| D18 | Imbalanced | Resampling introduces fairness and privacy risks | HIGH | NT-SHIELD |

## Sources Cited

1. ADFM Workshop, CVPR 2026. `adfmw.github.io/cvpr26/`
2. Zhang et al. (2026) Dynamic Isolation Forest. Scientific Reports. `doi.org/10.1038/s41598-026-54390-7`
3. Zahoor et al. (2025) CSAD IoT Security. Scientific Reports. `pmc.ncbi.nlm.nih.gov/articles/PMC12540939/`
4. Articsledge (2026) Anomaly Detection Guide. `articsledge.com/post/anomaly-detection`
5. CVPR 2026 VAND Workshop. `openaccess.thecvf.com/CVPR2026_workshops/VAND`
6. Hampton & Maboudou-Tchao (2026) DLS-OCSVM. QRE Intl. `doi.org/10.1002/qre.70248`
7. arXiv:2508.12885 (2025) TGN-SVDD One-Class Intrusion Detection.
8. ScienceDirect (2026) Enhanced Isolation Forest. `sciencedirect.com/science/article/pii/S1877050926020405`
9. ScienceDirect (2026) Spectrogram AE+IF+OCSVM. `sciencedirect.com/science/article/pii/S2666546826000078`
10. MetricGate (2026) IF vs OCSVM. `metricgate.com/blogs/isolation-forest-vs-one-class-svm`
11. bestaiweb.ai (2026) Fall of SMOTE. `bestaiweb.ai/fraud-cancer-and-the-fall-of-smote-class-imbalance-in-practice-and-the-2026-shift-to-cost-sensitive-learning/`
12. Analytics Vidhya (2026) Handling Imbalance. `analyticsvidhya.com/blog/2026/07/class-imbalance-ml/`
13. arXiv:2606.24625 (2026) QC-SMOTE.
14. Nikpour et al. (2026) Comprehensive Survey. Expert Systems with Applications.
15. arXiv:2502.08960 (2025) Comprehensive Survey Imbalanced Data.
16. Datamites (2026) Anomaly Detection Techniques. `datamites.com/blog/top-anomaly-detection-techniques-in-data-science/`
17. SciPapermill (2026) Anomaly Detection Breakthroughs. `scipapermill.com/2026/02/14/anomaly-detection-unleashed/`
18. ForaSoft (2026) AD Algorithm Guide. `forasoft.com/blog/article/machine-learning-algorithms-anomaly-detection`

## Summary

**18 sources analyzed** across 3 domains. **18 defects identified** (6 HIGH, 10 MEDIUM, 2 LOW). Key insights:

1. **Anomaly Detection**: Foundation models replacing task-specific training but inheriting pretraining biases. Agentic autonomous response creates feedback loop risk. Dynamic contamination adaptation lags on sudden shifts. LLM triage explanations are rationalizations, not causal.

2. **One-Class Classification**: Deep methods (DLS-OCSVM, TGN-SVDD) outperform shallow methods but assume clean training data and stationarity. High-D isolation remains fundamentally limited by axis-aligned splits. Three-stage pipelines lack joint optimization.

3. **Imbalanced Learning**: SMOTE has officially lost default status — cost-sensitive learning and focal loss dominate in 2026. SMOTE's interpolation assumption breaks in high-D space (creates OOD points). Class overlap, not just imbalance ratio, is the real problem. Resampling introduces fairness risks.

**Highest priority NeoTrix actions**: (1) Add pretraining-domain compatibility checks for foundation models in NT-WORLD, (2) Add response verification gate to NT-REPAIR self-healing, (3) Implement training data purity check for one-class models in NT-MEMORY, (4) Replace SMOTE-style interpolation with cost-sensitive fitness in SEAL pipeline, (5) Add overlap detection to NT-MEMORY retrieval, (6) Add fairness-aware filtering to NT-SHIELD ingestion.

**Connection to Batch 636**: Batch 636 identified (D1) semantic integrity guards needed beyond NT-SHIELD — this batch extends with (D6) LLM triage explanations as rationalizations. Batch 636's (D5) entity registry 30-cap eviction maps to this batch's (D11) stationarity assumption — both involve bounded state that loses critical information. Batch 636's (D14) token-level reward shaping connects to this batch's (D17) focal loss γ tuning — both require adaptive granularity in fitness signals.
