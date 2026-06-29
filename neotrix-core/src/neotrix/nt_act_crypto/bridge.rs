use super::chain::ChainType;

#[derive(Clone, Debug)]
pub struct BridgeRoute {
    pub name: String,
    pub protocol: BridgeProtocol,
    pub from_chain: ChainType,
    pub to_chain: ChainType,
    pub estimated_fee_usd: f64,
    pub estimated_time_min: u32,
    pub liquidity_usd: f64,
    pub supports_token: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BridgeProtocol {
    Stargate,
    Across,
    Hop,
    Ccip,
    LayerZero,
    Custom(String),
}

impl BridgeProtocol {
    pub fn name(&self) -> &str {
        match self {
            BridgeProtocol::Stargate => "Stargate",
            BridgeProtocol::Across => "Across",
            BridgeProtocol::Hop => "Hop",
            BridgeProtocol::Ccip => "CCIP",
            BridgeProtocol::LayerZero => "LayerZero",
            BridgeProtocol::Custom(n) => n,
        }
    }

    pub fn all() -> Vec<BridgeProtocol> {
        vec![
            BridgeProtocol::Stargate,
            BridgeProtocol::Across,
            BridgeProtocol::Hop,
            BridgeProtocol::Ccip,
            BridgeProtocol::LayerZero,
        ]
    }
}

#[derive(Clone, Debug)]
pub struct BridgeTx {
    pub protocol: BridgeProtocol,
    pub from_chain: ChainType,
    pub to_chain: ChainType,
    pub token: String,
    pub amount: u128,
    pub receiver: String,
    pub estimated_gas: u64,
    pub bridge_fee_usd: f64,
}

pub struct BridgeRegistry {
    routes: Vec<BridgeRoute>,
}

impl BridgeRegistry {
    pub fn new() -> Self {
        Self { routes: vec![] }
    }

    pub fn register_defaults(&mut self) {
        let default_routes = vec![
            BridgeRoute {
                name: "Stargate USDC".into(),
                protocol: BridgeProtocol::Stargate,
                from_chain: ChainType::Ethereum,
                to_chain: ChainType::Arbitrum,
                estimated_fee_usd: 0.6,
                estimated_time_min: 5,
                liquidity_usd: 50_000_000.0,
                supports_token: vec!["USDC".into(), "USDT".into(), "ETH".into()],
            },
            BridgeRoute {
                name: "Stargate USDC".into(),
                protocol: BridgeProtocol::Stargate,
                from_chain: ChainType::Ethereum,
                to_chain: ChainType::Optimism,
                estimated_fee_usd: 0.5,
                estimated_time_min: 5,
                liquidity_usd: 30_000_000.0,
                supports_token: vec!["USDC".into(), "USDT".into()],
            },
            BridgeRoute {
                name: "Across USDC".into(),
                protocol: BridgeProtocol::Across,
                from_chain: ChainType::Ethereum,
                to_chain: ChainType::Arbitrum,
                estimated_fee_usd: 0.4,
                estimated_time_min: 3,
                liquidity_usd: 100_000_000.0,
                supports_token: vec!["USDC".into(), "WETH".into()],
            },
            BridgeRoute {
                name: "Across USDC".into(),
                protocol: BridgeProtocol::Across,
                from_chain: ChainType::Ethereum,
                to_chain: ChainType::Base,
                estimated_fee_usd: 0.3,
                estimated_time_min: 2,
                liquidity_usd: 40_000_000.0,
                supports_token: vec!["USDC".into(), "ETH".into()],
            },
            BridgeRoute {
                name: "Hop USDC".into(),
                protocol: BridgeProtocol::Hop,
                from_chain: ChainType::Polygon,
                to_chain: ChainType::Ethereum,
                estimated_fee_usd: 0.8,
                estimated_time_min: 10,
                liquidity_usd: 20_000_000.0,
                supports_token: vec!["USDC".into(), "MATIC".into()],
            },
            BridgeRoute {
                name: "Stargate USDC".into(),
                protocol: BridgeProtocol::Stargate,
                from_chain: ChainType::Bsc,
                to_chain: ChainType::Polygon,
                estimated_fee_usd: 0.4,
                estimated_time_min: 4,
                liquidity_usd: 15_000_000.0,
                supports_token: vec!["USDC".into(), "USDT".into(), "BNB".into()],
            },
        ];
        self.routes.extend(default_routes);
    }

    pub fn find_routes(&self, from: &ChainType, to: &ChainType, token: &str) -> Vec<&BridgeRoute> {
        let token_upper = token.to_uppercase();
        self.routes
            .iter()
            .filter(|r| {
                r.from_chain == *from
                    && r.to_chain == *to
                    && r.supports_token
                        .iter()
                        .any(|t| t.to_uppercase() == token_upper)
            })
            .collect()
    }

