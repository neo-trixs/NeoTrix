//! Browser 命令组 — 浏览器窗口控制 + 凭据管理 + WebApp Agent + X 自动浏览

use log;
use neotrix::neotrix::nt_act_social::tweet_stream::{NegentropyScore, TweetStream};
use neotrix::neotrix::nt_act_social::web_navigator::{HumanBehavior, WebNavigator};
use neotrix::neotrix::nt_act_social::x_scraper::RawTweet;
use neotrix::neotrix::nt_mind::credential_manager::{
    AuditEntry, CredentialManager, PasswordHealthReport, load_or_generate_master_key,
};
use neotrix::neotrix::nt_mind::webapp_agent::WebAppRegistry;
use serde::Deserialize;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use neotrix::SelfIteratingBrain;
use tauri::AppHandle;
use tauri::{Emitter, State};

use crate::browser_host::{BrowserHost, BrowserState};

// ============================================================================
// 状态: 全局 CredentialManager + WebAppRegistry
// ============================================================================

const CREDENTIAL_IDLE_TIMEOUT_SECS: u64 = 300; // 5 min 无操作 → 自动锁定

pub struct CredentialState {
    pub mgr: Mutex<CredentialManager>,
    last_active: AtomicU64,
    locked: AtomicBool,
}

impl CredentialState {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        match load_or_generate_master_key() {
            Ok(key) => Self {
                mgr: Mutex::new(CredentialManager::load_default(key)),
                last_active: AtomicU64::new(now),
                locked: AtomicBool::new(false),
            },
            Err(e) => {
                log::error!("[credential] master key init failed: {} — using transient key", e);
                Self {
                    mgr: Mutex::new(CredentialManager::new()),
                    last_active: AtomicU64::new(now),
                    locked: AtomicBool::new(false),
                }
            }
        }
    }

    /// Check lock state; auto-lock on idle timeout. Must be called before any credential access.
    pub fn check_lock(&self) -> Result<(), String> {
        if self.locked.load(Ordering::SeqCst) {
            return Err("LOCKED: credential store is locked — call browser_credential_unlock".into());
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let last = self.last_active.load(Ordering::Relaxed);
        if last > 0 && now.saturating_sub(last) > CREDENTIAL_IDLE_TIMEOUT_SECS {
            self.locked.store(true, Ordering::SeqCst);
            return Err("LOCKED: session timed out — call browser_credential_unlock".into());
        }
        self.last_active.store(now, Ordering::Relaxed);
        Ok(())
    }
}

/// WebApp registry — needs .manage() in main.rs to activate.
/// Struct kept because all webapp_agent commands reference State<'_, WebAppState>.
// DEAD — kept for reference (needs .manage() in main.rs + handler registration)
#[allow(dead_code)]
pub struct WebAppState(pub Mutex<WebAppRegistry>);

impl WebAppState {
    /// Called from tauri State management on first access. Not directly invoked.
    // DEAD — kept for reference (used by DEAD WebAppState)
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(Mutex::new(WebAppRegistry::new()))
    }
}

// ============================================================================
// 浏览器控制
// ============================================================================

/// Called from frontend (api.ts:browserOpen). Needs registration in generate_handler![] in main.rs
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_open(app: AppHandle, url: String) -> Result<BrowserState, String> {
    BrowserHost::open_or_navigate(&app, &url)
}

/// Called from frontend (api.ts:browserBack). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_back(app: AppHandle) -> Result<(), String> {
    BrowserHost::go_back(&app)
}

/// Called from frontend (api.ts:browserForward). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_forward(app: AppHandle) -> Result<(), String> {
    BrowserHost::go_forward(&app)
}

/// Called from frontend (api.ts:browserReload). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_reload(app: AppHandle) -> Result<(), String> {
    BrowserHost::reload(&app)
}

/// Called from frontend (api.ts:browserClose). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_close(app: AppHandle) -> Result<(), String> {
    BrowserHost::close(&app)
}

// ============================================================================
// 凭据管理
// ============================================================================

#[derive(Serialize)]
pub struct CredentialInfo {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub notes: String,
    pub created_at: u64,
}

