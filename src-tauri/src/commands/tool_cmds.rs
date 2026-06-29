//! Tauri commands for consciousness tool primitives

use neotrix::nt_mind::brain_event_bus::{BrainEvent, GlobalBus, ToolOrigin};
use neotrix::neotrix::nt_tools::dispatch_tool;
use neotrix::neotrix::nt_world_search::WebSearchEngine;

#[derive(serde::Serialize)]
pub struct ToolResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(serde::Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Execute a consciousness tool (read/write/edit/bash/glob/grep/webfetch/websearch).
/// Emits a BrainEvent::Tool for the frontend to display.
#[tauri::command]
pub fn tool_execute(tool: String, args: serde_json::Value) -> ToolResponse {
    let result = dispatch_tool(&tool, &args);
    let summary = if result.success {
        let preview: String = result.output.chars().take(120).collect();
        if result.output.len() > 120 {
            format!("{}…", preview)
        } else {
            preview
        }
    } else {
        Some(result.error.clone()).filter(|s| !s.is_empty()).unwrap_or_default()
    };
    let _event = BrainEvent::Tool {
        tool: tool.clone(),
        success: result.success,
        duration_ms: result.duration_ms,
        origin: ToolOrigin::User,
        summary,
    };
    ToolResponse {
        success: result.success,
        output: result.output,
        error: Some(result.error),
        duration_ms: result.duration_ms,
    }
}

/// Search the web and return structured results.
#[tauri::command]
pub fn tool_search(query: String, count: Option<usize>) -> Result<Vec<SearchResultItem>, String> {
    let engine = WebSearchEngine::default();
    let results = engine.search(&query, count.unwrap_or(8))?;
    let items: Vec<SearchResultItem> = results.into_iter().map(|r| SearchResultItem {
        title: r.title,
        url: r.url,
        snippet: r.snippet,
    }).collect();
    let summary = format!("search: {} results for \"{}\"", items.len(), &query);
    GlobalBus::emit(BrainEvent::Tool {
        tool: "websearch".into(),
        success: true,
        duration_ms: 0,
        origin: ToolOrigin::User,
        summary,
    });
    Ok(items)
}
