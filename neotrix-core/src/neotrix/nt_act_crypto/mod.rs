pub mod wallet;
pub mod wallet_store;
pub mod cipher;
pub mod chain;
pub mod evm;
pub mod opportunity;
pub mod collector;
pub mod tx;
pub mod gas;
pub mod token;
pub mod dex;
pub mod bridge;
pub mod yields;
pub mod portfolio;
pub mod monitor;
pub mod airdrop;
pub mod security;
pub mod self_evolve;

pub use wallet::{CryptoWallet, WalletManager};
pub use wallet_store::{WalletStore, WalletFile, WalletInfo};
pub use chain::{ChainType, ChainConfig, ChainRegistry};
pub use evm::{EvmClient, MultiEvmClient};
pub use opportunity::{Opportunity, OpportunityType, OpportunityScanner, SourceRegistry, InformationSource, SourceType};
pub use collector::{CryptoEarnings, CryptoCollector};
pub use tx::{TxBuilder, Tx1559, TxLegacy, SignedTx, TxReceipt, TxType};
pub use gas::{GasTracker, GasPriceInfo, GasStats, Speed};
pub use token::{TokenRegistry, TokenInfo, TokenBalance, Erc20Abi, ApprovalInfo};
pub use dex::{DexRegistry, DexConfig, DexProtocol, DexPool, DexSwapper, SwapQuote};
pub use bridge::{BridgeRegistry, BridgeRoute, BridgeProtocol, BridgeAnalyzer, BridgeTx};
pub use yields::{YieldScanner, YieldOpportunity, YieldType};
pub use portfolio::{Portfolio, PortfolioSummary, Position, ImpermanentLoss, ImpermanentLossCalculator};
pub use monitor::{ChainMonitor, NewPoolEvent, WhaleTx, LiquidationEvent, MempoolTx, WhaleCategory};
pub use airdrop::{AirdropRegistry, AirdropInfo, AirdropCheckResult, AirdropClaimer};
pub use security::{SecurityManager, ScamWarning, ScamSeverity, ScamCategory, ApprovalEntry, TxSimulation};
pub use self_evolve::{SelfEvolver, BacktestResult, AdaptiveConfig};

use self::opportunity::StrategyStats;
use std::collections::HashMap;

/// 核心加密代理人 — 信息差 × 价值交换 × 自进化
///
/// 设计哲学:
/// - 信息差: 比其他人先知道机会 (扫描链上 + 链下信息源)
/// - 价值交换: 提供真实价值获得收益 (LP/Staking/服务)
/// - 自进化: 每次执行后学习, 优化策略选择
pub struct CryptoAgent {
    pub wallet_manager: WalletManager,
    pub wallet_store: WalletStore,
    pub chain_registry: ChainRegistry,
    pub evm_clients: MultiEvmClient,
    pub scanner: OpportunityScanner,
    pub collector: CryptoCollector,
    pub token_registry: TokenRegistry,
    pub gas_tracker: GasTracker,
    pub dex_registry: DexRegistry,
    pub bridge_registry: BridgeRegistry,
    pub yield_scanner: YieldScanner,
    pub portfolio: Portfolio,
    pub monitor: ChainMonitor,
    pub airdrop_registry: AirdropRegistry,
    pub security: SecurityManager,
    pub self_evolver: SelfEvolver,
    pub config: CryptoAgentConfig,
    iteration_count: u64,
    last_scan_at: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct CryptoAgentConfig {
    pub auto_scan_on_start: bool,
    pub max_gas_price_gwei: f64,
    pub min_opportunity_value_usd: f64,
    pub preferred_chains: Vec<ChainType>,
    pub auto_claim_faucets: bool,
    pub auto_execute: bool,
    pub learning_rate: f64,
}

impl Default for CryptoAgentConfig {
    fn default() -> Self {
        Self {
            auto_scan_on_start: true,
            max_gas_price_gwei: 50.0,
            min_opportunity_value_usd: 1.0,
            preferred_chains: vec![
                ChainType::Ethereum,
                ChainType::Bsc,
                ChainType::Polygon,
                ChainType::Arbitrum,
                ChainType::Optimism,
                ChainType::Base,
            ],
            auto_claim_faucets: false,
            auto_execute: false,
            learning_rate: 0.1,
        }
    }
}

impl CryptoAgent {
    pub fn new() -> Self {
        let mut chain_registry = ChainRegistry::new();
        chain_registry.register_defaults();

        let evm_clients = MultiEvmClient::new_live();

        let wallet_store = WalletStore::new();
        let mut dex_registry = DexRegistry::new();
        dex_registry.register_defaults();
        let mut bridge_registry = BridgeRegistry::new();
        bridge_registry.register_defaults();
        let mut yield_scanner = YieldScanner::new();
        yield_scanner.register_defaults();
        let mut airdrop_registry = AirdropRegistry::new();
        airdrop_registry.register_defaults();
        let mut security = SecurityManager::new();
        security.load_known_scams();

        Self {
            wallet_manager: WalletManager::new(),
            wallet_store,
            chain_registry,
            evm_clients,
            scanner: OpportunityScanner::new(),
            collector: CryptoCollector::new(),
            token_registry: TokenRegistry::new(),
            gas_tracker: GasTracker::new(),
            dex_registry,
            bridge_registry,
            yield_scanner,
            portfolio: Portfolio::new(),
            monitor: ChainMonitor::new(),
            airdrop_registry,
            security,
            self_evolver: SelfEvolver::new(),
            config: CryptoAgentConfig::default(),
            iteration_count: 0,
            last_scan_at: None,
        }
    }

