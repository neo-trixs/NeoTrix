# Research Batch 661 — 6-Repo Fusion Analysis

**Date**: 2026-09-13
**Repos Analyzed**: training-agents, worktrunk, vivid-figures-skill, trends-research, open-code-review, PrinterService

---

## 1. training-agents (⭐124)

**URL**: https://github.com/burtenshaw/training-agents
**Stack**: Python, TRL, Codex agents
**Domain**: Agent post-training resource collection

### Architecture Patterns
- **Public Codex context pattern**: Reusable AGENTS.md + skill definitions as composable training modules
- **4-stage tutorial ladder**: SFT → Distillation → RL → Environments (progressive complexity)
- **Workspace isolation**: Explicit "workspaces/" exclusion from tracked repo — separates data from instructions
- **Loop-shaped RL**: `docs/looping-rl.md` defines iterative agent improvement cycles

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| Workspace isolation (data outside repo) | SEAL pipeline temp workspaces — enforce workspace/ exclusion in gitignore rules | P1 |
| 4-stage tutorial ladder (SFT→Distill→RL→Env) | NT-MIND skill crystallization stages: learn→distill→practice→autonomize | P2 |
| Loop-shaped RL for agent improvement | ConsciousnessTree feedback loop — agent behavior shaped by accumulated experience | P2 |

### Key Insight
Training agents is about **structured progressive exposure** — the same pattern as NeoTrix's constellation maturity ladder (C0→C6). Both systems increase autonomy through staged capability gates.

---

## 2. worktrunk (⭐7.4k)

**URL**: https://github.com/max-sixty/worktrunk
**Stack**: Rust, CLI, Git worktree management
**Domain**: Parallel AI agent workflow infrastructure

### Architecture Patterns
- **Branch-addressed worktrees**: Paths computed from configurable template, worktrees addressed by branch name
- **Hook lifecycle**: create → pre-merge → post-merge hooks for workflow automation
- **Build cache sharing**: APFS/btrfs/XFS reflinks for zero-copy target/ and node_modules/ sharing across worktrees
- **LLM commit messages**: Diff-based auto-generated commit messages
- **Interactive picker**: Live diff and log previews per worktree
- **Template filters**: `hash_port` for unique per-worktree dev server ports

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| Branch-addressed worktrees with path templates | NT-ACT parallel task isolation — each agent session gets own worktree, addressed by session_id | P0 |
| Build cache sharing via reflinks | NeoTrix build optimization — share target/ across parallel cargo builds | P0 |
| Hook lifecycle (create/merge) | NT-SHIELD pre-commit hooks + SEAL pipeline stage gates | P1 |
| `wt list` with CI status + LLM summaries | HeartbeatAggregator — aggregate build/test/health across parallel workspaces | P1 |
| Interactive picker with live previews | NT-IO interactive workspace selector for multi-agent coordination | P2 |

### Key Insight
Worktrunk solves the **same isolation problem** as NeoTrix's dual specialization (POE Weapon Sets). Each agent/worktree is isolated but shares build infrastructure. The `hash_port` template pattern is directly applicable to NeoTrix's web server per-worktree pattern.

---

## 3. vivid-figures-skill (⭐132)

**URL**: https://github.com/yjz211/vivid-figures-skill
**Stack**: Python, matplotlib/seaborn, SKILL.md spec
**Domain**: AI-assisted scientific figure generation

### Architecture Patterns
- **108 recipe system**: Encoded chart templates as data-driven configurations
- **Dual-style framework**: "Vivid" vs "Stable" mode — same data, different visual expression
- **5 palette families**: Coral-teal, Ocean-orange, Iris-apricot, Forest-daylight, Berry-iceblue
- **Recipe-as-code**: Templates preserve gradients, transparency layers, key graphic elements
- **AI-resistant simplification**: Explicit instructions preventing AI from collapsing template complexity
- **Image + source code dual delivery**: PNG for preview, PDF for typesetting, Python for reproducibility

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| 108 recipe system (data-driven chart configs) | NT-IO design template library — SKILL.md recipes for NeoTrix UI generation | P1 |
| Dual-style framework (vivid/stable) | EmotionLabel dual expression — Joy/Neutral modes for same data, different emotional tone | P2 |
| AI-resistant template preservation | NT-SHIELD anti-slop guard — prevent AI from oversimplifying complex visual patterns | P1 |
| Image + source dual delivery | File ability dual output — both rendered artifact AND reproducible source | P2 |

