//! Wiki KB 入口（migrated from cli::commands::wiki_cmds）。
//!
//! 架构：先经 AutoOrchestrator 意图分类（可观测），再分发子命令到
//! `l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase` 后端。
//! 注意：`generate` 不再覆盖 AGENTS.md（指针守恒守卫禁止），
//! 输出到 `docs/architecture/CODEBASE-WIKI.md`。

use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

use neotrix::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
use neotrix::l6_meta::nt_auto_orchestrator::AutoOrchestrator;

/// 摘要预览阈值：超过 `PREVIEW_MAX` 个**字符**则取前 `PREVIEW_KEEP` 个加 `...`。
///
/// ⚠️ 原先是「按**字节**判断 + 按字节切」——`r.summary.len() > 80` 配
/// `&r.summary[..77]`。
///
/// ⛔ **真实 panic**：`WikiSearchResult.summary` 由 `nt_memory_wiki::extract_summary`
/// 产出（`chars().take(200)`，即最多 200 **字符** / 600 字节），而 `wiki sync` 的
/// 内容源是 `docs/**/*.md` —— 本仓架构文档正文几乎全是中文。一汉字 3 字节，
/// 27 个汉字即 81 字节 > 80 ⇒ `&r.summary[..77]` 必落在字符中间，
/// 一次 `wiki query <中文词>` 就 panic。
const PREVIEW_MAX: usize = 80;
const PREVIEW_KEEP: usize = 77;

/// 按**字符**截断到 `max` 个字符（不足 `max` 则原样返回）。
///
/// ⛔ 不能写 `&s[..max]`：`max` 是**字节**偏移，`&str` 按非边界字节切会 panic
/// （`byte index … is not a char boundary`），且 `&str[..]` 本身触发
/// `clippy::string_slice`（workspace warn 级 + CI `-D warnings`）。
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        s.chars().take(max).collect()
    } else {
        s.to_string()
    }
}

/// `wiki` 入口：意图分类 → 子命令分发。
pub fn run_wiki(args: &[String]) -> Result<(), String> {
    let orchestrator = AutoOrchestrator::new();
    let classification = orchestrator.classify_intent(&format!("wiki {}", args.join(" ")));
    println!(
        "🔍 意图识别: {:?} (conf={:.2})",
        classification.task_type, classification.confidence
    );
    let msg = if args.is_empty() {
        help_text().to_string()
    } else {
        match args[0].as_str() {
            "generate" => cmd_generate()?,
            "status" => cmd_status()?,
            "sync" => cmd_sync(args)?,
            "graph" => cmd_graph(args)?,
            "query" => cmd_query(args)?,
            other => return Err(format!("Unknown subcommand: {other}")),
        }
    };
    println!("{msg}");
    Ok(())
}

fn help_text() -> &'static str {
    "Wiki commands:\n  wiki generate       codebase wiki (→ docs/architecture/CODEBASE-WIKI.md)\n  wiki status         AGENTS.md status\n  wiki sync [dir]     sync docs/ -> KB wiki\n  wiki graph [file]   generate wiki graph HTML\n  wiki query <text>   search KB wiki pages"
}

fn cmd_sync(args: &[String]) -> Result<String, String> {
    let dir = args.get(1).map(|s| s.as_str()).unwrap_or("docs");
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Err(format!("Directory not found: {dir}"));
    }
    let kb = KnowledgeBase::open(None).map_err(|e| format!("Cannot open KB: {e}"))?;
    match kb.wiki_sync(dir_path, dir) {
        Ok(report) => {
            let mut msg = format!(
                "Wiki sync: {} pages, {} edges",
                report.synced, report.edges_created
            );
            if !report.errors.is_empty() {
                msg.push_str(&format!(" ({} errors)", report.errors.len()));
                for (stem, err) in report.errors.iter().take(5) {
                    msg.push_str(&format!("\n  {stem}: {err}"));
                }
            }
            Ok(msg)
        }
        Err(e) => Err(format!("Wiki sync failed: {e}")),
    }
}

fn cmd_graph(args: &[String]) -> Result<String, String> {
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("wiki-graph.html");
    let kb = KnowledgeBase::open(None).map_err(|e| format!("Cannot open KB: {e}"))?;
    match kb.wiki_graph_html() {
        Ok(html) => match std::fs::write(output, &html) {
            Ok(()) => Ok(format!("Wiki graph written to {output}")),
            Err(e) => Err(format!("Failed to write {output}: {e}")),
        },
        Err(e) => Err(format!("Wiki graph failed: {e}")),
    }
}

