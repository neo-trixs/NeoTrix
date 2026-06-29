use super::chain::ChainType;
use super::evm::MultiEvmClient;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GasPriceInfo {
    pub chain: ChainType,
    pub base_fee: f64,
    pub priority_fee: f64,
    pub max_fee: f64,
    pub timestamp: i64,
}

#[derive(Clone, Debug)]
pub struct GasStats {
    pub recent_base_fees: Vec<f64>,
    pub recent_priority_fees: Vec<f64>,
    pub avg_base_fee: f64,
    pub avg_priority_fee: f64,
    pub p25_base_fee: f64,
    pub p75_base_fee: f64,
}

pub struct GasTracker {
    history: HashMap<ChainType, Vec<GasPriceInfo>>,
    max_history: usize,
}

impl GasTracker {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            max_history: 100,
        }
    }

    pub fn with_max_history(max: usize) -> Self {
        Self {
            history: HashMap::new(),
            max_history: max,
        }
    }

    pub fn refresh(&mut self, clients: &MultiEvmClient) {
        for (chain, client) in clients.clients() {
            let base_fee = client.get_gas_price().unwrap_or(10.0);

            let priority_fee = match client.chain {
                ChainType::Ethereum
                | ChainType::Arbitrum
                | ChainType::Optimism
                | ChainType::Base => (base_fee * 0.1).max(0.1),
                _ => base_fee * 0.05,
            };

            let max_fee = (base_fee + priority_fee) * 1.5;

            let info = GasPriceInfo {
                chain: chain.clone(),
                base_fee,
                priority_fee,
                max_fee,
                timestamp: chrono::Utc::now().timestamp(),
            };

            let entry = self.history.entry(chain.clone()).or_default();
            entry.push(info);
            if entry.len() > self.max_history {
                entry.remove(0);
            }
        }
    }

    pub fn current(&self, chain: &ChainType) -> Option<&GasPriceInfo> {
        self.history.get(chain).and_then(|h| h.last())
    }

    pub fn optimal_gas(&self, chain: &ChainType, speed: Speed) -> (f64, f64) {
        let current = match self.current(chain) {
            Some(c) => c,
            None => return (10.0, 1.0),
        };

        match speed {
            Speed::Slow => (current.base_fee, current.priority_fee * 0.8),
            Speed::Standard => (current.base_fee * 1.1, current.priority_fee),
            Speed::Fast => (current.base_fee * 1.3, current.priority_fee * 1.5),
            Speed::Ape => (current.base_fee * 2.0, current.priority_fee * 3.0),
        }
    }

    pub fn stats(&self, chain: &ChainType) -> Option<GasStats> {
        let entries = self.history.get(chain)?;
        if entries.is_empty() {
            return None;
        }

        let base_fees: Vec<f64> = entries.iter().map(|e| e.base_fee).collect();
        let priority_fees: Vec<f64> = entries.iter().map(|e| e.priority_fee).collect();
        let mut sorted = base_fees.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        let p25 = sorted[len / 4];
        let p75 = sorted[len * 3 / 4];

        let avg_base = base_fees.iter().sum::<f64>() / base_fees.len() as f64;
        let avg_prio = priority_fees.iter().sum::<f64>() / priority_fees.len() as f64;

        Some(GasStats {
            recent_base_fees: base_fees,
            recent_priority_fees: priority_fees,
            avg_base_fee: avg_base,
            avg_priority_fee: avg_prio,
            p25_base_fee: p25,
            p75_base_fee: p75,
        })
    }

    pub fn is_gas_acceptable(&self, chain: &ChainType, max_acceptable_gwei: f64) -> bool {
        self.current(chain)
            .map(|g| g.max_fee <= max_acceptable_gwei)
            .unwrap_or(true)
    }

    pub fn cheapest_chain(&self, chains: &[ChainType]) -> Option<(ChainType, f64)> {
        chains
            .iter()
            .filter_map(|c| self.current(c).map(|g| (c.clone(), g.base_fee)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl Default for GasTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Speed {
    Slow,
    Standard,
    Fast,
    Ape,
}

impl Speed {
    pub fn label(&self) -> &str {
        match self {
            Speed::Slow => "slow",
            Speed::Standard => "standard",
            Speed::Fast => "fast",
            Speed::Ape => "ape",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s {
            "slow" => Speed::Slow,
            "fast" => Speed::Fast,
            "ape" => Speed::Ape,
            _ => Speed::Standard,
        }
    }
}

pub fn estimate_tx_cost_usd(gas_limit: u64, max_fee_gwei: f64, token_price_usd: f64) -> f64 {
    let gas_in_eth = (gas_limit as f64 * max_fee_gwei) / 1e9;
    gas_in_eth * token_price_usd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_tracker_creation() {
        let tracker = GasTracker::new();
        assert_eq!(tracker.max_history, 100);
    }

    #[test]
    fn test_current_none_initially() {
        let tracker = GasTracker::new();
        assert!(tracker.current(&ChainType::Ethereum).is_none());
    }

    #[test]
    fn test_optimal_gas_defaults() {
        let tracker = GasTracker::new();
        let (base, prio) = tracker.optimal_gas(&ChainType::Ethereum, Speed::Standard);
        assert_eq!(base, 10.0);
        assert_eq!(prio, 1.0);
    }

    #[test]
    fn test_speed_labels() {
        assert_eq!(Speed::Slow.label(), "slow");
        assert_eq!(Speed::Standard.label(), "standard");
        assert_eq!(Speed::Fast.label(), "fast");
        assert_eq!(Speed::Ape.label(), "ape");
    }

    #[test]
    fn test_speed_from_label() {
        assert_eq!(Speed::from_label("ape"), Speed::Ape);
        assert_eq!(Speed::from_label("unknown"), Speed::Standard);
    }

    #[test]
    fn test_estimate_tx_cost() {
        let cost = estimate_tx_cost_usd(21000, 50.0, 3000.0);
        assert!((cost - 3.15).abs() < 0.01);
    }

    #[test]
    fn test_speed_multipliers() {
        let mut tracker = GasTracker::new();
        let entry = GasPriceInfo {
            chain: ChainType::Ethereum,
            base_fee: 20.0,
            priority_fee: 2.0,
            max_fee: 30.0,
            timestamp: 1000,
        };
        tracker
            .history
            .entry(ChainType::Ethereum)
            .or_default()
            .push(entry);

        let (base, prio) = tracker.optimal_gas(&ChainType::Ethereum, Speed::Fast);
        assert!((base - 26.0).abs() < 0.01);
        assert!((prio - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_cheapest_chain() {
        let mut tracker = GasTracker::new();
        for (chain, base) in [
            (ChainType::Ethereum, 50.0),
            (ChainType::Polygon, 100.0),
            (ChainType::Bsc, 5.0),
        ] {
            let chain_name = chain.clone();
            tracker
                .history
                .entry(chain)
                .or_default()
                .push(GasPriceInfo {
                    chain: chain_name,
                    base_fee: base,
                    priority_fee: base * 0.1,
                    max_fee: base * 1.5,
                    timestamp: 1000,
                });
        }

        let chains = vec![ChainType::Ethereum, ChainType::Polygon, ChainType::Bsc];
        let cheapest = tracker.cheapest_chain(&chains);
        assert!(cheapest.is_some());
        assert_eq!(cheapest.unwrap().0, ChainType::Bsc);
    }
}
