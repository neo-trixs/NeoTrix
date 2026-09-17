# Trade Module Redundancy Report

**Date**: 2026-09-16
**Scope**: `neotrix-core/src/l1_action/nt_act/nt_act_trade/`

---

## Engine Inventory

| Engine | File | Lines | Purpose | Overlaps With |
|--------|------|-------|---------|---------------|
| TradeOrchestrator (v1) | orchestrator.rs | 1119 | Full trade lifecycle orchestrator, FT01-FT26, L1 capability composition | orchestrator_v2.rs, process_engine.rs |
| TradeOrchestrator (v2) | orchestrator_v2.rs | 1403 | Multi-agent orchestrator with TradeRouter, WorkerPool, async execution | orchestrator.rs |
| ProcessEngine | process_engine.rs | 1224 | Generic process orchestration framework | orchestrator.rs (phase management) |
| NegotiationEngine | trade_core.rs | 931 | Core algorithms: StateMachine, CostCalculator, NegotiationEngine, RiskAssessor, ProgressTracker | orchestrator.rs (quotation/negotiation re-impl) |
| FinanceEngine | finance_compliance.rs | 958 | Contract review, payment, LC review, settlement, tax refund | orchestrator.rs (FT10-FT11, FT21-FT24) |
| ProductionEngine | production_logistics.rs | 997 | Production orders, progress, inspection | orchestrator.rs (FT12-FT15) |
| LogisticsEngine | production_logistics.rs | (same file) | Booking, customs, bill of lading, shipment tracking | orchestrator.rs (FT16-FT20) |
| TradeCrmEngine | nt_trade_crm.rs | 768 | Customer/company/contact management, grading, dedup | orchestrator.rs (lead management overlap) |
| TradePipelineEngine | nt_trade_pipeline.rs | 553 | Sales pipeline, deal stages, win rate, conversion | orchestrator.rs (pipeline_summary overlap) |
| TradeEmailEngine | nt_trade_email.rs | 445 | Email templates, tracking, bulk send | — |
| TradeDocumentEngine | nt_trade_documents.rs | 435 | Trade document generation, review, document sets | — |
| TradeTaskEngine | nt_trade_tasks.rs | 353 | Task creation, reminders, calendar view | — |
| TradeDashboardEngine | nt_trade_dashboard.rs | 382 | Analytics dashboard, funnel, revenue, customer analytics | — |
| SupplierMgmtEngine | nt_trade_supplier_mgmt.rs | 315 | Supplier evaluation, AVL, comparison | — |

---

## Redundancy Analysis

### 1. `orchestrator.rs` vs `orchestrator_v2.rs` — Duplicate TradeOrchestrator

**Severity**: HIGH

Both files define a `TradeOrchestrator` struct. They represent fundamentally different architectures:

| Aspect | v1 (orchestrator.rs) | v2 (orchestrator_v2.rs) |
|--------|----------------------|-------------------------|
| Style | Synchronous, monolithic | Async multi-agent (tokio) |
| Pattern | L1 capability composition | Orchestrator-Worker + Router |
| Task management | HashMap of OrchTradeContext | TaskTracker with DAG dependencies |
| Error handling | Result<T, String> | Retry + circuit breaker |
| Concurrency | None | Semaphore-based parallelism |
| Message bus | None | mpsc channel with TradeMessage events |
| Phase model | Hardcoded 26-phase enum | Dynamic task decomposition |

**Conclusion**: v2 supersedes v1 for production use. v1 is a legacy implementation retained for backward compatibility.

### 2. `orchestrator.rs` Re-implements Sub-Engine Methods

**Severity**: HIGH

`TradeOrchestrator` v1 contains methods that duplicate logic already provided by the sub-engines it composes:

| Orchestrator Method | Already In | Duplication |
|---------------------|------------|-------------|
| `generate_quotation()` (FT07) | `quote_negotiation::QuoteGenerator` | Quotation creation logic duplicated |
| `handle_objection()` (FT08) | `trade_core::NegotiationEngine` | Negotiation handling partially reimplemented |
| `create_production_order()` (FT12) | `ProductionEngine::create_production_order()` | Production order creation duplicated |
| `track_production()` (FT13) | `ProductionEngine::generate_progress_report()` | Progress tracking duplicated |
| `_quality_inspect()` (FT14) | `ProductionEngine::perform_inspection()` | Quality inspection duplicated |
| `apply_inspection_cert()` (FT16) | `LogisticsEngine::apply_ciq()` | CIQ certificate creation duplicated |
| `book_and_pack()` (FT17) | `LogisticsEngine::book_and_pack()` | Direct delegation (no duplication) |
| `customs_clearance()` (FT18) | `LogisticsEngine::customs_clearance()` | Direct delegation (no duplication) |
| `_manage_bill_of_lading()` (FT19) | `LogisticsEngine::issue_bill_of_lading()` | BL management duplicated |
| `track_shipment()` (FT20) | `LogisticsEngine::track_shipment()` | Direct delegation (no duplication) |
| `verify_settlement()` (FT22) | `FinanceEngine::verify_settlement()` | Direct delegation (no duplication) |
| `declare_tax_refund()` (FT23) | `FinanceEngine::apply_tax_refund()` | Tax refund duplicated |
| `reconcile_accounts()` (FT24) | `FinanceEngine::reconcile_accounts()` | Direct delegation (no duplication) |

