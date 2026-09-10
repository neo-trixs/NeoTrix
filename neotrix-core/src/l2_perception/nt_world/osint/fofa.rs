//! FOFA OSINT 模块 — 网络空间资产发现与指纹识别
//!
//! **FOFA** (Fingerprint of All) — 湖南华数信安科技
//! 三层架构: 多维侦察 → 指纹扫描 → 搜索引擎
//! 核心: 40亿+资产, 35万+指纹规则, API驱动
//!
//! 熔炼进 NT-WORLD OSINT 骨架:
//! - 作为 `network::investigate` 的补充数据源
//! - 作为 `sweep::SweepSource` 的扫描后端
//! - 作为 `backend_router::Backend` 的有序备选
//! - 为 `asset_map_capability` 提供真实 API 能力

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{OsintConfig, OsintTarget};

// ═══════════════════════════════════════════════════════════════
// FOFA 数据模型
// ═══════════════════════════════════════════════════════════════

/// FOFA API 搜索响应
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FofaResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub error_code: u64,
    #[serde(default)]
    pub errmsg: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub results: Vec<Vec<String>>,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub consumed_fpoints: u64,
    #[serde(default)]
    pub remaining_fpoints: u64,
}

/// FOFA 资产信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FofaAsset {
    pub host: String,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub product: String,
    pub version: String,
    pub os: String,
    pub platform: String,
    pub service: String,
    pub vuln: String,
    pub country: String,
    pub city: String,
    pub org: String,
    pub domain: String,
    pub isp: String,
}

/// FOFA 账号信息
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FofaAccountInfo {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub fcoin: u64,
    #[serde(default)]
    pub vip_level: u8,
    #[serde(default)]
    pub remaining_api_calls: u64,
}

/// FOFA OSINT 发现
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FofaFindings {
    /// 搜索到的资产列表
    pub assets: Vec<FofaAsset>,
    /// 匹配的资产总数
    pub total: usize,
    /// 查询消耗的 F 点
    pub consumed_fpoints: u64,
    /// 剩余 F 点
    pub remaining_fpoints: u64,
    /// 从资产中提取的开放端口
    pub open_ports: Vec<u16>,
    /// 从资产中提取的服务指纹
    pub service_fingerprints: Vec<ServiceFingerprint>,
    /// 发现的漏洞
    pub vulns: Vec<VulnInfo>,
    /// 查询的域名/IP
    pub target: String,
}

/// 服务指纹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFingerprint {
    pub product: String,
    pub version: String,
    pub protocol: String,
    pub port: u16,
    pub count: usize,
}

/// 漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnInfo {
    pub cve_id: String,
    pub affected_product: String,
    pub affected_ip: String,
    pub affected_port: u16,
}

impl std::fmt::Display for FofaFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── FOFA Asset Discovery ──")?;
        writeln!(f, "    Target:     {}", self.target)?;
        writeln!(f, "    Total:      {}", self.total)?;
        writeln!(f, "    Assets:     {}", self.assets.len())?;
        writeln!(f, "    Ports:      {:?}", self.open_ports)?;
        writeln!(f, "    Fingerprints: {}", self.service_fingerprints.len())?;
        writeln!(f, "    Vulns:      {}", self.vulns.len())?;
        writeln!(
            f,
            "    F-points:   {} consumed / {} remaining",
            self.consumed_fpoints, self.remaining_fpoints
        )?;
        for fp in &self.service_fingerprints {
            writeln!(
                f,
                "      {}/{}/{} (x{})",
                fp.product, fp.version, fp.protocol, fp.count
            )?;
        }
        for v in &self.vulns {
            writeln!(
                f,
                "      {} on {}/{}",
                v.cve_id, v.affected_ip, v.affected_port
            )?;
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// FOFA API 客户端 (异步, 对齐骨架 dns/http 模式)
// ═══════════════════════════════════════════════════════════════

/// FOFA API 客户端
pub struct FofaClient {
    api_key: String,
    api_base: String,
}

impl std::fmt::Debug for FofaClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FofaClient")
            .field(
                "api_key",
                &if self.api_key.len() > 8 {
                    format!("{}...****", &self.api_key[..4])
                } else {
                    "****".to_string()
                },
            )
            .field("api_base", &self.api_base)
            .finish()
    }
}

