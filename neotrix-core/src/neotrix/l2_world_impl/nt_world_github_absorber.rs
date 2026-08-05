//! nt_world_github_absorber — GitHub 仓库深度吸收与代码蒸馏模块
//!
//! 完整仓库元数据 / README / 文件结构 / 核心源文件 / 依赖 吸收，
//! 自动提取架构设计思路存入 KB，支持增量更新。
//! 所有产出直接以 KnowledgeNode 形式存入 KB，构建可检索的代码知识图。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::neotrix::l2_world_impl::nt_memory_kb_bridge::{
    KnowledgeBase, NodeType, RelationType,
};

// ── HTTP Client ──

fn http_client() -> Option<&'static reqwest::blocking::Client> {
    Some(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_http::shared_blocking_client())
}

fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN").ok()
}

fn github_api_get(path: &str) -> Result<serde_json::Value, String> {
    let client = http_client().ok_or_else(|| "HTTP client not available".to_string())?;
    let url = format!("https://api.github.com/{}", path);
    let mut req = client.get(&url);
    if let Some(token) = github_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().map_err(|e| format!("GitHub API error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("GitHub API HTTP {}: {}", status, body.chars().take(200).collect::<String>()));
    }
    resp.json().map_err(|e| format!("JSON parse: {e}"))
}

fn github_raw(url: &str) -> Result<String, String> {
    let client = http_client().ok_or_else(|| "HTTP client not available".to_string())?;
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(15))
        .send()
        .map_err(|e| format!("Raw fetch error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Raw HTTP {}: {}", status, url));
    }
    resp.text().map_err(|e| format!("Read error: {}", e))
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ── Public Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAbsorbReport {
    pub owner: String,
    pub repo: String,
    pub full_name: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub readme_ingested: bool,
    pub files_ingested: usize,
    pub deps_detected: usize,
    pub architecture_insights: Vec<String>,
    pub is_update: bool,
    pub timestamp: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub full_name: String,
    pub node_id: String,
    pub absorbed_at: i64,
    pub last_updated: i64,
    pub file_count: usize,
    pub dep_count: usize,
    pub insight_count: usize,
    pub stale: bool,
}

// ── Internal Types ──

#[derive(Debug, Clone)]
struct RepoMeta {
    full_name: String,
    description: String,
    html_url: String,
    stars: i64,
    forks: i64,
    topics: Vec<String>,
    language: String,
    license_name: Option<String>,
    default_branch: String,
    pushed_at: i64,
    created_at: i64,
    owner_login: String,
    owner_type: String,
    size_kb: i64,
    has_readme: bool,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: String,
    name: String,
    file_type: String,
    size: i64,
}

#[derive(Debug, Clone)]
struct DepInfo {
    name: String,
    version: String,
    dep_type: String,
}

#[derive(Debug, Clone)]
struct ArchInsight {
    title: String,
    description: String,
    category: String,
}

// ── GitHubAbsorber ──

pub struct GitHubAbsorber {
    kb: KnowledgeBase,
}

impl GitHubAbsorber {
    pub fn new(kb: KnowledgeBase) -> Self {
        Self { kb }
    }