    pub fn find_all_from(&self, from: &ChainType) -> Vec<&BridgeRoute> {
        self.routes
            .iter()
            .filter(|r| r.from_chain == *from)
            .collect()
    }

    pub fn cheapest_route(
        &self,
        from: &ChainType,
        to: &ChainType,
        token: &str,
    ) -> Option<&BridgeRoute> {
        let mut routes = self.find_routes(from, to, token);
        routes.sort_by(|a, b| {
            a.estimated_fee_usd
                .partial_cmp(&b.estimated_fee_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        routes.into_iter().next()
    }

    pub fn all_routes(&self) -> &[BridgeRoute] {
        &self.routes
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BridgeOpportunity {
    pub route: BridgeRoute,
    pub amount_usd: f64,
    pub profit_after_fee: f64,
    pub arb_opportunity: bool,
}

pub struct BridgeAnalyzer;

impl BridgeAnalyzer {
    pub fn estimate_bridge_cost(
        _from_chain: &ChainType,
        gas_price_gwei: f64,
        gas_limit: u64,
        bridge_fee_usd: f64,
        native_price_usd: f64,
    ) -> f64 {
        let gas_cost_eth = (gas_limit as f64 * gas_price_gwei) / 1e9;
        let gas_cost_usd = gas_cost_eth * native_price_usd;
        gas_cost_usd + bridge_fee_usd
    }

    pub fn find_arbitrage(
        amount_usd: f64,
        price_on_from: f64,
        price_on_to: f64,
        bridge_cost_usd: f64,
    ) -> Option<f64> {
        if price_on_from <= 0.0 {
            return None;
        }
        let bought = amount_usd / price_on_from;
        let sold = bought * price_on_to;
        let profit = sold - amount_usd - bridge_cost_usd;
        if profit > 0.0 {
            Some(profit)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_registry_defaults() {
        let reg = BridgeRegistry::new();
        assert!(reg.all_routes().is_empty());

        let mut reg = BridgeRegistry::new();
        reg.register_defaults();
        assert!(!reg.all_routes().is_empty());
    }

    #[test]
    fn test_find_routes() {
        let mut reg = BridgeRegistry::new();
        reg.register_defaults();
        let routes = reg.find_routes(&ChainType::Ethereum, &ChainType::Arbitrum, "USDC");
        assert!(!routes.is_empty());
        assert!(routes
            .iter()
            .any(|r| r.protocol == BridgeProtocol::Stargate));
        assert!(routes.iter().any(|r| r.protocol == BridgeProtocol::Across));
    }

    #[test]
    fn test_cheapest_route() {
        let mut reg = BridgeRegistry::new();
        reg.register_defaults();
        let cheapest = reg.cheapest_route(&ChainType::Ethereum, &ChainType::Arbitrum, "USDC");
        assert!(cheapest.is_some());
        assert_eq!(cheapest.unwrap().protocol, BridgeProtocol::Across);
    }

    #[test]
    fn test_find_all_from() {
        let mut reg = BridgeRegistry::new();
        reg.register_defaults();
        let from_eth = reg.find_all_from(&ChainType::Ethereum);
        assert_eq!(from_eth.len(), 4);
    }

    #[test]
    fn test_bridge_protocol_names() {
        assert_eq!(BridgeProtocol::Stargate.name(), "Stargate");
        assert_eq!(BridgeProtocol::Custom("XBridge".into()).name(), "XBridge");
    }

    #[test]
    fn test_estimate_bridge_cost() {
        let cost =
            BridgeAnalyzer::estimate_bridge_cost(&ChainType::Ethereum, 50.0, 100_000, 0.5, 3000.0);
        assert!((cost - 15.5).abs() < 1.0);
    }

    #[test]
    fn test_find_arbitrage_positive() {
        let profit = BridgeAnalyzer::find_arbitrage(1000.0, 1.0, 1.05, 10.0);
        assert!(profit.is_some());
        assert!((profit.unwrap() - 40.0).abs() < 1.0);
    }

    #[test]
    fn test_find_arbitrage_negative() {
        let profit = BridgeAnalyzer::find_arbitrage(1000.0, 1.0, 0.95, 10.0);
        assert!(profit.is_none());
    }

    #[test]
    fn test_no_routes_for_unknown_pair() {
        let reg = BridgeRegistry::new();
        let routes = reg.find_routes(&ChainType::Ethereum, &ChainType::Solana, "BTC");
        assert!(routes.is_empty());
    }
}
