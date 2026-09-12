//! Internal Network Scanning
//!
//! 吸收 fscan (14K★) 内部网络侦察 + PentestCode engagement state:
//! - ARP ping / ICMP 主机发现
//! - 服务指纹识别
//! - 漏洞检测
//! - 攻击图构建
//! - 持久化 engagement state

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 内部网络扫描器 — fscan 抽象 (旧名称 FscanModule 兼容)
pub struct FscanModule {
    /// 发现的内部主机
    internal_hosts: Vec<_HostInfo>,
    /// 每主机服务指纹
    services: HashMap<String, Vec<ServiceInfo>>,
    /// 漏洞发现
    vulnerabilities: Vec<_VulnerabilityInfo>,
    /// 攻击图
    attack_graph: _AttackGraph,
    /// 扫描配置
    config: ScanConfig,
}

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub target_cidr: String,
    pub ports: Vec<u16>,
    pub timeout_ms: u64,
    pub max_concurrent: usize,
    pub enable_os_detection: bool,
    pub enable_script_scan: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target_cidr: "192.168.0.0/16".into(),
            ports: vec![22, 80, 443, 3306, 5432, 8080],
            timeout_ms: 5000,
            max_concurrent: 100,
            enable_os_detection: true,
            enable_script_scan: true,
        }
    }
}

/// 主机信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _HostInfo {
    pub ip: String,
    pub hostname: Option<String>,
    pub os: Option<String>,
    pub mac: Option<String>,
    pub state: _HostState,
    pub open_ports: Vec<u16>,
}

/// 主机状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _HostState {
    Up,
    Down,
    Unknown,
}

/// 服务指纹信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub port: u16,
    pub protocol: String,
    pub service: String,
    pub version: Option<String>,
    pub banner: String,
    pub vulnerability: Option<String>,
    pub confidence: f64,
}

/// 漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _VulnerabilityInfo {
    pub id: String,
    pub host_ip: String,
    pub port: u16,
    pub service: String,
    pub vuln_type: String,
    pub severity: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
    pub exploitable: bool,
}

/// 攻击图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AttackGraph {
    pub nodes: Vec<_AttackNode>,
    pub edges: Vec<_AttackEdge>,
    pub paths: Vec<AttackPath>,
}

/// 攻击节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AttackNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 攻击边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AttackEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub cost: f64,
    pub method: String,
}

/// 攻击路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPath {
    pub source: String,
    pub target: String,
    pub path: Vec<String>,
    pub total_cost: f64,
}

impl FscanModule {
    /// 创建新的扫描器
    pub fn new() -> Self {
        Self {
            internal_hosts: vec![],
            services: HashMap::new(),
            vulnerabilities: vec![],
            attack_graph: _AttackGraph {
                nodes: vec![],
                edges: vec![],
                paths: vec![],
            },
            config: ScanConfig::default(),
        }
    }

    /// 创建带配置的扫描器
    pub fn with_config(config: ScanConfig) -> Self {
        Self {
            internal_hosts: vec![],
            services: HashMap::new(),
            vulnerabilities: vec![],
            attack_graph: _AttackGraph {
                nodes: vec![],
                edges: vec![],
                paths: vec![],
            },
            config,
        }
    }

    /// 发现内部主机 (旧名称兼容)
    pub async fn discover_internal_hosts(&mut self) -> Option<Vec<String>> {
        match self.discover_hosts().await {
            Ok(hosts) => Some(hosts.iter().map(|h| h.ip.clone()).collect()),
            Err(_) => None,
        }
    }

    /// 发现内部主机 (ARP ping / ICMP)
    pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
        // TODO: 实际调用 fscan 或系统命令
        // 示例: fscan -t 192.168.0.0/16 -p all

        // 模拟发现结果
        let hosts = vec![
            _HostInfo {
                ip: "192.168.1.1".into(),
                hostname: Some("gateway".into()),
                os: Some("Linux".into()),
                mac: Some("00:11:22:33:44:55".into()),
                state: _HostState::Up,
                open_ports: vec![22, 80, 443],
            },
            _HostInfo {
                ip: "192.168.1.10".into(),
                hostname: Some("webserver".into()),
                os: Some("Ubuntu 20.04".into()),
                mac: None,
                state: _HostState::Up,
                open_ports: vec![22, 80, 3306],
            },
            _HostInfo {
                ip: "192.168.1.25".into(),
                hostname: Some("database".into()),
                os: Some("CentOS 7".into()),
                mac: None,
                state: _HostState::Up,
                open_ports: vec![22, 3306, 5432],
            },
        ];

