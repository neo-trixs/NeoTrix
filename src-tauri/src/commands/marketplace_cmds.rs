use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f64,
    pub rating_count: u32,
    pub is_installed: bool,
    pub has_update: bool,
    pub installed_version: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub size_kb: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchResult {
    pub total: u32,
    pub results: Vec<MarketplacePlugin>,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceReview {
    pub id: String,
    pub plugin_id: String,
    pub author: String,
    pub rating: u8,
    pub title: String,
    pub body: String,
    pub created_at: u64,
    pub helpful_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub enabled: bool,
    pub auto_check_updates: bool,
    pub update_channel: String,
    pub curated_only: bool,
    pub auto_install_security: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_check_updates: true,
            update_channel: "stable".into(),
            curated_only: true,
            auto_install_security: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_plugins: u32,
    pub total_downloads: u64,
    pub total_installed: usize,
    pub updates_available: usize,
    pub categories: usize,
}

struct MarketplaceState {
    plugins: HashMap<String, MarketplacePlugin>,
    installed: HashSet<String>,
    reviews: HashMap<String, Vec<MarketplaceReview>>,
    config: MarketplaceConfig,
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn build_catalog() -> HashMap<String, MarketplacePlugin> {
    let base = now_ts();
    let mut m = HashMap::new();
    let entries: Vec<MarketplacePlugin> = vec![
        // Security
        MarketplacePlugin { id:"code-scanner".into(), name:"Code Scanner".into(), version:"2.4.1".into(), author:"NeoTrix Security".into(), description:"Static analysis for vulnerabilities, secrets, and code quality issues in real-time.".into(), category:"security".into(), tags:vec!["sast".into(),"vulnerability".into(),"static-analysis".into()], downloads:45230, rating:4.7, rating_count:312, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/code-scanner".into()), repository:Some("https://github.com/neotrix/code-scanner".into()), license:Some("MIT".into()), size_kb:1842, created_at:base-86400*120, updated_at:base-86400*3 },
        MarketplacePlugin { id:"secrets-detector".into(), name:"Secrets Detector".into(), version:"1.8.3".into(), author:"NeoTrix Security".into(), description:"Detect leaked API keys, tokens, and credentials before commit.".into(), category:"security".into(), tags:vec!["secrets".into(),"credentials".into(),"pre-commit".into()], downloads:38910, rating:4.8, rating_count:245, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:Some("https://github.com/neotrix/secrets-detector".into()), license:Some("Apache-2.0".into()), size_kb:956, created_at:base-86400*180, updated_at:base-86400*15 },
        MarketplacePlugin { id:"dep-checker".into(), name:"Dependency Checker".into(), version:"3.0.0".into(), author:"NeoTrix Ops".into(), description:"Audit dependencies for known vulnerabilities, license compliance, and outdated packages.".into(), category:"security".into(), tags:vec!["dependencies".into(),"audit".into(),"license".into()], downloads:28450, rating:4.5, rating_count:178, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:Some("MIT".into()), size_kb:2104, created_at:base-86400*90, updated_at:base-86400*7 },
        MarketplacePlugin { id:"sbom-generator".into(), name:"SBOM Generator".into(), version:"1.2.0".into(), author:"NeoTrix Ops".into(), description:"Generate SPDX/CycloneDX SBOM for your projects with one click.".into(), category:"security".into(), tags:vec!["sbom".into(),"spdx".into(),"cyclonedx".into()], downloads:12450, rating:4.2, rating_count:89, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:None, size_kb:672, created_at:base-86400*60, updated_at:base-86400*30 },
        // Deployment
        MarketplacePlugin { id:"docker-deploy".into(), name:"Docker Deploy".into(), version:"2.1.0".into(), author:"NeoTrix Cloud".into(), description:"One-click Docker image build, tag, and push to any registry.".into(), category:"deployment".into(), tags:vec!["docker".into(),"container".into(),"registry".into()], downloads:50320, rating:4.6, rating_count:401, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/docker-deploy".into()), repository:Some("https://github.com/neotrix/docker-deploy".into()), license:Some("MIT".into()), size_kb:1532, created_at:base-86400*200, updated_at:base-86400*5 },
        MarketplacePlugin { id:"k8s-manager".into(), name:"K8s Manager".into(), version:"1.6.2".into(), author:"NeoTrix Cloud".into(), description:"Manage Kubernetes clusters, deployments, and pods from your editor.".into(), category:"deployment".into(), tags:vec!["kubernetes".into(),"k8s".into(),"cluster".into()], downloads:32100, rating:4.4, rating_count:215, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:Some("https://github.com/neotrix/k8s-manager".into()), license:Some("Apache-2.0".into()), size_kb:2890, created_at:base-86400*150, updated_at:base-86400*12 },
        MarketplacePlugin { id:"vercel-push".into(), name:"Vercel Push".into(), version:"1.0.3".into(), author:"NeoTrix Cloud".into(), description:"Deploy to Vercel directly from your project — preview, promote, rollback.".into(), category:"deployment".into(), tags:vec!["vercel".into(),"deploy".into(),"preview".into()], downloads:18760, rating:4.1, rating_count:132, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:None, size_kb:445, created_at:base-86400*45, updated_at:base-86400*20 },
        MarketplacePlugin { id:"cloudflare-publish".into(), name:"Cloudflare Publish".into(), version:"2.0.1".into(), author:"NeoTrix Cloud".into(), description:"Publish to Cloudflare Pages, Workers, and R2 with built-in secret management.".into(), category:"deployment".into(), tags:vec!["cloudflare".into(),"workers".into(),"pages".into()], downloads:15670, rating:4.3, rating_count:98, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:Some("MIT".into()), size_kb:712, created_at:base-86400*80, updated_at:base-86400*10 },
        // Testing
        MarketplacePlugin { id:"test-runner".into(), name:"Test Runner Pro".into(), version:"3.2.0".into(), author:"NeoTrix QA".into(), description:"Parallel test execution with watch mode, code coverage, and flaky test detection.".into(), category:"testing".into(), tags:vec!["testing".into(),"coverage".into(),"parallel".into()], downloads:42100, rating:4.9, rating_count:367, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/test-runner".into()), repository:Some("https://github.com/neotrix/test-runner".into()), license:Some("MIT".into()), size_kb:2240, created_at:base-86400*250, updated_at:base-86400*2 },
        MarketplacePlugin { id:"coverage-viz".into(), name:"Coverage Viz".into(), version:"1.5.0".into(), author:"NeoTrix QA".into(), description:"Interactive code coverage visualization with diff-aware reporting.".into(), category:"testing".into(), tags:vec!["coverage".into(),"visualization".into(),"report".into()], downloads:23500, rating:4.5, rating_count:189, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:Some("Apache-2.0".into()), size_kb:1230, created_at:base-86400*100, updated_at:base-86400*25 },
        MarketplacePlugin { id:"perf-benchmark".into(), name:"Perf Benchmark".into(), version:"2.0.0".into(), author:"NeoTrix QA".into(), description:"Benchmark your code with customizable scenarios and historical trend charts.".into(), category:"testing".into(), tags:vec!["performance".into(),"benchmark".into(),"profiling".into()], downloads:18900, rating:4.3, rating_count:145, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:None, size_kb:1678, created_at:base-86400*70, updated_at:base-86400*18 },
        MarketplacePlugin { id:"e2e-automation".into(), name:"E2E Automation".into(), version:"1.9.4".into(), author:"NeoTrix QA".into(), description:"Record, edit, and run end-to-end browser tests with visual diffing.".into(), category:"testing".into(), tags:vec!["e2e".into(),"browser".into(),"visual".into()], downloads:31200, rating:4.6, rating_count:278, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:Some("https://github.com/neotrix/e2e-automation".into()), license:Some("MIT".into()), size_kb:3450, created_at:base-86400*130, updated_at:base-86400*8 },
        // Frontend
        MarketplacePlugin { id:"react-inspector".into(), name:"React Inspector".into(), version:"2.3.1".into(), author:"NeoTrix Frontend".into(), description:"Inspect React component tree, props, state, and hooks in real-time.".into(), category:"frontend".into(), tags:vec!["react".into(),"debug".into(),"components".into()], downloads:47100, rating:4.8, rating_count:423, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/react-inspector".into()), repository:Some("https://github.com/neotrix/react-inspector".into()), license:Some("MIT".into()), size_kb:1890, created_at:base-86400*300, updated_at:base-86400*1 },
        MarketplacePlugin { id:"tailwind-helper".into(), name:"Tailwind Helper".into(), version:"1.4.2".into(), author:"NeoTrix Frontend".into(), description:"Visual Tailwind CSS class explorer, color picker, and responsive preview.".into(), category:"frontend".into(), tags:vec!["tailwind".into(),"css".into(),"design".into()], downloads:38200, rating:4.7, rating_count:312, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:Some("https://github.com/neotrix/tailwind-helper".into()), license:None, size_kb:1234, created_at:base-86400*160, updated_at:base-86400*14 },
        MarketplacePlugin { id:"component-gen".into(), name:"Component Gen".into(), version:"1.1.0".into(), author:"NeoTrix Frontend".into(), description:"AI-powered component generation from screenshots, prompts, or existing code.".into(), category:"frontend".into(), tags:vec!["generation".into(),"ai".into(),"components".into()], downloads:21400, rating:4.4, rating_count:167, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:Some("Apache-2.0".into()), size_kb:2890, created_at:base-86400*50, updated_at:base-86400*22 },
        MarketplacePlugin { id:"a11y-checker".into(), name:"A11y Checker".into(), version:"1.0.0".into(), author:"NeoTrix Frontend".into(), description:"Automated accessibility audit with WCAG 2.2 AA compliance suggestions.".into(), category:"frontend".into(), tags:vec!["accessibility".into(),"a11y".into(),"wcag".into()], downloads:12780, rating:4.6, rating_count:94, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:None, size_kb:890, created_at:base-86400*30, updated_at:base-86400*6 },
        // AI
        MarketplacePlugin { id:"prompt-optimizer".into(), name:"Prompt Optimizer".into(), version:"2.0.0".into(), author:"NeoTrix AI".into(), description:"Optimize LLM prompts with automatic testing, version comparison, and cost analysis.".into(), category:"ai".into(), tags:vec!["prompts".into(),"llm".into(),"optimization".into()], downloads:34100, rating:4.5, rating_count:256, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/prompt-optimizer".into()), repository:Some("https://github.com/neotrix/prompt-optimizer".into()), license:Some("MIT".into()), size_kb:1456, created_at:base-86400*110, updated_at:base-86400*9 },
        MarketplacePlugin { id:"model-benchmark".into(), name:"Model Benchmark".into(), version:"1.3.0".into(), author:"NeoTrix AI".into(), description:"Compare LLM models side-by-side on latency, quality, and cost per token.".into(), category:"ai".into(), tags:vec!["models".into(),"benchmark".into(),"llm".into()], downloads:19200, rating:4.2, rating_count:134, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:None, size_kb:980, created_at:base-86400*85, updated_at:base-86400*28 },
        MarketplacePlugin { id:"context-analyzer".into(), name:"Context Analyzer".into(), version:"1.0.2".into(), author:"NeoTrix AI".into(), description:"Analyze token usage, context window utilization, and suggest chunking strategies.".into(), category:"ai".into(), tags:vec!["tokens".into(),"context".into(),"chunking".into()], downloads:15300, rating:4.1, rating_count:112, is_installed:false, has_update:false, installed_version:None, homepage:None, repository:None, license:Some("MIT".into()), size_kb:723, created_at:base-86400*40, updated_at:base-86400*17 },
        MarketplacePlugin { id:"code-review-ai".into(), name:"Code Review AI".into(), version:"3.1.0".into(), author:"NeoTrix AI".into(), description:"Automated code review with architectural analysis, security audit, and best practices.".into(), category:"ai".into(), tags:vec!["code-review".into(),"ai".into(),"architecture".into()], downloads:49800, rating:4.9, rating_count:445, is_installed:false, has_update:false, installed_version:None, homepage:Some("https://neotrix.ai/plugins/code-review-ai".into()), repository:Some("https://github.com/neotrix/code-review-ai".into()), license:Some("MIT".into()), size_kb:2150, created_at:base-86400*190, updated_at:base-86400*4 },
    ];
    for p in entries {
        m.insert(p.id.clone(), p);
    }
    m
}

fn build_reviews(plugin_ids: &[String]) -> HashMap<String, Vec<MarketplaceReview>> {
    let mut r: HashMap<String, Vec<MarketplaceReview>> = HashMap::new();
    let base = now_ts();
    let authors = ["alice", "bob", "carol", "dave", "eve"];
    let titles = ["Great plugin!", "Works well", "Needs improvement", "Excellent tool", "Good but slow", "Highly recommended", "Solid plugin", "Could be better"];
    let bodies = [
        "I've been using this for a few weeks and it really improves my workflow. Highly recommend to anyone in the same space.",
        "Does exactly what it says. The integration with NeoTrix is seamless. Would give 5 stars if documentation was better.",
        "Useful but has some rough edges. The latest update fixed the major issues I had. Will update my review after more testing.",
        "One of the best plugins in the marketplace. The team behind it is very responsive to issues on GitHub.",
        "Solid performance and regular updates. The configuration options are extensive and well thought out.",
        "Saves me hours every week. The initial setup took a bit but once configured it's smooth sailing.",
        "Good plugin overall. The core functionality works perfectly. Some additional features would be nice.",
        "Decent tool but there are better alternatives. The UI could use some modernizing.",
    ];
    for pid in plugin_ids {
        let n = 3 + (pid.len() % 3) as usize;
        let mut reviews: Vec<MarketplaceReview> = Vec::new();
        for i in 0..n {
            let ts = base - 86400 * (i as u64 * 7 + 5);
            reviews.push(MarketplaceReview {
                id: format!("rev-{}-{}", pid, i),
                plugin_id: pid.clone(),
                author: authors[i % authors.len()].into(),
                rating: (4 + i % 2) as u8,
                title: titles[i % titles.len()].into(),
                body: bodies[i % bodies.len()].into(),
                created_at: ts,
                helpful_count: (10 - i as u32 * 2),
            });
        }
        r.insert(pid.clone(), reviews);
    }
    r
}

static MARKETPLACE: LazyLock<Mutex<MarketplaceState>> = LazyLock::new(|| {
    let catalog = build_catalog();
    let pids: Vec<String> = catalog.keys().cloned().collect();
    Mutex::new(MarketplaceState {
        plugins: catalog,
        installed: HashSet::new(),
        reviews: build_reviews(&pids),
        config: MarketplaceConfig::default(),
    })
});

fn paginate(plugins: Vec<MarketplacePlugin>, page: u32) -> MarketplaceSearchResult {
    let total = plugins.len() as u32;
    let per_page: u32 = 12;
    let total_pages = total.div_ceil(per_page);
    let page = page.min(total_pages.max(1));
    let start = ((page - 1) * per_page) as usize;
    let results = plugins.into_iter().skip(start).take(per_page as usize).collect();
    MarketplaceSearchResult { total, results, page, total_pages }
}

fn sync_installed(plugin: &mut MarketplacePlugin, state: &MarketplaceState) {
    plugin.is_installed = state.installed.contains(&plugin.id);
    if plugin.is_installed {
        if plugin.has_update {
            plugin.installed_version = Some(plugin.version.clone());
        } else {
            plugin.installed_version = Some(plugin.version.clone());
        }
    }
}

fn sync_all_installed(plugins: &mut [MarketplacePlugin], state: &MarketplaceState) {
    for p in plugins.iter_mut() {
        sync_installed(p, state);
    }
}

#[tauri::command]
pub fn marketplace_list(category: Option<String>, page: Option<u32>, sort: Option<String>) -> Result<MarketplaceSearchResult, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut plugins: Vec<MarketplacePlugin> = if let Some(ref cat) = category {
        state.plugins.values().filter(|p| p.category == *cat).cloned().collect()
    } else {
        state.plugins.values().cloned().collect()
    };
    sync_all_installed(&mut plugins, &state);

    match sort.as_deref() {
        Some("downloads") => plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads)),
        Some("rating") => plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)),
        Some("name") => plugins.sort_by(|a, b| a.name.cmp(&b.name)),
        _ => plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }

    Ok(paginate(plugins, page.unwrap_or(1)))
}

#[tauri::command]
pub fn marketplace_search(query: String, category: Option<String>, page: Option<u32>) -> Result<MarketplaceSearchResult, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let q = query.to_lowercase();
    let mut plugins: Vec<MarketplacePlugin> = state
        .plugins
        .values()
        .filter(|p| {
            if let Some(ref cat) = category {
                if p.category != *cat { return false; }
            }
            p.name.to_lowercase().contains(&q)
                || p.description.to_lowercase().contains(&q)
                || p.tags.iter().any(|t| t.contains(&q))
        })
        .cloned()
        .collect();
    sync_all_installed(&mut plugins, &state);
    plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    Ok(paginate(plugins, page.unwrap_or(1)))
}

#[tauri::command]
pub fn marketplace_get(plugin_id: String) -> Result<MarketplacePlugin, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut plugin = state
        .plugins
        .get(&plugin_id)
        .cloned()
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
    sync_installed(&mut plugin, &state);
    Ok(plugin)
}

