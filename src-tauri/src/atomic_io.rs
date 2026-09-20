//! Atomic file I/O utilities.
//!
//! All persistent state writes should use these utilities to prevent
//! corruption from concurrent reads or interrupted writes.
//!
//! Pattern: write to `.tmp` → `fs::rename()` → guaranteed atomic on POSIX/NTFS.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Write content atomically to a file.
///
/// 1. Write to `{path}.tmp`
/// 2. Sync to disk
/// 3. Rename to `path` (atomic on POSIX)
///
/// If any step fails, the original file is untouched.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), AtomicWriteError> {
    let tmp_path = tmp_path(path);

    // Write to temp file
    fs::write(&tmp_path, content)
        .map_err(|e| AtomicWriteError::Write(tmp_path.display().to_string(), e))?;

    // Sync to disk before rename
    fs::File::open(&tmp_path)
        .and_then(|f| f.sync_all())
        .map_err(|e| AtomicWriteError::Sync(tmp_path.display().to_string(), e))?;

    // Atomic rename
    fs::rename(&tmp_path, path).map_err(|e| {
        AtomicWriteError::Rename(
            tmp_path.display().to_string(),
            path.display().to_string(),
            e,
        )
    })?;

    Ok(())
}

/// Write JSON content atomically with pretty formatting.
pub fn write_json_atomic<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), AtomicWriteError> {
    let content =
        serde_json::to_vec_pretty(value).map_err(|e| AtomicWriteError::Serialize(e.to_string()))?;
    write_atomic(path, &content)
}

/// Read a file, falling back to a backup (.bak) if the main file is corrupted.
pub fn read_with_fallback(path: &Path) -> Result<Vec<u8>, AtomicReadError> {
    match fs::read(path) {
        Ok(content) => Ok(content),
        Err(_) => {
            let bak = backup_path(path);
            fs::read(&bak).map_err(|e| {
                AtomicReadError::NotFound(
                    path.display().to_string(),
                    bak.display().to_string(),
                    e,
                )
            })
        }
    }
}

/// Read and deserialize JSON with fallback.
pub fn read_json_with_fallback<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<T, AtomicReadError> {
    let content = read_with_fallback(path)?;
    serde_json::from_slice(&content)
        .map_err(|e| AtomicReadError::Deserialize(path.display().to_string(), e))
}

/// Create a backup of a file before overwriting.
pub fn create_backup(path: &Path) -> Result<(), AtomicWriteError> {
    let bak = backup_path(path);
    if path.exists() {
        fs::copy(path, &bak)
            .map_err(|e| AtomicWriteError::Backup(path.display().to_string(), e))?;
    }
    Ok(())
}

/// Write with backup: creates .bak of existing file, then writes atomically.
pub fn write_atomic_with_backup<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), AtomicWriteError> {
    create_backup(path)?;
    write_json_atomic(path, value)
}

/// Ensure parent directories exist.
pub fn ensure_parent_dir(path: &Path) -> Result<(), AtomicWriteError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AtomicWriteError::Write(parent.display().to_string(), e))?;
    }
    Ok(())
}

// ─── Internal Helpers ────────────────────────────────────

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

fn backup_path(path: &Path) -> PathBuf {
    let mut bak = path.as_os_str().to_owned();
    bak.push(".bak");
    PathBuf::from(bak)
}

// ─── Error Types ─────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AtomicWriteError {
    #[error("Failed to write to {0}: {1}")]
    Write(String, std::io::Error),
    #[error("Failed to sync {0}: {1}")]
    Sync(String, std::io::Error),
    #[error("Failed to rename {0} → {1}: {2}")]
    Rename(String, String, std::io::Error),
    #[error("Failed to serialize: {0}")]
    Serialize(String),
    #[error("Failed to backup {0}: {1}")]
    Backup(String, std::io::Error),
}

#[derive(Debug, Error)]
pub enum AtomicReadError {
    #[error("Neither {0} nor backup {1} readable: {2}")]
    NotFound(String, String, std::io::Error),
    #[error("Failed to deserialize {0}: {1}")]
    Deserialize(String, serde_json::Error),
}

// ─── Tests ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_write_atomic_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        write_json_atomic(&path, &serde_json::json!({"hello": "world"})).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_write_atomic_no_tmp_residue() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let tmp = dir.path().join("test.json.tmp");

        write_atomic(path, b"content").unwrap();

        assert!(path.exists());
        assert!(!tmp.exists());
    }

    #[test]
    fn test_read_with_fallback_to_bak() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data.json");
        let bak = dir.path().join("data.json.bak");

        fs::write(&bak, b"backup content").unwrap();

        let content = read_with_fallback(&path).unwrap();
        assert_eq!(content, b"backup content");
    }

    #[test]
    fn test_create_backup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");

        fs::write(&path, b"original").unwrap();
        create_backup(&path).unwrap();

        let bak = dir.path().join("state.json.bak");
        assert!(bak.exists());
        assert_eq!(fs::read(&bak).unwrap(), b"original");
    }
}
