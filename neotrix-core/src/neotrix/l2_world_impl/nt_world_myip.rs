#![forbid(unsafe_code)]

//! IP/网络探测工具 — 外部吸收批次 (jason5ng32/MyIP).
//!
//! - **数据源**: `jason5ng32/MyIP` — 一键获取本机/出口 IP、ASN、运营商、地理位置的轻量探测工具.
//! - **落点**: NT-WORLD 感知层 — 网络探测能力. 提供获取本机/目标 IP、ASN 等信息的 trait stub
//!   与基础结构, 作为 `nt_world_bgpview`/`nt_world_osint` 网络情报的探测入口 (R-P42 强化现有节点).
//! - **成熟度**: C1 — 单元测 + SelfTest 存在级. 真实探测接线推迟至 C2 (Egress + KB 接入).
//! - **Egress**: `nt_shield_sandbox::INTEL_MYIP_HOST` allow (deny-wins) — 仅探测类只读请求.

use crate::core::nt_core_self_test::SelfTest;

/// 网络探测结果 (对应 MyIP 输出字段的精简模型).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IpProbe {
    pub ip: String,
    #[serde(default)]
    pub asn: Option<String>,
    #[serde(default)]
    pub org: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

impl IpProbe {
    pub fn new(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
            asn: None,
            org: None,
            country: None,
            city: None,
        }
    }

    /// 轻量校验: IP 非空且为合法 IPv4/IPv6 形态 (仅格式检查, 不含端口).
    pub fn is_valid(&self) -> bool {
        if self.ip.is_empty() {
            return false;
        }
        is_ipv4(&self.ip) || is_ipv6(&self.ip)
    }
}

fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| {
        if p.is_empty() || p.len() > 3 {
            return false;
        }
        p.bytes().all(|b| b.is_ascii_digit()) && p.parse::<u8>().is_ok()
    })
}

fn is_ipv6(s: &str) -> bool {
    let compressed = s.contains("::");
    let parts: Vec<&str> = s.split(':').filter(|p| !p.is_empty()).collect();
    let well_formed = if compressed {
        parts.len() <= 7
    } else {
        parts.len() == 8
    };
    let groups_ok = parts
        .iter()
        .all(|p| p.len() <= 4 && p.bytes().all(|b| b.is_ascii_hexdigit()));
    well_formed && groups_ok
}

/// IP/网络探测接口 (stub — 真实采集由 C2 接线实现).
pub trait IpProbeProvider: Send + Sync {
    /// 探测本机/出口 IP.
    fn probe_self(&self) -> Result<IpProbe, String>;
    /// 探测指定目标 IP 的 ASN/地理信息.
    fn probe_target(&self, ip: &str) -> Result<IpProbe, String>;
}

/// 离线探测 stub — 返回 fixture, 不发起网络请求 (C1 阶段).
pub struct OfflineIpProbe;

impl IpProbeProvider for OfflineIpProbe {
    fn probe_self(&self) -> Result<IpProbe, String> {
        Ok(IpProbe::new("127.0.0.1"))
    }
    fn probe_target(&self, ip: &str) -> Result<IpProbe, String> {
        let mut p = IpProbe::new(ip);
        p.asn = Some("AS0".to_string());
        Ok(p)
    }
}

pub struct MyIpSelfTest;
impl SelfTest for MyIpSelfTest {
    fn name(&self) -> &str {
        "world:myip"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let probe = OfflineIpProbe;
        let self_p = probe.probe_self().map_err(|e| vec![e])?;
        if !self_p.is_valid() {
            return Err(vec!["myip self probe invalid".into()]);
        }
        let tgt = probe.probe_target("8.8.8.8").map_err(|e| vec![e])?;
        if !tgt.is_valid() || tgt.asn.is_none() {
            return Err(vec!["myip target probe invalid".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_probe_self_valid() {
        let p = OfflineIpProbe.probe_self().unwrap();
        assert!(p.is_valid());
    }

    #[test]
    fn ip_format_validation() {
        assert!(IpProbe::new("8.8.8.8").is_valid());
        assert!(IpProbe::new("2001:db8::1").is_valid());
        assert!(!IpProbe::new("not-an-ip").is_valid());
        assert!(!IpProbe::new("").is_valid());
        assert!(!IpProbe::new("999.1.1.1").is_valid());
    }

    #[test]
    fn target_probe_carries_asn() {
        let t = OfflineIpProbe.probe_target("1.1.1.1").unwrap();
        assert!(t.is_valid());
        assert_eq!(t.asn.as_deref(), Some("AS0"));
    }
}
