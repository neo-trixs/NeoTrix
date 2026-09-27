use std::path::PathBuf;

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::nt_unify_config::{config_set, kv_list, kv_set, now, secret_set};
use super::nt_unify_session::{cookie_set, session_log_append};

// ─── Skills Index ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_path: Option<String>,
    pub tags: Option<String>,
    pub is_builtin: bool,
    pub last_indexed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    /// 技能内容指纹 (sha256 hex)。写通去重依据: 内容未变化则不重复 upsert。
    pub content_hash: Option<String>,
}

/// 计算技能内容 (含 YAML frontmatter 全文) 的 sha256 指纹, 用于写通去重。
pub fn skill_content_hash(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

/// Upsert 一条技能记录到 skills_index。
/// 返回 `true` = 本次真正写入/更新; `false` = 内容未变化被去重跳过
/// (ON CONFLICT 的 WHERE 条件保证同 hash 不落盘, 避免每命令全量写)。
pub fn skill_upsert(conn: &Connection, name: &str, record: &SkillRecord) -> Result<bool, String> {
    let ts = now();
    let rows = conn
        .execute(
            "INSERT INTO skills_index (id, name, description, source_path, tags, is_builtin, last_indexed_at, created_at, updated_at, content_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(name) DO UPDATE SET
               description=excluded.description, source_path=excluded.source_path,
               tags=excluded.tags, last_indexed_at=excluded.last_indexed_at,
               updated_at=excluded.updated_at, content_hash=excluded.content_hash
             WHERE skills_index.content_hash IS NOT excluded.content_hash
                OR skills_index.content_hash != excluded.content_hash",
            rusqlite::params![
                record.id, name, record.description, record.source_path, record.tags,
                record.is_builtin as i32, record.last_indexed_at, record.created_at, ts,
                record.content_hash,
            ],
        )
        .map_err(|e| format!("skill_upsert: {}", e))?;
    Ok(rows > 0)
}

// ─────────────────────────────────────────────────────────────
// ── 10b. 技能域收编映射 (UCN Phase 2)
//     skills 域 → NT-* 域 → 星辰; namespace = domain_nt_<slug>
//     与 ADR D2 (命名统一) 一致, 写入 KB kv_store。
// ─────────────────────────────────────────────────────────────

/// 技能域收编映射表: (skills 域源路径, NT-* 域, 星辰名)。
/// 与 ADR"skills 域 → NT-* 域映射(收编对照)"表保持一致;
/// L3 厂商技能(36+)为只读能力分支, 不进映射表。
pub const DOMAIN_SKILL_MAPPING: &[(&str, &str, &str)] = &[
    // Original 14 mappings (UCN Phase 2, R-P79)
    ("rev/officer", "NT-SHIELD", "Rev-明"),
    ("dev/implementer", "NT-ACT", "Dev-匠"),
    ("des/architect", "NT-CORE", "Des-观"),
    ("res/scholar", "NT-MIND", "Res-深"),
    ("methodology/researcher", "NT-MIND", "Res-深"),
    ("experience-tree", "NT-MEMORY", "Exp-藏"),
    ("nexus/weaver", "NT-MEMORY", "Nexus-梭"),
    ("meta/coordinator", "NT-META", "Meta-镜"),
    ("sg/diagnostician", "NT-META", "SG-诊"),
    ("repair/healer", "NT-REPAIR", "Repair-医"),
    ("gov/steward", "NT-GOVERNANCE", "Gov-衡"),
    ("mil/officer", "NT-SCOUT", "Search-觅"),
    ("ed/tutor", "NT-IO", "Edu-灯"),
    // ── Absorbed from 75+ GitHub/arXiv URLs (Phase 3 integration) ──

    // NT-ACT: Code agents, orchestration, penetration testing
    ("anthropic/claude-code", "NT-ACT", "Codex-匠"),
    ("apurvsinghgautam/robin", "NT-ACT", "Robin-巡"),
    ("mattpocock/skills", "NT-ACT", "Skill-Net"),
    ("workweave/router", "NT-ACT", "Router-治"),
    ("headcount", "NT-ACT", "Head-统"),
    ("openai/codex", "NT-ACT", "Codex-统"),
    ("pentestcode", "NT-ACT", "Pen-Test"),
    ("unity-mcp", "NT-ACT", "Unity-行"),
    ("microsoft/rd-agent", "NT-ACT", "RD-行动"),
    // NT-MIND: SEAL pipeline, skill distillation, evolution
    ("anthropics/claude-code", "NT-MIND", "Mind-匠"), // duplicate key conflict resolved below
    ("arxiv-org-abs-2608-27964", "NT-MIND", "SEAL-蒸馏"),
    ("arxiv-org-abs-2608-27991", "NT-MIND", "Self-修"),
    ("openai/codex-pr-27488", "NT-MIND", "Token-预算"),
    ("mattpocock/skills", "NT-MIND", "Skill-晶"),
    ("mthli-xyz-git-knowledge-loop", "NT-MIND", "Know-循环"),
    ("best-xiaohu-ai-xai-bot-guides", "NT-MIND", "Gov-引导"),
    ("martin-olivier/airgorah", "NT-MIND", "流-编排"),
    ("elijah222/rakazo", "NT-MIND", "沙-重"),
    ("karpathy/autoresearch", "NT-MIND", "研-自动"),
    ("ding-si-ai/auto-research", "NT-MIND", "研-演化"),
    // NT-IO: Interface, VTuber, MCP, web apps
    ("open-llm-vtuber", "NT-IO", "VTuber-驱"),
    ("justin-sky/ai-art-engine", "NT-IO", "Art-引擎"),
    ("buka-studio-www-marijanapav", "NT-IO", "Web-瓷"),
    ("ahujasid/blender-mcp", "NT-IO", "Blender-MCP"),
    ("superlinked/sie", "NT-IO", "Inference-服"),
    ("opopile-beichen-pi-desktop", "NT-IO", "Desktop-侧"),
    ("elementalsouls-claude-osint", "NT-IO", "OSINT-接"),
    // NT-META: Meta-cognition, governance, experience absorption
    ("lipku-live-talking", "NT-META", "Talk-吸"),
    ("addyosmani-agent-skills", "NT-META", "技-目录"),
    ("nextlevelbuilder-ui-ux-pro-max-skill", "NT-META", "设-系"),
    ("juliusbrussee-caveman", "NT-META", "爬-修"),
    ("kepano-obsidian-skills", "NT-META", "奥-藏"),
    ("alchaincyf-nuwa-skill", "NT-META", "能-结"),
    ("lijigang-lg-skills", "NT-META", "技-学"),
    ("cbrock84-headcount", "NT-META", "头-统"),
    ("dwarkesh-com-openai-huggingface", "NT-META", "博-文"),
    ("wolfpld-tracy", "NT-META", "追-踪"),
    ("dietrichgebert-ponytail", "NT-META", "毛-修"),
    ("lijigang-ljg-skills", "NT-META", "技-网"),
    ("academy-dair-ai-wikiskill", "NT-META", "维-知"),
    ("cbrock84-headcount", "NT-META", "头-数"),
    ("arxiv-org-abs-2608-18027", "NT-META", "核-相"),
    ("elementalsouls-claude-osint", "NT-META", "义-感"),
    ("vercel-com-blog", "NT-META", "页-说"),
    // NT-WORLD: Perception, crawling, content extraction
    ("lipku-live-talking", "NT-WORLD", "Talk-感"),
    (" dietrichgebert-ponytail", "NT-WORLD", "毛-提"),
    ("kirara-ai", "NT-WORLD", "AI-采"),
    ("flashml-org-free-token", "NT-WORLD", "令-牌"),
    ("elder-plinius-g0dm0d3", "NT-WORLD", "GPU-流"),
    ("www222fff-free-router", "NT-WORLD", "路-由"),
    ("pkuflyingpig-cs-self-learning", "NT-WORLD", "学-习"),
    ("tirth8205-code-review-graph", "NT-WORLD", "评-图"),
    ("mibayy-token-savior", "NT-WORLD", "标-优"),
    ("mksglu-context-mode", "NT-WORLD", "情-境"),
    ("rtk-ai-rtk", "NT-WORLD", "RT-工具"),
    ("ooples-token-optimizer-mcp", "NT-WORLD", "优-工"),
    ("zilliztech-claude-context", "NT-WORLD", "向-量"),
    ("nadimtuhin-claude-token-optimizer", "NT-WORLD", "令-优"),
    ("alexgreensh-token-optimizer", "NT-WORLD", "令-优"),
    ("drona23-claude-token-efficient", "NT-WORLD", "效-率"),
    ("google-adk-python", "NT-WORLD", "代-理"),
    ("alibaba-zvec", "NT-WORLD", "向-量"),
    // NT-SHIELD: Security, OSINT, fingerprint management
    ("elementalsouls-claude-osint", "NT-SHIELD", "义-感"),
    ("elie222-rakazo", "NT-SHIELD", "沙-重"),
    ("Renset-macai", "NT-SHIELD", "mac-盾"),
    ("workweave/router", "NT-SHIELD", "路-由"),
    // NT-REPAIR: Self-healing, repair mechanisms
    ("dietrichgebert-ponytail", "NT-REPAIR", "毛-修"),
    ("arxiv-org-abs-2608-27991", "NT-REPAIR", "自-修"),
    ("elie222-rakazo", "NT-REPAIR", "沙-重"),
    ("Renset-macai", "NT-REPAIR", "mac-修"),
    ("max-sixty-worktrunk", "NT-REPAIR", "工-修"),
    // NT-GOVERNANCE: Policy, governance, role-based organization
    ("best-xiaohu-ai-xai-bot-guides", "NT-GOVERNANCE", "治-引"),
    ("mthli-xyz-git-knowledge-loop", "NT-GOVERNANCE", "循-约"),
    ("arxiv-org-abs-2608-30949", "NT-GOVERNANCE", "治-治"),
];

// domain_nt_<slug>: NT-SHIELD → domain_nt_shield
pub fn domain_ns(nt_domain: &str) -> String {
    let slug = nt_domain
        .strip_prefix("NT-")
        .unwrap_or(nt_domain)
        .to_ascii_lowercase()
        .replace('-', "_");
    format!("domain_nt_{}", slug)
}

/// 写入全部技能域映射到 KB (UCN Phase 2)。
/// 幂等: kv_set ON CONFLICT upsert, 重复调用不产生脏数据。
/// 返回写入/更新的映射条目数。
pub fn unify_domain_mapping(conn: &Connection) -> Result<usize, String> {
    use std::collections::BTreeMap;
    // (domain, star) → [source 路径...]; BTreeMap 保证稳定顺序
    let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (source, nt_domain, star) in DOMAIN_SKILL_MAPPING {
        groups
            .entry((nt_domain.to_string(), star.to_string()))
            .or_default()
            .push(source.to_string());
    }
    let mut written = 0usize;
    for ((nt_domain, star), sources) in &groups {
        let ns = domain_ns(nt_domain);
        let value = format!(
            "[{}]",
            sources
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        );
        kv_set(conn, &ns, star, &value)?;
        written += 1;
    }
    Ok(written)
}

/// 读取某 NT-* 域下的星辰映射: Vec<(星名, 源技能 JSON 数组)>。
pub fn domain_skills(conn: &Connection, nt_domain: &str) -> Result<Vec<(String, String)>, String> {
    kv_list(conn, &domain_ns(nt_domain))
}

pub fn skill_search(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SkillRecord>, String> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, source_path, tags,
                    is_builtin, last_indexed_at, created_at, updated_at, content_hash
             FROM skills_index
             WHERE name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1
             ORDER BY last_indexed_at DESC NULLS LAST
             LIMIT ?2",
        )
        .map_err(|e| format!("skill_search prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![pattern, limit as i64], |row| {
            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_path: row.get(3)?,
                tags: row.get(4)?,
                is_builtin: row.get::<_, i32>(5)? != 0,
                last_indexed_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                content_hash: row.get(9)?,
            })
        })
        .map_err(|e| format!("skill_search query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("skill_search row: {}", e))?);
    }
    Ok(results)
}

