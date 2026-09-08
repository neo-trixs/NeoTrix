# Iteration Batch 761 — Agile/Sprint/PM External Research

## Context
Previous batch (760) established: (1) new MIR move semantics UB for safe-code, (2) concurrent UB detection gap (Miri single-interleave), (3) deny-by-default invalid_runtime_symbol_definitions, (4) post-mono MIR RSS regression.

Batch 761 pivots to external Agile/PM landscape to extract defects and improvements for NeoTrix's own development workflow.

---

## Search 1: Agile 2026 / Scrum 2026 / Kanban 2026

### Sources
1. **agilekoc.com** — "Agile vs Scrum vs Kanban in 2026" (Feb 11 2026)
2. **eitt.academy** — "Scrum vs Kanban 2026" (Apr 29 2026)
3. **echometerapp.com** — "Scrum Best Practices 2026" (Aug 17 2026)
4. **knowledgehut.com** — "Top 10 Agile Trends 2026" (Jan 19 2026)
5. **zignuts.com** — "Kanban vs Scrum 2026" (Jun 4 2025)

### Key Findings → NeoTrix Defects

**Defect 761-A: AI-Experiment Work Pattern Mismatch**
- 2026 data: 94% of IT teams use Agile; 45% of Scrum teams add Kanban elements after 2-3 years (State of Agile 2026). Teams report "AI experiments" as a new work category that is hard to estimate upfront.
- **NeoTrix Impact**: The SEAL pipeline's rhythm_recalculator (`seal::rhythm_recalculator`) uses fixed SegmentType ratios (Setup/Conflict/Climax/Transition) but does not model "AI experiment" work items with high uncertainty. This creates **false predictability** in sprint velocity projections.
- **Fix**: Add `SegmentType::Experiment` variant to `rhythm_recalculator` with entropy multiplier >1.0 for velocity dampening.

**Defect 761-B: WIP-Limit Enforcement Gap**
- 2026 data: Kanban failure mode #1 is "board becomes fancy to-do list; WIP limits ignored" (agilekoc). 70% of Agile teams still use Scrum, but interruptions (incidents/compliance/security) collapse sprint goals.
- **NeoTrix Impact**: `ProductionOrchestrator` (nt_act::production_orchestrator) has no hard WIP limit enforcement. The `ParallelTaskManager` (nt_act::parallel_task) caps GPU concurrent tasks but has no logical WIP cap on task *types* (e.g., too many "quality gate" items blocking value delivery).
- **Fix**: Add `wip_limits: HashMap<TaskCategory, usize>` to `ProductionOrchestrator` config with deny-beyond-cap semantics.

**Defect 761-C: Velocity-as-Performance-Target Anti-Pattern**
- 2026 data: Scrum best practices explicitly warn against "velocity, story points, and utilization as performance targets" (echometerapp). Teams use velocity as quota → overcommitment → trust erosion.
- **NeoTrix Impact**: `BatchProductionManager` (now `ProductionOrchestrator`) uses throughput metrics for batch scheduling but has no guard against using velocity as a performance target. The `ResourceBudgetManager` (nt_act::resource_budget) tracks cost but not team capacity sustainability.
- **Fix**: Add `capacity_sustainability_score()` to `ResourceBudgetManager` that flags when utilization >80% for >3 consecutive cycles.

**Defect 761-D: Prompt-Alignment in Sprint Planning (New Ceremony)**
- 2026 data: Sprint Planning in 2026 includes "Prompt Alignment" — teams decide AI agent context/persona/architectural constraints before sprint begins (zignuts). Prompt Engineering is now a core Agile competency.
- **NeoTrix Impact**: No NeoTrix ceremony exists for aligning AI agent parameters (LLM provider, model, temperature, tool permissions) with sprint goals. The `nt_io::consistency_adapter` and `nt_io::platform_gateway` have no sprint-scoped configuration.
- **Fix**: Add `SprintAgentAlignment` struct to `nt_mind` that snapshots agent parameters at sprint start and diffs at sprint end for retro.

---

## Search 2: Sprint Planning 2026 / Backlog 2026 / Story Point 2026

### Sources
1. **teachingagile.com** — "Sprint Planning Complete Guide" (Jul 14 2026)
2. **ones.com** — "Sprint Planning Framework 2026" (Aug 29 2026)
3. **goodday.work** — "Ultimate Sprint Planning Guide 2026" (Feb 2 2026)
4. **rock.so** — "Sprint Planning: Agenda, Inputs, Examples" (Apr 29 2026)
5. **tasksboard.com** — "Sprint Planning Effective Session 2026" (Apr 6 2026)

### Key Findings → NeoTrix Defects