#[tauri::command]
pub fn browser_credential_store(
    cred_state: State<'_, CredentialState>,
    domain: String,
    username: String,
    password: String,
    notes: Option<String>,
) -> Result<CredentialInfo, String> {
    cred_state.check_lock()?;
    let mut mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    let entry = mgr.store(&domain, &username, &password, &notes.unwrap_or_default());
    mgr.save_default().map_err(|e| format!("persist failed: {}", e))?;
    Ok(CredentialInfo {
        id: entry.id,
        domain: entry.domain,
        username: entry.username,
        notes: entry.notes,
        created_at: entry.created_at,
    })
}

#[derive(Serialize)]
pub struct CredentialListItem {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub notes: String,
    pub created_at: u64,
}

#[tauri::command]
pub fn browser_credential_list(
    cred_state: State<'_, CredentialState>,
) -> Result<Vec<CredentialListItem>, String> {
    cred_state.check_lock()?;
    let mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    Ok(mgr
        .all()
        .iter()
        .map(|e| CredentialListItem {
            id: e.id.clone(),
            domain: e.domain.clone(),
            username: e.username.clone(),
            notes: e.notes.clone(),
            created_at: e.created_at,
        })
        .collect())
}

#[tauri::command]
pub fn browser_credential_remove(
    cred_state: State<'_, CredentialState>,
    id: String,
) -> Result<bool, String> {
    cred_state.check_lock()?;
    let mut mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    let removed = mgr.remove(&id);
    if removed {
        mgr.save_default().map_err(|e| format!("persist failed: {}", e))?;
    }
    Ok(removed)
}

#[tauri::command]
pub fn browser_credential_autofill(
    app: AppHandle,
    cred_state: State<'_, CredentialState>,
    domain: String,
) -> Result<String, String> {
    cred_state.check_lock()?;
    let mut mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    if let Some(script) = mgr.auto_fill_script(&domain) {
        let _ = BrowserHost::execute_js(&app, &script);
        Ok("autofill script injected".to_string())
    } else {
        Err("no credentials found for domain".to_string())
    }
}

// ============================================================================
// 锁定/解锁
// ============================================================================

#[tauri::command]
pub fn browser_credential_lock(
    cred_state: State<'_, CredentialState>,
) -> Result<String, String> {
    cred_state.locked.store(true, Ordering::SeqCst);
    Ok("locked".into())
}