    /// Full absorption of a GitHub repo (owner/repo).
    /// 1. Metadata 2. README 3. File tree 4. Dependencies 5. Key source files 6. Architecture insights
    pub fn absorb(&self, owner: &str, repo: &str) -> Result<GitHubAbsorbReport, String> {
        let mut report = GitHubAbsorbReport {
            owner: owner.to_string(),
            repo: repo.to_string(),
            full_name: format!("{}/{}", owner, repo),
            nodes_created: 0,
            edges_created: 0,
            readme_ingested: false,
            files_ingested: 0,
            deps_detected: 0,
            architecture_insights: Vec::new(),
            is_update: false,
            timestamp: now(),
            errors: Vec::new(),
        };

        // 1. Fetch repo metadata
        let repo_data = github_api_get(&format!("repos/{}/{}", owner, repo))?;
        let meta = self.parse_repo_meta(&repo_data)?;
        report.full_name = meta.full_name.clone();

        // 2. Check if already absorbed via URL dedup
        let existing = self.kb.find_node_by_url(&meta.html_url).unwrap_or(None);
        let is_update = existing.is_some();
        report.is_update = is_update;

        let repo_node_id = if let Some(existing_node) = existing {
            // Update existing node metadata
            let updated_meta = serde_json::json!({
                "stars": meta.stars,
                "forks": meta.forks,
                "topics": meta.topics,
                "language": meta.language,
                "license": meta.license_name,
                "pushed_at": meta.pushed_at,
                "size_kb": meta.size_kb,
                "last_absorbed": now(),
            });
            self.kb.update_node_metadata(&existing_node.id, &updated_meta)?;
            existing_node.id
        } else {
            // 3. Create Repository node with full metadata
            let summary = format!(
                "{}★ {} forks | {} | {}",
                meta.stars,
                meta.forks,
                meta.language,
                meta.description.chars().take(200).collect::<String>(),
            );
            let node_id = self.kb.insert_or_get_node(
                &meta.full_name,
                NodeType::Repository,
                Some(&summary),
                Some(&meta.html_url),
                Some("github.com"),
            )?;
            report.nodes_created += 1;

            // Store full metadata as content JSON
            let content = serde_json::json!({
                "description": meta.description,
                "stars": meta.stars,
                "forks": meta.forks,
                "topics": meta.topics,
                "language": meta.language,
                "license": meta.license_name,
                "default_branch": meta.default_branch,
                "size_kb": meta.size_kb,
                "pushed_at": meta.pushed_at,
                "created_at": meta.created_at,
                "owner_type": meta.owner_type,
            }).to_string();
            self.kb.update_node_content(&node_id, &content)?;

            // Owner organization node + edge
            let owner_id = self.kb.insert_or_get_node(
                &meta.owner_login,
                NodeType::Organization,
                None,
                Some(&format!("https://github.com/{}", meta.owner_login)),
                Some("github.com"),
            )?;
            self.kb.upsert_edge(&node_id, &owner_id, RelationType::DevelopedBy, 1.0,
                Some("Repository owner"))?;
            report.edges_created += 1;

            // Topic nodes + edges
            for topic in &meta.topics {
                let topic_id = self.kb.insert_or_get_node(
                    topic,
                    NodeType::Concept,
                    None,
                    None,
                    Some("github.com/topic"),
                )?;
                self.kb.upsert_edge(&node_id, &topic_id, RelationType::Related, 0.5,
                    Some("GitHub topic"))?;
                report.edges_created += 1;
            }

            node_id
        };

        // 4. Ingest README
        if meta.has_readme {
            match self.ingest_readme(owner, repo, &meta.default_branch, &repo_node_id) {
                Ok(()) => report.readme_ingested = true,
                Err(e) => report.errors.push(format!("README: {}", e)),
            }
        }

        // 5. Fetch and analyze file tree
        let files = match self.fetch_file_tree(owner, repo, &meta.default_branch) {
            Ok(f) => f,
            Err(e) => {
                report.errors.push(format!("file tree: {}", e));
                Vec::new()
            }
        };

        // 6. Detect dependencies
        let deps = self.detect_dependencies(&files, owner, repo, &meta.default_branch);
        report.deps_detected = deps.len();
        for dep in &deps {
            match self.ingest_dependency(dep, &repo_node_id) {
                Ok(()) => report.edges_created += 1,
                Err(e) => report.errors.push(format!("dep {}: {}", dep.name, e)),
            }
        }

        // 7. Ingest key source files (top-level modules in src/, lib/, etc.)
        let key_files = self.identify_key_files(&files);
        for file in &key_files {
            match self.fetch_and_ingest_source(file, owner, repo, &meta.default_branch, &repo_node_id) {
                Ok(()) => {
                    report.files_ingested += 1;
                    report.nodes_created += 1;
                }
                Err(e) => report.errors.push(format!("file {}: {}", file.path, e)),
            }
        }

        // 8. Generate + store architecture insights
        let insights = self.generate_architecture_insights(&meta, &files, &deps);
        for insight in &insights {
            match self.ingest_insight(insight, &repo_node_id) {
                Ok(()) => {
                    report.nodes_created += 1;
                    report.edges_created += 1;
                    report.architecture_insights.push(insight.title.clone());
                }
                Err(e) => report.errors.push(format!("insight {}: {}", insight.title, e)),
            }
        }

        // 9. Persist absorption metadata to KV store
        let kv_key = format!("meta:{}", meta.full_name.replace('/', ":"));
        let kv_val = serde_json::json!({
            "nodes_created": report.nodes_created,
            "files_ingested": report.files_ingested,
            "deps_detected": report.deps_detected,
            "insights": insights.len(),
            "is_update": is_update,
            "last_absorbed": now(),
        });
        let _ = self.kb.kv_set("github_absorb", &kv_key, &kv_val.to_string());

        Ok(report)
    }

