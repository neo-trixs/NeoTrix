//! NT-WORLD Asset Map: 指纹匹配引擎
//!
//! 基于多维度特征匹配识别资产类型、技术栈、服务指纹。
//! 参考: FOFA 350K+指纹库、Shodan DIT + HTTP中特征

use std::collections::HashMap;

/// 指纹维度
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FingerprintDimension {
    /// HTTP响应头
    HttpHeader,
    /// HTTP Body关键词
    HttpBody,
    /// TLS证书字段
    TlsCert,
    /// Banner特征
    Banner,
    /// 端口协议
    PortProtocol,
    /// Favicon哈希
    FaviconHash,
    /// JARM指纹
    JarmFingerprint,
    /// DIT数据特征
    DitCharacteristic,
}

/// 指纹规则
#[derive(Debug, Clone)]
pub struct FingerprintRule {
    /// 规则ID
    pub id: String,
    /// 维度
    pub dimension: FingerprintDimension,
    /// 匹配模式 (正则或精确)
    pub pattern: String,
    /// 匹配权重 (0-100)
    pub weight: u8,
    /// 标签 (技术栈/服务名)
    pub tags: Vec<String>,
}

/// 匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// 规则ID
    pub rule_id: String,
    /// 匹配置信度 (0-100)
    pub confidence: u8,
    /// 命中的标签
    pub tags: Vec<String>,
    /// 匹配详情
    pub details: HashMap<String, String>,
}

/// 指纹匹配引擎
pub struct FingerprintEngine {
    /// 规则库
    rules: Vec<FingerprintRule>,
    /// 规则索引 (dimension → rules)
    index: HashMap<FingerprintDimension, Vec<usize>>,
}

impl FingerprintEngine {
    /// 创建空的指纹引擎
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// 从JSON加载规则库
    pub fn from_rules(rules: Vec<FingerprintRule>) -> Self {
        let mut engine = Self::new();
        for rule in rules {
            engine.add_rule(rule);
        }
        engine
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: FingerprintRule) {
        let idx = self.rules.len();
        self.index.entry(rule.dimension.clone()).or_default().push(idx);
        self.rules.push(rule);
    }

    /// 匹配单个维度
    pub fn match_dimension(
        &self,
        dimension: &FingerprintDimension,
        data: &str,
    ) -> Vec<MatchResult> {
        let mut results = Vec::new();

        if let Some(rule_indices) = self.index.get(dimension) {
            for &idx in rule_indices {
                let rule = &self.rules[idx];
                if self.match_pattern(&rule.pattern, data) {
                    results.push(MatchResult {
                        rule_id: rule.id.clone(),
                        confidence: rule.weight,
                        tags: rule.tags.clone(),
                        details: HashMap::new(),
                    });
                }
            }
        }

        results
    }

    /// 综合匹配所有维度
    pub fn match_all(
        &self,
        probe_data: &ProbeData,
    ) -> Vec<MatchResult> {
        let mut all_results = Vec::new();

        // HTTP Header匹配
        for (key, value) in &probe_data.http_headers {
            let data = format!("{}: {}", key, value);
            all_results.extend(self.match_dimension(&FingerprintDimension::HttpHeader, &data));
        }

        // HTTP Body匹配
        if let Some(body) = &probe_data.http_body {
            all_results.extend(self.match_dimension(&FingerprintDimension::HttpBody, body));
        }

        // TLS证书匹配
        if let Some(cert) = &probe_data.tls_cert_issuer {
            all_results.extend(self.match_dimension(&FingerprintDimension::TlsCert, cert));
        }

        // Banner匹配
        if let Some(banner) = &probe_data.banner {
            all_results.extend(self.match_dimension(&FingerprintDimension::Banner, banner));
        }

        // Favicon哈希匹配
        if let Some(hash) = &probe_data.favicon_hash {
            all_results.extend(self.match_dimension(&FingerprintDimension::FaviconHash, &hash.to_string()));
        }

        // JARM指纹匹配
        if let Some(jarm) = &probe_data.jarm_fingerprint {
            all_results.extend(self.match_dimension(&FingerprintDimension::JarmFingerprint, jarm));
        }

        // 去重并按置信度排序
        all_results.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        all_results.dedup_by_key(|r| r.rule_id.clone());

        all_results
    }

    /// 模式匹配
    fn match_pattern(&self, pattern: &str, data: &str) -> bool {
        if pattern.starts_with("regex:") {
            let regex_pattern = &pattern[6..];
            // 简单正则匹配 (生产环境应使用regex crate)
            data.contains(regex_pattern)
        } else {
            data.contains(pattern)
        }
    }
}

/// 探测数据
#[derive(Debug, Clone, Default)]
pub struct ProbeData {
    /// HTTP响应头
    pub http_headers: HashMap<String, String>,
    /// HTTP Body
    pub http_body: Option<String>,
    /// TLS证书颁发者
    pub tls_cert_issuer: Option<String>,
    /// Banner
    pub banner: Option<String>,
    /// 端口号
    pub port: Option<u16>,
    /// Favicon哈希 (MurmurHash3)
    pub favicon_hash: Option<i32>,
    /// JARM指纹
    pub jarm_fingerprint: Option<String>,
}