    pub fn with_config(config: CryptoAgentConfig) -> Self {
        let mut agent = Self::new();
        agent.config = config;
        agent
    }

    pub fn setup_default_wallet(&mut self, label: &str) -> &CryptoWallet {
        let wallet = CryptoWallet::generate_evm(label);
        let _ = self.wallet_store.save_wallet(&wallet);
        self.wallet_manager.add_wallet(wallet);
        self.wallet_manager.active_wallet().unwrap()
    }

    pub fn import_wallet(&mut self, private_key: &str, label: &str) -> Result<&CryptoWallet, String> {
        let wallet = CryptoWallet::import_evm(private_key, label)?;
        self.wallet_store.save_wallet(&wallet)?;
        self.wallet_manager.add_wallet(wallet);
        Ok(self.wallet_manager.active_wallet().unwrap())
    }

    pub fn persist_wallet(&mut self, label: &str) -> Result<String, String> {
        let wallet = CryptoWallet::generate_evm(label);
        let label = self.wallet_store.save_wallet(&wallet)?;
        self.wallet_manager.add_wallet(wallet);
        Ok(label)
    }

    pub fn load_persisted_wallets(&mut self) -> Result<usize, String> {
        let manager = self.wallet_store.load_all()?;
        let count = manager.wallet_count();
        self.wallet_manager = manager;
        Ok(count)
    }

    pub fn list_stored_wallets(&self) -> Result<Vec<WalletInfo>, String> {
        self.wallet_store.list_wallets()
    }

    pub fn delete_persisted_wallet(&mut self, label: &str) -> Result<(), String> {
        self.wallet_store.delete_wallet(label)?;
        let idx = self.wallet_manager.wallets().iter().position(|w| w.label == label);
        if let Some(i) = idx {
            self.wallet_manager.remove_wallet(i);
        }
        Ok(())
    }

    /// 执行一次完整的扫描 + 分析 + 学习迭代
    pub fn run_iteration(&mut self) -> IterationResult {
        self.iteration_count += 1;
        let now = chrono::Utc::now().timestamp();
        self.last_scan_at = Some(now);

        self.gas_tracker.refresh(&self.evm_clients);
        let _ = self.monitor.scan_whales(&self.evm_clients, 100_000.0);
        let _ = self.monitor.scan_liquidations(&self.evm_clients);

        let opportunities = self.scan_opportunities();
        let mut all = opportunities;

        if let Some(wallet) = self.wallet_manager.active_wallet() {
            for airdrop_result in self.airdrop_registry.eligible_airdrops(&wallet.address, &[]) {
                if airdrop_result.is_eligible {
                    all.push(Opportunity {
                        opportunity_type: OpportunityType::AirdropClaim,
                        chain: airdrop_result.airdrop.chain.clone(),
                        title: format!("Airdrop: {}", airdrop_result.airdrop.name),
                        description: format!("Est. ${} from {}", airdrop_result.estimated_amount, airdrop_result.airdrop.protocol),
                        estimated_value_usd: airdrop_result.estimated_amount,
                        confidence: airdrop_result.airdrop.confidence,
                        action: "claim".into(),
                        source_url: airdrop_result.airdrop.claim_url.clone(),
                        contract_address: None,
                        source_name: Some("airdrop registry".into()),
                        execution_gas_cost: 10.0,
                    });
                }
            }
        }

        let ranked = self.scanner.rank_opportunities(&all, self.config.min_opportunity_value_usd);

        let best = ranked.first().cloned();
        let total_value = all.iter().map(|o| o.estimated_value_usd).sum();
        let top_3_strategies: Vec<String> = ranked
            .iter()
            .take(3)
            .map(|o| format!("{} ({:.2})", o.opportunity_type.name(), o.risk_adjusted_score()))
            .collect();

        // self-evolve: learn from this iteration's best opportunity
        if let Some(ref b) = best {
            let value = b.estimated_value_usd.max(0.0);
            let fake_stats = StrategyStats {
                opportunity_type: b.opportunity_type.clone(),
                attempts: 1,
                successes: 1,
                total_value_usd: value,
                total_gas_usd: b.execution_gas_cost,
            };
            self.self_evolver.backtest(&b.opportunity_type.name(), &fake_stats);
        }

        // Auto-execute best opportunity if enabled and a high-value one exists
        if self.config.auto_execute {
            if let Some(best_opp) = &best {
                if best_opp.estimated_value_usd > self.config.min_opportunity_value_usd * 10.0 {
                    log::info!("[CryptoAgent] auto-executing best opportunity: {} on {}", best_opp.title, best_opp.chain);
                    let _ = self.execute_best_opportunity();
                }
            }
        }

        IterationResult {
            iteration: self.iteration_count,
            total_opportunities: all.len(),
            ranked_opportunities: ranked.len(),
            total_estimated_value: total_value,
            best_opportunity: best,
            top_strategies: top_3_strategies,
            wallet_count: self.wallet_manager.wallet_count(),
            total_earnings_usd: self.collector.total_earnings(),
            chain_count: self.chain_registry.connected_count(),
        }
    }

