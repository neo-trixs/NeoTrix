//! Censys 互联网设备搜索引擎 — 实现 OsintSource trait，通过 run_osint() 门控分发调用。

use super::{OsintConfig, OsintTarget};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const CENSYS_API_BASE: &str = "https://search.censys.io/api/v2";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CensysFindings {
    pub ip: String,
    pub services: Vec<_CensysService>,
    pub protocols: Vec<String>,
    pub country: Option<String>,
    pub autonomous_system: Option<String>,
}

impl std::fmt::Display for CensysFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "══ Censys Host Discovery ═════════════════════════")?;
        writeln!(f, "  IP:        {}", self.ip)?;
        writeln!(f, "  Services:  {}", self.services.len())?;
        writeln!(f, "  Protocols: {:?}", self.protocols)?;
        if let Some(ref c) = self.country {
            writeln!(f, "  Country:   {}", c)?;
        }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct _CensysService {
    pub port: u16,
    pub service_name: String,
    pub transport_protocol: String,
    pub banner: Option<String>,
}

pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<CensysFindings, String> {
    let api_id = config
        .api_keys
        .get("censys_id")
        .ok_or("Censys API ID 未配置")?;
    let api_secret = config
        .api_keys
        .get("censys_secret")
        .ok_or("Censys API Secret 未配置")?;

    let ip = target
        .ip
        .as_ref()
        .or(target.domain.as_ref())
        .ok_or("Censys 需要 ip 或 domain")?;

    let url = format!("{}/hosts/{}", CENSYS_API_BASE, ip);

    let resp = client
        .get(&url)
        .basic_auth(api_id, Some(api_secret))
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("Censys API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Censys API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Censys 响应解析失败: {e}"))?;

    let result = body.get("result").unwrap_or(&body);

    let services: Vec<_CensysService> = result
        .get("services")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| {
                    Some(_CensysService {
                        port: s.get("port")?.as_u64()? as u16,
                        service_name: s
                            .get("service_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        transport_protocol: s
                            .get("transport_protocol")
                            .and_then(|v| v.as_str())
                            .unwrap_or("TCP")
                            .to_string(),
                        banner: s
                            .get("banner")
                            .and_then(|v| v.as_str())
                            .map(|s| s.chars().take(200).collect()),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let protocols: Vec<String> = result
        .get("protocols")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|p| p.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let country = result
        .get("location")
        .and_then(|l| l.get("country"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let autonomous_system = result
        .get("autonomous_system")
        .and_then(|a| a.get("asn"))
        .and_then(|v| v.as_u64())
        .map(|a| format!("AS{}", a));

    Ok(CensysFindings {
        ip: ip.clone(),
        services,
        protocols,
        country,
        autonomous_system,
    })
}

pub struct _CensysInvestigator;

pub const CENSYS_API_HOST: &str = "search.censys.io";
pub fn _censys_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CENSYS_API_HOST, "443")
}
pub fn _censys_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![_censys_egress_rule()], false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_censys_egress() {
        assert!(_censys_egress_policy().check("search.censys.io", 443));
        assert!(!_censys_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_censys_findings_display() {
        let findings = super::CensysFindings {
            ip: "93.184.216.34".into(),
            services: vec![super::_CensysService {
                port: 443,
                service_name: "HTTPS".into(),
                transport_protocol: "TCP".into(),
                banner: None,
            }],
            protocols: vec!["TCP".into()],
            country: Some("US".into()),
            autonomous_system: Some("AS15169".into()),
        };
        let display = format!("{}", findings);
        assert!(display.contains("93.184.216.34"));
        assert!(display.contains("Censys"));
    }

    #[test]
    fn test_censys_findings_default() {
        let findings = super::CensysFindings::default();
        assert!(findings.services.is_empty());
    }
}
