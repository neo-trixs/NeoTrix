use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStore {
    pub tokens: HashMap<String, PlatformTokens>,
}

pub struct SocialAuth {
    store_path: PathBuf,
    tokens: Mutex<TokenStore>,
}

impl Default for SocialAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialAuth {
    pub fn new() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix")
            .join("social_tokens");
        std::fs::create_dir_all(&base).ok();

        let store_path = base.join("tokens.json");
        let tokens = std::fs::read_to_string(&store_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        SocialAuth {
            store_path,
            tokens: Mutex::new(tokens),
        }
    }

    pub fn get_token(&self, platform: &str) -> Option<PlatformTokens> {
        let store = self.tokens.lock().expect("mutex poisoned");
        store.tokens.get(platform).cloned()
    }

    pub fn set_token(&self, platform: &str, token: PlatformTokens) {
        let mut store = self.tokens.lock().expect("mutex poisoned");
        store.tokens.insert(platform.to_string(), token);
        self.persist(&store);
    }

    pub fn remove_token(&self, platform: &str) {
        let mut store = self.tokens.lock().expect("mutex poisoned");
        store.tokens.remove(platform);
        self.persist(&store);
    }

    pub fn is_token_valid(&self, platform: &str) -> bool {
        let store = self.tokens.lock().expect("mutex poisoned");
        match store.tokens.get(platform) {
            Some(t) => {
                if let Some(expires) = t.expires_at {
                    chrono::Utc::now().timestamp() < expires - 60
                } else {
                    true
                }
            }
            None => false,
        }
    }

    pub fn all_platforms(&self) -> Vec<String> {
        let store = self.tokens.lock().expect("mutex poisoned");
        store.tokens.keys().cloned().collect()
    }

    fn persist(&self, store: &TokenStore) {
        if let Ok(json) = serde_json::to_string_pretty(store) {
            std::fs::write(&self.store_path, json).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(expires_at: Option<i64>) -> PlatformTokens {
        PlatformTokens {
            access_token: "tok_abc123".into(),
            refresh_token: Some("ref_xyz".into()),
            expires_at,
            scope: Some("read write".into()),
        }
    }

    #[test]
    fn test_platform_tokens_construction() {
        let t = make_token(Some(9999999999999));
        assert_eq!(t.access_token, "tok_abc123");
        assert_eq!(t.refresh_token.as_deref(), Some("ref_xyz"));
        assert_eq!(t.scope.as_deref(), Some("read write"));
    }

    #[test]
    fn test_is_token_valid_no_expiry() {
        let auth = SocialAuth {
            store_path: PathBuf::from("/tmp/neotrix_test_tokens.json"),
            tokens: Mutex::new(TokenStore::default()),
        };
        let token = make_token(None);
        auth.set_token("test_platform", token);
        assert!(auth.is_token_valid("test_platform"));
        auth.remove_token("test_platform");
        assert!(!auth.is_token_valid("test_platform"));
    }

    #[test]
    fn test_get_token_returns_none_for_missing() {
        let auth = SocialAuth {
            store_path: PathBuf::from("/tmp/neotrix_test_tokens.json"),
            tokens: Mutex::new(TokenStore::default()),
        };
        assert!(auth.get_token("nonexistent").is_none());
    }

    #[test]
    fn test_all_platforms_empty() {
        let auth = SocialAuth {
            store_path: PathBuf::from("/tmp/neotrix_test_tokens.json"),
            tokens: Mutex::new(TokenStore::default()),
        };
        assert!(auth.all_platforms().is_empty());
    }
}
