use super::chain::ChainType;

#[derive(Clone, Debug)]
pub struct AirdropInfo {
    pub name: String,
    pub protocol: String,
    pub chain: ChainType,
    pub estimated_value_usd: f64,
    pub eligibility_criteria: Vec<String>,
    pub claim_url: Option<String>,
    pub claim_deadline: Option<i64>,
    pub is_claimed: bool,
    pub confidence: f64,
    pub source_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AirdropCheckResult {
    pub airdrop: AirdropInfo,
    pub is_eligible: bool,
    pub estimated_amount: f64,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct AirdropRegistry {
    airdrops: Vec<AirdropInfo>,
}

impl AirdropRegistry {
    pub fn new() -> Self {
        Self { airdrops: vec![] }
    }

    pub fn register_defaults(&mut self) {
        let defaults = vec![
            AirdropInfo {
                name: "Arbitrum".into(),
                protocol: "Arbitrum Foundation".into(),
                chain: ChainType::Arbitrum,
                estimated_value_usd: 1500.0,
                eligibility_criteria: vec![
                    "Bridged to Arbitrum before June 2023".into(),
                    "Used Arbitrum DEXes (Uniswap, Camelot)".into(),
                    "Active for 3+ months".into(),
                ],
                claim_url: Some("https://arbitrum.foundation/claim".into()),
                claim_deadline: Some(1700000000),
                is_claimed: false,
                confidence: 0.9,
                source_url: Some("https://arbitrum.io".into()),
            },
            AirdropInfo {
                name: "zkSync".into(),
                protocol: "zkSync Era".into(),
                chain: ChainType::ZkSync,
                estimated_value_usd: 2000.0,
                eligibility_criteria: vec![
                    "Used zkSync Era before Feb 2024".into(),
                    "Interacted with 5+ contracts".into(),
                    "Bridged >$10k volume".into(),
                ],
                claim_url: Some("https://claim.zksync.io".into()),
                claim_deadline: Some(1710000000),
                is_claimed: false,
                confidence: 0.75,
                source_url: Some("https://zksync.io".into()),
            },
            AirdropInfo {
                name: "LayerZero".into(),
                protocol: "LayerZero Labs".into(),
                chain: ChainType::Ethereum,
                estimated_value_usd: 3000.0,
                eligibility_criteria: vec![
                    "Used LayerZero bridges (Stargate, etc.)".into(),
                    "Sent 10+ cross-chain messages".into(),
                    "Used multiple chains".into(),
                ],
                claim_url: None,
                claim_deadline: None,
                is_claimed: false,
                confidence: 0.6,
                source_url: Some("https://layerzero.network".into()),
            },
            AirdropInfo {
                name: "Scroll".into(),
                protocol: "Scroll".into(),
                chain: ChainType::Scroll,
                estimated_value_usd: 1000.0,
                eligibility_criteria: vec![
                    "Used Scroll mainnet before 2024 Q3".into(),
                    "Interacted with Scroll ecosystem".into(),
                ],
                claim_url: None,
                claim_deadline: None,
                is_claimed: false,
                confidence: 0.5,
                source_url: Some("https://scroll.io".into()),
            },
            AirdropInfo {
                name: "Linea".into(),
                protocol: "Linea (Consensys)".into(),
                chain: ChainType::Linea,
                estimated_value_usd: 1500.0,
                eligibility_criteria: vec![
                    "Used Linea mainnet (Voyage)".into(),
                    "Completed Linea tasks/quests".into(),
                ],
                claim_url: None,
                claim_deadline: None,
                is_claimed: false,
                confidence: 0.45,
                source_url: Some("https://linea.build".into()),
            },
        ];
        self.airdrops.extend(defaults);
    }

    pub fn all(&self) -> &[AirdropInfo] {
        &self.airdrops
    }

    pub fn unclaimed(&self) -> Vec<&AirdropInfo> {
        self.airdrops.iter().filter(|a| !a.is_claimed).collect()
    }

    pub fn by_chain(&self, chain: &ChainType) -> Vec<&AirdropInfo> {
        self.airdrops.iter().filter(|a| a.chain == *chain).collect()
    }

    pub fn eligible_airdrops(
        &self,
        wallet_address: &str,
        _interactions: &[(ChainType, String)],
    ) -> Vec<AirdropCheckResult> {
        let _ = wallet_address;
        self.airdrops
            .iter()
            .map(|a| AirdropCheckResult {
                airdrop: a.clone(),
                is_eligible: a.confidence > 0.7,
                estimated_amount: a.estimated_value_usd,
                reason: if a.confidence > 0.7 {
                    format!("Likely eligible based on {} confidence", a.confidence)
                } else {
                    format!(
                        "Low confidence ({:.0}%), need more data",
                        a.confidence * 100.0
                    )
                },
            })
            .collect()
    }

    pub fn mark_claimed(&mut self, name: &str) {
        if let Some(airdrop) = self.airdrops.iter_mut().find(|a| a.name == name) {
            airdrop.is_claimed = true;
        }
    }

    pub fn add_airdrop(&mut self, airdrop: AirdropInfo) {
        if !self.airdrops.iter().any(|a| a.name == airdrop.name) {
            self.airdrops.push(airdrop);
        }
    }

    pub fn total_unclaimed_value(&self) -> f64 {
        self.airdrops
            .iter()
            .filter(|a| !a.is_claimed)
            .map(|a| a.estimated_value_usd)
            .sum()
    }
}

impl Default for AirdropRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AirdropClaimer;

impl AirdropClaimer {
    pub fn estimate_claim_cost(gas_limit: u64, gas_price_gwei: f64, native_price_usd: f64) -> f64 {
        (gas_limit as f64 * gas_price_gwei) / 1e9 * native_price_usd
    }

    pub fn is_worth_claiming(estimated_value: f64, claim_cost: f64, min_profit: f64) -> bool {
        estimated_value > claim_cost + min_profit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_defaults() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        assert_eq!(reg.all().len(), 5);
    }

    #[test]
    fn test_unclaimed_all_initially() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        assert_eq!(reg.unclaimed().len(), 5);
    }

    #[test]
    fn test_mark_claimed() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        reg.mark_claimed("Arbitrum");
        assert_eq!(reg.unclaimed().len(), 4);
    }

