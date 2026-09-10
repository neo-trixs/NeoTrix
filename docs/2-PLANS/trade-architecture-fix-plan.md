# Trade 模块架构补齐 — 完整推进方案 v2

> 审计时间: 2026-09-09 | 发现总数: 28 (C:4 H:8 M:9 L:7)

## 1. 架构缺陷清单

### 1.1 Critical — 编译阻断

| # | 缺陷 | 位置 | 修复 |
|---|------|------|------|
| C1 | test 用 phantom `CollectionType`/`amount_collected` 字段 | finance_compliance.rs:1012 | 重写为实际 CollectionRecord 字段 |
| C2 | test 用 phantom `ContractReviewRecord`/`LcStatus` | finance_compliance.rs:1040 | 删除或改用现有类型 |
| C3 | test 用 phantom `ProductionOrder` 字段 + `calculate_kpi` | production_logistics.rs:1086 | 删除 phantom 字段，添加 calculate_kpi |
| C4 | test 调用不存在的 `Milestone::new()` | trade_core.rs:879 | 添加 Milestone::new 构造器 |

### 1.2 High — 类型重复 (8对)

| # | 重复类型 | full_cycle.rs | 其他文件 | 修复 |
|---|---------|---------------|---------|------|
| H1 | `PaymentProof` | full_cycle:322 (简化) | finance_compliance:44 (完整) | 删除 full_cycle 版本 |
| H2 | `LcReview` | full_cycle:333 (简化) | finance_compliance:115 (完整) | 删除 full_cycle 版本 |
| H3 | `CollectionRecord` | full_cycle:341 (简化) | finance_compliance:155 (完整) | 删除 full_cycle 版本 |
| H4 | `SettlementRecord` | full_cycle:349 (简化) | finance_compliance:197 (完整) | 删除 full_cycle 版本 |
| H5 | `TaxRefundClaim` | full_cycle:356 (简化) | finance_compliance:224 (完整) | 删除 full_cycle 版本 |
| H6 | `BuyerProfile` | full_cycle:145 | orchestrator:241 (不同) | orchestrator 重命名 |
| H7 | `Contract` | full_cycle:276 | orchestrator:269 (不同) | orchestrator 重命名 |
| H8 | `TradeContext` | full_cycle:136 | orchestrator:213 (不同) | orchestrator 重命名 |

### 1.3 Medium — 逻辑缺陷 + 缺失 trait

| # | 缺陷 | 位置 | 修复 |
|---|------|------|------|
| M1 | LC 风险分数 0.0-1.0 转 u32 匹配 0-100 阈值 | finance_compliance.rs:412 | 改为 0.0-1.0 匹配 |
| M2 | `risk_score < 50.0` 对 0.0-1.0 范围永远为真 | finance_compliance.rs:327 | 改为 `< 0.50` |
| M3 | 24 个公开枚举缺 Display | 全模块 | 添加 Display impl |
| M4 | 15 个公开结构体缺 Default | 全模块 | 添加 Default |
| M5 | orchestrator QuoteGenerator phantom `cost_breakdown` 字段 | orchestrator:402 | 删除该字段 |
| M6 | orchestrator 用 `PaymentType::TT` 但不存在 | orchestrator:1322 | 改为 `Deposit` |
| M7 | orchestrator test phantom struct fields | orchestrator:1396 | 改为实际字段 |
| M8 | orchestrator `BomRequirement` fields 不匹配 | orchestrator:1327 | 改为实际字段 |

### 1.4 Low — 风格/存根

| # | 缺陷 | 位置 | 修复 | 状态 |
|---|------|------|------|------|
| L1 | mod.rs re-export 名称冲突 | mod.rs:19-60 | 停止导出简化版本 + 添加 orchestrator 类型 | ✅ |
| L2 | `progress_pct * 100.0` 命名歧义 | orchestrator:821 | 无需修复 |
| L3 | `check_discrepancies` 始终返回空 | finance_compliance:465 | 保持 stub |
| L4 | `ProductionMilestone` 别名冗余 | production_logistics:101 | 保持 |
| L5 | double-negative 逻辑 | finance_compliance:328 | 改为 `!any` |
| L6 | `execute_trade_full_cycle` 是骨架 | full_cycle:524 | 保持 |
| L7 | FT26 后 phase 重置为 FT25 | orchestrator:1094 | 保持 |

---

## 2. 推进方案 (6 Phase)

### Phase 1: 修复编译阻断 — Critical (C1-C4)

**目标**: 4 个 phantom 测试改为可编译

| 任务 | 文件 | 操作 |
|------|------|------|
| C1 | finance_compliance.rs:1012 | 删除 test_complete_settlement_uses_trade_core，改用正确字段重写 |
| C2 | finance_compliance.rs:1040 | 删除 test_review_risk_scoring_comprehensive，改用正确类型 |
| C3 | production_logistics.rs:1086 | 删除 test_kpi_calculation_coverage，改用正确字段 |
| C4 | trade_core.rs:879 | 添加 `Milestone::new()` 构造器 |

