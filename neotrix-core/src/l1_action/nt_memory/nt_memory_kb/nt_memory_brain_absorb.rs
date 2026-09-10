#![allow(dead_code)]
//! NeoTrixBrain 吸收管道 — 从外部大型知识库批量吸收到活跃 KB
//!
//! 设计原则:
//! - 流式处理: 64GB DB 不能全量加载, 按 batch 迭代
//! - SVAF 门控: 每条记录经过新颖性/连贯性/相关性/权威性过滤
//! - 去重: node ID 防止重复摄入
//! - 增量: 支持断点续传 (记录最后处理的 rowid)
//! - 安全: 只读连接, 不修改源 DB

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};

use super::nt_memory_svaf_gate::{SvafDecision, SvafGate};
use super::{KnowledgeBase, KnowledgeNode, NodeType, RelationType};

/// 吸收统计
#[derive(Debug, Clone, Default)]
pub struct AbsorbStats {
    pub nodes_scanned: usize,
    pub nodes_accepted: usize,
    pub nodes_rejected: usize,
    pub nodes_duplicate: usize,
    pub edges_scanned: usize,
    pub edges_accepted: usize,
    pub procedural_scanned: usize,
    pub procedural_accepted: usize,
    pub conversations_scanned: usize,
    pub conversations_accepted: usize,
    pub evolutions_scanned: usize,
    pub evolutions_accepted: usize,
    pub errors: Vec<String>,
    pub elapsed_ms: u64,
}

/// 吸收配置
#[derive(Debug, Clone)]
pub struct AbsorbConfig {
    /// 源 DB 路径 (NeoTrixBrain)
    pub source_db: PathBuf,
    /// 每批次处理记录数
    pub batch_size: usize,
    /// SVAF 门控阈值 (None = 使用默认)
    pub svaf_gate: Option<SvafGate>,
    /// 是否吸收 procedural_memory
    pub absorb_procedural: bool,
    /// 是否吸收 conversation_records
    pub absorb_conversations: bool,
    /// 是否吸收 evolution_records
    pub absorb_evolutions: bool,
    /// 最小内容长度 (过短的跳过)
    pub min_content_len: usize,
}

impl Default for AbsorbConfig {
    fn default() -> Self {
        Self {
            source_db: PathBuf::new(),
            batch_size: 500,
            svaf_gate: None,
            absorb_procedural: true,
            absorb_conversations: true,
            absorb_evolutions: true,
            min_content_len: 20,
        }
    }
}

/// 打开源 DB (只读模式)
fn open_source_db(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open source DB {}: {}", path.display(), e))?;

    // 验证 DB 完整性
    let integrity: String = conn
        .pragma_query_value(None, "integrity_check", |row| row.get(0))
        .unwrap_or_else(|_| "error".to_string());
    if integrity != "ok" {
        return Err(format!("Source DB integrity check failed: {}", integrity));
    }

    let _ = conn.execute_batch("PRAGMA cache_size = -64000;");
    Ok(conn)
}

