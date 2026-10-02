//! fs — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::Duration;

use tokio::fs::{self};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;
use super::super::router;

// ═══════════════════════════════════════════════════════════════════════════
// Filename detection — content-disposition + URL path fallback
// ═══════════════════════════════════════════════════════════════════════════

/// Detect filename from HEAD content-disposition or URL path.
pub async fn detect_filename(client: &reqwest::Client, url: &reqwest::Url, dest: &Path) -> PathBuf {
    if dest.file_stem().is_some_and(|s| !s.to_string_lossy().is_empty()) {
        return dest.to_path_buf();
    }
    if let Ok(resp) = client.head(url.clone()).send().await {
        if let Some(cd) = resp.headers().get("content-disposition") {
            if let Ok(cd_str) = cd.to_str() {
                if let Some(name) = router::parse_content_disposition(cd_str) {
                    return dest.with_file_name(name);
                }
            }
        }
    }
    if let Some(name) = url.path().rsplit('/').next() {
        if !name.is_empty() {
            return dest.with_file_name(name);
        }
    }
    dest.to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════════════
// Disk space pre-check
// ═══════════════════════════════════════════════════════════════════════════

/// Check that the parent directory has enough free space for `needed` bytes.
/// Returns Ok(()) if sufficient, Err with message if not.
pub fn check_disk_space(path: &Path, needed: u64, min_free: u64) -> Result<(), String> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let meta = std::fs::metadata(parent)
        .map_err(|e| format!("cannot stat parent dir {}: {}", parent.display(), e))?;
    if !meta.is_dir() {
        return Err(format!("parent path is not a directory: {}", parent.display()));
    }
    // statvfs is not portable; use a heuristic: if the file already exists,
    // check its size vs needed. Otherwise, attempt a create+set_len probe.
    if path.exists() {
        if let Ok(m) = std::fs::metadata(path) {
            if m.len() >= needed {
                return Ok(());
            }
        }
    }
    // Try to probe free space by creating a temporary file
    let probe = parent.join(".nt_disk_probe");
    match std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&probe)
    {
        Ok(f) => {
            let probe_needed = needed.saturating_add(min_free);
            let _ = f.set_len(probe_needed);
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(e) => Err(format!("disk space probe failed: {}", e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Temp file merge strategy (.dl_* directories)
// ═══════════════════════════════════════════════════════════════════════════

/// Create a temp directory for chunk files: `.dl_{stem}/`
pub(crate) fn make_dl_tmp_dir(dest: &Path) -> PathBuf {
    let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
    dest.parent()
        .unwrap_or(Path::new("."))
        .join(format!(".dl_{}", stem))
}

/// Clean up stale `.dl_*` temp directories older than `max_age_secs`.
/// Returns the number of directories removed.
pub async fn cleanup_stale_temps(output_dir: &Path, max_age_secs: u64) -> usize {
    let mut removed = 0;
    let Ok(mut entries) = fs::read_dir(output_dir).await else {
        return 0;
    };
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(max_age_secs))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(".dl_") {
            continue;
        }
        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if modified < cutoff {
            let _ = fs::remove_dir_all(entry.path()).await;
            removed += 1;
        }
    }
    removed
}

/// Merge chunk files from `tmp_dir` into a single output file.
/// Chunks are expected to be named `c0000.tmp`, `c0001.tmp`, etc.
pub(crate) async fn merge_chunks(tmp_dir: &Path, dest: &Path, n_chunks: usize) -> Result<(), String> {
    let mut out = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(dest)
            .await
            .map_err(|e| format!("create output: {}", e))?,
    );
    let mut buf = vec![0u8; 8192];
    for i in 0..n_chunks {
        let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
        let mut f = fs::File::open(&chunk_file)
            .await
            .map_err(|e| format!("open chunk {}: {}", i, e))?;
        loop {
            let n = f
                .read(&mut buf)
                .await
                .map_err(|e| format!("read chunk {}: {}", i, e))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .await
                .map_err(|e| format!("write output: {}", e))?;
        }
    }
    out.flush()
        .await
        .map_err(|e| format!("flush output: {}", e))?;
    Ok(())
}

/// Write a .done marker file alongside the download.
pub(crate) async fn write_done_marker(
    dest: &Path,
    total_size: u64,
    url: &str,
) {
    let done_marker = dest.with_extension("done");
    // ⭐ 2026-10-02：原 `let _ =` 丢弃 ⇒ marker 写失败时 `is_done()` 返回 `None`
    // ⇒ **下次重下整个媒体文件**，而用户只看到「下载完成了」。
    // ⭐ 范式抄**同目录** `engine.rs:108-110`（同一 streaming 模块）。
    // ⛔ 不 return Err：内容已下完，不该因 marker 写失败把整次下载判为失败。
    if let Err(e) = fs::write(
        &done_marker,
        format!(
            "size={}\nurl={}\ntimestamp={}\n",
            total_size,
            url,
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ),
    )
        .await
    {
        eprintln!("[dl] .done marker 落盘失败 {}（下次将重下整个文件）: {e}", done_marker.display());
    }
}

/// Check if a .done marker indicates the file is already complete.
pub(crate) async fn is_done(dest: &Path) -> Option<u64> {
    let done_marker = dest.with_extension("done");
    if !dest.exists() || !done_marker.exists() {
        return None;
    }
    let meta = fs::metadata(dest).await.ok()?;
    let content = fs::read_to_string(&done_marker).await.ok()?;
    let done_size = content
        .lines()
        .find(|l| l.starts_with("size="))
        .and_then(|l| l.strip_prefix("size="))
        .and_then(|s| s.parse::<u64>().ok())?;
    if meta.len() >= done_size {
        Some(meta.len())
    } else {
        None
    }
}

pub(crate) fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            let host = u.host_str()?.to_string();
            if let Some(port) = u.port() {
                Some(format!("{}:{}", host, port))
            } else {
                Some(host)
            }
        })
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════════════════
// Cancel utilities
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn create_cancel_pair() -> (Arc<AtomicBool>, Arc<AtomicBool>) {
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();
    (flag, flag_clone)
}
