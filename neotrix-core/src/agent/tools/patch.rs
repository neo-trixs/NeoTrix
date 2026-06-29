use std::path::Path;

#[derive(serde::Deserialize)]
pub struct PatchEdit {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PatchReport {
    pub file_path: String,
    pub success: bool,
    pub error: Option<String>,
    pub lines_changed: usize,
}

fn is_binary(content: &[u8]) -> bool {
    content.iter().take(8192).any(|&b| b == 0)
}

pub fn apply_patch(edit: &PatchEdit, dry_run: bool) -> PatchReport {
    let path = Path::new(&edit.file_path);

    if !path.exists() {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some(format!("File not found: {}", edit.file_path)),
            lines_changed: 0,
        };
    }

    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(e) => {
            return PatchReport {
                file_path: edit.file_path.clone(),
                success: false,
                error: Some(format!("Cannot read metadata: {}", e)),
                lines_changed: 0,
            }
        }
    };

    if metadata.permissions().readonly() {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some("File is read-only".to_string()),
            lines_changed: 0,
        };
    }

    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(e) => {
            return PatchReport {
                file_path: edit.file_path.clone(),
                success: false,
                error: Some(format!("Cannot read file: {}", e)),
                lines_changed: 0,
            }
        }
    };

    if is_binary(&content) {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some("Binary file detected, refusing to patch".to_string()),
            lines_changed: 0,
        };
    }

    let text = match String::from_utf8(content) {
        Ok(t) => t,
        Err(e) => {
            return PatchReport {
                file_path: edit.file_path.clone(),
                success: false,
                error: Some(format!("File is not valid UTF-8: {}", e)),
                lines_changed: 0,
            }
        }
    };

    let count = text.matches(&edit.old_string).count();

    if count == 0 {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some("old_string not found in file".to_string()),
            lines_changed: 0,
        };
    }

    if count > 1 {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some(format!(
                "Found {} occurrences of old_string. Expected exactly 1.",
                count
            )),
            lines_changed: 0,
        };
    }

    let new_text = text.replace(&edit.old_string, &edit.new_string);

    let old_lines: Vec<&str> = edit.old_string.lines().collect();
    let new_lines: Vec<&str> = edit.new_string.lines().collect();
    let added = new_lines.len().max(old_lines.len()) - old_lines.len().min(new_lines.len());
    let changed_lines = old_lines.len().max(added);

    if dry_run {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: true,
            error: None,
            lines_changed: changed_lines,
        };
    }

    let bak_path = path.with_extension(format!(
        "{}.bak",
        path.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    if let Err(e) = std::fs::copy(path, &bak_path) {
        return PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some(format!("Failed to create backup: {}", e)),
            lines_changed: 0,
        };
    }

    let tmp = path.with_extension("tmp");
    match std::fs::write(&tmp, &new_text) {
        Ok(_) => match std::fs::rename(&tmp, path) {
            Ok(()) => PatchReport {
                file_path: edit.file_path.clone(),
                success: true,
                error: None,
                lines_changed: changed_lines,
            },
            Err(e) => PatchReport {
                file_path: edit.file_path.clone(),
                success: false,
                error: Some(format!("Failed to rename: {}", e)),
                lines_changed: 0,
            },
        },
        Err(e) => PatchReport {
            file_path: edit.file_path.clone(),
            success: false,
            error: Some(format!("Failed to write file: {}", e)),
            lines_changed: 0,
        },
    }
}

