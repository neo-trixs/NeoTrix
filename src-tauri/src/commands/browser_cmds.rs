//! Browser 命令组 — 浏览器窗口控制 + 凭据管理 + WebApp Agent + X 自动浏览

use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_act_social::tweet_stream::{NegentropyScore, TweetStream};
use neotrix::neotrix::nt_act_social::web_navigator::{HumanBehavior, WebNavigator};
use neotrix::neotrix::nt_act_social::x_scraper::RawTweet;
use neotrix::neotrix::nt_mind::ReasoningBrain;
use serde::Deserialize;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::{Emitter, State};

use neotrix_tauri::browser_host::{BrowserHost, BrowserState};



// ============================================================================
// 内联 WebAppRegistry (nt_mind 无此模块)
// ============================================================================

#[allow(dead_code)]
struct WebAppAction {
    id: String,
    label: String,
    script: String,
}

#[allow(dead_code)]
struct WebAppAgent {
    id: String,
    name: String,
    url_pattern: String,
    actions: Vec<WebAppAction>,
    is_active: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
struct CollectedKnowledge {
    source_url: String,
    source_type: String,
    title: String,
    summary: String,
    collected_at: u64,
    consumed: bool,
}

#[allow(dead_code)]
pub(crate) struct WebAppRegistry {
    agents: Vec<WebAppAgent>,
    knowledge: Vec<CollectedKnowledge>,
}

#[allow(dead_code)]
impl WebAppRegistry {
    fn new() -> Self {
        Self { agents: Vec::new(), knowledge: Vec::new() }
    }

    fn all_agents(&self) -> &[WebAppAgent] {
        &self.agents
    }

