use super::types::{FileEntry, FileIndex};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Scan a directory recursively and build a FileIndex with checksums.
pub fn scan_directory(
    root: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<FileIndex, String> {
    if !root.is_dir() {
        return Err(format!("Not a directory: {}", root.display()));
    }

    let canon_root = root
        .canonicalize()
        .map_err(|e| format!("Canonicalize: {}", e))?;
    let mut files = Vec::new();
    let mut total_size = 0u64;

    visit_dirs(
        &canon_root,
        &canon_root,
        include_patterns,
        exclude_patterns,
        &mut files,
        &mut total_size,
    )?;

    Ok(FileIndex {
        root: canon_root.to_string_lossy().to_string(),
        file_count: files.len() as u32,
        total_size,
        files,
    })
}

fn visit_dirs(
    root: &Path,
    dir: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    files: &mut Vec<FileEntry>,
    total_size: &mut u64,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Read dir {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Entry: {}", e))?;
        let path = entry.path();

        // Skip hidden files/dirs
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        if path.is_dir() {
            visit_dirs(
                root,
                &path,
                include_patterns,
                exclude_patterns,
                files,
                total_size,
            )?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Exclude patterns
            if matches_pattern(&relative, exclude_patterns) {
                continue;
            }

            // Include patterns (empty = include all)
            if !include_patterns.is_empty() && !matches_pattern(&relative, include_patterns) {
                continue;
            }

            let metadata =
                fs::metadata(&path).map_err(|e| format!("Metadata {}: {}", relative, e))?;
            let size = metadata.len();

            // Skip empty or very large files (>500MB)
            if size == 0 || size > 500 * 1024 * 1024 {
                continue;
            }

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let checksum = file_checksum(&path)?;

            files.push(FileEntry::new(
                PathBuf::from(&relative),
                size,
                modified,
                checksum,
            ));
            *total_size += size;
        }
    }
    Ok(())
}

fn matches_pattern(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        if p.starts_with('*') {
            path.ends_with(&p[1..])
        } else if p.ends_with('*') {
            path.starts_with(&p[..p.len() - 1])
        } else {
            path.contains(p.as_str())
        }
    })
}

/// Compute SHA-256 checksum of file contents.
fn file_checksum(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Open {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| format!("Hash {}: {}", path.display(), e))?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_scan_empty_dir() {
        let dir = std::env::temp_dir().join("neotrix_sync_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let index = scan_directory(&dir, &[], &[]).unwrap();
        assert_eq!(index.file_count, 0);
        assert_eq!(index.total_size, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_with_files() {
        let dir = std::env::temp_dir().join("neotrix_sync_test_files");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir.join("sub")).unwrap();

        let mut f = fs::File::create(dir.join("a.txt")).unwrap();
        f.write_all(b"hello").unwrap();
        let mut f = fs::File::create(dir.join("sub").join("b.txt")).unwrap();
        f.write_all(b"world").unwrap();

        let index = scan_directory(&dir, &[], &[]).unwrap();
        assert_eq!(index.file_count, 2);
        assert_eq!(index.total_size, 10);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_exclude_pattern() {
        let dir = std::env::temp_dir().join("neotrix_sync_test_exclude");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut f = fs::File::create(dir.join("main.rs")).unwrap();
        f.write_all(b"fn main() {}").unwrap();
        let mut f = fs::File::create(dir.join("Cargo.lock")).unwrap();
        f.write_all(b"lock").unwrap();

        let index = scan_directory(&dir, &[], &["Cargo.lock".to_string()]).unwrap();
        assert_eq!(index.file_count, 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_checksum_consistency() {
        let dir = std::env::temp_dir().join("neotrix_sync_test_checksum");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut f = fs::File::create(dir.join("data.txt")).unwrap();
        f.write_all(b"consistent content").unwrap();

        let index = scan_directory(&dir, &[], &[]).unwrap();
        assert_eq!(index.file_count, 1);
        let cs1 = index.files[0].checksum.clone();

        // Re-scan should produce same checksum
        let index2 = scan_directory(&dir, &[], &[]).unwrap();
        assert_eq!(index2.files[0].checksum, cs1);
        let _ = fs::remove_dir_all(&dir);
    }
}
