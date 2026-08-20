#![forbid(unsafe_code)]

//! ARTEX 双图分离 (吸收 Autumn-27/ARTEX, R-P79 生产接线)。
//!
//! 双图:
//! - **资产图 (Asset Graph)**: 全局共享真值。URL → root_domain → subdomain →
//!   service(path 首段) → endpoint(完整路径) 的层级, 落 KB nodes/edges
//!   (PartOf/BelongsTo 关系), 供全 agent 共享查询。
//! - **探索图 (Exploration Graph)**: 每 agent/scope 局部。独立 seen 集合 +
//!   独立优先级队列, 回答"谁探索到哪块", 隔离跨 agent 状态污染。
//!
//! 接线: UnifiedCrawler 每吸收一页即更新资产图; 探索图由 scope 隔离的
//! AssetScopeFrontier 提供, 替换全局单 seen (缺陷#1/#2/#3 修复)。
//!
//! 现有 KB 有 nodes/edges + subgraph/GraphCache 基建 (nt_memory_graph),
//! 复用 NodeType::Source/Concept + RelationType::PartOf/BelongsTo, 无需新表。

use std::collections::{HashMap, HashSet};

use crate::neotrix::nt_memory_kb::{KnowledgeBase, NodeType, RelationType};

/// 资产层级: 从 URL 解析出 root_domain / subdomain / service / endpoint。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetHierarchy {
    /// 顶层域, 如 "example.com"
    pub root_domain: String,
    /// 完整主机 (含子域), 如 "docs.example.com"
    pub full_host: String,
    /// service = 路径首段 (无时为空串), 如 "api"
    pub service: String,
    /// endpoint = 完整路径 (无时为空串), 如 "/v1/users"
    pub endpoint: String,
    /// 原始 URL
    pub url: String,
}

/// 从 URL 解析资产层级。仅处理 http/https; 解析失败返回 None。
pub fn parse_asset_hierarchy(url: &str) -> Option<AssetHierarchy> {
    let lower = url.trim().to_lowercase();
    let (scheme, rest) = if let Some(r) = lower.strip_prefix("http://") {
        ("http", r)
    } else if let Some(r) = lower.strip_prefix("https://") {
        ("https", r)
    } else {
        return None;
    };
    let _ = scheme;
    let (host, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{}", p)),
        None => (rest, String::new()),
    };
    if host.is_empty() {
        return None;
    }
    // 去端口
    let host = host.split(':').next().unwrap_or(host).to_string();
    // root_domain: 取最后两级
    let parts: Vec<&str> = host.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    let root_domain = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    let service = path.split('/').nth(1).unwrap_or("").to_string();
    Some(AssetHierarchy {
        root_domain,
        full_host: host,
        service,
        endpoint: path,
        url: url.to_string(),
    })
}

/// 资产图写入器: 把 URL 资产层级落 KB (nodes + edges)。
/// 节点名以 `asset:{host}` / `asset:{host}/{service}` 形式唯一化, upsert 幂等。
pub struct AssetGraphWriter;

impl AssetGraphWriter {
    /// 写入一条 URL 的资产层级。返回创建的边数 (重复调用返回 0)。
    /// 层级: root_domain → full_host → [service] → endpoint。
    pub fn ingest_url(kb: &KnowledgeBase, url: &str) -> Result<usize, String> {
        let Some(h) = parse_asset_hierarchy(url) else {
            return Ok(0);
        };
        let mut edges = 0;
        let root_id = kb.insert_or_get_node(
            &format!("asset:{}", h.root_domain),
            NodeType::Concept,
            Some(&format!("Root domain asset: {}", h.root_domain)),
            None,
            Some(&h.root_domain),
        )?;

        let host_id = kb.insert_or_get_node(
            &format!("asset:{}", h.full_host),
            NodeType::Source,
            Some(&format!("Host asset: {}", h.full_host)),
            Some(url),
            Some(&h.full_host),
        )?;
        if host_id != root_id {
            if !kb.edge_exists(&root_id, &host_id, RelationType::PartOf)? {
                kb.upsert_edge(&root_id, &host_id, RelationType::PartOf, 1.0, None)?;
                edges += 1;
            }
        }

        if !h.service.is_empty() {
            let service_id = kb.insert_or_get_node(
                &format!("asset:{}/{}", h.full_host, h.service),
                NodeType::Source,
                Some(&format!("Service asset: {}/{}", h.full_host, h.service)),
                None,
                Some(&h.full_host),
            )?;
            if service_id != host_id {
                if !kb.edge_exists(&host_id, &service_id, RelationType::PartOf)? {
                    kb.upsert_edge(&host_id, &service_id, RelationType::PartOf, 1.0, None)?;
                    edges += 1;
                }
            }
            if !h.endpoint.is_empty() && h.endpoint != format!("/{}", h.service) {
                let ep_id = kb.insert_or_get_node(
                    &format!("asset:{}", h.endpoint.trim_end_matches('/')),
                    NodeType::Concept,
                    Some(&format!("Endpoint: {}", h.endpoint)),
                    Some(url),
                    Some(&h.full_host),
                )?;
                if ep_id != service_id {
                    if !kb.edge_exists(&service_id, &ep_id, RelationType::BelongsTo)? {
                        kb.upsert_edge(&service_id, &ep_id, RelationType::BelongsTo, 1.0, None)?;
                        edges += 1;
                    }
                }
            }
        }
        Ok(edges)
    }
}

