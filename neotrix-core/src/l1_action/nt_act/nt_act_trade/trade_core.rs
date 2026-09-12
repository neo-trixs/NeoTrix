//! Trade Core — 统一算法骨架 (Single Source of Truth)
//!
//! 所有 trade 子模块的底层算法公式统一在此定义。
//! 子模块通过 `use super::trade_core::*` 引用，禁止重复定义。
//!
//! 算法公式:
//! - 成本: Total = Σ(Material + Labor + Overhead + ...) → CostCalculator
//! - 定价: Price = Cost × (1 + Margin) → CostCalculator::calculate_price
//! - 让步: Concession = (Current - BottomLine) × Rate × ObjectionScore → NegotiationEngine
//! - 风险: RiskScore = Σ(Severity × Weight) / N → RiskAssessor
//! - 进度: Deviation = Actual - Planned → ProgressTracker
//! - 状态机: P(next) = f(P(current), guard) → StateMachine<S>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 状态机骨架 — 泛型状态机
// ============================================================

pub trait State: Clone + PartialEq + std::fmt::Debug {
    fn next_states(&self) -> Vec<Self>;
    fn is_valid_transition(&self, next: &Self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StateMachine<S: State> {
    pub current: S,
    pub history: Vec<S>,
    pub max_history: usize,
}

impl<S: State> StateMachine<S> {
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
            max_history: 100,
        }
    }
    pub fn advance(&mut self, next: S) -> Result<(), String> {
        if self.current.is_valid_transition(&next) {
            self.history.push(self.current.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
            self.current = next;
            Ok(())
        } else {
            Err(format!(
                "Invalid transition from {:?} to {:?}",
                self.current, next
            ))
        }
    }
    pub fn can_advance_to(&self, next: &S) -> bool {
        self.current.is_valid_transition(next)
    }
    pub fn reset(&mut self, initial: S) {
        self.current = initial;
        self.history.clear();
    }
}

// ============================================================
// 2. 成本计算骨架 — CostCalculator
// ============================================================

pub(crate) trait CostComponent: Send + Sync {
    fn name(&self) -> &str;
    fn calculate(&self, ctx: &CostContext) -> f64;
}

#[derive(Debug, Clone, Default)]
pub struct CostContext {
    pub material_qty: f64,
    pub material_unit_price: f64,
    pub labor_hours: f64,
    pub labor_rate: f64,
    pub overhead_ratio: f64,
    pub packaging_cost: f64,
    pub logistics_cost: f64,
    pub certification_cost: f64,
    pub contingency_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostBreakdown {
    pub material: f64,
    pub labor: f64,
    pub overhead: f64,
    pub packaging: f64,
    pub logistics: f64,
    pub certification: f64,
    pub contingency: f64,
    pub total: f64,
}

impl Default for CostBreakdown {
    fn default() -> Self {
        Self {
            material: 0.0,
            labor: 0.0,
            overhead: 0.0,
            packaging: 0.0,
            logistics: 0.0,
            certification: 0.0,
            contingency: 0.0,
            total: 0.0,
        }
    }
}

pub struct CostCalculator {
    components: Vec<Box<dyn CostComponent>>,
    margin_strategy: MarginStrategy,
}

impl std::fmt::Debug for CostCalculator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostCalculator")
            .field("components", &self.components.len())
            .field("margin_strategy", &self.margin_strategy)
            .finish()
    }
}

impl Clone for CostCalculator {
    fn clone(&self) -> Self {
        Self {
            components: Vec::new(),
            margin_strategy: self.margin_strategy.clone(),
        }
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(MaterialCost),
                Box::new(LaborCost),
                Box::new(OverheadCost),
                Box::new(PackagingCost),
                Box::new(LogisticsCost),
                Box::new(CertificationCost),
                Box::new(ContingencyCost),
            ],
            margin_strategy: MarginStrategy::Fixed(0.20),
        }
    }
}

