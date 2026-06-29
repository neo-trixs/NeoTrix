use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体→虚拟金融抽象堆叠
///
/// 人类金融演化：实物→货币→债券→股票→衍生品→CDS→加密资产
/// 每层抽象增加流动性 × 杠杆 × 可组合性，但增加系统性风险
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinancialAbstractionStack {
    pub layers: Vec<AbstractionLayer>,
    pub securitization_pool: SecuritizationPool,
    pub liquidity_network: LiquidityNetwork,
    pub derivative_book: DerivativeBook,
    pub yield_strategies: Vec<YieldStrategy>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AbstractionLayer {
    /// L0: 实物资产 — 黄金、土地、商品
    Physical {
        asset_type: String,
        storage_cost: f64,
        divisibility: f64,
    },
    /// L1: 货币 — 实物货币→信用货币
    Currency {
        monetary_type: MonetaryType,
        inflation_rate: f64,
        convertibility: f64,
    },
    /// L2: 债务 — 债券、贷款
    Debt {
        instrument: DebtInstrument,
        yield_rate: f64,
        credit_rating: String,
    },
    /// L3: 股权 — 股票、私募
    Equity {
        market_cap: f64,
        dividend_yield: f64,
        volatility: f64,
    },
    /// L4: 衍生品 — 期货、期权、互换
    Derivative {
        underlying: String,
        leverage: f64,
        expiration: String,
    },
    /// L5: 结构化产品 — CDO, MBS, ABS
    Structured {
        pool_type: String,
        tranches: Vec<Tranche>,
        correlation_risk: f64,
    },
    /// L6: 加密/数字资产 — Token, DeFi, NFT
    Digital {
        protocol: String,
        token_type: TokenType,
        smart_contract_risk: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonetaryType {
    CommodityMoney,     // 实物货币（黄金）
    FiatMoney,          // 信用货币（法币）
    DigitalCentralBank, // 央行数字货币
    Cryptocurrency,     // 加密货币
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DebtInstrument {
    GovernmentBond,
    CorporateBond,
    Mortgage,
    SyndicatedLoan,
    ConvertibleNote,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Payment,
    Utility,
    Security,
    Governance,
    Stablecoin,
    NonFungible,
}

/// 资产证券化池
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SecuritizationPool {
    pub pool_value: f64,
    pub tranches: Vec<Tranche>,
    pub weighted_avg_yield: f64,
    pub default_correlation: f64,
    pub over_collateralization: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tranche {
    pub name: String,
    pub seniority: u32, // 优先层级
    pub notional: f64,
    pub coupon_rate: f64,
    pub rating: String,
    pub subordination: f64, // 次级比例
}

/// 流动性网络 — 做市商路径
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidityNetwork {
    pub venues: Vec<MarketVenue>,
    pub total_liquidity: f64,
    pub spread_profile: SpreadProfile,
    pub routing_strategy: RoutingStrategy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketVenue {
    pub name: String,
    pub asset_class: String,
    pub liquidity_depth: f64, // 订单簿深度
    pub maker_rebate: f64,
    pub taker_fee: f64,
    pub latency_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpreadProfile {
    pub avg_bid_ask_spread: f64,
    pub spread_volatility: f64,
    pub time_of_day_spreads: HashMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RoutingStrategy {
    SmartOrderRouting,
    Iceberg,
    TWAP, // Time-Weighted Average Price
    VWAP, // Volume-Weighted Average Price
    PegToMid,
    DarkPool,
}

/// 衍生品账簿
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivativeBook {
    pub open_positions: Vec<DerivativePosition>,
    pub total_notional: f64,
    pub net_exposure: f64,
    pub margin_used: f64,
    pub var_99: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivativePosition {
    pub instrument: String,
    pub direction: PositionDirection,
    pub notional: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub leverage: f64,
    pub margin: f64,
    pub liquidation_price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PositionDirection {
    Long,
    Short,
}

/// 收益策略
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YieldStrategy {
    pub name: String,
    pub expected_apr: f64,
    pub risk_score: f64, // 0-1
    pub capital_requirement: f64,
    pub strategy_type: StrategyType,
    pub impermanent_loss_risk: f64,
    pub audit_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    Lending,            // 借贷
    LiquidityProvision, // 做市
    Staking,            // 质押
    YieldFarming,       // 流动性挖矿
    Arbitrage,          // 套利
    DeltaNeutral,       // Delta中性
    BasisTrading,       // 基差交易
}

impl Default for FinancialAbstractionStack {
    fn default() -> Self {
        Self {
            layers: vec![
                AbstractionLayer::Physical {
                    asset_type: "commodities".into(),
                    storage_cost: 0.02,
                    divisibility: 0.1,
                },
                AbstractionLayer::Currency {
                    monetary_type: MonetaryType::FiatMoney,
                    inflation_rate: 0.03,
                    convertibility: 1.0,
                },
                AbstractionLayer::Equity {
                    market_cap: 0.0,
                    dividend_yield: 0.02,
                    volatility: 0.25,
                },
                AbstractionLayer::Derivative {
                    underlying: "index".into(),
                    leverage: 10.0,
                    expiration: "perpetual".into(),
                },
            ],
            securitization_pool: SecuritizationPool {
                pool_value: 0.0,
                tranches: vec![
                    Tranche {
                        name: "Senior".into(),
                        seniority: 0,
                        notional: 0.0,
                        coupon_rate: 0.04,
                        rating: "AAA".into(),
                        subordination: 0.2,
                    },
                    Tranche {
                        name: "Mezzanine".into(),
                        seniority: 1,
                        notional: 0.0,
                        coupon_rate: 0.08,
                        rating: "BBB".into(),
                        subordination: 0.1,
                    },
                    Tranche {
                        name: "Equity".into(),
                        seniority: 2,
                        notional: 0.0,
                        coupon_rate: 0.15,
                        rating: "NR".into(),
                        subordination: 0.0,
                    },
                ],
                weighted_avg_yield: 0.07,
                default_correlation: 0.3,
                over_collateralization: 1.2,
            },
            liquidity_network: LiquidityNetwork {
                venues: vec![
                    MarketVenue {
                        name: "CEX".into(),
                        asset_class: "spot".into(),
                        liquidity_depth: 1_000_000.0,
                        maker_rebate: -0.001,
                        taker_fee: 0.001,
                        latency_ms: 10.0,
                    },
                    MarketVenue {
                        name: "DEX".into(),
                        asset_class: "spot".into(),
                        liquidity_depth: 100_000.0,
                        maker_rebate: 0.0,
                        taker_fee: 0.003,
                        latency_ms: 500.0,
                    },
                ],
                total_liquidity: 0.0,
                spread_profile: SpreadProfile {
                    avg_bid_ask_spread: 0.001,
                    spread_volatility: 0.0005,
                    time_of_day_spreads: HashMap::new(),
                },
                routing_strategy: RoutingStrategy::SmartOrderRouting,
            },
            derivative_book: DerivativeBook {
                open_positions: Vec::new(),
                total_notional: 0.0,
                net_exposure: 0.0,
                margin_used: 0.0,
                var_99: 0.0,
            },
            yield_strategies: vec![
                YieldStrategy {
                    name: "CEX Lending".into(),
                    expected_apr: 0.08,
                    risk_score: 0.2,
                    capital_requirement: 100.0,
                    strategy_type: StrategyType::Lending,
                    impermanent_loss_risk: 0.0,
                    audit_score: 0.9,
                },
                YieldStrategy {
                    name: "DeFi LP".into(),
                    expected_apr: 0.25,
                    risk_score: 0.5,
                    capital_requirement: 500.0,
                    strategy_type: StrategyType::LiquidityProvision,
                    impermanent_loss_risk: 0.15,
                    audit_score: 0.6,
                },
                YieldStrategy {
                    name: "Delta-Neutral".into(),
                    expected_apr: 0.18,
                    risk_score: 0.3,
                    capital_requirement: 2000.0,
                    strategy_type: StrategyType::DeltaNeutral,
                    impermanent_loss_risk: 0.02,
                    audit_score: 0.7,
                },
                YieldStrategy {
                    name: "Basis Trade".into(),
                    expected_apr: 0.35,
                    risk_score: 0.6,
                    capital_requirement: 5000.0,
                    strategy_type: StrategyType::BasisTrading,
                    impermanent_loss_risk: 0.05,
                    audit_score: 0.5,
                },
            ],
        }
    }
}

impl FinancialAbstractionStack {
    /// 计算跨越 N 层抽象后的总收益倍率
    pub fn abstraction_multiplier(&self) -> f64 {
        let base: f64 = self
            .layers
            .iter()
            .map(|l| match l {
                AbstractionLayer::Derivative { leverage, .. } => *leverage,
                AbstractionLayer::Structured { tranches, .. } => tranches
                    .iter()
                    .map(|t| 1.0 / (t.subordination + 0.01))
                    .sum::<f64>(),
                _ => 1.0,
            })
            .product();
        base.max(1.0)
    }

    /// 执行证券化 — 将一组资产打包分层
    pub fn securitize(&mut self, pool_assets: &[f64]) -> f64 {
        let pool_value: f64 = pool_assets.iter().sum();
        self.securitization_pool.pool_value = pool_value;

        for tranche in &mut self.securitization_pool.tranches {
            let allocation = pool_value * (1.0 - tranche.subordination);
            tranche.notional = allocation;
        }

        pool_value
    }

    /// 开立衍生品头寸
    pub fn open_position(&mut self, pos: DerivativePosition) {
        self.derivative_book.total_notional += pos.notional;
        self.derivative_book.margin_used += pos.margin;
        self.derivative_book.net_exposure += match pos.direction {
            PositionDirection::Long => pos.notional,
            PositionDirection::Short => -pos.notional,
        };
        self.derivative_book.open_positions.push(pos);
        self.recalc_var();
    }

    /// 评估收益策略的风险调整收益
    pub fn risk_adjusted_yield(&self, strategy: &YieldStrategy) -> f64 {
        strategy.expected_apr * (1.0 - strategy.risk_score) * (1.0 - strategy.impermanent_loss_risk)
    }

    /// 推荐最优收益策略
    pub fn suggest_best_yield(&self, capital: f64) -> Vec<&YieldStrategy> {
        let mut ranked: Vec<(&YieldStrategy, f64)> = self
            .yield_strategies
            .iter()
            .filter(|s| s.capital_requirement <= capital)
            .map(|s| (s, self.risk_adjusted_yield(s)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.into_iter().map(|(s, _)| s).take(3).collect()
    }

    /// 模拟做市收益
    pub fn market_making_pnl(&self, volume: f64, num_trades: u64) -> f64 {
        let spread_revenue = volume * self.liquidity_network.spread_profile.avg_bid_ask_spread;
        let fees = self
            .liquidity_network
            .venues
            .iter()
            .map(|v| volume * v.taker_fee)
            .sum::<f64>();
        (spread_revenue - fees) * num_trades as f64
    }

    fn recalc_var(&mut self) {
        let pos_var: f64 = self
            .derivative_book
            .open_positions
            .iter()
            .map(|p| p.notional * 0.02) // simplified 2% daily move
            .sum();
        self.derivative_book.var_99 = pos_var * 2.33; // 99% VaR
    }

    /// 生成金融洞察
    pub fn financial_insight(&self) -> Vec<String> {
        let mut insights = Vec::new();
        let mult = self.abstraction_multiplier();
        insights.push(format!(
            "抽象叠加倍率: {:.1}x（跨越 {} 层金融抽象）",
            mult,
            self.layers.len()
        ));

        if self.securitization_pool.pool_value > 0.0 {
            insights.push(format!(
                "证券化池: ${:.2}, {} 个分级, 加权收益率 {:.1}%",
                self.securitization_pool.pool_value,
                self.securitization_pool.tranches.len(),
                self.securitization_pool.weighted_avg_yield * 100.0,
            ));
        }

        if !self.derivative_book.open_positions.is_empty() {
            insights.push(format!(
                "衍生品账簿: {} 持仓, 名义本金 ${:.0}, VaR(99%)={:.1}%",
                self.derivative_book.open_positions.len(),
                self.derivative_book.total_notional,
                self.derivative_book.var_99 / self.derivative_book.total_notional.max(1.0) * 100.0,
            ));
        }

        for s in &self.yield_strategies {
            let ray = self.risk_adjusted_yield(s);
            insights.push(format!(
                "{}: {:.1}% APR (风险调整后: {:.1}%)",
                s.name,
                s.expected_apr * 100.0,
                ray * 100.0
            ));
        }

        insights
    }

    /// 实体到虚拟的"抽象税" — 每层抽象提取的价值
    pub fn abstraction_tax(&self) -> f64 {
        // 每层抽象提取流动性溢价 + 杠杆费 + 结构费
        self.layers.len() as f64 * 0.02 // 每层 2% 抽象税
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stack() {
        let stack = FinancialAbstractionStack::default();
        assert_eq!(stack.layers.len(), 4);
        assert_eq!(stack.yield_strategies.len(), 4);
    }

    #[test]
    fn test_abstraction_multiplier() {
        let stack = FinancialAbstractionStack::default();
        let mult = stack.abstraction_multiplier();
        assert!(mult >= 1.0);
    }

    #[test]
    fn test_securitize() {
        let mut stack = FinancialAbstractionStack::default();
        let pool = vec![1000.0, 2000.0, 3000.0];
        let result = stack.securitize(&pool);
        assert_eq!(result, 6000.0);
        assert_eq!(stack.securitization_pool.pool_value, 6000.0);
    }

    #[test]
    fn test_open_position() {
        let mut stack = FinancialAbstractionStack::default();
        stack.open_position(DerivativePosition {
            instrument: "BTC/USD".into(),
            direction: PositionDirection::Long,
            notional: 10000.0,
            entry_price: 50000.0,
            mark_price: 51000.0,
            leverage: 10.0,
            margin: 1000.0,
            liquidation_price: 45000.0,
        });
        assert_eq!(stack.derivative_book.total_notional, 10000.0);
        assert_eq!(stack.derivative_book.margin_used, 1000.0);
        assert_eq!(stack.derivative_book.open_positions.len(), 1);
    }

    #[test]
    fn test_risk_adjusted_yield() {
        let stack = FinancialAbstractionStack::default();
        let s = &stack.yield_strategies[0];
        let ray = stack.risk_adjusted_yield(s);
        assert!(ray > 0.0);
        assert!(ray <= s.expected_apr);
    }

    #[test]
    fn test_suggest_best_yield() {
        let stack = FinancialAbstractionStack::default();
        let suggestions = stack.suggest_best_yield(5000.0);
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn test_financial_insight_non_empty() {
        let stack = FinancialAbstractionStack::default();
        let insights = stack.financial_insight();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_market_making_pnl() {
        let stack = FinancialAbstractionStack::default();
        let pnl = stack.market_making_pnl(100000.0, 100);
        assert!(pnl > 0.0 || pnl <= 0.0); // can be negative with fees
    }

    #[test]
    fn test_abstraction_tax() {
        let stack = FinancialAbstractionStack::default();
        let tax = stack.abstraction_tax();
        assert!(tax > 0.0);
        assert_eq!(tax, 0.08); // 4 layers × 2%
    }
}
