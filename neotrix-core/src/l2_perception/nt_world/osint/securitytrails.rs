//! SecurityTrails 域名情报搜索引擎 — 实现 OsintSource trait，通过 run_osint() 门控分发调用。

use super::{OsintConfig, OsintTarget};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// SecurityTrails API 基础 URL
pub const SECURITYTRAILS_API_BASE: &str = "https://api.securitytrails.com/v1";

/// SecurityTrails 调查结果
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SecurityTrailsFindings {
    pub domain: String,
    pub subdomains: Vec<_SecurityTrailsSubdomain>,
    pub records: Vec<_SecurityTrailsDnsRecord>,
    pub whois: Option<_SecurityTrailsWhois>,
}

impl std::fmt::Display for SecurityTrailsFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "══ SecurityTrails Domain Intelligence ═══════════════")?;
        writeln!(f, "  Domain:      {}", self.domain)?;
        writeln!(f, "  Subdomains:  {}", self.subdomains.len())?;
        writeln!(f, "  DNS Records: {}", self.records.len())?;
        if let Some(ref whois) = self.whois {
            writeln!(f, "  Registrar:   {}", whois.registrar.as_deref().unwrap_or("unknown"))?;
        }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct _SecurityTrailsSubdomain {
    pub name: String,
    pub ip: Option<String>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct _SecurityTrailsDnsRecord {
    pub record_type: String,
    pub value: String,
    pub ttl: Option<u32>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct _SecurityTrailsWhois {
    pub registrar: Option<String>,
    pub registration_date: Option<String>,
    pub expiration_date: Option<String>,
    pub nameservers: Vec<String>,
}

/// SecurityTrails 调查函数
pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<SecurityTrailsFindings, String> {
    let api_key = config
        .api_keys
        .get("securitytrails")
        .ok_or("SecurityTrails API key 未配置")?;

    let domain = target
        .domain
        .as_ref()
        .ok_or("SecurityTrails 需要 domain")?;

    // 1. 获取子域名
    let subdomains = fetch_subdomains(client, config, domain, api_key).await.unwrap_or_default();
    
    // 2. 获取DNS记录
    let records = fetch_dns_records(client, config, domain, api_key).await.unwrap_or_default();
    
    // 3. 获取WHOIS信息（可选）
    let whois = fetch_whois(client, config, domain, api_key).await.ok().flatten();

    Ok(SecurityTrailsFindings {
        domain: domain.clone(),
        subdomains,
        records,
        whois,
    })
}

async fn fetch_subdomains(
    client: &Client,
    config: &OsintConfig,
    domain: &str,
    api_key: &str,
) -> Result<Vec<_SecurityTrailsSubdomain>, String> {
    let url = format!("{}/domain/{}/subdomains?apikey={}", SECURITYTRAILS_API_BASE, domain, api_key);
    
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("SecurityTrails subdomains API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("SecurityTrails subdomains API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("SecurityTrails subdomains 响应解析失败: {e}"))?;

    let subdomains: Vec<_SecurityTrailsSubdomain> = body
        .get("subdomains")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| {
                    let name = s.as_str()?;
                    Some(_SecurityTrailsSubdomain {
                        name: format!("{}.{}", name, domain),
                        ip: None,
                        first_seen: None,
                        last_seen: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(subdomains)
}

async fn fetch_dns_records(
    client: &Client,
    config: &OsintConfig,
    domain: &str,
    api_key: &str,
) -> Result<Vec<_SecurityTrailsDnsRecord>, String> {
    let url = format!("{}/domain/{}/dns?apikey={}", SECURITYTRAILS_API_BASE, domain, api_key);
    
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("SecurityTrails DNS API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("SecurityTrails DNS API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("SecurityTrails DNS 响应解析失败: {e}"))?;

    let mut records = Vec::new();
    
    // 解析各种DNS记录类型
    for record_type in &["a", "aaaa", "cname", "mx", "ns", "txt", "soa"] {
        if let Some(record_array) = body.get(record_type).and_then(|v| v.as_array()) {
            for record in record_array {
                if let Some(value) = record.as_str() {
                    records.push(_SecurityTrailsDnsRecord {
                        record_type: record_type.to_uppercase(),
                        value: value.to_string(),
                        ttl: None,
                        first_seen: None,
                        last_seen: None,
                    });
                }
            }
        }
    }

    Ok(records)
}

async fn fetch_whois(
    client: &Client,
    config: &OsintConfig,
    domain: &str,
    api_key: &str,
) -> Result<Option<_SecurityTrailsWhois>, String> {
    let url = format!("{}/domain/{}/whois?apikey={}", SECURITYTRAILS_API_BASE, domain, api_key);
    
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("SecurityTrails WHOIS API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("SecurityTrails WHOIS API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("SecurityTrails WHOIS 响应解析失败: {e}"))?;

    let whois = _SecurityTrailsWhois {
        registrar: body.get("registrar").and_then(|v| v.as_str()).map(|s| s.to_string()),
        registration_date: body.get("created_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        expiration_date: body.get("expires_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        nameservers: body.get("nameservers")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|n| n.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
    };

    Ok(Some(whois))
}

// ═══════════════════════════════════════════════════════════════
// OsintSource trait 实现
// ═══════════════════════════════════════════════════════════════

pub struct _SecurityTrailsInvestigator;

pub const SECURITYTRAILS_API_HOST: &str = "api.securitytrails.com";
pub fn _securitytrails_egress_rule() -> super::super::l1_facade::EgressRule {
    super::super::l1_facade::EgressRule::allow(SECURITYTRAILS_API_HOST, "443")
}

pub fn _securitytrails_egress_policy() -> super::super::l1_facade::EgressPolicy {
    super::super::l1_facade::EgressPolicy::new(vec![_securitytrails_egress_rule()], false)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_securitytrails_egress() {
        assert!(_securitytrails_egress_policy().check("api.securitytrails.com", 443));
        assert!(!_securitytrails_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_securitytrails_findings_display() {
        let findings = super::SecurityTrailsFindings {
            domain: "example.com".into(),
            subdomains: vec![super::_SecurityTrailsSubdomain {
                name: "www.example.com".into(),
                ip: Some("93.184.216.34".into()),
                first_seen: Some("2020-01-01".into()),
                last_seen: Some("2024-01-01".into()),
            }],
            records: vec![super::_SecurityTrailsDnsRecord {
                record_type: "A".into(),
                value: "93.184.216.34".into(),
                ttl: Some(3600),
                first_seen: None,
                last_seen: None,
            }],
            whois: Some(super::_SecurityTrailsWhois {
                registrar: Some("Example Registrar".into()),
                registration_date: Some("2000-01-01".into()),
                expiration_date: Some("2025-01-01".into()),
                nameservers: vec!["ns1.example.com".into()],
            }),
        };
        let display = format!("{}", findings);
        assert!(display.contains("example.com"));
        assert!(display.contains("SecurityTrails"));
    }

    #[test]
    fn test_securitytrails_findings_default() {
        let findings = super::SecurityTrailsFindings::default();
        assert!(findings.subdomains.is_empty());
        assert!(findings.records.is_empty());
    }
}
