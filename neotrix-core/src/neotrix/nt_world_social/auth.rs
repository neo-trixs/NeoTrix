#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlatformTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
}

impl std::fmt::Debug for PlatformTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlatformTokens")
            .field("access_token", &"***REDACTED***")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "***REDACTED***"),
            )
            .field("expires_at", &self.expires_at)
            .field("scope", &self.scope)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStore {
    pub tokens: HashMap<String, PlatformTokens>,
}

pub struct SocialAuth {
    store_path: PathBuf,
    tokens: Mutex<TokenStore>,
}

impl SocialAuth {
    pub fn new() -> Self {
        Self {
            store_path: PathBuf::from("social_tokens.json"),
            tokens: Mutex::new(TokenStore::default()),
        }
    }

    pub fn is_token_valid(&self, _platform: &str) -> bool {
        let store = self.tokens.lock().unwrap_or_else(|e| e.into_inner());
        store.tokens.get(_platform).map_or(false, |t| {
            t.expires_at
                .map_or(true, |exp| exp > chrono::Utc::now().timestamp())
        })
    }

    pub fn set_token(&self, platform: &str, tokens: PlatformTokens) {
        let mut store = self.tokens.lock().unwrap_or_else(|e| e.into_inner());
        store.tokens.insert(platform.to_string(), tokens);
    }

    pub fn get_token(&self, platform: &str) -> Option<PlatformTokens> {
        let store = self.tokens.lock().unwrap_or_else(|e| e.into_inner());
        store.tokens.get(platform).cloned()
    }
}

impl Default for SocialAuth {
    fn default() -> Self {
        Self::new()
    }
}
