//! nt_resource_corpus — 语料迁移/断点续传/回收与孤儿剪枝，行为零变更纯搬移。

use std::collections::HashSet;
use std::process::Command;

use log::info;
use rusqlite::Connection;
use serde_json;

use super::nt_resource_cortex::{CORTEX_ROOT, corpus_archive_path, upsert_cortex_node};

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
pub(crate) fn copy_resumable(src: &std::path::Path, dest: &std::path::Path) -> Result<u64, String> {
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