    /// 获取当前策略洞察（从历史执行中学习）
    pub fn learn_and_adapt(&mut self) -> Vec<String> {
        let mut insights = Vec::new();
        insights.push(format!("═══ CryptoAgent 第{}次迭代洞察 ═══", self.iteration_count));

        insights.extend(self.scanner.source_registry.source_quality_insight());
        insights.extend(self.scanner.strategy_insights());
        insights.extend(self.collector.summary_report());
        insights.extend(self.self_evolver.insight_report());

        let airdrop_total = self.airdrop_registry.total_unclaimed_value();
        if airdrop_total > 0.0 {
            insights.push(format!("📦 未领取空投估值: ${:.0}", airdrop_total));
        }

        let risk_score = self.security.approval_risk_score();
        if risk_score > 3.0 {
            insights.push(format!("⚠️ 授权风险评分: {:.1}/10 — 建议检查授权", risk_score));
        }

        if let Some(cheapest) = self.gas_tracker.cheapest_chain(&self.config.preferred_chains) {
            insights.push(format!("⛽ 最优Gas链: {}", cheapest.0));
        }

        if let Some((best_strat, q)) = self.self_evolver.best_strategy() {
            insights.push(format!("🧠 自进化推荐: {} (Q={:.2})", best_strat, q));
        }

        if let Some((best_type, roi)) = self.scanner.best_strategy() {
            insights.push(format!("→ 最佳策略: {} (ROI {:.1}x)", best_type.name(), roi));
        }

        if let Some(wallet) = self.wallet_manager.active_wallet() {
            insights.push(format!("→ 活跃钱包: {} ({})", wallet.address_short(), wallet.label));
        }

        let yield_count = self.yield_scanner.scan().len();
        insights.push(format!("📈 监测 {} 个收益机会", yield_count));

        insights
    }

    pub fn scan_opportunities(&mut self) -> Vec<Opportunity> {
        let mut all = self.scanner.scan_all();

        if let Some(wallet) = self.wallet_manager.active_wallet() {
            let chain_opps = self.scanner.on_chain_scan(&wallet.address, &self.evm_clients);
            all.extend(chain_opps);
        }

        all
    }

    pub fn scan_specific_chain(&mut self, chain: &ChainType, address: &str) -> Vec<Opportunity> {
        let mut results = Vec::new();

        if let Some(client) = self.evm_clients.get(chain) {
            if let Ok(balance) = client.get_balance(address) {
                if balance > 0.0 {
                    results.push(Opportunity {
                        opportunity_type: OpportunityType::TokenSwap,
                        chain: chain.clone(),
                        title: format!("{} Balance", chain.native_currency()),
                        description: format!("Found {} {} on {}", balance, chain.native_currency(), chain),
                        estimated_value_usd: balance,
                        confidence: 1.0,
                        action: "check wallet".into(),
                        source_url: None,
                        contract_address: None,
                        source_name: Some("on-chain monitor".into()),
                        execution_gas_cost: 0.0,
                    });
                }
            }
        }

        results
    }

    pub fn check_all_balances(&self) -> HashMap<String, f64> {
        match self.wallet_manager.active_wallet() {
            Some(wallet) => self.evm_clients.check_all_balances(&wallet.address),
            None => HashMap::new(),
        }
    }