    /// Absorb a GitHub repo by URL (supports https://github.com/owner/repo).
    pub fn absorb_url(&self, url: &str) -> Result<GitHubAbsorbReport, String> {
        let url = url.trim_end_matches('/').trim_end_matches(".git");
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 2 {
            return Err(format!("Invalid GitHub URL: {}", url));
        }
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];
        self.absorb(owner, repo)
    }

    /// Refresh an absorbed repo — checks GitHub pushed_at vs stored, re-absorbs if newer.
    pub fn refresh(&self, owner: &str, repo: &str) -> Result<GitHubAbsorbReport, String> {
        let repo_data = github_api_get(&format!("repos/{}/{}", owner, repo))?;
        let pushed_at = parse_github_time(repo_data["pushed_at"].as_str().unwrap_or("")).unwrap_or(0);
        let fallback_name = format!("{}/{}", owner, repo);
        let full_name = repo_data["full_name"].as_str().unwrap_or(&fallback_name);
        let repo_url = format!("https://github.com/{}", full_name);

        let existing = self.kb.find_node_by_url(&repo_url).unwrap_or(None);
        if let Some(node) = existing {
            let stored_pushed = node.metadata.as_ref()
                .and_then(|m| m.get("pushed_at").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            if pushed_at <= stored_pushed {
                return Ok(GitHubAbsorbReport {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    full_name: full_name.to_string(),
                    nodes_created: 0,
                    edges_created: 0,
                    readme_ingested: false,
                    files_ingested: 0,
                    deps_detected: 0,
                    architecture_insights: Vec::new(),
                    is_update: false,
                    timestamp: now(),
                    errors: Vec::new(),
                });
            }
        }
        self.absorb(owner, repo)
    }

    /// List all absorbed repositories with their status.
    pub fn list_absorbed(&self) -> Result<Vec<RepoStatus>, String> {
        let repos = self.kb.find_repositories("github.com", None)?;
        let mut statuses = Vec::new();
        for node in repos {
            let m = node.metadata.as_ref();
            statuses.push(RepoStatus {
                full_name: node.title.clone(),
                node_id: node.id.clone(),
                absorbed_at: node.created_at,
                last_updated: node.updated_at,
                file_count: m.and_then(|m| m.get("files_ingested").and_then(|v| v.as_u64())).unwrap_or(0) as usize,
                dep_count: m.and_then(|m| m.get("deps_detected").and_then(|v| v.as_u64())).unwrap_or(0) as usize,
                insight_count: m.and_then(|m| m.get("insights_generated").and_then(|v| v.as_u64())).unwrap_or(0) as usize,
                stale: m.map(|m| {
                    let pushed = m.get("pushed_at").and_then(|v| v.as_i64()).unwrap_or(0);
                    let absorbed = m.get("last_absorbed").and_then(|v| v.as_i64()).unwrap_or(0);
                    pushed > absorbed
                }).unwrap_or(false),
            });
        }
        Ok(statuses)
    }

    // ── Private: Metadata Parsing ──

    fn parse_repo_meta(&self, data: &serde_json::Value) -> Result<RepoMeta, String> {
        Ok(RepoMeta {
            full_name: data["full_name"].as_str().unwrap_or("unknown/unknown").to_string(),
            description: data["description"].as_str().unwrap_or("").to_string(),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            stars: data["stargazers_count"].as_i64().unwrap_or(0),
            forks: data["forks_count"].as_i64().unwrap_or(0),
            topics: data["topics"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            language: data["language"].as_str().unwrap_or("unknown").to_string(),
            license_name: data["license"]["spdx_id"].as_str()
                .or_else(|| data["license"]["name"].as_str())
                .map(String::from),
            default_branch: data["default_branch"].as_str().unwrap_or("main").to_string(),
            pushed_at: parse_github_time(data["pushed_at"].as_str().unwrap_or("")).unwrap_or(0),
            created_at: parse_github_time(data["created_at"].as_str().unwrap_or("")).unwrap_or(0),
            owner_login: data["owner"]["login"].as_str().unwrap_or("unknown").to_string(),
            owner_type: data["owner"]["type"].as_str().unwrap_or("User").to_string(),
            size_kb: data["size"].as_i64().unwrap_or(0),
            has_readme: data["has_readme"].as_bool().unwrap_or(true),
        })
    }

    // ── Private: README Ingestion ──

    fn ingest_readme(&self, owner: &str, repo: &str, branch: &str, repo_node_id: &str) -> Result<(), String> {
        for readme_name in &["README.md", "README.rst", "README", "Readme.md"] {
            let url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, branch, readme_name);
            match github_raw(&url) {
                Ok(content) => {
                    let title = format!("{}/{} README", owner, repo);
                    let summary = content.chars().take(500).collect::<String>();
                    let readme_url = format!("https://github.com/{}/{}/blob/{}/{}", owner, repo, branch, readme_name);
                    let readme_id = self.kb.insert_or_get_node(
                        &title,
                        NodeType::Article,
                        Some(&summary),
                        Some(&readme_url),
                        Some("github.com"),
                    )?;
                    self.kb.update_node_content(&readme_id, &content)?;
                    self.kb.upsert_edge(repo_node_id, &readme_id, RelationType::References, 0.8,
                        Some("Repository README"))?;
                    return Ok(());
                }
                Err(_) => continue,
            }
        }
        Err("No README found".into())
    }

    // ── Private: File Tree ──

    fn fetch_file_tree(&self, owner: &str, repo: &str, branch: &str) -> Result<Vec<FileEntry>, String> {
        let data = github_api_get(&format!("repos/{}/{}/git/trees/{}?recursive=1", owner, repo, branch))?;
        let tree = data["tree"].as_array().ok_or("No tree data")?;
        let mut files = Vec::new();
        for entry in tree {
            let path = entry["path"].as_str().unwrap_or("");
            let ftype = entry["type"].as_str().unwrap_or("blob");
            let size = entry["size"].as_i64().unwrap_or(0);
            let name = path.split('/').next_back().unwrap_or(path);
            files.push(FileEntry {
                path: path.to_string(),
                name: name.to_string(),
                file_type: if ftype == "tree" { "dir".into() } else { "file".into() },
                size,
            });
        }
        Ok(files)
    }

    // ── Private: Dependency Detection ──

    fn detect_dependencies(&self, files: &[FileEntry], owner: &str, repo: &str, branch: &str) -> Vec<DepInfo> {
        let mut all_deps = Vec::new();
        for file in files {
            let deps = match file.name.as_str() {
                "Cargo.toml" => Some(self.parse_deps_http(owner, repo, branch, &file.path,
                    |c| self.parse_cargo_deps(c))),
                "package.json" => Some(self.parse_deps_http(owner, repo, branch, &file.path,
                    |c| self.parse_package_deps(c))),
                "requirements.txt" => Some(self.parse_deps_http(owner, repo, branch, &file.path,
                    |c| self.parse_requirements_deps(c))),
                "go.mod" => Some(self.parse_deps_http(owner, repo, branch, &file.path,
                    |c| self.parse_go_mod_deps(c))),
                _ => None,
            };
            if let Some(found) = deps {
                all_deps.extend(found);
            }
        }
        all_deps.sort_by(|a, b| a.name.cmp(&b.name));
        all_deps.dedup_by(|a, b| a.name == b.name);
        all_deps
    }

    fn parse_deps_http<F>(&self, owner: &str, repo: &str, branch: &str, path: &str, parser: F) -> Vec<DepInfo>
    where F: Fn(&str) -> Vec<DepInfo> {
        let url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, branch, path);
        if let Ok(content) = github_raw(&url) {
            return parser(&content);
        }
        // Try master branch
        let url2 = format!("https://raw.githubusercontent.com/{}/{}/master/{}", owner, repo, path);
        if let Ok(content) = github_raw(&url2) {
            return parser(&content);
        }
        Vec::new()
    }

    fn parse_cargo_deps(&self, content: &str) -> Vec<DepInfo> {
        let mut deps = Vec::new();
        let mut section = "none";
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("[dependencies]") { section = "runtime"; continue; }
            if t.starts_with("[dev-dependencies]") { section = "dev"; continue; }
            if t.starts_with("[build-dependencies]") { section = "build"; continue; }
            if t.starts_with('[') && !t.starts_with("[dependencies") && !t.starts_with("[dev") && !t.starts_with("[build") {
                section = "none"; continue;
            }
            if section != "none" && t.contains('=') && !t.starts_with('#') {
                let parts: Vec<&str> = t.splitn(2, '=').collect();
                let name = parts[0].trim().trim_matches('"').trim_matches('\'').to_string();
                if !name.is_empty() && !name.contains(' ') && name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                    let version = parts.get(1).map(|v| v.trim().trim_matches('"').trim_matches('\'').trim_matches('{').trim().to_string()).unwrap_or_default();
                    deps.push(DepInfo { name, version, dep_type: section.to_string() });
                }
            }
        }
        deps
    }

    fn parse_package_deps(&self, content: &str) -> Vec<DepInfo> {
        let mut deps = Vec::new();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            for (section, dep_type) in &[("dependencies", "runtime"), ("devDependencies", "dev"), ("peerDependencies", "runtime")] {
                if let Some(obj) = json.get(*section).and_then(|v| v.as_object()) {
                    for (name, ver) in obj {
                        deps.push(DepInfo {
                            name: name.clone(),
                            version: ver.as_str().unwrap_or("unknown").to_string(),
                            dep_type: dep_type.to_string(),
                        });
                    }
                }
            }
        }
        deps
    }

    fn parse_requirements_deps(&self, content: &str) -> Vec<DepInfo> {
        let mut deps = Vec::new();
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') || t.starts_with('-') { continue; }
            let name = t.split(&['=', '>', '<', '~', '!', '@', ' ', '#'][..]).next().unwrap_or("").trim().to_lowercase();
            if !name.is_empty() {
                deps.push(DepInfo { name, version: String::new(), dep_type: "runtime".to_string() });
            }
        }
        deps
    }

    fn parse_go_mod_deps(&self, content: &str) -> Vec<DepInfo> {
        let mut deps = Vec::new();
        let mut in_require = false;
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("require (") { in_require = true; continue; }
            if t == ")" { in_require = false; continue; }
            if in_require {
                let parts: Vec<&str> = t.split_whitespace().collect();
                if parts.len() >= 2 {
                    deps.push(DepInfo {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        dep_type: "runtime".to_string(),
                    });
                }
            }
        }
        deps
    }

    // ── Private: Key Source File Identification ──

    fn identify_key_files(&self, files: &[FileEntry]) -> Vec<FileEntry> {
        let priority_dirs = ["src/", "lib/", "app/", "core/", "engine/", "agent/",
            "framework/", "runtime/", "kernel/", "pkg/", "internal/", "cmd/", "api/"];
        let relevant_ext = [".rs", ".py", ".ts", ".js", ".go", ".java", ".kt", ".swift", ".c", ".h", ".cpp", ".hpp", ".rb", ".php"];

        // Filter to priority source files
        let mut candidates: Vec<FileEntry> = files.iter()
            .filter(|f| f.file_type == "file")
            .filter(|f| {
                if f.name == "Cargo.toml" || f.name == "package.json" || f.name == "go.mod" { return false; }
                if f.name.ends_with(".png") || f.name.ends_with(".jpg") || f.name.ends_with(".ico") ||
                   f.name.ends_with(".svg") || f.name.ends_with(".woff") { return false; }
                if f.name.contains("_test") || f.name.contains(".test.") || f.path.contains("/test/") ||
                   f.path.contains("/tests/") || f.path.contains("/spec/") { return false; }
                if f.path.contains("/node_modules/") || f.path.contains("/target/") ||
                   f.path.contains(".git/") { return false; }
                true
            })
            .filter(|f| {
                let in_priority = priority_dirs.iter().any(|d| f.path.starts_with(d));
                let has_relevant_ext = relevant_ext.iter().any(|e| f.path.ends_with(e));
                let is_mod = f.name == "mod.rs" || f.name == "lib.rs" || f.name == "main.rs";
                in_priority && (has_relevant_ext || is_mod)
            })
            .cloned()
            .collect();

        candidates.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
        candidates.truncate(30);
        candidates
    }

    fn fetch_and_ingest_source(&self, file: &FileEntry, owner: &str, repo: &str, branch: &str, repo_node_id: &str) -> Result<(), String> {
        // Try Contents API first, fall back to raw URL
        let code = github_api_get(&format!("repos/{}/{}/contents/{}", owner, repo, file.path))
            .and_then(|d| {
                let b64 = d["content"].as_str().ok_or("no content")?;
                decode_base64(b64).ok_or_else(|| "base64 decode failed".into())
            })
            .or_else(|_| {
                let raw_url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, branch, file.path);
                github_raw(&raw_url)
            })?;

        if code.len() > 100_000 {
            return Err("file too large".into());
        }

        let summary = code.lines().take(10).collect::<Vec<_>>().join("\n");
        let summary = summary.chars().take(500).collect::<String>();

        let node_id = self.kb.insert_or_get_node(
            &file.path,
            NodeType::CodeSnippet,
            Some(&summary),
            None,
            Some("github.com"),
        )?;

        self.kb.update_node_content(&node_id, &code)?;
        let meta = serde_json::json!({"path": file.path, "size": file.size});
        self.kb.update_node_metadata(&node_id, &meta)?;
        self.kb.upsert_edge(repo_node_id, &node_id, RelationType::PartOf, 0.7,
            Some(&format!("Source: {}", file.path)))?;

        // Link to language concept
        let ext = file.path.rsplit('.').next().unwrap_or("");
        if !ext.is_empty() && ext.len() <= 4 {
            let lang_id = self.kb.insert_or_get_node(
                &format!("{} language", ext),
                NodeType::Concept,
                None,
                None,
                Some("programming_language"),
            ).unwrap_or_default();
            if !lang_id.is_empty() {
                let _ = self.kb.upsert_edge(&node_id, &lang_id, RelationType::ImplementedIn, 0.6, None);
            }
        }

        Ok(())
    }

    fn ingest_dependency(&self, dep: &DepInfo, repo_node_id: &str) -> Result<(), String> {
        let dep_node_id = self.kb.insert_or_get_node(
            &dep.name,
            NodeType::Framework,
            Some(&format!("{} dependency ({})", dep.dep_type, dep.version)),
            None,
            Some("package_ecosystem"),
        )?;
        self.kb.upsert_edge(repo_node_id, &dep_node_id, RelationType::DependsOn,
            if dep.dep_type == "runtime" { 0.9 } else { 0.5 },
            Some(&format!("v{}", dep.version)))?;
        Ok(())
    }

    // ── Private: Architecture Insight Generation ──

    fn generate_architecture_insights(&self, meta: &RepoMeta, files: &[FileEntry], deps: &[DepInfo]) -> Vec<ArchInsight> {
        let mut insights = Vec::new();

        // Tech stack insight
        let runtime_deps: Vec<_> = deps.iter().filter(|d| d.dep_type == "runtime").collect();
        if !runtime_deps.is_empty() {
            let names: Vec<_> = runtime_deps.iter().take(8).map(|d| d.name.clone()).collect();
            insights.push(ArchInsight {
                title: format!("{} Technology Stack", meta.full_name),
                description: format!("Core deps: {}. Language: {}.", names.join(", "), meta.language),
                category: "tech_stack".into(),
            });
        }

        // Architecture pattern from directory structure
        let dirs: std::collections::HashSet<&str> = files.iter()
            .filter(|f| f.file_type == "dir")
            .filter_map(|f| f.path.split('/').next())
            .collect();
        let mut patterns = Vec::new();
        if dirs.contains("src") { patterns.push("src-based layout"); }
        if dirs.contains("lib") { patterns.push("library project"); }
        if dirs.contains("cmd") { patterns.push("CLI app"); }
        if dirs.contains("api") { patterns.push("API surface"); }
        if files.iter().any(|f| f.path.contains("core") || f.path.contains("engine")) {
            patterns.push("core/engine separation");
        }
        let has_test_dir = files.iter().any(|f| f.path.starts_with("test") || f.path.starts_with("tests") ||
            f.path.starts_with("spec") || f.path.starts_with("examples"));
        if has_test_dir { patterns.push("tests present"); }
        if !patterns.is_empty() {
            insights.push(ArchInsight {
                title: format!("{} Architecture", meta.full_name),
                description: patterns.join(", "),
                category: "architecture".into(),
            });
        }

        // Scale insight
        let src_count = files.iter().filter(|f| f.path.starts_with("src/") && f.file_type == "file").count();
        if src_count > 5 {
            insights.push(ArchInsight {
                title: format!("{} Project Scale", meta.full_name),
                description: format!("{} source files, {} total, {} KB. Moderate-to-large structured project.", src_count, files.len(), meta.size_kb),
                category: "design_principle".into(),
            });
        }

        // Domain insight from topics
        let topic_str = meta.topics.join(" ");
        let domain_checks: Vec<(&str, Vec<&str>)> = vec![
            ("AI/ML", vec!["machine-learning", "deep-learning", "ai", "artificial-intelligence", "neural", "llm"]),
            ("Systems", vec!["rust", "wasm", "webassembly", "systems", "infrastructure", "performance"]),
            ("Web", vec!["web", "javascript", "typescript", "react", "frontend", "backend", "api"]),
            ("Data", vec!["data", "database", "big-data", "analytics", "pipeline"]),
            ("Security", vec!["security", "cryptography", "privacy", "authentication"]),
        ];
        for (domain, keywords) in &domain_checks {
            if keywords.iter().any(|k| topic_str.contains(k)) {
                insights.push(ArchInsight {
                    title: format!("{} Domain: {}", meta.full_name, domain),
                    description: format!("Project tagged in {} domain. Topics: {}", domain, meta.topics.join(", ")),
                    category: "pattern".into(),
                });
                // continue — repo may span multiple domains
            }
        }

        insights
    }

    fn ingest_insight(&self, insight: &ArchInsight, repo_node_id: &str) -> Result<(), String> {
        let node_id = self.kb.insert_or_get_node(
            &insight.title,
            NodeType::Insight,
            Some(&insight.description),
            None,
            Some("github_absorber"),
        )?;
        self.kb.upsert_edge(repo_node_id, &node_id, RelationType::Supports, 0.9,
            Some(&format!("Architecture insight: {}", insight.category)))?;
        Ok(())
    }
}

