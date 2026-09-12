# Model Reverse Engineering — Cycle 430

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination, multi-agent reasoning
**Sources**: arXiv (Sep 2026), ACL 2026, EMNLP 2026, Meta AI Research

---

## 5 Selected Papers/Models

### 1. RouteRelay: Event-Triggered Cross-Layer Route Reuse for Efficient Dynamic Sparse Attention
- **arXiv**: 2609.07306 (Sep 7, 2026)
- **Key Innovation**: Reuses routing metadata across Transformer layers instead of recomputing chunk-chunk scores at every layer. Anchor layers do full routing; intermediate layers only rescore when a "sentinel" chunk challenges the weakest selected chunk.
- **Results**: 99.99% route recall while rerouting only 25-78% of rows; evaluates 38-52% of full routing score pairs.
- **Pattern**: Cross-layer route reuse with lazy rerouting — "don't recompute what hasn't changed."

**NeoTrix Mapping**:
| Domain | Integration |
|--------|------------|
| NT-CORE (GWT) | GWT attention routing optimization: reuse salience routes across broadcast cycles instead of recomputing from scratch |
| NT-MIND | SEAL pipeline: anchor cycles do full distillation, intermediate cycles only refine when drift detected |
| NT-MEMORY | KB query routing: reuse cached route metadata across search iterations |

**Actionable**: Apply RouteRelay's sentinel-based rerouting to GWT attention broadcast — compute full salience at anchor cycles (e.g., every N growth cycles), reuse routes at intermediate cycles with drift detection.

---

### 2. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
- **arXiv**: 2609.07237 (Sep 7, 2026)
- **Key Innovation**: Coarse-to-fine attention routing. Each semantic chunk contributes a cheap key-value summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Output-error bound governed by within-chunk key/value dispersion.
- **Results**: 98% reduction in reconstruction error vs hard dropping; ~3× kernel speedup at 128K context.
- **Pattern**: Error-bounded refinement — "summarize cheap, refine where it matters."

**NeoTrix Mapping**:
| Domain | Integration |
|--------|------------|
| NT-MEMORY | KB embedding: coarse summary for all nodes, exact vectors only for high-relevance chunks (error-bounded retrieval) |
| NT-CORE (HyperCube) | VSA embedding: residual summaries for low-attention concepts, full resolution for high-salience nodes |
| NT-IO | Context window management: compact early context with error bounds, expand only when approximation error exceeds threshold |

**Actionable**: Implement CEDAR's error-bounded residual pattern for NT-MEMORY search — store coarse summaries (cheap), expand to exact vectors only when approximation error exceeds dispersion threshold. Axiom A2 (Context as Scarce Resource) alignment.

---

### 3. CRISP: Cliff-awaRe Input-adaptive Sparse Prefilling
- **arXiv**: 2609.01925 (Sep 1, 2026) — Accepted to EMNLP 2026
- **Key Innovation**: Two structural insights: (1) routing decisions can be read directly from the proxy attention map structure (replacing JSD with O(n) structural proxy), (2) post-softmax mass cliff — cumulative coverage thresholds accumulate O(n) noise. Sink-aware threshold grounded in noise floor eliminates this.
- **Results**: Matches or exceeds dense attention on retrieval tasks; 5.30× attention speedup at 512K tokens.
- **Pattern**: Structural proxy + noise floor awareness — "read the structure, don't compute the divergence."

**NeoTrix Mapping**:
| Domain | Integration |
|--------|------------|
| NT-CORE (GWT) | GWT salience computation: use structural proxy (mass distribution) instead of full KL divergence for routing decisions |
| NT-MIND | Skill routing: structural proxy for task-skill matching instead of full semantic similarity |
| NT-WORLD | Content classification: structural mass analysis for fast content-type routing |

**Actionable**: Replace full JSD-based salience computation in GWT with CRISP's structural proxy for O(n) routing decisions. Apply noise-floor thresholding to attention broadcast to eliminate low-signal broadcasts.

---

### 4. BIGMAS: Brain-Inspired Graph Multi-Agent Systems
- **arXiv**: 2603.15371 (Mar 2026, updated Sep 2026)
- **Key Innovation**: GWT-grounded multi-agent coordination. A GraphDesigner agent constructs task-specific directed agent graphs per problem. All agents coordinate through a centralized shared workspace (GWT). Orchestrator uses complete shared state for routing decisions.
- **Results**: Consistently improves reasoning for both standard LLMs and LRMs. Multi-agent architectural coordination provides gains orthogonal to model-level reasoning. Routing count = natural proxy for instance-level difficulty.
- **Pattern**: Dynamic graph construction + shared workspace — "design the graph per problem, broadcast globally."