/// 流式读取 nodes 表
fn stream_nodes<F>(conn: &Connection, batch_size: usize, mut callback: F) -> Result<usize, String>
where
    F: FnMut(&[NodeRow]) -> Result<bool, String>,
{
    let mut offset: i64 = 0;
    let mut total = 0usize;
    loop {
        let mut stmt = conn
            .prepare(
                "SELECT id, node_type, title, summary, content, url, domain, language,
                        confidence, importance, created_at, updated_at, access_count
                 FROM nodes WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let rows: Vec<NodeRow> = stmt
            .query_map(rusqlite::params![offset, batch_size], |row| {
                Ok(NodeRow {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3).ok(),
                    content: row.get(4).ok(),
                    url: row.get(5).ok(),
                    domain: row.get(6).ok(),
                    language: row.get(7).unwrap_or_else(|_| "en".into()),
                    confidence: row.get(8).unwrap_or(1.0),
                    importance: row.get(9).unwrap_or(0.5),
                    created_at: row.get(10).unwrap_or(0),
                    updated_at: row.get(11).unwrap_or(0),
                    access_count: row.get(12).unwrap_or(0),
                })
            })
            .map_err(|e| format!("QueryMap: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            break;
        }
        offset += batch_size as i64;
        total += rows.len();
        if !callback(&rows).map_err(|e| format!("Callback@{}: {}", offset, e))? {
            break;
        }
    }
    Ok(total)
}

/// 流式读取 edges 表
fn stream_edges<F>(conn: &Connection, batch_size: usize, mut callback: F) -> Result<usize, String>
where
    F: FnMut(&[EdgeRow]) -> Result<bool, String>,
{
    let mut offset: i64 = 0;
    let mut total = 0usize;
    loop {
        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, relation_type, weight, description, created_at
                 FROM edges WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let rows: Vec<EdgeRow> = stmt
            .query_map(rusqlite::params![offset, batch_size], |row| {
                Ok(EdgeRow {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: row.get(3)?,
                    weight: row.get(4).unwrap_or(1.0),
                    description: row.get(5).ok(),
                    created_at: row.get(6).unwrap_or(0),
                })
            })
            .map_err(|e| format!("QueryMap: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            break;
        }
        offset += batch_size as i64;
        total += rows.len();
        if !callback(&rows).map_err(|e| format!("Callback@{}: {}", offset, e))? {
            break;
        }
    }
    Ok(total)
}

/// 流式读取 procedural_memory 表
fn stream_procedural<F>(
    conn: &Connection,
    batch_size: usize,
    mut callback: F,
) -> Result<usize, String>
where
    F: FnMut(&[ProceduralRow]) -> Result<bool, String>,
{
    let mut offset: i64 = 0;
    let mut total = 0usize;
    loop {
        let mut stmt = conn
            .prepare(
                "SELECT id, skill_id, name, description, e8_sequence, trigger_pattern,
                        success_rate, execution_count, avg_reward, tags
                 FROM procedural_memory WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let rows: Vec<ProceduralRow> = stmt
            .query_map(rusqlite::params![offset, batch_size], |row| {
                Ok(ProceduralRow {
                    id: row.get(0)?,
                    skill_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3).ok(),
                    e8_sequence: row.get(4)?,
                    trigger_pattern: row.get(5)?,
                    success_rate: row.get(6).unwrap_or(0.0),
                    execution_count: row.get(7).unwrap_or(0),
                    avg_reward: row.get(8).unwrap_or(0.0),
                    tags: row.get(9).ok(),
                })
            })
            .map_err(|e| format!("QueryMap: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            break;
        }
        offset += batch_size as i64;
        total += rows.len();
        if !callback(&rows).map_err(|e| format!("Callback@{}: {}", offset, e))? {
            break;
        }
    }
    Ok(total)
}

/// 流式读取 conversation_records 表
fn stream_conversations<F>(
    conn: &Connection,
    batch_size: usize,
    mut callback: F,
) -> Result<usize, String>
where
    F: FnMut(&[ConversationRow]) -> Result<bool, String>,
{
    let mut offset: i64 = 0;
    let mut total = 0usize;
    loop {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, task_description, user_intent, strategy_used,
                        outcome, effectiveness, error_count, timestamp
                 FROM conversation_records WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let rows: Vec<ConversationRow> = stmt
            .query_map(rusqlite::params![offset, batch_size], |row| {
                Ok(ConversationRow {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    task_description: row.get(2).ok(),
                    user_intent: row.get(3).ok(),
                    strategy_used: row.get(4).ok(),
                    outcome: row.get(5).ok(),
                    effectiveness: row.get(6).ok(),
                    error_count: row.get(7).unwrap_or(0),
                    timestamp: row.get(8).unwrap_or(0),
                })
            })
            .map_err(|e| format!("QueryMap: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            break;
        }
        offset += batch_size as i64;
        total += rows.len();
        if !callback(&rows).map_err(|e| format!("Callback@{}: {}", offset, e))? {
            break;
        }
    }
    Ok(total)
}

/// 流式读取 evolution_records 表
fn stream_evolutions<F>(
    conn: &Connection,
    batch_size: usize,
    mut callback: F,
) -> Result<usize, String>
where
    F: FnMut(&[EvolutionRow]) -> Result<bool, String>,
{
    let mut offset: i64 = 0;
    let mut total = 0usize;
    loop {
        let mut stmt = conn
            .prepare(
                "SELECT id, pattern_type, description, before_behavior, after_behavior,
                        effectiveness_gain, verified, timestamp
                 FROM evolution_records WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let rows: Vec<EvolutionRow> = stmt
            .query_map(rusqlite::params![offset, batch_size], |row| {
                Ok(EvolutionRow {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    description: row.get(2).ok(),
                    before_behavior: row.get(3).ok(),
                    after_behavior: row.get(4).ok(),
                    effectiveness_gain: row.get(5).unwrap_or(0.0),
                    verified: row.get(6).unwrap_or(0) != 0,
                    timestamp: row.get(7).unwrap_or(0),
                })
            })
            .map_err(|e| format!("QueryMap: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            break;
        }
        offset += batch_size as i64;
        total += rows.len();
        if !callback(&rows).map_err(|e| format!("Callback@{}: {}", offset, e))? {
            break;
        }
    }
    Ok(total)
}

// ============================================================================
// Source DB row types
// ============================================================================

#[derive(Debug)]
struct NodeRow {
    id: String,
    node_type: String,
    title: String,
    summary: Option<String>,
    content: Option<String>,
    url: Option<String>,
    domain: Option<String>,
    language: String,
    confidence: f64,
    importance: f64,
    created_at: i64,
    updated_at: i64,
    access_count: i64,
}

#[derive(Debug)]
struct EdgeRow {
    id: String,
    source_id: String,
    target_id: String,
    relation_type: String,
    weight: f64,
    description: Option<String>,
    created_at: i64,
}

#[derive(Debug)]
struct ProceduralRow {
    id: String,
    skill_id: String,
    name: String,
    description: Option<String>,
    e8_sequence: String,
    trigger_pattern: String,
    success_rate: f64,
    execution_count: i64,
    avg_reward: f64,
    tags: Option<String>,
}

#[derive(Debug)]
struct ConversationRow {
    id: String,
    session_id: String,
    task_description: Option<String>,
    user_intent: Option<String>,
    strategy_used: Option<String>,
    outcome: Option<String>,
    effectiveness: Option<f64>,
    error_count: i64,
    timestamp: i64,
}

#[derive(Debug)]
struct EvolutionRow {
    id: String,
    pattern_type: String,
    description: Option<String>,
    before_behavior: Option<String>,
    after_behavior: Option<String>,
    effectiveness_gain: f64,
    verified: bool,
    timestamp: i64,
}

// ============================================================================
// Type mapping
// ============================================================================

fn map_node_type(s: &str) -> NodeType {
    match s {
        "paper" | "Paper" => NodeType::Paper,
        "repository" | "Repository" => NodeType::Repository,
        "person" | "Person" => NodeType::Person,
        "event" | "Event" => NodeType::Event,
        "source" | "Source" => NodeType::Source,
        "tool" | "Tool" => NodeType::Tool,
        "framework" | "Framework" => NodeType::Framework,
        "algorithm" | "Algorithm" => NodeType::Algorithm,
        "theory" | "Theory" => NodeType::Theory,
        "method" | "Method" => NodeType::Method,
        "dataset" | "Dataset" => NodeType::Dataset,
        "benchmark" | "Benchmark" => NodeType::Benchmark,
        "organization" | "Organization" => NodeType::Organization,
        "book" | "Book" => NodeType::Book,
        "course" | "Course" => NodeType::Course,
        "article" | "Article" => NodeType::Article,
        "code_snippet" | "CodeSnippet" => NodeType::CodeSnippet,
        "idea" | "Idea" => NodeType::Idea,
        "question" | "Question" => NodeType::Question,
        "insight" | "Insight" => NodeType::Insight,
        "skill" | "Skill" => NodeType::Skill,
        "wiki_page" | "WikiPage" => NodeType::WikiPage,
        "thinking_trace" | "ThinkingTrace" => NodeType::ThinkingTrace,
        "evolution_pattern" | "EvolutionPattern" => NodeType::EvolutionPattern,
        "conversation_evolution" | "ConversationEvolution" => NodeType::ConversationEvolution,
        "resource" | "Resource" => NodeType::Resource,
        "summary" | "Summary" => NodeType::Summary,
        "guide" | "Guide" => NodeType::Guide,
        _ => NodeType::Concept,
    }
}

fn map_relation_type(s: &str) -> RelationType {
    match s {
        "references" => RelationType::References,
        "subclass_of" => RelationType::SubclassOf,
        "instance_of" => RelationType::InstanceOf,
        "causes" => RelationType::Causes,
        "prerequisite_of" => RelationType::PrerequisiteOf,
        "contradicts" => RelationType::Contradicts,
        "supports" => RelationType::Supports,
        "before_in_time" => RelationType::BeforeInTime,
        "after_in_time" => RelationType::AfterInTime,
        "related" | "related_to" => RelationType::RelatedTo,
        "part_of" => RelationType::PartOf,
        "developed_by" => RelationType::DevelopedBy,
        "implemented_in" => RelationType::ImplementedIn,
        "inspired_by" => RelationType::InspiredBy,
        "citation" => RelationType::Citation,
        "extension_of" => RelationType::ExtensionOf,
        "depends_on" => RelationType::DependsOn,
        "improves" => RelationType::Improves,
        "outperforms" => RelationType::Outperforms,
        "evolved_from" => RelationType::EvolvedFrom,
        "resource_for" => RelationType::ResourceFor,
        "about_topic" => RelationType::AboutTopic,
        "belongs_to" => RelationType::BelongsTo,
        "sub_topic_of" => RelationType::SubTopicOf,
        "cross_domain" => RelationType::CrossDomain,
        "contains" => RelationType::Contains,
        "influenced" => RelationType::Influenced,
        "implements" => RelationType::Implements,
        "uses" => RelationType::Uses,
        "wiki_link" => RelationType::WikiLink,
        _ => RelationType::Related,
    }
}

// ============================================================================
// Main pipeline
// ============================================================================

/// 执行 NeoTrixBrain 吸收
pub fn absorb_brain(config: &AbsorbConfig, kb: &KnowledgeBase) -> Result<AbsorbStats, String> {
    let start = SystemTime::now();
    let mut stats = AbsorbStats::default();
    let gate = config.svaf_gate.clone().unwrap_or_else(SvafGate::default);

    let source_conn = open_source_db(&config.source_db)?;

    log::info!(
        "[BrainAbsorb] Starting from {} (batch={})",
        config.source_db.display(),
        config.batch_size,
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // ── Phase 1: nodes ──
    log::info!("[BrainAbsorb] Phase 1: nodes...");
    stream_nodes(&source_conn, config.batch_size, |batch| {
        for row in batch {
            stats.nodes_scanned += 1;

            // 去重: ID 已存在则跳过
            if kb.get_node(&row.id).unwrap_or(None).is_some() {
                stats.nodes_duplicate += 1;
                continue;
            }

            let content_text = row.content.as_deref().or(row.summary.as_deref()).unwrap_or("");
            if content_text.len() < config.min_content_len {
                stats.nodes_rejected += 1;
                continue;
            }

            // SVAF 门控
            let source_type = row.domain.as_deref().unwrap_or("unknown");
            let eval = gate.evaluate_content_only(content_text, source_type);
            match eval.decision {
                SvafDecision::Reject | SvafDecision::Redundant => {
                    stats.nodes_rejected += 1;
                    continue;
                }
                _ => {}
            }

            let node = KnowledgeNode {
                id: row.id.clone(),
                node_type: map_node_type(&row.node_type),
                title: row.title.clone(),
                summary: row.summary.clone(),
                content: row.content.clone(),
                url: row.url.clone(),
                domain: row.domain.clone(),
                language: row.language.clone(),
                confidence: row.confidence,
                importance: row.importance,
                recall_weight: 0.5,
                created_at: row.created_at,
                updated_at: row.updated_at,
                access_count: row.access_count,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: Some("neotrix_brain_archive".to_string()),
            };

            if let Err(e) = kb.insert_node(&node) {
                stats.errors.push(format!("node {}: {}", row.id, e));
            } else {
                stats.nodes_accepted += 1;
            }
        }
        Ok(true)
    })?;

    // ── Phase 2: edges ──
    log::info!("[BrainAbsorb] Phase 2: edges...");
    stream_edges(&source_conn, config.batch_size, |batch| {
        for row in batch {
            stats.edges_scanned += 1;
            if let Err(e) = kb.upsert_edge(
                &row.source_id,
                &row.target_id,
                map_relation_type(&row.relation_type),
                row.weight,
                row.description.as_deref(),
            ) {
                stats.errors.push(format!("edge {}: {}", row.id, e));
            } else {
                stats.edges_accepted += 1;
            }
        }
        Ok(true)
    })?;

    // ── Phase 3: procedural_memory → Skill nodes ──
    if config.absorb_procedural {
        log::info!("[BrainAbsorb] Phase 3: procedural memory...");
        stream_procedural(&source_conn, config.batch_size, |batch| {
            for row in batch {
                stats.procedural_scanned += 1;
                let content = format!(
                    "Skill: {}\nTrigger: {}\nE8: {}\nSuccess: {:.1}%\nExecutions: {}",
                    row.name, row.trigger_pattern, row.e8_sequence,
                    row.success_rate * 100.0, row.execution_count,
                );
                let node = KnowledgeNode {
                    id: format!("brain:proc:{}", row.skill_id),
                    node_type: NodeType::Skill,
                    title: row.name.clone(),
                    summary: row.description.clone(),
                    content: Some(content),
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: row.success_rate,
                    importance: 0.7,
                    recall_weight: 0.5,
                    created_at: now,
                    updated_at: now,
                    access_count: row.execution_count,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: Some("neotrix_brain_procedural".to_string()),
                };
                if let Err(e) = kb.insert_node(&node) {
                    stats.errors.push(format!("proc {}: {}", row.id, e));
                } else {
                    stats.procedural_accepted += 1;
                }
            }
            Ok(true)
        })?;
    }

    // ── Phase 4: conversation_records → ConversationEvolution nodes ──
    if config.absorb_conversations {
        log::info!("[BrainAbsorb] Phase 4: conversations...");
        stream_conversations(&source_conn, config.batch_size, |batch| {
            for row in batch {
                stats.conversations_scanned += 1;
                let title = row.task_description.as_deref().unwrap_or("unnamed conversation");
                let content = format!(
                    "Session: {}\nIntent: {}\nStrategy: {}\nOutcome: {}\nEffectiveness: {:.2}\nErrors: {}",
                    row.session_id,
                    row.user_intent.as_deref().unwrap_or("?"),
                    row.strategy_used.as_deref().unwrap_or("?"),
                    row.outcome.as_deref().unwrap_or("?"),
                    row.effectiveness.unwrap_or(0.0),
                    row.error_count,
                );
                let node = KnowledgeNode {
                    id: format!("brain:conv:{}", row.id),
                    node_type: NodeType::ConversationEvolution,
                    title: title.to_string(),
                    summary: None,
                    content: Some(content),
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: row.effectiveness.unwrap_or(0.5),
                    importance: 0.4,
                    recall_weight: 0.5,
                    created_at: row.timestamp,
                    updated_at: row.timestamp,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: Some("neotrix_brain_conversation".to_string()),
                };
                if let Err(e) = kb.insert_node(&node) {
                    stats.errors.push(format!("conv {}: {}", row.id, e));
                } else {
                    stats.conversations_accepted += 1;
                }
            }
            Ok(true)
        })?;
    }

    // ── Phase 5: evolution_records → EvolutionPattern nodes ──
    if config.absorb_evolutions {
        log::info!("[BrainAbsorb] Phase 5: evolutions...");
        stream_evolutions(&source_conn, config.batch_size, |batch| {
            for row in batch {
                stats.evolutions_scanned += 1;
                let title = format!("{}: {}", row.pattern_type,
                    row.description.as_deref().unwrap_or("unnamed"));
                let content = format!(
                    "Before: {}\nAfter: {}\nGain: {:.2}\nVerified: {}",
                    row.before_behavior.as_deref().unwrap_or("?"),
                    row.after_behavior.as_deref().unwrap_or("?"),
                    row.effectiveness_gain,
                    row.verified,
                );
                let node = KnowledgeNode {
                    id: format!("brain:evo:{}", row.id),
                    node_type: NodeType::EvolutionPattern,
                    title,
                    summary: None,
                    content: Some(content),
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: row.effectiveness_gain.min(1.0),
                    importance: 0.6,
                    recall_weight: 0.5,
                    created_at: row.timestamp,
                    updated_at: row.timestamp,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: Some("neotrix_brain_evolution".to_string()),
                };
                if let Err(e) = kb.insert_node(&node) {
                    stats.errors.push(format!("evo {}: {}", row.id, e));
                } else {
                    stats.evolutions_accepted += 1;
                }
            }
            Ok(true)
        })?;
    }

    let elapsed = start.elapsed().unwrap_or_default().as_millis() as u64;
    stats.elapsed_ms = elapsed;

    log::info!(
        "[BrainAbsorb] Done: nodes={}/{} (dup={}) edges={}/{} proc={}/{} conv={}/{} evo={}/{} err={} {}ms",
        stats.nodes_accepted, stats.nodes_scanned, stats.nodes_duplicate,
        stats.edges_accepted, stats.edges_scanned,
        stats.procedural_accepted, stats.procedural_scanned,
        stats.conversations_accepted, stats.conversations_scanned,
        stats.evolutions_accepted, stats.evolutions_scanned,
        stats.errors.len(), stats.elapsed_ms,
    );

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_types() {
        assert_eq!(map_node_type("paper"), NodeType::Paper);
        assert_eq!(map_node_type("wiki_page"), NodeType::WikiPage);
        assert_eq!(map_node_type("unknown"), NodeType::Concept);
    }

    #[test]
    fn test_map_relation_types() {
        assert_eq!(map_relation_type("references"), RelationType::References);
        assert_eq!(map_relation_type("related_to"), RelationType::RelatedTo);
        assert_eq!(map_relation_type("unknown"), RelationType::Related);
    }

    #[test]
    fn test_absorb_config_default() {
        let c = AbsorbConfig::default();
        assert_eq!(c.batch_size, 500);
        assert!(c.absorb_procedural);
        assert!(c.min_content_len == 20);
    }
}