#[tauri::command]
pub fn browser_credential_unlock(
    cred_state: State<'_, CredentialState>,
    password: Option<String>,
) -> Result<String, String> {
    if let Ok(expected) = std::env::var("NEOTRIX_MASTER_PASSWORD") {
        match password {
            Some(p) if p.as_str() == expected => {},
            Some(_) => return Err("LOCKED: incorrect master password".into()),
            None => return Err("LOCKED: master password required".into()),
        }
    }
    cred_state.locked.store(false, Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    cred_state.last_active.store(now, Ordering::Relaxed);
    Ok("unlocked".into())
}

// ============================================================================
// 密码健康检查
// ============================================================================

#[tauri::command]
pub fn browser_credential_health_check(
    cred_state: State<'_, CredentialState>,
) -> Result<PasswordHealthReport, String> {
    cred_state.check_lock()?;
    let mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    Ok(mgr.health_check())
}

// ============================================================================
// 审计日志
// ============================================================================

#[tauri::command]
pub fn browser_credential_audit_log(
    cred_state: State<'_, CredentialState>,
    domain: Option<String>,
    since: Option<u64>,
) -> Result<Vec<AuditEntry>, String> {
    cred_state.check_lock()?;
    let mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    let entries: Vec<AuditEntry> = match (domain, since) {
        (Some(d), None) => mgr.audit_log_by_domain(&d).into_iter().cloned().collect(),
        (None, Some(s)) => mgr.audit_log_since(s).into_iter().cloned().collect(),
        (Some(d), Some(s)) => mgr
            .audit_log_by_domain(&d)
            .into_iter()
            .filter(|e| e.timestamp >= s)
            .cloned()
            .collect(),
        (None, None) => mgr.audit_log().cloned().collect(),
    };
    Ok(entries)
}

#[tauri::command]
pub fn browser_credential_clear_audit_log(
    cred_state: State<'_, CredentialState>,
) -> Result<String, String> {
    cred_state.check_lock()?;
    let mut mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    mgr.clear_audit_log();
    Ok("audit log cleared".into())
}

// ============================================================================
// 密钥轮换
// ============================================================================

#[tauri::command]
pub fn browser_credential_rotate_key(
    cred_state: State<'_, CredentialState>,
) -> Result<String, String> {
    cred_state.check_lock()?;
    let mut mgr = cred_state.mgr.lock().map_err(|e| e.to_string())?;
    mgr.rotate_key().map_err(|e| format!("key rotation failed: {}", e))?;
    mgr.save_default().map_err(|e| format!("persist after rotation failed: {}", e))?;
    Ok("key rotated".into())
}

// ============================================================================
// WebApp Agent 管理
// ============================================================================

/// Referenced by frontend (api.ts:WebAppAgentInfo). Needs registration in generate_handler![]
/// AND WebAppState needs .manage() in main.rs to work.
// DEAD — kept for reference (used only by DEAD webapp agent commands)
#[allow(dead_code)]
#[derive(Serialize, Clone)]
pub struct WebAppAgentInfo {
    pub id: String,
    pub name: String,
    pub url_pattern: String,
    pub actions: Vec<WebAppActionInfo>,
    pub is_active: bool,
}

/// Referenced by frontend (api.ts:WebAppAgentInfo.actions). Needs registration.
// DEAD — kept for reference (used only by DEAD WebAppAgentInfo)
#[allow(dead_code)]
#[derive(Serialize, Clone)]
pub struct WebAppActionInfo {
    pub id: String,
    pub label: String,
}

/// Called from frontend (api.ts:browserAgentList). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_agent_list(
    webapp_state: State<'_, WebAppState>,
) -> Result<Vec<WebAppAgentInfo>, String> {
    let reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
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

/// Called from frontend (api.ts:browserAgentDetect). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_agent_detect(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    url: String,
    title: String,
) -> Result<Option<WebAppAgentInfo>, String> {
    let agent_result = {
        let mut reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
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
                let mut reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
                let knowledge = reg.ingest_from_browser(&content.url, &content.title, &content.html);
                let _ = app.emit("browser:knowledge-collected", serde_json::json!({
                    "source_url": knowledge.source_url,
                    "title": knowledge.title,
                    "summary": knowledge.summary,
                }));
            }
            Err(e) => {
                log::error!("fetch content error: {}", e);
            }
        }

        let _ = app.emit("browser:agent-updated", ());
        Ok(Some(agent))
    } else {
        Ok(None)
    }
}

/// Called from frontend (api.ts:browserAgentExecute). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_agent_execute(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    agent_id: String,
    action_id: String,
) -> Result<String, String> {
    let reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
    let agent = reg
        .get_by_id(&agent_id)
        .ok_or_else(|| "agent not found".to_string())?;
    let action = agent
        .actions
        .iter()
        .find(|a| a.id == action_id)
        .ok_or_else(|| "action not found".to_string())?;

    BrowserHost::execute_js(&app, &action.script)?;
    Ok("executed".to_string())
}

// ============================================================================
// 页面内容智能提取
// ============================================================================

/// Called from frontend (api.ts:browserExtractContent). Needs registration in generate_handler![].
// DEAD — kept for reference (used only by DEAD browser_extract_content)
#[allow(dead_code)]
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

/// Arg type for browser_extract_content. Needs registration.
// DEAD — kept for reference (used only by DEAD browser_extract_content)
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ExtractContentArgs {
    pub url: String,
}