The orchestrator should be a thin orchestration layer that delegates to sub-engines, not re-implement their logic.

### 3. `RiskLevel` Enum Defined 4 Times

**Severity**: MEDIUM

| Location | Definition | Lines |
|----------|-----------|-------|
| `trade_core.rs:385` | `pub enum RiskLevel { Low, Medium, High, Critical }` with `.score()` method | Canonical (SSOT) |
| `full_cycle.rs:576` | `pub enum RiskLevel { Low, Medium, High, Critical }` | Duplicate |
| `production_logistics.rs:308` | `pub enum RiskLevel { Low, Medium, High, Critical }` | Duplicate |
| `capabilities/risk_assessor.rs:47` | `pub enum RiskLevel { Low, Medium, High, Critical }` | Duplicate |

`trade_core.rs` is the canonical SSOT (see `mod.rs:69` re-export). All other definitions should be removed and replaced with `use super::trade_core::RiskLevel`.

### 4. `orchestrator.rs` vs `process_engine.rs` — Phase Management Overlap

**Severity**: LOW

| Aspect | orchestrator.rs | process_engine.rs |
|--------|----------------|-------------------|
| Phase model | Hardcoded `TradePhase26` enum | Generic `ProcessDefinition` with steps |
| State tracking | `OrchTradeContext` per trade | `ProcessInstance` with step results |
| Step execution | Direct method calls | `StepHandler` trait dispatch |
| Conditional steps | None | `StepCondition` with evaluation |
| Retry | None | `max_retries` per step |
| Event system | `Vec<TradeEvent>` inline | `EventBus` with subscribers |

`ProcessEngine` is a more general-purpose framework that could replace the phase management in `orchestrator.rs`, but they serve different abstraction levels. `orchestrator.rs` is domain-specific while `process_engine.rs` is reusable infrastructure.

---

## Consolidation Recommendations

### High Priority (Merge Now)

1. **`orchestrator.rs` → Deprecate, migrate to `orchestrator_v2.rs`**
   - Reason: v2 provides async execution, retry, parallelism, and dynamic task decomposition
   - Action: Add `#[deprecated]` to v1 `TradeOrchestrator`, update imports to use `orchestrator_v2::TradeOrchestrator`
   - Risk: v1 has 13 active tests; ensure v2 coverage matches

2. **Remove duplicate `RiskLevel` definitions**
   - Keep: `trade_core.rs` (canonical SSOT)
   - Remove from: `full_cycle.rs`, `production_logistics.rs`, `capabilities/risk_assessor.rs`
   - Action: Replace local definitions with `use super::trade_core::RiskLevel`

### Medium Priority (Future)

3. **Refactor `orchestrator.rs` to be a thin facade**
   - Remove all method bodies that duplicate sub-engine logic
   - Keep only orchestration sequencing (phase transitions, context management)
   - Delegate actual work to `ProductionEngine`, `LogisticsEngine`, `FinanceEngine`

4. **Merge `orchestrator.rs` type aliases into `orchestrator_v2.rs`**
   - Move `TradePhase26`, `OrchTradeContext`, `Quotation`, etc. into v2 or a shared types module
   - v2 already has its own task model; reconcile the two

### Low Priority (Keep Separate)

5. **`process_engine.rs` — Keep as-is**
   - Generic infrastructure, not redundant with domain engines
   - Could be used as foundation for v2's phase management if needed

6. **Domain engines (CRM, Pipeline, Email, Documents, Tasks, Dashboard, SupplierMgmt)**
   - All distinct responsibilities, no overlap between them
   - Each manages a separate concern

---

## Migration Plan

### Phase 1: Deprecation Comments (Immediate)
- Add `#[deprecated]` to v1 `TradeOrchestrator` struct
- Add deprecation comments to duplicate `RiskLevel` definitions
- Add deprecation comments to duplicated methods in `orchestrator.rs`

### Phase 2: Import Migration (1-2 weeks)
- Update all callers of `orchestrator::TradeOrchestrator` to use `orchestrator_v2::TradeOrchestrator`
- Update `mod.rs` re-exports to prioritize v2
- Remove duplicate `RiskLevel` from `full_cycle.rs`, `production_logistics.rs`, `capabilities/risk_assessor.rs`

### Phase 3: Code Cleanup (After all imports updated)
- Remove v1 `TradeOrchestrator` struct and all its methods
- Remove duplicate `RiskLevel` definitions
- Refactor remaining orchestrator v1 types (TradePhase26, OrchTradeContext) into shared types

### Phase 4: ProcessEngine Integration (Optional)
- Evaluate if `ProcessEngine` can replace remaining phase management in v2
- If yes, define trade-specific `ProcessDefinition` for FT01-FT26

---

## Files Modified

| File | Change |
|------|--------|
| `orchestrator.rs` | Added deprecation comment on `TradeOrchestrator` |
| `orchestrator.rs` | Added deprecation comments on duplicated methods |
| `production_logistics.rs` | Added deprecation comment on `RiskLevel` enum |
| `full_cycle.rs` | Added deprecation comment on `RiskLevel` enum (not in this scan, mark separately) |
| `capabilities/risk_assessor.rs` | Added deprecation comment on `RiskLevel` enum (not in this scan, mark separately) |
| `REDUNDANCY_REPORT.md` | Created |