/// 探索图: 每 scope 独立的 seen + 待爬优先级队列。
/// 隔离跨 agent/任务的状态 (原 DualQueueFrontier.seen 全局单集 → 缺陷#1 修复)。
pub struct ScopeFrontier {
    /// scope 标识 (agent_id / task_id / 域名组)
    pub scope: String,
    seen: HashSet<String>,
    /// 优先级队列: (priority, url)
    queue: Vec<(u8, String)>,
}

impl ScopeFrontier {
    pub fn new(scope: &str) -> Self {
        Self { scope: scope.to_string(), seen: HashSet::new(), queue: Vec::new() }
    }

    /// 入队 (去重; 已在 seen 的忽略)。返回是否新入队。
    pub fn enqueue(&mut self, url: &str, priority: u8) -> bool {
        let normalized = normalize_url(url);
        if self.seen.contains(&normalized) {
            return false;
        }
        self.seen.insert(normalized.clone());
        self.queue.push((priority, normalized));
        true
    }

    /// 按优先级降序弹出。空返回 None。
    pub fn pop(&mut self) -> Option<String> {
        if self.queue.is_empty() {
            return None;
        }
        self.queue.sort_by(|a, b| b.0.cmp(&a.0));
        Some(self.queue.remove(0).1)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_seen(&self, url: &str) -> bool {
        self.seen.contains(&normalize_url(url))
    }
}

/// URL 规范化: 去 fragment + 尾斜杠 (缺陷#6 去重无指纹的轻量修复)。
fn normalize_url(url: &str) -> String {
    let without_frag = url.split('#').next().unwrap_or(url);
    let trimmed = without_frag.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

/// 多 scope 探索图管理器: scope → ScopeFrontier。
#[derive(Default)]
pub struct ExplorationGraph {
    frontiers: HashMap<String, ScopeFrontier>,
}

impl ExplorationGraph {
    pub fn new() -> Self {
        Self { frontiers: HashMap::new() }
    }

    pub fn frontier(&mut self, scope: &str) -> &mut ScopeFrontier {
        self.frontiers.entry(scope.to_string()).or_insert_with(|| ScopeFrontier::new(scope))
    }

    pub fn scopes(&self) -> Vec<&str> {
        self.frontiers.keys().map(|s| s.as_str()).collect()
    }

    /// 全 scope 待爬总数。
    pub fn total_pending(&self) -> usize {
        self.frontiers.values().map(|f| f.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hierarchy_basic() {
        let h = parse_asset_hierarchy("https://docs.example.com/api/v1/users").expect("parse");
        assert_eq!(h.root_domain, "example.com");
        assert_eq!(h.full_host, "docs.example.com");
        assert_eq!(h.service, "api");
        assert_eq!(h.endpoint, "/api/v1/users");
    }

    #[test]
    fn test_parse_hierarchy_root() {
        let h = parse_asset_hierarchy("https://example.com").expect("parse");
        assert_eq!(h.root_domain, "example.com");
        assert_eq!(h.service, "");
        assert_eq!(h.endpoint, "");
    }

    #[test]
    fn test_parse_rejects_non_http() {
        assert!(parse_asset_hierarchy("ftp://x.com/f").is_none());
        assert!(parse_asset_hierarchy("").is_none());
        assert!(parse_asset_hierarchy("not a url").is_none());
    }

    #[test]
    fn test_scope_frontier_dedup_and_priority() {
        let mut f = ScopeFrontier::new("agent_a");
        assert!(f.enqueue("https://a.com/x", 2));
        assert!(f.enqueue("https://a.com/y#frag", 5));
        assert!(!f.enqueue("https://a.com/x", 2), "重复应拒绝");
        assert_eq!(f.len(), 2);
        // 优先级 5 的先出
        assert_eq!(f.pop().as_deref(), Some("https://a.com/y"));
    }

    #[test]
    fn test_normalize_strips_fragment_and_trailing_slash() {
        assert_eq!(normalize_url("https://a.com/x#sec"), "https://a.com/x");
        assert_eq!(normalize_url("https://a.com/x/"), "https://a.com/x");
        assert_eq!(normalize_url("https://a.com/"), "https://a.com");
    }

    #[test]
    fn test_exploration_graph_isolates_scopes() {
        let mut g = ExplorationGraph::new();
        g.frontier("agent_a").enqueue("https://a.com/1", 1);
        g.frontier("agent_b").enqueue("https://b.com/1", 1);
        assert_eq!(g.total_pending(), 2);
        assert!(!g.frontier("agent_a").is_seen("https://b.com/1"), "scope 应隔离");
        assert_eq!(g.scopes().len(), 2);
    }

    #[test]
    fn test_asset_graph_writer_idempotent() {
        let dir = std::env::temp_dir().join(format!("nt_artex_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let db = dir.join("artex.db");
        let _ = std::fs::remove_file(&db);
        let kb = KnowledgeBase::open(Some(db.clone())).expect("open kb");
        let edges1 = AssetGraphWriter::ingest_url(&kb, "https://docs.example.com/api/v1/users").expect("ingest");
        assert!(edges1 >= 2, "应有 host→service 和 service→endpoint 边, got {edges1}");
        // 幂等: 重复 ingest 不再新增边
        let edges2 = AssetGraphWriter::ingest_url(&kb, "https://docs.example.com/api/v1/users").expect("ingest");
        assert_eq!(edges2, 0, "幂等应返回 0");
        // 不同 endpoint 新增边
        let edges3 = AssetGraphWriter::ingest_url(&kb, "https://docs.example.com/api/v2/orders").expect("ingest");
        assert!(edges3 >= 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}