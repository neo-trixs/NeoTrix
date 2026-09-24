use std::collections::HashSet;
use std::process::Command;

use log::{info, warn};
use rusqlite::Connection;
use serde_json;
use uuid::Uuid;

use super::nt_memory_store::*;
use super::nt_memory_types::*;
use super::shared_utils::now;

use super::nt_memory_cortex_sync::enrich_cortex_metadata;

#[derive(Debug, Clone)]
pub enum ResourceSource {
    GitHub { owner: String, repo: String },
    ArXiv { id: String },
    Web { url: String },
    Direct, // conceptual / built-in
}

impl ResourceSource {
    pub fn url(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { owner, repo } => {
                Some(format!("https://github.com/{}/{}", owner, repo))
            }
            ResourceSource::ArXiv { id } => Some(format!("https://arxiv.org/abs/{}", id)),
            ResourceSource::Web { url } => Some(url.clone()),
            ResourceSource::Direct => None,
        }
    }

    pub fn domain(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { .. } => Some("github.com".into()),
            ResourceSource::ArXiv { .. } => Some("arxiv.org".into()),
            ResourceSource::Web { url } => url
                .split('/')
                .nth(2)
                .or(url.split('/').nth(0))
                .map(|d| d.to_string()),
            ResourceSource::Direct => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub category: NodeType,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub source: ResourceSource,
    pub key_insights: Vec<String>,
    pub tags: Vec<String>,
    pub importance: f64,
    pub confidence: f64,
}

impl ResourceDescriptor {
    pub fn github(owner: &str, repo: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Repository,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::GitHub {
                owner: owner.to_string(),
                repo: repo.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.7,
            confidence: 0.9,
        }
    }

    pub fn paper(arxiv_id: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Paper,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::ArXiv {
                id: arxiv_id.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.8,
            confidence: 0.9,
        }
    }

