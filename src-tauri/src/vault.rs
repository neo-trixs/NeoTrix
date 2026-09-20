use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Credential Vault - encrypted storage for API keys and secrets.
/// Keys never leave the Rust backend. Frontend only sees scoped tokens.
/// Pattern from ARES zero-trust + onecli credential isolation.

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ScopedToken {
    pub provider: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub scopes: Vec<String>,
}

impl ScopedToken {
    pub fn new(provider: impl Into<String>, ttl_secs: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            provider: provider.into(),
            created_at: now,
            expires_at: now + ttl_secs,
            scopes: vec!["read".into(), "execute".into()],
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now < self.expires_at
    }
}

pub struct CredentialVault {
    /// In-memory encrypted store (keyed by provider name)
    /// In production, this would use OS keyring + AES-256-GCM
    store: Arc<RwLock<HashMap<String, String>>>,
    /// Active scoped tokens
    tokens: Arc<RwLock<HashMap<String, ScopedToken>>>,
}

impl CredentialVault {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a credential and return a scoped token for frontend access
    pub async fn store_credential(
        &self,
        provider: &str,
        api_key: &str,
    ) -> Result<ScopedToken, VaultError> {
        // In production: encrypt with AES-256-GCM before storing
        // For now, store directly (keyring integration is in anthropic/client.rs)
        self.store.write().await.insert(provider.to_string(), api_key.to_string());

        let token = ScopedToken::new(provider, 3600); // 1 hour TTL
        self.tokens.write().await.insert(token_hash(&token), token.clone());

        Ok(token)
    }

    /// Resolve a scoped token to the actual API key
    pub async fn resolve(&self, token: &ScopedToken) -> Result<String, VaultError> {
        if !token.is_valid() {
            return Err(VaultError::TokenExpired);
        }

        self.store.read().await
            .get(&token.provider)
            .cloned()
            .ok_or(VaultError::NotFound(token.provider.clone()))
    }

    /// Check if a provider has a stored credential
    pub async fn has_credential(&self, provider: &str) -> bool {
        self.store.read().await.contains_key(provider)
    }

    /// Remove a credential
    pub async fn remove_credential(&self, provider: &str) -> Result<(), VaultError> {
        self.store.write().await.remove(provider);
        Ok(())
    }

    /// List all stored providers (not the actual keys)
    pub async fn list_providers(&self) -> Vec<String> {
        self.store.read().await.keys().cloned().collect()
    }
}

fn token_hash(token: &ScopedToken) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.provider.hash(&mut hasher);
    token.created_at.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Debug, Clone, Error)]
pub enum VaultError {
    #[error("Token expired")]
    TokenExpired,
    #[error("No credential found for provider: {0}")]
    NotFound(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}