**Defect 761-E: Definition-of-Ready Gate Missing**
- 2026 data: "Implement a definition of ready checklist that PO must sign off before story enters planning. If not ready, it stays in backlog" (ones.com). Without DoR, stories enter sprint without acceptance criteria → estimation fails.
- **NeoTrix Impact**: `QualityControlPipeline` (nt_meta::quality_control) has quality gates for output but no **input readiness gate** for incoming tasks. `ProductionOrchestrator` accepts any task without pre-validation.
- **Fix**: Add `definition_of_ready: Vec<ReadinessCriterion>` to `ProductionOrchestrator` with `is_ready()` check before task enters active pipeline.

**Defect 761-F: Stretch-Story Overcommitment**
- 2026 data: "If you load up on three stretch stories, you're not stretching; you're gambling. Limit to one, mark as at-risk" (ones.com). Overcommitment erodes trust and predictability.
- **NeoTrix Impact**: `ResourceBudgetManager` (nt_act::resource_budget) has no concept of "stretch" vs "committed" items. All items treated equally in budget allocation.
- **Fix**: Add `ItemPriority::Stretch { risk_flag: bool }` variant and cap stretch items to 1 per cycle in budget allocation.

**Defect 761-G: Technical Debt Capacity Reservation**
- 2026 data: "Reserve 15-20% of capacity for refactoring, bug fixes, and tooling. Treat as non-negotiable investment" (ones.com). Without it, quality decays.
- **NeoTrix Impact**: `RhythmRecalculator` (seal::rhythm_recalculator) allocates segment time by power-law ratio but has no mandatory "technical debt" segment. All cycles are 100% feature work by default.
- **Fix**: Add `tech_debt_reservation_pct: f64` (default 0.15) to `RhythmRecalculator` config and enforce minimum allocation per cycle.

**Defect 761-H: Integration Complexity Underestimation**
- 2026 data: "Teams underestimate integration complexity. Solution: add integration buffer (20-30% of estimated time)" (ones.com). Integration is consistently the #1 underestimated category.
- **NeoTrix Impact**: `StoryboardExtractor` (nt_core::storyboard_extractor) extracts narrative structure but has no integration complexity estimator. `TemporalContinuityChecker` (nt_act::temporal_continuity) checks output quality but not integration effort forecasting.
- **Fix**: Add `integration_complexity_score()` to `ResourceBudgetManager` that applies 1.25x multiplier to cross-module tasks.

**Defect 761-I: Stakeholder Scope-Change Negotiation Protocol**
- 2026 data: "When stakeholder demands new feature, explain adding it jeopardizes sprint goal. Offer to swap with equal-size story or add to next sprint" (ones.com). No formal swap protocol → scope creep.
- **NeoTrix Impact**: `ProductionOrchestrator` has no scope-change negotiation protocol. Task injection is unrestricted.
- **Fix**: Add `ScopeChangeRequest { requested: Task, trade_off: Option<Task> }` with `negotiate_scope_change()` requiring either swap or deferral.

---

## Search 3: Project Management 2026 / Jira 2026 / Linear 2026

### Sources
1. **utilo.io** — "Linear Review 2026: AI Agents" (May 4 2026)
2. **tech-insider.org** — "Linear vs Jira: Why 30% of Teams Switched" (Jun 4 2026)
3. **thesoftwarescout.com** — "Linear vs Jira 2026" (May 24 2026)
4. **atlassian.com** — "Jira Summer 2026 Release" (Aug 6 2026)
5. **community.atlassian.com** — "Jira 2026 Summer Release GA" (Aug 19 2026)

### Key Findings → NeoTrix Defects

**Defect 761-J: Linear Agent Platform — Workspace-Aware AI Gap**
- 2026 data: Linear Agent is "a full agent platform with Cursor and Codex integration" — workspace-aware AI produces "the most useful project management AI responses" (utilo.io). 25,000+ companies.
- **NeoTrix Impact**: NeoTrix's LLM integration (`nt_io`) has no workspace-awareness — the `PlatformGateway` (nt_io::platform_gateway) routes to external platforms but does not inject project context (file tree, dependency graph, recent commits) into prompts.
- **Fix**: Add `WorkspaceContextInjector` to `nt_io::platform_gateway` that serializes project graph into prompt prefix for external LLM calls.

**Defect 761-K: Jira 2026 Summer Release — Capacity Planning & Formula Fields**
- 2026 data: Jira Summer 2026 release adds "capacity planning, formula fields, and major performance upgrades" plus "rebuilt core views from the ground up" (atlassian). AI agents now integrated into core views.
- **NeoTrix Impact**: `BatchProductionManager`/`ProductionOrchestrator` lacks capacity planning visualization. `QualityControlPipeline` has no formula-based metric computation.
- **Fix**: Add `CapacityView` struct to `nt_act::production_orchestrator` and `FormulaMetric { expr: String, bindings: HashMap<String, f64> }` to `nt_meta::quality_control`.