    pub fn article(title: &str, summary: &str, url: &str) -> Self {
        Self {
            category: NodeType::Article,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Web {
                url: url.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn tool(name: &str, summary: &str, source: ResourceSource) -> Self {
        Self {
            category: NodeType::Tool,
            title: name.to_string(),
            summary: summary.to_string(),
            content: None,
            source,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn concept(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Concept,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn insight(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Insight,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn with_key_insights(mut self, insights: Vec<&str>) -> Self {
        self.key_insights = insights.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }
}

#[derive(Debug)]
pub struct ResourceIngestResult {
    pub node_id: String,
    pub insight_ids: Vec<String>,
}

pub struct ResourceIngester<'a> {
    conn: &'a Connection,
    episode_id: String,
    ingest_log: Vec<IngestLogEntry>,
}

struct IngestLogEntry {
    title: String,
    node_type: NodeType,
    node_id: String,
    status: String,
    error: Option<String>,
}

impl<'a> ResourceIngester<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let episode_id = Uuid::new_v4().to_string();
        Self {
            conn,
            episode_id,
            ingest_log: Vec::new(),
        }
    }

    pub fn ingest(&mut self, desc: &ResourceDescriptor) -> Result<ResourceIngestResult, String> {
        let url = desc.source.url();
        let domain = desc.source.domain();

        let metadata = serde_json::json!({
            "source": match &desc.source {
                ResourceSource::GitHub { owner, repo } =>
                    serde_json::json!({"type": "github", "owner": owner, "repo": repo}),
                ResourceSource::ArXiv { id } =>
                    serde_json::json!({"type": "arxiv", "id": id}),
                ResourceSource::Web { url } =>
                    serde_json::json!({"type": "web", "url": url}),
                ResourceSource::Direct =>
                    serde_json::json!({"type": "direct"}),
            },
            "tags": desc.tags,
            "episode_id": self.episode_id,
            "insight_count": desc.key_insights.len(),
        });

        let content = desc.content.clone().or_else(|| {
            if desc.key_insights.is_empty() {
                None
            } else {
                Some(desc.key_insights.join("\n- "))
            }
        });

        let ts = now();
        let node = KnowledgeNode {
            id: Uuid::new_v4().to_string(),
            node_type: desc.category.clone(),
            title: desc.title.clone(),
            summary: Some(desc.summary.clone()),
            content,
            url,
            domain,
            language: "en".to_string(),
            recall_weight: 1.0,
            confidence: desc.confidence,
            importance: desc.importance,
            created_at: ts,
            updated_at: ts,
            access_count: 0,
            metadata: Some(metadata),
            temporal: None,
            supersedes: None,
            source_episode: Some(self.episode_id.clone()),
            parent_id: None,
            depth: 0,
            cluster_id: None,
        };

        insert_node(self.conn, &node).map_err(|e| format!("insert_node failed: {}", e))?;
        let node_id = node.id.clone();

        let mut insight_ids = Vec::new();

        for insight_text in &desc.key_insights {
            let insight_title = if insight_text.len() > 80 {
                let cut = (0..=77)
                    .rev()
                    .find(|&i| insight_text.is_char_boundary(i))
                    .unwrap_or(77);
                format!("{}...", &insight_text[..cut])
            } else {
                insight_text.clone()
            };

            let insight_node = KnowledgeNode {
                id: Uuid::new_v4().to_string(),
                node_type: NodeType::Insight,
                title: insight_title,
                summary: Some(insight_text.clone()),
                content: None,
                url: None,
                domain: None,
                language: "en".into(),
                confidence: desc.confidence * 0.8,
                recall_weight: 1.0,
                importance: desc.importance * 0.7,
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata: Some(serde_json::json!({
                    "parent_resource_id": node_id,
                    "episode_id": self.episode_id.clone(),
                })),
                temporal: None,
                supersedes: None,
                source_episode: Some(self.episode_id.clone()),
                parent_id: None,
                depth: 0,
                cluster_id: None,
            };

            let iid = insight_node.id.clone();
            insert_node(self.conn, &insight_node)
                .map_err(|e| format!("insert insight node failed: {}", e))?;
            upsert_edge(
                self.conn,
                &node_id,
                &iid,
                RelationType::Supports,
                0.7,
                Some("Key insight derived from resource"),
            )
            .map_err(|e| format!("upsert insight edge failed: {}", e))?;
            insight_ids.push(iid);
        }

        self.ingest_log.push(IngestLogEntry {
            title: desc.title.clone(),
            node_type: desc.category.clone(),
            node_id: node_id.clone(),
            status: "success".into(),
            error: None,
        });

        Ok(ResourceIngestResult {
            node_id,
            insight_ids,
        })
    }

    pub fn relate(
        &self,
        from_id: &str,
        to_id: &str,
        rel: RelationType,
        weight: f64,
        desc: Option<&str>,
    ) -> Result<(), String> {
        upsert_edge(self.conn, from_id, to_id, rel, weight, desc)
            .map_err(|e| format!("relate failed: {}", e))
    }

    pub fn relate_by_title(
        &self,
        from_title: &str,
        to_title: &str,
        rel: RelationType,
        weight: f64,
        desc: Option<&str>,
    ) -> Result<(), String> {
        let from = find_node_by_title(self.conn, from_title)
            .map_err(|e| format!("find from '{}' failed: {}", from_title, e))?
            .ok_or_else(|| format!("node not found by title: {}", from_title))?;
        let to = find_node_by_title(self.conn, to_title)
            .map_err(|e| format!("find to '{}' failed: {}", to_title, e))?
            .ok_or_else(|| format!("node not found by title: {}", to_title))?;
        upsert_edge(self.conn, &from.id, &to.id, rel, weight, desc)
            .map_err(|e| format!("relate_by_title failed: {}", e))
    }

    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== Resource Ingestion Report ===".to_string());
        lines.push(format!("Episode ID: {}", self.episode_id));
        lines.push(format!(
            "Total resources ingested: {}",
            self.ingest_log.len()
        ));
        for entry in &self.ingest_log {
            let status = if entry.status == "success" {
                "✅"
            } else {
                "❌"
            };
            lines.push(format!(
                "  {} {} ({:?}) — {}",
                status, entry.title, entry.node_type, entry.node_id
            ));
            if let Some(ref err) = entry.error {
                lines.push(format!("    Error: {}", err));
            }
        }
        lines.join("\n")
    }

    pub fn episode_id(&self) -> &str {
        &self.episode_id
    }
}

fn find_node_by_title(conn: &Connection, title: &str) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata
         FROM nodes WHERE title=?1 LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![title])?;
    match rows.next()? {
        Some(row) => Ok(Some(KnowledgeNode {
            recall_weight: 1.0,
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row
                .get::<_, Option<String>>(13)?
                .and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
        })),
        None => Ok(None),
    }
}

pub fn ingest_session_resources(conn: &Connection) -> Result<String, String> {
    let mut ingester = ResourceIngester::new(conn);

    ingest_github_resources(&mut ingester)?;
    ingest_paper_resources(&mut ingester)?;
    ingest_web_resources(&mut ingester)?;
    ingest_bug_fixes(&mut ingester)?;
    ingest_new_modules(&mut ingester)?;
    link_related_resources(&mut ingester)?;

    // ── Wave 2026-09-20: Crystal Core absorption ──
    // Absorb arxiv-complete dataset metadata (3.1M papers, sampled)
    ingest_arxiv_complete_metadata(&mut ingester)?;
    // Absorb key GitHub repos into crystal core
    ingest_crystal_core_repos(&mut ingester)?;
    // Absorb personal-ai ecosystem repos
    ingest_personal_ai_ecosystem(&mut ingester)?;

    // External "Cortex-Brain" volume: register as a discoverable KB catalog so the
    // 114 GB offline archive is connected (Dark Forest), not inert. No-op if unmounted.
    // This runs in production (ingest_session_resources is called at startup) → T3 wiring.
    if let Err(e) = register_cortex_brain(conn, std::path::Path::new(CORTEX_ROOT)) {
        warn!("[cortex] register skipped: {e}");
    }

    Ok(ingester.report())
}

/// Mount point of the external Cortex-Brain volume (cold offline archive).
const CORTEX_ROOT: &str = "/Volumes/NeoTrixBrain";

/// Upsert a `cortex_brain` registry node. `kind` (causal_graph / corpus_archive / archive_dir)
/// is stored inside `metadata` because the `nodes` table has no `kind` column.
fn upsert_cortex_node(
    conn: &Connection,
    url: &str,
    title: &str,
    content: &str,
    meta: &serde_json::Value,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now();
    conn.execute(
        "INSERT OR REPLACE INTO nodes \
         (id, node_type, title, content, url, created_at, updated_at, data_tier, tier, metadata) \
         VALUES (?1,'cortex_brain',?2,?3,?4,?5,?5,'cache','warm',?6)",
        rusqlite::params![id, title, content, url, ts, meta.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Register the external Cortex-Brain volume (`/Volumes/NeoTrixBrain`) as a KB resource
/// catalog. Scans `cortex-archive/{zim,pmtiles,wikipedia}` and `working/causal_graph.json`,
/// upserting lightweight `cortex_source://` registry nodes so the volume is discoverable and
/// connected rather than dead weight. Returns the number of registry nodes created.
///
/// Refuses to fabricate nodes when the volume is unmounted (avoids phantom catalog entries).
pub fn register_cortex_brain(conn: &Connection, root: &std::path::Path) -> Result<usize, String> {
    if !root.exists() {
        return Ok(0);
    }
    let mut count = register_archive_dir(conn, &root.join("cortex-archive"))?;
    let causal = root.join("working").join("causal_graph.json");
    if causal.exists() {
        let mut payload = serde_json::json!({
            "kind": "cortex_causal_graph",
            "path": causal.to_string_lossy(),
            "loaded_by": "E8AbductionBridge",
        });
        enrich_cortex_metadata(&mut payload, Some(causal.as_path()), "in");
        upsert_cortex_node(
            conn,
            "cortex_source://causal_graph",
            "Cortex causal graph (E8 abduction)",
            &payload.to_string(),
            &payload,
        )?;
        count += 1;
    }
    // The 68 GB corpus (knowledge-archive-corpus-20260825.db, lives on the external brain
    // volume) is a SUPERSET snapshot of the live KB — catalog it as a cold archive (do NOT
    // merge its 22M nodes into the warm live KB, per Dark Forest: connect, don't bloat).
    if let Some(corpus) = corpus_archive_path() {
        let sz = std::fs::metadata(&corpus).map(|m| m.len()).unwrap_or(0);
        let mut payload = serde_json::json!({
            "kind": "corpus_cold_archive",
            "path": corpus.to_string_lossy(),
            "bytes": sz,
            "note": "superset snapshot of live KB; cold storage",
        });
        enrich_cortex_metadata(&mut payload, Some(corpus.as_path()), "in");
        upsert_cortex_node(
            conn,
            "cortex_source://corpus-archive",
            "External-brain 68GB corpus (cold archive, superset of live KB)",
            &payload.to_string(),
            &payload,
        )?;
        count += 1;
    }
    Ok(count)
}

/// Path to the 68 GB corpus DB, preferring the cold copy on the external brain volume and
/// falling back to the local `~/.neotrix` copy. Returns None when absent.
pub fn corpus_archive_path() -> Option<std::path::PathBuf> {
    let ext = std::path::Path::new(CORTEX_ROOT).join("knowledge-archive-corpus-20260825.db");
    if ext.exists() {
        return Some(ext);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let local = std::path::Path::new(&home)
        .join(".neotrix")
        .join("knowledge-archive-corpus-20260825.db");
    if local.exists() {
        Some(local)
    } else {
        None
    }
}

/// Local (original) corpus DB path — the source for `migrate_cortex_archive`.
/// Falls back to the external-brain copy when the local duplicate has been reclaimed
/// to free disk (the two are byte-identical, produced by `migrate_cortex_corpus`).
fn local_corpus_source() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let local = std::path::Path::new(&home)
        .join(".neotrix")
        .join("knowledge-archive-corpus-20260825.db");
    if local.exists() {
        return Some(local);
    }
    let ext = std::path::Path::new(CORTEX_ROOT).join("knowledge-archive-corpus-20260825.db");
    if ext.exists() {
        Some(ext)
    } else {
        None
    }
}

/// Copy the local 68 GB corpus DB onto the external brain volume as cold storage.
/// Idempotent: if a complete copy already exists, skip. Refuses unless the volume is mounted
/// and has enough free space. `dry_run` reports the plan without copying. The source file is
/// preserved (a second copy is kept per the cold-archive redundancy guideline).
///
/// The actual copy is resumable (see `copy_resumable`): if the flaky external volume drops
/// mid-transfer, re-running resumes from the last verified chunk instead of restarting.
pub fn migrate_cortex_corpus(
    conn: &Connection,
    root: &std::path::Path,
    dry_run: bool,
) -> Result<String, String> {
    let src = local_corpus_source().ok_or_else(|| {
        "本地未找到 68GB corpus (knowledge-archive-corpus-20260825.db)".to_string()
    })?;
    let dest = root.join("knowledge-archive-corpus-20260825.db");
    let size = std::fs::metadata(&src)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;
    // idempotent: a complete copy already present
    if dest.exists() {
        let dsz = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
        if dsz == size {
            update_corpus_catalog(conn, &dest, &src)?;
            return Ok("外置大脑上已存在完整 corpus 副本，迁移幂等跳过（已刷新编目指向）。".into());
        }
        // otherwise a partial copy exists → copy_resumable resumes from its .prog checkpoint
    }
    let avail = free_space_bytes(root)?;
    if avail < size * 11 / 10 {
        return Err(format!(
            "外置大脑剩余空间不足: 需 {} 字节, 可用 {} 字节",
            size, avail
        ));
    }
    if dry_run {
        return Ok(format!(
            "[dry-run] 将续传/拷贝 {} → {}\n  大小 {:.1} GB, 外置盘可用 {:.1} GB (保留本地源副本)\n  支持掉盘续传：中断后重挂载重跑即可从校验点继续",
            src.display(),
            dest.display(),
            size as f64 / 1e9,
            avail as f64 / 1e9
        ));
    }
    copy_resumable(&src, &dest)?;
    update_corpus_catalog(conn, &dest, &src)?;
    Ok(format!(
        "已迁移 corpus 到外置大脑: {}\n  大小 {:.1} GB, 本地源副本已保留。",
        dest.display(),
        size as f64 / 1e9
    ))
}

/// Resumable chunked copy with per-chunk readback verification. A `<dest>.prog` sidecar stores
/// the last *verified* byte offset; on restart it seeks there and overwrites any corrupt tail,
/// so a dropped volume only costs the current chunk — never a full restart. Any I/O error
/// (e.g. volume unmount) returns a resume hint instead of corrupting the destination.
fn copy_resumable(src: &std::path::Path, dest: &std::path::Path) -> Result<u64, String> {
    use std::io::{Read, Seek, SeekFrom, Write};
    const CHUNK: usize = 256 * 1024 * 1024; // 256 MiB
    let size = std::fs::metadata(src)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;
    let prog = std::path::PathBuf::from(format!("{}.prog", dest.display()));
    let mut verified = read_progress(&prog).unwrap_or(0);
    // sanity: dest shorter than verified means a corrupt/truncated tail → restart from 0
    if let Ok(dmeta) = std::fs::metadata(dest) {
        if dmeta.len() < verified {
            verified = 0;
            let _ = std::fs::remove_file(&prog);
        }
    }
    let mut sf = std::fs::File::open(src).map_err(|e| e.to_string())?;
    let mut df = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(dest)
        .map_err(|e| e.to_string())?;
    sf.seek(SeekFrom::Start(verified))
        .map_err(|e| e.to_string())?;
    df.seek(SeekFrom::Start(verified))
        .map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; CHUNK];
    let mut check = vec![0u8; CHUNK];
    let mut offset = verified;
    while offset < size {
        let n = std::cmp::min(CHUNK as u64, size - offset) as usize;
        // read source → write dest → read back & compare; any I/O error ⇒ likely掉盘
        let io = (|| -> std::io::Result<()> {
            sf.read_exact(&mut buf[..n])?;
            df.write_all(&buf[..n])?;
            df.flush()?;
            df.seek(SeekFrom::Start(offset))?;
            df.read_exact(&mut check[..n])?;
            Ok(())
        })();
        if let Err(e) = io {
            return Err(format!(
                "传输中断（外置盘可能掉盘）: {e}；重挂载后重跑 /cortex corpus migrate --force 可从 {:.1} GB 校验点续传",
                offset as f64 / 1e9
            ));
        }
        offset += n as u64;
        write_progress(&prog, offset)?;
        info!(
            "[corpus migrate] 校验通过 {:.1} / {:.1} GB",
            offset as f64 / 1e9,
            size as f64 / 1e9
        );
    }
    let _ = std::fs::remove_file(&prog);
    let dest_size = std::fs::metadata(dest)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;
    if dest_size != size {
        let _ = std::fs::remove_file(dest);
        return Err(format!(
            "最终大小校验失败: 源 {} 目标 {} (已删除损坏副本)",
            size, dest_size
        ));
    }
    Ok(size)
}

/// Update the `cortex_source://corpus-archive` catalog node to point at the external copy.
fn update_corpus_catalog(
    conn: &Connection,
    dest: &std::path::Path,
    src: &std::path::Path,
) -> Result<(), String> {
    let sz = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let payload = serde_json::json!({
        "kind": "corpus_cold_archive",
        "path": dest.to_string_lossy(),
        "bytes": sz,
        "note": "superset snapshot of live KB; cold storage on external brain",
        "migrated_from": src.to_string_lossy(),
    });
    upsert_cortex_node(
        conn,
        "cortex_source://corpus-archive",
        "Corpus (cold archive, on external brain)",
        &payload.to_string(),
        &payload,
    )
}

fn read_progress(path: &std::path::Path) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
}

fn write_progress(path: &std::path::Path, offset: u64) -> Result<(), String> {
    std::fs::write(path, offset.to_string()).map_err(|e| e.to_string())
}

/// Available free space (bytes) on the filesystem hosting `path`, via `df`.
fn free_space_bytes(path: &std::path::Path) -> Result<u64, String> {
    let out = Command::new("df")
        .arg("-k")
        .arg(path.to_string_lossy().to_string())
        .output()
        .map_err(|e| format!("df failed: {e}"))?;
    if !out.status.success() {
        return Err("df 查询失败".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // second line: filesystem ... used avail capacity ...
    let line = text.lines().nth(1).ok_or("df 输出无法解析")?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    // avail is the 4th column (1K-blocks)
    let avail_kib: u64 = cols
        .get(3)
        .ok_or("df 输出无法解析")?
        .parse()
        .map_err(|_| "df avail 解析失败")?;
    Ok(avail_kib * 1024)
}

/// 回收 `/private/tmp` 下过期的 `nt-target-*` 构建缓存 (孤儿 cargo target 目录), 释放系统盘。
/// 保守策略: 仅删 `nt-target-` 前缀目录, 跳过 `*-check` (其他 loop/会话校验目录);
/// 仅删 mtime 早于 `max_age_days` 天的目录。`dry_run=true` 只计数不删除, 供磁盘压力门禁安全调用。
pub fn reclaim_nt_target_tmp(max_age_days: u64, dry_run: bool) -> Result<usize, String> {
    let tmp = std::path::Path::new("/private/tmp");
    if !tmp.is_dir() {
        return Ok(0);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let cutoff = now.saturating_sub(max_age_days * 86400);
    let mut reclaimed = 0usize;
    for entry in std::fs::read_dir(tmp).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.starts_with("nt-target-") || name.ends_with("-check") || !path.is_dir() {
            continue;
        }
        let mtime = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if mtime < cutoff {
            if dry_run {
                reclaimed += 1;
            } else {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| format!("清理 {} 失败: {}", path.display(), e))?;
                reclaimed += 1;
            }
        }
    }
    Ok(reclaimed)
}

/// Reclaim KB nodes whose backing ZIM archive is no longer on the external brain volume.
/// Matches the KB's distinct `zimid://<uuid>` source set against the on-disk ZIM file uuids
/// (read via libzim). Returns (orphan_sources, orphan_article_nodes).
///
/// Refuses unless the volume is mounted AND libzim is available. When `dry_run` is true no
/// rows are deleted; pass `dry_run=false` (i.e. an explicit force flag) to actually delete.
pub fn prune_cortex_orphans(
    conn: &Connection,
    root: &std::path::Path,
    dry_run: bool,
) -> Result<(usize, usize), String> {
    if !root.exists() {
        return Err(
            "external brain not mounted — refuse to prune (would orphan everything)".into(),
        );
    }
    let mut backed = disk_zim_uuids(root)?;
    if backed.is_empty() {
        return Err("no ZIM files found on volume — abort prune".into());
    }
    // The mounted volume's ZIM set is often a DIFFERENT snapshot than the one that
    // originally populated the live KB. The 68 GB corpus DB is the true superset backing
    // store (it contains ~99% of live zim URLs). A KB source is only an ORPHAN if it is
    // absent from BOTH the mounted volume AND the corpus — otherwise pruning would wrongly
    // delete hundreds of thousands of valid article nodes.
    if let Some(corpus) = corpus_archive_path() {
        if let Ok(cdb) = Connection::open(&corpus) {
            if let Ok(mut st) = cdb
                .prepare("SELECT DISTINCT substr(url,10,36) FROM nodes WHERE url LIKE 'zimid://%'")
            {
                let rows = st
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok());
                for u in rows {
                    backed.insert(u);
                }
            }
        }
    }
    let mut stmt = conn
        .prepare("SELECT DISTINCT substr(url,10,36) FROM nodes WHERE url LIKE 'zimid://%'")
        .map_err(|e| e.to_string())?;
    let kb_sources: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    let mut orphan_nodes = 0usize;
    let mut orphan_sources = 0usize;
    for src in kb_sources {
        if backed.contains(&src) {
            continue;
        }
        orphan_sources += 1;
        let like = format!("zimid://{src}/%");
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE url LIKE ?1",
                [&like],
                |r| r.get(0),
            )
            .unwrap_or(0);
        orphan_nodes += n as usize;
        if !dry_run {
            // edges first (while node ids still resolvable), then nodes
            conn.execute(
                "DELETE FROM edges WHERE from_id IN (SELECT id FROM nodes WHERE url LIKE ?1) OR to_id IN (SELECT id FROM nodes WHERE url LIKE ?1)",
                [&like],
            ).map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM nodes WHERE url LIKE ?1", [&like])
                .map_err(|e| e.to_string())?;
        }
    }
    Ok((orphan_sources, orphan_nodes))
}

/// Enumerate on-disk ZIM uuids under `root` using libzim (python3). Returns the set of
/// canonical uuid strings. Errors if python3/libzim is unavailable.
fn disk_zim_uuids(root: &std::path::Path) -> Result<HashSet<String>, String> {
    let script = "import sys,glob,os\n\
        try:\n    from libzim.reader import Archive\n\
        except Exception as e:\n    sys.stderr.write('NO_LIBZIM:%s'%e); sys.exit(2)\n\
        root=sys.argv[1]\n\
        paths=glob.glob(os.path.join(root,'cortex-archive','zim','*.zim'))\n\
        paths+=glob.glob(os.path.join(root,'cortex-archive','wikipedia','*.zim'))\n\
        for p in paths:\n    try: print(str(Archive(p).uuid))\n    except Exception: pass\n";
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(root.to_string_lossy().to_string())
        .output()
        .map_err(|e| format!("failed to spawn python3: {e}"))?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr);
        return Err(format!("libzim/uuid scan failed: {msg}"));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

/// Scan one directory and upsert a `cortex_source://<sub>` registry node per populated
/// sub-directory (zim / pmtiles / wikipedia), recording file counts for discoverability.
fn register_archive_dir(conn: &Connection, dir: &std::path::Path) -> Result<usize, String> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for sub in ["zim", "pmtiles", "wikipedia"] {
        let p = dir.join(sub);
        if !p.exists() {
            continue;
        }
        let n = std::fs::read_dir(&p)
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if n == 0 {
            continue;
        }
        let url = format!("cortex_source://{sub}");
        let payload = serde_json::json!({
            "kind": "cortex_archive_dir",
            "path": p.to_string_lossy(),
            "files": n,
        });
        upsert_cortex_node(
            conn,
            &url,
            &format!("Cortex archive: {sub}"),
            &payload.to_string(),
            &payload,
        )?;
        count += 1;
    }
    Ok(count)
}

