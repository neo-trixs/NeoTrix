use super::chain::ChainType;
use super::opportunity::OpportunityType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CryptoEarnings {
    pub timestamp: DateTime<Utc>,
    pub chain: ChainType,
    pub opportunity_type: OpportunityType,
    pub token: String,
    pub amount: f64,
    pub value_usd: f64,
    pub tx_hash: Option<String>,
    pub label: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CryptoCollector {
    earnings: Vec<CryptoEarnings>,
}

impl CryptoCollector {
    pub fn new() -> Self {
        Self { earnings: Vec::new() }
    }

    pub fn record(&mut self, earning: CryptoEarnings) {
        self.earnings.push(earning);
    }

    pub fn record_simple(
        &mut self,
        chain: ChainType,
        opp_type: OpportunityType,
        token: &str,
        amount: f64,
        value_usd: f64,
        label: &str,
    ) {
        self.earnings.push(CryptoEarnings {
            timestamp: Utc::now(),
            chain,
            opportunity_type: opp_type,
            token: token.to_string(),
            amount,
            value_usd,
            tx_hash: None,
            label: label.to_string(),
        });
    }

    pub fn total_earnings(&self) -> f64 {
        self.earnings.iter().map(|e| e.value_usd).sum()
    }

    pub fn earnings_by_chain(&self) -> Vec<(ChainType, f64)> {
        let mut map: std::collections::HashMap<ChainType, f64> =
            std::collections::HashMap::new();
        for e in &self.earnings {
            *map.entry(e.chain.clone()).or_insert(0.0) += e.value_usd;
        }
        let mut result: Vec<_> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn earnings_by_type(&self) -> Vec<(OpportunityType, f64)> {
        let mut map: std::collections::HashMap<OpportunityType, f64> =
            std::collections::HashMap::new();
        for e in &self.earnings {
            *map.entry(e.opportunity_type.clone()).or_insert(0.0) += e.value_usd;
        }
        let mut result: Vec<_> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn recent_earnings(&self, limit: usize) -> Vec<&CryptoEarnings> {
        let mut sorted: Vec<_> = self.earnings.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.into_iter().take(limit).collect()
    }

    pub fn all_earnings(&self) -> &[CryptoEarnings] {
        &self.earnings
    }

    pub fn earning_count(&self) -> usize {
        self.earnings.len()
    }

    pub fn best_opportunity_type(&self) -> Option<OpportunityType> {
        self.earnings_by_type()
            .first()
            .map(|(t, _)| t.clone())
    }

    pub fn best_chain(&self) -> Option<ChainType> {
        self.earnings_by_chain()
            .first()
            .map(|(c, _)| c.clone())
    }

    pub fn summary_report(&self) -> Vec<String> {
        let mut report = Vec::new();
        report.push("═══ Crypto Earnings Report ═══".into());
        report.push(format!("Total earnings: ${:.2}", self.total_earnings()));
        report.push(format!("Total operations: {}", self.earnings.len()));

        report.push("── By Chain ──".into());
        for (chain, total) in self.earnings_by_chain() {
            report.push(format!("  {}: ${:.2}", chain, total));
        }

        report.push("── By Type ──".into());
        for (typ, total) in self.earnings_by_type() {
            report.push(format!("  {}: ${:.2}", typ.name(), total));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_total() {
        let mut collector = CryptoCollector::new();
        collector.record_simple(
            ChainType::Ethereum,
            OpportunityType::FaucetClaim,
            "ETH",
            0.1,
            200.0,
            "test faucet",
        );
        collector.record_simple(
            ChainType::Bsc,
            OpportunityType::AirdropClaim,
            "ARB",
            100.0,
            150.0,
            "test airdrop",
        );
        assert_eq!(collector.total_earnings(), 350.0);
        assert_eq!(collector.earning_count(), 2);
    }

    #[test]
    fn test_earnings_by_chain() {
        let mut collector = CryptoCollector::new();
        collector.record_simple(ChainType::Ethereum, OpportunityType::FaucetClaim, "ETH", 1.0, 2000.0, "");
        collector.record_simple(ChainType::Bsc, OpportunityType::AirdropClaim, "BNB", 1.0, 500.0, "");
        collector.record_simple(ChainType::Ethereum, OpportunityType::AirdropClaim, "ETH", 0.5, 1000.0, "");
        let by_chain = collector.earnings_by_chain();
        assert_eq!(by_chain.len(), 2);
        assert_eq!(by_chain[0].0, ChainType::Ethereum);
        assert_eq!(by_chain[0].1, 3000.0);
    }

    #[test]
    fn test_summary_report() {
        let mut collector = CryptoCollector::new();
        collector.record_simple(ChainType::Ethereum, OpportunityType::FaucetClaim, "ETH", 0.1, 200.0, "");
        let report = collector.summary_report();
        assert!(!report.is_empty());
        assert!(report.iter().any(|l| l.contains("200")));
    }

    #[test]
    fn test_empty_collector() {
        let collector = CryptoCollector::new();
        assert_eq!(collector.total_earnings(), 0.0);
    }
}
