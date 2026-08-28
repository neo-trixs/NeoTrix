//! reasoning_engine — 推理引擎接线 (SESSION C)
//!
//! Connects `causal_rules` (170 rows) + `reasoning_chains` (30 rows) from the
//! KB to forward / backward / abductive reasoning over KB nodes & edges.
//!
//! 受控边界: 仅消费 `crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase`
//! (core 逻辑核心消费 neotrix 层基础设施, 与 nt_core_reasoning 同边界). 零 unsafe.

use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 推理沿用的边关系类型 (KB `edges.relation_type`).
/// 这些为领域专用关系, 以字符串形态直接匹配 edges 表, 不依赖 RelationType 枚举变体.
pub const REASONING_RELATIONS: &[&str] = &[
    "leads_to",
    "synthesizes",
    "analogizes",
    "counterfactual_of",
];

/// KB `causal_rules` 表单行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRule {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub outcome: String,
    pub confidence: f64,
}

/// KB `reasoning_chains` 表单行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChainRecord {
    pub id: String,
    pub chain_type: String,
    pub premise_ids: Vec<String>,
    pub conclusion_id: String,
    pub rule_ids: Vec<String>,
}

/// 推理链中的单步节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStepNode {
    pub node_id: String,
    pub label: String,
    pub relation: Option<String>,
    pub confidence: f64,
}

/// 一条边的遍历记录 (内部)
#[derive(Debug, Clone)]
pub struct EdgeStep {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub mode: String,
    pub query: String,
    pub steps: Vec<ReasoningStepNode>,
    pub conclusion: Option<String>,
    pub confidence: f64,
    pub rules_used: Vec<CausalRule>,
}

/// 链执行器 — 持有 KB 句柄, 驱动三种推理模式.
pub struct ChainExecutor {
    kb: Arc<KnowledgeBase>,
}

impl ChainExecutor {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self { kb }
    }

    pub fn kb(&self) -> &Arc<KnowledgeBase> {
        &self.kb
    }

    /// 前向推理: 从 premise 沿因果规则与推理边推导出结论.
    pub fn execute_forward(&self, premise: &str, max_depth: u8) -> ReasoningResult {
        super::forward::forward(self, premise, max_depth)
    }

    /// 反向推理: 从 goal 反推支撑 premise.
    pub fn execute_backward(&self, goal: &str) -> ReasoningResult {
        super::backward::backward(self, goal)
    }

    /// 归因推理: 为 observation 找出最佳解释 (因果规则 condition).
    pub fn execute_abductive(&self, observation: &str) -> ReasoningResult {
        super::abductive::abductive(self, observation)
    }

    // ─── 共享 KB 访问原语 ────────────────────────────────────────────────

    /// FTS 搜索 premise 相关节点, 失败回退 LIKE 查询.
    pub(crate) fn find_nodes(&self, q: &str, limit: usize) -> Vec<ReasoningStepNode> {
        let lower = q.to_lowercase();
        let mut out = Vec::new();

        if let Ok(results) = self.kb.search(q, limit) {
            for r in results {
                out.push(ReasoningStepNode {
                    node_id: r.node.id.clone(),
                    label: r.node.title.clone(),
                    relation: None,
                    confidence: r.score,
                });
            }
        }

        if out.is_empty() {
            if let Ok(conn) = self.kb.raw_conn() {
                let mut stmt = match conn.prepare(
                    "SELECT id, title FROM nodes \
                     WHERE lower(title) LIKE ?1 \
                        OR lower(coalesce(summary,'')) LIKE ?1 \
                        OR lower(coalesce(content,'')) LIKE ?1 \
                     LIMIT ?2",
                ) {
                    Ok(s) => s,
                    Err(_) => return out,
                };
                let pat = format!("%{}%", lower);
                let mut rows = match stmt.query(rusqlite::params![pat.as_str(), limit as i64]) {
                    Ok(r) => r,
                    Err(_) => return out,
                };
                while let Ok(Some(row)) = rows.next() {
                        let id: String = row.get(0).unwrap_or_default();
                        let title: String = row.get(1).unwrap_or_default();
                        out.push(ReasoningStepNode {
                            node_id: id,
                            label: title,
                            relation: None,
                            confidence: 0.5,
                        });
                    }
            }
        }
        out
    }

    /// 根据 id 取节点标题.
    pub(crate) fn node_label(&self, id: &str) -> String {
        self.kb
            .get_node(id)
            .ok()
            .flatten()
            .map(|n| n.title)
            .unwrap_or_else(|| id.to_string())
    }

    /// 读取全部 causal_rules (缺失表时返回空).
    pub(crate) fn query_causal_rules(&self) -> Vec<CausalRule> {
        let conn = match self.kb.raw_conn() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare(
            "SELECT rowid, condition, action, outcome, confidence FROM causal_rules",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            Ok(CausalRule {
                id: format!("cr_{}", row.get::<_, i64>(0).unwrap_or_default()),
                condition: row.get::<_, String>(1).unwrap_or_default(),
                action: row.get::<_, String>(2).unwrap_or_default(),
                outcome: row.get::<_, String>(3).unwrap_or_default(),
                confidence: row.get::<_, f64>(4).unwrap_or(0.0),
            })
        });
        match rows {
            Ok(r) => r.flatten().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 读取全部 reasoning_chains (缺失表时返回空).
    #[allow(dead_code)]
    pub(crate) fn query_reasoning_chains(&self) -> Vec<ReasoningChainRecord> {
        let conn = match self.kb.raw_conn() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare(
            "SELECT rowid, chain_type, premise_ids, conclusion_id, rule_ids FROM reasoning_chains",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            let premise_raw: String = row.get(2).unwrap_or_default();
            let rule_raw: String = row.get(4).unwrap_or_default();
            Ok(ReasoningChainRecord {
                id: format!("rc_{}", row.get::<_, i64>(0).unwrap_or_default()),
                chain_type: row.get(1).unwrap_or_default(),
                premise_ids: parse_id_list(&premise_raw),
                conclusion_id: row.get(3).unwrap_or_default(),
                rule_ids: parse_id_list(&rule_raw),
            })
        });
        match rows {
            Ok(r) => r.flatten().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 从 start_ids 正向遍历推理边 (source -> target).
    pub(crate) fn traverse_edges(&self, start_ids: &[String], max_depth: u8) -> Vec<EdgeStep> {
        let conn = match self.kb.raw_conn() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        if start_ids.is_empty() {
            return Vec::new();
        }
        let rel_list = REASONING_RELATIONS
            .iter()
            .map(|r| format!("'{}'", r))
            .collect::<Vec<_>>()
            .join(",");
        let start_list = start_ids
            .iter()
            .map(|s| format!("'{}'", s.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT source_id, target_id, relation_type, weight FROM edges \
             WHERE relation_type IN ({}) \
               AND (source_id IN ({}) OR target_id IN ({})) \
             LIMIT 2000",
            rel_list, start_list, start_list
        );
        collect_edge_steps(&conn, &sql, max_depth)
    }

    /// 反向遍历推理边 (target -> source), 用于 backward 反推 premise.
    pub(crate) fn traverse_edges_rev(&self, goal_ids: &[String], max_depth: u8) -> Vec<EdgeStep> {
        let conn = match self.kb.raw_conn() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        if goal_ids.is_empty() {
            return Vec::new();
        }
        let rel_list = REASONING_RELATIONS
            .iter()
            .map(|r| format!("'{}'", r))
            .collect::<Vec<_>>()
            .join(",");
        let goal_list = goal_ids
            .iter()
            .map(|s| format!("'{}'", s.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT source_id, target_id, relation_type, weight FROM edges \
             WHERE relation_type IN ({}) \
               AND target_id IN ({}) \
             LIMIT 2000",
            rel_list, goal_list
        );
        collect_edge_steps(&conn, &sql, max_depth)
    }
}

/// 执行边 SQL 并收集 EdgeStep, 受 max_depth 截断.
fn collect_edge_steps(conn: &Connection, sql: &str, max_depth: u8) -> Vec<EdgeStep> {
    let mut steps = Vec::new();
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(EdgeStep {
                from: row.get(0)?,
                to: row.get(1)?,
                relation: row.get(2)?,
                weight: row.get(3).unwrap_or(0.0),
            })
        }) {
            for e in rows.flatten() {
                steps.push(e);
            }
        }
    }
    if (max_depth as usize) < steps.len() {
        steps.truncate(max_depth as usize);
    }
    steps
}