impl Default for FofaClient {
    fn default() -> Self {
        Self::from_env()
    }
}

impl FofaClient {
    /// 从环境变量创建 (NEOTRIX_FOFA_EMAIL + NEOTRIX_FOFA_KEY)
    pub fn from_env() -> Self {
        let email = std::env::var("NEOTRIX_FOFA_EMAIL").unwrap_or_default();
        let key = std::env::var("NEOTRIX_FOFA_KEY").unwrap_or_default();
        let api_key = if email.is_empty() || key.is_empty() {
            String::new()
        } else {
            format!("{}:{}", email, key)
        };
        Self {
            api_key,
            api_base: "https://fofa.info/api/v1".to_string(),
        }
    }

    /// 从 OsintConfig 创建
    pub fn from_config(config: &OsintConfig) -> Self {
        let api_key = config
            .api_keys
            .get("fofa")
            .cloned()
            .unwrap_or_else(|| Self::from_env().api_key);
        Self {
            api_key,
            api_base: "https://fofa.info/api/v1".to_string(),
        }
    }

    /// 是否已配置 API Key
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Base64 编码查询
    fn encode_query(query: &str) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(query.as_bytes())
    }

    /// 构建搜索 URL
    fn build_search_url(&self, query: &str, fields: &str, page: usize, size: usize) -> String {
        let encoded = Self::encode_query(query);
        format!(
            "{}/search/all?key={}&qbase64={}&fields={}&page={}&size={}",
            self.api_base,
            urlencoding::encode(&self.api_key),
            urlencoding::encode(&encoded),
            urlencoding::encode(fields),
            page,
            size
        )
    }

    /// 异步执行搜索 (对齐骨架 dns/http 的 async Client 模式)
    pub async fn search(
        &self,
        query: &str,
        fields: &str,
        page: usize,
        size: usize,
        client: &Client,
    ) -> Result<FofaResponse, String> {
        if !self.is_configured() {
            return Err("FOFA API key 未配置 (设置 NEOTRIX_FOFA_EMAIL + NEOTRIX_FOFA_KEY)".into());
        }
        let url = self.build_search_url(query, fields, page, size);
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("FOFA request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("FOFA HTTP {}", resp.status()));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| format!("FOFA read body: {}", e))?;
        let parsed: FofaResponse =
            serde_json::from_str(&text).map_err(|e| format!("FOFA parse failed: {}", e))?;
        if parsed.error_code != 0 {
            return Err(format!(
                "FOFA API error [{}]: {}",
                parsed.error_code, parsed.errmsg
            ));
        }
        Ok(parsed)
    }

    /// 异步获取账号信息
    pub async fn get_account_info(&self, client: &Client) -> Result<FofaAccountInfo, String> {
        if !self.is_configured() {
            return Err("FOFA API key 未配置".into());
        }
        let url = format!(
            "{}/info/my?key={}",
            self.api_base,
            urlencoding::encode(&self.api_key)
        );
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("FOFA account request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("FOFA account HTTP {}", resp.status()));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| format!("FOFA account read body: {}", e))?;
        let info: FofaAccountInfo =
            serde_json::from_str(&text).map_err(|e| format!("FOFA account parse failed: {}", e))?;
        Ok(info)
    }

    /// 解析搜索结果为标准化资产
    pub fn parse_assets(&self, resp: &FofaResponse) -> Vec<FofaAsset> {
        let field_names = &resp.fields;
        resp.results
            .iter()
            .map(|row| {
                let mut asset = FofaAsset::default();
                for (i, field_name) in field_names.iter().enumerate() {
                    if let Some(value) = row.get(i) {
                        match field_name.as_str() {
                            "host" => asset.host = value.clone(),
                            "ip" => asset.ip = value.clone(),
                            "port" => asset.port = value.parse().unwrap_or(0),
                            "protocol" => asset.protocol = value.clone(),
                            "product" => asset.product = value.clone(),
                            "version" => asset.version = value.clone(),
                            "os" => asset.os = value.clone(),
                            "platform" => asset.platform = value.clone(),
                            "service" => asset.service = value.clone(),
                            "vuln" => asset.vuln = value.clone(),
                            "country" => asset.country = value.clone(),
                            "city" => asset.city = value.clone(),
                            "org" => asset.org = value.clone(),
                            "domain" => asset.domain = value.clone(),
                            "isp" => asset.isp = value.clone(),
                            _ => {}
                        }
                    }
                }
                asset
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// FOFA OSINT 调查 (对齐骨架 async 模式)
// ═══════════════════════════════════════════════════════════════

/// FOFA 默认搜索字段
pub const FOFA_DEFAULT_FIELDS: &str =
    "host,ip,port,protocol,product,version,os,service,vuln,country,org,domain,isp";

/// 使用 FOFA 调查目标 (async, 对齐 dns/http investigate 模式)
pub async fn investigate(
    target: &OsintTarget,
    client: &Client,
    config: &OsintConfig,
) -> Result<FofaFindings, String> {
    let fofa = FofaClient::from_config(config);
    if !fofa.is_configured() {
        return Err("FOFA API key 未配置, 跳过 FOFA 调查".into());
    }

    // 根据目标类型构建查询
    let query = build_fofa_query(target)?;
    if query.is_empty() {
        return Err("无法从目标构建 FOFA 查询".into());
    }

    // 执行搜索 (前 100 条)
    let resp = fofa
        .search(&query, FOFA_DEFAULT_FIELDS, 1, 100, client)
        .await?;
    let assets = fofa.parse_assets(&resp);

    // 提取服务指纹
    let service_fingerprints = extract_service_fingerprints(&assets);

    // 提取开放端口
    let mut open_ports: Vec<u16> = assets
        .iter()
        .filter(|a| a.port > 0)
        .map(|a| a.port)
        .collect();
    open_ports.sort();
    open_ports.dedup();

    // 提取漏洞
    let vulns = extract_vulns(&assets);

    Ok(FofaFindings {
        total: resp.size as usize,
        consumed_fpoints: resp.consumed_fpoints,
        remaining_fpoints: resp.remaining_fpoints,
        open_ports,
        service_fingerprints,
        vulns,
        assets,
        target: target_str(target),
    })
}

/// 从目标构建 FOFA 查询语法
fn build_fofa_query(target: &OsintTarget) -> Result<String, String> {
    if let Some(ref domain) = target.domain {
        return Ok(format!("domain=\"{}\"", domain));
    }
    if let Some(ref ip) = target.ip {
        return Ok(format!("ip=\"{}\"", ip));
    }
    if let Some(ref email) = target.email {
        return Ok(format!("email=\"{}\"", email));
    }
    if let Some(ref url) = target.url {
        if let Some(host) = url.split("://").nth(1) {
            let host = host.split('/').next().unwrap_or(host);
            return Ok(format!("domain=\"{}\"", host));
        }
    }
    if let Some(ref username) = target.username {
        return Ok(format!("title=\"{}\"", username));
    }
    Err("无法构建 FOFA 查询".into())
}

/// 从资产列表提取服务指纹
fn extract_service_fingerprints(assets: &[FofaAsset]) -> Vec<ServiceFingerprint> {
    let mut map: HashMap<String, (String, String, String, u16, usize)> = HashMap::new();
    for a in assets {
        if a.product.is_empty() {
            continue;
        }
        let key = format!("{}|{}|{}|{}", a.product, a.version, a.protocol, a.port);
        let entry = map.entry(key).or_insert_with(|| {
            (
                a.product.clone(),
                a.version.clone(),
                a.protocol.clone(),
                a.port,
                0,
            )
        });
        entry.4 += 1;
    }
    let mut fps: Vec<ServiceFingerprint> = map
        .values()
        .map(|(p, v, proto, port, count)| ServiceFingerprint {
            product: p.clone(),
            version: v.clone(),
            protocol: proto.clone(),
            port: *port,
            count: *count,
        })
        .collect();
    fps.sort_by(|a, b| b.count.cmp(&a.count));
    fps
}

/// 从资产列表提取漏洞
fn extract_vulns(assets: &[FofaAsset]) -> Vec<VulnInfo> {
    assets
        .iter()
        .filter(|a| !a.vuln.is_empty())
        .map(|a| VulnInfo {
            cve_id: a.vuln.clone(),
            affected_product: a.product.clone(),
            affected_ip: a.ip.clone(),
            affected_port: a.port,
        })
        .collect()
}

/// 目标字符串 (委托到 OsintTarget::primary_label)
fn target_str(target: &OsintTarget) -> String {
    target.primary_label()
}

// ═══════════════════════════════════════════════════════════════
// 同步客户端 (供 asset_map_capability 调用, 非 async 上下文)
// ═══════════════════════════════════════════════════════════════

/// 同步 FOFA 客户端 — 供 `UnifiedCapability::execute` (sync) 调用
pub struct FofaSyncClient {
    inner: FofaClient,
    client: reqwest::blocking::Client,
}

impl std::fmt::Debug for FofaSyncClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FofaSyncClient")
            .field("inner", &self.inner)
            .finish()
    }
}

impl Default for FofaSyncClient {
    fn default() -> Self {
        Self::from_env()
    }
}

impl FofaSyncClient {
    pub fn from_env() -> Self {
        Self {
            inner: FofaClient::from_env(),
            client: reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.1 (OSINT-FOFA-Sync)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.inner.is_configured()
    }

    /// 同步搜索 (供 Capability execute 调用)
    pub fn search(
        &self,
        query: &str,
        fields: &str,
        page: usize,
        size: usize,
    ) -> Result<FofaResponse, String> {
        if !self.is_configured() {
            return Err("FOFA API key 未配置".into());
        }
        let url = self.inner.build_search_url(query, fields, page, size);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("FOFA request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("FOFA HTTP {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("FOFA read body: {}", e))?;
        let parsed: FofaResponse =
            serde_json::from_str(&text).map_err(|e| format!("FOFA parse failed: {}", e))?;
        if parsed.error_code != 0 {
            return Err(format!(
                "FOFA API error [{}]: {}",
                parsed.error_code, parsed.errmsg
            ));
        }
        Ok(parsed)
    }

    pub fn parse_assets(&self, resp: &FofaResponse) -> Vec<FofaAsset> {
        self.inner.parse_assets(resp)
    }
}

// ═══════════════════════════════════════════════════════════════
// Egress 策略
// ═══════════════════════════════════════════════════════════════

pub const FOFA_API_HOST: &str = "fofa.info";

pub fn fofa_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(FOFA_API_HOST, "443")
}

pub fn fofa_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(
        vec![fofa_egress_rule()],
        false,
    )
}

// ═══════════════════════════════════════════════════════════════
// Fixture (测试)
// ═══════════════════════════════════════════════════════════════

pub const FOFA_FIXTURE_JSON: &str = r#"{
    "status": "ok",
    "error_code": 0,
    "errmsg": "",
    "size": 2,
    "results": [
        ["example.com", "1.2.3.4", "443", "https", "nginx", "1.18.0", "Linux", "ssl", "", "CN", "AS12345 Corp", "example.com", "CT"],
        ["api.example.com", "1.2.3.5", "8080", "http", "Apache httpd", "2.4.41", "Linux", "http", "CVE-2021-41773", "CN", "AS12345 Corp", "example.com", "CU"]
    ],
    "fields": ["host", "ip", "port", "protocol", "product", "version", "os", "service", "vuln", "country", "org", "domain", "isp"],
    "consumed_fpoints": 0,
    "remaining_fpoints": 10000
}"#;

#[cfg(test)]
mod tests {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fofa_response() {
        let resp: FofaResponse = serde_json::from_str(FOFA_FIXTURE_JSON).expect("parse");
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.size, 2);
        assert_eq!(resp.results.len(), 2);
    }

    #[test]
    fn test_parse_assets() {
        let client = FofaClient {
            api_key: String::new(),
            api_base: String::new(),
        };
        let resp: FofaResponse = serde_json::from_str(FOFA_FIXTURE_JSON).unwrap();
        let assets = client.parse_assets(&resp);
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].ip, "1.2.3.4");
        assert_eq!(assets[0].port, 443);
        assert_eq!(assets[0].product, "nginx");
        assert_eq!(assets[1].vuln, "CVE-2021-41773");
    }

    #[test]
    fn test_build_fofa_query_domain() {
        let target = OsintTarget::from_domain("example.com");
        let q = build_fofa_query(&target).unwrap();
        assert_eq!(q, r#"domain="example.com""#);
    }

    #[test]
    fn test_build_fofa_query_ip() {
        let target = OsintTarget {
            ip: Some("1.2.3.4".into()),
            ..Default::default()
        };
        let q = build_fofa_query(&target).unwrap();
        assert_eq!(q, r#"ip="1.2.3.4""#);
    }

    #[test]
    fn test_build_fofa_query_email() {
        let target = OsintTarget::from_email("user@example.com");
        let q = build_fofa_query(&target).unwrap();
        assert_eq!(q, r#"email="user@example.com""#);
    }

    #[test]
    fn test_build_fofa_query_url() {
        let target = OsintTarget {
            url: Some("https://sub.example.com/path".into()),
            ..Default::default()
        };
        let q = build_fofa_query(&target).unwrap();
        assert_eq!(q, r#"domain="sub.example.com""#);
    }

    #[test]
    fn test_build_fofa_query_username() {
        let target = OsintTarget::from_username("admin");
        let q = build_fofa_query(&target).unwrap();
        assert_eq!(q, r#"title="admin""#);
    }

    #[test]
    fn test_build_fofa_query_empty() {
        let target = OsintTarget::default();
        assert!(build_fofa_query(&target).is_err());
    }

    #[test]
    fn test_extract_service_fingerprints() {
        let assets = vec![
            FofaAsset {
                product: "nginx".into(),
                version: "1.18.0".into(),
                protocol: "https".into(),
                port: 443,
                ..Default::default()
            },
            FofaAsset {
                product: "nginx".into(),
                version: "1.18.0".into(),
                protocol: "https".into(),
                port: 443,
                ..Default::default()
            },
            FofaAsset {
                product: "Apache httpd".into(),
                version: "2.4.41".into(),
                protocol: "http".into(),
                port: 8080,
                ..Default::default()
            },
        ];
        let fps = extract_service_fingerprints(&assets);
        assert_eq!(fps.len(), 2);
        assert_eq!(fps[0].count, 2);
        assert_eq!(fps[1].count, 1);
    }

    #[test]
    fn test_extract_vulns() {
        let assets = vec![
            FofaAsset {
                vuln: "CVE-2021-41773".into(),
                product: "Apache".into(),
                ip: "1.2.3.5".into(),
                port: 8080,
                ..Default::default()
            },
            FofaAsset {
                vuln: "".into(),
                ..Default::default()
            },
        ];
        let vulns = extract_vulns(&assets);
        assert_eq!(vulns.len(), 1);
        assert_eq!(vulns[0].cve_id, "CVE-2021-41773");
    }

    #[test]
    fn test_fofa_client_not_configured() {
        let client = FofaClient::from_env();
        if std::env::var("NEOTRIX_FOFA_EMAIL").is_err() {
            assert!(!client.is_configured());
        }
    }

    #[test]
    fn test_sync_client_not_configured() {
        let client = FofaSyncClient::from_env();
        if std::env::var("NEOTRIX_FOFA_EMAIL").is_err() {
            assert!(!client.is_configured());
        }
    }

    #[test]
    fn test_egress_policy() {
        assert!(fofa_egress_policy().check("fofa.info", 443));
        assert!(!fofa_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_display_findings() {
        let findings = FofaFindings {
            total: 100,
            assets: vec![],
            open_ports: vec![80, 443],
            service_fingerprints: vec![],
            vulns: vec![],
            consumed_fpoints: 0,
            remaining_fpoints: 10000,
            target: "example.com".into(),
        };
        let s = format!("{findings}");
        assert!(s.contains("FOFA Asset Discovery"));
        assert!(s.contains("example.com"));
    }

    #[test]
    fn test_debug_impl() {
        let client = FofaClient::from_env();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("FofaClient"));
    }
}