pub fn patch_handler(args: &serde_json::Value) -> Result<String, String> {
    let edits: Vec<PatchEdit> = serde_json::from_value(
        args.get("edits")
            .ok_or_else(|| "Missing required field: edits".to_string())?
            .clone(),
    )
    .map_err(|e| format!("Invalid edits format: {}", e))?;

    let dry_run = args
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let reports: Vec<PatchReport> = edits
        .iter()
        .map(|edit| apply_patch(edit, dry_run))
        .collect();

    serde_json::to_string_pretty(&reports).map_err(|e| format!("Serialization error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_file(content: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().expect("create tempdir for test setup");
        let path = dir.path().join("test.rs");
        std::fs::write(&path, content).expect("write test file");
        let path_str = path.to_string_lossy().to_string();
        (dir, path_str)
    }

    #[test]
    fn test_simple_replace() {
        let (_dir, path) = setup_test_file("fn hello() {\n    println!(\"old\");\n}\n");
        let edit = PatchEdit {
            file_path: path.clone(),
            old_string: "old".to_string(),
            new_string: "new".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(report.success);
        assert!(report.error.is_none());
        assert_eq!(report.lines_changed, 1);
        let content = std::fs::read_to_string(&path).expect("read test file");
        assert!(content.contains("new"));
        assert!(!content.contains("old"));
    }

    #[test]
    fn test_file_not_found() {
        let edit = PatchEdit {
            file_path: "/nonexistent/path/file.rs".to_string(),
            old_string: "foo".to_string(),
            new_string: "bar".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(!report.success);
        assert!(report
            .error
            .expect("error should be Some")
            .contains("not found"));
    }

    #[test]
    fn test_old_string_not_found() {
        let (_dir, path) = setup_test_file("fn hello() {}\n");
        let edit = PatchEdit {
            file_path: path,
            old_string: "nonexistent".to_string(),
            new_string: "replacement".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(!report.success);
        assert!(report
            .error
            .expect("error should be Some")
            .contains("not found"));
    }

    #[test]
    fn test_multiple_occurrences() {
        let (_dir, path) = setup_test_file("let x = 1;\nlet x = 2;\nlet x = 3;\n");
        let edit = PatchEdit {
            file_path: path,
            old_string: "let x =".to_string(),
            new_string: "let y =".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(!report.success);
        assert!(report
            .error
            .expect("error should be Some")
            .contains("3 occurrences"));
    }

    #[test]
    fn test_dry_run_no_modify() {
        let (_dir, path) = setup_test_file("original content");
        let edit = PatchEdit {
            file_path: path.clone(),
            old_string: "original".to_string(),
            new_string: "modified".to_string(),
        };
        let report = apply_patch(&edit, true);
        assert!(report.success);
        let content = std::fs::read_to_string(&path).expect("read test file");
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_binary_file_rejection() {
        let dir = tempfile::tempdir().expect("create tempdir for binary test");
        let path = dir.path().join("binary.bin");
        let bytes: Vec<u8> = vec![0u8, 0x48, 0x65, 0x6c]; // starts with null byte
        std::fs::write(&path, &bytes).expect("write binary test file");
        let edit = PatchEdit {
            file_path: path.to_string_lossy().to_string(),
            old_string: "foo".to_string(),
            new_string: "bar".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(!report.success);
        assert!(report
            .error
            .expect("error should be Some for binary")
            .contains("Binary"));
    }

    #[test]
    fn test_readonly_file_rejection() {
        let (_dir, path) = setup_test_file("readonly content");
        let edit = PatchEdit {
            file_path: path.clone(),
            old_string: "readonly".to_string(),
            new_string: "writable".to_string(),
        };
        let mut perm = std::fs::metadata(&path)
            .expect("read metadata")
            .permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(&path, perm).expect("set readonly permissions");
        let report = apply_patch(&edit, false);
        assert!(!report.success);
    }

    #[test]
    fn test_multiline_replace() {
        let (_dir, path) =
            setup_test_file("fn old_func() {\n    let x = 1;\n    println!(\"{}\", x);\n}\n");
        let edit = PatchEdit {
            file_path: path.clone(),
            old_string: "fn old_func() {\n    let x = 1;\n    println!(\"{}\", x);\n}".to_string(),
            new_string: "fn new_func() {\n    let y = 2;\n    println!(\"{}\", y);\n}".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(report.success);
        let content = std::fs::read_to_string(&path).expect("read multiline result");
        assert!(content.contains("new_func"));
        assert!(content.contains("y = 2"));
        assert!(!content.contains("old_func"));
    }

    #[test]
    fn test_backup_created() {
        let (_dir, path) = setup_test_file("before");
        let edit = PatchEdit {
            file_path: path.clone(),
            old_string: "before".to_string(),
            new_string: "after".to_string(),
        };
        let report = apply_patch(&edit, false);
        assert!(report.success);
        let p = Path::new(&path);
        let bak = p.with_extension("rs.bak");
        assert!(bak.exists());
        let bak_content = std::fs::read_to_string(bak).expect("read backup file");
        assert_eq!(bak_content, "before");
    }

    #[test]
    fn test_patch_handler_single_edit() {
        let (_dir, path) = setup_test_file("handler content");
        let args = serde_json::json!({
            "edits": [{
                "file_path": path,
                "old_string": "handler",
                "new_string": "patched"
            }],
            "dry_run": false
        });
        let result = patch_handler(&args).expect("patch handler should succeed");
        let reports: Vec<PatchReport> = serde_json::from_str(&result).expect("deserialize reports");
        assert_eq!(reports.len(), 1);
        assert!(reports[0].success);
    }

    #[test]
    fn test_patch_handler_missing_edits() {
        let args = serde_json::json!({});
        let result = patch_handler(&args);
        let err = result.expect_err("patch handler should fail with missing edits");
        assert!(err.contains("edits"));
    }
}
