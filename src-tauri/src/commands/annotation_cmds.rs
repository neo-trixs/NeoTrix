//! Browser page annotation commands for NeoTrix Desktop.
//!
//! Users can highlight parts of a web page and leave comments that guide the AI.
//! Annotations are stored with the page URL and recalled when the browser revisits.

use std::sync::{LazyLock, Mutex};
use serde::{Serialize, Deserialize};
use chrono::Utc;

// ── Data Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAnnotation {
    pub id: String,
    pub url: String,
    pub page_title: String,
    pub selector: String,
    pub highlighted_text: String,
    pub comment: String,
    pub annotation_type: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub resolved: bool,
    pub resolved_at: Option<String>,
    pub tags: Vec<String>,
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub annotations: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub annotation_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStats {
    pub total_annotations: u32,
    pub unresolved: u32,
    pub resolved_today: u32,
    pub collections: u32,
    pub urls_tracked: u32,
    pub top_tags: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationConfig {
    pub enabled: bool,
    pub auto_collect: bool,
    pub collect_on_navigate: bool,
    pub show_on_page_load: bool,
    pub notify_on_unresolved: bool,
    pub max_annotations_per_page: u32,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_collect: true,
            collect_on_navigate: true,
            show_on_page_load: true,
            notify_on_unresolved: true,
            max_annotations_per_page: 50,
        }
    }
}

// ── Internal State ────────────────────────────────────────────────────────

struct AnnotationState {
    annotations: Vec<PageAnnotation>,
    collections: Vec<AnnotationCollection>,
    config: AnnotationConfig,
}

fn short_uid() -> String {
    let uuid = uuid::Uuid::new_v4();
    let hex = uuid.to_string().replace('-', "");
    format!("ann-{}", &hex[..12])
}

fn collection_uid() -> String {
    let uuid = uuid::Uuid::new_v4();
    let hex = uuid.to_string().replace('-', "");
    format!("col-{}", &hex[..12])
}

fn now_str() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn today_str() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

fn make_annotation(
    id: &str, url: &str, page_title: &str, selector: &str,
    highlighted_text: &str, comment: &str, annotation_type: &str,
    author: &str, tags: Vec<&str>, resolved: bool,
) -> PageAnnotation {
    let now = now_str();
    PageAnnotation {
        id: id.to_string(),
        url: url.to_string(),
        page_title: page_title.to_string(),
        selector: selector.to_string(),
        highlighted_text: highlighted_text.to_string(),
        comment: comment.to_string(),
        annotation_type: annotation_type.to_string(),
        author: author.to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        resolved,
        resolved_at: if resolved { Some(now.clone()) } else { None },
        tags: tags.into_iter().map(String::from).collect(),
        screenshot_path: None,
    }
}

const MAX_ANNOTATIONS: usize = 1000;
const MAX_COLLECTIONS: usize = 100;

static ANNOTATION_STATE: LazyLock<Mutex<AnnotationState>> = LazyLock::new(|| {
    Mutex::new(AnnotationState {
        annotations: vec![
            make_annotation(
                "ann-seed-001",
                "https://github.com/neo/neotrix",
                "neotrix/neotrix — GitHub",
                "#readme > h1",
                "NeoTrix — AI-Native Developer Toolkit",
                "Feature request: would be great to add WASM plugin support for custom tools",
                "feature",
                "alice",
                vec!["feature", "wasm", "plugin"],
                false,
            ),
            make_annotation(
                "ann-seed-002",
                "https://github.com/neo/neotrix",
                "neotrix/neotrix — GitHub",
                ".issue-label",
                "bug: pipeline fails on empty queue",
                "Bug report: when the crawl queue is empty, the pipeline panics instead of gracefully handling",
                "issue",
                "bob",
                vec!["bug", "pipeline", "crawl"],
                true,
            ),
            make_annotation(
                "ann-seed-003",
                "https://github.com/neo/neotrix",
                "neotrix/neotrix — GitHub",
                "#installation > code",
                "cargo install neotrix",
                "Question: does this require nightly Rust or does it work on stable?",
                "question",
                "charlie",
                vec!["question", "installation", "rust"],
                false,
            ),
            make_annotation(
                "ann-seed-004",
                "https://docs.rs/tauri/latest/",
                "Tauri — Rust Docs",
                ".method.impl > .fn-name",
                "pub fn invoke<M>(&self, cmd: M) -> Result<T>",
                "API note: invoke now uses serde_json::Value instead of Serialize trait directly in V2",
                "highlight",
                "alice",
                vec!["api", "tauri", "invoke"],
                false,
            ),
            make_annotation(
                "ann-seed-005",
                "https://docs.rs/tauri/latest/",
                "Tauri — Rust Docs",
                "pre code.language-rust",
                "fn setup<F>(self, f: F) -> App",
                "Example code highlight: this is the canonical way to add initialisation logic in V2",
                "highlight",
                "bob",
                vec!["example", "setup", "initialization"],
                false,
            ),
            make_annotation(
                "ann-seed-006",
                "https://docs.rs/tauri/latest/",
                "Tauri — Rust Docs",
                ".warning > p",
                "Deprecated: use Builder::plugin instead",
                "Documentation fix: the deprecation notice should mention the replacement path explicitly",
                "task",
                "charlie",
                vec!["doc-fix", "deprecation", "migration"],
                false,
            ),
            make_annotation(
                "ann-seed-007",
                "https://crates.io/crates/neotrix",
                "neotrix — crates.io",
                ".page-description",
                "AI-Native Developer Toolkit with self-evolving reasoning",
                "Review comment: the crate description could highlight the VSA HyperCube more prominently",
                "comment",
                "alice",
                vec!["review", "description", "vsa"],
                false,
            ),
            make_annotation(
                "ann-seed-008",
                "https://crates.io/crates/neotrix",
                "neotrix — crates.io",
                ".downloads",
                "10k+ downloads",
                "Suggestion: add a feature comparison table in the README to differentiate from other AI toolkits",
                "task",
                "bob",
                vec!["suggestion", "readme", "comparison"],
                false,
            ),
        ],
        collections: vec![],
        config: AnnotationConfig::default(),
    })
});

