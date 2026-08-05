use serde::{Deserialize, Serialize};

/// 金字塔财富积累机制 — 人类文明顶层获取资源的结构化模型
///
/// 基于历史观察：从部落酋长→封建领主→工业资本→金融资本→科技资本
/// 每个阶段的共同抽象：杠杆 × 不对称性 × 复利
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WealthModel {
    pub mechanisms: Vec<WealthMechanism>,
    pub active_strategies: Vec<ActiveStrategy>,
    pub capital_layers: CapitalStack,
    pub leverage_profile: LeverageProfile,
    pub compounding_tracker: CompoundingTracker,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WealthMechanism {
    /// 资本杠杆 — 用别人的钱(OPM)生钱
    CapitalLeverage {
        description: String,
        roi_multiplier: f64,
        risk_factor: f64,
    },
    /// 网络效应护城河 — 平台经济
    NetworkMoat {
        network_type: NetworkType,
        lock_in_effect: f64,
        switching_cost: f64,
    },
    /// 信息不对称 — 利用信息差
    InformationAsymmetry {
        advantage_type: InfoAdvantage,
        signal_to_noise: f64,
        decay_rate: f64,
    },
    /// 监管捕获 — 规则制定权
    RegulatoryCapture {
        barrier_type: RegulatoryBarrier,
        compliance_cost: f64,
        grandfathered: bool,
    },
    /// 资产复利 — 资产增值 > 劳动收入
    AssetCompounding {
        asset_class: AssetClass,
        appreciation_rate: f64,
        income_yield: f64,
        tax_efficiency: f64,
    },
    /// 注意力垄断 — 注意力经济
    AttentionMonopoly {
        platform: String,
        user_base: u64,
        attention_revenue_per_user: f64,
    },
    /// 算法套利 — 算法化金融
    AlgorithmicArbitrage {
        strategy: ArbitrageStrategy,
        latency_advantage_ms: f64,
        capacity_per_day: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NetworkType {
    Direct,          // 直接网络效应（电话网络）
    Indirect,        // 间接网络效应（平台双边市场）
    Data,            // 数据网络效应（更多用户→更好数据→更好产品）
    Social,          // 社交网络效应
    Protocol,        // 协议网络效应（区块链/标准）
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InfoAdvantage {
    InsiderKnowledge,
    ProprietaryData,
    SignalProcessing,
    TemporalArbitrage,   // 先知道
    CrossDomainInsight,  // 跨领域关联
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RegulatoryBarrier {
    License,           // 牌照壁垒
    Patent,            // 专利壁垒
    Standard,          // 标准制定权
    ComplianceMoat,    // 合规成本壁垒
    GrandfatherClause, // 既有权益保护
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AssetClass {
    RealEstate,
    Equities,
    FixedIncome,
    Commodities,
    PrivateEquity,
    VentureCapital,
    IntellectualProperty,
    DigitalAssets,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ArbitrageStrategy {
    Statistical,       // 统计套利
    Triangular,        // 三角套利
    CrossExchange,     // 跨交易所
    Temporal,          // 时间套利
    Merger,            // 并购套利
    Volatility,        // 波动率套利
    YieldFarming,      // DeFi 收益 farming
}

/// 活跃策略 — 当前正在执行的套利/积累
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveStrategy {
    pub mechanism: WealthMechanism,
    pub capital_deployed: f64,
    pub current_roi: f64,
    pub running_since: chrono::NaiveDate,
    pub last_assessment: chrono::NaiveDate,
    pub enabled: bool,
}

/// 资本堆叠 — 模拟多重杠杆叠加
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapitalStack {
    /// 基础资本（自有）
    pub equity_capital: f64,
    /// 杠杆资本（借贷/OPM）
    pub leveraged_capital: f64,
    /// 杠杆倍数
    pub leverage_multiple: f64,
    /// 各层资本成本
    pub layer_costs: Vec<CapitalLayer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapitalLayer {
    pub name: String,
    pub amount: f64,
    pub cost_rate: f64,       // 年化成本率
    pub seniority: u32,       // 优先层级（0=最优先）
    pub source: CapitalSource,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CapitalSource {
    RetainedEarnings,
    BankDebt,
    BondIssuance,
    EquityIssuance,
    VentureCapital,
    PrivateEquity,
    TokenSale,
    MarginLoan,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeverageProfile {
    pub total_leverage_ratio: f64,  // 总杠杆率
    pub risk_adjusted_return: f64,  // 风险调整后回报
    pub max_drawdown: f64,          // 最大回撤
    pub sharpe_ratio: f64,          // 夏普比率
    pub var_95: f64,                // 95% VaR
}

/// 复利追踪器 — 资产增值 vs 劳动收入曲线
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompoundingTracker {
    pub asset_value: f64,
    pub labor_income: f64,
    pub capital_income: f64,
    pub compounding_rate: f64,       // 年化复利率
    pub capital_to_labor_ratio: f64, // 资本收入/劳动收入比
    pub doublings: u32,              // 资产翻倍次数
    pub time_horizon_days: u64,
    pub daily_records: Vec<DailyWealthRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyWealthRecord {
    pub date: String,
    pub asset_value: f64,
    pub labor_income: f64,
    pub capital_income: f64,
    pub leverage_contribution: f64,
}

impl Default for WealthModel {
    fn default() -> Self {
        Self {
            mechanisms: vec![
                WealthMechanism::CapitalLeverage {
                    description: "使用杠杆放大资本回报".into(),
                    roi_multiplier: 3.0,
                    risk_factor: 0.4,
                },
                WealthMechanism::NetworkMoat {
                    network_type: NetworkType::Data,
                    lock_in_effect: 0.7,
                    switching_cost: 0.6,
                },
                WealthMechanism::InformationAsymmetry {
                    advantage_type: InfoAdvantage::SignalProcessing,
                    signal_to_noise: 0.8,
                    decay_rate: 0.05,
                },
                WealthMechanism::AssetCompounding {
                    asset_class: AssetClass::DigitalAssets,
                    appreciation_rate: 0.25,
                    income_yield: 0.05,
                    tax_efficiency: 0.7,
                },
            ],
            active_strategies: Vec::new(),
            capital_layers: CapitalStack {
                equity_capital: 1000.0,
                leveraged_capital: 0.0,
                leverage_multiple: 1.0,
                layer_costs: vec![CapitalLayer {
                    name: "Retained".into(),
                    amount: 1000.0,
                    cost_rate: 0.0,
                    seniority: 0,
                    source: CapitalSource::RetainedEarnings,
                }],
            },
            leverage_profile: LeverageProfile {
                total_leverage_ratio: 1.0,
                risk_adjusted_return: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
                var_95: 0.0,
            },
            compounding_tracker: CompoundingTracker {
                asset_value: 1000.0,
                labor_income: 0.0,
                capital_income: 0.0,
                compounding_rate: 0.0,
                capital_to_labor_ratio: 0.0,
                doublings: 0,
                time_horizon_days: 0,
                daily_records: Vec::new(),
            },
        }
    }
}

impl WealthModel {
    /// 分析给定财富积累策略的预期 ROI
    pub fn analyze_strategy(&self, mechanism: &WealthMechanism) -> f64 {
        match mechanism {
            WealthMechanism::CapitalLeverage { roi_multiplier, risk_factor, .. } => {
                roi_multiplier * (1.0 - risk_factor)
            }
            WealthMechanism::NetworkMoat { lock_in_effect, switching_cost, .. } => {
                (lock_in_effect + switching_cost) / 2.0 * 0.3  // 护城河年化收益
            }
            WealthMechanism::InformationAsymmetry { signal_to_noise, decay_rate, .. } => {
                signal_to_noise * (1.0 - decay_rate) * 0.5
            }
            WealthMechanism::RegulatoryCapture { compliance_cost, .. } => {
                (1.0 / (1.0 + compliance_cost)) * 0.4
            }
            WealthMechanism::AssetCompounding { appreciation_rate, income_yield, .. } => {
                appreciation_rate + income_yield
            }
            WealthMechanism::AttentionMonopoly { attention_revenue_per_user, user_base, .. } => {
                *user_base as f64 * attention_revenue_per_user * 0.001
            }
            WealthMechanism::AlgorithmicArbitrage { capacity_per_day, .. } => {
                *capacity_per_day * 365.0
            }
        }
    }

    /// 应用杠杆 — 模拟借贷放大资本，返回总资本
    pub fn apply_leverage(&mut self, additional_capital: f64, cost_rate: f64, source: CapitalSource) -> f64 {
        let layer = CapitalLayer {
            name: format!("{:?}-{}", source, self.capital_layers.layer_costs.len()),
            amount: additional_capital,
            cost_rate,
            seniority: self.capital_layers.layer_costs.len() as u32,
            source,
        };
        self.capital_layers.leveraged_capital += additional_capital;
        self.capital_layers.layer_costs.push(layer);
        self.capital_layers.total_capital()
    }

    /// 计算一天的复利
    pub fn compound_day(&mut self, labor_income: f64, capital_return_rate: f64) {
        let assets = self.capital_layers.total_capital();
        let daily_capital_return = assets * capital_return_rate / 365.0;
        let total_cost: f64 = self.capital_layers.layer_costs.iter()
            .map(|l| l.amount * l.cost_rate / 365.0)
            .sum();
        let net_capital_income = daily_capital_return - total_cost;

        let record = DailyWealthRecord {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            asset_value: self.compounding_tracker.asset_value,
            labor_income,
            capital_income: net_capital_income,
            leverage_contribution: self.capital_layers.leveraged_capital * capital_return_rate / 365.0,
        };

        self.compounding_tracker.asset_value += net_capital_income + labor_income;
        self.compounding_tracker.labor_income += labor_income;
        self.compounding_tracker.capital_income += net_capital_income;
        self.compounding_tracker.time_horizon_days += 1;
        self.compounding_tracker.daily_records.push(record);

        let total_income = self.compounding_tracker.capital_income + self.compounding_tracker.labor_income;
        self.compounding_tracker.capital_to_labor_ratio = if self.compounding_tracker.labor_income > 0.0 {
            self.compounding_tracker.capital_income / self.compounding_tracker.labor_income
        } else {
            f64::MAX
        };

        if total_income > 0.0 {
            self.compounding_tracker.compounding_rate =
                (self.compounding_tracker.capital_income / self.compounding_tracker.labor_income.max(1.0))
                .min(100.0);
        }
    }

    /// 检测是否进入"资本自动增长"模式（资本收入>劳动收入）
    pub fn is_capital_self_sustaining(&self) -> bool {
        self.compounding_tracker.capital_income > self.compounding_tracker.labor_income
    }

    /// 生成人类文明层级的财富洞察
    pub fn wealth_insight(&self) -> Vec<String> {
        let mut insights = Vec::new();

        if self.is_capital_self_sustaining() {
            insights.push(format!(
                "资本已自持: 资本收入 ${:.2} > 劳动收入 ${:.2}，进入复利自动增长模式",
                self.compounding_tracker.capital_income,
                self.compounding_tracker.labor_income,
            ));
        }

        if self.capital_layers.leverage_multiple > 2.0 {
            insights.push(format!(
                "杠杆放大 {:.1}x，预期 ROI 提升但风险上升 (VaR95={:.1}%)",
                self.capital_layers.leverage_multiple,
                self.leverage_profile.var_95 * 100.0,
            ));
        }

        for m in &self.mechanisms {
            let roi = self.analyze_strategy(m);
            insights.push(format!("{:?} 预期年化: {:.1}%", m, roi * 100.0));
        }

        insights
    }

    /// 建议最优财富积累策略组合
    pub fn suggest_optimal_strategy(&self) -> Vec<WealthMechanism> {
        let mut ranked: Vec<(f64, &WealthMechanism)> = self.mechanisms.iter()
            .map(|m| (self.analyze_strategy(m), m))
            .collect();
        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        ranked.into_iter()
            .filter(|(score, _)| *score > 0.1)
            .map(|(_, m)| m.clone())
            .take(3)
            .collect()
    }
}

impl CapitalStack {
    pub fn total_capital(&self) -> f64 {
        self.equity_capital + self.leveraged_capital
    }

    pub fn effective_leverage(&self) -> f64 {
        if self.equity_capital > 0.0 {
            self.total_capital() / self.equity_capital
        } else {
            1.0
        }
    }

    pub fn weighted_cost_rate(&self) -> f64 {
        let total = self.total_capital();
        if total == 0.0 { return 0.0; }
        self.layer_costs.iter()
            .map(|l| l.amount * l.cost_rate)
            .sum::<f64>() / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        let m = WealthModel::default();
        assert_eq!(m.capital_layers.total_capital(), 1000.0);
    }

    #[test]
    fn test_apply_leverage() {
        let mut m = WealthModel::default();
        m.apply_leverage(5000.0, 0.05, CapitalSource::BankDebt);
        assert_eq!(m.capital_layers.leveraged_capital, 5000.0);
        assert_eq!(m.capital_layers.layer_costs.len(), 2);
    }

    #[test]
    fn test_compound_day_labor_dominant() {
        let mut m = WealthModel::default();
        m.compound_day(100.0, 0.0);
        assert!(!m.is_capital_self_sustaining());
    }

    #[test]
    fn test_compound_day_capital_dominant() {
        let mut m = WealthModel::default();
        m.apply_leverage(100000.0, 0.03, CapitalSource::BankDebt);
        m.compound_day(0.0, 0.2);
        assert!(m.is_capital_self_sustaining());
    }

    #[test]
    fn test_analyze_strategies() {
        let m = WealthModel::default();
        for mech in &m.mechanisms {
            let roi = m.analyze_strategy(mech);
            assert!(roi >= 0.0, "{:?} has negative ROI", mech);
        }
    }

    #[test]
    fn test_suggest_optimal_strategy_returns_top_3() {
        let m = WealthModel::default();
        let suggestions = m.suggest_optimal_strategy();
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn test_wealth_insight_format() {
        let m = WealthModel::default();
        let insights = m.wealth_insight();
        assert!(!insights.is_empty());
        for i in &insights {
            assert!(!i.is_empty());
        }
    }

    #[test]
    fn test_capital_stack_weighted_cost() {
        let mut cs = CapitalStack {
            equity_capital: 1000.0,
            leveraged_capital: 0.0,
            leverage_multiple: 1.0,
            layer_costs: vec![],
        };
        cs.layer_costs.push(CapitalLayer {
            name: "equity".into(), amount: 1000.0, cost_rate: 0.0,
            seniority: 0, source: CapitalSource::RetainedEarnings,
        });
        assert_eq!(cs.weighted_cost_rate(), 0.0);
        cs.layer_costs.push(CapitalLayer {
            name: "debt".into(), amount: 4000.0, cost_rate: 0.05,
            seniority: 1, source: CapitalSource::BankDebt,
        });
        let wacc = cs.weighted_cost_rate();
        assert!((wacc - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_doublings_tracking() {
        let mut m = WealthModel::default();
        for _ in 0..365 {
            m.compound_day(10.0, 0.04);
        }
        assert!(m.compounding_tracker.time_horizon_days >= 365);
    }
}
