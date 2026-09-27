//! NT-WORLD Asset Map: 资产数据模型
//!
//! 定义资产、端口、服务、证书、变更历史等核心数据结构。

use std::collections::HashMap;
use std::net::IpAddr;
use chrono::{DateTime, Utc};

/// 资产
#[derive(Debug, Clone)]
pub struct Asset {
    /// 资产ID (数据库主键)
    pub id: Option<i64>,
    /// IP地址
    pub ip_addr: IpAddr,
    /// 主机名
    pub hostname: Option<String>,
    /// 资产类型
    pub asset_type: AssetType,
    /// 所有者/组织
    pub org: Option<String>,
    /// 地理位置
    pub geo: Option<GeoLocation>,
    /// 标签
    pub tags: Vec<String>,
    /// 发现时间
    pub discovered_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 资产类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// 服务器
    Server,
    /// 负载均衡器
    LoadBalancer,
    /// 防火墙
    Firewall,
    /// 路由器
    Router,
    /// 交换机
    Switch,
    /// 终端设备
    Endpoint,
    /// 容器
    Container,
    /// 未知
    Unknown,
}

/// 地理位置
#[derive(Debug, Clone)]
pub struct GeoLocation {
    /// 国家代码 (ISO 3166-1)
    pub country_code: String,
    /// 国家名称
    pub country_name: String,
    /// 省份/州
    pub region: Option<String>,
    /// 城市
    pub city: Option<String>,
    /// 纬度
    pub latitude: Option<f64>,
    /// 经度
    pub longitude: Option<f64>,
    /// ASN
    pub asn: Option<u32>,
    /// 组织
    pub organization: Option<String>,
}

/// 端口
#[derive(Debug, Clone)]
pub struct Port {
    /// 端口号
    pub port_number: u16,
    /// 协议
    pub protocol: TransportProtocol,
    /// 服务
    pub service: Service,
    /// 状态
    pub state: PortState,
    /// Banner
    pub banner: Option<String>,
    /// 最后扫描时间
    pub last_scanned: DateTime<Utc>,
}

/// 传输层协议
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

/// 端口状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    /// 开放
    Open,
    /// 关闭
    Closed,
    /// 过滤
    Filtered,
    /// 开放|过滤
    OpenFiltered,
}

/// 服务
#[derive(Debug, Clone)]
pub struct Service {
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: Option<String>,
    /// 产品名
    pub product: Option<String>,
    /// 置信度 (0-100)
    pub confidence: u8,
    /// 额外信息
    pub extra_info: HashMap<String, String>,
}

/// SSL证书
#[derive(Debug, Clone)]
pub struct SslCertificate {
    /// 序列号
    pub serial_number: String,
    /// 颁发者
    pub issuer: String,
    /// 主题
    pub subject: String,
    /// 有效期开始
    pub not_before: DateTime<Utc>,
    /// 有效期结束
    pub not_after: DateTime<Utc>,
    /// SAN域名列表
    pub san_domains: Vec<String>,
    /// SAN IP列表
    pub san_ips: Vec<IpAddr>,
    /// 证书链
    pub chain: Vec<String>,
    /// 是否自签名
    pub self_signed: bool,
    /// 签名算法
    pub signature_algorithm: String,
    /// 公钥长度
    pub key_size: Option<u32>,
    /// 证书指纹
    pub fingerprint: String,
}

/// 网页信息
#[derive(Debug, Clone)]
pub struct WebInfo {
    /// 标题
    pub title: Option<String>,
    /// 状态码
    pub status_code: u16,
    /// 重定向URL
    pub redirect_url: Option<String>,
    /// HTTP头
    pub headers: HashMap<String, String>,
    /// Favicon哈希 (MurmurHash3)
    pub favicon_hash: Option<i32>,
    /// JARM指纹
    pub jarm_fingerprint: Option<String>,
    /// 技术标签
    pub tech_tags: Vec<String>,
    /// CMS
    pub cms: Option<String>,
    /// Web服务器
    pub web_server: Option<String>,
    /// 编程语言
    pub programming_language: Option<String>,
    /// 框架
    pub framework: Option<String>,
    /// 数据库
    pub database: Option<String>,
    /// 操作系统
    pub os: Option<String>,
}

