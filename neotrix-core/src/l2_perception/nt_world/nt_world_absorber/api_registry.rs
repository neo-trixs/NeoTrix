//! API/服务注册表元数据 (G30, public-apis / free-for-dev 吸收)。
//!
//! 提供公共 API 注册表 schema (与 public-apis 社区维护格式对齐) 与分类目标树,
//! 输出可直接作为 `nt_agent_mcp_registry` 的发现种子 — 吸收器落地的新知识
//! 节点经此 schema 归一, 供 MCP registry 索引为可调用工具面。

use serde::{Deserialize, Serialize};

/// 单个 API 条目 schema (public-apis 对齐)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiEntry {
    /// API 名称 (如 "OpenAI")。
    pub name: String,
    /// 描述。
    pub description: String,
    /// 认证方式: "" / "apiKey" / "OAuth" / "X-Mashape-Key" 等。
    pub auth: String,
    /// 是否 HTTPS。
    pub https: bool,
    /// 分类 (映射到分类目标树叶子)。
    pub category: String,
    /// 是否 CORS 支持。
    pub cors: bool,
    /// 链接。
    pub url: String,
}

/// 分类目标树节点 — 吸收发现的目标类别层级。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiCategoryNode {
    pub name: String,
    pub children: Vec<ApiCategoryNode>,
    /// 是否为叶子 (可直接挂 API)。
    pub leaf: bool,
}

impl ApiCategoryNode {
    /// 构建一棵叶子分类目标树 (public-apis 大类)。
    pub fn default_tree() -> Vec<ApiCategoryNode> {
        [
            "Development", "Machine Learning", "Data", "Media",
            "Science & Math", "News", "Finance", "Weather",
        ]
        .iter()
        .map(|c| ApiCategoryNode {
            name: c.to_string(),
            children: Vec::new(),
            leaf: true,
        })
        .collect()
    }

    /// 收集所有叶子分类名。
    pub fn leaf_names(nodes: &[ApiCategoryNode]) -> Vec<String> {
        let mut out = Vec::new();
        for n in nodes {
            if n.leaf && n.children.is_empty() {
                out.push(n.name.clone());
            } else {
                out.extend(Self::leaf_names(&n.children));
            }
        }
        out
    }
}

/// API 注册表 — 发现种子源: 提供按分类检索 + 种子 URL 生成。
#[derive(Debug, Clone, Default)]
pub struct ApiRegistry {
    entries: Vec<ApiEntry>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 API 条目 (同 name 去重更新)。
    pub fn register(&mut self, entry: ApiEntry) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.name == entry.name) {
            *e = entry;
            return;
        }
        self.entries.push(entry);
    }

    /// 按分类检索。
    pub fn by_category(&self, category: &str) -> Vec<&ApiEntry> {
        self.entries.iter().filter(|e| e.category == category).collect()
    }

    /// 把整个条目集作为发现种子 URL 列表输出 (供 crawl queue 使用)。
    pub fn seed_urls(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.url.clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 汇总统计: (总条目, HTTPS 比例, 认证类型分布前 3)。
    pub fn stats(&self) -> (usize, f64, Vec<(String, usize)>) {
        let n = self.entries.len();
        let https = if n == 0 {
            0.0
        } else {
            self.entries.iter().filter(|e| e.https).count() as f64 / n as f64
        };
        let mut auths: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for e in &self.entries {
            let key = if e.auth.is_empty() { "none" } else { &e.auth };
            *auths.entry(key.to_string()).or_insert(0) += 1;
        }
        let mut sorted: Vec<(String, usize)> = auths.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        (n, https, sorted.into_iter().take(3).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ApiEntry {
        ApiEntry {
            name: "OpenAI".into(),
            description: "AI models API".into(),
            auth: "apiKey".into(),
            https: true,
            category: "Machine Learning".into(),
            cors: true,
            url: "https://api.openai.com".into(),
        }
    }

    #[test]
    fn test_register_dedup_by_name() {
        let mut reg = ApiRegistry::new();
        reg.register(sample());
        reg.register(sample());
        assert_eq!(reg.len(), 1, "same-name entries dedupe");
    }

    #[test]
    fn test_by_category_and_seed_urls() {
        let mut reg = ApiRegistry::new();
        reg.register(sample());
        reg.register(ApiEntry {
            name: "GitHub".into(),
            category: "Development".into(),
            url: "https://api.github.com".into(),
            ..sample()
        });
        assert_eq!(reg.by_category("Machine Learning").len(), 1);
        assert_eq!(reg.by_category("Development").len(), 1);
        let seeds = reg.seed_urls();
        assert!(seeds.contains(&"https://api.openai.com".to_string()));
        assert!(seeds.contains(&"https://api.github.com".to_string()));
    }

    #[test]
    fn test_default_category_tree_leaves() {
        let tree = ApiCategoryNode::default_tree();
        let leaves = ApiCategoryNode::leaf_names(&tree);
        assert!(leaves.contains(&"Machine Learning".to_string()));
        assert!(leaves.contains(&"Development".to_string()));
        assert!(leaves.contains(&"Finance".to_string()));
    }

    #[test]
    fn test_stats_https_ratio_and_auth_distribution() {
        let mut reg = ApiRegistry::new();
        reg.register(sample()); // https=true, apiKey
        reg.register(ApiEntry {
            name: "P".into(),
            https: false,
            auth: "apiKey".into(),
            ..sample()
        });
        reg.register(ApiEntry {
            name: "Q".into(),
            auth: "".into(),
            ..sample()
        });
        let (n, https, auths) = reg.stats();
        assert_eq!(n, 3);
        assert!((https - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(auths[0].0, "apiKey");
    }
}