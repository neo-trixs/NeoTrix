//! nt_resource_cortex — Cortex-Brain 外置卷编目 (upsert/register + 卷路径)，行为零变更纯搬移。

use rusqlite::Connection;
use serde_json;

use super::super::nt_memory_cortex_sync::enrich_cortex_metadata;
use super::super::shared_utils::now;

/// Mount point of the external Cortex-Brain volume (cold offline archive).
pub(crate) const CORTEX_ROOT: &str = "/Volumes/NeoTrixBrain";

/// Upsert a `cortex_brain` registry node. `kind` (causal_graph / corpus_archive / archive_dir)
/// is stored inside `metadata` because the `nodes` table has no `kind` column.
pub(crate) fn upsert_cortex_node(
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
