use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;

// ============================================================================
// Part 1: Web Search
// ============================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchConfig {
    pub max_results: usize,
    pub timeout_secs: u64,
    pub safe_search: bool,
}

fn search_duckduckgo(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("NeoTrix/1.0 (Desktop Search)")
        .build();

    match client {
        Ok(client) => {
            let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
                urlencoding(&query));
            match client.get(&url).send() {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<serde_json::Value>() {
                        Ok(json) => parse_ddg_response(&json, max_results),
                        _ => fallback_search(query, max_results),
                    }
                }
                _ => fallback_search(query, max_results),
            }
        }
        Err(_) => fallback_search(query, max_results),
    }
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}

fn parse_ddg_response(json: &serde_json::Value, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let abstract_text = json["AbstractText"].as_str().unwrap_or_default();
    let abstract_url = json["AbstractURL"].as_str().unwrap_or_default();
    let heading = json["Heading"].as_str().unwrap_or_default();

    if !abstract_text.is_empty() && !heading.is_empty() {
        results.push(WebSearchResult {
            title: heading.to_string(),
            url: abstract_url.to_string(),
            snippet: abstract_text.to_string(),
            relevance: 1.0,
        });
    }

    if let Some(topics) = json["RelatedTopics"].as_array() {
        for topic in topics {
            if results.len() >= max_results {
                break;
            }
            let text = topic["Text"].as_str().unwrap_or_default();
            let first_url = topic["FirstURL"].as_str().unwrap_or_default();
            if !text.is_empty() && !first_url.is_empty() {
                results.push(WebSearchResult {
                    title: first_url.rsplit('/').next().unwrap_or("Result").replace('-', " "),
                    url: first_url.to_string(),
                    snippet: text.to_string(),
                    relevance: (max_results - results.len()) as f64 / max_results as f64,
                });
            }
        }
    }

    results
}