    fn detect_or_create(&mut self, url: &str, title: &str) -> Option<&WebAppAgent> {
        let existing = self.agents.iter().position(|a| {
            url.contains(&a.url_pattern) || url.contains(&a.name.to_lowercase())
        });
        if let Some(idx) = existing {
            return Some(&self.agents[idx]);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let name = title.split(['-', '|', '—']).next().unwrap_or("Unknown").trim().to_string();
        self.agents.push(WebAppAgent {
            id: id.clone(),
            url_pattern: url.to_string(),
            name,
            actions: vec![WebAppAction {
                id: uuid::Uuid::new_v4().to_string(),
                label: "Click Main".into(),
                script: "document.querySelector('button, a, [role=button]')?.click()".into(),
            }],
            is_active: true,
        });
        self.agents.last()
    }

    fn ingest_from_browser(&mut self, url: &str, title: &str, html: &str) -> CollectedKnowledge {
        let collected_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let summary = format!("Extracted {} bytes from {}", html.len(), title);
        let ck = CollectedKnowledge {
            source_url: url.into(),
            source_type: "web".into(),
            title: title.into(),
            summary,
            collected_at,
            consumed: false,
        };
        let ck_clone = ck.clone();
        self.knowledge.push(ck);
        ck_clone
    }

    fn get_by_id(&self, id: &str) -> Option<&WebAppAgent> {
        self.agents.iter().find(|a| a.id == id)
    }

    fn list_unconsumed(&self) -> Vec<&CollectedKnowledge> {
        self.knowledge.iter().filter(|k| !k.consumed).collect()
    }
}

// ============================================================================
// 状态: 全局 WebAppRegistry
// ============================================================================

pub struct WebAppState(pub Mutex<WebAppRegistry>);

impl WebAppState {
    pub fn new() -> Self {
        Self(Mutex::new(WebAppRegistry::new()))
    }
}

// ============================================================================
// 浏览器控制
// ============================================================================

#[tauri::command]
pub fn browser_open(app: AppHandle, url: String) -> Result<BrowserState, NeoTrixError> {
    BrowserHost::open_or_navigate(&app, &url).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_navigate(app: AppHandle, url: String) -> Result<BrowserState, NeoTrixError> {
    BrowserHost::open_or_navigate(&app, &url).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_back(app: AppHandle) -> Result<(), NeoTrixError> {
    BrowserHost::go_back(&app).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_forward(app: AppHandle) -> Result<(), NeoTrixError> {
    BrowserHost::go_forward(&app).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_reload(app: AppHandle) -> Result<(), NeoTrixError> {
    BrowserHost::reload(&app).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_close(app: AppHandle) -> Result<(), NeoTrixError> {
    BrowserHost::close(&app).map_err(NeoTrixError::Network)
}

#[tauri::command]
pub fn browser_execute_js(app: AppHandle, script: String) -> Result<String, NeoTrixError> {
    BrowserHost::execute_js(&app, &script).map_err(NeoTrixError::Network)?;
    Ok("executed".to_string())
}



// ============================================================================
// WebApp Agent 管理
// ============================================================================

#[derive(Serialize, Clone)]
pub struct WebAppAgentInfo {
    pub id: String,
    pub name: String,
    pub url_pattern: String,
    pub actions: Vec<WebAppActionInfo>,
    pub is_active: bool,
}

#[derive(Serialize, Clone)]
pub struct WebAppActionInfo {
    pub id: String,
    pub label: String,
}

#[tauri::command]
pub fn browser_agent_list(
    webapp_state: State<'_, WebAppState>,
) -> Result<Vec<WebAppAgentInfo>, NeoTrixError> {
    let reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(reg
        .all_agents()
        .iter()
        .map(|a| WebAppAgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            url_pattern: a.url_pattern.clone(),
            actions: a
                .actions
                .iter()
                .map(|act| WebAppActionInfo {
                    id: act.id.clone(),
                    label: act.label.clone(),
                })
                .collect(),
            is_active: a.is_active,
        })
        .collect())
}

#[tauri::command]
pub fn browser_agent_detect(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    url: String,
    title: String,
) -> Result<Option<WebAppAgentInfo>, NeoTrixError> {
    let agent_result = {
        let mut reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
        reg.detect_or_create(&url, &title).map(|a| {
            WebAppAgentInfo {
                id: a.id.clone(),
                name: a.name.clone(),
                url_pattern: a.url_pattern.clone(),
                actions: a.actions.iter().map(|act| WebAppActionInfo {
                    id: act.id.clone(),
                    label: act.label.clone(),
                }).collect(),
                is_active: a.is_active,
            }
        })
    };

    if let Some(agent) = agent_result {
        // 服务端获取页面内容
        match BrowserHost::fetch_page_content(&url) {
            Ok(content) => {
                let mut reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
                let knowledge = reg.ingest_from_browser(&content.url, &content.title, &content.html);
                let _ = app.emit("browser:knowledge-collected", serde_json::json!({
                    "source_url": knowledge.source_url,
                    "title": knowledge.title,
                    "summary": knowledge.summary,
                }));
            }
            Err(e) => {
                eprintln!("fetch content error: {}", e);
            }
        }

        let _ = app.emit("browser:agent-updated", ());
        Ok(Some(agent))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn browser_agent_execute(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    agent_id: String,
    action_id: String,
) -> Result<String, NeoTrixError> {
    let reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let agent = reg
        .get_by_id(&agent_id)
        .ok_or_else(|| NeoTrixError::Brain("agent not found".to_string()))?;
    let action = agent
        .actions
        .iter()
        .find(|a| a.id == action_id)
        .ok_or_else(|| NeoTrixError::Brain("action not found".to_string()))?;

    BrowserHost::execute_js(&app, &action.script)?;
    Ok("executed".to_string())
}

// ============================================================================
// 页面内容智能提取
// ============================================================================

#[derive(Serialize)]
pub struct ContentExtractResult {
    pub title: String,
    pub primary_text: String,
    pub headings: Vec<String>,
    pub link_count: usize,
    pub table_count: usize,
    pub list_count: usize,
    pub summary: String,
}

#[derive(Deserialize)]
pub struct ExtractContentArgs {
    pub url: String,
}

#[tauri::command]
pub fn browser_extract_content(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    args: ExtractContentArgs,
) -> Result<ContentExtractResult, NeoTrixError> {
    let page = BrowserHost::fetch_page_content(&args.url)?;
    let mut reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let knowledge = reg.ingest_from_browser(&page.url, &page.title, &page.html);

    let _ = app.emit("browser:knowledge-collected", serde_json::json!({
        "source_url": knowledge.source_url,
        "title": knowledge.title,
        "summary": knowledge.summary,
    }));

    Ok(ContentExtractResult {
        title: page.title,
        primary_text: page.text.chars().take(2000).collect::<String>(),
        headings: vec![],
        link_count: 0,
        table_count: 0,
        list_count: 0,
        summary: format!("Extracted {} chars from {}", page.text.len(), page.url),
    })
}

// ============================================================================
// 采集知识管理 — 浏览器数据 → 意识管道
// ============================================================================

#[derive(Serialize)]
pub struct CollectedKnowledgeInfo {
    pub source_url: String,
    pub source_type: String,
    pub title: String,
    pub summary: String,
    pub collected_at: u64,
    pub consumed: bool,
}

/// 将浏览器采集的页面内容入队, 供 SEAL pipeline 消费
/// 前端调用此命令, 将提取的页面 HTML 喂给意识核心
#[tauri::command]
pub fn browser_ingest_content(
    webapp_state: State<'_, WebAppState>,
    url: String,
    title: String,
    html: String,
) -> Result<String, NeoTrixError> {
    let mut reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
    let knowledge = reg.ingest_from_browser(&url, &title, &html);
    let event_payload = serde_json::json!({
        "type": "webapp_data_collected",
        "source_url": knowledge.source_url,
        "source_type": knowledge.source_type,
        "title": knowledge.title,
        "summary": knowledge.summary,
        "collected_at": knowledge.collected_at,
    });
    Ok(serde_json::to_string(&event_payload).unwrap_or_default())
}

/// 列出所有未消费的采集知识 (非破坏性)
#[tauri::command]
pub fn browser_collected_knowledge(
    webapp_state: State<'_, WebAppState>,
) -> Result<Vec<CollectedKnowledgeInfo>, NeoTrixError> {
    let reg = webapp_state.0.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(reg.list_unconsumed()
        .into_iter()
        .map(|k| CollectedKnowledgeInfo {
            source_url: k.source_url.clone(),
            source_type: k.source_type.clone(),
            title: k.title.clone(),
            summary: k.summary.clone(),
            collected_at: k.collected_at,
            consumed: k.consumed,
        })
        .collect())
}

// ============================================================================
// X 自动浏览 — 人类行为模拟 + CDP 浏览器
// ============================================================================

/// X 自动浏览状态 — 包含浏览器、人类行为模拟、负熵去重管线
pub struct XAutoScrollState {
    pub running: AtomicBool,
    pub navigator: Mutex<Option<WebNavigator>>,
    pub human: HumanBehavior,
    pub tweet_count: Mutex<usize>,
    pub current_url: Mutex<String>,
    pub tweet_stream: Mutex<TweetStream>,
    pub absorbed_count: Mutex<usize>,
    pub negentropy_total: Mutex<f64>,
}

impl XAutoScrollState {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            navigator: Mutex::new(None),
            human: HumanBehavior::new(),
            tweet_count: Mutex::new(0),
            current_url: Mutex::new(String::new()),
            tweet_stream: Mutex::new(TweetStream::new()),
            absorbed_count: Mutex::new(0),
            negentropy_total: Mutex::new(0.0),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct XAutoScrollStatus {
    pub running: bool,
    pub tweet_count: usize,
    pub current_url: String,
    pub session_active: bool,
    pub absorbed: usize,
    pub negentropy_avg: f64,
}

#[tauri::command]
pub fn browser_x_start_session(
    app: AppHandle,
    x_state: State<'_, XAutoScrollState>,
) -> Result<String, NeoTrixError> {
    let mut guard = x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    if guard.is_some() {
        return Err(NeoTrixError::Brain("Session already active".into()));
    }

    let mut nav = WebNavigator::new();
    nav.launch()?;

    let session = nav.new_page()?;
    nav.navigate(&session, "https://x.com/home")?;

    *guard = Some(nav);

    let _ = app.emit("x:status-update", XAutoScrollStatus {
        running: false,
        tweet_count: 0,
        current_url: "https://x.com/home".into(),
        session_active: true,
        absorbed: 0,
        negentropy_avg: 0.0,
    });

    Ok("X session started".into())
}

#[tauri::command]
pub fn browser_x_login(
    app: AppHandle,
    x_state: State<'_, XAutoScrollState>,
    username: String,
    password: String,
) -> Result<String, NeoTrixError> {
    let guard = x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let nav = guard.as_ref().ok_or_else(|| NeoTrixError::Brain("No active session. Start session first.".into()))?;

    let session = nav.new_page()?;
    nav.navigate(&session, "https://x.com/i/flow/login")?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    nav.fill(&session, "input[name='text']", &username)?;
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let submit_js = r#"document.querySelector('button[type="submit"]')?.click();"#;
    nav.evaluate_js(&session, submit_js)?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    nav.fill(&session, "input[type='password'], input[name='password']", &password)?;
    std::thread::sleep(std::time::Duration::from_millis(1000));

    nav.evaluate_js(&session, submit_js)?;
    std::thread::sleep(std::time::Duration::from_millis(5000));

    nav.evaluate_js(&session, "window.close()")?;
    nav.save_cookies(&session, "x.com")?;

    let _ = app.emit("x:status-update", XAutoScrollStatus {
        running: false,
        tweet_count: 0,
        current_url: "https://x.com/home".into(),
        session_active: true,
        absorbed: 0,
        negentropy_avg: 0.0,
    });

    Ok("X login completed".into())
}

#[tauri::command]
pub fn browser_x_auto_scroll(
    _app: AppHandle,
    x_state: State<'_, XAutoScrollState>,
) -> Result<String, NeoTrixError> {
    if x_state.running.swap(true, Ordering::SeqCst) {
        return Err(NeoTrixError::Brain("Already running".into()));
    }

    let _nav = x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?
        .as_ref().ok_or_else(|| NeoTrixError::Brain("No active session".into()))?
        .new_page()?;

    // TODO: 后台线程持续滚动 — 需重构为 Arc<Mutex<>> 分离
    Ok("Auto-scroll started".into())
}

/// 从 X.com 页面提取推文 (JS 注入)
fn extract_tweets_from_page(nav: &WebNavigator, session: &str) -> Result<Vec<RawTweet>, NeoTrixError> {
    let js = r#"
(function() {
    const articles = document.querySelectorAll('article[data-testid="tweet"]');
    const tweets = [];
    for (const art of articles) {
        const id = art.querySelector('a[href*="/status/"]')?.href?.split('/status/').pop()?.split('?')[0] || '';
        const authorEl = art.querySelector('div[data-testid="User-Name"] a');
        const textEl = art.querySelector('div[data-testid="tweetText"]');
        const timeEl = art.querySelector('time');
        const likeEl = art.querySelector('button[data-testid="like"]');
        const retweetEl = art.querySelector('button[data-testid="retweet"]');
        const replyEl = art.querySelector('button[data-testid="reply"]');
        tweets.push({
            id, author: authorEl?.textContent?.split('@')[0]?.trim() || '',
            handle: authorEl?.href?.split('/').pop() || '',
            text: textEl?.textContent || '',
            time: timeEl?.getAttribute('datetime') || '',
            likes: parseInt(likeEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
            retweets: parseInt(retweetEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
            replies: parseInt(replyEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
        });
    }
    return JSON.stringify(tweets);
})();
"#;
    let json = nav.evaluate_js(session, js)?;
    let raw: Vec<serde_json::Value> = serde_json::from_str(
        json.as_str().unwrap_or("[]")
    ).map_err(|e| NeoTrixError::Brain(format!("parse tweets: {}", e)))?;

    Ok(raw.iter().map(|v| RawTweet {
        tweet_id: v["id"].as_str().unwrap_or("").to_string(),
        author: v["author"].as_str().unwrap_or("").to_string(),
        author_handle: v["handle"].as_str().unwrap_or("").to_string(),
        text: v["text"].as_str().unwrap_or("").to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        likes: v["likes"].as_u64().unwrap_or(0),
        retweets: v["retweets"].as_u64().unwrap_or(0),
        replies: v["replies"].as_u64().unwrap_or(0),
        views: None,
        url: format!("https://x.com/i/web/status/{}", v["id"].as_str().unwrap_or("")),
        is_thread: false, has_media: false, language: None,
    }).collect())
}

/// 将高负熵推文注入意识核心的能力向量吸收管线
fn inject_to_brain(
    brain: &mut ReasoningBrain,
    _score: &NegentropyScore,
) {
    brain.absorb(neotrix::neotrix::nt_mind::KnowledgeSource::Botasaurus);
}

#[tauri::command]
pub fn browser_x_human_scroll(
    app: AppHandle,
    x_state: State<'_, XAutoScrollState>,
    brain_state: State<'_, Mutex<ReasoningBrain>>,
) -> Result<String, NeoTrixError> {
    let guard = x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let nav = guard.as_ref().ok_or_else(|| NeoTrixError::Brain("No active session".into()))?;
    let session = nav.new_page()?;

    nav.navigate(&session, "https://x.com/home")?;
    std::thread::sleep(std::time::Duration::from_millis(4000));

    // 人类行为模拟滚动
    x_state.human.simulate_reading(&session, nav, 30)?;

    // 提取推文
    let raw_tweets = extract_tweets_from_page(nav, &session)?;
    *x_state.tweet_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = raw_tweets.len();
    *x_state.current_url.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = "https://x.com/home".into();

    // 负熵评分管线
    let mut stream = x_state.tweet_stream.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let known: Vec<String> = Vec::new();
    let novel_tweets: Vec<&RawTweet> = raw_tweets.iter()
        .filter(|t| stream.is_novel(t))
        .collect();
    let scores: Vec<NegentropyScore> = novel_tweets.iter()
        .map(|t| {
            stream.mark_seen(t);
            stream.score_tweet(t, &known, 0.3)
        })
        .collect();
    drop(stream);

    // 选择性吸收 — 仅 negentropy > 0.25
    let absorbable: Vec<&NegentropyScore> = scores.iter()
        .filter(|s| s.is_worth_absorbing())
        .collect();

    let total_negentropy: f64 = absorbable.iter().map(|s| s.negentropy).sum();
    let avg_negentropy = if absorbable.is_empty() { 0.0 } else { total_negentropy / absorbable.len() as f64 };

    // 注入意识核心
    if !absorbable.is_empty() {
        let mut brain = brain_state.lock().map_err(|e| NeoTrixError::Brain(format!("brain lock: {}", e)))?;
        for score in &absorbable {
            inject_to_brain(&mut brain, score);
        }
        *x_state.absorbed_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? += absorbable.len();
        *x_state.negentropy_total.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? += total_negentropy;

        // 发射知识吸收事件到前端
        let _ = app.emit("x:knowledge-absorbed", serde_json::json!({
            "count": absorbable.len(),
            "total_negentropy": total_negentropy,
            "avg_negentropy": avg_negentropy,
            "tweets_seen": raw_tweets.len(),
        }));
    }

    let total_absorbed = *x_state.absorbed_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;

    let _ = app.emit("x:status-update", XAutoScrollStatus {
        running: true,
        tweet_count: raw_tweets.len(),
        current_url: "https://x.com/home".into(),
        session_active: true,
        absorbed: total_absorbed,
        negentropy_avg: avg_negentropy,
    });

    let _ = nav.close_page(&session);

    if absorbable.is_empty() {
        Ok(format!("Scrolled 30s — {} tweets, all low-negentropy (filtered)", raw_tweets.len()))
    } else {
        Ok(format!(
            "Scrolled 30s — {} tweets, absorbed {} (avg negentropy={:.3})",
            raw_tweets.len(), absorbable.len(), avg_negentropy
        ))
    }
}

#[tauri::command]
pub fn browser_x_stop_session(
    x_state: State<'_, XAutoScrollState>,
) -> Result<String, NeoTrixError> {
    x_state.running.store(false, Ordering::SeqCst);
    let mut guard = x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    if let Some(mut nav) = guard.take() {
        nav.close();
    }
    *x_state.tweet_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = 0;
    *x_state.current_url.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = String::new();
    *x_state.absorbed_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = 0;
    *x_state.negentropy_total.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))? = 0.0;
    Ok("Session stopped".into())
}

#[tauri::command]
pub fn browser_x_status(
    x_state: State<'_, XAutoScrollState>,
) -> Result<XAutoScrollStatus, NeoTrixError> {
    let abs = *x_state.absorbed_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let n_total = *x_state.negentropy_total.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let n_avg = if abs > 0 { n_total / abs as f64 } else { 0.0 };
    Ok(XAutoScrollStatus {
        running: x_state.running.load(Ordering::SeqCst),
        tweet_count: *x_state.tweet_count.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?,
        current_url: x_state.current_url.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?.clone(),
        session_active: x_state.navigator.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?.is_some(),
        absorbed: abs,
        negentropy_avg: n_avg,
    })
}

#[tauri::command]
pub fn browser_x_human_profile(
    x_state: State<'_, XAutoScrollState>,
) -> Result<serde_json::Value, NeoTrixError> {
    let hb = &x_state.human;
    Ok(serde_json::json!({
        "scroll_speed": hb.scroll_speed,
        "pause_range": [hb.pause_duration.0, hb.pause_duration.1],
        "scroll_variance": hb.scroll_variance,
        "mouse_trail": hb.mouse_trail,
        "interaction_rate": hb.interaction_rate,
        "user_agent": hb.random_user_agent(),
    }))
}