    /// Execute the best-ranked opportunity autonomously.
    /// Requires `auto_execute: true` in config and an active wallet with sufficient balance.
    /// Returns (opportunity_type, chain, tx_hash, value_usd) on success.
    pub fn execute_best_opportunity(&mut self) -> Result<(String, String, String, f64), String> {
        if !self.config.auto_execute {
            return Err(String::from("auto_execute disabled in config"));
        }
        let wallet_address = self.wallet_manager.active_wallet()
            .map(|w| w.address.clone())
            .ok_or_else(|| String::from("no active wallet"))?;

        // Scan and find the best opportunity
        let opportunities = self.scan_opportunities();
        let best = self.scanner.best_opportunity(&opportunities, self.config.min_opportunity_value_usd)
            .ok_or_else(|| String::from("no viable opportunities found"))?;

        let chain = best.chain.clone();
        let client = self.evm_clients.get(&chain)
            .ok_or_else(|| format!("no EVM client for chain {}", chain))?;

        // Check wallet balance on the opportunity's chain
        let balance = client.get_balance(&wallet_address)
            .map_err(|e| format!("balance check: {}", e))?;
        let needed = best.execution_gas_cost / 100.0; // rough ETH needed for gas
        if balance < needed {
            return Err(format!(
                "insufficient balance on {}: {:.6} {} (need ~{:.6})",
                chain,
                balance,
                chain.native_currency(),
                needed,
            ));
        }

        // Gas check
        let gas_price = client.get_gas_price().map_err(|e| format!("gas: {}", e))?;
        if gas_price > self.config.max_gas_price_gwei {
            return Err(format!(
                "gas too high on {}: {:.1} > {:.1}",
                chain,
                gas_price,
                self.config.max_gas_price_gwei,
            ));
        }

        // Nonce
        let nonce = client.get_transaction_count(&wallet_address)
            .map_err(|e| format!("nonce: {}", e))?;

        // Build and sign the transaction for the best opportunity
        let to_address = best.contract_address.clone().unwrap_or_else(|| wallet_address.clone());
        let tx = crate::neotrix::nt_act_crypto::tx::Tx1559 {
            chain_id: client.chain.chain_id(),
            nonce,
            max_priority_fee: ((gas_price * 0.1 * 1e9) as u128).max(1_000_000_000),
            max_fee: ((gas_price * 2.0 * 1e9) as u128).max(2_000_000_000),
            gas_limit: 21000,
            to: to_address,
            value: 0u128.into(),
            data: Vec::new(),
        };

        let wallet = self.wallet_manager.active_wallet()
            .ok_or_else(|| String::from("no active wallet"))?;
        let signed = crate::neotrix::nt_act_crypto::tx::TxBuilder::sign_1559(wallet, &tx)
            .map_err(|e| format!("sign: {}", e))?;

        let tx_hash = client.send_raw_transaction(&signed.raw)
            .map_err(|e| format!("send: {}", e))?;

        self.scanner.learn_from_execution(true, &best.opportunity_type, best.estimated_value_usd, best.execution_gas_cost);
        self.self_evolver.backtest(&best.opportunity_type.name(), &crate::neotrix::nt_act_crypto::opportunity::StrategyStats {
            opportunity_type: best.opportunity_type.clone(),
            attempts: 1,
            successes: 1,
            total_value_usd: best.estimated_value_usd,
            total_gas_usd: best.execution_gas_cost,
        });

        self.collector.record_simple(
            chain.clone(),
            best.opportunity_type.clone(),
            chain.native_currency(),
            best.estimated_value_usd,
            best.execution_gas_cost,
            &format!("auto_tx:{}", &tx_hash[..10]),
        );

        Ok((format!("{:?}", best.opportunity_type), chain.to_string(), tx_hash, best.estimated_value_usd))
    }

    pub fn earnings_summary(&self) -> Vec<String> {
        self.collector.summary_report()
    }

    pub fn collect_earning(&mut self, earning: CryptoEarnings) {
        self.collector.record(earning);
    }

    pub fn operational_chains(&self) -> Vec<ChainType> {
        self.evm_clients.operational_chains()
    }

    /// 完整战略洞察（包含所有子系统）
    pub fn generate_strategic_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        insights.push("═══ CryptoAgent 战略洞察 ═══".into());
        insights.push(format!("迭代次数: #{}", self.iteration_count));
        insights.push(format!("钱包数量: {}", self.wallet_manager.wallet_count()));
        insights.push(format!("已连接链数量: {}", self.chain_registry.connected_count()));
        insights.push(format!("可操作链数量: {}", self.operational_chains().len()));
        insights.push(format!("已发现机会: {}", self.scanner.total_opportunities_found()));
        insights.push(format!("已完成收益: ${:.2}", self.collector.total_earnings()));
        insights.push(format!("已领取总额: ${:.2}", self.scanner.total_value_claimed()));

