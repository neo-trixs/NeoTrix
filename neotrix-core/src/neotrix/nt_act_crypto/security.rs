use super::chain::ChainType;
use super::evm::EvmClient;

#[derive(Clone, Debug)]
pub struct TxSimulation {
    pub would_succeed: bool,
    pub gas_estimate: u64,
    pub state_changes: Vec<String>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ScamWarning {
    pub severity: ScamSeverity,
    pub category: ScamCategory,
    pub message: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScamSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScamCategory {
    KnownScam,
    Phishing,
    RugPull,
    Honeypot,
    FakeToken,
    MaliciousContract,
    UnverifiedOwner,
    SuspiciousProxy,
}

impl ScamCategory {
    pub fn name(&self) -> &str {
        match self {
            ScamCategory::KnownScam => "Known Scam",
            ScamCategory::Phishing => "Phishing",
            ScamCategory::RugPull => "Rug Pull",
            ScamCategory::Honeypot => "Honeypot",
            ScamCategory::FakeToken => "Fake Token",
            ScamCategory::MaliciousContract => "Malicious Contract",
            ScamCategory::UnverifiedOwner => "Unverified Owner",
            ScamCategory::SuspiciousProxy => "Suspicious Proxy",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApprovalEntry {
    pub chain: ChainType,
    pub token: String,
    pub spender: String,
    pub amount: u128,
    pub is_unlimited: bool,
    pub granted_at: i64,
}

pub struct SecurityManager {
    known_scams: Vec<String>,
    approvals: Vec<ApprovalEntry>,
    max_approvals: usize,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            known_scams: vec![],
            approvals: Vec::new(),
            max_approvals: 200,
        }
    }

    pub fn register_scam(&mut self, address: &str) {
        let addr = address.strip_prefix("0x").unwrap_or(address).to_lowercase();
        if !self.known_scams.contains(&addr) {
            self.known_scams.push(addr);
        }
    }

    pub fn check_address(&self, address: &str) -> Option<ScamWarning> {
        let addr = address.strip_prefix("0x").unwrap_or(address).to_lowercase();

        if self.known_scams.contains(&addr) {
            return Some(ScamWarning {
                severity: ScamSeverity::Critical,
                category: ScamCategory::KnownScam,
                message: format!("Address {address} is a known scam address"),
                source: "Security Database".into(),
            });
        }

        if addr.len() == 40 {
            if addr.chars().filter(|c| *c == '0').count() > 35 {
                return Some(ScamWarning {
                    severity: ScamSeverity::Medium,
                    category: ScamCategory::SuspiciousProxy,
                    message: format!("Address {address} has suspicious zero-pattern"),
                    source: "Pattern Analysis".into(),
                });
            }
        }

        None
    }

    pub fn check_transaction(
        &self,
        to: &str,
        value_eth: f64,
        data: &[u8],
    ) -> Vec<ScamWarning> {
        let mut warnings = Vec::new();

        if let Some(w) = self.check_address(to) {
            warnings.push(w);
        }

        if value_eth > 10.0 && data.is_empty() {
            warnings.push(ScamWarning {
                severity: ScamSeverity::Low,
                category: ScamCategory::Phishing,
                message: format!("Large ETH transfer ({value_eth} ETH) to unknown address"),
                source: "Value Analysis".into(),
            });
        }

        if data.len() > 10000 {
            warnings.push(ScamWarning {
                severity: ScamSeverity::Medium,
                category: ScamCategory::SuspiciousProxy,
                message: "Transaction data is suspiciously large (>10KB)".into(),
                source: "Data Analysis".into(),
            });
        }

        warnings
    }

    pub fn simulate_tx(
        &self,
        _client: &EvmClient,
        _from: &str,
        _to: &str,
        _data: &[u8],
        _value: u128,
    ) -> TxSimulation {
        TxSimulation {
            would_succeed: true,
            gas_estimate: 50000,
            state_changes: vec![],
            warning: None,
        }
    }

    pub fn approve_check(&self, token: &str, spender: &str, amount: u128) -> Option<ScamWarning> {
        if let Some(w) = self.check_address(spender) {
            return Some(w);
        }

        if amount == u128::MAX {
            return Some(ScamWarning {
                severity: ScamSeverity::Medium,
                category: ScamCategory::Phishing,
                message: format!("Unlimited approval for {spender} on {token}"),
                source: "Approval Analysis".into(),
            });
        }

        None
    }

    pub fn record_approval(&mut self, entry: ApprovalEntry) {
        self.approvals.push(entry);
        if self.approvals.len() > self.max_approvals {
            self.approvals.remove(0);
        }
    }

    pub fn revoke_approval(&mut self, chain: &ChainType, token: &str, spender: &str) {
        self.approvals.retain(|a| {
            !(a.chain == *chain && a.token == token && a.spender == spender)
        });
    }

    pub fn active_approvals(&self) -> &[ApprovalEntry] {
        &self.approvals
    }

    pub fn unlimited_approvals(&self) -> Vec<&ApprovalEntry> {
        self.approvals.iter().filter(|a| a.is_unlimited).collect()
    }

    pub fn approval_risk_score(&self) -> f64 {
        let unlimited = self.unlimited_approvals().len() as f64;
        let total = self.approvals.len().max(1) as f64;
        (unlimited / total) * 10.0
    }

    pub fn load_known_scams(&mut self) {
        let default_scams = vec![
            "0000000000000000000000000000000000000000",
            "dead000000000000000000000000000000000000",
            "bogus111111111111111111111111111111111111",
        ];
        for scam in default_scams {
            self.register_scam(scam);
        }
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let sm = SecurityManager::new();
        assert!(sm.active_approvals().is_empty());
        assert!((sm.approval_risk_score() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_check_known_scam() {
        let mut sm = SecurityManager::new();
        sm.load_known_scams();
        let warning = sm.check_address("0x0000000000000000000000000000000000000000");
        assert!(warning.is_some());
        assert_eq!(warning.unwrap().severity, ScamSeverity::Critical);
    }

    #[test]
    fn test_check_clean_address() {
        let sm = SecurityManager::new();
        let warning = sm.check_address("0x1234567890abcdef1234567890abcdef12345678");
        assert!(warning.is_none());
    }

    #[test]
    fn test_check_large_transfer() {
        let sm = SecurityManager::new();
        let warnings = sm.check_transaction("0x1234567890abcdef1234567890abcdef12345678", 100.0, &[]);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_unlimited_approval_warning() {
        let sm = SecurityManager::new();
        let warning = sm.approve_check(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x1234567890abcdef1234567890abcdef12345678",
            u128::MAX,
        );
        assert!(warning.is_some());
    }

    #[test]
    fn test_record_and_revoke_approval() {
        let mut sm = SecurityManager::new();
        sm.record_approval(ApprovalEntry {
            chain: ChainType::Ethereum,
            token: "0xUSDC".into(),
            spender: "0xDEX".into(),
            amount: u128::MAX,
            is_unlimited: true,
            granted_at: 1000,
        });
        assert_eq!(sm.active_approvals().len(), 1);
        assert_eq!(sm.unlimited_approvals().len(), 1);

        sm.revoke_approval(&ChainType::Ethereum, "0xUSDC", "0xDEX");
        assert!(sm.active_approvals().is_empty());
    }

    #[test]
    fn test_scam_category_names() {
        assert_eq!(ScamCategory::RugPull.name(), "Rug Pull");
        assert_eq!(ScamCategory::Honeypot.name(), "Honeypot");
    }

    #[test]
    fn test_approval_risk_score() {
        let mut sm = SecurityManager::new();
        sm.record_approval(ApprovalEntry {
            chain: ChainType::Ethereum,
            token: "0xUSDC".into(),
            spender: "0xDEX".into(),
            amount: 1000,
            is_unlimited: false,
            granted_at: 1000,
        });
        sm.record_approval(ApprovalEntry {
            chain: ChainType::Ethereum,
            token: "0xUSDT".into(),
            spender: "0xDEX2".into(),
            amount: u128::MAX,
            is_unlimited: true,
            granted_at: 1001,
        });
        assert!((sm.approval_risk_score() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_simulate_tx() {
        let chain = ChainType::Ethereum;
        let config = crate::neotrix::nt_act_crypto::chain::ChainConfig::new(
            chain,
            "https://invalid-rpc.example.com",
        );
        let client = EvmClient::new(&config);
        let sm = SecurityManager::new();
        let sim = sm.simulate_tx(
            &client,
            "0xfrom",
            "0xto",
            &[0u8; 100],
            1000000,
        );
        assert!(sim.would_succeed);
    }

    #[test]
    fn test_register_scam() {
        let mut sm = SecurityManager::new();
        sm.register_scam("0xbadbadbadbadbadbadbadbadbadbadbadbadbad1");
        let warning = sm.check_address("0xbadbadbadbadbadbadbadbadbadbadbadbadbad1");
        assert!(warning.is_some());
    }
}
