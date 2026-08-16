//! Filesystem-level Rust source editor with backup/rollback.
//!
//! Provides atomic line-level edits to .rs files with
//! automatic backup, rollback on failure, and diff tracking.
//!
//! # Safety
//! All file I/O uses `std::fs` — no unsafe.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A single source-level edit operation.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceEdit {
    /// Replace the line at `line` (1-indexed) with `new_content`.
    ReplaceLine { line: usize, new_content: String },
    /// Insert `content` after line `after_line`.
    InsertAfter { after_line: usize, content: String },
    /// Delete line at `line` (1-indexed).
    DeleteLine { line: usize },
    /// Replace a range of lines [start, end) with `content`.
    ReplaceRange {
        start: usize,
        end: usize,
        content: Vec<String>,
    },
    /// Add an import statement if not already present.
    AddImport { path: String, name: String },
}

#[derive(Debug, Clone)]
pub struct SourceEditResult {
    pub file: PathBuf,
    pub applied: Vec<SourceEdit>,
    pub backup_path: Option<PathBuf>,
    pub new_checksum: u64,
}

#[derive(Debug, Clone)]
pub struct SourceEditor {
    backup_dir: PathBuf,
    /// Active backups: file path → backup path
    active_backups: HashMap<PathBuf, PathBuf>,
    /// Files modified in current session
    modified_files: Vec<PathBuf>,
}

impl SourceEditor {
    pub fn new(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir,
            active_backups: HashMap::new(),
            modified_files: Vec::new(),
        }
    }

    /// Apply a single edit to a file. Returns the backup path.
    pub fn apply_edit(
        &mut self,
        file: &Path,
        edit: &SourceEdit,
    ) -> Result<SourceEditResult, String> {
        // Create backup on first edit to this file
        if !self.active_backups.contains_key(file) {
            let backup = self.create_backup(file)?;
            self.active_backups.insert(file.to_path_buf(), backup);
        }

        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {}", file.display(), e))?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        match edit {
            SourceEdit::ReplaceLine { line, new_content } => {
                if *line == 0 || *line > lines.len() {
                    return Err(format!(
                        "line {line} out of range (file has {} lines)",
                        lines.len()
                    ));
                }
                lines[*line - 1] = new_content.clone();
            }
            SourceEdit::InsertAfter {
                after_line,
                content,
            } => {
                if *after_line > lines.len() {
                    return Err(format!("after_line {after_line} out of range"));
                }
                lines.insert(*after_line, content.clone());
            }
            SourceEdit::DeleteLine { line } => {
                if *line == 0 || *line > lines.len() {
                    return Err(format!("line {line} out of range"));
                }
                lines.remove(*line - 1);
            }
            SourceEdit::ReplaceRange {
                start,
                end,
                content,
            } => {
                if *start == 0 || *start > lines.len() || *end > lines.len() || *start >= *end {
                    return Err(format!(
                        "range [{}, {}) invalid for {} lines",
                        start,
                        end,
                        lines.len()
                    ));
                }
                lines.splice(*start - 1..*end - 1, content.clone());
            }
            SourceEdit::AddImport {
                path: import_path,
                name,
            } => {
                let import_stmt = if name.is_empty() {
                    format!("use {};", import_path)
                } else {
                    format!("use {}::{};", import_path, name)
                };
                // Check if import already exists
                let already = lines.iter().any(|l| l.trim() == import_stmt);
                if !already {
                    // Insert after the last existing use statement, or at line 1
                    let insert_pos = lines
                        .iter()
                        .rposition(|l| l.trim().starts_with("use "))
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    lines.insert(insert_pos, import_stmt);
                }
            }
        }

        let new_content = lines.join("\n") + "\n";
        let checksum = compute_checksum(&new_content);
        std::fs::write(file, &new_content)
            .map_err(|e| format!("cannot write {}: {}", file.display(), e))?;
        self.modified_files.push(file.to_path_buf());

        Ok(SourceEditResult {
            file: file.to_path_buf(),
            applied: vec![edit.clone()],
            backup_path: self.active_backups.get(file).cloned(),
            new_checksum: checksum,
        })
    }

    /// Rollback a single file to its backed-up state.
    pub fn rollback(&mut self, file: &Path) -> Result<(), String> {
        let backup = self
            .active_backups
            .get(file)
            .ok_or_else(|| format!("no backup found for {}", file.display()))?;
        std::fs::copy(backup, file)
            .map_err(|e| format!("rollback failed for {}: {}", file.display(), e))?;
        self.modified_files.retain(|f| f != file);
        self.active_backups.remove(file);
        Ok(())
    }

    /// Rollback ALL modified files.
    pub fn rollback_all(&mut self) -> Vec<(PathBuf, Result<(), String>)> {
        let files: Vec<PathBuf> = self.modified_files.clone();
        files
            .iter()
            .map(|f| {
                let result = self.rollback(f);
                (f.clone(), result)
            })
            .collect()
    }

    fn create_backup(&self, file: &Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| format!("cannot create backup dir: {}", e))?;
        let backup_name = format!("{}_{}", file_name_safe(file), timestamp_ns());
        let backup_path = self.backup_dir.join(&backup_name);
        std::fs::copy(file, &backup_path)
            .map_err(|e| format!("cannot backup {}: {}", file.display(), e))?;
        Ok(backup_path)
    }

    /// List files that were modified in the current session.
    pub fn modified_files(&self) -> &[PathBuf] {
        &self.modified_files
    }

    /// Number of active backups.
    pub fn backup_count(&self) -> usize {
        self.active_backups.len()
    }
}

