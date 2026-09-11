//! CryptoPub 加密货币地址情报 — 实现 OsintSource trait

use reqwest::Client;
use serde::{Deserialize, Serialize};
use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CryptoPubFindings {
    pub address: String,
    pub chain: String,
    pub balance: String,
    pub tx_count: u64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

impl std::fmt::Display for CryptoPubFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "══ CryptoPub Address ═════════════════════════")?;
        writeln!(f, "  Address:  {}", self.address)?;
        writeln!(f, "  Chain:    {}", self.chain)?;
        writeln!(f, "  Balance:  {}", self.balance)?;
        writeln!(f, "  Txs:      {}", self.tx_count)?;
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<CryptoPubFindings, String> {
    let address = target.url.as_ref()
        .or(target.domain.as_ref())
        .ok_or("CryptoPub 需要地址或域名")?;

    // 检测链类型
    let chain = if address.starts_with("0x") { "ethereum" }
        else if address.starts_with("bc1") || address.starts_with("1") || address.starts_with("3") { "bitcoin" }
        else { "unknown" };

    Ok(CryptoPubFindings {
        address: address.clone(),
        chain: chain.into(),
        balance: "0".into(),
        tx_count: 0,
        first_seen: None,
        last_seen: None,
    })
}

pub struct CryptoPubInvestigator;

impl super::OsintSource for CryptoPubInvestigator {
    type Findings = CryptoPubFindings;
    fn name(&self) -> &'static str { "cryptopub" }
    fn priority(&self) -> u8 { 6 }

    async fn investigate(
        &self,
        target: &super::OsintTarget,
        client: &Client,
        config: &super::OsintConfig,
    ) -> Result<CryptoPubFindings, String> {
        investigate(target, client, config).await
    }
}
