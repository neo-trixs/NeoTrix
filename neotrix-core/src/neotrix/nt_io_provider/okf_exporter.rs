use log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Re-export knowledge engine types
// ---------------------------------------------------------------------------
use crate::neotrix::nt_mind::knowledge_engine::graph::KnowledgeEngine;
use crate::neotrix::nt_mind::knowledge_engine::types::KnowledgeEntry;

/// OKF concept types matching NeoTrix knowledge entry types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OkfConceptType {
    Table,
    Dataset,
    Metric,
    Playbook,
    ApiEndpoint,
    Reference,
    Concept,
    Entity,
    Relation,
    Pattern,
}

impl OkfConceptType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OkfConceptType::Table => "table",
            OkfConceptType::Dataset => "dataset",
            OkfConceptType::Metric => "metric",
            OkfConceptType::Playbook => "playbook",
            OkfConceptType::ApiEndpoint => "api-endpoint",
            OkfConceptType::Reference => "reference",
            OkfConceptType::Concept => "concept",
            OkfConceptType::Entity => "entity",
            OkfConceptType::Relation => "relation",
            OkfConceptType::Pattern => "pattern",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "table" => OkfConceptType::Table,
            "dataset" => OkfConceptType::Dataset,
            "metric" => OkfConceptType::Metric,
            "playbook" => OkfConceptType::Playbook,
            "api-endpoint" => OkfConceptType::ApiEndpoint,
            "reference" => OkfConceptType::Reference,
            "concept" => OkfConceptType::Concept,
            "entity" => OkfConceptType::Entity,
            "relation" => OkfConceptType::Relation,
            "pattern" => OkfConceptType::Pattern,
            _ => OkfConceptType::Concept,
        }
    }

    /// Map a KnowledgeEntry `source.name()` to the best OKF type.
    pub fn from_entry_type(entry_type: &str) -> Self {
        match entry_type.to_lowercase().as_str() {
            "table" | "dataset" => OkfConceptType::Table,
            "metric" | "measurement" | "kpi" => OkfConceptType::Metric,
            "playbook" | "tutorial" | "guide" | "howto" => OkfConceptType::Playbook,
            "api" | "endpoint" | "api-endpoint" => OkfConceptType::ApiEndpoint,
            "reference" | "doc" | "documentation" => OkfConceptType::Reference,
            "entity" | "person" | "organization" | "place" => OkfConceptType::Entity,
            "relation" | "edge" | "link" => OkfConceptType::Relation,
            "pattern" | "template" | "blueprint" => OkfConceptType::Pattern,
            _ => OkfConceptType::Concept,
        }
    }
}

/// YAML frontmatter for an OKF concept document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkfFrontmatter {
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    pub tags: Option<Vec<String>>,
    pub timestamp: Option<String>,
    pub vsa_id: Option<String>,
}

/// An OKF concept document.
#[derive(Debug, Clone)]
pub struct OkfConcept {
    pub frontmatter: OkfFrontmatter,
    pub body: String,
    pub path: String,
}

/// OKF bundle: a directory tree of markdown files.
#[derive(Debug, Clone)]
pub struct OkfBundle {
    pub root: PathBuf,
    pub concepts: Vec<OkfConcept>,
    pub log_entries: Vec<String>,
}

/// Minimal knowledge graph interface for OKF export.
/// Provides lookup needed to resolve cross-links and VSA IDs.
pub struct KnowledgeGraph {
    entries: HashMap<String, KnowledgeEntry>,
    title_to_id: HashMap<String, String>,
}

impl KnowledgeGraph {
    pub fn new(entries: Vec<KnowledgeEntry>) -> Self {
        let mut title_to_id = HashMap::new();
        let mut map = HashMap::new();
        for e in entries {
            title_to_id.insert(e.title.clone(), e.id.clone());
            map.insert(e.id.clone(), e);
        }
        KnowledgeGraph {
            entries: map,
            title_to_id,
        }
    }

    pub fn get(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.entries.get(id)
    }

    pub fn find_by_title(&self, title: &str) -> Option<&KnowledgeEntry> {
        self.title_to_id
            .get(title)
            .and_then(|id| self.entries.get(id))
    }

    pub fn from_engine(engine: &KnowledgeEngine) -> Self {
        let entries: Vec<KnowledgeEntry> = engine.entries.values().cloned().collect();
        KnowledgeGraph::new(entries)
    }
}

// ---------------------------------------------------------------------------
// OkfExporter
// ---------------------------------------------------------------------------

pub struct OkfExporter {
    bundle: OkfBundle,
}

impl OkfExporter {
    pub fn new(root: PathBuf) -> Self {
        OkfExporter {
            bundle: OkfBundle {
                root,
                concepts: Vec::new(),
                log_entries: Vec::new(),
            },
        }
    }

    /// Convert a KnowledgeEntry to an OkfConcept and add it to the bundle.
    pub fn add_concept(&mut self, entry: &KnowledgeEntry, kg: &KnowledgeGraph) {
        let concept = knowledge_entry_to_okf(entry, kg);
        self.bundle.concepts.push(concept);
    }

