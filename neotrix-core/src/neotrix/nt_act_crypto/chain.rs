use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Bsc,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    Avalanche,
    Fantom,
    Cronos,
    ZkSync,
    Linea,
    Scroll,
    Solana,
    Bitcoin,
}

impl ChainType {
    pub fn all() -> Vec<ChainType> {
        vec![
            ChainType::Ethereum,
            ChainType::Bsc,
            ChainType::Polygon,
            ChainType::Arbitrum,
            ChainType::Optimism,
            ChainType::Base,
            ChainType::Avalanche,
            ChainType::Fantom,
            ChainType::Cronos,
            ChainType::ZkSync,
            ChainType::Linea,
            ChainType::Scroll,
        ]
    }

    pub fn evm_chains() -> Vec<ChainType> {
        ChainType::all()
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            ChainType::Ethereum => 1,
            ChainType::Bsc => 56,
            ChainType::Polygon => 137,
            ChainType::Arbitrum => 42161,
            ChainType::Optimism => 10,
            ChainType::Base => 8453,
            ChainType::Avalanche => 43114,
            ChainType::Fantom => 250,
            ChainType::Cronos => 25,
            ChainType::ZkSync => 324,
            ChainType::Linea => 59144,
            ChainType::Scroll => 534352,
            ChainType::Solana => 0,
            ChainType::Bitcoin => 0,
        }
    }

    pub fn explorer_url(&self) -> &str {
        match self {
            ChainType::Ethereum => "https://etherscan.io",
            ChainType::Bsc => "https://bscscan.com",
            ChainType::Polygon => "https://polygonscan.com",
            ChainType::Arbitrum => "https://arbiscan.io",
            ChainType::Optimism => "https://optimistic.etherscan.io",
            ChainType::Base => "https://basescan.org",
            ChainType::Avalanche => "https://snowtrace.io",
            ChainType::Fantom => "https://ftmscan.com",
            ChainType::Cronos => "https://cronoscan.com",
            ChainType::ZkSync => "https://explorer.zksync.io",
            ChainType::Linea => "https://lineascan.build",
            ChainType::Scroll => "https://scrollscan.com",
            ChainType::Solana => "https://solscan.io",
            ChainType::Bitcoin => "https://blockstream.info",
        }
    }

    pub fn native_currency(&self) -> &str {
        match self {
            ChainType::Ethereum => "ETH",
            ChainType::Bsc => "BNB",
            ChainType::Polygon => "MATIC",
            ChainType::Arbitrum => "ETH",
            ChainType::Optimism => "ETH",
            ChainType::Base => "ETH",
            ChainType::Avalanche => "AVAX",
            ChainType::Fantom => "FTM",
            ChainType::Cronos => "CRO",
            ChainType::ZkSync => "ETH",
            ChainType::Linea => "ETH",
            ChainType::Scroll => "ETH",
            ChainType::Solana => "SOL",
            ChainType::Bitcoin => "BTC",
        }
    }
}

impl fmt::Display for ChainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            ChainType::Ethereum => "ethereum",
            ChainType::Bsc => "bsc",
            ChainType::Polygon => "polygon",
            ChainType::Arbitrum => "arbitrum",
            ChainType::Optimism => "optimism",
            ChainType::Base => "base",
            ChainType::Avalanche => "avalanche",
            ChainType::Fantom => "fantom",
            ChainType::Cronos => "cronos",
            ChainType::ZkSync => "zksync",
            ChainType::Linea => "linea",
            ChainType::Scroll => "scroll",
            ChainType::Solana => "solana",
            ChainType::Bitcoin => "bitcoin",
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChainConfig {
    pub chain: ChainType,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub api_key: Option<String>,
}

impl ChainConfig {
    pub fn new(chain: ChainType, rpc_url: &str) -> Self {
        Self {
            chain,
            rpc_url: rpc_url.to_string(),
            ws_url: None,
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChainRegistry {
    chains: HashMap<ChainType, ChainConfig>,
}

impl ChainRegistry {
    pub fn new() -> Self {
        Self { chains: HashMap::new() }
    }

    pub fn register(&mut self, config: ChainConfig) {
        self.chains.insert(config.chain.clone(), config);
    }

    pub fn get(&self, chain: &ChainType) -> Option<&ChainConfig> {
        self.chains.get(chain)
    }

    pub fn chains(&self) -> &HashMap<ChainType, ChainConfig> {
        &self.chains
    }

    pub fn connected_count(&self) -> usize {
        self.chains.len()
    }

    pub fn register_defaults(&mut self) {
        let defaults: Vec<(ChainType, &str)> = vec![
            (ChainType::Ethereum, "https://eth-mainnet.g.alchemy.com/v2/demo"),
            (ChainType::Bsc, "https://bsc-dataseed.binance.org/"),
            (ChainType::Polygon, "https://polygon-mainnet.g.alchemy.com/v2/demo"),
            (ChainType::Arbitrum, "https://arb-mainnet.g.alchemy.com/v2/demo"),
            (ChainType::Optimism, "https://opt-mainnet.g.alchemy.com/v2/demo"),
            (ChainType::Base, "https://base-mainnet.g.alchemy.com/v2/demo"),
            (ChainType::Avalanche, "https://api.avax.network/ext/bc/C/rpc"),
            (ChainType::ZkSync, "https://mainnet.era.zksync.io"),
            (ChainType::Scroll, "https://rpc.scroll.io"),
            (ChainType::Linea, "https://rpc.linea.build"),
        ];
        for (chain, url) in defaults {
            let env_key = format!("NEOTRIX_{}_RPC_URL", chain.to_string().to_uppercase());
            let final_url = std::env::var(&env_key).unwrap_or_else(|_| url.to_string());
            self.register(ChainConfig::new(chain, &final_url));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_evm_filter() {
        let chains = ChainType::evm_chains();
        assert!(chains.contains(&ChainType::Ethereum));
        assert!(chains.contains(&ChainType::Bsc));
        assert!(!chains.contains(&ChainType::Solana));
    }

    #[test]
    fn test_chain_id() {
        assert_eq!(ChainType::Ethereum.chain_id(), 1);
        assert_eq!(ChainType::Bsc.chain_id(), 56);
        assert_eq!(ChainType::Solana.chain_id(), 0);
    }

    #[test]
    fn test_registry_defaults() {
        let mut reg = ChainRegistry::new();
        reg.register_defaults();
        assert!(reg.connected_count() > 0);
        assert!(reg.get(&ChainType::Ethereum).is_some());
    }

    #[test]
    fn test_native_currency() {
        assert_eq!(ChainType::Ethereum.native_currency(), "ETH");
        assert_eq!(ChainType::Solana.native_currency(), "SOL");
    }
}
