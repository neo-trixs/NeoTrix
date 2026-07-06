use super::pipeline::{BrainStage, StageDecision};
use super::SelfIteratingBrain;

use crate::core::nt_core_error_parse::{self, CompilerDiagnostic, DiagnosticSeverity};
use crate::core::nt_core_source_edit::{SourceEdit, SourceEditor};

use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_FIX_ITERATIONS: usize = 5;
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub struct CompileFixStage {
    editor: Mutex<SourceEditor>,
}

impl Default for CompileFixStage {
    fn default() -> Self {
        let backup_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("source-backups");
        Self {
            editor: Mutex::new(SourceEditor::new(backup_dir)),
        }
    }
}

impl CompileFixStage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BrainStage for CompileFixStage {
    fn name(&self) -> &str {
        "compile_fix"
    }

    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, crate::neotrix::nt_core_error::NeoTrixError> {
        for iteration in 0..MAX_FIX_ITERATIONS {
            let output = run_cargo_check()?;
            let diagnostics = nt_core_error_parse::parse_compiler_output(&output);

            let errors: Vec<&CompilerDiagnostic> = diagnostics.iter()
                .filter(|d| d.severity == DiagnosticSeverity::Error)
                .collect();

            if errors.is_empty() {
                if iteration > 0 {
                    log::info!("compile_fix: all errors resolved after {iteration} iterations");
                }
                return Ok(StageDecision::Continue);
            }

            let fixable: Vec<&CompilerDiagnostic> = errors.iter()
                .filter(|d| nt_core_error_parse::is_fixable(d))
                .copied()
                .collect();

            if fixable.is_empty() {
                log::warn!("compile_fix: {} errors remain, none fixable automatically", errors.len());
                self.rollback_all();
                return Ok(StageDecision::Skip("non-fixable errors remain after compile fix".into()));
            }

            let mut editor = self.editor.lock().map_err(|e| {
                crate::neotrix::nt_core_error::NeoTrixError::Io(format!("editor lock: {e}"))
            })?;

            for diag in &fixable {
                let file_path = resolve_path(&diag.file);
                if let Some(edit) = generate_fix(diag, &output) {
                    if let Err(e) = editor.apply_edit(&file_path, &edit) {
                        log::warn!("compile_fix: failed to apply fix for {}:{}: {e}", diag.file, diag.line);
                    }
                }
            }
        }

        log::warn!("compile_fix: exceeded {MAX_FIX_ITERATIONS} iterations without resolving all errors");
        self.rollback_all();
        Ok(StageDecision::Skip("max fix iterations exceeded, rolled back".into()))
    }
}

impl CompileFixStage {
    fn rollback_all(&self) {
        if let Ok(mut editor) = self.editor.lock() {
            let results = editor.rollback_all();
            for (file, result) in &results {
                if let Err(e) = result {
                    log::error!("compile_fix: rollback failed for {}: {e}", file.display());
                } else {
                    log::info!("compile_fix: rolled back {}", file.display());
                }
            }
        }
    }
}

fn run_cargo_check() -> Result<String, crate::neotrix::nt_core_error::NeoTrixError> {
    let output = std::process::Command::new("cargo")
        .args(["check", "--lib"])
        .current_dir(WORKSPACE_DIR)
        .output()
        .map_err(|e| crate::neotrix::nt_core_error::NeoTrixError::Io(format!("cargo check failed: {e}")))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(format!("{stdout}\n{stderr}"))
}

fn resolve_path(relative: &str) -> PathBuf {
    Path::new(WORKSPACE_DIR).join(relative)
}

fn generate_fix(diag: &CompilerDiagnostic, _output: &str) -> Option<SourceEdit> {
    match diag.code.as_deref() {
        Some("E0425" | "E0433") => {
            if let Some(name) = extract_missing_name(&diag.message) {
                Some(SourceEdit::AddImport {
                    path: infer_crate(&name),
                    name,
                })
            } else {
                None
            }
        }
        Some("E0428") => Some(SourceEdit::DeleteLine { line: diag.line }),
        Some("E0063") => Some(SourceEdit::ReplaceLine {
            line: diag.line,
            new_content: "// TODO: missing fields — auto-fix needed".into(),
        }),
        Some("dead_code") | Some("unused_imports") | Some("unused_variable") | Some("unused_mut") => {
            Some(SourceEdit::ReplaceLine {
                line: diag.line,
                new_content: format!("#[allow({})]", diag.code.as_deref().unwrap_or("dead_code")),
            })
        }
        _ => None,
    }
}

fn extract_missing_name(msg: &str) -> Option<String> {
    for pattern in &["value `", "type `", "function `", "constant `", "module `"] {
        if let Some(start) = msg.find(pattern) {
            let rest = &msg[start + pattern.len()..];
            if let Some(end) = rest.find('`') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

fn infer_crate(name: &str) -> String {
    match name {
        "HashMap" | "HashSet" => "std::collections".into(),
        "Path" | "PathBuf" => "std::path".into(),
        "String" | "Vec" | "Box" | "Option" | "Result" => "std".into(),
        "Arc" | "Mutex" | "RwLock" => "std::sync".into(),
        "Instant" | "Duration" | "SystemTime" => "std::time".into(),
        "Formatter" => "std::fmt".into(),
        _ => format!("crate::{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_missing_name() {
        assert_eq!(
            extract_missing_name("cannot find value `x` in this scope"),
            Some("x".into())
        );
        assert_eq!(
            extract_missing_name("use of undeclared type `HashMap`"),
            Some("HashMap".into())
        );
        assert_eq!(
            extract_missing_name("no errors"),
            None
        );
    }

    #[test]
    fn test_infer_crate_common() {
        assert_eq!(infer_crate("HashMap"), "std::collections");
        assert_eq!(infer_crate("PathBuf"), "std::path");
        assert_eq!(infer_crate("Vec"), "std");
        assert_eq!(infer_crate("MyStruct"), "crate::MyStruct");
    }

    #[test]
    fn test_generate_fix_import() {
        let diag = CompilerDiagnostic {
            file: "src/lib.rs".into(), line: 10, column: 5,
            severity: DiagnosticSeverity::Error,
            code: Some("E0433".into()),
            message: "use of undeclared type `HashMap`".into(),
            span_text: None,
        };
        let fix = generate_fix(&diag, "");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap(), SourceEdit::AddImport {
            path: "std::collections".into(),
            name: "HashMap".into(),
        });
    }

    #[test]
    fn test_generate_fix_delete_duplicate() {
        let diag = CompilerDiagnostic {
            file: "src/main.rs".into(), line: 42, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0428".into()),
            message: "name defined multiple times".into(),
            span_text: None,
        };
        let fix = generate_fix(&diag, "");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap(), SourceEdit::DeleteLine { line: 42 });
    }
}
