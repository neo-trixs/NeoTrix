//! ZoomEye 网络空间搜索引擎 — 实现 OsintSource trait，通过 run_osint() 门控分发调用。

use super::{OsintConfig, OsintTarget};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const ZOOMEYE_API_BASE: &str = "https://api.zoomeye.org";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ZoomEyeFindings {
    pub host: String,
    pub ip: String,
    pub ports: Vec<u16>,
    pub services: Vec<ZoomEyeService>,
    pub os: Option<String>,
    pub country: Option<String>,
}

impl std::fmt::Display for ZoomEyeFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "══ ZoomEye Host Discovery ═════════════════════════")?;
        writeln!(f, "  Host:     {}", self.host)?;
        writeln!(f, "  IP:       {}", self.ip)?;
        writeln!(f, "  Ports:    {:?}", self.ports)?;
        writeln!(f, "  Services: {}", self.services.len())?;
        if let Some(ref os) = self.os {
            writeln!(f, "  OS:       {}", os)?;
        }
        if let Some(ref c) = self.country {
            writeln!(f, "  Country:  {}", c)?;
        }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ZoomEyeService {
    pub port: u16,
    pub service: String,
    pub product: Option<String>,
    pub version: Option<String>,
}

pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<ZoomEyeFindings, String> {
    let api_key = config
        .api_keys
        .get("zoomeye")
        .ok_or("ZoomEye API key 未配置")?;

    let host = target
        .domain
        .as_ref()
        .or(target.ip.as_ref())
        .ok_or("ZoomEye 需要 domain 或 ip")?;

    let url = format!("{}/host/{}", ZOOMEYE_API_BASE, host);

    let resp = client
        .get(&url)
        .header("API-KEY", api_key)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("ZoomEye API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("ZoomEye API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("ZoomEye 响应解析失败: {e}"))?;

    let ip = body
        .get("ip")
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
    let country = body
        .get("country")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let services: Vec<ZoomEyeService> = body
        .get("services")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| {
                    Some(ZoomEyeService {
                        port: s.get("port")?.as_u64()? as u16,
                        service: s
                            .get("service")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        product: s
                            .get("product")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        version: s
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ZoomEyeFindings {
        host: host.clone(),
        ip,
        ports: ports.clone(),
        services,
        os,
        country,
    })
}

pub struct ZoomEyeInvestigator;

pub const ZOOMEYE_API_HOST: &str = "api.zoomeye.org";
pub fn zoomeye_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ZOOMEYE_API_HOST, "443")
}
pub fn zoomeye_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![zoomeye_egress_rule()], false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoomeye_egress() {
        assert!(zoomeye_egress_policy().check("api.zoomeye.org", 443));
        assert!(!zoomeye_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_zoomeye_findings_display() {
        let findings = super::ZoomEyeFindings {
            host: "example.com".into(),
            ip: "93.184.216.34".into(),
            ports: vec![22, 80],
            services: vec![super::ZoomEyeService {
                port: 22,
                service: "ssh".into(),
                product: Some("OpenSSH".into()),
                version: Some("8.2".into()),
            }],
            os: Some("Ubuntu 20.04".into()),
            country: Some("US".into()),
        };
        let display = format!("{}", findings);
        assert!(display.contains("93.184.216.34"));
        assert!(display.contains("ZoomEye"));
    }

    #[test]
    fn test_zoomeye_findings_default() {
        let findings = super::ZoomEyeFindings::default();
        assert!(findings.ports.is_empty());
    }
}
