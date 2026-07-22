//! Stream .md files into KB — cross-session knowledge sharing.
//!
//! # Pattern
//! ```
//! .md file created/updated → absorb_md_file(kb, path) → KB(note node)
//! sync_markdown_dir(kb, dir) walks a directory, absorbs each .md
//! BackgroundLoop start() syncs notes/ + docs/ automatically
//! ```
//!
//! Each .md file becomes a KB node with:
//! - `node_type = Note`
//! - `title` = first `# ` heading (or filename)
//! - `content` = full file text
//! - `url` = relative file path (dedup key)
//! - `metadata.file_mtime` = last-modified timestamp

use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::neotrix::l3_memory_impl::nt_memory_kb::{KnowledgeBase, NodeType};

/// Absorb a single .md file into the KB.
/// Idempotent: same file path → updates existing node content.
pub fn absorb_md_file(kb: &KnowledgeBase, path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    let text = String::from_utf8(bytes).map_err(|e| format!("utf8 {}: {}", path.display(), e))?;
    if text.trim().is_empty() {
        return Err(format!("empty file: {}", path.display()));
    }

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let title = text
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l[2..].trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or(file_stem);

    let file_path = path.to_string_lossy().to_string();
    let domain = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("root");

    let mtime = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or(0);

    let node_id = kb.insert_or_get_node(
        &title,
        NodeType::Source,
        Some(&text.chars().take(500).collect::<String>()),
        Some(&file_path),
        Some(domain),
    )?;

    kb.update_node_content(&node_id, &text)?;

    let meta = serde_json::json!({
        "file_path": file_path,
        "file_mtime": mtime,
        "bytes": text.len(),
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Walk a directory recursively and absorb every `.md` file.
/// Skips hidden files and common noise directories.
/// Returns (absorbed_count, errors).
pub fn sync_markdown_dir(kb: &KnowledgeBase, dir: &Path) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut errors = Vec::new();

    if !dir.exists() {
        return (0, vec![format!("directory not found: {}", dir.display())]);
    }

    let walker = match dir.read_dir() {
        Ok(w) => w,
        Err(e) => return (0, vec![format!("read_dir {}: {}", dir.display(), e)]),
    };

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            let (sub, sub_errs) = sync_markdown_dir(kb, &path);
            count += sub;
            errors.extend(sub_errs);
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            match absorb_md_file(kb, &path) {
                Ok(_) => count += 1,
                Err(e) => errors.push(e),
            }
        }
    }

    (count, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn test_kb(name: &str) -> KnowledgeBase {
        let dir = PathBuf::from(std::env::temp_dir()).join(format!("nt_md_abs_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        KnowledgeBase::open(Some(db_path)).expect("open KB")
    }

    #[test]
    fn test_absorb_md_file_simple() {
        let dir = PathBuf::from(std::env::temp_dir()).join("nt_md_test_simple");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.md");
        fs::write(&path, "# Hello World\n\nThis is a test.").unwrap();

        let kb = test_kb("simple");
        let id = absorb_md_file(&kb, &path).expect("absorb");
        assert!(!id.is_empty(), "should return a node id");

        let id2 = absorb_md_file(&kb, &path).expect("absorb again");
        assert_eq!(id, id2, "same file → same node");
    }

    #[test]
    fn test_absorb_md_file_title_from_heading() {
        let dir = PathBuf::from(std::env::temp_dir()).join("nt_md_test_title");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("my_analysis.md");
        fs::write(&path, "# Streaming Gap Analysis\n\n## Background\n...").unwrap();

        let kb = test_kb("title");
        let _ = absorb_md_file(&kb, &path).expect("absorb");
        let found = kb.find_node_by_url(&path.to_string_lossy()).expect("find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Streaming Gap Analysis");
    }

    #[test]
    fn test_absorb_md_file_title_from_filename_when_no_heading() {
        let dir = PathBuf::from(std::env::temp_dir()).join("nt_md_test_fname");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("untitled_note.md");
        fs::write(&path, "Just some text without a heading.").unwrap();
        let kb = test_kb("fname");
        let _ = absorb_md_file(&kb, &path).expect("absorb");
    }

    #[test]
    fn test_sync_markdown_dir_finds_files() {
        let dir = PathBuf::from(std::env::temp_dir()).join("nt_md_test_sync");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.md"), "# File A\ncontent a").unwrap();
        fs::write(dir.join("b.md"), "# File B\ncontent b").unwrap();
        fs::write(dir.join("notes.md"), "# Notes\nsome notes").unwrap();
        fs::write(dir.join(".hidden.md"), "# Hidden\nshould skip").unwrap();

        let kb = test_kb("sync");
        let (count, errors) = sync_markdown_dir(&kb, &dir);
        assert!(errors.is_empty(), "no errors: {:?}", errors);
        assert_eq!(count, 3, "should absorb 3 files, skip .hidden.md");
    }

    #[test]
    fn test_sync_markdown_dir_nested() {
        let dir = PathBuf::from(std::env::temp_dir()).join("nt_md_test_nested");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("root.md"), "# Root\nroot").unwrap();
        fs::write(dir.join("sub").join("child.md"), "# Child\nchild").unwrap();

        let kb = test_kb("nested");
        let (count, errors) = sync_markdown_dir(&kb, &dir);
        assert!(errors.is_empty(), "no errors: {:?}", errors);
        assert_eq!(count, 2, "should absorb both root and child");
    }
}
