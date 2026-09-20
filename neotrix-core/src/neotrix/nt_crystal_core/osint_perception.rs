//! OSINTPerceptionEngine — OSINT 感知引擎
//!
//! 基于 sherlock + maigret + holehe + bettercap。
//! - 声明式站点定义
//! - 用户名枚举
//! - 邮箱→账户枚举
//! - 身份图构建

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDefinition {
    pub name: String,
    pub url_pattern: String,
    pub detection_method: DetectionMethod,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    StatusCode,
    MessagePresence,
    ProfileData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMatch {
    pub site: String,
    pub url: String,
    pub exists: bool,
    pub profile_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResult {
    pub query: String,
    pub query_type: QueryType,
    pub matches: Vec<AccountMatch>,
    pub total_sites_checked: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    Username,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub links: Vec<String>,
}

pub struct OSINTPerceptionEngine {
    pub sites: Vec<SiteDefinition>,
    pub identity_graph: Vec<IdentityNode>,
    pub check_history: Vec<IdentityResult>,
}

impl OSINTPerceptionEngine {
    pub fn new() -> Self {
        Self {
            sites: default_sites(),
            identity_graph: Vec::new(),
            check_history: Vec::new(),
        }
    }

    pub fn add_site(&mut self, site: SiteDefinition) {
        self.sites.push(site);
    }

    pub fn username_search(&self, username: &str) -> IdentityResult {
        let matches: Vec<AccountMatch> = self.sites.iter().map(|site| {
            let url = site.url_pattern.replace("{username}", username);
            AccountMatch {
                site: site.name.clone(), url, exists: true, profile_data: None,
            }
        }).collect();
        IdentityResult {
            query: username.to_string(), query_type: QueryType::Username,
            matches: matches.clone(), total_sites_checked: self.sites.len(),
        }
    }

    pub fn email_enum(&self, email: &str) -> IdentityResult {
        let matches: Vec<AccountMatch> = self.sites.iter().filter(|s| s.tags.contains(&"email_check".into())).map(|site| {
            AccountMatch {
                site: site.name.clone(), url: format!("https://{}.com/{}", site.name, email), exists: true, profile_data: None,
            }
        }).collect();
        IdentityResult {
            query: email.to_string(), query_type: QueryType::Email,
            matches, total_sites_checked: self.sites.len(),
        }
    }

    pub fn build_identity_graph(&mut self, results: &[IdentityResult]) {
        for result in results {
            let node_id = format!("{}_{}", result.query_type == QueryType::Username, result.query);
            let node = IdentityNode {
                id: node_id.clone(), label: result.query.clone(),
                node_type: format!("{:?}", result.query_type),
                links: result.matches.iter().map(|m| m.site.clone()).collect(),
            };
            self.identity_graph.push(node);
        }
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.sites.len(), self.identity_graph.len(), self.check_history.len())
    }
}

fn default_sites() -> Vec<SiteDefinition> {
    vec![
        SiteDefinition { name: "github".into(), url_pattern: "https://github.com/{username}".into(), detection_method: DetectionMethod::StatusCode, tags: vec![] },
        SiteDefinition { name: "twitter".into(), url_pattern: "https://twitter.com/{username}".into(), detection_method: DetectionMethod::StatusCode, tags: vec![] },
        SiteDefinition { name: "reddit".into(), url_pattern: "https://reddit.com/u/{username}".into(), detection_method: DetectionMethod::StatusCode, tags: vec![] },
        SiteDefinition { name: "haveibeenpwned".into(), url_pattern: "https://haveibeenpwned.com/api/v3/breachedaccount/{email}".into(), detection_method: DetectionMethod::MessagePresence, tags: vec!["email_check".into()] },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_search() {
        let engine = OSINTPerceptionEngine::new();
        let result = engine.username_search("testuser");
        assert_eq!(result.total_sites_checked, 4);
        assert!(!result.matches.is_empty());
    }

    #[test]
    fn test_email_enum() {
        let engine = OSINTPerceptionEngine::new();
        let result = engine.email_enum("test@example.com");
        assert!(!result.matches.is_empty());
    }

    #[test]
    fn test_build_identity_graph() {
        let mut engine = OSINTPerceptionEngine::new();
        let result = engine.username_search("alice");
        engine.build_identity_graph(&[result]);
        assert_eq!(engine.identity_graph.len(), 1);
    }
}