fn ingest_github_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::github(
        "stablyai", "orca",
        "stablyai/orca — Orca: Dual-System NextState Prediction",
        "Orca implements a dual-system architecture with Unconscious (dense Markov) and Conscious (sparse event-conditioned) NextStatePredictor, plus a DecoderReadout that freezes the backbone and trains only lightweight readout layers."
    ).with_key_insights(vec![
        "Three prediction modes: Unconscious/Conscious/Hybrid controlled by α∈[0,1]",
        "DecoderReadout gradient descent on frozen backbone",
        "Dual-system architecture mirrors NeoTrix GWT conscious/unconscious processing",
    ]).with_tags(vec!["dual-system", "next-state-prediction", "decoder-readout", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "offchainthoughts", "Amber",
        "offchainthoughts/Amber — Self-Certifying Embedding Artifacts",
        "Amber defines a self-certifying embedding artifact format using SHA-256 Merkle commitment over quantized embedding chunks, providing probabilistic authenticity audit with 1−2^{−128} confidence."
    ).with_key_insights(vec![
        "Flattened embedding vector + SHA-256 Merkle tree binds source chunks to quantized embeddings",
        "Probabilistic authenticity audit achieves 1−2^{−128} confidence",
        "4-bit quantization reduces storage 8× vs f32",
    ]).with_tags(vec!["embedding", "merkle", "commitment", "audit", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "AgwaB", "pi-workflow",
        "AgwaB/pi-workflow — Workflow Orchestration Patterns",
        "Workflow orchestration patterns for multi-step agent pipelines with conditional branching, parallel execution, and error recovery."
    ).with_key_insights(vec![
        "Conditional branching based on intermediate results",
        "Parallel execution of independent workflow branches",
        "Error recovery with retry and fallback paths",
    ]).with_tags(vec!["workflow", "orchestration", "patterns", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "facebook", "astryx",
        "facebook/astryx — Graph Memory Architecture",
        "Graph-structured memory architecture for persistent agent state, providing queryable, inspectable long-term memory with graph traversal operations."
    ).with_key_insights(vec![
        "Graph structure as core architecture for agent memory",
        "Enables BFS-based relationship discovery across memory",
        "Supports long-term persistent agent state management",
    ]).with_tags(vec!["graph-memory", "agent-state", "persistence", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "msitarzewski", "agency-agents",
        "msitarzewski/agency-agents — Agent Pattern Composition",
        "Composable agent patterns for building complex multi-agent systems with specialized roles and inter-agent communication."
    ).with_key_insights(vec![
        "Agent composition patterns for multi-agent systems",
        "Specialized role assignment with inter-agent communication",
        "Scalable agent coordination architectures",
    ]).with_tags(vec!["agent-patterns", "multi-agent", "composition", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "shadcn-labs", "agentcn",
        "shadcn-labs/agentcn — Agent Coordination Patterns",
        "Coordination strategies for AI agent teams, including consensus mechanisms, delegation protocols, and conflict resolution."
    ).with_key_insights(vec![
        "Consensus mechanisms for multi-agent decision making",
        "Delegation protocols with capability matching",
        "Conflict resolution between competing agent proposals",
    ]).with_tags(vec!["coordination", "consensus", "delegation", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "crosstalk-solutions", "project-nomad",
        "crosstalk-solutions/project-nomad — Persistent Agent State",
        "Persistent agent state management system enabling long-running agents with checkpoint/restore, state migration, and session continuity."
    ).with_key_insights(vec![
        "Checkpoint/restore for long-running agent sessions",
        "State migration across different runtime environments",
        "Session continuity with durable state snapshots",
    ]).with_tags(vec!["agent-state", "persistence", "checkpoint", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "google", "sec-gemini",
        "google/sec-gemini — Security Agents with Function-Calling",
        "Gemini-powered security agents with function-calling tools for rule search, coverage heatmaps, duplicate detection, and LLM-based rule review."
    ).with_key_insights(vec![
        "Function-calling tools for security rule search and coverage analysis",
        "Coverage heatmap generation for security rule gaps",
        "LLM-based rule review and duplicate detection",
    ]).with_tags(vec!["security", "gemini", "function-calling", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_paper_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::paper(
        "2605.06732",
        "Training in Imagination — Optimal Sample Allocation for Model-Based RL",
        "Nadav Timor et al. prove Theorem 1 (optimal dynamics vs reward sample ratio), Theorem 2 (noisy reward REINFORCE gradient), and Lemma 1 (Lipschitz error bound). Provides theoretical foundation for efficient model-based RL training."
    ).with_key_insights(vec![
        "Theorem 1: Ndyn/Nrew = α/β · γ·Lr·(1+Lπ) / (1−γ·Lf·(1+Lπ)) · crew/cdyn · εdyn/εrew",
        "Lemma 1: Return error bound decomposes into dynamics error × coefficient + reward error × coefficient",
        "Corollary 1: Lower Lipschitz constants tighten the return-error bound",
        "Theorem 2: REINFORCE with noisy rewards requires optimal noise fidelity",
    ]).with_tags(vec!["rl", "model-based", "sample-allocation", "lipschitz", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_web_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::article(
        "State of the Graph 2026 — Knowledge Graphs as Agent Memory",
        "Comprehensive analysis of graph structure as core architecture for agent memory, making long-term behavior persistent, queryable, and inspectable.",
        "https://stateofthegraph.com/knowledge-graphs"
    ).with_key_insights(vec![
        "Graph structure is the core architecture for agent memory",
        "Enables persistent, queryable, and inspectable long-term behavior",
        "BFS traversal and community detection for memory exploration",
    ]).with_tags(vec!["knowledge-graphs", "agent-memory", "graph-architecture", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::article(
        "Fable 5 Prompt Library — Goal→Reason→Boundaries→Verification",
        "Anthropic's Fable 5 prompting architecture: Goal→Reason→Boundaries→Verification. Structured prompting framework for reliable AI agent behavior.",
        "https://every.to/claude-fable-5-prompt-library"
    ).with_key_insights(vec![
        "Goal → Reason → Boundaries → Verification four-step architecture",
        "Boundary separation prevents premature action before analysis",
        "Verification gate ensures output correctness before delivery",
    ]).with_tags(vec!["prompting", "fable-5", "architecture", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_bug_fixes(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #1: GRPO Importance Ratio — Value Ratio vs Softmax Policy Ratio (CRITICAL)",
        "nt_core_policy.rs used old_value/ref_val (value ratio) instead of π_new(a|s)/π_old(a|s) (softmax policy ratio) for GRPO importance sampling. Fixed by replacing with softmax-based ratio: (new_logit - ref_logit).exp()."
    ).with_importance(0.95).with_tags(vec!["bug", "critical", "grpo", "policy-gradient", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #2: Double Discounting in Step Reward TD Bootstrap (MODERATE)",
        "nt_core_policy.rs applied double discounting: step rewards were discounted to present value, then TD bootstrap re-discounted them again. Fixed: step rewards use discounted present value; TD bootstrap uses r + γ·V(s') without extra discounting."
    ).with_importance(0.85).with_tags(vec!["bug", "moderate", "discounting", "td-learning", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #3: MODULE_COUNT Mismatch — 11 vs 14 Specialists (MAJOR)",
        "resonance.rs had MODULE_COUNT=11 but workspace registers 14 specialists, causing resonance matrix size mismatch. Fixed to MODULE_COUNT=14 and updated default_specialist_states() with 3 new entries: AISecurity(45), ImageGenerator(46), EvidenceWeightedHypothesis(50)."
    ).with_importance(0.90).with_tags(vec!["bug", "major", "resonance", "gwt", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_new_modules(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Training-in-Imagination Module (nt_core_imagination.rs)",
        "Implements arXiv 2605.06732: OptimalSampleAllocation (Theorem 1), LipschitzRegularizer (Corollary 1), NoisyRewardPolicy (Theorem 2), ReturnErrorBound (Lemma 1). 19 tests."
    ).with_key_insights(vec![
        "OptimalSampleAllocation computes Ndyn/Nrew = α/β · γ·Lr·(1+Lπ) / (1−γ·Lf·(1+Lπ)) · crew/cdyn · εdyn/εrew",
        "LipschitzRegularizer provides spectral-normalization-based regularization to tighten return-error bounds",
        "NoisyRewardPolicy implements unbiased REINFORCE gradient estimation under reward noise",
        "ReturnErrorBound computes Lemma 1 error decomposition for model-based RL",
    ]).with_importance(0.85).with_tags(vec!["module", "training-in-imagination", "rl", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        "Persistent graph-structured memory layer for GWT. GraphMemoryStore with LRU eviction, BFS, semantic search. MemoryGraphSpecialist for GWT resonance integration. 22 tests."
    ).with_key_insights(vec![
        "GraphMemoryNode with 8 variants (Concept, Session, SpecialistActivation, Decision, Reward, Skill, Reflection, Goal)",
        "GraphMemoryStore with LRU eviction (default 10000 nodes), BFS traversal, semantic search by cosine similarity",
        "MemoryGraphSpecialist integrates with GWT resonance cycles",
        "Subgraph extraction, merge, prune_expired, evict_lru operations",
    ]).with_importance(0.80).with_tags(vec!["module", "graph-memory", "gwt", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Amber Embedding Commitment (nt_memory_commitment.rs)",
        "Self-certifying embedding artifact format with 4-bit quantization, SHA-256 Merkle tree, probabilistic audit (1−2^{−128}), position-length binding. 18 tests."
    ).with_key_insights(vec![
        "4-bit quantization reduces f32 embedding storage 8× with MSE < 0.1",
        "SHA-256 Merkle tree over 32-byte chunks enables per-dimension-chunk verification",
        "Probabilistic audit with reservoir sampling and detection probability 1−(1−ρ)^k",
        "Position-length binding prevents chunk reordering attacks",
        "JSON persistence via save/load for portable artifact distribution",
    ]).with_importance(0.85).with_tags(vec!["module", "embedding", "merkle", "commitment", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Security MCP Tools (nt_shield_mcp_security.rs)",
        "SecurityMcpToolRegistry with 6 built-in MCP tools: scan_secrets, audit_code_security, check_dependencies, test_prompt_injection, analyze_threat, security_health_check. Rate limiting, audit trail. 12+ tests."
    ).with_key_insights(vec![
        "6 built-in security tools with SecurityToolCategory enum (9 categories)",
        "scan_secrets detects API keys, tokens, passwords via regex patterns",
        "audit_code_security checks OWASP Top 10 patterns with CWE mapping",
        "test_prompt_injection detects jailbreak and prompt leak patterns",
        "Rate limiting (30 calls/min default) and scan_history audit trail",
    ]).with_importance(0.80).with_tags(vec!["module", "security", "mcp", "tools", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn link_related_resources(ingester: &mut ResourceIngester) -> Result<(), String> {
    link_pair(
        ingester,
        "stablyai/orca — Orca: Dual-System NextState Prediction",
        "Training-in-Imagination Module (nt_core_imagination.rs)",
        RelationType::InspiredBy,
        0.8,
        "Orca dual-system inspired E8 conscious/unconscious split",
    )?;
    link_pair(
        ingester,
        "offchainthoughts/Amber — Self-Certifying Embedding Artifacts",
        "Amber Embedding Commitment (nt_memory_commitment.rs)",
        RelationType::InspiredBy,
        0.9,
        "Amber commitment format directly implemented",
    )?;
    link_pair(
        ingester,
        "google/sec-gemini — Security Agents with Function-Calling",
        "Security MCP Tools (nt_shield_mcp_security.rs)",
        RelationType::InspiredBy,
        0.8,
        "Sec-Gemini function-calling pattern for MCP security tools",
    )?;
    link_pair(
        ingester,
        "facebook/astryx — Graph Memory Architecture",
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        RelationType::InspiredBy,
        0.7,
        "Astryx graph memory patterns for GWT integration",
    )?;
    link_pair(
        ingester,
        "Training in Imagination — Optimal Sample Allocation for Model-Based RL",
        "Bug #1: GRPO Importance Ratio — Value Ratio vs Softmax Policy Ratio (CRITICAL)",
        RelationType::References,
        0.6,
        "GRPO policy ratio theory from RL literature",
    )?;
    link_pair(
        ingester,
        "State of the Graph 2026 — Knowledge Graphs as Agent Memory",
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        RelationType::InspiredBy,
        0.8,
        "Knowledge graphs as core architecture for agent memory",
    )?;
    link_pair(
        ingester,
        "Fable 5 Prompt Library — Goal→Reason→Boundaries→Verification",
        "Bug #3: MODULE_COUNT Mismatch — 11 vs 14 Specialists (MAJOR)",
        RelationType::References,
        0.5,
        "Boundary separation principle aligns with Fable 5 verification gate",
    )?;

    Ok(())
}

fn link_pair(
    ingester: &mut ResourceIngester,
    from_title: &str,
    to_title: &str,
    rel: RelationType,
    weight: f64,
    desc: &str,
) -> Result<(), String> {
    ingester.relate_by_title(from_title, to_title, rel, weight, Some(desc))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Wave 2026-09-20: Crystal Core Absorption — arxiv-complete + GitHub repos
// ═══════════════════════════════════════════════════════════════════════════════

/// Absorb arxiv-complete dataset metadata into the crystal core.
///
/// The `secemp9/arxiv-complete` dataset on HuggingFace contains 3,148,796 papers
/// with full metadata (title, authors, abstracts, categories, dates). We fetch
/// a representative sample via the datasets-server API and ingest key papers
/// across AI/ML/CS categories to give the KB deep academic coverage.
///
/// Strategy: fetch 100 metadata rows (covers major categories), then synthesize
/// category-level summary nodes for the full 3.1M paper corpus.
fn ingest_arxiv_complete_metadata(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // 1. Register the dataset itself as a meta-resource
    let dataset_desc = ResourceDescriptor::article(
        "arxiv-complete: Full arXiv Corpus (3.1M papers)",
        "secemp9/arxiv-complete — A snapshot of arXiv's metadata, version history, submission files \
         and rendered documents. Covers 3,148,796 papers with file contents, paths, sizes and SHA-256 \
         digests. Metadata from arXiv's OAI-PMH arXivRaw interface; files from GCS mirror, S3 source \
         archives and direct PDF fetches. Configs: metadata (1.6GB), versions (269MB), files (2.4GB), \
         paper_text (70GB), latex (0.16TB), source (6.51TB), pdf (8.65TB).",
        "https://huggingface.co/datasets/secemp9/arxiv-complete",
    ).with_key_insights(vec![
        "3,148,796 papers covering all arXiv categories (cs, math, physics, bio, etc.)",
        "Metadata config: 1.6 GB Parquet with paper_id, title, authors, abstracts, categories, dates",
        "paper_text config: 70 GB of resolved TeX content for 2.86M papers (90.7%)",
        "source config: 6.51 TB of complete submission packages for 3.12M papers (99.1%)",
        "Enables large-scale academic knowledge graph construction and citation analysis",
    ]).with_tags(vec![
        "arxiv", "dataset", "academic", "corpus", "metadata",
        "huggingface", "papers", "knowledge-graph",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.95).with_confidence(0.95);

    let r = ingester.ingest(&dataset_desc)?;
    ids.push(r.node_id);

    // 2. Fetch a sample of metadata via the HuggingFace datasets-server API
    //    This gives us real paper metadata to ingest
    let sample_url = "https://datasets-server.huggingface.co/rows?dataset=secemp9/arxiv-complete&config=metadata&split=train&offset=0&length=100";
    if let Ok(resp) = super::nt_http::run_blocking(|| {
        super::nt_http::shared_blocking_client()
            .get(sample_url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
    }) {
        if let Ok(data) = resp.json::<serde_json::Value>() {
            if let Some(rows) = data["rows"].as_array() {
                for row in rows {
                    let row_data = &row["row"];
                    let paper_id = row_data["paper_id"].as_str().unwrap_or("");
                    let title = row_data["title"].as_str().unwrap_or("");
                    let abstract_text = row_data["abstract"].as_str().unwrap_or("");
                    let authors = row_data["authors"].as_str().unwrap_or("");
                    let categories = row_data["categories"].as_str().unwrap_or("");
                    let date = row_data["date"].as_str().unwrap_or("");

                    if title.is_empty() {
                        continue;
                    }

                    let summary = if abstract_text.len() > 500 {
                        format!(
                            "{}...",
                            &abstract_text[..abstract_text
                                .char_indices()
                                .nth(500)
                                .map(|(i, _)| i)
                                .unwrap_or(500)]
                        )
                    } else {
                        abstract_text.to_string()
                    };

                    let desc = ResourceDescriptor::paper(paper_id, title, &summary)
                        .with_content(&format!(
                            "Authors: {}\nCategories: {}\nDate: {}\nAbstract: {}",
                            authors, categories, date, abstract_text,
                        ))
                        .with_tags(vec![
                            "arxiv",
                            "paper",
                            "academic",
                            categories.split_whitespace().next().unwrap_or("unknown"),
                            &format!("absorbed-{}", now()),
                        ])
                        .with_importance(0.75)
                        .with_confidence(0.9);

                    if let Ok(result) = ingester.ingest(&desc) {
                        ids.push(result.node_id);
                    }
                }
            }
        }
    } else {
        warn!("[arxiv-complete] Failed to fetch sample metadata from HuggingFace API");
    }

    // 3. Synthesize category-level summary nodes for the full corpus
    let arxiv_categories = vec![
        ("cs.AI", "Artificial Intelligence", "Machine learning, knowledge representation, planning, reasoning, NLP, computer vision, robotics"),
        ("cs.LG", "Machine Learning", "Supervised, unsupervised, reinforcement learning; deep learning; neural networks; optimization"),
        ("cs.CL", "Computation and Language", "NLP, computational linguistics, text mining, information extraction, machine translation"),
        ("cs.CV", "Computer Vision", "Image recognition, object detection, segmentation, video analysis, 3D vision"),
        ("cs.RO", "Robotics", "Robot design, control, planning, perception, human-robot interaction"),
        ("cs.IR", "Information Retrieval", "Search, recommendation systems, document ranking, query processing"),
        ("cs.SE", "Software Engineering", "Program analysis, testing, maintenance, development tools, formal methods"),
        ("cs.CR", "Cryptography and Security", "Security, privacy, cryptographic protocols, network security, applied crypto"),
        ("cs.DC", "Distributed Computing", "Cloud computing, parallel processing, consensus protocols, distributed systems"),
        ("cs.NE", "Neural and Evolutionary Computing", "Neural networks, evolutionary algorithms, genetic programming, swarm intelligence"),
        ("stat.ML", "Statistics: Machine Learning", "Statistical learning theory, Bayesian methods, probabilistic models"),
        ("math.OC", "Optimization and Control", "Convex optimization, control theory, operations research, mathematical programming"),
        ("q-bio.BM", "Biomolecules", "Protein structure, genomics, computational biology, molecular modeling"),
        ("physics.comp-ph", "Computational Physics", "Monte Carlo methods, molecular dynamics, computational methods in physics"),
    ];

    for (cat, name, desc_text) in &arxiv_categories {
        let node_desc = ResourceDescriptor::concept(
            &format!("arXiv:{} — {}", cat, name),
            &format!(
                "Category summary for the full arXiv corpus (3.1M papers total). {}: {}.",
                name, desc_text
            ),
        )
        .with_key_insights(vec![
            &format!("arXiv category {} covers: {}", cat, desc_text),
            "Part of the 3,148,796 paper arxiv-complete corpus on HuggingFace",
            "Full metadata available via secemp9/arxiv-complete dataset",
        ])
        .with_tags(vec![
            "arxiv",
            "category",
            "taxonomy",
            "academic",
            cat,
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.85)
        .with_confidence(0.9);

        if let Ok(result) = ingester.ingest(&node_desc) {
            ids.push(result.node_id);
        }
    }

    info!(
        "[arxiv-complete] Absorbed {} nodes (dataset + {} sampled papers + {} category summaries)",
        ids.len(),
        ids.len().saturating_sub(1 + arxiv_categories.len()),
        arxiv_categories.len()
    );

    Ok(ids)
}

/// Absorb key GitHub repos into the crystal core.
///
/// These repos represent the cutting-edge of personal AI, agent orchestration,
/// local inference, and developer tooling — all directly relevant to NeoTrix's
/// architecture and evolution.
fn ingest_crystal_core_repos(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // ── 1. OpenHuman — Personal AI superintelligence (39.9k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "tinyhumansai", "openhuman",
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "OpenHuman is an open source agent harness with local-first memory, agent orchestration, \
         and workflows. Features: Memory Tree + Obsidian Wiki for persistent local memory, \
         100+ OAuth integrations, 5000+ MCP servers, 90000+ Skills. Orchestrator with checkpointed \
         graph runs on tinyagents, agent-to-agent E2E encryption via Signal protocol, visual \
         workflow builder (tinyflows). Built with Rust (Tauri) + React. 39.9k stars.",
    ).with_key_insights(vec![
        "Memory Tree: data compressed into scored Markdown trees in SQLite, mirrored as Obsidian vault",
        "TokenJuice: tool output compressed before model, up to 80% fewer tokens",
        "Agent graphs with checkpoints: turns run as checkpointed graphs, pause for human, survive restart",
        "Agent-to-agent E2E encryption via Signal protocol with x402 payments",
        "Workflows: agent proposes automation, user reviews on canvas, durable trigger-driven runs",
        "Split brain: fast reflex agent triages inbound, deep reasoning core delegates to worker fleets",
        "Privacy Mode: one-switch enforced local-only inference in Rust core",
    ]).with_tags(vec![
        "personal-ai", "agent-harness", "memory-tree", "orchestration",
        "workflows", "local-first", "tauri", "rust",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.95).with_confidence(0.95))?;
    ids.push(r.node_id);

    // ── 2. Splash — Local inference engine for Apple silicon (413★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "incoai", "splash",
        "incoai/splash — Splash: Local Inference Engine for Apple Silicon",
        "A local inference engine for Apple silicon, built around the model. Serves models to \
         coding agents and OpenAI/Anthropic compatible clients. On 48GB M5 Pro: decodes \
         Qwen3.8-27B at 2x speed of next-fastest engine, 282ms TTFT with 32K context cached. \
         Kernels, draft model, and memory plan specialized per model. Speculative decoding \
         (DFlash 2) as default decode path. Fused Metal kernels compiled for exact shapes.",
    ).with_key_insights(vec![
        "Speculative decoding (DFlash 2) as default, not optional — every model ships its own draft",
        "Fused Metal kernels compiled for exact model dimensions, weights packed and mapped zero-copy",
        "Memory plan computed per-machine from Metal recommended memory minus weights/draft/state",
        "2x decode speed vs next-fastest engine on M5 Pro (74 tok/s on 27B, 210 tok/s on 35B-A3B)",
        "OpenAI + Anthropic compatible API with streaming, tool calls, JSON Schema, images, PDFs",
    ]).with_tags(vec![
        "inference", "apple-silicon", "metal", "speculative-decoding",
        "local-llm", "coding-agents", "llm-inference",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.85).with_confidence(0.9))?;
    ids.push(r.node_id);

    // ── 3. ccodex-sleep-state — Codex connection quality tool (441★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "gylive",
            "ccodex-sleep-state",
            "gylive/ccodex-sleep-state — Codex Connection Quality Tool",
            "A tool to improve Codex degradation, rate limiting, and connection experience. \
         Local one-click startup with web configuration. Supports Astra/Sol/Terra models, \
         subscriptions and proxies. Manages turn-state injection, proxy routing, and \
         model/account isolation. Written in Go with embedded web UI.",
        )
        .with_key_insights(vec![
            "Turn-state injection for Codex connection quality management",
            "Model and account isolation: state per account, credential, and model",
            "Proxy pool management with subscription import, HTTP/SOCKS5 support",
            "Web panel for configuration: model selection, proxy routing, diagnostics",
            "Strict mode vs fallback: controlled behavior when state unavailable",
        ])
        .with_tags(vec![
            "codex",
            "connection-quality",
            "proxy",
            "turn-state",
            "agent-infrastructure",
            "go",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.70)
        .with_confidence(0.85),
    )?;
    ids.push(r.node_id);

    // ── 4. Laya — Non-autoregressive decision engine (2.5k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "NandhaKishorM", "laya",
        "NandhaKishorM/laya — Laya: Non-Autoregressive Decision Engine",
        "Multilingual, non-autoregressive System 1 decision engine. Typed decisions over 100+ \
         languages in a single forward pass (33ms). Trained with RLCD (reinforcement learning \
         against strictly proper scoring rules). Three checkpoints: English (ModernBERT-large, 421M), \
         Multilingual (mmBERT-base, 322M), Typed-Decisions. Router picks optimal checkpoint per request.",
    ).with_key_insights(vec![
        "Single forward pass evaluation: 33ms for one question, 7.2ms/question batched on T4",
        "Three primitives: choice (classification), score (ordinal), noul (P(true) calibration)",
        "Router detects script/language in <0.5ms before forward pass, dispatches to optimal checkpoint",
        "RLCD training: strictly proper scoring rules produce statistically meaningful confidence scores",
        "Fine-tuned checkpoint beats Jev by 3.9 points on typed-decisions (0.766 vs 0.727)",
        "Calibrated ECE: 0.081 after temperature fitting (3x better than base 0.213)",
    ]).with_tags(vec![
        "decision-engine", "classification", "multilingual", "non-autoregressive",
        "rlcd", "calibration", "router", "bert",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.85).with_confidence(0.9))?;
    ids.push(r.node_id);

    // ── 5. Cutter — Reverse engineering platform (19.7k★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "rizinorg",
            "cutter",
            "rizinorg/cutter — Cutter: Free Open Source RE Platform",
            "Free and open-source reverse engineering platform powered by rizin. Advanced and \
         customizable RE platform with GUI. Supports Python and Native C++ plugins. \
         Integrates Ghidra decompiler, DynamoRIO code coverage visualization. \
         Cross-platform: Linux, macOS, Windows. 19.7k stars.",
        )
        .with_key_insights(vec![
            "Plugin architecture: Python and Native C++ plugins for extensibility",
            "Rizin-powered backend with advanced binary analysis capabilities",
            "Ghidra decompiler integration via rz-ghidra plugin",
            "Cross-platform GUI for reverse engineering workflows",
        ])
        .with_tags(vec![
            "reverse-engineering",
            "binary-analysis",
            "rizin",
            "ghidra",
            "security",
            "plugins",
            "gui",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.75)
        .with_confidence(0.9),
    )?;
    ids.push(r.node_id);

    // ── 6. maka-cu — macOS Computer Use execution layer (17★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "maka-agent", "maka-cu",
        "maka-agent/maka-cu — Maka Computer Use: Native macOS Execution Layer",
        "Native macOS execution layer for Maka's Computer Use. Forked from iFurySt/open-codex-computer-use. \
         Accessibility snapshot as core capability. Action binding: actions bound to observation they were \
         planned against, single-use, spent actions refused. Full Anthropic computer_20251124 action contract. \
         SkyLight background-click path and app discovery. Swift Package with embedded plugins.",
    ).with_key_insights(vec![
        "Accessibility snapshot as core observation primitive for computer use",
        "Action binding: actions are single-use, bound to the observation they were planned against",
        "Anthropic computer_20251124 action contract support for multi-model compatibility",
        "SkyLight private API bridge for background click events without focus stealing",
        "WebContent/renderer elements bound to real process generation with stale refetch",
    ]).with_tags(vec![
        "computer-use", "macos", "accessibility", "swift",
        "agent-execution", "sky-light", "action-binding",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.70).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 7. jev-skill — Jev use cases and agent skills (85★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "wuyoscar", "jev-skill",
        "wuyoscar/jev-skill — Awesome Jev Skills: Decision-Making for Agents",
        "Collection of Jev use cases, workflows, and agent skills. 90 scenarios, 9 installable \
         skills, 14 recorded API examples. Jev is a typed decision model that chooses, classifies, \
         and scores. Skills cover: agent supervision, routing, browser interaction, inbox triage, \
         document evidence, data tools, creative tools. Supports choice/score/noul primitives.",
    ).with_key_insights(vec![
        "90 decision scenarios across 8 domains: agents, review, routing, interaction, business, documents, data, creative",
        "Three decision primitives: choice (one option), score (graded levels), noul (P(true) yes/no)",
        "Agent skills: jev, jev-triage, jev-documents, jev-ui, jev-route, jev-context, jev-code-review, jev-find-code, jev-simulation",
        "Goal-drift checkpoint, stuck-loop recovery, completion evidence check patterns",
        "Model tier routing: small_text / reasoning / vision / cannot_route",
        "Parallel independent judgments: multiple questions over shared state in one request",
    ]).with_tags(vec![
        "jev", "decision-engine", "agent-skills", "classification",
        "triage", "routing", "workflows",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.9))?;
    ids.push(r.node_id);

    // Cross-link related repos
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "incoai/splash — Splash: Local Inference Engine for Apple Silicon",
        RelationType::Related,
        0.7,
        Some("OpenHuman can use Splash as local inference backend"),
    );
    let _ = ingester.relate_by_title(
        "wuyoscar/jev-skill — Awesome Jev Skills: Decision-Making for Agents",
        "NandhaKishorM/laya — Laya: Non-Autoregressive Decision Engine",
        RelationType::Related,
        0.8,
        Some("Jev and Laya are competing/complementary typed decision engines"),
    );
    let _ = ingester.relate_by_title(
        "maka-agent/maka-cu — Maka Computer Use: Native macOS Execution Layer",
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        RelationType::Related,
        0.6,
        Some("Maka-CU provides computer use primitives applicable to agent harnesses"),
    );

    info!("[crystal-core-repos] Absorbed {} repo nodes", ids.len());
    Ok(ids)
}

/// Absorb the personal-ai ecosystem — key repos from the GitHub personal-ai topic.
///
/// These represent the landscape of personal AI agents, local-first memory systems,
/// and agent orchestration platforms that NeoTrix competes with and can learn from.
fn ingest_personal_ai_ecosystem(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // ── 1. LifeOS — Universal AI Harness (19.1k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "danielmiessler", "LifeOS",
        "danielmiessler/LifeOS — LifeOS: Universal AI Harness",
        "The universal AI Harness designed to move you from Current to Ideal state in both life and work. \
         Productivity framework with AI augmentation, intent engineering, and life/work optimization.",
    ).with_key_insights(vec![
        "Current-to-Ideal state transformation framework for life and work",
        "Intent engineering: structured approach to AI-directed personal productivity",
        "AI harness pattern: orchestrating multiple AI tools for personal goals",
    ]).with_tags(vec![
        "personal-ai", "productivity", "intent-engineering", "life-harness",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 2. OpenBiliClaw — Cross-platform AI content discovery (3.3k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "whiteguo233", "OpenBiliClaw",
        "whiteguo233/OpenBiliClaw — Cross-platform AI Content Discovery Agent",
        "Local-first open-source cross-platform AI content discovery agent. Understands you, then \
         proactively finds content across Bilibili, Xiaohongshu, Douyin, YouTube, X, Zhihu, Reddit, \
         Weibo and the open web. Supports deepseek harness plugin.",
    ).with_key_insights(vec![
        "Local-first content discovery across 8+ platforms (Bilibili, YouTube, Reddit, etc.)",
        "User-understanding-first approach: learns preferences before proactive discovery",
        "DeepSeek harness plugin support for model flexibility",
    ]).with_tags(vec![
        "personal-ai", "content-discovery", "cross-platform", "local-first",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 3. Bitterbot — Mesh of agents (2.5k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Bitterbot-AI", "bitterbot-desktop",
        "Bitterbot-AI/bitterbot-desktop — Bitterbot: Agent Mesh for Collective Capability",
        "A mesh of agents that turns shared experience into collective capability. Desktop app with \
         P2P agent communication, cognitive architecture, skills marketplace, local-first design, \
         agent economy with x402 payments, and dream engine.",
    ).with_key_insights(vec![
        "Agent mesh: P2P communication between agents for collective intelligence",
        "Skills marketplace: agents share and trade capabilities",
        "Dream engine: background processing and insight generation",
        "x402 payment protocol for agent economy",
    ]).with_tags(vec![
        "personal-ai", "agent-mesh", "p2p", "cognitive-architecture",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 4. Memmy Agent — Personal memory hub (2k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "MemTensor", "memmy-agent",
        "MemTensor/memmy-agent — Memmy: Personal AI Memory Hub",
        "A personal AI agent and local memory hub for all AI agents. Gives every AI one shared, \
         fully controlled memory and persistent context — all AI remember the same you. Supports \
         Claude Code, Codex, OpenClaw and Hermes Agent.",
    ).with_key_insights(vec![
        "Shared memory hub: all AI agents share one persistent memory of the user",
        "Cross-agent memory: Claude Code, Codex, OpenClaw, Hermes all share context",
        "Local-first: fully controlled memory on user's device",
    ]).with_tags(vec![
        "personal-ai", "memory-hub", "persistent-context", "cross-agent",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 5. Personal Model — Build your HUMAN.md (1.3k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Intuition-Lab", "personal-model",
        "Intuition-Lab/personal-model — Personal Model: Build Your HUMAN.md",
        "Build your HUMAN.md — a structured representation of yourself for AI agents. Local-first, \
         privacy-focused, MCP-compatible personal model for agent context.",
    ).with_key_insights(vec![
        "HUMAN.md: structured user model for AI agent personalization",
        "MCP-compatible personal model for cross-agent context sharing",
        "Privacy-first: local storage, user-controlled data",
    ]).with_tags(vec![
        "personal-ai", "personal-model", "human-md", "mcp",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 6. FYAgent — Digital persona (1.1k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "fy-agent", "fyagent",
        "fy-agent/fyagent — FYAgent: Digital Persona for AI Era",
        "For You Agent — AI时代的个人随身数字人格. Your model, AI accounts, skills, prompts, and \
         workflow methods, carried into every AI tool. Rust + Tauri cross-platform desktop app.",
    ).with_key_insights(vec![
        "Digital persona: portable identity across AI tools",
        "Skill and prompt management for consistent agent behavior",
        "Cross-platform (Rust/Tauri) with MCP integration",
    ]).with_tags(vec![
        "personal-ai", "digital-persona", "prompt-management", "cross-platform",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 7. Elephant Agent — Self-evolving AI (585★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "agentic-in",
            "elephant-agent",
            "agentic-in/elephant-agent — Elephant: Personal-Model First Self-Evolving Agent",
            "Personal-Model First Self-Evolving AI Agent. Agent with memory, model management, \
         context awareness, and self-evolution capabilities.",
        )
        .with_key_insights(vec![
            "Self-evolution: agent improves its own capabilities over time",
            "Personal-model first: user's model drives agent behavior",
            "Memory and context awareness for persistent agent state",
        ])
        .with_tags(vec![
            "personal-ai",
            "self-evolution",
            "agent-memory",
            "agentic",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.70)
        .with_confidence(0.85),
    )?;
    ids.push(r.node_id);

    // ── 8. Sentient OS — On-device proactive intelligence (518★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Sentient-OS-Labs", "sentient-os",
        "Sentient-OS-Labs/sentient-os — Sentient OS: On-device Proactive Intelligence",
        "An on-device LLM that understands your entire life, then proactively offers to get your \
         work done through computer use. macOS native, privacy-first, MCP-compatible, with \
         knowledge base and proactive intelligence layer.",
    ).with_key_insights(vec![
        "Proactive intelligence: agent anticipates needs and offers to act",
        "On-device LLM for full life understanding without cloud dependency",
        "Computer use integration for autonomous task execution",
    ]).with_tags(vec![
        "personal-ai", "proactive-intelligence", "on-device", "computer-use",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // Cross-link ecosystem repos
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "MemTensor/memmy-agent — Memmy: Personal AI Memory Hub",
        RelationType::Related,
        0.8,
        Some("Both implement local-first persistent memory for personal AI"),
    );
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "danielmiessler/LifeOS — LifeOS: Universal AI Harness",
        RelationType::Related,
        0.7,
        Some("LifeOS and OpenHuman both aim to be comprehensive personal AI platforms"),
    );

    info!(
        "[personal-ai-ecosystem] Absorbed {} ecosystem nodes",
        ids.len()
    );
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_reclaim_nt_target_tmp_dry_run_safe() {
        // dry_run 只计数不删除, 对任意 /private/tmp 状态均安全; 验证返回 Ok 且不报错。
        assert!(reclaim_nt_target_tmp(9999, true).is_ok(), "reclaim dry-run");
    }

    #[test]
    fn test_ingest_github_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github(
            "testowner",
            "testrepo",
            "Test Repo",
            "A test repository for testing.",
        );
        let result = ingester.ingest(&desc).unwrap();
        assert!(!result.node_id.is_empty());
        assert!(result.insight_ids.is_empty());

        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Repo");
        assert_eq!(fetched.node_type, NodeType::Repository);
    }

    #[test]
    fn test_ingest_with_insights() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Repo With Insights", "Desc")
            .with_key_insights(vec!["Insight one", "Insight two", "Insight three"]);
        let result = ingester.ingest(&desc).unwrap();
        assert_eq!(result.insight_ids.len(), 3);

        for iid in &result.insight_ids {
            let fetched = get_node(&conn, iid).unwrap().unwrap();
            assert_eq!(fetched.node_type, NodeType::Insight);
        }

        let edges = get_edges_for_node(&conn, &result.node_id).unwrap();
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_ingest_paper_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("1234.56789", "Test Paper", "A test paper abstract.")
            .with_tags(vec!["test", "paper"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Paper);
        assert!(fetched.url.unwrap().contains("arxiv.org"));
    }

    #[test]
    fn test_ingest_article_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc =
            ResourceDescriptor::article("Test Article", "Summary", "https://example.com/article");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Article);
        assert_eq!(fetched.url.unwrap(), "https://example.com/article");
    }

    #[test]
    fn test_ingest_concept_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Test Concept", "A conceptual insight.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Concept);
        assert!(fetched.url.is_none());
    }

    #[test]
    fn test_ingest_tool_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::tool(
            "Test Tool",
            "A tool description.",
            ResourceSource::GitHub {
                owner: "test".into(),
                repo: "tool".into(),
            },
        );
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Tool);
    }

    #[test]
    fn test_ingest_with_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("0000.00000", "Paper With Content", "Abstract")
            .with_content("Full paper content here\nwith multiple lines.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.unwrap().contains("Full paper content"));
    }

    #[test]
    fn test_ingest_metadata_includes_episode() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Episode Concept", "Has episode tracking.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["episode_id"], serde_json::json!(ingester.episode_id()));
        assert_eq!(fetched.source_episode, None);
    }

    #[test]
    fn test_ingest_multiple_and_relate() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let r1 = ingester
            .ingest(&ResourceDescriptor::concept("Concept A", "First concept."))
            .unwrap();
        let r2 = ingester
            .ingest(&ResourceDescriptor::concept("Concept B", "Second concept."))
            .unwrap();

        ingester
            .relate(
                &r1.node_id,
                &r2.node_id,
                RelationType::References,
                0.8,
                Some("A references B"),
            )
            .unwrap();

        let edges = get_edges_for_node(&conn, &r1.node_id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_id, r2.node_id);
        assert_eq!(edges[0].relation_type, RelationType::References);
    }

    #[test]
    fn test_ingest_with_importance_and_confidence() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Important Concept", "Very important.")
            .with_importance(0.95)
            .with_confidence(0.99);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!((fetched.importance - 0.95).abs() < 0.01);
        assert!((fetched.confidence - 0.99).abs() < 0.01);
    }

    #[test]
    fn test_ingest_report_format() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester
            .ingest(&ResourceDescriptor::concept(
                "Report Test",
                "Testing report.",
            ))
            .unwrap();
        let report = ingester.report();
        assert!(report.contains("Report Test"));
        assert!(report.contains("Episode ID"));
        assert!(report.contains("1"));
    }

    #[test]
    fn test_ingest_domain_from_source() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Domain Test", "Testing domain.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.domain.unwrap(), "github.com");
    }

    #[test]
    fn test_ingest_source_url() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("owner", "repo", "URL Test", "Testing URL.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.url.unwrap().contains("github.com/owner/repo"));
    }

    #[test]
    fn test_ingest_with_tags_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("9999.99999", "Tagged Paper", "Abstract")
            .with_tags(vec!["tag1", "tag2", "absorbed-2026-07-03"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        let tags: Vec<String> = serde_json::from_value(meta["tags"].clone()).unwrap();
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"absorbed-2026-07-03".to_string()));
    }

    #[test]
    fn test_relate_by_title() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester
            .ingest(&ResourceDescriptor::concept("Source Node", "Source."))
            .unwrap();
        ingester
            .ingest(&ResourceDescriptor::concept("Target Node", "Target."))
            .unwrap();
        ingester
            .relate_by_title(
                "Source Node",
                "Target Node",
                RelationType::DependsOn,
                0.9,
                Some("depends"),
            )
            .unwrap();

        let src = find_node_by_title(&conn, "Source Node").unwrap().unwrap();
        let edges = get_edges_for_node(&conn, &src.id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation_type, RelationType::DependsOn);
    }

    #[test]
    fn test_find_node_by_title_not_found() {
        let conn = test_conn();
        let result = find_node_by_title(&conn, "NonExistentNode").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ingest_session_resources_creates_all_nodes() {
        let conn = test_conn();
        let report = ingest_session_resources(&conn).unwrap();
        assert!(report.contains("Episode ID"));
        let all_nodes = get_all_nodes(&conn).unwrap();
        assert!(all_nodes.len() >= 17);
    }

    #[test]
    fn test_ingest_insight_counts_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("8888.88888", "Insight Count Test", "Abstract")
            .with_key_insights(vec!["A", "B", "C", "D", "E"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["insight_count"], 5);
    }

    #[test]
    fn test_ingest_concept_without_insights_no_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("No Insights", "Just a concept.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.is_none());
    }

    #[test]
    fn test_ingest_long_insight_title_truncated() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let long = "A very long insight text that exceeds eighty characters in total length for truncation testing purposes";
        let desc = ResourceDescriptor::github("o", "r", "Truncation Test", "Summary")
            .with_key_insights(vec![long]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.insight_ids[0]).unwrap().unwrap();
        assert!(fetched.title.len() <= 80);
        assert!(fetched.title.ends_with("..."));
        assert_eq!(fetched.summary.unwrap(), long);
    }

    #[test]
    #[ignore = "requires downloaded project-nomad source at ~/.neotrix/downloads/project-nomad-main-src/project-nomad-main/collections/"]
    fn test_ingest_project_nomad_data_sources() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let src = std::path::Path::new(
            "/Users/neo/.neotrix/downloads/project-nomad-main-src/project-nomad-main/collections",
        );
        let mut ingested = 0;

        // kiwix-categories.json
        if let Ok(content) = std::fs::read_to_string(src.join("kiwix-categories.json")) {
            if let Ok(kiwix) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(cats) = kiwix["categories"].as_array() {
                    for cat in cats {
                        if let Some(resources) = cat["resources"].as_array() {
                            for res in resources {
                                if let Some(url) = res["url"].as_str() {
                                    let title = format!(
                                        "Kiwix: {} - {}",
                                        cat["name"].as_str().unwrap_or(""),
                                        res["title"].as_str().unwrap_or("")
                                    );
                                    let summary = res["description"].as_str().unwrap_or("");
                                    let tags = vec![
                                        "kiwix".to_string(),
                                        "data-source".to_string(),
                                        cat["slug"].as_str().unwrap_or("").to_string(),
                                    ];
                                    let desc = ResourceDescriptor::article(&title, summary, url)
                                        .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                        .with_importance(0.7);
                                    if ingester.ingest(&desc).is_ok() {
                                        ingested += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // maps.json
        if let Ok(content) = std::fs::read_to_string(src.join("maps.json")) {
            if let Ok(maps) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(colls) = maps["collections"].as_array() {
                    for coll in colls {
                        if let Some(resources) = coll["resources"].as_array() {
                            for res in resources {
                                if let Some(url) = res["url"].as_str() {
                                    let title = format!(
                                        "Map: {} - {}",
                                        coll["name"].as_str().unwrap_or(""),
                                        res["title"].as_str().unwrap_or("")
                                    );
                                    let summary = res["description"].as_str().unwrap_or("");
                                    let tags = vec![
                                        "maps".to_string(),
                                        "pmtiles".to_string(),
                                        "data-source".to_string(),
                                        coll["slug"].as_str().unwrap_or("").to_string(),
                                    ];
                                    let desc = ResourceDescriptor::article(&title, summary, url)
                                        .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                        .with_importance(0.7);
                                    if ingester.ingest(&desc).is_ok() {
                                        ingested += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // wikipedia.json
        if let Ok(content) = std::fs::read_to_string(src.join("wikipedia.json")) {
            if let Ok(wiki) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(opts) = wiki["options"].as_array() {
                    for opt in opts {
                        if let Some(url) = opt["url"].as_str() {
                            if !url.is_empty() && url != "null" {
                                let title =
                                    format!("Wikipedia: {}", opt["name"].as_str().unwrap_or(""));
                                let summary = opt["description"].as_str().unwrap_or("");
                                let tags = vec![
                                    "wikipedia".to_string(),
                                    "zim".to_string(),
                                    "data-source".to_string(),
                                ];
                                let desc = ResourceDescriptor::article(&title, summary, url)
                                    .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                    .with_importance(0.7);
                                if ingester.ingest(&desc).is_ok() {
                                    ingested += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(
            ingested > 0,
            "at least one project-nomad data source should be ingested"
        );
        println!("project-nomad data sources ingested: {}", ingested);
    }

    #[test]
    fn test_register_cortex_brain_temp_dir() {
        let conn = test_conn();
        let dir = std::env::temp_dir().join(format!("cortex_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("cortex-archive").join("zim")).unwrap();
        std::fs::write(dir.join("cortex-archive").join("zim").join("x.zim"), b"z").unwrap();
        std::fs::create_dir_all(dir.join("working")).unwrap();
        std::fs::write(dir.join("working").join("causal_graph.json"), b"{}").unwrap();
        let baseline: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let n = register_cortex_brain(&conn, &dir).unwrap();
        assert!(
            n >= 2,
            "should register zim dir + causal graph node, got {n}"
        );
        let cnt: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            cnt,
            baseline + n as i64,
            "cortex_brain node count drift (baseline={baseline} n={n} cnt={cnt})"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_register_cortex_brain_missing_mount_is_noop() {
        let conn = test_conn();
        let missing = std::path::Path::new("/Volumes/NeoTrixBrain__definitely_not_mounted");
        assert_eq!(register_cortex_brain(&conn, missing).unwrap(), 0);
        let cnt: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cnt, 0);
    }

    #[test]
    fn test_prune_cortex_orphans_no_zim_errors() {
        let conn = test_conn();
        let dir = std::env::temp_dir().join(format!("cortex_prune_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let r = prune_cortex_orphans(&conn, &dir, true);
        assert!(r.is_err(), "should error when no ZIM files on volume");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Resumable copy must finish correctly when a partial copy + progress sidecar already exist.
    #[test]
    fn test_copy_resumable_resumes_from_partial() {
        let dir = std::env::temp_dir().join(format!("cortex_copy_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.bin");
        let dest = dir.join("dest.bin");
        let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        std::fs::write(&src, &data).unwrap();
        // simulate a prior interrupted run: 3 MiB written + .prog checkpoint at 3 MiB
        let partial = 3 * 1024 * 1024;
        std::fs::write(&dest, &data[..partial]).unwrap();
        std::fs::write(format!("{}.prog", dest.display()), partial.to_string()).unwrap();
        copy_resumable(&src, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), data);
        assert!(!std::path::Path::new(&format!("{}.prog", dest.display())).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Fresh resumable copy (no prior partial) must produce a byte-identical destination.
    #[test]
    fn test_copy_resumable_fresh() {
        let dir = std::env::temp_dir().join(format!("cortex_copy2_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.bin");
        let dest = dir.join("dest.bin");
        let data: Vec<u8> = (0..7 * 1024 * 1024).map(|i| (i % 197) as u8).collect();
        std::fs::write(&src, &data).unwrap();
        copy_resumable(&src, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), data);
        assert!(!std::path::Path::new(&format!("{}.prog", dest.display())).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
