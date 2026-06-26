use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

pub(crate) mod key;
#[cfg(test)]
mod tests;
pub mod types;

// ── Re-exports ──────────────────────────────────────────────────────────────

pub use key::{derive_key_from_password, load_or_generate_master_key};
pub use types::{
    AuditEntry, CredentialEntry, DecryptedEntry, PasswordExpiry, PasswordHealthReport,
    PasswordIssue, ReusedPasswordGroup, MAX_AUDIT_LOG_SIZE,
};

// ── CredentialManager ───────────────────────────────────────────────────────

/// Encrypted in-memory credential store.
///
/// Each password is individually AES-256-GCM encrypted within `CredentialEntry`.
/// The `find()` and `auto_fill_script()` methods transparently decrypt on access.
/// Serialization to disk preserves encrypted blobs and requires the same key to load.
pub struct CredentialManager {
    entries: Vec<CredentialEntry>,
    by_domain: HashMap<String, Vec<usize>>,
    next_id: u64,
    cipher: Aes256Gcm,
    audit_log: VecDeque<AuditEntry>,
}

impl CredentialManager {
    /// Create with a random key (useful for testing / transient use).
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        Self::with_key(key)
    }

    /// Create with an explicit 32-byte encryption key.
    pub fn with_key(key: [u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES-256 key construction failed - key must be exactly 32 bytes");
        Self {
            entries: Vec::new(),
            by_domain: HashMap::new(),
            next_id: 0,
            cipher,
            audit_log: VecDeque::new(),
        }
    }

    /// Create using `load_or_generate_master_key()` logic.
    pub fn auto() -> NeoTrixResult<Self> {
        let key = load_or_generate_master_key()?;
        Ok(Self::with_key(key))
    }

    /// Create with a key derived from a master password (Argon2id).
    pub fn with_master_password(password: &str) -> NeoTrixResult<Self> {
        let mut salt = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        let key = derive_key_from_password(password, &salt);
        Ok(Self::with_key(key))
    }

    /// Store a credential. Password is encrypted before storage.
    pub fn store(
        &mut self,
        domain: &str,
        username: &str,
        password: &str,
        notes: &str,
    ) -> CredentialEntry {
        let (nonce, ct) = self.encrypt_password(password);
        let id = format!("cred-{}", self.next_id);
        self.next_id += 1;
        let entry = CredentialEntry {
            id,
            domain: domain.to_lowercase(),
            username: username.to_string(),
            password_nonce: nonce,
            password_ct: ct,
            notes: notes.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: 0,
        };
        let idx = self.entries.len();
        let domain_lower = entry.domain.clone();
        self.by_domain
            .entry(domain_lower.clone())
            .or_default()
            .push(idx);
        self.entries.push(entry.clone());
        self.record_audit(
            Some(domain_lower),
            "store",
            "internal",
            true,
            &format!("username={}", username),
        );
        entry
    }

    /// Find credentials for a domain. Returns decrypted entries.
    pub fn find(&mut self, domain: &str) -> Vec<DecryptedEntry> {
        let key = domain.to_lowercase();
        let result: Vec<_> = self
            .by_domain
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.entries.get(i))
                    .filter_map(|e| {
                        let password = self
                            .decrypt_password(&e.password_nonce, &e.password_ct)
                            .ok()?;
                        Some(DecryptedEntry {
                            id: e.id.clone(),
                            domain: e.domain.clone(),
                            username: e.username.clone(),
                            password,
                            notes: e.notes.clone(),
                            created_at: e.created_at,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.record_audit(
            Some(key),
            "find",
            "internal",
            true,
            &format!("entry_count={}", result.len()),
        );
        result
    }

    /// Return all entries (encrypted — password not exposed).
    pub fn all(&self) -> &[CredentialEntry] {
        &self.entries
    }

    /// Remove a credential by id.
    pub fn remove(&mut self, id: &str) -> bool {
        let pos = match self.entries.iter().position(|e| e.id == id) {
            Some(p) => p,
            None => return false,
        };
        let domain = self.entries[pos].domain.clone();
        self.entries.remove(pos);
        let mut new_by_domain: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, entry) in self.entries.iter().enumerate() {
            new_by_domain
                .entry(entry.domain.clone())
                .or_default()
                .push(i);
        }
        self.by_domain = new_by_domain;
        self.record_audit(
            Some(domain),
            "remove",
            "internal",
            true,
            &format!("id={}", id),
        );
        true
    }

    /// Generate an autofill JavaScript snippet for the first matched domain credential.
    pub fn auto_fill_script(&mut self, domain: &str) -> Option<String> {
        let entries = self.find(domain);
        if entries.is_empty() {
            return None;
        }
        let entry = &entries[0];
        let escaped_user = entry.username.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_pass = entry.password.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            r#"
(function() {{
    var flds = document.querySelectorAll('input[type="email"], input[type="text"][name*="user"], input[type="text"][name*="email"], input[name="login"], input[name="username"]');
    var pass = document.querySelectorAll('input[type="password"]');
    if (flds.length > 0) {{
        flds[0].value = '{}';
        flds[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
    }}
    if (pass.length > 0) {{
        pass[0].value = '{}';
        pass[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
    }}
}})();
"#,
            escaped_user, escaped_pass,
        );
        Some(script)
    }

    // ── Audit Log ──────────────────────────────────────────────────────────

    /// Record an access audit entry. Trims the log if it exceeds MAX_AUDIT_LOG_SIZE.
    pub fn record_audit(
        &mut self,
        domain: Option<String>,
        operation: &str,
        source: &str,
        success: bool,
        details: &str,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.audit_log.push_back(AuditEntry {
            timestamp,
            domain,
            operation: operation.to_string(),
            source: source.to_string(),
            success,
            details: details.to_string(),
        });
        if self.audit_log.len() > MAX_AUDIT_LOG_SIZE {
            self.audit_log.pop_front();
        }
    }

    /// Iterate over all audit entries, newest last.
    pub fn audit_log(&self) -> impl Iterator<Item = &AuditEntry> {
        self.audit_log.iter()
    }

    /// Filter audit entries by domain (case-insensitive exact match).
    pub fn audit_log_by_domain(&self, domain: &str) -> Vec<&AuditEntry> {
        let key = domain.to_lowercase();
        self.audit_log
            .iter()
            .filter(|e| e.domain.as_deref() == Some(&key))
            .collect()
    }

    /// Filter audit entries with timestamp >= since.
    pub fn audit_log_since(&self, since: u64) -> Vec<&AuditEntry> {
        self.audit_log
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Clear all audit entries.
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }

    // ── Health check ───────────────────────────────────────────────────────

    /// Run a password health check across all stored credentials.
    ///
    /// Checks each password for weakness (length < 8, missing digits,
    /// missing special characters, common patterns), detects password reuse
    /// across entries, and flags passwords older than 180 days.
    /// Returns an overall score from 0.0 (worst) to 1.0 (best).
    pub fn health_check(&self) -> PasswordHealthReport {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let common_patterns = [
            "password", "123456", "qwerty", "admin", "letmein", "welcome",
        ];
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";

        let mut weak_passwords: Vec<PasswordIssue> = Vec::new();
        let mut password_map: HashMap<String, Vec<PasswordIssue>> = HashMap::new();

        for entry in &self.entries {
            let password = match self.decrypt_password(&entry.password_nonce, &entry.password_ct) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let mut reasons: Vec<&str> = Vec::new();

            if password.len() < 8 {
                reasons.push("too_short");
            }
            if !password.chars().any(|c| c.is_ascii_digit()) {
                reasons.push("no_number");
            }
            if !password.chars().any(|c| special_chars.contains(c)) {
                reasons.push("no_special");
            }
            let lower = password.to_lowercase();
            if common_patterns.iter().any(|&p| lower.contains(p)) {
                reasons.push("common_pattern");
            }

            let issue = PasswordIssue {
                entry_id: entry.id.clone(),
                domain: entry.domain.clone(),
                username: entry.username.clone(),
                reason: reasons.join(", "),
            };

            if !reasons.is_empty() {
                weak_passwords.push(issue.clone());
            }

            password_map.entry(password).or_default().push(issue);
        }

        let mut reused_passwords: Vec<ReusedPasswordGroup> = Vec::new();
        for (_pw, issues) in password_map.iter() {
            if issues.len() > 1 {
                reused_passwords.push(ReusedPasswordGroup {
                    entries: issues.clone(),
                    count: issues.len(),
                });
            }
        }

        let mut expired_passwords: Vec<PasswordExpiry> = Vec::new();
        for entry in &self.entries {
            let age_days = now.saturating_sub(entry.created_at) / 86400;
            if age_days >= 180 {
                expired_passwords.push(PasswordExpiry {
                    entry_id: entry.id.clone(),
                    domain: entry.domain.clone(),
                    username: entry.username.clone(),
                    age_days,
                    expired: true,
                });
            }
        }

        let mut score = 1.0;
        score -= 0.15 * weak_passwords.len() as f64;
        score -= 0.10 * reused_passwords.len() as f64;
        score -= 0.05 * expired_passwords.len() as f64;
        let overall_score = score.max(0.0);

        PasswordHealthReport {
            total_entries: self.entries.len(),
            weak_passwords,
            reused_passwords,
            expired_passwords,
            overall_score,
        }
    }

    // ── Persistence ────────────────────────────────────────────────────────

    /// Save all entries to a JSON file (per-entry passwords remain encrypted).
    pub fn save_to(&self, path: &Path) -> NeoTrixResult<()> {
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        }
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &json).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        std::fs::rename(&tmp_path, path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        Ok(())
    }

    /// Load entries from a JSON file saved by `save_to()`.
    pub fn load_from(path: &Path, key: [u8; 32]) -> NeoTrixResult<Self> {
        let json =
            std::fs::read_to_string(path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        let entries: Vec<CredentialEntry> =
            serde_json::from_str(&json).map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        let next_id = entries
            .iter()
            .filter_map(|e| {
                e.id.strip_prefix("cred-")
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(0);

        let mut by_domain: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, entry) in entries.iter().enumerate() {
            by_domain.entry(entry.domain.clone()).or_default().push(i);
        }

        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid AES-256 key");
        Ok(Self {
            entries,
            by_domain,
            next_id,
            cipher,
            audit_log: VecDeque::new(),
        })
    }

    /// Convenience: save to default path `~/.neotrix/credentials.json`.
    pub fn save_default(&self) -> NeoTrixResult<()> {
        let path = key::default_cred_path();
        self.save_to(&path)
    }

    /// Convenience: load from default path `~/.neotrix/credentials.json`.
    pub fn load_default(key: [u8; 32]) -> Self {
        let path = key::default_cred_path();
        if path.exists() {
            Self::load_from(&path, key).unwrap_or_else(|_| Self::with_key(key))
        } else {
            Self::with_key(key)
        }
    }

    // ── Internal crypto ────────────────────────────────────────────────────

    fn encrypt_password(&self, password: &str) -> (Vec<u8>, Vec<u8>) {
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self
            .cipher
            .encrypt(nonce, password.as_bytes())
            .expect("AES-256-GCM encryption should not fail with valid key");
        (nonce_bytes.to_vec(), ct)
    }

    fn decrypt_password(&self, nonce: &[u8], ct: &[u8]) -> NeoTrixResult<String> {
        let nonce = Nonce::from_slice(nonce);
        let plaintext = self
            .cipher
            .decrypt(nonce, ct)
            .map_err(|_| NeoTrixError::General {
                msg: "password decryption failed (wrong key or tampered data)".into(),
                backtrace: None,
            })?;
        String::from_utf8(plaintext).map_err(|e| NeoTrixError::Serde(e.to_string()))
    }

    /// Re-encrypt all stored credentials with a new encryption key.
    ///
    /// Decrypts each entry with the current cipher, then re-encrypts
    /// with a new cipher created from `new_key`. Replaces `self.cipher`
    /// in-place. All entries remain valid after this operation.
    pub fn re_encrypt(&mut self, new_key: [u8; 32]) -> NeoTrixResult<()> {
        let new_cipher = Aes256Gcm::new_from_slice(&new_key).expect("valid AES-256 key");
        for entry in self.entries.iter_mut() {
            let nonce = Nonce::from_slice(&entry.password_nonce);
            let plaintext = self
                .cipher
                .decrypt(nonce, &*entry.password_ct)
                .map_err(|_| NeoTrixError::General {
                    msg: "password decryption failed during re_encrypt".into(),
                    backtrace: None,
                })?;
            let mut new_nonce_bytes = [0u8; 12];
            rand::rngs::OsRng.fill_bytes(&mut new_nonce_bytes);
            let new_nonce = Nonce::from_slice(&new_nonce_bytes);
            let ct = new_cipher
                .encrypt(new_nonce, plaintext.as_ref())
                .expect("AES-256-GCM encryption should not fail with valid key");
            entry.password_nonce = new_nonce_bytes.to_vec();
            entry.password_ct = ct;
        }
        self.cipher = new_cipher;
        Ok(())
    }

    /// Convenience: generate a new random key and re-encrypt all entries.
    pub fn rotate_key(&mut self) -> NeoTrixResult<()> {
        let mut new_key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut new_key);
        self.re_encrypt(new_key)
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}