fn cmd_query(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err("Usage: wiki query <text>".to_string());
    }
    let query = args[1..].join(" ");
    let kb = KnowledgeBase::open(None).map_err(|e| format!("Cannot open KB: {e}"))?;
    match kb.wiki_query(&query, 20) {
        Ok(results) => {
            if results.is_empty() {
                return Ok("No wiki pages found.".to_string());
            }
            let mut msg = format!("Wiki results for \"{query}\":\n");
            for (i, r) in results.iter().enumerate() {
                let s = if r.summary.chars().count() > PREVIEW_MAX {
                    format!("{}...", truncate_chars(&r.summary, PREVIEW_KEEP))
                } else {
                    r.summary.clone()
                };
                msg.push_str(&format!(
                    "  {}. {} (score: {:.2})\n     {}\n",
                    i + 1,
                    r.title,
                    r.score,
                    s
                ));
            }
            Ok(msg)
        }
        Err(e) => Err(format!("Wiki query failed: {e}")),
    }
}

// ---- Wiki generate / status ----

fn cmd_generate() -> Result<String, String> {
    let root = Path::new(".");
    let project_name = detect_project_name(root);
    let src_dir = root.join("neotrix-core").join("src");
    let stats = collect_stats(&src_dir).map_err(|e| format!("Failed to scan source tree: {e}"))?;
    let mut md = String::new();
    md.push_str(&format!("# {project_name} — Wiki (Auto-generated)\n\n"));
    md.push_str(&format!(
        "> Auto-generated by `wiki generate` on {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str("## Codebase Overview\n\n| Metric | Value |\n|--------|-------|\n");
    md.push_str(&format!("| Total `.rs` files | {} |\n", stats.file_count));
    md.push_str(&format!(
        "| Total lines of code | {} |\n",
        stats.total_lines
    ));
    md.push_str(&format!(
        "| Modules (subdirectories with mod.rs) | {} |\n",
        stats.module_count
    ));
    md.push_str(&format!("| Source root | `{}` |\n", src_dir.display()));
    md.push('\n');
    md.push_str("## Layer Architecture\n\n| Layer | Directory | Files | Lines |\n|-------|-----------|-------|-------|\n");
    for (layer, info) in &stats.layers {
        md.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            layer, info.path, info.files, info.lines
        ));
    }
    md.push('\n');
    md.push_str("## Project Structure\n\n```\n");
    md.push_str(&build_tree(root, 0, 3));
    md.push_str("```\n\n");
    md.push_str("## Build & Test\n\n```bash\ncargo build -p neotrix\ncargo test -p neotrix --lib\ncargo check --lib -p neotrix\n```\n\n");
    md.push_str("## Architecture Domains\n\n| Domain | Prefix | Files | Lines |\n|--------|--------|-------|-------|\n");
    for (domain, info) in &stats.domains {
        md.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            domain, info.path, info.files, info.lines
        ));
    }
    md.push('\n');
    if !stats.key_types.is_empty() {
        md.push_str(
            "## Key Types & Traits\n\n| Type/Trait | File | Kind |\n|------------|------|------|\n",
        );
        for kt in &stats.key_types {
            md.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                kt.name, kt.file, kt.kind
            ));
        }
        md.push('\n');
    }
    // NOTE: AGENTS.md 受指针守恒守卫保护，不可覆盖；生成物写入架构目录。
    let out = root
        .join("docs")
        .join("architecture")
        .join("CODEBASE-WIKI.md");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
    }
    match std::fs::write(&out, &md) {
        Ok(()) => Ok(format!(
            "Wiki written to {}: {} bytes, {} files, {} lines (AGENTS.md untouched — guard protected)",
            out.display(),
            md.len(),
            stats.file_count,
            stats.total_lines
        )),
        Err(e) => Err(format!("Failed to write {}: {e}", out.display())),
    }
}