        self.internal_hosts = hosts.clone();
        Ok(hosts)
    }

    /// 枚举主机服务
    pub async fn enumerate_services(&mut self, host: &str) -> Result<Vec<ServiceInfo>, String> {
        // TODO: 实际调用 nmap 或服务枚举工具

        let services = match host {
            "192.168.1.1" => vec![
                ServiceInfo {
                    port: 22,
                    protocol: "tcp".into(),
                    service: "ssh".into(),
                    version: Some("OpenSSH 8.2p1".into()),
                    banner: "SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3".into(),
                    vulnerability: None,
                    confidence: 0.95,
                },
                ServiceInfo {
                    port: 80,
                    protocol: "tcp".into(),
                    service: "http".into(),
                    version: Some("nginx 1.18.0".into()),
                    banner: "nginx/1.18.0 (Ubuntu)".into(),
                    vulnerability: Some("Server version exposed".into()),
                    confidence: 0.9,
                },
            ],
            "192.168.1.10" => vec![
                ServiceInfo {
                    port: 22,
                    protocol: "tcp".into(),
                    service: "ssh".into(),
                    version: Some("OpenSSH 8.2p1".into()),
                    banner: "SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3".into(),
                    vulnerability: None,
                    confidence: 0.95,
                },
                ServiceInfo {
                    port: 80,
                    protocol: "tcp".into(),
                    service: "http".into(),
                    version: Some("Apache/2.4.41".into()),
                    banner: "Apache/2.4.41 (Ubuntu)".into(),
                    vulnerability: Some("HTTP-Only flag missing".into()),
                    confidence: 0.9,
                },
                ServiceInfo {
                    port: 3306,
                    protocol: "tcp".into(),
                    service: "mysql".into(),
                    version: Some("MySQL 8.0.28".into()),
                    banner: "MySQL 8.0.28-0ubuntu0.20.04.3".into(),
                    vulnerability: Some("Default account 'root' accessible".into()),
                    confidence: 0.85,
                },
            ],
            _ => vec![],
        };

        self.services.insert(host.to_string(), services.clone());
        Ok(services)
    }

    /// 检测漏洞
    pub async fn detect_vulnerabilities(&mut self, host: &str) -> Result<Vec<_VulnerabilityInfo>, String> {
        let mut vulns = Vec::new();

        if let Some(services) = self.services.get(host) {
            for service in services {
                if let Some(ref vuln_desc) = service.vulnerability {
                    let vuln = _VulnerabilityInfo {
                        id: format!("VULN-{}-{}", host, service.port),
                        host_ip: host.to_string(),
                        port: service.port,
                        service: service.service.clone(),
                        vuln_type: "misconfiguration".into(),
                        severity: "medium".into(),
                        evidence: vec![vuln_desc.clone()],
                        confidence: service.confidence,
                        exploitable: true,
                    };
                    vulns.push(vuln);
                }
            }
        }

        self.vulnerabilities.extend(vulns.clone());
        Ok(vulns)
    }

    /// 构建攻击图
    pub fn _build_attack_graph(&mut self) -> _AttackGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 添加主机节点
        for host in &self.internal_hosts {
            nodes.push(_AttackNode {
                id: host.ip.clone(),
                node_type: "host".into(),
                label: host.hostname.clone().unwrap_or_else(|| host.ip.clone()),
                properties: serde_json::json!({
                    "ip": host.ip,
                    "os": host.os,
                    "open_ports": host.open_ports,
                }).as_object().unwrap().clone(),
            });
        }

        // 添加服务节点和边
        for (host_ip, services) in &self.services {
            for service in services {
                let service_node_id = format!("{}:{}", host_ip, service.port);
                nodes.push(_AttackNode {
                    id: service_node_id.clone(),
                    node_type: "service".into(),
                    label: format!("{} ({})", service.service, service.port),
                    properties: serde_json::json!({
                        "port": service.port,
                        "service": service.service,
                        "version": service.version,
                    }).as_object().unwrap().clone(),
                });

                edges.push(_AttackEdge {
                    source: host_ip.clone(),
                    target: service_node_id.clone(),
                    edge_type: "hosts".into(),
                    cost: 1.0,
                    method: "direct".into(),
                });
            }
        }

        // 添加漏洞利用边
        for vuln in &self.vulnerabilities {
            if vuln.exploitable {
                let vuln_node_id = format!("vuln:{}", vuln.id);
                nodes.push(_AttackNode {
                    id: vuln_node_id.clone(),
                    node_type: "vulnerability".into(),
                    label: format!("{} ({})", vuln.vuln_type, vuln.severity),
                    properties: serde_json::json!({
                        "severity": vuln.severity,
                        "confidence": vuln.confidence,
                    }).as_object().unwrap().clone(),
                });

                edges.push(_AttackEdge {
                    source: format!("{}:{}", vuln.host_ip, vuln.port),
                    target: vuln_node_id.clone(),
                    edge_type: "vulnerable_to".into(),
                    cost: 0.5,
                    method: vuln.vuln_type.clone(),
                });
            }
        }

        // 计算攻击路径
        let paths = self.find_attack_paths();

        self.attack_graph = _AttackGraph {
            nodes,
            edges,
            paths,
        };

        self.attack_graph.clone()
    }

    /// 查找攻击路径 — 简化版 Dijkstra
    fn find_attack_paths(&self) -> Vec<AttackPath> {
        let mut paths = Vec::new();

        // 从每个主机到每个漏洞
        for host in &self.internal_hosts {
            for vuln in &self.vulnerabilities {
                if vuln.host_ip == host.ip {
                    paths.push(AttackPath {
                        source: host.ip.clone(),
                        target: format!("vuln:{}", vuln.id),
                        path: vec![
                            host.ip.clone(),
                            format!("{}:{}", vuln.host_ip, vuln.port),
                            format!("vuln:{}", vuln.id),
                        ],
                        total_cost: 1.5,
                    });
                }
            }
        }

        paths
    }

    /// 获取发现的主机
    pub fn _get_hosts(&self) -> &[_HostInfo] {
        &self.internal_hosts
    }

    /// 获取服务信息
    pub fn _get_services(&self) -> &HashMap<String, Vec<ServiceInfo>> {
        &self.services
    }

    /// 获取漏洞信息
    pub fn _get_vulnerabilities(&self) -> &[_VulnerabilityInfo] {
        &self.vulnerabilities
    }

    /// 获取攻击图
    pub fn _get_attack_graph(&self) -> &_AttackGraph {
        &self.attack_graph
    }
}
