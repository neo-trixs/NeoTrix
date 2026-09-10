//! OSINT 统一资产模型 — 参考 Amass Open Asset Model
//!
//! 将所有 OSINT 发现统一到一个资产实体模型中，
//! 支持跨模块关联分析和攻击面管理。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 资产类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetType {
    Domain,
    Ip,
    Port,
    Service,
    Certificate,
    Technology,
    Vulnerability,
    Person,
    Email,
    Username,
    Url,
    Onion,
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Domain
    }
}

/// 统一资产实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsintAsset {
    pub id: String,
    pub asset_type: AssetType,
    pub value: String,
    pub properties: HashMap<String, String>,
    pub relations: Vec<AssetRelation>,
    pub source: String,
    pub confidence: f64,
    pub first_seen: i64,
    pub last_seen: i64,
}

/// 资产关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRelation {
    pub target_id: String,
    pub relation_type: RelationType,
    pub confidence: f64,
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    ResolvesTo,      // domain → ip
    Exposes,         // ip → port
    Runs,            // port → service
    HasVuln,         // service → vulnerability
    Uses,            // service → technology
    Owns,            // person → domain/email
    Mentions,        // url → person/domain
}

/// 资产图 — 管理所有发现的资产
#[derive(Debug, Clone, Default)]
pub struct AssetGraph {
    pub assets: HashMap<String, OsintAsset>,
    pub relations: Vec<AssetRelation>,
}

impl AssetGraph {
    pub fn new() -> Self { Self::default() }

    /// 添加资产
    pub fn add_asset(&mut self, asset: OsintAsset) {
        self.assets.insert(asset.id.clone(), asset);
    }

    /// 添加关系
    pub fn add_relation(&mut self, relation: AssetRelation) {
        self.relations.push(relation);
    }

    /// 查找相关资产
    pub fn find_related(&self, asset_id: &str, relation_type: &RelationType) -> Vec<&OsintAsset> {
        self.relations.iter()
            .filter(|r| r.target_id == asset_id && r.relation_type == *relation_type)
            .filter_map(|r| self.assets.get(&r.target_id))
            .collect()
    }

    /// 计算攻击面评分
    pub fn attack_surface_score(&self) -> f64 {
        let domain_count = self.assets.values().filter(|a| a.asset_type == AssetType::Domain).count() as f64;
        let ip_count = self.assets.values().filter(|a| a.asset_type == AssetType::Ip).count() as f64;
        let port_count = self.assets.values().filter(|a| a.asset_type == AssetType::Port).count() as f64;
        let vuln_count = self.assets.values().filter(|a| a.asset_type == AssetType::Vulnerability).count() as f64;

        // 攻击面 = 域名数 * 1.0 + IP数 * 0.8 + 端口数 * 0.5 + 漏洞数 * 2.0
        (domain_count * 1.0 + ip_count * 0.8 + port_count * 0.5 + vuln_count * 2.0) / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_graph() {
        let mut graph = AssetGraph::new();
        let domain = OsintAsset {
            id: "example.com".into(),
            asset_type: AssetType::Domain,
            value: "example.com".into(),
            source: "dns".into(),
            confidence: 1.0,
            ..Default::default()
        };
        graph.add_asset(domain);
        assert_eq!(graph.assets.len(), 1);
    }

    #[test]
    fn test_attack_surface_score() {
        let mut graph = AssetGraph::new();
        // 添加测试资产
        for i in 0..5 {
            graph.add_asset(OsintAsset {
                id: format!("ip-{}", i),
                asset_type: AssetType::Ip,
                value: format!("192.168.1.{}", i),
                source: "network".into(),
                confidence: 1.0,
                ..Default::default()
            });
        }
        let score = graph.attack_surface_score();
        assert!(score > 0.0);
    }
}