/// Called from frontend (api.ts:browserExtractContent). Needs registration in generate_handler![].
// DEAD — kept for reference (not registered in main.rs handler)
#[allow(dead_code)]
#[tauri::command]
pub fn browser_extract_content(
    app: AppHandle,
    webapp_state: State<'_, WebAppState>,
    args: ExtractContentArgs,
) -> Result<ContentExtractResult, String> {
    let page = BrowserHost::fetch_page_content(&args.url)?;
    let mut reg = webapp_state.0.lock().map_err(|e| e.to_string())?;
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
            human: HumanBehavior {
                typing_speed_ms: 100,
                scroll_pause_ms: 300,
                click_delay_ms: 150,
                random_delay_range: (50, 250),
            },
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
) -> Result<String, String> {
    let mut guard = x_state.navigator.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("Session already active".into());
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
) -> Result<String, String> {
    let guard = x_state.navigator.lock().map_err(|e| e.to_string())?;
    let nav = guard.as_ref().ok_or("No active session. Start session first.")?;

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

/// 从 X.com 页面提取推文 (JS 注入)
fn extract_tweets_from_page(nav: &WebNavigator, session: &str) -> Result<Vec<RawTweet>, String> {
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
    ).map_err(|e| format!("parse tweets: {}", e))?;

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
fn inject_to_agent(
    agent: &mut SelfIteratingBrain,
    _score: &NegentropyScore,
) {
    agent.brain.absorb(neotrix::neotrix::nt_mind::KnowledgeSource::SocialFeed);
}

#[tauri::command]
pub fn browser_x_human_scroll(
    app: AppHandle,
    x_state: State<'_, XAutoScrollState>,
    agent_state: State<'_, Arc<RwLock<SelfIteratingBrain>>>,
) -> Result<String, String> {
    let guard = x_state.navigator.lock().map_err(|e| e.to_string())?;
    let nav = guard.as_ref().ok_or("No active session")?;
    let session = nav.new_page()?;

    nav.navigate(&session, "https://x.com/home")?;
    std::thread::sleep(std::time::Duration::from_millis(4000));

    // 人类行为模拟滚动
    nav.scroll_to_bottom(&session, 30)?;

    // 提取推文
    let raw_tweets = extract_tweets_from_page(nav, &session)?;
    *x_state.tweet_count.lock().unwrap() = raw_tweets.len();
    *x_state.current_url.lock().unwrap() = "https://x.com/home".into();

    // 负熵评分管线
    let mut stream = x_state.tweet_stream.lock().map_err(|e| e.to_string())?;
    let known: Vec<String> = Vec::new();
    let novel: Vec<_> = raw_tweets.iter().filter(|t| stream.is_novel(t)).collect();
    let scores: Vec<NegentropyScore> = novel.iter()
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
        let mut agent = agent_state.blocking_write();
        for score in &absorbable {
            inject_to_agent(&mut agent, score);
        }
        *x_state.absorbed_count.lock().unwrap() += absorbable.len();
        *x_state.negentropy_total.lock().unwrap() += total_negentropy;

        // 发射知识吸收事件到前端
        let _ = app.emit("x:knowledge-absorbed", serde_json::json!({
            "count": absorbable.len(),
            "total_negentropy": total_negentropy,
            "avg_negentropy": avg_negentropy,
            "tweets_seen": raw_tweets.len(),
        }));
    }

    let total_absorbed = *x_state.absorbed_count.lock().unwrap();

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
) -> Result<String, String> {
    x_state.running.store(false, Ordering::SeqCst);
    let mut guard = x_state.navigator.lock().map_err(|e| e.to_string())?;
    if let Some(mut nav) = guard.take() {
        nav.close();
    }
    *x_state.tweet_count.lock().unwrap() = 0;
    *x_state.current_url.lock().unwrap() = String::new();
    *x_state.absorbed_count.lock().unwrap() = 0;
    *x_state.negentropy_total.lock().unwrap() = 0.0;
    Ok("Session stopped".into())
}

#[tauri::command]
pub fn browser_x_status(
    x_state: State<'_, XAutoScrollState>,
) -> Result<XAutoScrollStatus, String> {
    let abs = *x_state.absorbed_count.lock().unwrap();
    let n_total = *x_state.negentropy_total.lock().unwrap();
    let n_avg = if abs > 0 { n_total / abs as f64 } else { 0.0 };
    Ok(XAutoScrollStatus {
        running: x_state.running.load(Ordering::SeqCst),
        tweet_count: *x_state.tweet_count.lock().unwrap(),
        current_url: x_state.current_url.lock().unwrap().clone(),
        session_active: x_state.navigator.lock().unwrap().is_some(),
        absorbed: abs,
        negentropy_avg: n_avg,
    })
}

#[tauri::command]
pub fn browser_x_human_profile(
    x_state: State<'_, XAutoScrollState>,
) -> Result<serde_json::Value, String> {
    let hb = &x_state.human;
    Ok(serde_json::json!({
        "typing_speed_ms": hb.typing_speed_ms,
        "scroll_pause_ms": hb.scroll_pause_ms,
        "click_delay_ms": hb.click_delay_ms,
        "random_delay_range": [hb.random_delay_range.0, hb.random_delay_range.1],
    }))
}