fn file_name_safe(path: &Path) -> String {
    path.to_str()
        .unwrap_or("unknown")
        .replace(['/', '\\', '.'], "_")
}

fn timestamp_ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn compute_checksum(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("source_edit_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_replace_line() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "line1\nline2\nline3\n");
        let backup_dir = dir.join("backups");
        let mut editor = SourceEditor::new(backup_dir);

        let result = editor
            .apply_edit(
                &file,
                &SourceEdit::ReplaceLine {
                    line: 2,
                    new_content: "modified2".into(),
                },
            )
            .unwrap();

        assert_eq!(result.file, file);
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "line1\nmodified2\nline3\n");
    }

    #[test]
    fn test_insert_after() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "line1\nline2\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &file,
                &SourceEdit::InsertAfter {
                    after_line: 1,
                    content: "inserted".into(),
                },
            )
            .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "line1\ninserted\nline2\n");
    }

    #[test]
    fn test_delete_line() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "line1\nline2\nline3\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(&file, &SourceEdit::DeleteLine { line: 2 })
            .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "line1\nline3\n");
    }

    #[test]
    fn test_replace_range() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "a\nb\nc\nd\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &file,
                &SourceEdit::ReplaceRange {
                    start: 2,
                    end: 4,
                    content: vec!["x".into(), "y".into()],
                },
            )
            .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "a\nx\ny\nd\n");
    }

    #[test]
    fn test_add_import() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "lib.rs", "pub fn foo() {}\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &file,
                &SourceEdit::AddImport {
                    path: "std::collections".into(),
                    name: "HashMap".into(),
                },
            )
            .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.contains("use std::collections::HashMap;"));
    }

    #[test]
    fn test_add_import_no_duplicate() {
        let dir = temp_dir();
        let file = write_test_file(
            &dir,
            "lib.rs",
            "use std::collections::HashMap;\npub fn foo() {}\n",
        );
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &file,
                &SourceEdit::AddImport {
                    path: "std::collections".into(),
                    name: "HashMap".into(),
                },
            )
            .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content.lines().filter(|l| l.contains("HashMap")).count(), 1);
    }

    #[test]
    fn test_rollback() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "original\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &file,
                &SourceEdit::ReplaceLine {
                    line: 1,
                    new_content: "modified".into(),
                },
            )
            .unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "modified\n");

        editor.rollback(&file).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "original\n");
    }

    #[test]
    fn test_rollback_all() {
        let dir = temp_dir();
        let f1 = write_test_file(&dir, "a.rs", "aaa\n");
        let f2 = write_test_file(&dir, "b.rs", "bbb\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        editor
            .apply_edit(
                &f1,
                &SourceEdit::ReplaceLine {
                    line: 1,
                    new_content: "AAA".into(),
                },
            )
            .unwrap();
        editor
            .apply_edit(
                &f2,
                &SourceEdit::ReplaceLine {
                    line: 1,
                    new_content: "BBB".into(),
                },
            )
            .unwrap();

        let results = editor.rollback_all();
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_ok());
        assert!(results[1].1.is_ok());
        assert_eq!(std::fs::read_to_string(&f1).unwrap(), "aaa\n");
        assert_eq!(std::fs::read_to_string(&f2).unwrap(), "bbb\n");
    }

    #[test]
    fn test_line_out_of_range() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "only one line\n");
        let mut editor = SourceEditor::new(dir.join("backups"));

        let result = editor.apply_edit(
            &file,
            &SourceEdit::ReplaceLine {
                line: 5,
                new_content: "x".into(),
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_created() {
        let dir = temp_dir();
        let file = write_test_file(&dir, "test.rs", "content\n");
        let backup_dir = dir.join("backups");
        let mut editor = SourceEditor::new(backup_dir.clone());

        let result = editor
            .apply_edit(
                &file,
                &SourceEdit::ReplaceLine {
                    line: 1,
                    new_content: "modified".into(),
                },
            )
            .unwrap();

        assert!(result.backup_path.is_some());
        assert!(result.backup_path.unwrap().exists());
    }
}