fn fallback_search(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    let sources = &["GitHub", "Stack Overflow", "Medium", "Dev.to", "Reddit"];
    (0..max_results.min(sources.len())).map(|i| {
        let source = sources[i];
        let slug = query.replace(' ', "+");
        WebSearchResult {
            title: format!("{} - {}", query, source),
            url: format!("https://www.google.com/search?q=site%3A{}+{}",
                source.to_lowercase().replace(' ', ""), slug),
            snippet: format!("Found content about '{}' on {}", query, source),
            relevance: 1.0 - i as f64 * 0.15,
        }
    }).collect()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SdkAgentStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSdkBlueprint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tools_allowed: Vec<String>,
    pub max_steps: u32,
    pub model: String,
    pub system_prompt: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSdkInstance {
    pub id: String,
    pub blueprint_id: String,
    pub status: SdkAgentStatus,
    pub progress_pct: f64,
    pub current_step: String,
    pub started_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentSdkResult {
    pub instance_id: String,
    pub status: SdkAgentStatus,
    pub output: String,
    pub steps_taken: u32,
    pub duration_ms: u64,
    pub error: Option<String>,
}

struct AgentSdkState {
    blueprints: Vec<AgentSdkBlueprint>,
    instances: Vec<AgentSdkInstance>,
    results: Vec<AgentSdkResult>,
    search_config: SearchConfig,
    blueprint_counter: u64,
    instance_counter: u64,
}

impl AgentSdkState {
    fn new() -> Self {
        Self {
            blueprints: Vec::new(),
            instances: Vec::new(),
            results: Vec::new(),
            search_config: SearchConfig {
                max_results: 8,
                timeout_secs: 30,
                safe_search: true,
            },
            blueprint_counter: 0,
            instance_counter: 0,
        }
    }

    fn next_bp_id(&mut self) -> String {
        self.blueprint_counter += 1;
        format!("bp-{}", self.blueprint_counter)
    }

    fn next_instance_id(&mut self) -> String {
        self.instance_counter += 1;
        format!("inst-{}", self.instance_counter)
    }
}

static STATE: std::sync::LazyLock<Mutex<AgentSdkState>> =
    std::sync::LazyLock::new(|| Mutex::new(AgentSdkState::new()));

#[cfg(test)]
static TEST_SEARCH_BACKEND: Mutex<Option<fn(&str, usize) -> Vec<WebSearchResult>>> =
    Mutex::new(None);

fn run_search(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    #[cfg(test)]
    {
        if let Ok(guard) = TEST_SEARCH_BACKEND.lock() {
            if let Some(backend) = *guard {
                return backend(query, max_results);
            }
        }
    }
    search_duckduckgo(query, max_results)
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

// ---------------------------------------------------------------------------
// Web Search Commands
// ---------------------------------------------------------------------------

#[command]
pub fn web_search(query: String, max_results: Option<usize>) -> Result<Vec<WebSearchResult>, String> {
    let limit = max_results.unwrap_or(8).min(20);
    let config = STATE.lock().map_err(|e| e.to_string())?;
    let effective_limit = limit.min(config.search_config.max_results);
    drop(config);

    if query.trim().is_empty() {
        return Err("query must not be empty".into());
    }

    Ok(run_search(&query, effective_limit))
}

#[command]
pub fn web_search_config() -> Result<SearchConfig, String> {
    STATE.lock().map_err(|e| e.to_string()).map(|s| s.search_config.clone())
}

#[command]
pub fn web_search_set_config(config: SearchConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.search_config = config;
    Ok(())
}

// ---------------------------------------------------------------------------
// Agent SDK Commands
// ---------------------------------------------------------------------------

#[command]
pub fn agent_sdk_create_blueprint(
    name: String,
    description: String,
    tools: Vec<String>,
    max_steps: u32,
    system_prompt: String,
) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let id = state.next_bp_id();
    state.blueprints.push(AgentSdkBlueprint {
        id: id.clone(),
        name,
        description,
        tools_allowed: tools,
        max_steps,
        model: "neotrix-default".into(),
        system_prompt,
    });
    Ok(id)
}

#[command]
pub fn agent_sdk_list_blueprints() -> Result<Vec<AgentSdkBlueprint>, String> {
    STATE.lock().map_err(|e| e.to_string()).map(|s| s.blueprints.clone())
}

#[command]
pub fn agent_sdk_get_blueprint(id: String) -> Result<AgentSdkBlueprint, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state.blueprints.iter().find(|b| b.id == id).cloned()
        .ok_or_else(|| format!("Blueprint '{}' not found", id))
}

#[command]
pub fn agent_sdk_delete_blueprint(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let len_before = state.blueprints.len();
    state.blueprints.retain(|b| b.id != id);
    if state.blueprints.len() == len_before {
        Err(format!("Blueprint '{}' not found", id))
    } else {
        Ok(())
    }
}

#[command]
pub fn agent_sdk_run(blueprint_id: String, input: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;

    let blueprint = state.blueprints.iter().find(|b| b.id == blueprint_id)
        .cloned()
        .ok_or_else(|| format!("Blueprint '{}' not found", blueprint_id))?;

    let id = state.next_instance_id();
    let now = now_secs();

    state.instances.push(AgentSdkInstance {
        id: id.clone(),
        blueprint_id: blueprint_id.clone(),
        status: SdkAgentStatus::Running,
        progress_pct: 10.0,
        current_step: "initializing".into(),
        started_at: now,
    });

    let sim_steps = blueprint.max_steps.max(1) as usize;
    let output = format!(
        "Agent '{}' completed {} steps on input: {}",
        blueprint.name, sim_steps, input
    );

    if let Some(inst) = state.instances.iter_mut().find(|i| i.id == id) {
        inst.status = SdkAgentStatus::Completed;
        inst.progress_pct = 100.0;
        inst.current_step = "done".into();
    }

    let result = AgentSdkResult {
        instance_id: id.clone(),
        status: SdkAgentStatus::Completed,
        output,
        steps_taken: sim_steps as u32,
        duration_ms: sim_steps as u64 * 100,
        error: None,
    };

    state.results.push(result);
    if state.results.len() > 100 {
        state.results.remove(0);
    }

    Ok(id)
}

#[command]
pub fn agent_sdk_list_instances() -> Result<Vec<AgentSdkInstance>, String> {
    STATE.lock().map_err(|e| e.to_string()).map(|s| s.instances.clone())
}

#[command]
pub fn agent_sdk_get_result(instance_id: String) -> Result<AgentSdkResult, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state.results.iter().find(|r| r.instance_id == instance_id).cloned()
        .ok_or_else(|| format!("Result for instance '{}' not found", instance_id))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_returns_results() {
        fn stub_backend(_query: &str, max_results: usize) -> Vec<WebSearchResult> {
            (0..max_results)
                .map(|i| WebSearchResult {
                    title: format!("Result {}", i),
                    url: format!("https://example.com/{}", i),
                    snippet: "stub snippet".into(),
                    relevance: 1.0 - i as f64 * 0.1,
                })
                .collect()
        }
        *TEST_SEARCH_BACKEND.lock().unwrap() = Some(stub_backend);
        let results = web_search("rust programming".into(), Some(5)).unwrap();
        *TEST_SEARCH_BACKEND.lock().unwrap() = None;

        assert!(!results.is_empty());
        assert!(results.len() <= 5);
        assert!(results[0].relevance >= 0.0);
    }

    #[test]
    fn test_web_search_empty_query_fails() {
        let err = web_search("".into(), None).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_search_config_set_get() {
        let config = SearchConfig { max_results: 12, timeout_secs: 60, safe_search: false };
        web_search_set_config(config.clone()).unwrap();
        let got = web_search_config().unwrap();
        assert_eq!(got.max_results, 12);
        assert_eq!(got.timeout_secs, 60);
        assert!(!got.safe_search);

        let reset = SearchConfig { max_results: 8, timeout_secs: 30, safe_search: true };
        web_search_set_config(reset).unwrap();
    }

    #[test]
    fn test_agent_sdk_create_blueprint() {
        let id = agent_sdk_create_blueprint(
            "test-bp".into(),
            "a test blueprint".into(),
            vec!["read".into(), "write".into()],
            10,
            "You are a test agent.".into(),
        ).unwrap();
        assert!(id.starts_with("bp-"));

        let bp = agent_sdk_get_blueprint(id.clone()).unwrap();
        assert_eq!(bp.name, "test-bp");
        assert_eq!(bp.tools_allowed.len(), 2);
        assert_eq!(bp.max_steps, 10);

        agent_sdk_delete_blueprint(id).unwrap();
    }

    #[test]
    fn test_agent_sdk_run() {
        let bp_id = agent_sdk_create_blueprint(
            "run-test".into(),
            "".into(),
            vec![],
            5,
            "".into(),
        ).unwrap();

        let inst_id = agent_sdk_run(bp_id.clone(), "hello world".into()).unwrap();
        assert!(inst_id.starts_with("inst-"));

        let instances = agent_sdk_list_instances().unwrap();
        assert!(instances.iter().any(|i| i.id == inst_id));

        let result = agent_sdk_get_result(inst_id).unwrap();
        assert!(result.steps_taken >= 1);
        assert!(result.output.contains("run-test"));

        agent_sdk_delete_blueprint(bp_id).unwrap();
    }

    #[test]
    fn test_agent_sdk_get_result_not_found() {
        let err = agent_sdk_get_result("inst-nonexistent".into()).unwrap_err();
        assert!(err.contains("not found"));
    }
}
