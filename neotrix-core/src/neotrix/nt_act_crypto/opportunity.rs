use super::chain::ChainType;
use super::evm::MultiEvmClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_KNOWN_ITEMS: usize = 500;
const MAX_SCAN_HISTORY: usize = 100;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OpportunityType {
    FaucetClaim,
    AirdropClaim,
    Arbitrage,
    YieldFarming,
    CrossChainTransfer,
    Bounty,
    TestnetFaucet,
    StakingReward,
    TokenSwap,
    LiquidityProvision,
    Unknown,
}

impl OpportunityType {
    pub fn name(&self) -> &str {
        match self {
            OpportunityType::FaucetClaim => "faucet claim",
            OpportunityType::AirdropClaim => "airdrop claim",
            OpportunityType::Arbitrage => "arbitrage",
            OpportunityType::YieldFarming => "yield farming",
            OpportunityType::CrossChainTransfer => "cross-chain transfer",
            OpportunityType::Bounty => "bounty",
            OpportunityType::TestnetFaucet => "testnet faucet",
            OpportunityType::StakingReward => "staking reward",
            OpportunityType::TokenSwap => "token swap",
            OpportunityType::LiquidityProvision => "liquidity provision",
            OpportunityType::Unknown => "unknown",
        }
    }

    pub fn risk_level(&self) -> f64 {
        match self {
            OpportunityType::FaucetClaim => 0.05,
            OpportunityType::TestnetFaucet => 0.0,
            OpportunityType::StakingReward => 0.1,
            OpportunityType::AirdropClaim => 0.15,
            OpportunityType::Bounty => 0.2,
            OpportunityType::TokenSwap => 0.25,
            OpportunityType::LiquidityProvision => 0.3,
            OpportunityType::YieldFarming => 0.4,
            OpportunityType::CrossChainTransfer => 0.5,
            OpportunityType::Arbitrage => 0.6,
            OpportunityType::Unknown => 0.9,
        }
    }

    pub fn expected_effort(&self) -> &str {
        match self {
            OpportunityType::FaucetClaim => "low (1-5 min)",
            OpportunityType::TestnetFaucet => "low (1 min)",
            OpportunityType::StakingReward => "medium (10-30 min)",
            OpportunityType::AirdropClaim => "medium (15-60 min)",
            OpportunityType::Bounty => "high (1-8 hours)",
            OpportunityType::TokenSwap => "low (2-5 min)",
            OpportunityType::LiquidityProvision => "medium (10-20 min)",
            OpportunityType::YieldFarming => "high (30-120 min)",
            OpportunityType::CrossChainTransfer => "medium (10-30 min)",
            OpportunityType::Arbitrage => "high (requires monitoring)",
            OpportunityType::Unknown => "unknown",
        }
    }
}

/// Information source — tracks which sources produce valuable opportunities
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InformationSource {
    pub name: String,
    pub source_type: OpportunitySourceType,
    pub url: String,
    pub trust_score: f64,
    pub successes: u32,
    pub failures: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OpportunitySourceType {
    Twitter,
    Discord,
    Telegram,
    Website,
    OnChain,
    News,
    Forum,
    Other,
}

impl InformationSource {
    pub fn new(name: &str, source_type: OpportunitySourceType, url: &str) -> Self {
        Self {
            name: name.to_string(),
            source_type,
            url: url.to_string(),
            trust_score: 0.5,
            successes: 0,
            failures: 0,
        }
    }

    pub fn record_success(&mut self) {
        self.successes += 1;
        self.trust_score = (self.trust_score + 0.05).min(1.0);
    }

    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.trust_score = (self.trust_score - 0.02).max(0.0);
    }

    pub fn total_attempts(&self) -> u32 {
        self.successes + self.failures
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_attempts() == 0 {
            return 0.5;
        }
        self.successes as f64 / self.total_attempts() as f64
    }
}