#[tauri::command]
pub fn marketplace_install(plugin_id: String) -> Result<String, String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if state.installed.contains(&plugin_id) {
        return Err(format!("Plugin '{}' is already installed", plugin_id));
    }
    let (name, ver) = state
        .plugins
        .get(&plugin_id)
        .map(|p| (p.name.clone(), p.version.clone()))
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
    state.installed.insert(plugin_id.clone());
    if let Some(plugin) = state.plugins.get_mut(&plugin_id) {
        plugin.downloads += 1;
        plugin.is_installed = true;
        plugin.installed_version = Some(ver.clone());
    }
    Ok(format!("Installed {} v{}", name, ver))
}

#[tauri::command]
pub fn marketplace_uninstall(plugin_id: String) -> Result<(), String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !state.installed.contains(&plugin_id) {
        return Err(format!("Plugin '{}' is not installed", plugin_id));
    }
    state.installed.remove(&plugin_id);
    if let Some(plugin) = state.plugins.get_mut(&plugin_id) {
        plugin.is_installed = false;
        plugin.installed_version = None;
        plugin.has_update = false;
    }
    Ok(())
}

#[tauri::command]
pub fn marketplace_update(plugin_id: String) -> Result<String, String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !state.installed.contains(&plugin_id) {
        return Err(format!("Plugin '{}' is not installed", plugin_id));
    }
    let plugin = state
        .plugins
        .get_mut(&plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
    if !plugin.has_update {
        return Err(format!("Plugin '{}' is already up to date", plugin_id));
    }
    plugin.has_update = false;
    plugin.installed_version = Some(plugin.version.clone());
    Ok(format!("Updated {} to v{}", plugin.name, plugin.version))
}

