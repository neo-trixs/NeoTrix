# Trade 模块能力融合设计

## 1. 底层算法公式分析

### 1.1 核心算法模式

| 算法模式 | 公式/逻辑 | 应用场景 |
|----------|-----------|----------|
| **状态机转换** | `P(next) = f(P(current), guard)` | 阶段流转 |
| **成本计算** | `Total = Σ(Material + Labor + Overhead + ...)` | 报价生成 |
| **利润边际** | `Price = Cost × (1 + Margin)` | 定价策略 |
| **让步策略** | `Concession = (Current - BottomLine) × Rate` | 谈判博弈 |
| **风险评分** | `Risk = Σ(Severity × Weight)` | 风险评估 |
| **进度偏差** | `Deviation = Actual - Planned` | 进度跟踪 |
| **AQL 抽样** | `SampleSize = f(LotSize, AQLLevel)` | 质量检验 |

### 1.2 推理逻辑核心

```
输入 → 状态判断 → 规则匹配 → 输出动作
  ↓           ↓           ↓
Context    StateMachine  RuleEngine
```

## 2. 冗余识别

### 2.1 类型冗余

| 冗余类型 | 位置 | 说明 |
|----------|------|------|
| `QuoteSheet` | full_cycle + quote_negotiation | 重复定义 |
| `IntentLevel` | full_cycle + quote_negotiation | 重复定义 |
| `MilestoneStatus` | full_cycle + production_logistics | 重复定义 |
| `InspectionReport` | full_cycle + production_logistics | 重复定义 |
| `Defect` | full_cycle + production_logistics | 重复定义 |
| `DefectSeverity` | full_cycle + production_logistics | 重复定义 |

### 2.2 逻辑冗余

| 冗余逻辑 | 位置 | 说明 |
|----------|------|------|
| 成本计算 | quote_negotiation | 与 full_cycle 重复 |
| 风险评估 | 多处 | 分散实现 |
| 进度跟踪 | production_logistics | 与 full_cycle 重复 |

## 3. 能力融合方案

### 3.1 统一骨架设计

```rust
// trade_core.rs - 统一算法骨架

/// 状态机骨架
pub struct StateMachine<S: State> {
    current: S,
    history: Vec<S>,
    guards: Vec<Box<dyn Guard<S>>>,
}

/// 成本计算器骨架
pub struct CostCalculator {
    components: Vec<Box<dyn CostComponent>>,
    margin_strategy: Box<dyn MarginStrategy>,
}

/// 风险评估器骨架
pub struct RiskAssessor {
    rules: Vec<Box<dyn RiskRule>>,
    weights: HashMap<String, f64>,
}

/// 进度跟踪器骨架
pub struct ProgressTracker {
    milestones: Vec<Milestone>,
    critical_path: Vec<String>,
}
```

### 3.2 算法公式统一

#### 成本计算公式
```rust
// 统一成本计算
pub fn calculate_total_cost(
    material: f64,
    labor: f64,
    overhead_ratio: f64,
    packaging: f64,
    logistics: f64,
    certification: f64,
    contingency_ratio: f64,
) -> f64 {
    let overhead = (material + labor) * overhead_ratio;
    let subtotal = material + labor + overhead + packaging + logistics + certification;
    let contingency = subtotal * contingency_ratio;
    subtotal + contingency
}

// 统一定价公式
pub fn calculate_price(cost: f64, margin: f64) -> f64 {
    cost * (1.0 + margin)
}
```

#### 让步策略公式
```rust
// 统一让步策略
pub fn calculate_concession(
    current_price: f64,
    bottom_line: f64,
    concession_rate: f64,
    max_rounds: u32,
    current_round: u32,
) -> Option<f64> {
    if current_price > bottom_line * 1.05 && current_round < max_rounds {
        let concession_amount = (current_price - bottom_line) * concession_rate;
        Some(current_price - concession_amount)
    } else {
        None
    }
}
```

#### 风险评分公式
```rust
// 统一风险评分
pub fn calculate_risk_score(
    findings: &[RiskFinding],
    weights: &HashMap<String, f64>,
) -> f64 {
    findings.iter()
        .map(|f| {
            let weight = weights.get(&f.category).copied().unwrap_or(1.0);
            f.severity_score() * weight
        })
        .sum::<f64>() / findings.len() as f64
}
```

#### 进度偏差公式
```rust
// 统一进度偏差
pub fn calculate_schedule_deviation(
    planned: &[Milestone],
    actual: &[Milestone],
) -> ScheduleDeviation {
    let delays: Vec<MilestoneDelay> = planned.iter()
        .zip(actual.iter())
        .filter_map(|(p, a)| {
            if a.actual_date > p.planned_date {
                Some(MilestoneDelay {
                    milestone: p.name.clone(),
                    delay_days: calculate_days_diff(&p.planned_date, &a.actual_date),
                    cause: a.delay_cause.clone().unwrap_or_default(),
                })
            } else {
                None
            }
        })
        .collect();
    
    let critical_delay = delays.iter()
        .map(|d| d.delay_days)
        .max()
        .unwrap_or(0);
    
    ScheduleDeviation {
        critical_path_delay_days: critical_delay,
        milestone_delays: delays,
        recovery_plan: None,
    }
}
```

## 4. 重构计划

### 4.1 Phase 1: 提取统一骨架 (2h)

| 任务 | 文件 | 说明 |
|------|------|------|
| 创建 trade_core.rs | 新建 | 统一算法骨架 |
| 提取状态机骨架 | trade_core.rs | 泛型状态机 |
| 提取成本计算 | trade_core.rs | 统一公式 |
| 提取风险评估 | trade_core.rs | 统一规则 |

### 4.2 Phase 2: 消除类型冗余 (1h)

| 任务 | 说明 |
|------|------|
| 合并 QuoteSheet | full_cycle 为唯一事实源 |
| 合并 IntentLevel | full_cycle 为唯一事实源 |
| 合并 MilestoneStatus | full_cycle 为唯一事实源 |
| 合并 InspectionReport | production_logistics 为唯一事实源 |
| 合并 Defect | production_logistics 为唯一事实源 |

### 4.3 Phase 3: 重构模块依赖 (1h)

| 任务 | 说明 |
|------|------|
| quote_negotiation 依赖 trade_core | 使用统一骨架 |
| production_logistics 依赖 trade_core | 使用统一骨架 |
| finance_compliance 依赖 trade_core | 使用统一骨架 |
| full_cycle 依赖 trade_core | 使用统一骨架 |

## 5. 预期收益

| 指标 | 重构前 | 重构后 | 提升 |
|------|--------|--------|------|
| 代码行数 | ~3500 | ~2000 | -43% |
| 类型定义 | ~80 | ~40 | -50% |
| 编译时间 | ~60s | ~30s | -50% |
| 维护成本 | 高 | 低 | -60% |

## 6. 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| 破坏现有 API | 高 | 保持向后兼容 |
| 测试覆盖不足 | 中 | 补充单元测试 |
| 性能回归 | 低 | 基准测试验证 |