/// Tracks information sources and their quality over time
#[derive(Clone, Debug)]
pub struct SourceRegistry {
    sources: Vec<InformationSource>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: vec![
                InformationSource::new(
                    "Etherscan",
                    OpportunitySourceType::OnChain,
                    "https://etherscan.io",
                ),
                InformationSource::new(
                    "DefiLlama",
                    OpportunitySourceType::Website,
                    "https://defillama.com",
                ),
                InformationSource::new(
                    "CoinGecko",
                    OpportunitySourceType::News,
                    "https://coingecko.com",
                ),
            ],
        }
    }

    pub fn add(&mut self, source: InformationSource) {
        self.sources.push(source);
    }

    pub fn record_success(&mut self, name: &str) {
        if let Some(s) = self.sources.iter_mut().find(|s| s.name == name) {
            s.record_success();
        }
    }

    pub fn record_failure(&mut self, name: &str) {
        if let Some(s) = self.sources.iter_mut().find(|s| s.name == name) {
            s.record_failure();
        }
    }

    pub fn best_sources(&self, limit: usize) -> Vec<&InformationSource> {
        let mut sorted: Vec<_> = self.sources.iter().collect();
        sorted.sort_by(|a, b| {
            b.trust_score
                .partial_cmp(&a.trust_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(limit).collect()
    }

    pub fn all_sources(&self) -> &[InformationSource] {
        &self.sources
    }

    pub fn source_quality_insight(&self) -> Vec<String> {
        let mut insights = Vec::new();
        insights.push("── 信息源质量分析 ──".into());
        for s in self.best_sources(5) {
            insights.push(format!(
                "  {} ({}): 信任度 {:.1}%, 成功率 {:.1}% ({}/{})",
                s.name,
                s.url,
                s.trust_score * 100.0,
                s.success_rate() * 100.0,
                s.successes,
                s.total_attempts(),
            ));
        }
        insights
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Opportunity {
    pub opportunity_type: OpportunityType,
    pub chain: ChainType,
    pub title: String,
    pub description: String,
    pub estimated_value_usd: f64,
    pub confidence: f64,
    pub action: String,
    pub source_url: Option<String>,
    pub contract_address: Option<String>,
    pub source_name: Option<String>,
    pub execution_gas_cost: f64,
}

impl Opportunity {
    pub fn net_value(&self) -> f64 {
        (self.estimated_value_usd * self.confidence) - self.execution_gas_cost
    }

    pub fn risk_adjusted_score(&self) -> f64 {
        let risk = self.opportunity_type.risk_level();
        let value = self.net_value().max(0.01);
        value * (1.0 - risk) * self.confidence
    }

    pub fn summary(&self) -> String {
        format!(
            "[{}] {} — ${:.2} est. | confidence {:.0}% | risk {:.0}% | score {:.2}",
            self.opportunity_type.name(),
            self.title,
            self.estimated_value_usd,
            self.confidence * 100.0,
            self.opportunity_type.risk_level() * 100.0,
            self.risk_adjusted_score(),
        )
    }

    pub fn effort_description(&self) -> String {
        format!(
            "{} — effort: {}",
            self.title,
            self.opportunity_type.expected_effort()
        )
    }

    pub fn is_worth_pursuing(&self, min_value: f64) -> bool {
        self.risk_adjusted_score() >= min_value
    }
}

#[derive(Clone, Debug)]
pub struct FaucetInfo {
    pub name: String,
    pub url: String,
    pub chain: ChainType,
    pub amount_per_claim: f64,
    pub cooldown_hours: u64,
    pub requires_social: bool,
}

#[derive(Clone, Debug)]
pub struct AirdropInfo {
    pub name: String,
    pub contract: String,
    pub chain: ChainType,
    pub estimated_value: f64,
    pub claim_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ScannedOpportunity {
    pub opportunity: Opportunity,
    pub scanned_at: i64,
    pub executed: bool,
    pub result: Option<String>,
}

/// Core scanner with information asymmetry and self-learning
#[derive(Clone, Debug)]
pub struct OpportunityScanner {
    known_faucets: Vec<FaucetInfo>,
    known_airdrops: Vec<AirdropInfo>,
    scan_history: Vec<ScannedOpportunity>,
    pub source_registry: SourceRegistry,
    strategy_performance: HashMap<String, StrategyStats>,
    total_value_claimed: f64,
    total_gas_spent: f64,
}

#[derive(Clone, Debug)]
pub struct StrategyStats {
    pub opportunity_type: OpportunityType,
    pub attempts: u32,
    pub successes: u32,
    pub total_value_usd: f64,
    pub total_gas_usd: f64,
}

impl StrategyStats {
    pub fn new(opportunity_type: OpportunityType) -> Self {
        Self {
            opportunity_type,
            attempts: 0,
            successes: 0,
            total_value_usd: 0.0,
            total_gas_usd: 0.0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        self.successes as f64 / self.attempts as f64
    }

    pub fn net_profit(&self) -> f64 {
        self.total_value_usd - self.total_gas_usd
    }

    pub fn roi(&self) -> f64 {
        if self.total_gas_usd == 0.0 {
            return self.net_profit();
        }
        self.net_profit() / self.total_gas_usd
    }
}

impl OpportunityScanner {
    pub fn new() -> Self {
        Self {
            known_faucets: vec![
                FaucetInfo {
                    name: "BSC Testnet Faucet".into(),
                    url: "https://testnet.binance.org/faucet-smart".into(),
                    chain: ChainType::Bsc,
                    amount_per_claim: 0.5,
                    cooldown_hours: 24,
                    requires_social: false,
                },
                FaucetInfo {
                    name: "Polygon Neon Faucet".into(),
                    url: "https://neonfaucet.org/".into(),
                    chain: ChainType::Polygon,
                    amount_per_claim: 0.1,
                    cooldown_hours: 24,
                    requires_social: true,
                },
                FaucetInfo {
                    name: "Arbitrum Goerli Faucet".into(),
                    url: "https://faucet.quicknode.com/arbitrum".into(),
                    chain: ChainType::Arbitrum,
                    amount_per_claim: 0.1,
                    cooldown_hours: 12,
                    requires_social: false,
                },
            ],
            known_airdrops: vec![
                AirdropInfo {
                    name: "Arbitrum Airdrop".into(),
                    contract: "0x912CE59144191C1204E64559FE8253a0e49E6548".into(),
                    chain: ChainType::Arbitrum,
                    estimated_value: 500.0,
                    claim_url: Some("https://arbitrum.foundation/claim".into()),
                },
                AirdropInfo {
                    name: "Optimism Airdrop".into(),
                    contract: "0x4200000000000000000000000000000000000042".into(),
                    chain: ChainType::Optimism,
                    estimated_value: 200.0,
                    claim_url: Some("https://app.optimism.io/airdrops".into()),
                },
            ],
            scan_history: Vec::new(),
            source_registry: SourceRegistry::new(),
            strategy_performance: HashMap::new(),
            total_value_claimed: 0.0,
            total_gas_spent: 0.0,
        }
    }

    pub fn scan_faucets(&self) -> Vec<Opportunity> {
        self.known_faucets
            .iter()
            .map(|f| {
                let gas = 0.001;
                Opportunity {
                    opportunity_type: OpportunityType::FaucetClaim,
                    chain: f.chain.clone(),
                    title: format!("{} Faucet", f.name),
                    description: format!(
                        "Claim {} {} from {} (cooldown: {}h, social: {})",
                        f.amount_per_claim,
                        f.chain.native_currency(),
                        f.name,
                        f.cooldown_hours,
                        if f.requires_social { "yes" } else { "no" },
                    ),
                    estimated_value_usd: f.amount_per_claim,
                    confidence: 0.9,
                    action: format!("visit {} and claim", f.url),
                    source_url: Some(f.url.clone()),
                    contract_address: None,
                    source_name: Some("faucet list".into()),
                    execution_gas_cost: gas,
                }
            })
            .collect()
    }

    pub fn scan_airdrops(&self) -> Vec<Opportunity> {
        self.known_airdrops
            .iter()
            .map(|a| {
                let gas = 5.0;
                Opportunity {
                    opportunity_type: OpportunityType::AirdropClaim,
                    chain: a.chain.clone(),
                    title: format!("{} Airdrop", a.name),
                    description: format!(
                        "Claim {} airdrop (estimated ${}, gas ~${:.2})",
                        a.name, a.estimated_value, gas
                    ),
                    estimated_value_usd: a.estimated_value,
                    confidence: 0.6,
                    action: a
                        .claim_url
                        .clone()
                        .unwrap_or_else(|| format!("interact with contract {}", a.contract)),
                    source_url: a.claim_url.clone(),
                    contract_address: Some(a.contract.clone()),
                    source_name: Some("airdrop tracker".into()),
                    execution_gas_cost: gas,
                }
            })
            .collect()
    }

    pub fn scan_all(&mut self) -> Vec<Opportunity> {
        let mut opportunities = Vec::new();
        opportunities.extend(self.scan_faucets());
        opportunities.extend(self.scan_airdrops());

        let now = chrono::Utc::now().timestamp();
        for opp in &opportunities {
            if self.scan_history.len() >= MAX_SCAN_HISTORY {
                self.scan_history.remove(0);
            }
            self.scan_history.push(ScannedOpportunity {
                opportunity: opp.clone(),
                scanned_at: now,
                executed: false,
                result: None,
            });
        }

        opportunities
    }

    pub fn on_chain_scan(&self, address: &str, evm_clients: &MultiEvmClient) -> Vec<Opportunity> {
        let mut results = Vec::new();

        for (_chain, client) in evm_clients.clients() {
            if let Ok(balance) = client.get_balance(address) {
                if balance > 0.0 {
                    results.push(Opportunity {
                        opportunity_type: OpportunityType::TokenSwap,
                        chain: client.chain.clone(),
                        title: format!("{} Balance Found", client.chain.native_currency()),
                        description: format!(
                            "{} {} on {}",
                            balance,
                            client.chain.native_currency(),
                            client.chain
                        ),
                        estimated_value_usd: balance,
                        confidence: 1.0,
                        action: "check wallet and consider swapping or transferring".into(),
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

    pub fn rank_opportunities(
        &self,
        opportunities: &[Opportunity],
        min_value: f64,
    ) -> Vec<Opportunity> {
        let mut ranked: Vec<_> = opportunities
            .iter()
            .filter(|o| o.is_worth_pursuing(min_value))
            .cloned()
            .collect();
        ranked.sort_by(|a, b| {
            b.risk_adjusted_score()
                .partial_cmp(&a.risk_adjusted_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }

    pub fn best_opportunity(
        &self,
        opportunities: &[Opportunity],
        min_value: f64,
    ) -> Option<Opportunity> {
        let ranked = self.rank_opportunities(opportunities, min_value);
        ranked.into_iter().next()
    }

    pub fn learn_from_execution(
        &mut self,
        success: bool,
        opp_type: &OpportunityType,
        value_usd: f64,
        gas_usd: f64,
    ) {
        let key = format!("{:?}", opp_type);
        let stats = self
            .strategy_performance
            .entry(key)
            .or_insert_with(|| StrategyStats::new(opp_type.clone()));

        stats.attempts += 1;
        if success {
            stats.successes += 1;
            stats.total_value_usd += value_usd;
            self.total_value_claimed += value_usd;
        }
        stats.total_gas_usd += gas_usd;
        self.total_gas_spent += gas_usd;
    }

    pub fn best_strategy(&self) -> Option<(OpportunityType, f64)> {
        let mut ranked: Vec<_> = self
            .strategy_performance
            .values()
            .map(|s| (s.opportunity_type.clone(), s.roi()))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.into_iter().next()
    }

    pub fn strategy_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();
        insights.push("── 策略表现分析 ──".into());
        for (_key, stats) in &self.strategy_performance {
            insights.push(format!(
                "  {}: {}/{} 成功 ({:.0}%), 净利润 ${:.2}, ROI {:.1}x)",
                stats.opportunity_type.name(),
                stats.successes,
                stats.attempts,
                stats.success_rate() * 100.0,
                stats.net_profit(),
                stats.roi(),
            ));
        }
        if self.strategy_performance.is_empty() {
            insights.push("  (暂无执行数据，开始扫描后自动学习)".into());
        }
        insights.push(format!(
            "  总收益: ${:.2}, 总Gas: ${:.2}",
            self.total_value_claimed, self.total_gas_spent
        ));
        insights
    }

    pub fn scan_history(&self) -> &[ScannedOpportunity] {
        &self.scan_history
    }

    pub fn mark_executed(&mut self, index: usize, result: String) {
        if index < self.scan_history.len() {
            self.scan_history[index].executed = true;
            self.scan_history[index].result = Some(result);
        }
    }

    pub fn total_opportunities_found(&self) -> usize {
        self.scan_history.len()
    }

    pub fn add_faucet(&mut self, faucet: FaucetInfo) {
        if self.known_faucets.len() >= MAX_KNOWN_ITEMS {
            self.known_faucets.remove(0);
        }
        self.known_faucets.push(faucet);
    }

    pub fn add_airdrop(&mut self, airdrop: AirdropInfo) {
        if self.known_airdrops.len() >= MAX_KNOWN_ITEMS {
            self.known_airdrops.remove(0);
        }
        self.known_airdrops.push(airdrop);
    }

    pub fn total_value_claimed(&self) -> f64 {
        self.total_value_claimed
    }

    pub fn total_gas_spent(&self) -> f64 {
        self.total_gas_spent
    }
}

impl Default for OpportunityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_faucets() {
        let scanner = OpportunityScanner::new();
        let faucets = scanner.scan_faucets();
        assert!(!faucets.is_empty());
        assert_eq!(faucets[0].opportunity_type, OpportunityType::FaucetClaim);
    }

    #[test]
    fn test_scan_airdrops() {
        let scanner = OpportunityScanner::new();
        let airdrops = scanner.scan_airdrops();
        assert!(!airdrops.is_empty());
    }

    #[test]
    fn test_scan_all() {
        let mut scanner = OpportunityScanner::new();
        let opps = scanner.scan_all();
        assert_eq!(opps.len(), scanner.total_opportunities_found());
    }

    #[test]
    fn test_mark_executed() {
        let mut scanner = OpportunityScanner::new();
        scanner.scan_all();
        scanner.mark_executed(0, "claimed".into());
        assert!(scanner.scan_history[0].executed);
    }

    #[test]
    fn test_risk_adjusted_score() {
        let opp = Opportunity {
            opportunity_type: OpportunityType::FaucetClaim,
            chain: ChainType::Ethereum,
            title: "test".into(),
            description: "test".into(),
            estimated_value_usd: 100.0,
            confidence: 0.9,
            action: "test".into(),
            source_url: None,
            contract_address: None,
            source_name: None,
            execution_gas_cost: 5.0,
        };
        assert!(opp.risk_adjusted_score() > 0.0);
        assert!(opp.is_worth_pursuing(1.0));
    }

    #[test]
    fn test_rank_opportunities() {
        let mut scanner = OpportunityScanner::new();
        let opps = scanner.scan_all();
        let ranked = scanner.rank_opportunities(&opps, 0.0);
        assert_eq!(ranked.len(), opps.len());
        if ranked.len() >= 2 {
            assert!(ranked[0].risk_adjusted_score() >= ranked[1].risk_adjusted_score());
        }
    }

    #[test]
    fn test_learn_from_execution() {
        let mut scanner = OpportunityScanner::new();
        scanner.learn_from_execution(true, &OpportunityType::FaucetClaim, 10.0, 0.5);
        scanner.learn_from_execution(false, &OpportunityType::FaucetClaim, 0.0, 0.5);
        assert!(scanner.best_strategy().is_some());
        let insights = scanner.strategy_insights();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_source_registry() {
        let mut registry = SourceRegistry::new();
        registry.record_success("Etherscan");
        registry.record_success("Etherscan");
        registry.record_failure("DefiLlama");
        let best = registry.best_sources(1);
        assert_eq!(best[0].name, "Etherscan");
        assert!((best[0].trust_score - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_opportunity_type_names() {
        assert_eq!(OpportunityType::FaucetClaim.name(), "faucet claim");
        assert_eq!(OpportunityType::AirdropClaim.name(), "airdrop claim");
    }

    #[test]
    fn test_risk_levels() {
        assert!(
            OpportunityType::FaucetClaim.risk_level() < OpportunityType::Arbitrage.risk_level()
        );
    }

    #[test]
    fn test_net_value() {
        let opp = Opportunity {
            opportunity_type: OpportunityType::FaucetClaim,
            chain: ChainType::Ethereum,
            title: "test".into(),
            description: "test".into(),
            estimated_value_usd: 100.0,
            confidence: 0.9,
            action: "test".into(),
            source_url: None,
            contract_address: None,
            source_name: None,
            execution_gas_cost: 10.0,
        };
        assert!((opp.net_value() - 80.0).abs() < 0.001);
    }
}
