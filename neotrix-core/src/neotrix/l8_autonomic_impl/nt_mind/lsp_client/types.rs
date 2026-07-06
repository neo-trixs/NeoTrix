use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: Option<DiagnosticSeverity>,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspHover {
    pub contents: Vec<String>,
    pub range: Option<LspRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Keyword = 14,
    Snippet = 15,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspLocation {
    pub uri: String,
    pub range: LspRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: LspRange,
    pub selection_range: LspRange,
    pub children: Vec<LspDocumentSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Function = 12,
    Variable = 13,
    Constant = 14,
    Interface = 15,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Vec<TextEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: LspRange,
    pub new_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    pub name: String,
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub root_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_position_serde_roundtrip() {
        let pos = LspPosition { line: 42, character: 7 };
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: LspPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.line, 42);
        assert_eq!(deserialized.character, 7);
    }

    #[test]
    fn test_lsp_range_ordering() {
        let range = LspRange {
            start: LspPosition { line: 1, character: 0 },
            end: LspPosition { line: 1, character: 10 },
        };
        assert!(range.start.line <= range.end.line);
    }

    #[test]
    fn test_diagnostic_severity_values() {
        assert_eq!(DiagnosticSeverity::Error as i32, 1);
        assert_eq!(DiagnosticSeverity::Warning as i32, 2);
        assert_eq!(DiagnosticSeverity::Information as i32, 3);
        assert_eq!(DiagnosticSeverity::Hint as i32, 4);
    }

    #[test]
    fn test_lsp_diagnostic_full() {
        let diag = LspDiagnostic {
            range: LspRange {
                start: LspPosition { line: 0, character: 0 },
                end: LspPosition { line: 0, character: 5 },
            },
            severity: Some(DiagnosticSeverity::Error),
            message: "unused variable".into(),
            source: Some("rust-analyzer".into()),
            code: Some("E0001".into()),
        };
        assert_eq!(diag.message, "unused variable");
        assert_eq!(diag.source.as_deref(), Some("rust-analyzer"));
    }

    #[test]
    fn test_completion_item_kind_values() {
        assert_eq!(CompletionItemKind::Text as i32, 1);
        assert_eq!(CompletionItemKind::Function as i32, 3);
        assert_eq!(CompletionItemKind::Keyword as i32, 14);
        assert_eq!(CompletionItemKind::Snippet as i32, 15);
    }

    #[test]
    fn test_lsp_hover_with_range() {
        let hover = LspHover {
            contents: vec!["```rust\nfn foo()\n```".into()],
            range: Some(LspRange {
                start: LspPosition { line: 5, character: 0 },
                end: LspPosition { line: 5, character: 3 },
            }),
        };
        assert_eq!(hover.contents.len(), 1);
        assert!(hover.range.is_some());
    }

    #[test]
    fn test_lsp_server_config_defaults() {
        let config = LspServerConfig {
            name: "test-ls".into(),
            language_id: "test".into(),
            command: "test-language-server".into(),
            args: vec!["--stdio".into()],
            root_patterns: vec!["test.config".into()],
        };
        assert_eq!(config.name, "test-ls");
        assert_eq!(config.args.len(), 1);
    }

    #[test]
    fn test_lsp_location_uri() {
        let loc = LspLocation {
            uri: "file:///test.rs".into(),
            range: LspRange {
                start: LspPosition { line: 0, character: 0 },
                end: LspPosition { line: 0, character: 0 },
            },
        };
        assert_eq!(loc.uri, "file:///test.rs");
    }

    #[test]
    fn test_workspace_edit_single_change() {
        let edit = WorkspaceEdit {
            changes: vec![TextEdit {
                range: LspRange {
                    start: LspPosition { line: 1, character: 0 },
                    end: LspPosition { line: 1, character: 5 },
                },
                new_text: "foo".into(),
            }],
        };
        assert_eq!(edit.changes.len(), 1);
        assert_eq!(edit.changes[0].new_text, "foo");
    }

    #[test]
    fn test_symbol_kind_values() {
        assert_eq!(SymbolKind::Function as i32, 12);
        assert_eq!(SymbolKind::Class as i32, 5);
        assert_eq!(SymbolKind::Module as i32, 2);
    }

    #[test]
    fn test_lsp_document_symbol_nested() {
        let child = LspDocumentSymbol {
            name: "nested_fn".into(),
            kind: SymbolKind::Function,
            range: LspRange { start: LspPosition { line: 0, character: 0 }, end: LspPosition { line: 0, character: 0 } },
            selection_range: LspRange { start: LspPosition { line: 0, character: 0 }, end: LspPosition { line: 0, character: 0 } },
            children: vec![],
        };
        let parent = LspDocumentSymbol {
            name: "module".into(),
            kind: SymbolKind::Module,
            range: LspRange { start: LspPosition { line: 0, character: 0 }, end: LspPosition { line: 10, character: 0 } },
            selection_range: LspRange { start: LspPosition { line: 0, character: 0 }, end: LspPosition { line: 2, character: 0 } },
            children: vec![child],
        };
        assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].name, "nested_fn");
    }
}