impl CostCalculator {
    /// 成本公式: Total = Σ(component.calculate(ctx))
    pub fn calculate(&self, ctx: &CostContext) -> CostBreakdown {
        let mut b = CostBreakdown::default();
        for c in &self.components {
            let cost = c.calculate(ctx);
            match c.name() {
                "material" => b.material = cost,
                "labor" => b.labor = cost,
                "overhead" => b.overhead = cost,
                "packaging" => b.packaging = cost,
                "logistics" => b.logistics = cost,
                "certification" => b.certification = cost,
                "contingency" => b.contingency = cost,
                _ => {}
            }
        }
        b.total = b.material
            + b.labor
            + b.overhead
            + b.packaging
            + b.logistics
            + b.certification
            + b.contingency;
        b
    }

    /// 定价公式: Price = Cost × (1 + Margin)
    pub fn calculate_price(&self, cost: f64) -> f64 {
        match self.margin_strategy {
            MarginStrategy::Fixed(m) => cost * (1.0 + m),
            MarginStrategy::Tiered(ref tiers) => {
                for (threshold, m) in tiers {
                    if cost >= *threshold {
                        return cost * (1.0 + m);
                    }
                }
                cost * 1.20
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MarginStrategy {
    Fixed(f64),
    Tiered(Vec<(f64, f64)>),
}

struct MaterialCost;
struct LaborCost;
struct OverheadCost;
struct PackagingCost;
struct LogisticsCost;
struct CertificationCost;
struct ContingencyCost;

impl CostComponent for MaterialCost {
    fn name(&self) -> &str {
        "material"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        c.material_qty * c.material_unit_price
    }
}
impl CostComponent for LaborCost {
    fn name(&self) -> &str {
        "labor"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        c.labor_hours * c.labor_rate
    }
}
impl CostComponent for OverheadCost {
    fn name(&self) -> &str {
        "overhead"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        (c.material_qty * c.material_unit_price + c.labor_hours * c.labor_rate) * c.overhead_ratio
    }
}
impl CostComponent for PackagingCost {
    fn name(&self) -> &str {
        "packaging"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        c.packaging_cost
    }
}
impl CostComponent for LogisticsCost {
    fn name(&self) -> &str {
        "logistics"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        c.logistics_cost
    }
}
impl CostComponent for CertificationCost {
    fn name(&self) -> &str {
        "certification"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        c.certification_cost
    }
}
impl CostComponent for ContingencyCost {
    fn name(&self) -> &str {
        "contingency"
    }
    fn calculate(&self, c: &CostContext) -> f64 {
        let sub = c.material_qty * c.material_unit_price
            + c.labor_hours * c.labor_rate
            + c.packaging_cost
            + c.logistics_cost
            + c.certification_cost;
        sub * c.contingency_ratio
    }
}

// ============================================================
// 3. 谈判引擎骨架 — NegotiationEngine
//    公式: Concession = (Current - BottomLine) × ConcessionRate × ObjectionScore
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationStrategy {
    Collaborative,
    Competitive,
    Compromise,
    Accommodating,
    Avoiding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concession {
    pub round: u32,
    pub item: String,
    pub original: f64,
    pub conceded: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorData {
    pub competitor_name: String,
    pub quoted_price: f64,
    pub quoted_terms: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationEngine {
    pub strategy: NegotiationStrategy,
    pub bottom_line: f64,
    pub current_quote: f64,
    pub round: u32,
    pub concessions_made: Vec<Concession>,
    pub max_rounds: u32,
    pub concession_rate: f64,
    pub competitor_data: Option<CompetitorData>,
}

impl Default for NegotiationEngine {
    fn default() -> Self {
        Self {
            strategy: NegotiationStrategy::Collaborative,
            bottom_line: 0.0,
            current_quote: 0.0,
            round: 0,
            concessions_made: Vec::new(),
            max_rounds: 3,
            concession_rate: 0.3,
            competitor_data: None,
        }
    }
}

impl NegotiationEngine {
    pub fn new(strategy: NegotiationStrategy, bottom_line: f64, initial_quote: f64) -> Self {
        Self {
            strategy,
            bottom_line,
            current_quote: initial_quote,
            ..Default::default()
        }
    }

    /// 让步公式: ConcessionAmount = (CurrentQuote - BottomLine) × AdjustedRate
    /// AdjustedRate = ConcessionRate × ObjectionScore
    pub fn calculate_concession(&mut self, objection_score: f64) -> Option<Concession> {
        if self.current_quote <= self.bottom_line * 1.05 || self.round >= self.max_rounds {
            return None;
        }
        let adjusted_rate = self.concession_rate * objection_score;
        let amount = (self.current_quote - self.bottom_line) * adjusted_rate;
        let new_price = self.current_quote - amount;
        let c = Concession {
            round: self.round,
            item: "unit_price".into(),
            original: self.current_quote,
            conceded: new_price,
            reason: format!(
                "Round {} concession (score={:.2})",
                self.round, objection_score
            ),
        };
        self.concessions_made.push(c.clone());
        self.current_quote = new_price;
        self.round += 1;
        Some(c)
    }

    pub fn is_at_bottom_line(&self) -> bool {
        self.current_quote <= self.bottom_line
    }
    pub fn get_concession_history(&self) -> &[Concession] {
        &self.concessions_made
    }
    pub fn reset(&mut self, initial_quote: f64) {
        self.current_quote = initial_quote;
        self.round = 0;
        self.concessions_made.clear();
    }
}

// ============================================================
// 4. 风险评估骨架 — RiskAssessor
//    公式: RiskScore = Σ(Finding.severity_score()) / N
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl RiskLevel {
    pub fn score(&self) -> f64 {
        match self {
            RiskLevel::Low => 0.25,
            RiskLevel::Medium => 0.50,
            RiskLevel::High => 0.75,
            RiskLevel::Critical => 1.00,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFinding {
    pub category: String,
    pub description: String,
    pub level: RiskLevel,
    pub weight: f64,
}

impl RiskFinding {
    pub fn severity_score(&self) -> f64 {
        self.level.score() * self.weight
    }
}

pub trait RiskRule: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, context: &dyn std::any::Any) -> Option<RiskFinding>;
}

pub struct RiskAssessor {
    rules: Vec<Box<dyn RiskRule>>,
    weights: HashMap<String, f64>,
}

impl std::fmt::Debug for RiskAssessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RiskAssessor")
            .field("rules", &self.rules.len())
            .field("weights", &self.weights)
            .finish()
    }
}

impl Clone for RiskAssessor {
    fn clone(&self) -> Self {
        Self {
            rules: Vec::new(), // Rules are not cloneable, start fresh
            weights: self.weights.clone(),
        }
    }
}

impl Default for RiskAssessor {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            weights: HashMap::new(),
        }
    }
}

impl RiskAssessor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_rule(&mut self, rule: Box<dyn RiskRule>) {
        self.rules.push(rule);
    }
    pub fn set_weight(&mut self, category: &str, weight: f64) {
        self.weights.insert(category.into(), weight);
    }

    pub fn evaluate(&self, ctx: &dyn std::any::Any) -> Vec<RiskFinding> {
        self.rules.iter().filter_map(|r| r.evaluate(ctx)).collect()
    }

    /// 风险评分公式
    pub fn calculate_score(&self, findings: &[RiskFinding]) -> f64 {
        if findings.is_empty() {
            return 0.0;
        }
        let total: f64 = findings.iter().map(|f| f.severity_score()).sum();
        total / findings.len() as f64
    }

    pub fn assess_risk(&self, ctx: &dyn std::any::Any) -> RiskAssessment {
        let findings = self.evaluate(ctx);
        let score = self.calculate_score(&findings);
        let level = self.score_to_level(score);
        RiskAssessment {
            score,
            level,
            findings,
            recommendations: self.gen_recs(score),
        }
    }

    fn score_to_level(&self, s: f64) -> RiskLevel {
        if s >= 0.75 {
            RiskLevel::Critical
        } else if s >= 0.50 {
            RiskLevel::High
        } else if s >= 0.25 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
    fn gen_recs(&self, s: f64) -> Vec<String> {
        if s >= 0.75 {
            vec!["Reject or require safeguards".into()]
        } else if s >= 0.50 {
            vec!["Require additional documentation".into()]
        } else if s >= 0.25 {
            vec!["Proceed with monitoring".into()]
        } else {
            vec!["Proceed with confidence".into()]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: f64,
    pub level: RiskLevel,
    pub findings: Vec<RiskFinding>,
    pub recommendations: Vec<String>,
}

// ============================================================
// 5. 进度跟踪骨架 — ProgressTracker
//    公式: Deviation = max(actual_date - planned_date) on critical_path
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Delayed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub planned_date: String,
    pub actual_date: Option<String>,
    pub status: MilestoneStatus,
    pub dependencies: Vec<String>,
}

impl Milestone {
    pub fn new(name: &str, planned_date: &str, dependencies: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            planned_date: planned_date.into(),
            actual_date: None,
            status: MilestoneStatus::Pending,
            dependencies: dependencies.into_iter().map(String::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneDelay {
    pub milestone: String,
    pub delay_days: i32,
    pub cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDeviation {
    pub critical_path_delay_days: i32,
    pub milestone_delays: Vec<MilestoneDelay>,
    pub recovery_plan: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProgressTracker {
    milestones: Vec<Milestone>,
    critical_path: Vec<String>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self {
            milestones: Vec::new(),
            critical_path: Vec::new(),
        }
    }
}

impl ProgressTracker {
    pub fn new(milestones: Vec<Milestone>, critical_path: Vec<String>) -> Self {
        Self {
            milestones,
            critical_path,
        }
    }

    /// 进度偏差公式
    pub fn calculate_deviation(
        &self,
        actual_dates: &[(String, Option<String>)],
    ) -> ScheduleDeviation {
        let delays: Vec<MilestoneDelay> = self
            .milestones
            .iter()
            .filter_map(|m| {
                actual_dates
                    .iter()
                    .find(|(n, _)| n == &m.name)
                    .and_then(|(_, a)| a.as_ref().map(|a| (m, a)))
                    .and_then(|(m, a)| {
                        let d = days_diff(&m.planned_date, a);
                        if d > 0 {
                            Some(MilestoneDelay {
                                milestone: m.name.clone(),
                                delay_days: d,
                                cause: String::new(),
                            })
                        } else {
                            None
                        }
                    })
            })
            .collect();
        let crit = self
            .critical_path
            .iter()
            .filter_map(|n| {
                delays
                    .iter()
                    .find(|d| d.milestone == *n)
                    .map(|d| d.delay_days)
            })
            .max()
            .unwrap_or(0);
        ScheduleDeviation {
            critical_path_delay_days: crit,
            milestone_delays: delays,
            recovery_plan: None,
        }
    }

    pub fn get_overall_progress(&self) -> f64 {
        if self.milestones.is_empty() {
            return 0.0;
        }
        self.milestones
            .iter()
            .filter(|m| m.status == MilestoneStatus::Completed)
            .count() as f64
            / self.milestones.len() as f64
    }
}

fn days_diff(planned: &str, actual: &str) -> i32 {
    // simplified: assume YYYY-MM-DD format
    let parse = |s: &str| -> i64 {
        let parts: Vec<i64> = s.split('-').filter_map(|p| p.parse().ok()).collect();
        if parts.len() == 3 {
            parts[0] * 365 + parts[1] * 30 + parts[2]
        } else {
            0
        }
    };
    (parse(actual) - parse(planned)) as i32
}

// ============================================================
// 6. AQL 抽样公式 — ISO 2859-1
// ============================================================

/// AQL 抽样样本量公式
/// 公式: SampleSize = f(LotSize, AQLLevel)
/// 基于 ISO 2859-1 一般检验水平 II
pub(crate) fn aql_sample_size(lot_size: u64, aql_level: f64) -> u32 {
    let base = match lot_size {
        0..=8 => 2,
        9..=15 => 3,
        16..=25 => 5,
        26..=50 => 8,
        51..=90 => 13,
        91..=150 => 20,
        151..=280 => 32,
        281..=500 => 50,
        501..=1200 => 80,
        1201..=3200 => 125,
        3201..=10000 => 200,
        10001..=35000 => 315,
        _ => 500,
    };
    // AQL 系数: 严格 AQL 需要更大样本
    let factor = if aql_level <= 0.65 {
        0.7
    } else if aql_level <= 1.0 {
        0.85
    } else if aql_level <= 2.5 {
        1.0
    } else {
        1.15
    };
    ((base as f64 * factor) as u32).max(2)
}

/// AQL 判定: Ac (接收数) 和 Re (拒收数)
pub(crate) fn aql_accept_reject(sample_size: u32, aql_level: f64) -> (u32, u32) {
    let ac = ((sample_size as f64 * aql_level / 100.0) as u32).max(0);
    (ac, ac + 1)
}

// ============================================================
// 7. 汇率换算公式
// ============================================================

/// 汇率换算
/// 公式: TargetAmount = Amount × TargetRate / SourceRate
pub(crate) fn convert_currency(amount: f64, source_rate: f64, target_rate: f64) -> f64 {
    if source_rate == 0.0 {
        return 0.0;
    }
    amount * target_rate / source_rate
}

/// 结算金额计算
/// 公式: Net = Received × (1 - FeeRatio), Settled = Net × FxRate
pub(crate) fn settle_amount(received: f64, fx_rate: f64, bank_fee_ratio: f64) -> (f64, f64, f64) {
    let bank_fees = received * bank_fee_ratio;
    let net = received - bank_fees;
    let settled = net * fx_rate;
    (bank_fees, net, settled)
}

// ============================================================
// 8. 退税计算公式
// ============================================================

/// 退税金额
/// 公式: RefundAmount = ExportValue × RefundRate
pub(crate) fn tax_refund_amount(export_value: f64, refund_rate: f64) -> f64 {
    export_value * refund_rate
}

// ============================================================
// 9. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator() {
        let calc = CostCalculator::default();
        let ctx = CostContext {
            material_qty: 100.0,
            material_unit_price: 10.0,
            labor_hours: 20.0,
            labor_rate: 25.0,
            overhead_ratio: 0.15,
            packaging_cost: 5.0,
            logistics_cost: 50.0,
            certification_cost: 20.0,
            contingency_ratio: 0.05,
        };
        let b = calc.calculate(&ctx);
        assert!(b.total > 0.0);
        assert_eq!(b.material, 1000.0);
        assert_eq!(b.labor, 500.0);
        assert!((calc.calculate_price(b.total) - b.total * 1.20).abs() < 0.01);
    }

    #[test]
    fn test_negotiation_engine() {
        let mut e = NegotiationEngine::new(NegotiationStrategy::Collaborative, 100.0, 120.0);
        let c = e.calculate_concession(0.5);
        assert!(c.is_some());
        assert!(e.current_quote < 120.0);
        assert!(e.round == 1);
    }

    #[test]
    fn test_risk_assessor() {
        let mut a = RiskAssessor::new();
        a.set_weight("financial", 1.5);
        let f = vec![RiskFinding {
            category: "financial".into(),
            description: "High risk".into(),
            level: RiskLevel::High,
            weight: 1.0,
        }];
        assert!(a.calculate_score(&f) > 0.0);
    }

    #[test]
    fn test_progress_tracker() {
        let ms = vec![Milestone {
            name: "Design".into(),
            planned_date: "2024-01-01".into(),
            actual_date: Some("2024-01-05".into()),
            status: MilestoneStatus::Completed,
            dependencies: vec![],
        }];
        let t = ProgressTracker::new(ms, vec!["Design".into()]);
        let d = t.calculate_deviation(&[("Design".into(), Some("2024-01-05".into()))]);
        assert!(d.critical_path_delay_days > 0);
    }

    #[test]
    fn test_aql_sample_size() {
        assert_eq!(aql_sample_size(50, 2.5), 8); // 26-50 → base 8
        assert_eq!(aql_sample_size(200, 2.5), 32); // 151-280 → base 32
        assert_eq!(aql_sample_size(1000, 2.5), 80); // 501-1200 → base 80
        assert!(aql_sample_size(5, 0.65) >= 2); // min 2
    }

    #[test]
    fn test_aql_accept_reject() {
        let (ac, re) = aql_accept_reject(80, 2.5);
        assert!(ac < re);
        assert_eq!(re, ac + 1);
    }

    #[test]
    fn test_convert_currency() {
        // 100 USD → CNY (rate 7.2)
        let cny = convert_currency(100.0, 1.0, 7.2);
        assert!((cny - 720.0).abs() < 0.01);
        // zero source rate
        assert_eq!(convert_currency(100.0, 0.0, 7.2), 0.0);
    }

    #[test]
    fn test_settle_amount() {
        let (fees, net, settled) = settle_amount(35000.0, 7.2, 0.005);
        assert!((fees - 175.0).abs() < 0.01);
        assert!((net - 34825.0).abs() < 0.01);
        assert!((settled - 250740.0).abs() < 1.0);
    }

    #[test]
    fn test_tax_refund_amount() {
        assert!((tax_refund_amount(50000.0, 0.13) - 6500.0).abs() < 0.01);
        assert_eq!(tax_refund_amount(0.0, 0.13), 0.0);
    }

    // --- StateMachine tests ---

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum TestPhase {
        A,
        B,
        C,
    }

    impl State for TestPhase {
        fn next_states(&self) -> Vec<Self> {
            match self {
                Self::A => vec![Self::B],
                Self::B => vec![Self::C],
                Self::C => vec![],
            }
        }
        fn is_valid_transition(&self, next: &Self) -> bool {
            self.next_states().contains(next)
        }
    }

    #[test]
    fn test_state_machine_new() {
        let sm = StateMachine::new(TestPhase::A);
        assert_eq!(sm.current, TestPhase::A);
        assert!(sm.history.is_empty());
    }

    #[test]
    fn test_state_machine_advance() {
        let mut sm = StateMachine::new(TestPhase::A);
        sm.advance(TestPhase::B).unwrap();
        assert_eq!(sm.current, TestPhase::B);
        assert_eq!(sm.history, vec![TestPhase::A]);
        sm.advance(TestPhase::C).unwrap();
        assert_eq!(sm.current, TestPhase::C);
        assert_eq!(sm.history, vec![TestPhase::A, TestPhase::B]);
    }

    #[test]
    fn test_state_machine_invalid_transition() {
        let mut sm = StateMachine::new(TestPhase::A);
        assert!(sm.advance(TestPhase::C).is_err());
        assert_eq!(sm.current, TestPhase::A); // unchanged
    }

    #[test]
    fn test_state_machine_terminal() {
        let mut sm = StateMachine::new(TestPhase::C);
        assert!(sm.advance(TestPhase::A).is_err());
    }

    #[test]
    fn test_cost_breakdown_serialize() {
        let b = CostBreakdown {
            material: 100.0,
            labor: 50.0,
            overhead: 15.0,
            packaging: 5.0,
            logistics: 20.0,
            certification: 10.0,
            contingency: 8.0,
            total: 208.0,
        };
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.contains("\"material\":100"));
        let b2: CostBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(b2.total, 208.0);
    }

    #[test]
    fn test_milestone_serialize() {
        let m = Milestone::new("Design", "2024-01-01", vec![]);
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("Design"));
        let m2: Milestone = serde_json::from_str(&json).unwrap();
        assert_eq!(m2.name, "Design");
    }
}