// ── Tauri Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn annotation_create(
    url: String,
    page_title: String,
    selector: String,
    highlighted_text: String,
    comment: String,
    annotation_type: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    if state.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Annotation store full (max 1000 annotations)".to_string());
    }
    let id = short_uid();
    let now = now_str();
    let ann = PageAnnotation {
        id: id.clone(),
        url,
        page_title,
        selector,
        highlighted_text,
        comment,
        annotation_type: annotation_type.unwrap_or_else(|| "highlight".to_string()),
        author: "user".to_string(),
        created_at: now.clone(),
        updated_at: now,
        resolved: false,
        resolved_at: None,
        tags: tags.unwrap_or_default(),
        screenshot_path: None,
    };
    state.annotations.push(ann);
    Ok(id)
}

#[tauri::command]
pub fn annotation_list(
    url: Option<String>,
    resolved: Option<bool>,
    page: Option<u32>,
) -> Result<Vec<PageAnnotation>, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let mut results: Vec<PageAnnotation> = state.annotations.iter()
        .filter(|a| {
            let url_match = url.as_ref().map_or(true, |u| a.url == *u);
            let resolved_match = resolved.map_or(true, |r| a.resolved == r);
            url_match && resolved_match
        })
        .cloned()
        .collect();
    results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let page = page.unwrap_or(1).max(1) as usize;
    let per_page = 50;
    let start = (page - 1) * per_page;
    Ok(results.into_iter().skip(start).take(per_page).collect())
}

#[tauri::command]
pub fn annotation_get(id: String) -> Result<PageAnnotation, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    state.annotations.iter()
        .find(|a| a.id == id)
        .cloned()
        .ok_or_else(|| format!("Annotation {} not found", id))
}

#[tauri::command]
pub fn annotation_update(
    id: String,
    comment: Option<String>,
    annotation_type: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let ann = state.annotations.iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("Annotation {} not found", id))?;
    if let Some(c) = comment { ann.comment = c; }
    if let Some(t) = annotation_type { ann.annotation_type = t; }
    if let Some(t) = tags { ann.tags = t; }
    ann.updated_at = now_str();
    Ok(())
}

#[tauri::command]
pub fn annotation_delete(id: String) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let len_before = state.annotations.len();
    state.annotations.retain(|a| a.id != id);
    if state.annotations.len() == len_before {
        return Err(format!("Annotation {} not found", id));
    }
    Ok(())
}

#[tauri::command]
pub fn annotation_resolve(id: String) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let ann = state.annotations.iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("Annotation {} not found", id))?;
    if !ann.resolved {
        ann.resolved = true;
        ann.resolved_at = Some(now_str());
        ann.updated_at = now_str();
    }
    Ok(())
}

#[tauri::command]
pub fn annotation_unresolve(id: String) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let ann = state.annotations.iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("Annotation {} not found", id))?;
    if ann.resolved {
        ann.resolved = false;
        ann.resolved_at = None;
        ann.updated_at = now_str();
    }
    Ok(())
}

#[tauri::command]
pub fn annotation_collection_create(
    name: String,
    description: Option<String>,
    url: String,
    annotation_ids: Vec<String>,
) -> Result<String, String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    if state.collections.len() >= MAX_COLLECTIONS {
        return Err("Collection store full (max 100 collections)".to_string());
    }
    let id = collection_uid();
    let now = now_str();
    let col = AnnotationCollection {
        id: id.clone(),
        name,
        description: description.unwrap_or_default(),
        url,
        annotations: annotation_ids,
        created_at: now.clone(),
        updated_at: now,
        annotation_count: 0,
    };
    state.collections.push(col);
    Ok(id)
}