### Key Insight
The "recipe-as-code" pattern maps to NeoTrix's **skill crystallization** — each recipe is a frozen, composable unit. The anti-simplification guard is directly relevant to preventing AI slop in design capabilities.

---

## 4. trends-research (⭐72)

**URL**: https://github.com/Lucas-Joly-GH/trends-research
**Stack**: Python, C++ (pybind11), PyTorch, NumPyro/JAX, LaTeX
**Domain**: Systematic momentum trading research — academic thesis

### Architecture Patterns
- **45-signal alpha library**: Each signal independently backtested, ranked by Sharpe
- **Multi-alpha composite**: Equal-weight combination survives where individual signals don't
- **Deflated Sharpe test**: Bailey & López de Prado — honest multiple-testing correction
- **GAN robustness pipeline**: Conditional WGAN generates synthetic market paths to stress-test strategy
- **Zero-parameter philosophy**: All values from first principles or literature citations
- **Walk-forward P&L**: Day-by-day compounding with C++ accelerator

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| 45-signal library with individual backtesting | NT-MIND skill library — each skill independently validated before composite use | P2 |
| Multi-alpha composite (equal-weight) | GWT salience aggregation — multiple attention signals combined for routing | P2 |
| Deflated Sharpe / multiple-testing correction | ConsciousnessTree — honest meta-evaluation of capability claims | P1 |
| GAN synthetic market stress test | SEAL pipeline adversarial testing — generate synthetic failure modes | P1 |
| Zero-parameter first-principles derivation | NeoTrix axiom system (A1-A3) — derive from axioms, don't fit to data | P0 |

### Key Insight
The **Deflated Sharpe** methodology is directly applicable to NeoTrix's meta-evaluation — when you test many capabilities, you need honest correction for multiple comparisons. The GAN robustness pattern maps to SEAL's self-testing.

---

## 5. open-code-review (⭐22.8k)

**URL**: https://github.com/alibaba/open-code-review
**Stack**: Go, LLM agent, deterministic pipeline hybrid
**Domain**: AI-powered code review CLI

### Architecture Patterns
- **Hybrid architecture**: Deterministic pipelines (file selection, bundling, rule matching) + LLM agent (dynamic decisions)
- **Smart file bundling**: Related files grouped into review units — divide-and-conquer on large changesets
- **Fine-grained rule matching**: Template-engine-based rule-to-file mapping (not language-driven)
- **External positioning & reflection modules**: Independent comment-positioning and content-reflection
- **Scenario-tuned toolset**: Distilled from production tool-call traces — purpose-built for code review
- **Delegation mode**: OCR handles file selection/rule resolution; host agent performs actual review
- **90% coverage threshold**: Enforced via `make coverage`

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| Hybrid deterministic + agent architecture | NT-SHIELD/NX-META audit pipeline — deterministic scanning + LLM interpretation | P0 |
| Smart file bundling for large changesets | SEAL pipeline task decomposition — group related changes for parallel review | P1 |
| External positioning & reflection modules | NT-REPAIR independent reflection — separate "what changed" from "what it means" | P1 |
| Delegation mode (OCR selects, agent reviews) | NT-ACT capability delegation — router selects files, specialist modules execute | P0 |
| Scenario-tuned toolset from production traces | NT-MIND tool crystallization — distill tool usage patterns from KB experience | P1 |
| 90% coverage gate | Constellation C2 integration test gate — enforce coverage threshold | P2 |

### Key Insight
The **deterministic-first, agent-second** architecture is NeoTrix's strongest fusion candidate. OCR's hybrid model proves that hard engineering constraints improve agent quality. The delegation mode (separate file selection from review execution) maps directly to NT-ACT's CapabilityRegistry routing.