    /// Add a log entry (timestamped message for log.md).
    pub fn add_log_entry(&mut self, message: String) {
        let ts = chrono::Utc::now().to_rfc3339();
        self.bundle
            .log_entries
            .push(format!("- {} {}", ts, message));
    }

    /// Write the complete OKF bundle to disk.
    pub fn export(&self) -> Result<(), ExportError> {
        log::warn!("TODO: Split into multiple files (okf_exporter.rs >1000 lines)");
        let root = &self.bundle.root;
        fs::create_dir_all(root).map_err(|e| ExportError::Io(e.to_string()))?;

        // Group concepts by directory prefix for hierarchical index generation.
        let mut dir_groups: HashMap<String, Vec<&OkfConcept>> = HashMap::new();
        for concept in &self.bundle.concepts {
            // Determine the directory key: parent of path (or "" for root).
            let dir = Path::new(&concept.path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            dir_groups.entry(dir).or_default().push(concept);
        }

        for concept in &self.bundle.concepts {
            let file_path = root.join(format!("{}.md", concept.path));
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ExportError::Io(format!("mkdir {}: {}", parent.display(), e)))?;
            }
            let content = concept_to_markdown(concept);
            let mut f = fs::File::create(&file_path)
                .map_err(|e| ExportError::Io(format!("create {}: {}", file_path.display(), e)))?;
            f.write_all(content.as_bytes())
                .map_err(|e| ExportError::Io(format!("write {}: {}", file_path.display(), e)))?;
        }

