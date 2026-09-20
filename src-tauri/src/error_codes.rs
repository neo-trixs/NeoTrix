//! Centralized error code constants.
//! All error codes must be defined here — no magic strings in business logic.

// ── I/O ──
pub const IO_ERROR: &str = "IO_ERROR";
pub const ATOMIC_WRITE: &str = "ATOMIC_WRITE";
pub const ATOMIC_SYNC: &str = "ATOMIC_SYNC";
pub const ATOMIC_RENAME: &str = "ATOMIC_RENAME";
pub const ATOMIC_BACKUP: &str = "ATOMIC_BACKUP";
pub const ATOMIC_READ_NOT_FOUND: &str = "ATOMIC_READ_NOT_FOUND";
pub const ATOMIC_DESERIALIZE: &str = "ATOMIC_DESERIALIZE";

// ── Serialization ──
pub const SERDE_ERROR: &str = "SERDE_ERROR";
pub const TOML_PARSE_ERROR: &str = "TOML_PARSE_ERROR";
pub const ATOMIC_SERIALIZE: &str = "ATOMIC_SERIALIZE";

// ── Database ──
pub const DB_ERROR: &str = "DB_ERROR";

// ── Auth ──
pub const TOKEN_EXPIRED: &str = "TOKEN_EXPIRED";
pub const ENGINE_AUTH_FAILED: &str = "ENGINE_AUTH_FAILED";

// ── Not Found ──
pub const CREDENTIAL_NOT_FOUND: &str = "CREDENTIAL_NOT_FOUND";
pub const BOT_NOT_FOUND: &str = "BOT_NOT_FOUND";
pub const BROWSER_WINDOW_NOT_FOUND: &str = "BROWSER_WINDOW_NOT_FOUND";
pub const PROVIDER_NOT_FOUND: &str = "PROVIDER_NOT_FOUND";
pub const PROXY_NOT_FOUND: &str = "PROXY_NOT_FOUND";
pub const IM_CHANNEL_NOT_FOUND: &str = "IM_CHANNEL_NOT_FOUND";

// ── Invalid Input ──
pub const BOT_INVALID_INPUT: &str = "BOT_INVALID_INPUT";
pub const ENGINE_CONTEXT_TOO_LONG: &str = "ENGINE_CONTEXT_TOO_LONG";
pub const BROWSER_INVALID_URL: &str = "BROWSER_INVALID_URL";
pub const CONFIG_VALIDATION: &str = "CONFIG_VALIDATION";

// ── Duplicate ──
pub const BOT_DUPLICATE: &str = "BOT_DUPLICATE";

// ── Rate Limited ──
pub const ENGINE_RATE_LIMITED: &str = "ENGINE_RATE_LIMITED";

// ── Network ──
pub const ENGINE_NETWORK_ERROR: &str = "ENGINE_NETWORK_ERROR";
pub const BROWSER_NAVIGATION: &str = "BROWSER_NAVIGATION";
pub const BROWSER_NETWORK: &str = "BROWSER_NETWORK";

// ── Timeout ──
pub const ENGINE_TIMEOUT: &str = "ENGINE_TIMEOUT";

// ── Platform ──
pub const NOTIFICATION_PLATFORM: &str = "NOTIFICATION_PLATFORM";
pub const BROWSER_WINDOW_CREATE: &str = "BROWSER_WINDOW_CREATE";
pub const NOTIFICATION_PERMISSION: &str = "NOTIFICATION_PERMISSION";

// ── Engine ──
pub const ENGINE_PROVIDER_ERROR: &str = "ENGINE_PROVIDER_ERROR";
pub const ENGINE_CONTENT_FILTERED: &str = "ENGINE_CONTENT_FILTERED";
pub const ENGINE_UNKNOWN: &str = "ENGINE_UNKNOWN";
pub const BROWSER_JS_EVAL: &str = "BROWSER_JS_EVAL";

// ── Config ──
pub const CONFIG_FILE_READ: &str = "CONFIG_FILE_READ";
pub const CONFIG_PARSE: &str = "CONFIG_PARSE";

// ── Crypto ──
pub const ENCRYPTION_ERROR: &str = "ENCRYPTION_ERROR";

// ── Domain (catch-all) ──
pub const DOMAIN_ERROR: &str = "DOMAIN_ERROR";