    #[test]
    fn test_by_chain() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        let arb = reg.by_chain(&ChainType::Arbitrum);
        assert_eq!(arb.len(), 1);
    }

    #[test]
    fn test_eligible_airdrops() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        let results = reg.eligible_airdrops("0x1234", &[]);
        let eligible: Vec<_> = results.iter().filter(|r| r.is_eligible).collect();
        assert_eq!(eligible.len(), 2);
        assert_eq!(eligible[0].airdrop.name, "Arbitrum");
    }

    #[test]
    fn test_estimate_claim_cost() {
        let cost = AirdropClaimer::estimate_claim_cost(150000, 50.0, 3000.0);
        assert!((cost - 22.5).abs() < 0.1);
    }

    #[test]
    fn test_is_worth_claiming() {
        assert!(AirdropClaimer::is_worth_claiming(100.0, 20.0, 10.0));
        assert!(!AirdropClaimer::is_worth_claiming(25.0, 20.0, 10.0));
    }

    #[test]
    fn test_add_airdrop() {
        let mut reg = AirdropRegistry::new();
        reg.add_airdrop(AirdropInfo {
            name: "Test Airdrop".into(),
            protocol: "Test".into(),
            chain: ChainType::Ethereum,
            estimated_value_usd: 500.0,
            eligibility_criteria: vec![],
            claim_url: None,
            claim_deadline: None,
            is_claimed: false,
            confidence: 0.5,
            source_url: None,
        });
        assert_eq!(reg.all().len(), 1);
    }

    #[test]
    fn test_total_unclaimed_value() {
        let mut reg = AirdropRegistry::new();
        reg.register_defaults();
        let total = reg.total_unclaimed_value();
        assert!((total - 9000.0).abs() < 0.01);
    }
}
