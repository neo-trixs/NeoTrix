use super::chain::ChainType;

#[derive(Clone, Debug)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub chain: ChainType,
    pub asset: String,
    pub apy: f64,
    pub tvl_usd: f64,
    pub risk_score: f64,
    pub yield_type: YieldType,
    pub min_deposit_usd: f64,
    pub lock_days: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum YieldType {
    Lending,
    Staking,
    Lp,
    Vault,
    LiquidStaking,
}

impl YieldType {
    pub fn name(&self) -> &str {
        match self {
            YieldType::Lending => "Lending",
            YieldType::Staking => "Staking",
            YieldType::Lp => "LP",
            YieldType::Vault => "Vault",
            YieldType::LiquidStaking => "Liquid Staking",
        }
    }

    pub fn base_risk(&self) -> f64 {
        match self {
            YieldType::Lending => 0.15,
            YieldType::Staking => 0.25,
            YieldType::LiquidStaking => 0.20,
            YieldType::Vault => 0.40,
            YieldType::Lp => 0.50,
        }
    }
}

pub struct YieldScanner {
    opportunities: Vec<YieldOpportunity>,
}

impl YieldScanner {
    pub fn new() -> Self {
        Self {
            opportunities: vec![],
        }
    }

    pub fn register_defaults(&mut self) {
        let defaults = vec![
            YieldOpportunity {
                protocol: "AAVE V3".into(),
                chain: ChainType::Ethereum,
                asset: "USDC".into(),
                apy: 5.2,
                tvl_usd: 2_500_000_000.0,
                risk_score: 0.15,
                yield_type: YieldType::Lending,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "AAVE V3".into(),
                chain: ChainType::Ethereum,
                asset: "WETH".into(),
                apy: 1.8,
                tvl_usd: 3_000_000_000.0,
                risk_score: 0.15,
                yield_type: YieldType::Lending,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Compound III".into(),
                chain: ChainType::Ethereum,
                asset: "USDC".into(),
                apy: 4.8,
                tvl_usd: 1_800_000_000.0,
                risk_score: 0.15,
                yield_type: YieldType::Lending,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Lido".into(),
                chain: ChainType::Ethereum,
                asset: "stETH".into(),
                apy: 3.5,
                tvl_usd: 20_000_000_000.0,
                risk_score: 0.20,
                yield_type: YieldType::LiquidStaking,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Rocket Pool".into(),
                chain: ChainType::Ethereum,
                asset: "rETH".into(),
                apy: 3.4,
                tvl_usd: 3_000_000_000.0,
                risk_score: 0.20,
                yield_type: YieldType::LiquidStaking,
                min_deposit_usd: 100.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Uniswap V3".into(),
                chain: ChainType::Ethereum,
                asset: "USDC/ETH 0.05%".into(),
                apy: 15.0,
                tvl_usd: 500_000_000.0,
                risk_score: 0.50,
                yield_type: YieldType::Lp,
                min_deposit_usd: 1000.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Uniswap V3".into(),
                chain: ChainType::Arbitrum,
                asset: "USDC/ETH 0.05%".into(),
                apy: 18.0,
                tvl_usd: 200_000_000.0,
                risk_score: 0.50,
                yield_type: YieldType::Lp,
                min_deposit_usd: 1000.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "PancakeSwap".into(),
                chain: ChainType::Bsc,
                asset: "CAKE Staking".into(),
                apy: 25.0,
                tvl_usd: 1_500_000_000.0,
                risk_score: 0.35,
                yield_type: YieldType::Staking,
                min_deposit_usd: 10.0,
                lock_days: 7,
            },
            YieldOpportunity {
                protocol: "Aave V3".into(),
                chain: ChainType::Polygon,
                asset: "USDC".into(),
                apy: 6.5,
                tvl_usd: 500_000_000.0,
                risk_score: 0.18,
                yield_type: YieldType::Lending,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
            YieldOpportunity {
                protocol: "Aave V3".into(),
                chain: ChainType::Arbitrum,
                asset: "USDC".into(),
                apy: 7.0,
                tvl_usd: 600_000_000.0,
                risk_score: 0.18,
                yield_type: YieldType::Lending,
                min_deposit_usd: 0.0,
                lock_days: 0,
            },
        ];
        self.opportunities.extend(defaults);
    }

    pub fn scan(&self) -> &[YieldOpportunity] {
        &self.opportunities
    }

    pub fn by_chain(&self, chain: &ChainType) -> Vec<&YieldOpportunity> {
        self.opportunities
            .iter()
            .filter(|o| o.chain == *chain)
            .collect()
    }

    pub fn by_type(&self, yield_type: &YieldType) -> Vec<&YieldOpportunity> {
        self.opportunities
            .iter()
            .filter(|o| o.yield_type == *yield_type)
            .collect()
    }

    pub fn best_apy(&self, min_tvl: f64, max_risk: f64) -> Vec<&YieldOpportunity> {
        let mut filtered: Vec<&YieldOpportunity> = self
            .opportunities
            .iter()
            .filter(|o| o.tvl_usd >= min_tvl && o.risk_score <= max_risk)
            .collect();
        filtered.sort_by(|a, b| {
            b.apy
                .partial_cmp(&a.apy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        filtered
    }

    pub fn risk_adjusted_rank(&self) -> Vec<(&YieldOpportunity, f64)> {
        let mut ranked: Vec<(&YieldOpportunity, f64)> = self
            .opportunities
            .iter()
            .map(|o| {
                let score = o.apy / (o.risk_score + o.yield_type.base_risk());
                (o, score)
            })
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    pub fn by_chain_and_type(
        &self,
        chain: &ChainType,
        yield_type: &YieldType,
    ) -> Vec<&YieldOpportunity> {
        self.opportunities
            .iter()
            .filter(|o| o.chain == *chain && o.yield_type == *yield_type)
            .collect()
    }

    pub fn estimate_monthly_earnings(amount_usd: f64, apy: f64) -> f64 {
        amount_usd * apy / 100.0 / 12.0
    }

    pub fn estimate_gas_cost_proportion(gas_cost_usd: f64, deposit_usd: f64) -> f64 {
        if deposit_usd <= 0.0 {
            return 1.0;
        }
        (gas_cost_usd / deposit_usd).min(1.0)
    }
}

impl Default for YieldScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_type_names() {
        assert_eq!(YieldType::Lending.name(), "Lending");
        assert_eq!(YieldType::Lp.name(), "LP");
        assert_eq!(YieldType::LiquidStaking.name(), "Liquid Staking");
    }

    #[test]
    fn test_yield_type_risk() {
        assert!((YieldType::Lending.base_risk() - 0.15).abs() < 0.01);
        assert!((YieldType::Lp.base_risk() - 0.50).abs() < 0.01);
    }

    #[test]
    fn test_scanner_defaults() {
        let scanner = YieldScanner::new();
        assert!(scanner.scan().is_empty());

        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        assert_eq!(scanner.scan().len(), 10);
    }

    #[test]
    fn test_by_chain() {
        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        let eth = scanner.by_chain(&ChainType::Ethereum);
        assert!(eth.len() >= 5);
        let arb = scanner.by_chain(&ChainType::Arbitrum);
        assert_eq!(arb.len(), 2);
    }

    #[test]
    fn test_by_type() {
        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        let lending = scanner.by_type(&YieldType::Lending);
        assert_eq!(lending.len(), 5);
    }

    #[test]
    fn test_best_apy() {
        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        let best = scanner.best_apy(0.0, 1.0);
        assert!(!best.is_empty());
        assert!(best[0].apy >= best.last().unwrap().apy);
    }

    #[test]
    fn test_risk_adjusted_rank() {
        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        let ranked = scanner.risk_adjusted_rank();
        assert_eq!(ranked.len(), 10);
        assert!(ranked[0].1 >= ranked.last().unwrap().1);
    }

    #[test]
    fn test_by_chain_and_type() {
        let mut scanner = YieldScanner::new();
        scanner.register_defaults();
        let result = scanner.by_chain_and_type(&ChainType::Ethereum, &YieldType::Lending);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_estimate_monthly() {
        let monthly = YieldScanner::estimate_monthly_earnings(10000.0, 12.0);
        assert!((monthly - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_gas_cost_proportion() {
        let prop = YieldScanner::estimate_gas_cost_proportion(50.0, 1000.0);
        assert!((prop - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_gas_cost_proportion_high() {
        let prop = YieldScanner::estimate_gas_cost_proportion(100.0, 50.0);
        assert!((prop - 1.0).abs() < 0.01);
    }
}