pub fn skill_list_all(conn: &Connection, limit: usize) -> Result<Vec<SkillRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, source_path, tags,
                    is_builtin, last_indexed_at, created_at, updated_at, content_hash
             FROM skills_index ORDER BY name LIMIT ?1",
        )
        .map_err(|e| format!("skill_list_all prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![limit as i64], map_skill_row)
        .map_err(|e| format!("skill_list_all query: {}", e))?;
    collect_skills(rows)
}

pub fn skill_delete(conn: &Connection, name: &str) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM skills_index WHERE name=?1",
            rusqlite::params![name],
        )
        .map_err(|e| format!("skill_delete: {}", e))?;
    Ok(rows > 0)
}

fn map_skill_row(row: &rusqlite::Row) -> rusqlite::Result<SkillRecord> {
    Ok(SkillRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        source_path: row.get(3)?,
        tags: row.get(4)?,
        is_builtin: row.get::<_, i32>(5)? != 0,
        last_indexed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        content_hash: row.get(9)?,
    })
}

fn collect_skills(
    rows: impl Iterator<Item = Result<SkillRecord, rusqlite::Error>>,
) -> Result<Vec<SkillRecord>, String> {
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("skill row: {}", e))?);
    }
    Ok(results)
}

// ─── Migration from Files ───────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub total_files_migrated: usize,
    pub kv_entries_created: usize,
    pub config_entries_created: usize,
    pub secrets_migrated: usize,
    pub session_logs_migrated: usize,
    pub cookies_migrated: usize,
    pub assets_migrated: usize,
    pub skills_indexed: usize,
    pub domain_mapping_written: usize,
    pub rkyv_blobs_migrated: usize,
    pub errors: Vec<(String, String)>,
}