---

## 6. PrinterService (⭐1.1k)

**URL**: https://github.com/cp9no1/PrinterService
**Stack**: Python, Flask/bottle, system tray, PyInstaller
**Domain**: LAN printing service

### Architecture Patterns
- **Web-to-print bridge**: Browser uploads → server converts to PDF → prints on connected printer
- **Multi-format auto-conversion**: doc/docx/xls/xlsx/ppt/pptx/txt/md/log/jpg/png/gif → PDF
- **System tray daemon**: Background service with tray icon, no UI window
- **Drag-and-drop upload**: File management with preview/delete
- **Offline packaging**: All styles bundled, no online resource dependency

### Fusion Opportunities
| Source Pattern | NeoTrix Mapping | Priority |
|---|---|---|
| Multi-format auto-conversion (anything → PDF) | nt_file_ability unified conversion pipeline — extend FileModel to route through conversion | P1 |
| System tray daemon pattern | NT-IO background service mode — tray icon for daemon status | P3 |
| Offline packaging (all assets bundled) | NT-SHIELD sandbox — all dependencies self-contained, no external fetch | P2 |

### Key Insight
PrinterService is a narrow utility but validates the **universal conversion pattern** — any input format → standardized intermediate (PDF) → output. This maps to NeoTrix's FileModel pipeline where diverse inputs normalize to a common representation.

---

## Cross-Repo Synthesis

### Top 5 Fusion Priorities

| Priority | Pattern | Sources | NeoTrix Target |
|---|---|---|---|
| **P0** | Deterministic-first + agent-second hybrid | open-code-review + worktrunk | NT-SHIELD audit pipeline, NT-ACT capability routing |
| **P0** | Worktree isolation for parallel agents | worktrunk | NT-ACT parallel task management |
| **P1** | Delegation mode (select → specialist execute) | open-code-review | CapabilityRegistry routing pattern |
| **P1** | Deflated evaluation / honest meta-assessment | trends-research | ConsciousnessTree meta-evaluation |
| **P1** | AI-resistant template preservation | vivid-figures-skill | NT-SHIELD anti-slop guard |

### Cross-Source Patterns Identified

| Pattern | Definition | Repos | NeoTrix Mapping |
|---|---|---|---|
| **P6: Hybrid Deterministic-Agent** | Hard engineering for invariants, LLM for dynamic decisions | open-code-review, worktrunk | GWT routing + deterministic pre-filter |
| **P7: Recipe-as-Code** | Frozen, composable, versioned expert templates | vivid-figures-skill, training-agents | SKILL-SPEC.md contract |
| **P8: Honest Multiple-Testing** | Correct statistical claims when evaluating many hypotheses | trends-research | ConsciousnessTree capability validation |
| **P9: Universal Format Normalization** | Diverse inputs → single intermediate representation | PrinterService, nt_file_ability | FileModel pipeline |
| **P10: Progressive Autonomy Ladder** | Staged capability gates from supervised to autonomous | training-agents, worktrunk, trends-research | Constellation C0→C6 maturity |

### Contradictions & Resolutions

| Tension | Resolution |
|---|---|
| open-code-review Go vs NeoTrix Rust | Absorb methodology (hybrid architecture pattern), not code |
| PrinterService Python vs NeoTrix Rust | File format conversion pattern is language-agnostic |
| vivid-figures-skill non-commercial license | Pattern observation only, no code absorption |
| trends-research proprietary data dependency | Statistical methodology (Deflated Sharpe) is universally applicable |

---

## Implementation Recommendations

1. **Immediate** (this cycle): Add deterministic pre-filter to NT-SHIELD audit pipeline (P0)
2. **Short-term**: Implement delegation mode in CapabilityRegistry (P1)
3. **Medium-term**: Add Deflated Sharpe-equivalent to ConsciousnessTree capability validation (P1)
4. **Long-term**: Progressive autonomy ladder aligned with constellation maturity (P2)