        // Generate index.md at each directory level.
        for (dir, concepts_in_dir) in &dir_groups {
            let index_content = self.generate_index(concepts_in_dir);
            let index_path = if dir.is_empty() {
                root.join("index.md")
            } else {
                root.join(format!("{}/index.md", dir))
            };
            if let Some(parent) = index_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ExportError::Io(format!("mkdir {}: {}", parent.display(), e)))?;
            }
            let mut f = fs::File::create(&index_path)
                .map_err(|e| ExportError::Io(format!("create {}: {}", index_path.display(), e)))?;
            f.write_all(index_content.as_bytes())
                .map_err(|e| ExportError::Io(format!("write {}: {}", index_path.display(), e)))?;
        }

        // Write log.md.
        let log_path = root.join("log.md");
        let log_content = self.generate_log();
        {
            let mut f = fs::File::create(&log_path)
                .map_err(|e| ExportError::Io(format!("create log.md: {}", e)))?;
            f.write_all(log_content.as_bytes())
                .map_err(|e| ExportError::Io(format!("write log.md: {}", e)))?;
        }

        // Write graph.json (NeoTrix extension: VSA similarity edges).
        let graph_json = self.generate_graph_json();
        let graph_path = root.join("graph.json");
        {
            let mut f = fs::File::create(&graph_path)
                .map_err(|e| ExportError::Io(format!("create graph.json: {}", e)))?;
            f.write_all(graph_json.as_bytes())
                .map_err(|e| ExportError::Io(format!("write graph.json: {}", e)))?;
        }

        Ok(())
    }

    /// Import an OKF bundle from disk, returning parsed concepts.
    pub fn import(path: &Path) -> Result<Vec<OkfConcept>, ExportError> {
        if !path.is_dir() {
            return Err(ExportError::NotFound(format!(
                "{} is not a directory",
                path.display()
            )));
        }
        let mut concepts = Vec::new();
        let md_files = collect_md_files(path)?;
        for file_path in md_files {
            // Skip index.md and log.md.
            let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if fname == "index.md" || fname == "log.md" {
                continue;
            }
            let content = fs::read_to_string(&file_path)
                .map_err(|e| ExportError::Io(format!("read {}: {}", file_path.display(), e)))?;
            if let Some(concept) = parse_okf_document(&content, &file_path, path) {
                concepts.push(concept);
            }
        }
        Ok(concepts)
    }

    /// Generate index.md content for a list of concepts.
    pub fn generate_index(&self, concepts: &[&OkfConcept]) -> String {
        let mut lines = Vec::new();
        lines.push("# OKF Bundle Index\n".to_string());
        let now = chrono::Utc::now();
        lines.push(format!(
            "_Generated: {}_\n",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        lines.push(format!("Total concepts: {}\n", concepts.len()));
        lines.push("## Concepts\n".to_string());

        // Group by type.
        let mut by_type: HashMap<&str, Vec<&&OkfConcept>> = HashMap::new();
        for c in concepts {
            by_type
                .entry(c.frontmatter.r#type.as_str())
                .or_default()
                .push(c);
        }
        let mut types: Vec<&&str> = by_type.keys().collect();
        types.sort();

        for t in types {
            let items = &by_type[t];
            lines.push(format!("### {}\n", t));
            for c in items {
                let title_display = c.frontmatter.title.as_deref().unwrap_or("(untitled)");
                let desc = c.frontmatter.description.as_deref().unwrap_or("");
                // Use an absolute-like path that is relative to bundle root.
                let rel_path = format!("{}.md", c.path);
                lines.push(format!("- [{}]({})", title_display, rel_path));
                if !desc.is_empty() {
                    lines.push(format!("  _{}_", desc));
                }
                lines.push(String::new());
            }
        }
        lines.join("\n")
    }

    /// Generate log.md content from log entries.
    pub fn generate_log(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# OKF Bundle Log\n".to_string());
        lines.push("\n## History\n".to_string());
        if self.bundle.log_entries.is_empty() {
            lines.push("_No entries._".to_string());
        } else {
            for entry in &self.bundle.log_entries {
                lines.push(entry.clone());
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }

    /// Generate graph.json with VSA similarity edges (NeoTrix extension).
    pub fn generate_graph_json(&self) -> String {
        let mut nodes = Vec::new();
        let mut node_map: HashMap<String, usize> = HashMap::new();

        for (i, c) in self.bundle.concepts.iter().enumerate() {
            let name = c.frontmatter.title.as_deref().unwrap_or("").to_string();
            let vsa_id = c.frontmatter.vsa_id.as_deref().unwrap_or("").to_string();
            nodes.push(format!(
                r#"{{"id":{},"name":"{}","type":"{}","vsa_id":"{}"}}"#,
                i,
                name.replace('\\', "\\\\").replace('"', "\\\""),
                c.frontmatter.r#type,
                vsa_id
            ));
            node_map.insert(c.path.clone(), i);
        }

        // Extract cross-links from body and add edges.
        let mut edges = Vec::new();
        for c in &self.bundle.concepts {
            let from_idx = match node_map.get(&c.path) {
                Some(&idx) => idx,
                None => continue,
            };
            let links = extract_links(&c.body);
            for (_, target_path) in &links {
                // Normalize target: strip .md suffix if present, get path-like key.
                let target_key = target_path
                    .strip_suffix(".md")
                    .unwrap_or(target_path)
                    .to_string();
                if let Some(&to_idx) = node_map.get(&target_key) {
                    if from_idx != to_idx {
                        edges.push(format!(
                            r#"{{"source":{},"target":{},"label":"cross-ref"}}"#,
                            from_idx, to_idx
                        ));
                    }
                }
            }
        }

        format!(
            r#"{{"nodes":[{}],"edges":[{}]}}"#,
            nodes.join(","),
            edges.join(",")
        )
    }

    /// Access the underlying bundle.
    pub fn bundle(&self) -> &OkfBundle {
        &self.bundle
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Generate YAML frontmatter string (with enclosing --- markers).
pub fn yaml_frontmatter(fm: &OkfFrontmatter) -> String {
    let mut lines = Vec::new();
    lines.push("---".to_string());
    lines.push(format!("type: {}", fm.r#type));
    if let Some(ref title) = fm.title {
        lines.push(format!("title: {}", escape_yaml(title)));
    }
    if let Some(ref desc) = fm.description {
        lines.push(format!("description: {}", escape_yaml(desc)));
    }
    if let Some(ref resource) = fm.resource {
        lines.push(format!("resource: {}", resource));
    }
    if let Some(ref tags) = fm.tags {
        if !tags.is_empty() {
            lines.push(format!("tags: [{}]", tags.join(", ")));
        }
    }
    if let Some(ref ts) = fm.timestamp {
        lines.push(format!("timestamp: {}", ts));
    }
    if let Some(ref vsa_id) = fm.vsa_id {
        lines.push(format!("vsa_id: {}", vsa_id));
    }
    lines.push("---".to_string());
    lines.push(String::new());
    lines.join("\n")
}

/// Extract markdown links `[text](url)` from body text.
pub fn extract_links(body: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let mut pos = 0;
    while pos < body.len() {
        // Look for `[` which may start a link
        if let Some(open_bracket) = body[pos..].find('[') {
            let start = pos + open_bracket;
            // Find the closing `]`
            if let Some(close_bracket) = body[start..].find(']') {
                let text_end = start + close_bracket;
                // Check for `(url)` right after `]`
                let after = text_end + 1;
                if after < body.len() && body.as_bytes()[after] == b'(' {
                    if let Some(close_paren) = body[after..].find(')') {
                        let text = &body[start + 1..text_end];
                        let url = &body[after + 1..after + close_paren];
                        links.push((text.to_string(), url.to_string()));
                        pos = after + close_paren + 1;
                        continue;
                    }
                }
            }
            pos = start + 1;
        } else {
            break;
        }
    }
    links
}

/// Look up a concept name from VSA ID in the knowledge graph.
/// Returns "vsa:{vsa_id}" as fallback if not found.
pub fn vsa_to_concept_name(vsa_id: &str, kg: &KnowledgeGraph) -> String {
    // Search all entries for a matching vsa_id (stored in source_url or description as convention).
    for entry in kg.entries.values() {
        if entry.source_url.contains(vsa_id) || entry.id == vsa_id {
            return entry.title.clone();
        }
        // Check tags for VSA ID match.
        if entry.tags.iter().any(|t| t == vsa_id) {
            return entry.title.clone();
        }
    }
    format!("vsa:{}", vsa_id)
}

/// Convert a KnowledgeEntry to an OkfConcept, resolving cross-links via the graph.
pub fn knowledge_entry_to_okf(entry: &KnowledgeEntry, kg: &KnowledgeGraph) -> OkfConcept {
    let concept_type = OkfConceptType::from_entry_type(entry.source.name());

    let resource = entry
        .embedding
        .as_ref()
        .map(|_| format!("vsa://{}", entry.id));
    let tags = if entry.tags.is_empty() {
        None
    } else {
        Some(entry.tags.clone())
    };
    let timestamp = Some(chrono::Utc::now().to_rfc3339());

    let frontmatter = OkfFrontmatter {
        r#type: concept_type.as_str().to_string(),
        title: Some(entry.title.clone()),
        description: Some(entry.summary.clone()),
        resource,
        tags,
        timestamp,
        vsa_id: Some(entry.id.clone()),
    };

    // Build body: schema table + citations + cross-links.
    let mut body_parts = Vec::new();

    // Schema section.
    body_parts.push("## Schema".to_string());
    body_parts.push("| Field | Value |".to_string());
    body_parts.push("|-------|-------|".to_string());
    body_parts.push(format!("| ID | {} |", entry.id));
    body_parts.push(format!("| Source | {} |", entry.source.name()));
    body_parts.push(format!("| Confidence | {:.2} |", entry.confidence));
    body_parts.push(format!("| Importance | {:.2} |", entry.importance));
    if let Some(ref vsa_id) = frontmatter.vsa_id {
        body_parts.push(format!("| VSA | `{}` |", vsa_id));
    }
    body_parts.push(String::new());

    // Description / body content.
    if !entry.body.is_empty() {
        body_parts.push("## Content".to_string());
        body_parts.push(String::new());
        body_parts.push(entry.body.clone());
        body_parts.push(String::new());
    }

    // Cross-links to related entries.
    let mut cross_links: Vec<String> = Vec::new();
    for related_id in &entry.related_ids {
        if let Some(related) = kg.get(related_id) {
            let (safe_path, rel_title) = concept_path_for_entry(related);
            cross_links.push(format!("- [{}]({}.md)", rel_title, safe_path));
        }
    }
    if !cross_links.is_empty() {
        body_parts.push("## Cross-References".to_string());
        body_parts.push(String::new());
        body_parts.extend(cross_links);
        body_parts.push(String::new());
    }

    // Citations.
    if !entry.source_url.is_empty() {
        body_parts.push("## Citations".to_string());
        body_parts.push(String::new());
        body_parts.push(format!(
            "- Source: [{}]({})",
            entry.source_url, entry.source_url
        ));
        body_parts.push(String::new());
    }

    let body = body_parts.join("\n");

    // Derive file path from title (sanitized).
    let safe_path = sanitize_path(&entry.title);
    let dir_prefix = concept_type.as_str();
    let path = format!("{}/{}", dir_prefix, safe_path);

    OkfConcept {
        frontmatter,
        body,
        path,
    }
}

/// Generate the markdown representation of an OkfConcept.
pub fn concept_to_markdown(concept: &OkfConcept) -> String {
    let mut output = String::new();
    output.push_str(&yaml_frontmatter(&concept.frontmatter));
    output.push_str(&concept.body);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Parse an OKF markdown document, returning an OkfConcept if valid.
fn parse_okf_document(content: &str, file_path: &Path, bundle_root: &Path) -> Option<OkfConcept> {
    let content_trimmed = content.trim();
    if !content_trimmed.starts_with("---") {
        return None;
    }

    // Split on the second "---".
    let end_marker = content_trimmed[3..].find("---")?;
    let yaml_block = &content_trimmed[3..3 + end_marker].trim();
    let body = content_trimmed[3 + end_marker + 3..].trim().to_string();

    // Parse YAML frontmatter (simple line-by-line parser).
    let fm = parse_simple_yaml(yaml_block)?;

    // Derive relative path from bundle root.
    let rel_path = file_path
        .strip_prefix(bundle_root)
        .ok()
        .and_then(|p| {
            let s = p.to_string_lossy().to_string();
            s.strip_suffix(".md").map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            // Fallback: use file stem as path.
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    Some(OkfConcept {
        frontmatter: fm,
        body,
        path: rel_path,
    })
}

/// Minimal YAML parser: handles key: value and key: [list] lines.
fn parse_simple_yaml(yaml: &str) -> Option<OkfFrontmatter> {
    let mut r#type = String::new();
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut resource: Option<String> = None;
    let mut tags: Option<Vec<String>> = None;
    let mut timestamp: Option<String> = None;
    let mut vsa_id: Option<String> = None;

    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "type" => r#type = value.to_string(),
                "title" => title = Some(unquote_yaml(value)),
                "description" => description = Some(unquote_yaml(value)),
                "resource" => resource = Some(unquote_yaml(value)),
                "tags" => {
                    // Handle [a, b, c] or "a, b, c"
                    let inner = value
                        .strip_prefix('[')
                        .and_then(|v| v.strip_suffix(']'))
                        .unwrap_or(value);
                    let parsed: Vec<String> = inner
                        .split(',')
                        .map(|s| unquote_yaml(s.trim()))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !parsed.is_empty() {
                        tags = Some(parsed);
                    }
                }
                "timestamp" => timestamp = Some(unquote_yaml(value)),
                "vsa_id" => vsa_id = Some(unquote_yaml(value)),
                _ => {}
            }
        }
    }

    if r#type.is_empty() {
        return None;
    }

    Some(OkfFrontmatter {
        r#type,
        title,
        description,
        resource,
        tags,
        timestamp,
        vsa_id,
    })
}

/// Remove surrounding quotes from a YAML value.
fn unquote_yaml(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Escape special characters for YAML string values.
fn escape_yaml(s: &str) -> String {
    if s.contains(':') || s.contains('#') || s.contains('"') || s.contains('\'') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// Sanitize a string for use as a file path component.
fn sanitize_path(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('_').to_string();
    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized.to_lowercase()
    }
}

/// Derive an OKF concept path (relative) for a KnowledgeEntry.
fn concept_path_for_entry(entry: &KnowledgeEntry) -> (String, String) {
    let ctype = OkfConceptType::from_entry_type(entry.source.name());
    let safe = sanitize_path(&entry.title);
    (format!("{}/{}", ctype.as_str(), safe), entry.title.clone())
}

/// Recursively collect all .md files in a directory tree.
fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>, ExportError> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    collect_md_files_rec(dir, dir, &mut files)?;
    Ok(files)
}

fn collect_md_files_rec(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ExportError> {
    if current.is_dir() {
        for entry in fs::read_dir(current)
            .map_err(|e| ExportError::Io(format!("read_dir {}: {}", current.display(), e)))?
        {
            let entry = entry.map_err(|e| ExportError::Io(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                collect_md_files_rec(root, &path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ExportError {
    Io(String),
    NotFound(String),
    Parse(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(msg) => write!(f, "IO error: {}", msg),
            ExportError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ExportError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ExportError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::knowledge_engine::types::KnowledgeSourceType;

    fn sample_entry(id: &str, title: &str, entry_type: &str, body: &str) -> KnowledgeEntry {
        let mut e = KnowledgeEntry::new(
            title,
            body,
            KnowledgeSourceType::WebPage,
            "https://example.com",
        );
        e.id = id.to_string();
        e.tags = vec!["test".to_string(), entry_type.to_string()];
        e.confidence = 0.85;
        e.importance = 0.7;
        e
    }

    fn sample_graph(entries: Vec<KnowledgeEntry>) -> KnowledgeGraph {
        KnowledgeGraph::new(entries)
    }

    // -----------------------------------------------------------------------
    // Roundtrip: OkfConcept → markdown → parse back
    // -----------------------------------------------------------------------

    #[test]
    fn test_concept_roundtrip() {
        let mut entry = sample_entry("id-1", "Test Concept", "reference", "Some body content.");
        let mut related = sample_entry("id-2", "Related Item", "concept", "Related body.");
        related.related_ids.push("id-1".to_string());
        entry.related_ids.push("id-2".to_string());

        let kg = sample_graph(vec![entry.clone(), related]);
        let concept = knowledge_entry_to_okf(&entry, &kg);

        let md = concept_to_markdown(&concept);

        // Parse it back using a temp file.
        let tmpdir = std::env::temp_dir().join(format!("okf_roundtrip_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmpdir);
        let file_path = tmpdir.join("test.md");
        fs::write(&file_path, &md).unwrap();

        let parsed = parse_okf_document(&md, &file_path, &tmpdir).unwrap();
        assert_eq!(parsed.frontmatter.r#type, "reference");
        assert_eq!(parsed.frontmatter.title.as_deref(), Some("Test Concept"));
        assert!(parsed.body.contains("Some body content."));
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn test_roundtrip_identity() {
        let entry = sample_entry(
            "r-id",
            "Roundtrip Identity",
            "metric",
            "Metric description.",
        );
        let kg = sample_graph(vec![entry.clone()]);
        let concept = knowledge_entry_to_okf(&entry, &kg);
        let md = concept_to_markdown(&concept);

        let tmpdir = std::env::temp_dir().join(format!("okf_rt_id_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmpdir);
        let parsed = parse_okf_document(&md, &tmpdir.join("x.md"), &tmpdir).unwrap();

        assert_eq!(parsed.frontmatter.title, concept.frontmatter.title);
        assert_eq!(parsed.frontmatter.r#type, concept.frontmatter.r#type);
        assert!(parsed.body.contains("Metric description."));
        let _ = fs::remove_dir_all(&tmpdir);
    }

    // -----------------------------------------------------------------------
    // Index generation
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_index_with_three_concepts() {
        let concepts = vec![
            OkfConcept {
                frontmatter: OkfFrontmatter {
                    r#type: "concept".to_string(),
                    title: Some("Alpha".to_string()),
                    description: Some("First concept".to_string()),
                    resource: None,
                    tags: None,
                    timestamp: None,
                    vsa_id: None,
                },
                body: "Alpha body".to_string(),
                path: "concept/alpha".to_string(),
            },
            OkfConcept {
                frontmatter: OkfFrontmatter {
                    r#type: "reference".to_string(),
                    title: Some("Beta".to_string()),
                    description: Some("Second concept".to_string()),
                    resource: None,
                    tags: None,
                    timestamp: None,
                    vsa_id: None,
                },
                body: "Beta body".to_string(),
                path: "reference/beta".to_string(),
            },
            OkfConcept {
                frontmatter: OkfFrontmatter {
                    r#type: "metric".to_string(),
                    title: Some("Gamma".to_string()),
                    description: Some("Third concept".to_string()),
                    resource: None,
                    tags: None,
                    timestamp: None,
                    vsa_id: None,
                },
                body: "Gamma body".to_string(),
                path: "metric/gamma".to_string(),
            },
        ];

        let exporter = OkfExporter::new(PathBuf::from("/tmp/okf_test"));
        let refs: Vec<&OkfConcept> = concepts.iter().collect();
        let index = exporter.generate_index(&refs);

        assert!(index.contains("Alpha"), "Index should contain Alpha");
        assert!(index.contains("Beta"), "Index should contain Beta");
        assert!(index.contains("Gamma"), "Index should contain Gamma");
        assert!(
            index.contains("concept/alpha.md"),
            "Index should link to alpha.md"
        );
        assert!(
            index.contains("reference/beta.md"),
            "Index should link to beta.md"
        );
        assert!(
            index.contains("metric/gamma.md"),
            "Index should link to gamma.md"
        );
        assert!(
            index.contains("First concept"),
            "Index should contain description"
        );
        assert!(
            index.contains("Third concept"),
            "Index should contain description"
        );
        assert!(
            index.starts_with("# OKF Bundle Index"),
            "Index should start with title"
        );
    }

    #[test]
    fn test_generate_index_empty() {
        let exporter = OkfExporter::new(PathBuf::from("/tmp/okf_empty"));
        let index = exporter.generate_index(&[]);
        assert!(index.contains("Total concepts: 0"));
        assert!(index.starts_with("# OKF Bundle Index"));
    }

    // -----------------------------------------------------------------------
    // Cross-link extraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_links_simple() {
        let body = "See [example](https://example.com) for details.";
        let links = extract_links(body);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "example");
        assert_eq!(links[0].1, "https://example.com");
    }

    #[test]
    fn test_extract_links_multiple() {
        let body = "Read [paper](arxiv.org/paper1) and [code](github.com/repo).";
        let links = extract_links(body);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].0, "paper");
        assert_eq!(links[1].0, "code");
    }

    #[test]
    fn test_extract_links_no_links() {
        let body = "Plain text without any markdown links.";
        let links = extract_links(body);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_brackets_no_parens() {
        let body = "This has [brackets] but no URL.";
        let links = extract_links(body);
        assert!(links.is_empty());
    }

    // -----------------------------------------------------------------------
    // YAML frontmatter
    // -----------------------------------------------------------------------

    #[test]
    fn test_yaml_frontmatter_full() {
        let fm = OkfFrontmatter {
            r#type: "concept".to_string(),
            title: Some("My Concept".to_string()),
            description: Some("A test concept".to_string()),
            resource: Some("vsa://abc123".to_string()),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            timestamp: Some("2026-06-14T00:00:00Z".to_string()),
            vsa_id: Some("abc123".to_string()),
        };
        let yaml = yaml_frontmatter(&fm);
        assert!(yaml.starts_with("---\n"));
        assert!(yaml.contains("type: concept"));
        assert!(yaml.contains("title: My Concept"));
        assert!(yaml.contains("resource: vsa://abc123"));
        assert!(yaml.contains("tags: [tag1, tag2]"));
        assert!(yaml.contains("timestamp: 2026-06-14T00:00:00Z"));
        assert!(yaml.contains("vsa_id: abc123"));
        assert!(yaml.ends_with("---\n\n"));
    }

    #[test]
    fn test_yaml_frontmatter_minimal() {
        let fm = OkfFrontmatter {
            r#type: "metric".to_string(),
            title: None,
            description: None,
            resource: None,
            tags: None,
            timestamp: None,
            vsa_id: None,
        };
        let yaml = yaml_frontmatter(&fm);
        assert!(yaml.starts_with("---\ntype: metric\n"));
        assert!(!yaml.contains("title:"));
        assert!(!yaml.contains("tags:"));
    }

    // -----------------------------------------------------------------------
    // concept_to_markdown + parse back
    // -----------------------------------------------------------------------

    #[test]
    fn test_concept_to_markdown_basic() {
        let concept = OkfConcept {
            frontmatter: OkfFrontmatter {
                r#type: "entity".to_string(),
                title: Some("Test Entity".to_string()),
                description: None,
                resource: None,
                tags: None,
                timestamp: None,
                vsa_id: None,
            },
            body: "# Content\n\nSome markdown body.".to_string(),
            path: "entity/test_entity".to_string(),
        };
        let md = concept_to_markdown(&concept);
        assert!(md.starts_with("---\ntype: entity\ntitle: Test Entity\n---"));
        assert!(md.contains("# Content"));
        assert!(md.contains("Some markdown body."));
    }

    // -----------------------------------------------------------------------
    // Empty bundle export (mock directory)
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_bundle_export() {
        let tmpdir = std::env::temp_dir().join(format!("okf_empty_bundle_{}", std::process::id()));
        let exporter = OkfExporter::new(tmpdir.clone());
        let result = exporter.export();
        assert!(
            result.is_ok(),
            "Empty bundle export should succeed: {:?}",
            result
        );
        assert!(tmpdir.join("index.md").exists());
        assert!(tmpdir.join("log.md").exists());
        assert!(tmpdir.join("graph.json").exists());
        let _ = fs::remove_dir_all(&tmpdir);
    }

    // -----------------------------------------------------------------------
    // Single concept export
    // -----------------------------------------------------------------------

    #[test]
    fn test_single_concept_export() {
        let tmpdir = std::env::temp_dir().join(format!("okf_single_{}", std::process::id()));
        let mut exporter = OkfExporter::new(tmpdir.clone());
        let entry = sample_entry("single-1", "Single Concept", "reference", "Reference body.");
        let kg = sample_graph(vec![entry.clone()]);
        exporter.add_concept(&entry, &kg);
        let result = exporter.export();
        assert!(result.is_ok(), "Single concept export: {:?}", result);

        let concept_file = tmpdir.join("reference/single_concept.md");
        assert!(
            concept_file.exists(),
            "Concept file should exist: {:?}",
            concept_file
        );
        let content = fs::read_to_string(&concept_file).unwrap();
        assert!(content.contains("type: reference"));
        assert!(content.contains("title: Single Concept"));
        assert!(content.contains("Reference body."));
        let _ = fs::remove_dir_all(&tmpdir);
    }

    // -----------------------------------------------------------------------
    // Log entry formatting
    // -----------------------------------------------------------------------

    #[test]
    fn test_log_entry_formatting() {
        let mut exporter = OkfExporter::new(PathBuf::from("/tmp/okf_log_test"));
        exporter.add_log_entry("Exported 5 concepts.".to_string());
        exporter.add_log_entry("Rebuilt index.".to_string());
        let log = exporter.generate_log();
        assert!(log.contains("## History"));
        assert!(log.contains("Exported 5 concepts."));
        assert!(log.contains("Rebuilt index."));
        // Each entry should start with "- " (markdown list).
        let lines: Vec<&str> = log.lines().filter(|l| l.starts_with("- ")).collect();
        assert_eq!(lines.len(), 2, "Should have 2 log entries");
    }

    #[test]
    fn test_log_entry_empty() {
        let exporter = OkfExporter::new(PathBuf::from("/tmp/okf_log_empty"));
        let log = exporter.generate_log();
        assert!(log.contains("_No entries._"));
    }

    // -----------------------------------------------------------------------
    // graph.json generation
    // -----------------------------------------------------------------------

    #[test]
    fn test_graph_json_with_links() {
        let concepts = vec![
            OkfConcept {
                frontmatter: OkfFrontmatter {
                    r#type: "concept".to_string(),
                    title: Some("A".to_string()),
                    description: None,
                    resource: None,
                    tags: None,
                    timestamp: None,
                    vsa_id: Some("vsa-1".to_string()),
                },
                body: "See [B](concept/b.md) for more.".to_string(),
                path: "concept/a".to_string(),
            },
            OkfConcept {
                frontmatter: OkfFrontmatter {
                    r#type: "concept".to_string(),
                    title: Some("B".to_string()),
                    description: None,
                    resource: None,
                    tags: None,
                    timestamp: None,
                    vsa_id: Some("vsa-2".to_string()),
                },
                body: "Related to [A](concept/a.md).".to_string(),
                path: "concept/b".to_string(),
            },
        ];
        let exporter = OkfExporter {
            bundle: OkfBundle {
                root: PathBuf::from("/tmp/graph_test"),
                concepts,
                log_entries: Vec::new(),
            },
        };
        let json_str = exporter.generate_graph_json();
        assert!(json_str.contains("vsa-1"));
        assert!(json_str.contains("vsa-2"));
        // Should have edges from cross-links.
        assert!(json_str.contains("cross-ref"));
    }

    // -----------------------------------------------------------------------
    // Import from disk
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_roundtrip() {
        let tmpdir = std::env::temp_dir().join(format!("okf_import_{}", std::process::id()));
        let mut exporter = OkfExporter::new(tmpdir.clone());

        let entry = sample_entry("imp-1", "Importable", "pattern", "Pattern details.");
        let kg = sample_graph(vec![entry]);
        exporter.add_concept(&kg.get("imp-1").unwrap(), &kg);
        exporter.export().unwrap();

        let imported = OkfExporter::import(&tmpdir).unwrap();
        assert_eq!(imported.len(), 1, "Should import 1 concept");
        assert_eq!(imported[0].frontmatter.title.as_deref(), Some("Importable"));
        assert_eq!(imported[0].frontmatter.r#type, "pattern");
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn test_import_nonexistent_directory() {
        let result = OkfExporter::import(Path::new("/nonexistent/okf_bundle"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // KnowledgeEntry → OkfConcept conversion
    // -----------------------------------------------------------------------

    #[test]
    fn test_knowledge_entry_conversion() {
        let entry = sample_entry("conv-1", "Conversion Test", "dataset", "Dataset content.");
        let kg = sample_graph(vec![entry.clone()]);
        let concept = knowledge_entry_to_okf(&entry, &kg);

        assert_eq!(concept.frontmatter.r#type, "table");
        assert_eq!(
            concept.frontmatter.title.as_deref(),
            Some("Conversion Test")
        );
        assert!(concept.frontmatter.vsa_id.is_some());
        assert!(concept.body.contains("Dataset content."));
        assert!(concept.path.starts_with("table/"));
    }

    #[test]
    fn test_okf_concept_type_from_str() {
        assert_eq!(OkfConceptType::from_str("table"), OkfConceptType::Table);
        assert_eq!(OkfConceptType::from_str("pattern"), OkfConceptType::Pattern);
        assert_eq!(
            OkfConceptType::from_str("unknown_type"),
            OkfConceptType::Concept
        );
    }

    #[test]
    fn test_okf_concept_type_as_str() {
        assert_eq!(OkfConceptType::Table.as_str(), "table");
        assert_eq!(OkfConceptType::ApiEndpoint.as_str(), "api-endpoint");
        assert_eq!(OkfConceptType::Playbook.as_str(), "playbook");
    }

    #[test]
    fn test_okf_concept_type_from_entry_type() {
        assert_eq!(
            OkfConceptType::from_entry_type("Metric"),
            OkfConceptType::Metric
        );
        assert_eq!(
            OkfConceptType::from_entry_type("howto"),
            OkfConceptType::Playbook
        );
        assert_eq!(
            OkfConceptType::from_entry_type("reference"),
            OkfConceptType::Reference
        );
        assert_eq!(
            OkfConceptType::from_entry_type("random_string"),
            OkfConceptType::Concept
        );
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_sanitize_path_spaces_and_special_chars() {
        let path = sanitize_path("My Awesome Concept!!! (2026)");
        assert_eq!(path, "my_awesome_concept__2026_");
    }

    #[test]
    fn test_sanitize_path_empty() {
        assert_eq!(sanitize_path(""), "untitled");
    }

    #[test]
    fn test_extract_links_with_nested_brackets() {
        let body =
            "See [text [with] brackets](https://example.com) and [normal](https://normal.com).";
        let links = extract_links(body);
        // The simple parser considers the first `]` as the end.
        assert_eq!(links.len(), 2, "Should extract 2 links");
        assert_eq!(links[0].0, "text [with] brackets");
    }

    #[test]
    fn test_knowledge_graph_from_entries() {
        let e1 = sample_entry("kg-1", "Graph Entry 1", "concept", "Body 1");
        let e2 = sample_entry("kg-2", "Graph Entry 2", "reference", "Body 2");
        let kg = sample_graph(vec![e1, e2]);
        assert!(kg.get("kg-1").is_some());
        assert!(kg.get("kg-2").is_some());
        assert!(kg.find_by_title("Graph Entry 1").is_some());
        assert!(kg.find_by_title("Nonexistent").is_none());
    }

    #[test]
    fn test_yaml_frontmatter_parse_basic() {
        let yaml = r#"
type: concept
title: Parser Test
description: "A description with colons: yes"
tags: [a, b, c]
"#;
        let fm = parse_simple_yaml(yaml).unwrap();
        assert_eq!(fm.r#type, "concept");
        assert_eq!(fm.title.as_deref(), Some("Parser Test"));
        assert_eq!(
            fm.description.as_deref(),
            Some("A description with colons: yes")
        );
        let expected: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(fm.tags.as_deref(), Some(expected.as_slice()));
    }

    #[test]
    fn test_okf_exporter_new_and_bundle() {
        let exporter = OkfExporter::new(PathBuf::from("/tmp/test_root"));
        assert_eq!(exporter.bundle().root, PathBuf::from("/tmp/test_root"));
        assert!(exporter.bundle().concepts.is_empty());
        assert!(exporter.bundle().log_entries.is_empty());
    }
}
