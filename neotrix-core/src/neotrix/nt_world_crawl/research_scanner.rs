use std::time::{Duration, Instant};

const FIELD_NAMES_DOMAIN: &[&str; 23] = &[
    "compound_composition","tailwind","accessibility","react_aria",
    "ai_native_states","semantic_layer","verification","quality_gates",
    "video_rendering","html_composition","secret_detection","nt_shield_audit",
    "vulnerability_knowledge","anti_detection","web_scraping","react_lint",
    "health_scoring","vector_design_canvas","mcp_design_tools",
    "agent_trading","signal_sync","esp32_firmware","quadruped_kinematics",
];

const FIELD_SEARCH_TERMS: &[&str; 23] = &[
    "compound+component+composition+design+system","tailwind+css+utility+framework",
    "web+accessibility+a11y+WCAG","react+aria+accessible+components",
    "AI+native+user+interface+adaptive+UX","semantic+layer+design+token+system",
    "verification+testing+formal+validation+quality","quality+gates+CI+pipeline+automation",
    "video+rendering+HTML+snapshot+capture","html+composition+layout+engine",
    "secret+detection+key+leak+nt_shield","nt_shield+audit+web+vulnerability+scanning",
    "vulnerability+knowledge+base+CVE+exploit","anti+detection+nt_world_browse+fingerprinting+stealth",
    "web+scraping+nt_world_crawl+automation+extraction","react+lint+static+analysis+code+quality",
    "health+scoring+codebase+quality+metrics","vector+design+canvas+graphics+editor",
    "MCP+tool+server+model+context+protocol","agent+trading+quantitative+finance+algorithm",
    "signal+synchronization+real+time+data+stream","ESP32+firmware+embedded+IoT+microcontroller",
    "quadruped+robot+kinematics+locomotion+control",
];

#[derive(Clone)]
pub struct DiscoResult {
    pub source: DiscoSource,
    pub title: String,
    pub url: String,
    pub relevance: f64,
    pub summary: String,
    pub matched_dimensions: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiscoSource {
    Arxiv,
    GitHub,
}

pub struct ResearchScanner {
    cycle: u64,
    scan_history: Vec<DiscoResult>,
    max_history: usize,
}

impl ResearchScanner {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            scan_history: Vec::new(),
            max_history: 500,
        }
    }

    pub fn run_scan(&mut self, dims: &[f64], weak_threshold: f64) -> Vec<DiscoResult> {
        self.cycle += 1;
        let weak_dims: Vec<(usize, &str)> = FIELD_NAMES_DOMAIN.iter().enumerate()
            .filter(|(i, _)| dims.get(*i).copied().unwrap_or(0.0) < weak_threshold)
            .map(|(i, name)| (i, *name))
            .collect();

        if weak_dims.is_empty() {
//            println!("[disco] cycle {}: no weak dimensions — skipping", self.cycle);
            return Vec::new();
        }

//        println!("[disco] cycle {}: {} weak dims — scanning", self.cycle, weak_dims.len());
        let mut results = Vec::new();
        let start = Instant::now();

        for (idx, name) in &weak_dims {
            let term = FIELD_SEARCH_TERMS.get(*idx).unwrap_or(name);
            if let Ok(repo) = search_github(term, name) {
                results.push(repo);
            }
            if let Ok(paper) = search_arxiv(term, name) {
                results.push(paper);
            }
        }

        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);

        self.scan_history.extend(results.clone());
        if self.scan_history.len() > self.max_history {
            self.scan_history.drain(..self.scan_history.len() - self.max_history);
        }

        let elapsed = start.elapsed();
        println!("[disco] cycle {}: {} results in {}ms",
            self.cycle, results.len(), elapsed.as_millis());

        results
    }

    pub fn recent_results(&self) -> &[DiscoResult] {
        &self.scan_history
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle
    }
}

fn search_github(query: &str, dim_name: &str) -> Result<DiscoResult, String> {
    let url = format!("https://api.github.com/search/repositories?q={}+sort:stars&per_page=3", query);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("NeoTrix/0.18")
        .build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
        if let Some(first) = items.first() {
            let name = first.get("full_name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let desc = first.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let stars = first.get("stargazers_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let relevance = (stars as f64 / 1000.0).min(1.0) * 0.8 + 0.2;
            return Ok(DiscoResult {
                source: DiscoSource::GitHub,
                title: format!("{}({}⭐)", name, stars),
                url: format!("https://github.com/{}", name),
                relevance,
                summary: desc.to_string(),
                matched_dimensions: vec![dim_name.to_string()],
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("result").as_secs() as i64,
            });
        }
    }
    Err("no results".into())
}

fn search_arxiv(query: &str, dim_name: &str) -> Result<DiscoResult, String> {
    let url = format!("https://export.arxiv.org/api/query?search_query=all:{}&sortBy=relevance&sortOrder=descending&max_results=2", query);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("NeoTrix/0.18")
        .build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let text = resp.text().map_err(|e| e.to_string())?;
    let title = extract_arxiv_title(&text).unwrap_or_default();
    let id = extract_arxiv_id(&text).unwrap_or_default();
    let summary = extract_arxiv_summary(&text).unwrap_or_default();
    if title.is_empty() || id.is_empty() {
        return Err("empty result".into());
    }
    Ok(DiscoResult {
        source: DiscoSource::Arxiv,
        title,
        url: format!("https://arxiv.org/abs/{}", id),
        relevance: 0.6,
        summary,
        matched_dimensions: vec![dim_name.to_string()],
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("result").as_secs() as i64,
    })
}

fn extract_arxiv_title(xml: &str) -> Option<String> {
    let s = xml.split("<title>").nth(1)?;
    let end = s.split("</title>").next()?;
    Some(end.trim().to_string())
}

fn extract_arxiv_id(xml: &str) -> Option<String> {
    let s = xml.split("<id>").nth(1)?;
    let end = s.split("</id>").next()?;
    let id = end.trim().trim_start_matches("http://arxiv.org/abs/").trim_start_matches("https://arxiv.org/abs/");
    Some(id.to_string())
}

fn extract_arxiv_summary(xml: &str) -> Option<String> {
    let s = xml.split("<summary>").nth(1)?;
    let end = s.split("</summary>").next()?;
    Some(end.trim().replace('\n', " ").chars().take(300).collect())
}