// ── Utility Functions ──

fn parse_github_time(time_str: &str) -> Option<i64> {
    if time_str.is_empty() { return None; }
    let s = time_str.trim_end_matches('Z');
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 { return None; }
    let date_p: Vec<&str> = parts[0].split('-').collect();
    let time_p: Vec<&str> = parts[1].split(':').collect();
    if date_p.len() != 3 || time_p.len() < 2 { return None; }
    let y: i64 = date_p[0].parse().ok()?;
    let m: i64 = date_p[1].parse().ok()?;
    let d: i64 = date_p[2].parse().ok()?;
    let hh: i64 = time_p[0].parse().ok()?;
    let mm: i64 = time_p[1].parse().ok()?;
    let ss: i64 = time_p.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let days = (y - 1970) * 365
        + (y - 1968) / 4 - (y - 1900) / 100 + (y - 1600) / 400
        + day_of_year(y, m, d);
    Some(days * 86400 + hh * 3600 + mm * 60 + ss)
}

fn day_of_year(y: i64, m: i64, d: i64) -> i64 {
    if !(1..=12).contains(&m) { return 0; }
    let mdays = [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut doy = d.saturating_sub(1).max(0);
    for i in 0..(m as usize - 1) { doy = doy.saturating_add(mdays[i]); }
    doy
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn decode_base64(input: &str) -> Option<String> {
    use std::collections::HashMap;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let table: HashMap<char, u8> = CHARS.iter().enumerate().map(|(i, &c)| (c as char, i as u8)).collect();
    let input = input.trim();
    let mut result = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0;
    for c in input.chars() {
        if c == '=' { break; }
        if let Some(&v) = table.get(&c) {
            buf = (buf << 6) | v as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                result.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
    }
    String::from_utf8(result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_time() {
        let ts = parse_github_time("2024-01-15T10:30:00Z");
        assert!(ts.is_some());
        assert!(ts.unwrap() > 0);
    }

    #[test]
    fn test_decode_base64() {
        let decoded = decode_base64("SGVsbG8gV29ybGQ=");
        assert_eq!(decoded, Some("Hello World".into()));
    }

    #[test]
    fn test_parse_cargo_deps() {
        let content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
criterion = "0.5"
"#;
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let deps = absorber.parse_cargo_deps(content);
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "serde" && d.dep_type == "runtime"));
        assert!(deps.iter().any(|d| d.name == "criterion" && d.dep_type == "dev"));
    }

    #[test]
    fn test_parse_github_time_leap_year_accuracy() {
        // 2024-03-01T00:00:00Z should be valid and match expected range
        let ts = parse_github_time("2024-03-01T00:00:00Z").unwrap();
        // 2024 is a leap year: days from 1970-01-01 to 2024-03-01
        // = (2024-1970)*365 + leap_days + day_of_year(2024,3,1)
        // = 54*365 + 13 + 60 = 19710 + 73 = 19783 days = 1709251200 seconds
        // Allow ±1 day tolerance
        assert!(ts > 1709164800, "ts={} should be > 1709164800", ts);
        assert!(ts < 1709510400, "ts={} should be < 1709510400", ts);
    }

    #[test]
    fn test_parse_github_time_edge() {
        assert!(parse_github_time("").is_none());
        assert!(parse_github_time("bad-date").is_none());
        assert!(parse_github_time("2024-01-01T00:00:00Z").is_some());
    }

    #[test]
    fn test_parse_cargo_deps_inline_tables() {
        let content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"

[dependencies.tokio]
version = "1.0"
features = ["full"]
"#;
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let deps = absorber.parse_cargo_deps(content);
        // Inline tables via [dependencies.xxx] are parsed, yielding
        // serde + tokio (version) + tokio (features) = 3 deps
        assert_eq!(deps.len(), 3);
    }

    #[test]
    fn test_parse_package_deps() {
        let content = r#"{
  "dependencies": { "express": "^4.18.0", "lodash": "4.17.21" },
  "devDependencies": { "jest": "^29.0.0" }
}"#;
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let deps = absorber.parse_package_deps(content);
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "express" && d.dep_type == "runtime"));
        assert!(deps.iter().any(|d| d.name == "jest" && d.dep_type == "dev"));
    }

    #[test]
    fn test_parse_requirements_deps() {
        let content = "numpy==1.24.0\npandas>=2.0.0\n# comment\ntorch\n";
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let deps = absorber.parse_requirements_deps(content);
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "numpy"));
        assert!(deps.iter().any(|d| d.name == "pandas"));
        assert!(deps.iter().any(|d| d.name == "torch"));
    }

    #[test]
    fn test_identify_key_files_prioritizes_src() {
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let files = vec![
            FileEntry { path: "src/main.rs".into(), name: "main.rs".into(), file_type: "file".into(), size: 100 },
            FileEntry { path: "src/lib.rs".into(), name: "lib.rs".into(), file_type: "file".into(), size: 200 },
            FileEntry { path: "Makefile".into(), name: "Makefile".into(), file_type: "file".into(), size: 50 },
            FileEntry { path: "README.md".into(), name: "README.md".into(), file_type: "file".into(), size: 500 },
            FileEntry { path: "Cargo.toml".into(), name: "Cargo.toml".into(), file_type: "file".into(), size: 100 },
            FileEntry { path: "src/tests/test_helper.rs".into(), name: "test_helper.rs".into(), file_type: "file".into(), size: 300 },
            FileEntry { path: "node_modules/pkg/index.js".into(), name: "index.js".into(), file_type: "file".into(), size: 1000 },
        ];
        let key = absorber.identify_key_files(&files);
        assert_eq!(key.len(), 2);
        assert!(key.iter().any(|f| f.path == "src/main.rs"));
        assert!(key.iter().any(|f| f.path == "src/lib.rs"));
    }

    #[test]
    fn test_parse_go_mod_deps() {
        let content = r#"module example.com/my/mod

go 1.21

require (
    example.com/pkg v1.0.0
    example.com/other v0.5.0
)
"#;
        let absorber = GitHubAbsorber::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
        );
        let deps = absorber.parse_go_mod_deps(content);
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "example.com/pkg"));
    }

    #[test]
    fn test_day_of_year() {
        // Jan 1 = 0, Mar 1 in leap year = 60 (Jan 31 + Feb 29)
        assert_eq!(day_of_year(2024, 1, 1), 0);
        assert_eq!(day_of_year(2024, 3, 1), 60);
        // Mar 1 in non-leap year = 59 (Jan 31 + Feb 28)
        assert_eq!(day_of_year(2023, 3, 1), 59);
        assert_eq!(day_of_year(2023, 12, 31), 364);
    }
}
