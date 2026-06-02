use super::chain::ChainType;
use super::evm::MultiEvmClient;

#[derive(Clone, Debug)]
pub struct NewPoolEvent {
    pub chain: ChainType,
    pub pair_address: String,
    pub token0: String,
    pub token1: String,
    pub dex_name: String,
    pub created_at_block: u64,
    pub initial_liquidity_usd: f64,
}

#[derive(Clone, Debug)]
pub struct WhaleTx {
    pub chain: ChainType,
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value_usd: f64,
    pub token: String,
    pub timestamp: i64,
    pub category: WhaleCategory,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WhaleCategory {
    LargeTransfer,
    DexSwap,
    CexDeposit,
    CexWithdrawal,
    BridgeTransfer,
    Unknown,
}

impl WhaleCategory {
    pub fn name(&self) -> &str {
        match self {
            WhaleCategory::LargeTransfer => "Large Transfer",
            WhaleCategory::DexSwap => "DEX Swap",
            WhaleCategory::CexDeposit => "CEX Deposit",
            WhaleCategory::CexWithdrawal => "CEX Withdrawal",
            WhaleCategory::BridgeTransfer => "Bridge",
            WhaleCategory::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiquidationEvent {
    pub chain: ChainType,
    pub protocol: String,
    pub borrower: String,
    pub debt_token: String,
    pub debt_amount_usd: f64,
    pub collateral_token: String,
    pub collateral_amount_usd: f64,
    pub health_factor: f64,
    pub profit_potential_usd: f64,
    pub block_number: u64,
}

#[derive(Clone, Debug)]
pub struct MempoolTx {
    pub tx_hash: String,
    pub to: String,
    pub value_eth: f64,
    pub gas_price_gwei: f64,
    pub is_frontrunable: bool,
    pub is_sandwichable: bool,
}

#[allow(dead_code)]
#[allow(dead_code)]
const _WHALE_THRESHOLD_USD: f64 = 100_000.0;
const LIQUIDATION_MIN_PROFIT: f64 = 50.0;
const MIN_HEALTH_FACTOR: f64 = 1.05;

pub struct ChainMonitor {
    new_pools: Vec<NewPoolEvent>,
    whale_txs: Vec<WhaleTx>,
    liquidations: Vec<LiquidationEvent>,
    mempool: Vec<MempoolTx>,
    max_history: usize,
}

impl ChainMonitor {
    pub fn new() -> Self {
        Self {
            new_pools: Vec::new(),
            whale_txs: Vec::new(),
            liquidations: Vec::new(),
            mempool: Vec::new(),
            max_history: 500,
        }
    }

    pub fn record_pool(&mut self, pool: NewPoolEvent) {
        self.new_pools.push(pool);
        if self.new_pools.len() > self.max_history {
            self.new_pools.remove(0);
        }
    }

    pub fn record_whale(&mut self, tx: WhaleTx) {
        self.whale_txs.push(tx);
        if self.whale_txs.len() > self.max_history {
            self.whale_txs.remove(0);
        }
    }

    pub fn record_liquidation(&mut self, event: LiquidationEvent) {
        self.liquidations.push(event);
        if self.liquidations.len() > self.max_history {
            self.liquidations.remove(0);
        }
    }

    pub fn recent_pools(&self, limit: usize) -> &[NewPoolEvent] {
        let start = self.new_pools.len().saturating_sub(limit);
        &self.new_pools[start..]
    }

    pub fn recent_whales(&self, limit: usize) -> &[WhaleTx] {
        let start = self.whale_txs.len().saturating_sub(limit);
        &self.whale_txs[start..]
    }

    pub fn profitable_liquidations(&self) -> Vec<&LiquidationEvent> {
        self.liquidations
            .iter()
            .filter(|e| e.profit_potential_usd >= LIQUIDATION_MIN_PROFIT)
            .collect()
    }

    pub fn liquidation_alerts(&self) -> Vec<&LiquidationEvent> {
        self.liquidations
            .iter()
            .filter(|e| e.health_factor <= MIN_HEALTH_FACTOR)
            .collect()
    }

    pub fn scan_new_pools(
        &mut self,
        _clients: &MultiEvmClient,
    ) -> Vec<NewPoolEvent> {
        vec![]
    }

    pub fn scan_whales(
        &mut self,
        _clients: &MultiEvmClient,
        _min_usd: f64,
    ) -> Vec<WhaleTx> {
        vec![]
    }

    pub fn scan_liquidations(
        &mut self,
        _clients: &MultiEvmClient,
    ) -> Vec<LiquidationEvent> {
        vec![]
    }

    pub fn add_to_mempool(&mut self, tx: MempoolTx) {
        self.mempool.push(tx);
        if self.mempool.len() > self.max_history {
            self.mempool.remove(0);
        }
    }

    pub fn sandwichable_txs(&self) -> Vec<&MempoolTx> {
        self.mempool.iter().filter(|tx| tx.is_sandwichable).collect()
    }
}

impl Default for ChainMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ChainMonitor::new();
        assert!(monitor.recent_pools(10).is_empty());
        assert!(monitor.recent_whales(10).is_empty());
    }

    #[test]
    fn test_record_pool() {
        let mut monitor = ChainMonitor::new();
        monitor.record_pool(NewPoolEvent {
            chain: ChainType::Ethereum,
            pair_address: "0x1234".into(),
            token0: "0xaaaa".into(),
            token1: "0xbbbb".into(),
            dex_name: "Uniswap V3".into(),
            created_at_block: 1000,
            initial_liquidity_usd: 50000.0,
        });
        assert_eq!(monitor.recent_pools(10).len(), 1);
    }

    #[test]
    fn test_record_whale() {
        let mut monitor = ChainMonitor::new();
        monitor.record_whale(WhaleTx {
            chain: ChainType::Ethereum,
            tx_hash: "0xabc".into(),
            from: "0xwhale".into(),
            to: "0xcex".into(),
            value_usd: 5_000_000.0,
            token: "ETH".into(),
            timestamp: 1000,
            category: WhaleCategory::CexDeposit,
        });
        assert_eq!(monitor.recent_whales(10).len(), 1);
    }

    #[test]
    fn test_profitable_liquidations() {
        let mut monitor = ChainMonitor::new();
        monitor.record_liquidation(LiquidationEvent {
            chain: ChainType::Ethereum,
            protocol: "AAVE".into(),
            borrower: "0xborrower".into(),
            debt_token: "USDC".into(),
            debt_amount_usd: 10000.0,
            collateral_token: "ETH".into(),
            collateral_amount_usd: 15000.0,
            health_factor: 0.95,
            profit_potential_usd: 100.0,
            block_number: 1000,
        });
        assert_eq!(monitor.profitable_liquidations().len(), 1);
    }

    #[test]
    fn test_liquidation_alerts() {
        let mut monitor = ChainMonitor::new();
        monitor.record_liquidation(LiquidationEvent {
            chain: ChainType::Ethereum,
            protocol: "Compound".into(),
            borrower: "0xborrower".into(),
            debt_token: "USDC".into(),
            debt_amount_usd: 5000.0,
            collateral_token: "ETH".into(),
            collateral_amount_usd: 5100.0,
            health_factor: 1.02,
            profit_potential_usd: 10.0,
            block_number: 2000,
        });
        assert_eq!(monitor.liquidation_alerts().len(), 1);
    }

    #[test]
    fn test_sandwichable_txs() {
        let mut monitor = ChainMonitor::new();
        monitor.add_to_mempool(MempoolTx {
            tx_hash: "0x1".into(),
            to: "0xrouter".into(),
            value_eth: 0.0,
            gas_price_gwei: 100.0,
            is_frontrunable: true,
            is_sandwichable: true,
        });
        monitor.add_to_mempool(MempoolTx {
            tx_hash: "0x2".into(),
            to: "0xcontract".into(),
            value_eth: 0.5,
            gas_price_gwei: 50.0,
            is_frontrunable: false,
            is_sandwichable: false,
        });
        assert_eq!(monitor.sandwichable_txs().len(), 1);
    }

    #[test]
    fn test_whale_category_names() {
        assert_eq!(WhaleCategory::CexDeposit.name(), "CEX Deposit");
        assert_eq!(WhaleCategory::DexSwap.name(), "DEX Swap");
    }

    #[test]
    fn test_max_history() {
        let mut monitor = ChainMonitor::new();
        for i in 0..600 {
            monitor.record_whale(WhaleTx {
                chain: ChainType::Ethereum,
                tx_hash: format!("0x{i}"),
                from: "0xfrom".into(),
                to: "0xto".into(),
                value_usd: 100000.0,
                token: "ETH".into(),
                timestamp: i,
                category: WhaleCategory::LargeTransfer,
            });
        }
        assert_eq!(monitor.whale_txs.len(), 500);
    }
}