#[tauri::command]
pub fn annotation_collection_get(id: String) -> Result<AnnotationCollection, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    state.collections.iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| format!("Collection {} not found", id))
}

#[tauri::command]
pub fn annotation_collection_list() -> Result<Vec<AnnotationCollection>, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let mut list = state.collections.clone();
    list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(list)
}

#[tauri::command]
pub fn annotation_collection_delete(id: String) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let len_before = state.collections.len();
    state.collections.retain(|c| c.id != id);
    if state.collections.len() == len_before {
        return Err(format!("Collection {} not found", id));
    }
    Ok(())
}

#[tauri::command]
pub fn annotation_stats() -> Result<AnnotationStats, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let total = state.annotations.len() as u32;
    let unresolved = state.annotations.iter().filter(|a| !a.resolved).count() as u32;
    let today = today_str();
    let resolved_today = state.annotations.iter()
        .filter(|a| a.resolved_at.as_deref().map_or(false, |d| d.starts_with(&today)))
        .count() as u32;
    let mut unique_urls: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for a in &state.annotations {
        unique_urls.insert(a.url.as_str());
    }
    let mut tag_count: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for a in &state.annotations {
        for t in &a.tags {
            *tag_count.entry(t.clone()).or_insert(0) += 1;
        }
    }
    let mut top_tags: Vec<(String, u32)> = tag_count.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1));
    top_tags.truncate(10);

    Ok(AnnotationStats {
        total_annotations: total,
        unresolved,
        resolved_today,
        collections: state.collections.len() as u32,
        urls_tracked: unique_urls.len() as u32,
        top_tags,
    })
}

#[tauri::command]
pub fn annotation_config() -> Result<AnnotationConfig, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn annotation_set_config(config: AnnotationConfig) -> Result<(), String> {
    let mut state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn annotation_get_for_url(url: String) -> Result<Vec<PageAnnotation>, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let mut results: Vec<PageAnnotation> = state.annotations.iter()
        .filter(|a| a.url == url)
        .cloned()
        .collect();
    results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(results)
}

#[tauri::command]
pub fn annotation_search(query: String) -> Result<Vec<PageAnnotation>, String> {
    let state = ANNOTATION_STATE.lock().map_err(|e| e.to_string())?;
    let q = query.to_lowercase();
    let mut results: Vec<PageAnnotation> = state.annotations.iter()
        .filter(|a| {
            a.comment.to_lowercase().contains(&q)
                || a.highlighted_text.to_lowercase().contains(&q)
                || a.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();
    results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(results)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_create_and_get() {
        let id = annotation_create(
            "https://example.com".into(),
            "Example".into(),
            "#main".into(),
            "Hello world".into(),
            "Test comment".into(),
            Some("question".into()),
            Some(vec!["test".into()]),
        ).unwrap();
        assert!(id.starts_with("ann-"));

        let ann = annotation_get(id.clone()).unwrap();
        assert_eq!(ann.url, "https://example.com");
        assert_eq!(ann.annotation_type, "question");
        assert_eq!(ann.tags, vec!["test"]);
        assert!(!ann.resolved);
    }

    #[test]
    fn test_annotation_resolve_unresolve() {
        let id = annotation_create(
            "https://example.com".into(),
            "Example".into(),
            "#main".into(),
            "text".into(),
            "a bug".into(),
            Some("issue".into()),
            None,
        ).unwrap();

        annotation_resolve(id.clone()).unwrap();
        let ann = annotation_get(id.clone()).unwrap();
        assert!(ann.resolved);
        assert!(ann.resolved_at.is_some());

        annotation_unresolve(id.clone()).unwrap();
        let ann = annotation_get(id.clone()).unwrap();
        assert!(!ann.resolved);
        assert!(ann.resolved_at.is_none());
    }

    #[test]
    fn test_annotation_list_filters() {
        let all = annotation_list(None, None, None).unwrap();
        assert!(all.len() >= 8);

        let unresolved = annotation_list(None, Some(false), None).unwrap();
        assert!(unresolved.len() < all.len());

        let github = annotation_list(
            Some("https://github.com/neo/neotrix".into()),
            None, None,
        ).unwrap();
        assert_eq!(github.len(), 3);
    }

    #[test]
    fn test_annotation_search() {
        let results = annotation_search("feature".into()).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|a| a.id == "ann-seed-001"));

        let results = annotation_search("HyperCube".into()).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|a| a.id == "ann-seed-007"));
    }
}
