use super::chain::ChainType;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Position {
    pub chain: ChainType,
    pub asset: String,
    pub amount: f64,
    pub value_usd: f64,
    pub cost_basis_usd: f64,
}

#[derive(Clone, Debug)]
pub struct PortfolioSummary {
    pub total_value_usd: f64,
    pub total_cost_basis: f64,
    pub total_pnl: f64,
    pub pnl_percent: f64,
    pub positions: Vec<Position>,
    pub by_chain: Vec<(ChainType, f64)>,
    pub by_asset: Vec<(String, f64)>,
    pub diversification_score: f64,
}

#[derive(Clone, Debug)]
pub struct ImpermanentLoss {
    pub pair: String,
    pub price_change_pct_a: f64,
    pub price_change_pct_b: f64,
    pub hold_value: f64,
    pub lp_value: f64,
    pub il_pct: f64,
}

pub struct Portfolio {
    positions: Vec<Position>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self { positions: vec![] }
    }

    pub fn add_position(&mut self, position: Position) {
        self.positions.push(position);
    }

    pub fn remove_position(&mut self, chain: &ChainType, asset: &str) {
        self.positions
            .retain(|p| !(p.chain == *chain && p.asset == asset));
    }

    pub fn update_value(&mut self, chain: &ChainType, asset: &str, value_usd: f64) {
        if let Some(pos) = self
            .positions
            .iter_mut()
            .find(|p| p.chain == *chain && p.asset == asset)
        {
            pos.value_usd = value_usd;
        }
    }

    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    pub fn summary(&self) -> PortfolioSummary {
        let total_value: f64 = self.positions.iter().map(|p| p.value_usd).sum();
        let total_cost: f64 = self.positions.iter().map(|p| p.cost_basis_usd).sum();
        let total_pnl = total_value - total_cost;
        let pnl_pct = if total_cost > 0.0 {
            (total_pnl / total_cost) * 100.0
        } else {
            0.0
        };

        let mut by_chain: HashMap<ChainType, f64> = HashMap::new();
        let mut by_asset: HashMap<String, f64> = HashMap::new();

        for pos in &self.positions {
            *by_chain.entry(pos.chain.clone()).or_default() += pos.value_usd;
            *by_asset.entry(pos.asset.clone()).or_default() += pos.value_usd;
        }

        let mut by_chain_vec: Vec<(ChainType, f64)> = by_chain.into_iter().collect();
        by_chain_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut by_asset_vec: Vec<(String, f64)> = by_asset.into_iter().collect();
        by_asset_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let n = self.positions.len();
        let div_score = if n <= 1 {
            0.0
        } else {
            let max_single = self
                .positions
                .iter()
                .map(|p| p.value_usd)
                .fold(0.0, f64::max);
            if total_value > 0.0 {
                1.0 - (max_single / total_value)
            } else {
                0.0
            }
        };

        PortfolioSummary {
            total_value_usd: total_value,
            total_cost_basis: total_cost,
            total_pnl,
            pnl_percent: pnl_pct,
            positions: self.positions.clone(),
            by_chain: by_chain_vec,
            by_asset: by_asset_vec,
            diversification_score: div_score,
        }
    }

    pub fn by_chain(&self, chain: &ChainType) -> Vec<&Position> {
        self.positions
            .iter()
            .filter(|p| p.chain == *chain)
            .collect()
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ImpermanentLossCalculator;

impl ImpermanentLossCalculator {
    pub fn calculate(price_change_a: f64, price_change_b: f64, hold_value: f64) -> ImpermanentLoss {
        let k = price_change_a / price_change_b;
        let lp_value_ratio = 2.0 * sqrt(k) / (1.0 + k);
        let lp_value = hold_value * lp_value_ratio;
        let il_pct = (lp_value - hold_value) / hold_value * 100.0;

        ImpermanentLoss {
            pair: format!(
                "{:.1}% / {:.1}%",
                price_change_a * 100.0,
                price_change_b * 100.0
            ),
            price_change_pct_a: price_change_a,
            price_change_pct_b: price_change_b,
            hold_value,
            lp_value,
            il_pct,
        }
    }

    pub fn il_formula(price_ratio: f64) -> f64 {
        let sqrt_ratio = sqrt(price_ratio);
        2.0 * sqrt_ratio / (1.0 + price_ratio) - 1.0
    }

    pub fn rebalance_suggestion(
        current_allocations: &[(String, f64)],
        target_allocations: &[(String, f64)],
        total_value: f64,
    ) -> Vec<(String, f64, f64)> {
        let target_map: HashMap<&str, f64> = target_allocations
            .iter()
            .map(|(a, p)| (a.as_str(), *p))
            .collect();

        let mut suggestions = Vec::new();

        for (asset, curr_pct) in current_allocations {
            let target_pct = target_map.get(asset.as_str()).copied().unwrap_or(0.0);
            let diff = target_pct - curr_pct;
            let abs_diff_usd = (diff / 100.0) * total_value;
            if abs_diff_usd.abs() > 10.0 {
                suggestions.push((asset.clone(), diff, abs_diff_usd));
            }
        }

        suggestions
    }
}

fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<Position> {
        vec![
            Position {
                chain: ChainType::Ethereum,
                asset: "ETH".into(),
                amount: 1.5,
                value_usd: 4500.0,
                cost_basis_usd: 3000.0,
            },
            Position {
                chain: ChainType::Ethereum,
                asset: "USDC".into(),
                amount: 5000.0,
                value_usd: 5000.0,
                cost_basis_usd: 5000.0,
            },
            Position {
                chain: ChainType::Bsc,
                asset: "BNB".into(),
                amount: 10.0,
                value_usd: 6000.0,
                cost_basis_usd: 4000.0,
            },
        ]
    }

    #[test]
    fn test_portfolio_add() {
        let mut p = Portfolio::new();
        assert!(p.positions().is_empty());

        for pos in sample_positions() {
            p.add_position(pos);
        }
        assert_eq!(p.positions().len(), 3);
    }

    #[test]
    fn test_portfolio_summary() {
        let mut p = Portfolio::new();
        for pos in sample_positions() {
            p.add_position(pos);
        }
        let s = p.summary();
        assert!((s.total_value_usd - 15500.0).abs() < 0.01);
        assert!((s.total_cost_basis - 12000.0).abs() < 0.01);
        assert!((s.total_pnl - 3500.0).abs() < 0.01);
        assert!((s.pnl_percent - 29.166).abs() < 0.1);
    }

    #[test]
    fn test_by_chain() {
        let mut p = Portfolio::new();
        for pos in sample_positions() {
            p.add_position(pos);
        }
        let eth = p.by_chain(&ChainType::Ethereum);
        assert_eq!(eth.len(), 2);
        let bsc = p.by_chain(&ChainType::Bsc);
        assert_eq!(bsc.len(), 1);
    }

    #[test]
    fn test_remove_position() {
        let mut p = Portfolio::new();
        for pos in sample_positions() {
            p.add_position(pos);
        }
        p.remove_position(&ChainType::Ethereum, "USDC");
        assert_eq!(p.positions().len(), 2);
    }

    #[test]
    fn test_update_value() {
        let mut p = Portfolio::new();
        for pos in sample_positions() {
            p.add_position(pos);
        }
        p.update_value(&ChainType::Ethereum, "ETH", 6000.0);
        let s = p.summary();
        assert!((s.total_value_usd - 17000.0).abs() < 0.01);
    }

    #[test]
    fn test_diversification_score() {
        let mut p = Portfolio::new();
        p.add_position(Position {
            chain: ChainType::Ethereum,
            asset: "ETH".into(),
            amount: 1.0,
            value_usd: 10000.0,
            cost_basis_usd: 5000.0,
        });
        let s = p.summary();
        assert!((s.diversification_score - 0.0).abs() < 0.01);

        p.add_position(Position {
            chain: ChainType::Bsc,
            asset: "BNB".into(),
            amount: 10.0,
            value_usd: 5000.0,
            cost_basis_usd: 4000.0,
        });
        let s = p.summary();
        assert!(s.diversification_score > 0.3);
    }

    #[test]
    fn test_impermanent_loss() {
        let il = ImpermanentLossCalculator::calculate(1.5, 1.0, 1000.0);
        assert!(il.il_pct < 0.0);
        assert!(il.lp_value < il.hold_value);
    }

    #[test]
    fn test_impermanent_loss_no_change() {
        let il = ImpermanentLossCalculator::calculate(1.0, 1.0, 1000.0);
        assert!((il.il_pct).abs() < 0.01);
        assert!((il.lp_value - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_il_formula() {
        let loss = ImpermanentLossCalculator::il_formula(1.0);
        assert!((loss).abs() < 0.01);

        let loss = ImpermanentLossCalculator::il_formula(4.0);
        assert!((loss - (-0.2)).abs() < 0.01);
    }

    #[test]
    fn test_rebalance_suggestion() {
        let current = vec![("ETH".into(), 50.0), ("USDC".into(), 50.0)];
        let target = vec![("ETH".into(), 40.0), ("USDC".into(), 60.0)];
        let suggestions =
            ImpermanentLossCalculator::rebalance_suggestion(&current, &target, 10000.0);
        assert!(!suggestions.is_empty());
        let eth_suggestion = suggestions.iter().find(|(a, _, _)| a == "ETH").unwrap();
        assert!((eth_suggestion.1 - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_empty_portfolio_summary() {
        let p = Portfolio::new();
        let s = p.summary();
        assert!((s.total_value_usd - 0.0).abs() < 0.01);
        assert!((s.total_pnl - 0.0).abs() < 0.01);
    }
}