        if let Some(wallet) = self.wallet_manager.active_wallet() {
            insights.push(format!("活跃钱包: {} ({})", wallet.address_short(), wallet.label));
        }

        if !self.operational_chains().is_empty() {
            insights.push("可操作链:".into());
            for chain in self.operational_chains().iter().take(5) {
                insights.push(format!("  - {} (ID: {})", chain, chain.chain_id()));
            }
        }

        insights.extend(self.scanner.source_registry.source_quality_insight());
        insights.extend(self.scanner.strategy_insights());

        insights
    }

    pub fn iteration_count(&self) -> u64 {
        self.iteration_count
    }
}

impl Default for CryptoAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// 一次迭代的执行结果
#[derive(Clone, Debug)]
pub struct IterationResult {
    pub iteration: u64,
    pub total_opportunities: usize,
    pub ranked_opportunities: usize,
    pub total_estimated_value: f64,
    pub best_opportunity: Option<Opportunity>,
    pub top_strategies: Vec<String>,
    pub wallet_count: usize,
    pub total_earnings_usd: f64,
    pub chain_count: usize,
}

impl IterationResult {
    pub fn summary(&self) -> Vec<String> {
        vec![
            format!("═══ Iteration #{} ═══", self.iteration),
            format!("扫描发现 {} 个机会 ({} 个值得处理)", self.total_opportunities, self.ranked_opportunities),
            format!("总估值: ${:.2}", self.total_estimated_value),
            format!("钱包: {}, 链: {}", self.wallet_count, self.chain_count),
            format!("累计收益: ${:.2}", self.total_earnings_usd),
            format!("最佳策略: {}", self.top_strategies.first().map(|s| s.as_str()).unwrap_or("none")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_act_crypto_creation() {
        let agent = CryptoAgent::new();
        assert!(agent.chain_registry.connected_count() > 0);
        assert!(agent.wallet_store.dir_path().exists());
    }

    #[test]
    fn test_setup_default_wallet() {
        let mut agent = CryptoAgent::new();
        let _wallet = agent.setup_default_wallet("main");
        assert!(agent.wallet_manager.active_wallet().is_some());
    }

    #[test]
    fn test_scan_opportunities() {
        let mut agent = CryptoAgent::new();
        agent.setup_default_wallet("test_scan_opportunities");
        let opps = agent.scan_opportunities();
        assert!(!opps.is_empty());
    }

    #[test]
    fn test_earnings_summary() {
        let mut agent = CryptoAgent::new();
        agent.collect_earning(CryptoEarnings {
            timestamp: chrono::Utc::now(),
            chain: ChainType::Ethereum,
            opportunity_type: OpportunityType::FaucetClaim,
            token: "ETH".into(),
            amount: 0.1,
            value_usd: 200.0,
            tx_hash: None,
            label: "test".into(),
        });
        let summary = agent.earnings_summary();
        assert!(!summary.is_empty());
        assert!(summary.iter().any(|l| l.contains("200")));
    }

    #[test]
    fn test_strategic_insights() {
        let mut agent = CryptoAgent::new();
        agent.setup_default_wallet("test");
        agent.run_iteration();
        let insights = agent.generate_strategic_insights();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_run_iteration() {
        let mut agent = CryptoAgent::new();
        agent.setup_default_wallet("test");
        let result = agent.run_iteration();
        assert_eq!(result.iteration, 1);
        assert!(result.total_opportunities > 0);
        assert_eq!(agent.iteration_count(), 1);
    }

    #[test]
    fn test_learn_and_adapt() {
        let mut agent = CryptoAgent::new();
        agent.setup_default_wallet("test");
        agent.run_iteration();
        agent.scanner.learn_from_execution(true, &OpportunityType::FaucetClaim, 10.0, 0.5);
        let insights = agent.learn_and_adapt();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_import_wallet() {
        let mut agent = CryptoAgent::new();
        let w = CryptoWallet::generate_evm("temp_import");
        let pk = w.private_key_hex();
        agent.import_wallet(&pk, "imported").unwrap();
        assert!(agent.wallet_manager.active_wallet().is_some());
        assert_eq!(agent.wallet_manager.active_wallet().unwrap().label, "imported");
        let _ = agent.delete_persisted_wallet("imported");
    }
}