/// DNS记录
#[derive(Debug, Clone)]
pub struct DnsRecord {
    /// 记录类型
    pub record_type: DnsRecordType,
    /// 记录值
    pub value: String,
    /// TTL
    pub ttl: Option<u32>,
}

/// DNS记录类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    MX,
    NS,
    TXT,
    SOA,
    PTR,
}

/// 资产变更历史
#[derive(Debug, Clone)]
pub struct AssetChange {
    /// 变更ID
    pub id: Option<i64>,
    /// 资产ID
    pub asset_id: i64,
    /// 变更类型
    pub change_type: ChangeType,
    /// 变更字段
    pub field: String,
    /// 旧值
    pub old_value: Option<String>,
    /// 新值
    pub new_value: Option<String>,
    /// 变更时间
    pub changed_at: DateTime<Utc>,
}

/// 变更类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// 新增端口
    PortAdded,
    /// 端口关闭
    PortRemoved,
    /// 服务变更
    ServiceChanged,
    /// 证书更新
    CertificateRenewed,
    /// DNS变更
    DnsChanged,
    /// 内容变更
    ContentChanged,
}

/// 端口统计
#[derive(Debug, Clone, Default)]
pub struct PortStats {
    /// 总端口数
    pub total: u32,
    /// 开放端口数
    pub open: u32,
    /// 关闭端口数
    pub closed: u32,
    /// 过滤端口数
    pub filtered: u32,
    /// 按服务分类
    pub by_service: HashMap<String, u32>,
}

/// 扫描结果
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// 目标地址
    pub target: IpAddr,
    /// 扫描时间
    pub scanned_at: DateTime<Utc>,
    /// 发现的端口
    pub ports: Vec<Port>,
    /// SSL证书
    pub certificate: Option<SslCertificate>,
    /// 网页信息
    pub web_info: Option<WebInfo>,
    /// DNS记录
    pub dns_records: Vec<DnsRecord>,
    /// Banner信息
    pub banners: HashMap<u16, String>,
    /// 扫描耗时
    pub scan_duration_ms: u64,
    /// 扫描状态
    pub status: ScanStatus,
}

/// 扫描状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    /// 成功
    Success,
    /// 部分成功
    PartialSuccess,
    /// 失败
    Failed { reason: String },
}

impl ScanResult {
    /// 计算端口统计
    pub fn port_stats(&self) -> PortStats {
        let mut stats = PortStats::default();
        for port in &self.ports {
            stats.total += 1;
            match port.state {
                PortState::Open => stats.open += 1,
                PortState::Closed => stats.closed += 1,
                PortState::Filtered | PortState::OpenFiltered => stats.filtered += 1,
            }
            *stats.by_service.entry(port.service.name.clone()).or_default() += 1;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn asset_creation() {
        let asset = Asset {
            id: None,
            ip_addr: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            hostname: Some("example.com".into()),
            asset_type: AssetType::Server,
            org: Some("Example Corp".into()),
            geo: None,
            tags: vec!["web-server".into()],
            discovered_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(asset.ip_addr, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn scan_result_stats() {
        let result = ScanResult {
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            scanned_at: Utc::now(),
            ports: vec![
                Port {
                    port_number: 80,
                    protocol: TransportProtocol::Tcp,
                    service: Service { name: "http".into(), version: None, product: None, confidence: 90, extra_info: HashMap::new() },
                    state: PortState::Open,
                    banner: None,
                    last_scanned: Utc::now(),
                },
                Port {
                    port_number: 443,
                    protocol: TransportProtocol::Tcp,
                    service: Service { name: "https".into(), version: None, product: None, confidence: 90, extra_info: HashMap::new() },
                    state: PortState::Open,
                    banner: None,
                    last_scanned: Utc::now(),
                },
                Port {
                    port_number: 22,
                    protocol: TransportProtocol::Tcp,
                    service: Service { name: "ssh".into(), version: None, product: None, confidence: 85, extra_info: HashMap::new() },
                    state: PortState::Closed,
                    banner: None,
                    last_scanned: Utc::now(),
                },
            ],
            certificate: None,
            web_info: None,
            dns_records: vec![],
            banners: HashMap::new(),
            scan_duration_ms: 150,
            status: ScanStatus::Success,
        };

        let stats = result.port_stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.open, 2);
        assert_eq!(stats.closed, 1);
    }
}