**预估**: 30min

---

### Phase 2: 消除类型重复 — High (H1-H8)

**目标**: 每个类型唯一事实源，消除 re-export 冲突

#### 2.1 full_cycle.rs 简化类型删除 (H1-H5)

删除 full_cycle.rs 中的 5 个简化版本，只保留 finance_compliance.rs 的完整版本:
- `PaymentProof` → 删除 (full_cycle:322)
- `LcReview` → 删除 (full_cycle:333)
- `CollectionRecord` → 删除 (full_cycle:341)
- `SettlementRecord` → 删除 (full_cycle:349)
- `TaxRefundClaim` → 删除 (full_cycle:356)

更新 full_cycle.rs 中对这些类型的引用改为 `super::finance_compliance::Xxx`。

#### 2.2 orchestrator 类型重命名 (H6-H8)

orchestrator.rs 中的 3 个类型与 full_cycle 同名但不同:
- `BuyerProfile` → `OrchBuyerProfile`
- `Contract` → `OrchContract`
- `TradeContext` → `OrchTradeContext`

**预估**: 1h

---

### Phase 3: 修复逻辑缺陷 — Medium (M1-M2, M5-M8)

**目标**: 风险评分阈值正确，orchestrator phantom fields 清理

| 任务 | 文件 | 操作 |
|------|------|------|
| M1 | finance_compliance.rs:412 | `match risk_score as u32 { 0..=1 => Accept, .. }` → 改为 `if risk_score < 0.25 { Accept }` 等 |
| M2 | finance_compliance.rs:327 | `risk_score < 50.0` → `risk_score < 0.50` |
| M5 | orchestrator.rs:402 | 删除 QuoteGenerator 的 `cost_breakdown` 字段 |
| M6 | orchestrator.rs:1322 | `PaymentType::TT` → `PaymentType::Deposit` |
| M7 | orchestrator.rs:1396 | 改为实际 struct fields |
| M8 | orchestrator.rs:1327 | BomRequirement 改为实际字段 |

**预估**: 1h

---

### Phase 4: 添加 Display + Default — Medium (M3-M4)

**目标**: 公开枚举有 Display，公开结构体有 Default

#### 4.1 Display impls (选择关键枚举)

| 枚举 | 文件 |
|------|------|
| `TradePhase` | full_cycle.rs |
| `TradePhaseGroup` | full_cycle.rs |
| `TradePhase26` | orchestrator.rs |
| `TradeGroup` | orchestrator.rs |
| `FindingSeverity` | finance_compliance.rs |
| `LcRecommendation` | finance_compliance.rs |
| `CollectionStatus` | finance_compliance.rs |
| `RefundStatus` | finance_compliance.rs |
| `MaterialStatus` | production_logistics.rs |
| `InspectionResult` | production_logistics.rs |
| `ObjectionCategory` | quote_negotiation.rs |

#### 4.2 Default impls (选择关键结构体)

| 结构体 | 文件 |
|--------|------|
| `Milestone` | trade_core.rs |
| `RiskFinding` | trade_core.rs |
| `ContractReview` | finance_compliance.rs |
| `InspectionReport` | production_logistics.rs |
| `QuoteSheet` | full_cycle.rs |

**预估**: 1h

---

### Phase 5: mod.rs re-export 清理 (L1)

**目标**: 消除 re-export 名称冲突

修改 mod.rs，停止从 full_cycle 导出已删除的简化类型。只保留:
- full_cycle 的领域类型: TradePhase, TradeContext (非orchestrator), Contract, BuyerProfile 等
- finance_compliance 的完整类型: PaymentProof, LcReview, CollectionRecord 等
- orchestrator 的重命名类型: OrchBuyerProfile, OrchContract, OrchTradeContext

**预估**: 30min

---

### Phase 6: 编译验证 + 测试

1. `cargo check -p neotrix --lib` — 全量编译
2. `cargo test -p neotrix --lib -- nt_act_trade` — trade 模块测试
3. 清理未使用 import

**预估**: 30min

---

## 3. 工时估算

| Phase | 工时 | 依赖 |
|-------|------|------|
| Phase 1: Critical 修复 | 30min | 无 |
| Phase 2: 类型去重 | 1h | Phase 1 |
| Phase 3: 逻辑缺陷 | 1h | Phase 1-2 |
| Phase 4: Display/Default | 1h | Phase 1-2 |
| Phase 5: mod.rs 清理 | 30min | Phase 2 |
| Phase 6: 编译验证 | 30min | Phase 1-5 |
| **总计** | **4.5h** | |

## 4. 验收标准

- [x] 4 个 Critical phantom test 已修复
- [x] 8 对重复类型已消除
- [x] LC 风险评分阈值正确 (0.0-1.0 范围)
- [x] orchestrator phantom fields 已清理
- [x] 关键枚举有 Display
- [x] mod.rs re-export 无名称冲突
- [ ] `cargo check -p neotrix --lib` 零 trade 相关 error
- [ ] 全部 trade 测试可编译通过