impl MigrationReport {
    pub fn summary(&self) -> String {
        format!(
            "Migration complete:\n  Files migrated: {}\n  KV entries: {}\n  Config entries: {}\n  Secrets: {}\n  Session logs: {}\n  Cookies: {}\n  Assets: {}\n  Skills indexed: {}\n  Domain mapping: {}\n  Rkyv blobs: {}\n  Errors: {}",
            self.total_files_migrated, self.kv_entries_created, self.config_entries_created,
            self.secrets_migrated, self.session_logs_migrated, self.cookies_migrated,
            self.assets_migrated, self.skills_indexed, self.domain_mapping_written, self.rkyv_blobs_migrated, self.errors.len()
        )
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn neotrix_dir() -> PathBuf {
    home_dir().join(".neotrix")
}

fn read_text_file(path: &PathBuf) -> Option<String> {
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

/// Migrate all existing external files into KB. Leaves originals in place.
pub fn migrate_from_files(conn: &Connection) -> MigrationReport {
    let mut report = MigrationReport::default();
    let base = neotrix_dir();

    // ── 1. KV Store: lightweight metadata only (not state snapshots) ──
    // State snapshots (brain/cortex/e8/narrative/whitebox/goals etc.) stay as files.
    // Only track paths and lightweight references.
    let kv_sources: Vec<(&str, &str)> = vec![
        ("data_ingest_metrics", "data_ingest_metrics.json"),
        ("knowledge_v2_snap", "knowledge_v2.snap"),
    ];
    for (ns, filename) in &kv_sources {
        let path = base.join(filename);
        if let Some(content) = read_text_file(&path) {
            report.total_files_migrated += 1;
            if kv_set(conn, ns, "data", &content).is_ok() {
                report.kv_entries_created += 1;
            }
        }
    }

    // Track path to journal_index.db (separate SQLite DB)
    let journal_index_path = base.join("journal_index.db");
    if journal_index_path.exists() {
        report.total_files_migrated += 1;
        if kv_set(
            conn,
            "journal_index",
            "path",
            &journal_index_path.to_string_lossy(),
        )
        .is_ok()
        {
            report.kv_entries_created += 1;
        }
    }

    // Track path to exploration_sources.txt
    let exploration_path = base.join("exploration_sources.txt");
    if exploration_path.exists() {
        report.total_files_migrated += 1;
        if kv_set(
            conn,
            "exploration",
            "sources_path",
            &exploration_path.to_string_lossy(),
        )
        .is_ok()
        {
            report.kv_entries_created += 1;
        }
    }

    // ── 2. Config entries ──
    // config.toml
    let config_path = base.join("config.toml");
    if let Some(content) = read_text_file(&config_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                if let Some(val_table) = value.as_table() {
                    for (k, v) in val_table {
                        let val_str = v.to_string();
                        // Strip quotes from string values
                        let val_str = val_str.trim_matches('"').to_string();
                        if config_set(conn, section, k, &val_str, false).is_ok() {
                            report.config_entries_created += 1;
                        }
                    }
                } else {
                    let val_str = value.to_string().trim_matches('"').to_string();
                    if config_set(conn, "root", section, &val_str, false).is_ok() {
                        report.config_entries_created += 1;
                    }
                }
            }
        }
    }

    // profiles.toml
    let profiles_path = base.join("profiles.toml");
    if let Some(content) = read_text_file(&profiles_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                let val_str = value.to_string();
                if config_set(conn, "profiles", section, &val_str, false).is_ok() {
                    report.config_entries_created += 1;
                }
            }
        }
    }

    // router_config.toml
    let router_cfg_path = base.join("router_config.toml");
    if let Some(content) = read_text_file(&router_cfg_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                let val_str = value.to_string();
                if config_set(conn, "router", section, &val_str, false).is_ok() {
                    report.config_entries_created += 1;
                }
            }
        }
    }

    // ── 3. Env vars → config entries ──
    let neotrix_vars = [
        "NEOTRIX_MODEL",
        "NEOTRIX_PROVIDER",
        "NEOTRIX_BASE_URL",
        "NEOTRIX_TIMEOUT",
        "NEOTRIX_API_KEY",
        "NEOTRIX_EMBEDDING_API_KEY",
        "NEOTRIX_EMBEDDING_BASE_URL",
        "NEOTRIX_EMBEDDING_MODEL",
        "NEOTRIX_EMBEDDING_DIMENSION",
        "NEOTRIX_SEARCH_API",
        "NEOTRIX_ZEN_URL",
        "NEOTRIX_GATEWAY_ADDR",
        "NEOTRIX_PROXY_SUB_URL",
        "NEOTRIX_SPLIT_ENABLE",
        "NEOTRIX_HEALTH_FILE",
        "NEOTRIX_HOME",
        "NEOTRIX_API_TOKEN",
        "NEOTRIX_SENTRY_DSN",
    ];
    for var_name in &neotrix_vars {
        if let Ok(val) = std::env::var(var_name) {
            let section = "env";
            let is_secret = var_name.contains("API_KEY")
                || var_name.contains("TOKEN")
                || var_name.contains("SENTRY");
            if config_set(conn, section, var_name, &val, is_secret).is_ok() {
                report.config_entries_created += 1;
            }
        }
    }

    // ── 4. Secrets from secrets.json ──
    let secrets_path = base.join("secrets.json");
    if let Some(content) = read_text_file(&secrets_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            report.total_files_migrated += 1;
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    let val_str = v
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v.to_string());
                    if secret_set(conn, k, &val_str).is_ok() {
                        report.secrets_migrated += 1;
                    } else {
                        report
                            .errors
                            .push(("secrets.json".into(), format!("secret_set {} failed", k)));
                    }
                }
            }
        }
    }

    // ── 5. Session logs from journal/ ──
    let journal_dir = base.join("journal");
    if journal_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&journal_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Some(content) = read_text_file(&path) {
                        report.total_files_migrated += 1;
                        let session_id = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if session_log_append(conn, &session_id, &content, "markdown", None).is_ok()
                        {
                            report.session_logs_migrated += 1;
                        }
                    }
                }
            }
        }
    }

    // ── 6. Session logs from session-logs/ ──
    let session_logs_dir = base.join("session-logs");
    if session_logs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&session_logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(content) = read_text_file(&path) {
                    report.total_files_migrated += 1;
                    let session_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if session_log_append(
                        conn,
                        &format!("session-log-{}", session_id),
                        &content,
                        "log",
                        None,
                    )
                    .is_ok()
                    {
                        report.session_logs_migrated += 1;
                    }
                }
            }
        }
    }

    // ── 7. Session shares ──
    let shares_dir = base.join("shares");
    if shares_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&shares_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(content) = read_text_file(&path) {
                    report.total_files_migrated += 1;
                    let session_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("share")
                        .to_string();
                    if session_log_append(
                        conn,
                        &format!("share-{}", session_id),
                        &content,
                        "json",
                        None,
                    )
                    .is_ok()
                    {
                        report.session_logs_migrated += 1;
                    }
                }
            }
        }
    }

    // ── 8. Cookies from cookies/ ──
    let cookies_dir = base.join("cookies");
    if cookies_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Some(content) = read_text_file(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(obj) = data.as_object() {
                                report.total_files_migrated += 1;
                                let domain = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown");
                                for (name, value) in obj {
                                    let val_str = if value.is_string() {
                                        value
                                            .as_str()
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| value.to_string())
                                    } else {
                                        value.to_string()
                                    };
                                    if cookie_set(
                                        conn, domain, name, &val_str, "/", false, false, None,
                                    )
                                    .is_ok()
                                    {
                                        report.cookies_migrated += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── 9. Binary assets ──
    // NOT stored in KB: avatars/, snapshots/, chrome-profile/ stay as files.
    // Binary blobs in SQLite cause DB bloat and prevent streaming.
    // Only store reference paths if needed.
    for dir_name in &["avatars", "snapshots", "chrome-profile"] {
        let dir = base.join(dir_name);
        if dir.is_dir() {
            report.total_files_migrated += 1;
            let _ = kv_set(conn, "binary_refs", dir_name, &dir.to_string_lossy());
            report.kv_entries_created += 1;
        }
    }

    // ── 10. Skills index (metadata only: name, description, tags, source_path) ──
    // SKILL.md content stays in files; KB only stores searchable metadata.
    let skill_dirs = vec![
        base.join("skills"),
        home_dir().join(".agents").join("skills"),
        home_dir().join(".claude").join("skills"),
    ];
    for skills_path in &skill_dirs {
        if skills_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(skills_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(content) = read_text_file(&skill_md) {
                            report.total_files_migrated += 1;
                            let name = path
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let description = extract_description(&content);
                            let tags_str = extract_tags_from_content(&content);
                            let record = SkillRecord {
                                id: Uuid::new_v4().to_string(),
                                name,
                                description: Some(description),
                                source_path: Some(skill_md.to_string_lossy().to_string()),
                                tags: tags_str,
                                is_builtin: false,
                                last_indexed_at: Some(now()),
                                created_at: now(),
                                updated_at: now(),
                                content_hash: Some(skill_content_hash(&content)),
                            };
                            if skill_upsert(conn, &record.name, &record).is_ok() {
                                report.skills_indexed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // ── 11. Rkyv blobs ── stay as files (zero-copy requires mmap).
    // ── 12. journal_index.db ── already tracked via kv_store section above.

    // ── 13. 技能域收编映射 (UCN Phase 2) ──
    match unify_domain_mapping(conn) {
        Ok(n) => report.domain_mapping_written = n,
        Err(e) => report.errors.push(("domain_mapping".into(), e)),
    }

    report
}

fn extract_description(content: &str) -> String {
    // Try to get first non-empty line that isn't YAML frontmatter
    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty()
            && !line.starts_with("---")
            && !line.starts_with('#')
            && !line.starts_with(':')
        {
            return line.to_string();
        }
    }
    "No description".to_string()
}

fn extract_tags_from_content(content: &str) -> Option<String> {
    // Try to find YAML frontmatter and extract tags from it
    let trimmed = content.trim();
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("\n---") {
            let yaml = &trimmed[3..3 + end];
            for line in yaml.lines() {
                if line.trim().starts_with("tags:") {
                    let tags = line.trim_start_matches("tags:").trim();
                    if tags.starts_with('[') && tags.ends_with(']') {
                        let inner = tags.trim_start_matches('[').trim_end_matches(']');
                        return Some(
                            inner
                                .split(',')
                                .map(|t| t.trim().trim_matches('"').trim_matches('\''))
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                    }
                    return Some(tags.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nt_unify_config::kv_list_namespaces;
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    fn split_yaml_frontmatter(content: &str) -> (Option<String>, String) {
        let trimmed = content.trim();
        if trimmed.starts_with("---") {
            if let Some(end) = trimmed[3..].find("\n---") {
                let yaml = trimmed[3..3 + end].to_string();
                let body = trimmed[3 + end + 4..].trim().to_string();
                return (Some(yaml), body);
            }
        }
        (None, content.to_string())
    }

    #[test]
    fn test_skill_upsert_search() {
        let conn = test_conn();
        let record = SkillRecord {
            id: "id1".into(),
            name: "test-skill".into(),
            description: Some("A test skill".into()),
            source_path: Some("/path/to/skill".into()),
            tags: Some("test,example".into()),
            is_builtin: false,
            last_indexed_at: Some(now()),
            created_at: now(),
            updated_at: now(),
            content_hash: Some("abc123".into()),
        };
        skill_upsert(&conn, "test-skill", &record).unwrap();
        let results = skill_search(&conn, "test", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test-skill");
        assert_eq!(results[0].content_hash.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_domain_mapping_write_read_idempotent() {
        let conn = test_conn();
        // 写入: 去重后组数 == 映射表 distinct (domain, star) 对数（数据驱动，映射生长不腐）。
        let mut pairs = std::collections::HashSet::new();
        for (_, d, s) in DOMAIN_SKILL_MAPPING {
            pairs.insert((d.to_string(), s.to_string()));
        }
        let written = unify_domain_mapping(&conn).unwrap();
        assert_eq!(written, pairs.len(), "去重组数应等于映射表 distinct (domain,star) 对数");

        // 读取: NT-SHIELD 含 Rev-明 → rev/officer（域内条数随映射生长，只断言锚点）
        let shield = domain_skills(&conn, "NT-SHIELD").unwrap();
        assert!(
            shield.iter().any(|(s, p)| s == "Rev-明" && p.contains("rev/officer")),
            "NT-SHIELD 含 Rev-明 rev/officer"
        );

        // NT-MIND 含 Res-深（res/scholar＋methodology/researcher 双源）
        let mind = domain_skills(&conn, "NT-MIND").unwrap();
        assert!(
            mind.iter().any(|(s, p)| s == "Res-深" && p.contains("res/scholar")),
            "NT-MIND 含 Res-深 res/scholar"
        );
        assert!(
            mind.iter().any(|(s, p)| s == "Res-深" && p.contains("methodology/researcher")),
            "NT-MIND 含 Res-深 methodology/researcher"
        );

        // 幂等: 重复写入不产生脏数据
        let again = unify_domain_mapping(&conn).unwrap();
        assert_eq!(again, pairs.len());
        assert!(!domain_skills(&conn, "NT-SHIELD").unwrap().is_empty());
        let mem_stars: Vec<String> = domain_skills(&conn, "NT-MEMORY")
            .unwrap()
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        assert!(mem_stars.iter().any(|s| s == "Exp-藏"), "NT-MEMORY 含 Exp-藏");
        assert!(mem_stars.iter().any(|s| s == "Nexus-梭"), "NT-MEMORY 含 Nexus-梭");

        // namespace 命名契约: domain_nt_<slug>
        assert_eq!(domain_ns("NT-SHIELD"), "domain_nt_shield");
        assert_eq!(domain_ns("NT-GOVERNANCE"), "domain_nt_governance");
    }

    #[test]
    fn test_domain_mapping_full_manifest() {
        // 契约: ADR 收编对照表全部 13 个源技能必须在映射表中
        let conn = test_conn();
        unify_domain_mapping(&conn).unwrap();
        let mut total = 0usize;
        let mut domains = std::collections::HashSet::new();
        for (_, d, _) in DOMAIN_SKILL_MAPPING {
            total += 1;
            domains.insert(d.to_string());
        }
        assert_eq!(total, DOMAIN_SKILL_MAPPING.len(), "映射表条数自洽");
        // 各域 namespace 全部可读（域数随映射生长，数据驱动）
        let mut ns_count = 0;
        for ns in kv_list_namespaces(&conn).unwrap() {
            if ns.starts_with("domain_nt_") {
                ns_count += 1;
            }
        }
        assert_eq!(ns_count, domains.len(), "各域各有 1 个 domain_nt_* namespace");
    }

    fn dedup_record(hash: &str) -> SkillRecord {
        SkillRecord {
            id: "id-dedup".into(),
            name: "dedup-skill".into(),
            description: Some("desc".into()),
            source_path: Some("/path".into()),
            tags: None,
            is_builtin: false,
            last_indexed_at: Some(now()),
            created_at: now(),
            updated_at: now(),
            content_hash: Some(hash.into()),
        }
    }

    #[test]
    fn test_skill_upsert_dedup_same_hash_skips() {
        let conn = test_conn();
        assert!(
            skill_upsert(&conn, "dedup-skill", &dedup_record("h1")).unwrap(),
            "首次插入应写入"
        );
        assert!(
            !skill_upsert(&conn, "dedup-skill", &dedup_record("h1")).unwrap(),
            "内容 hash 未变化时应去重跳过 (避免每命令全量写)"
        );
        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skills_index WHERE name='dedup-skill'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(rows, 1, "去重不得产生重复行");
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].content_hash.as_deref(), Some("h1"));
    }

    #[test]
    fn test_skill_upsert_dedup_changed_hash_rewrites() {
        let conn = test_conn();
        assert!(skill_upsert(&conn, "dedup-skill", &dedup_record("h1")).unwrap());
        assert!(
            skill_upsert(&conn, "dedup-skill", &dedup_record("h2")).unwrap(),
            "内容 hash 变化时应重新写入"
        );
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "同 name 更新而非新增");
        assert_eq!(recs[0].content_hash.as_deref(), Some("h2"));
        // 新 hash 后再次同 hash → 去重
        assert!(!skill_upsert(&conn, "dedup-skill", &dedup_record("h2")).unwrap());
    }

    #[test]
    fn test_skill_content_hash_deterministic() {
        let h1 = skill_content_hash("---\nname: test\n---\nbody");
        let h2 = skill_content_hash("---\nname: test\n---\nbody");
        let h3 = skill_content_hash("---\nname: other\n---\nbody");
        assert_eq!(h1, h2, "同内容 hash 必须稳定");
        assert_ne!(h1, h3, "不同内容 hash 必须不同");
        assert_eq!(h1.len(), 64, "sha256 hex 长度为 64");
    }

    #[test]
    fn test_yaml_frontmatter_split() {
        let content = "---\ntitle: Test\ntags: [a, b]\n---\n\nBody text";
        let (yaml, body) = split_yaml_frontmatter(content);
        assert!(yaml.is_some());
        assert!(yaml.as_ref().unwrap().contains("title: Test"));
        assert!(body.contains("Body text"));
    }

    #[test]
    fn test_yaml_no_frontmatter() {
        let content = "Just body text";
        let (yaml, body) = split_yaml_frontmatter(content);
        assert!(yaml.is_none());
        assert_eq!(body, "Just body text");
    }

    #[test]
    fn test_extract_description() {
        let content = "---\ntitle: X\n---\n\nThis is the description.\nMore text.";
        let desc = extract_description(content);
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_extract_tags() {
        let tags = extract_tags_from_content("---\ntags: [rust, testing, ai]\n---\ncontent here");
        assert!(tags.is_some());
        let t = tags.unwrap();
        assert!(t.contains("rust"));
    }
}