**NeoTrix Mapping**:
| Domain | Integration |
|--------|------------|
| NT-CORE (GWT) | Direct GWT implementation pattern: shared workspace + dynamic agent graph construction |
| NT-MIND | SEAL pipeline: GraphDesigner = ConsciousnessTree designing task-specific evolution graphs |
| NT-ACT | Multi-agent orchestration: dynamic agent topology based on task complexity |
| NT-META | Meta-cognition: routing count as difficulty proxy for self-monitoring |

**Actionable**: BIGMAS is the strongest external validation of NeoTrix's GWT design. Integrate GraphDesigner pattern into ConsciousnessTree for per-task agent topology optimization. Use routing count as meta-cognitive difficulty signal.

---

### 5. COMPASS: Context-Organized Multi-Agent Planning and Strategy System
- **ACL 2026** (acl-long.152)
- **Key Innovation**: Three-component hierarchical framework separating tactical execution (Main Agent), strategic oversight (Meta-Thinker), and context organization (Context Manager). Context Manager maintains concise progress briefs across reasoning stages, drawing from notes, trajectory, and strategic signals.
- **Results**: Up to 20% accuracy improvement over single- and multi-agent baselines on GAIA, BrowseComp, and Humanity's Last Exam. Post-training pipeline delegates context management to smaller models.
- **Pattern**: Tactical/strategic/context separation — "execute, oversee, organize as three distinct functions."

**NeoTrix Mapping**:
| Domain | Integration |
|--------|------------|
| NT-CORE | Main Agent = NT-CORE reasoning; Meta-Thinker = NT-META meta-cognition; Context Manager = NT-MEMORY context organization |
| NT-MIND | Strategic oversight = SEAL pipeline monitoring; context briefs = experience-tree distilled summaries |
| NT-IO | Context window management: structured briefs instead of raw history |
| NT-META | Meta-Thinker pattern: async monitoring + strategic interventions |

**Actionable**: Map COMPASS's three components directly to NeoTrix's L5-L6 layers: Main Agent (L5 cognition), Meta-Thinker (L6 meta-cognition), Context Manager (L4 emotion/L3 embodiment boundary). Implement structured context briefs for NT-MEMORY cross-session persistence.

---

## Cross-Paper Synthesis

### Common Patterns Identified

| Pattern | Papers | NeoTrix Integration |
|---------|--------|-------------------|
| **Anchor + Lazy Refinement** | RouteRelay, CEDAR | SEAL pipeline: anchor cycles + intermediate refinement |
| **Error-Bounded Approximation** | CEDAR, CRISP | GWT: error bounds on salience computation |
| **Structural Proxies** | CRISP, RouteRelay | GWT routing: O(n) structural proxies instead of O(n²) divergence |
| **Shared Workspace** | BIGMAS, COMPASS | GWT broadcast: centralized shared state |
| **Dynamic Graph Construction** | BIGMAS | ConsciousnessTree: per-task agent topology |
| **Tactical/Strategic Separation** | COMPASS | 6-layer architecture: L5 tactics, L6 strategy |

### Axiom Alignment

| Axiom | Paper Support |
|-------|-------------|
| **A1: Cost-Aware Routing** | CRISP (O(n) proxy), RouteRelay (lazy rerouting), CEDAR (error-bounded) |
| **A2: Context as Scarce Resource** | CEDAR (residual summaries), COMPASS (structured briefs), CRISP (noise elimination) |
| **A3: Skill as Production Template** | BIGMAS (dynamic graph per problem), COMPASS (separation of concerns) |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| BIGMAS dynamic graph vs fixed NeoTrix architecture | ConsciousnessTree already designs per-task graphs via growth cycles — align with BIGMAS pattern |
| COMPASS context compression vs full context retention | Use CEDAR's error-bounded summaries — compress with guarantees, expand on demand |
| CRISP structural proxy vs full salience computation | Axiom A1 — use proxy for most routing, full computation only for anchor cycles |

---

## Implementation Priority

| # | Paper | Integration Target | Effort |
|---|-------|-------------------|--------|
| 1 | BIGMAS | GWT agent topology optimization | Medium |
| 2 | COMPASS | L5-L6 tactical/strategic separation | Medium |
| 3 | CRISP | GWT O(n) routing proxy | Low |
| 4 | CEDAR | KB search error-bounded retrieval | Low |
| 5 | RouteRelay | SEAL pipeline anchor/lazy refinement | Medium |