#[tauri::command]
pub fn marketplace_check_updates() -> Result<Vec<MarketplacePlugin>, String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut updated: Vec<MarketplacePlugin> = Vec::new();
    for pid in state.installed.clone() {
        if let Some(plugin) = state.plugins.get_mut(&pid) {
            let has_update = (plugin.downloads.wrapping_mul(7) as u64 % 5) == 0;
            plugin.has_update = has_update;
            if has_update {
                let mut p = plugin.clone();
                p.is_installed = true;
                p.installed_version = Some(plugin.version.clone());
                updated.push(p);
            }
        }
    }
    Ok(updated)
}

#[tauri::command]
pub fn marketplace_update_all() -> Result<usize, String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut count = 0usize;
    let ids: Vec<String> = state.installed.iter().cloned().collect();
    for pid in ids {
        if let Some(plugin) = state.plugins.get_mut(&pid) {
            if plugin.has_update {
                plugin.has_update = false;
                count += 1;
            }
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn marketplace_reviews(plugin_id: String) -> Result<Vec<MarketplaceReview>, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !state.plugins.contains_key(&plugin_id) {
        return Err(format!("Plugin '{}' not found", plugin_id));
    }
    Ok(state.reviews.get(&plugin_id).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn marketplace_submit_review(plugin_id: String, rating: u8, title: String, body: String) -> Result<(), String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !state.plugins.contains_key(&plugin_id) {
        return Err(format!("Plugin '{}' not found", plugin_id));
    }
    if rating < 1 || rating > 5 {
        return Err("Rating must be between 1 and 5".into());
    }
    let review = MarketplaceReview {
        id: format!("rev-{}-{}", plugin_id, state.reviews.get(&plugin_id).map_or(0, |v| v.len())),
        plugin_id: plugin_id.clone(),
        author: "you".into(),
        rating,
        title,
        body,
        created_at: now_ts(),
        helpful_count: 0,
    };
    state.reviews.entry(plugin_id).or_default().push(review);
    Ok(())
}

#[tauri::command]
pub fn marketplace_categories() -> Result<Vec<MarketplaceCategory>, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut cat_map: HashMap<String, u32> = HashMap::new();
    for plugin in state.plugins.values() {
        *cat_map.entry(plugin.category.clone()).or_default() += 1;
    }
    let mut cats = Vec::new();
    for (id, count) in cat_map {
        let (name, desc) = match id.as_str() {
            "security" => ("Security", "Vulnerability scanning, secrets detection, dependency auditing"),
            "deployment" => ("Deployment", "CI/CD, Docker, Kubernetes, cloud publishing"),
            "testing" => ("Testing", "Test runners, coverage, benchmarking, E2E automation"),
            "frontend" => ("Frontend", "React, CSS, component generation, accessibility"),
            "ai" => ("AI", "LLM prompts, model benchmarks, code review automation"),
            _ => (&*id, "Category"),
        };
        cats.push(MarketplaceCategory {
            id: id.clone(),
            name: name.into(),
            description: desc.into(),
            count,
        });
    }
    cats.sort_by(|a, b| b.count.cmp(&a.count));
    Ok(cats)
}

#[tauri::command]
pub fn marketplace_stats() -> Result<MarketplaceStats, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let total_plugins = state.plugins.len() as u32;
    let total_downloads: u64 = state.plugins.values().map(|p| p.downloads).sum();
    let total_installed = state.installed.len();
    let updates_available = state.plugins.values().filter(|p| state.installed.contains(&p.id) && p.has_update).count();
    let mut categories: HashSet<&str> = HashSet::new();
    for p in state.plugins.values() {
        categories.insert(p.category.as_str());
    }
    Ok(MarketplaceStats {
        total_plugins,
        total_downloads,
        total_installed,
        updates_available,
        categories: categories.len(),
    })
}