**Defect 761-L: Local-First Sync Architecture**
- 2026 data: Linear uses "local-first sync with optimistic updates, so UI never waits for server round-trip" — sub-100ms response (tech-insider). This is architectural, not just UX.
- **NeoTrix Impact**: `nt_memory` KB operations use synchronous SQLite writes with no optimistic UI update path. The `KB pipeline` blocks on disk I/O.
- **Fix**: Add `OptimisticWriteGuard` to `nt_memory` that returns immediately with projected state and reconciles asynchronously.

**Defect 761-M: Opinionated Workflow Trade-off**
- 2026 data: Linear's "opinionated approach is a double-edged sword. No custom issue states, no time tracking, no offline mode" (utilo). 30% of teams switched to Linear specifically for this.
- **NeoTrix Impact**: NeoTrix's `Skill Tree` (per-domain capability progression) is opinionated (3 tiers: Small/Notable/Keystone) but has no mechanism to express "this domain needs custom tiers" without breaking the model.
- **Fix**: Add `TierOverride { domain: Domain, custom_tiers: Vec<TierDefinition> }` to `SkillTree` config for domains that need non-standard progression.

**Defect 761-N: Migration Cost Blindspot**
- 2026 data: "Migration costs from Jira are real — workflows, integrations, team training. Don't migrate just because Linear is trendy" (thesoftwarescout). Hybrid approach common: Linear for engineering, Jira for portfolio.
- **NeoTrix Impact**: `KB pipeline` has no migration tooling for importing external project data (Jira/Linear exports). `MediaAssetRegistry` (nt_world::media_asset_registry) cannot import from external asset management systems.
- **Fix**: Add `ExternalImportPipeline` to `nt_world` with format adapters for Jira JSON/CSV, Linear JSON exports.

**Defect 761-O: Performance Regression Under Load (Cross-Tool)**
- 2026 data: Jira "can feel sluggish" vs Linear's "sub-100ms". Jira's Feb 2026 Data Center price increase for on-prem customers (tech-insider). Performance is the #1 differentiator.
- **NeoTrix Impact**: This mirrors batch 760's post-mono MIR RSS regression. NeoTrix's `KB pipeline` performance under large dataset loads (10K+ nodes) is not benchmarked against Linear's local-first architecture.
- **Fix**: Add `KBLoadBenchmark` test that measures latency at 1K/10K/100K node scales with write/read/scan operations.

---

## Summary: 15 New Defects Found (761-A through 761-O)

| ID | Category | Severity | Source |
|----|----------|----------|--------|
| 761-A | Planning | Medium | agilekoc, eitt.academy |
| 761-B | WIP Enforcement | High | agilekoc |
| 761-C | Metrics Anti-Pattern | Medium | echometerapp |
| 761-D | Ceremony Gap | Low | zignuts |
| 761-E | Input Quality Gate | High | ones.com |
| 761-F | Budget Allocation | Medium | ones.com |
| 761-G | Debt Reservation | High | ones.com |
| 761-H | Complexity Estimation | Medium | ones.com |
| 761-I | Scope Governance | Medium | ones.com |
| 761-J | AI Integration | High | utilo.io |
| 761-K | Capacity Planning | Medium | atlassian |
| 761-L | Sync Architecture | High | tech-insider |
| 761-M | Workflow Flexibility | Low | utilo.io |
| 761-N | Migration Tooling | Medium | thesoftwarescout |
| 761-O | Performance Benchmark | High | tech-insider |

## Sources Cited
1. https://www.agilekoc.com/en/blog/agile-scrum-kanban-differences-2026
2. https://eitt.academy/knowledge-base/scrum-vs-kanban-agile-frameworks-comparison-2026/
3. https://echometerapp.com/en/scrum-best-practices-2026
4. https://www.knowledgehut.com/blog/agile/top-agile-trends
5. https://zignuts.com/blog/kanban-vs-scrum
6. https://teachingagile.com/scrum/psm-1/scrum-framework/scrum-events/sprint-planning
7. https://ones.com/blog/sprint-planning-in-agile-a-step-by-step-framework-for-2026/
8. https://www.goodday.work/blog/sprint-planning-guide/
9. https://www.rock.so/blog/sprint-planning
10. https://tasksboard.com/blog/sprint-planning-guide
11. https://utilo.io/blog/linear-review-2026-project-management
12. https://tech-insider.org/linear-vs-jira-2026/
13. https://thesoftwarescout.com/linear-vs-jira-2026-which-project-management-tool-is-right-for-your-team/
14. https://www.atlassian.com/blog/development/jira-summer-release
15. https://community.atlassian.com/forums/Jira-articles/Introducing-the-Jira-2026-Summer-Release-%EF%B8%8F/ba-p/3268349