/// 解析 id 列表 (JSON 数组优先, 否则逗号分隔).
#[allow(dead_code)]
fn parse_id_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return arr;
    }
    trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
    use std::path::PathBuf;

    fn test_kb() -> Arc<KnowledgeBase> {
        let kb = KnowledgeBase::open(Some(PathBuf::from(":memory:"))).expect("in-memory KB");
        let conn = kb.raw_conn().expect("conn");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS causal_rules \
             (id INTEGER PRIMARY KEY, condition TEXT, action TEXT, outcome TEXT, confidence REAL);
             CREATE TABLE IF NOT EXISTS reasoning_chains \
             (id INTEGER PRIMARY KEY, chain_type TEXT, premise_ids TEXT, conclusion_id TEXT, rule_ids TEXT);
             INSERT INTO causal_rules (condition, action, outcome, confidence) VALUES \
             ('rust unsafe', 'forbid', 'memory safe', 0.9), \
             ('slow build', 'cache', 'fast build', 0.7);
             INSERT INTO reasoning_chains (chain_type, premise_ids, conclusion_id, rule_ids) VALUES \
             ('deductive', '[\"n1\"]', 'n2', '[\"cr_1\"]');",
        )
        .expect("seed");
        drop(conn);
        Arc::new(kb)
    }

    #[test]
    fn test_query_causal_rules_reads_rows() {
        let ce = ChainExecutor::new(test_kb());
        let rules = ce.query_causal_rules();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].condition, "rust unsafe");
        assert!((rules[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_query_reasoning_chains_parses_ids() {
        let ce = ChainExecutor::new(test_kb());
        let chains = ce.query_reasoning_chains();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].chain_type, "deductive");
        assert_eq!(chains[0].premise_ids, vec!["n1".to_string()]);
        assert_eq!(chains[0].rule_ids, vec!["cr_1".to_string()]);
    }

    #[test]
    fn test_parse_id_list_variants() {
        assert_eq!(parse_id_list("[\"a\",\"b\"]"), vec!["a", "b"]);
        assert_eq!(parse_id_list("a, b ,c"), vec!["a", "b", "c"]);
        assert!(parse_id_list("").is_empty());
    }

    #[test]
    fn test_execute_forward_empty_premise_is_safe() {
        let ce = ChainExecutor::new(test_kb());
        let r = ce.execute_forward("nonexistent topic xyz", 4);
        assert_eq!(r.mode, "forward");
        // 无种子节点 -> 置信度 0, 不 panic
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn test_execute_abductive_picks_best_rule() {
        let ce = ChainExecutor::new(test_kb());
        let r = ce.execute_abductive("memory safe");
        assert_eq!(r.mode, "abductive");
        assert!(!r.rules_used.is_empty());
        assert_eq!(r.conclusion.as_deref(), Some("rust unsafe"));
        assert!((r.confidence - 0.9).abs() < 1e-9);
    }
}