#[tauri::command]
pub fn marketplace_config() -> Result<MarketplaceConfig, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn marketplace_set_config(config: MarketplaceConfig) -> Result<(), String> {
    let mut state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn marketplace_featured() -> Result<Vec<MarketplacePlugin>, String> {
    let state = MARKETPLACE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut plugins: Vec<MarketplacePlugin> = state.plugins.values().cloned().collect();
    sync_all_installed(&mut plugins, &state);
    plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    Ok(plugins.into_iter().take(4).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        let mut state = MARKETPLACE.lock().unwrap();
        state.installed.clear();
        state.config = MarketplaceConfig::default();
        for p in state.plugins.values_mut() {
            p.is_installed = false;
            p.has_update = false;
            p.installed_version = None;
        }
    }

    #[test]
    fn test_marketplace_list_default() {
        reset();
        let result = marketplace_list(None, None, None).unwrap();
        assert_eq!(result.total, 20);
        assert_eq!(result.page, 1);
        assert!(result.total_pages >= 1);
        assert_eq!(result.results.len(), 12);
    }

    #[test]
    fn test_marketplace_list_category_filter() {
        reset();
        let result = marketplace_list(Some("security".into()), None, None).unwrap();
        assert_eq!(result.total, 4);
        assert!(result.results.iter().all(|p| p.category == "security"));
    }

    #[test]
    fn test_marketplace_search_by_name() {
        reset();
        let result = marketplace_search("scanner".into(), None, None).unwrap();
        assert!(result.total > 0);
        assert!(result.results.iter().any(|p| p.name.to_lowercase().contains("scanner")));
    }

    #[test]
    fn test_marketplace_install_uninstall() {
        reset();
        let msg = marketplace_install("code-scanner".into()).unwrap();
        assert!(msg.contains("Installed"));

        let dup = marketplace_install("code-scanner".into());
        assert!(dup.is_err());

        let plugin = marketplace_get("code-scanner".into()).unwrap();
        assert!(plugin.is_installed);

        marketplace_uninstall("code-scanner".into()).unwrap();
        let plugin = marketplace_get("code-scanner".into()).unwrap();
        assert!(!plugin.is_installed);
    }

    #[test]
    fn test_marketplace_featured() {
        reset();
        let featured = marketplace_featured().unwrap();
        assert_eq!(featured.len(), 4);
        assert!(featured[0].downloads >= featured[1].downloads);
    }
}