fn cmd_status() -> Result<String, String> {
    let path = Path::new("AGENTS.md");
    if !path.exists() {
        return Err("AGENTS.md does not exist".to_string());
    }
    let meta = path
        .metadata()
        .map_err(|e| format!("Cannot read metadata: {e}"))?;
    let size = meta.len();
    let age_str = meta
        .modified()
        .ok()
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .map(|d| {
            let secs = d.as_secs();
            if secs < 60 {
                format!("{secs}s ago")
            } else if secs < 3600 {
                format!("{}m {}s ago", secs / 60, secs % 60)
            } else if secs < 86400 {
                format!("{}h {}m ago", secs / 3600, (secs % 3600) / 60)
            } else {
                format!("{}d {}h ago", secs / 86400, (secs % 86400) / 3600)
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    Ok(format!("AGENTS.md: {size} bytes, modified {age_str}"))
}

// ---- Stats collection ----

#[derive(Default)]
struct CodebaseStats {
    file_count: usize,
    total_lines: usize,
    module_count: usize,
    layers: BTreeMap<String, LayerInfo>,
    domains: BTreeMap<String, DomainInfo>,
    key_types: Vec<TypeInfo>,
}
struct LayerInfo {
    path: String,
    files: usize,
    lines: usize,
}
struct DomainInfo {
    path: String,
    files: usize,
    lines: usize,
}
struct TypeInfo {
    name: String,
    file: String,
    kind: String,
}

fn detect_project_name(root: &Path) -> String {
    let cargo = root.join("Cargo.toml");
    if cargo.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(name) = trimmed.strip_prefix("name = \"") {
                    if let Some(end) = name.find('"') {
                        return name[..end].to_string();
                    }
                }
            }
        }
    }
    "NeoTrix".to_string()
}

