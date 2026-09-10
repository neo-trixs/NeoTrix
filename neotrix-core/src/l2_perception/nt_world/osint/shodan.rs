//! Shodan 网络设备搜索引擎 — 实现 OsintSource trait，通过 run_osint() 门控分发调用。

use super::{OsintConfig, OsintTarget};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Shodan API 基础 URL
pub const SHODAN_API_BASE: &str = "https://api.shodan.io";

/// Shodan 调查结果
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ShodanFindings {
    pub host: String,
    pub ip: String,
    pub ports: Vec<u16>,
    pub services: Vec<ShodanService>,
    pub vulns: Vec<String>,
    pub os: Option<String>,
    pub org: Option<String>,
    pub isp: Option<String>,
}

impl std::fmt::Display for ShodanFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "══ Shodan Host Discovery ═════════════════════════")?;
        writeln!(f, "  Host:     {}", self.host)?;
        writeln!(f, "  IP:       {}", self.ip)?;
        writeln!(f, "  Ports:    {:?}", self.ports)?;
        writeln!(f, "  Services: {}", self.services.len())?;
        writeln!(f, "  Vulns:    {}", self.vulns.len())?;
        if let Some(ref os) = self.os {
            writeln!(f, "  OS:       {}", os)?;
        }
        if let Some(ref org) = self.org {
            writeln!(f, "  Org:      {}", org)?;
        }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ShodanService {
    pub port: u16,
    pub protocol: String,
    pub product: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
}

/// Shodan 调查函数
pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<ShodanFindings, String> {
    let api_key = config
        .api_keys
        .get("shodan")
        .ok_or("Shodan API key 未配置")?;

    let host = target
        .domain
        .as_ref()
        .or(target.ip.as_ref())
        .ok_or("Shodan 需要 domain 或 ip")?;

    let url = format!("{}/shodan/host/{}?key={}", SHODAN_API_BASE, host, api_key);

    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("Shodan API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Shodan API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Shodan 响应解析失败: {e}"))?;

    let ip = body
        .get("ip_str")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let ports: Vec<u16> = body
        .get("ports")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|p| p.as_u64())
                .map(|p| p as u16)
                .collect()
        })
        .unwrap_or_default();
    let os = body
        .get("os")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let org = body
        .get("org")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let isp = body
        .get("isp")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let services: Vec<ShodanService> = body
        .get("data")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| {
                    Some(ShodanService {
                        port: s.get("port")?.as_u64()? as u16,
                        protocol: s
                            .get("transport")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tcp")
                            .to_string(),
                        product: s
                            .get("product")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        version: s
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        banner: s
                            .get("data")
                            .and_then(|v| v.as_str())
                            .map(|s| s.chars().take(200).collect()),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let vulns: Vec<String> = body
        .get("vulns")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(ShodanFindings {
        host: host.clone(),
        ip,
        ports: ports.clone(),
        services,
        vulns,
        os,
        org,
        isp,
    })
}

// ═══════════════════════════════════════════════════════════════
// OsintSource trait 实现
// ═══════════════════════════════════════════════════════════════

pub struct ShodanInvestigator;

pub const SHODAN_API_HOST: &str = "api.shodan.io";
pub fn shodan_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SHODAN_API_HOST, "443")
}
pub fn shodan_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![shodan_egress_rule()], false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shodan_egress() {
        assert!(shodan_egress_policy().check("api.shodan.io", 443));
        assert!(!shodan_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_shodan_findings_display() {
        let findings = super::ShodanFindings {
            host: "example.com".into(),
            ip: "93.184.216.34".into(),
            ports: vec![80, 443],
            services: vec![super::ShodanService {
                port: 80,
                protocol: "tcp".into(),
                product: Some("nginx".into()),
                version: Some("1.19".into()),
                banner: None,
            }],
            vulns: vec!["CVE-2021-12345".into()],
            os: Some("Linux".into()),
            org: Some("Example Org".into()),
            isp: Some("Example ISP".into()),
        };
        let display = format!("{}", findings);
        assert!(display.contains("93.184.216.34"));
        assert!(display.contains("Shodan"));
    }

    #[test]
    fn test_shodan_findings_default() {
        let findings = super::ShodanFindings::default();
        assert!(findings.ports.is_empty());
        assert!(findings.services.is_empty());
    }
}
