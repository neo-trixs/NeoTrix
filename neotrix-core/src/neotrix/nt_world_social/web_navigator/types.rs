use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic action result from web interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
    pub screenshot: Option<String>,
}

/// Navigation target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavTarget {
    pub url: String,
    pub wait_selector: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// Browser context state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub current_url: String,
    pub title: String,
    pub cookies: u32,
    pub html: Option<String>,
    pub screenshot_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageExtract {
    pub title: String,
    pub text: String,
    pub url: String,
    pub links: Vec<String>,
    pub html: Option<String>,
    pub screenshot_b64: Option<String>,
    pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HumanBehavior {
    pub typing_speed_ms: u64,
    pub scroll_pause_ms: u64,
    pub click_delay_ms: u64,
    pub random_delay_range: (u64, u64),
}

pub struct LoginCredentials {
    pub username_field: String, // CSS selector
    pub password_field: String,
    pub username: String,
    pub password: String,
    pub submit_selector: Option<String>,
    pub wait_for_url: Option<String>, // 登录成功后等待的 URL 模式
}

impl std::fmt::Debug for LoginCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginCredentials")
            .field("username", &self.username)
            .field("password", &"***REDACTED***")
            .field("url", &self.wait_for_url)
            .finish()
    }
}

impl Default for LoginCredentials {
    fn default() -> Self {
        Self {
            username_field: "input[name='text'], input[autocomplete='username'], input[name='session[username_or_email]']".into(),
            password_field: "input[type='password'], input[name='session[password]']".into(),
            username: String::new(),
            password: String::new(),
            submit_selector: None,
            wait_for_url: None,
        }
    }
}