fn collect_stats(src_dir: &Path) -> Result<CodebaseStats, String> {
    let mut stats = CodebaseStats::default();
    if !src_dir.exists() {
        return Err(format!("Source dir not found: {}", src_dir.display()));
    }
    let mut dirs_to_scan: Vec<std::path::PathBuf> = vec![src_dir.to_path_buf()];
    let mut seen_dirs = std::collections::HashSet::new();
    let mut key_types_seen = std::collections::HashSet::new();
    while let Some(dir) = dirs_to_scan.pop() {
        if !seen_dirs.insert(dir.clone()) {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs_to_scan.push(path.clone());
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let relative = path
                    .strip_prefix(src_dir.parent().unwrap_or(Path::new(".")))
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let line_count = content.lines().count();
                stats.file_count += 1;
                stats.total_lines += line_count;
                let path_str = path.display().to_string();
                for (layer_key, layer_prefix) in &[
                    ("L0 Substrate", "l0_substrate"),
                    ("L1 Action", "l1_action"),
                    ("L2 Perception", "l2_perception"),
                    ("L3 Embodiment", "l3_embodiment"),
                    ("L4 Emotion", "l4_emotion"),
                    ("L5 Cognition", "l5_cognition"),
                    ("L6 Meta", "l6_meta"),
                    ("Entry", "entry"),
                    ("Agent", "agent"),
                    ("Server", "server"),
                ] {
                    if path_str.contains(layer_prefix) {
                        let entry =
                            stats
                                .layers
                                .entry(layer_key.to_string())
                                .or_insert_with(|| LayerInfo {
                                    path: format!("src/{layer_prefix}"),
                                    files: 0,
                                    lines: 0,
                                });
                        entry.files += 1;
                        entry.lines += line_count;
                        break;
                    }
                }
                for component in path.components() {
                    if let std::path::Component::Normal(name) = component {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("nt_") && name_str.contains('_') {
                            let entry =
                                stats
                                    .domains
                                    .entry(name_str.to_string())
                                    .or_insert_with(|| DomainInfo {
                                        path: relative.clone(),
                                        files: 0,
                                        lines: 0,
                                    });
                            entry.files += 1;
                            entry.lines += line_count;
                            break;
                        }
                    }
                }
                for line in content.lines() {
                    let trimmed = line.trim();
                    for prefix in [
                        "pub struct ",
                        "struct ",
                        "pub trait ",
                        "trait ",
                        "pub enum ",
                        "enum ",
                    ] {
                        if let Some(name) = trimmed.strip_prefix(prefix) {
                            if let Some(end) = name.find(|c: char| !c.is_alphanumeric() && c != '_')
                            {
                                let type_name = name[..end].to_string();
                                if key_types_seen.insert(type_name.clone()) {
                                    let kind = if prefix.contains("struct") {
                                        "struct"
                                    } else if prefix.contains("trait") {
                                        "trait"
                                    } else {
                                        "enum"
                                    };
                                    stats.key_types.push(TypeInfo {
                                        name: type_name,
                                        file: relative.clone(),
                                        kind: kind.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut mod_dirs = std::collections::HashSet::new();
    let mut scan_dirs: Vec<std::path::PathBuf> = vec![src_dir.to_path_buf()];
    while let Some(dir) = scan_dirs.pop() {
        if dir.join("mod.rs").exists() {
            mod_dirs.insert(dir.clone());
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dirs.push(path);
                }
            }
        }
    }
    stats.module_count = mod_dirs.len();
    stats.key_types.sort_by(|a, b| a.name.cmp(&b.name));
    stats.key_types.truncate(50);
    Ok(stats)
}

fn build_tree(root: &Path, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return String::new();
    }
    let mut out = String::new();
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return out,
    };
    let mut items: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            !name_str.starts_with('.')
                && !name_str.starts_with("target")
                && !name_str.starts_with("node_modules")
                && name_str != "Cargo.lock"
        })
        .collect();
    items.sort_by_key(|e| {
        (
            !e.file_type().map(|t| t.is_dir()).unwrap_or(false),
            e.file_name().to_string_lossy().to_string(),
        )
    });
    for item in &items {
        let name = item.file_name().to_string_lossy().to_string();
        let indent = "  ".repeat(depth);
        if item.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            out.push_str(&format!("{indent}{name}/\n"));
            out.push_str(&build_tree(&item.path(), depth + 1, max_depth));
        } else if name.ends_with(".rs") || name == "Cargo.toml" || name.ends_with(".md") {
            out.push_str(&format!("{indent}{name}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{truncate_chars, PREVIEW_KEEP, PREVIEW_MAX};

    /// 复刻 `cmd_query` 的摘要展示表达式（不含 DB 查询），供测试直接驱动。
    fn preview(s: &str) -> String {
        if s.chars().count() > PREVIEW_MAX {
            format!("{}...", truncate_chars(s, PREVIEW_KEEP))
        } else {
            s.to_string()
        }
    }

    fn cjk(n: usize) -> String {
        "模块边界".chars().cycle().take(n).collect()
    }

    #[test]
    fn preview_cjk_over_threshold_does_not_panic() {
        // 200 汉字 = 600 字节（`extract_summary` 的上限形态）：
        // 旧代码 `len() > 80` 命中后 `&s[..77]` 必落在字符中间 ⇒ panic。
        let s = cjk(200);
        let out = preview(&s);
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), PREVIEW_KEEP + 3);
        assert_eq!(out.len(), PREVIEW_KEEP * 3 + 3, "77 汉字 == 231 字节");
    }

    #[test]
    fn preview_cjk_just_over_byte_threshold_passes_through() {
        // 30 汉字 = 90 字节：字节阈值 80 会被误触发，但字符数 30 < 80 ⇒ 原样输出。
        // 旧代码在此 panic（`&s[..77]` on 90-byte string）。
        let s = cjk(30);
        assert!(s.len() > 80, "byte-guard would have fired");
        assert_eq!(preview(&s), s);
    }

    #[test]
    fn preview_4byte_emoji_boundary() {
        let s: String = std::iter::repeat('🙂').take(200).collect();
        let out = preview(&s);
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), PREVIEW_KEEP + 3);
        assert_eq!(out.len(), PREVIEW_KEEP * 4 + 3);
    }

    #[test]
    fn preview_exact_threshold_is_not_truncated() {
        let s = cjk(PREVIEW_MAX); // 80 字符 / 240 字节
        assert_eq!(preview(&s), s);
        assert_eq!(truncate_chars(&s, PREVIEW_KEEP), cjk(PREVIEW_KEEP));
    }

    #[test]
    fn preview_ascii_keeps_original_byte_semantics() {
        let s = "a".repeat(120);
        assert_eq!(preview(&s), format!("{}...", "a".repeat(PREVIEW_KEEP)));
        assert_eq!(truncate_chars(&s, PREVIEW_KEEP).len(), PREVIEW_KEEP);
    }

    #[test]
    fn truncate_chars_empty_and_short_inputs() {
        assert_eq!(truncate_chars("", PREVIEW_KEEP), "");
        assert_eq!(truncate_chars("山海", PREVIEW_KEEP), "山海");
    }
}
