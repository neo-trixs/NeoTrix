pub mod crate_query;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::neotrix::lsp::{Diagnostic, LspClient, LspError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedCrate {
    pub name: String,
    pub version: String,
}

pub struct CodeQueryEngine;

impl Default for CodeQueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeQueryEngine {
    pub fn new() -> Self { Self }
    pub fn analyze_crate(_name: &str) -> AnalyzedCrate {
        AnalyzedCrate { name: _name.to_string(), version: "unknown".into() }
    }

    /// Analyze a crate using LSP. Opens the file and pulls diagnostics.
    pub async fn analyze_with_lsp(
        client: &mut LspClient,
        path: &Path,
        text: &str,
    ) -> Result<AnalyzedCrate, LspError> {
        client.open_file(path, text).await?;
        let _diags = client.diagnostics(path).await?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(AnalyzedCrate {
            name,
            version: "0.1.0".into(),
        })
    }

    /// Pull LSP diagnostics for a file in the workspace.
    pub async fn get_lsp_diagnostics(
        client: &mut LspClient,
        path: &Path,
    ) -> Result<Vec<Diagnostic>, LspError> {
        client.diagnostics(path).await
    }
}