/// 预定义指纹规则库
pub mod rules {
    use super::*;

    /// 创建 Web服务器指纹规则
    pub fn web_server_rules() -> Vec<FingerprintRule> {
        vec![
            FingerprintRule {
                id: "nginx".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "Server: nginx".into(),
                weight: 90,
                tags: vec!["web-server".into(), "nginx".into()],
            },
            FingerprintRule {
                id: "apache".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "Server: Apache".into(),
                weight: 90,
                tags: vec!["web-server".into(), "apache".into()],
            },
            FingerprintRule {
                id: "iis".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "Server: Microsoft-IIS".into(),
                weight: 90,
                tags: vec!["web-server".into(), "iis".into()],
            },
            FingerprintRule {
                id: "caddy".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "Server: Caddy".into(),
                weight: 90,
                tags: vec!["web-server".into(), "caddy".into()],
            },
        ]
    }

    /// 创建 CMS指纹规则
    pub fn cms_rules() -> Vec<FingerprintRule> {
        vec![
            FingerprintRule {
                id: "wordpress".into(),
                dimension: FingerprintDimension::HttpBody,
                pattern: "wp-content".into(),
                weight: 95,
                tags: vec!["cms".into(), "wordpress".into()],
            },
            FingerprintRule {
                id: "drupal".into(),
                dimension: FingerprintDimension::HttpBody,
                pattern: "Drupal.settings".into(),
                weight: 95,
                tags: vec!["cms".into(), "drupal".into()],
            },
            FingerprintRule {
                id: "joomla".into(),
                dimension: FingerprintDimension::HttpBody,
                pattern: "/media/jui/".into(),
                weight: 90,
                tags: vec!["cms".into(), "joomla".into()],
            },
        ]
    }

    /// 创建框架指纹规则
    pub fn framework_rules() -> Vec<FingerprintRule> {
        vec![
            FingerprintRule {
                id: "laravel".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "X-Powered-By: Laravel".into(),
                weight: 95,
                tags: vec!["framework".into(), "laravel".into(), "php".into()],
            },
            FingerprintRule {
                id: "django".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "csrftoken".into(),
                weight: 85,
                tags: vec!["framework".into(), "django".into(), "python".into()],
            },
            FingerprintRule {
                id: "express".into(),
                dimension: FingerprintDimension::HttpHeader,
                pattern: "X-Powered-By: Express".into(),
                weight: 95,
                tags: vec!["framework".into(), "express".into(), "nodejs".into()],
            },
        ]
    }

    /// 创建数据库指纹规则
    pub fn database_rules() -> Vec<FingerprintRule> {
        vec![
            FingerprintRule {
                id: "mysql".into(),
                dimension: FingerprintDimension::Banner,
                pattern: "mysql".into(),
                weight: 90,
                tags: vec!["database".into(), "mysql".into()],
            },
            FingerprintRule {
                id: "postgresql".into(),
                dimension: FingerprintDimension::Banner,
                pattern: "PostgreSQL".into(),
                weight: 90,
                tags: vec!["database".into(), "postgresql".into()],
            },
            FingerprintRule {
                id: "redis".into(),
                dimension: FingerprintDimension::Banner,
                pattern: "redis_version".into(),
                weight: 95,
                tags: vec!["database".into(), "redis".into()],
            },
        ]
    }

    /// 创建完整的默认规则库
    pub fn default_rules() -> Vec<FingerprintRule> {
        let mut rules = Vec::new();
        rules.extend(web_server_rules());
        rules.extend(cms_rules());
        rules.extend(framework_rules());
        rules.extend(database_rules());
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_nginx_server() {
        let engine = FingerprintEngine::from_rules(rules::web_server_rules());
        let mut probe = ProbeData::default();
        probe.http_headers.insert("Server".into(), "nginx/1.18.0".into());

        let results = engine.match_all(&probe);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.tags.contains(&"nginx".into())));
    }

    #[test]
    fn match_wordpress() {
        let engine = FingerprintEngine::from_rules(rules::cms_rules());
        let mut probe = ProbeData::default();
        probe.http_body = Some("<link rel='stylesheet' href='https://example.com/wp-content/themes/style.css'>".into());

        let results = engine.match_all(&probe);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.tags.contains(&"wordpress".into())));
    }

    #[test]
    fn match_multiple_dimensions() {
        let engine = FingerprintEngine::from_rules(rules::default_rules());
        let mut probe = ProbeData::default();
        probe.http_headers.insert("Server".into(), "Apache/2.4.41".into());
        probe.http_body = Some("wp-content/uploads/2024/01/logo.png".into());

        let results = engine.match_all(&probe);
        assert!(results.len() >= 2);
    }

    #[test]
    fn match_confidence_ordering() {
        let engine = FingerprintEngine::from_rules(rules::default_rules());
        let mut probe = ProbeData::default();
        probe.http_headers.insert("Server".into(), "nginx".into());
        probe.http_body = Some("wp-content".into());

        let results = engine.match_all(&probe);
        assert!(results.windows(2).all(|w| w[0].confidence >= w[1].confidence));
    }
}
