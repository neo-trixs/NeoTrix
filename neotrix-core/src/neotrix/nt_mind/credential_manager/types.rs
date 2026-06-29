use serde::{Deserialize, Serialize};

/// A credential entry in the vault. Passwords stored encrypted at rest.
#[derive(Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub password_nonce: Vec<u8>,
    pub password_ct: Vec<u8>,
    pub notes: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Decrypted credential returned to callers — password is plaintext.
#[derive(Clone, Serialize)]
pub struct DecryptedEntry {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub created_at: u64,
}

impl std::fmt::Debug for DecryptedEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecryptedEntry")
            .field("id", &self.id)
            .field("domain", &self.domain)
            .field("username", &self.username)
            .field("password", &"***REDACTED***")
            .finish_non_exhaustive()
    }
}

/// Credential summary for listing (no plaintext password).
#[derive(Debug, Clone, Serialize)]
pub struct CredentialSummary {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub created_at: u64,
}

pub const MAX_AUDIT_LOG_SIZE: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub domain: Option<String>,
    pub operation: String,
    pub source: String,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordIssue {
    pub entry_id: String,
    pub domain: String,
    pub username: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReusedPasswordGroup {
    pub entries: Vec<PasswordIssue>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordExpiry {
    pub entry_id: String,
    pub domain: String,
    pub username: String,
    pub age_days: u64,
    pub expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHealthReport {
    pub total_entries: usize,
    pub weak_passwords: Vec<PasswordIssue>,
    pub reused_passwords: Vec<ReusedPasswordGroup>,
    pub expired_passwords: Vec<PasswordExpiry>,
    pub overall_score: f64,
}
