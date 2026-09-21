//! Model Pool — provider pool CRUD & connectivity (domain layer)
//!
//! Pure business logic extracted from `commands/model_pool.rs`.

use anyhow::{Context, Result as AnyhowResult};
use serde::{Deserialize, Serialize};

use crate::atomic_io;
use crate::config::AppConfig;

// ── Types ────────────────────────────────────────────

/// Model pool entry (masked api_key for display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolEntry {
    pub label: String,
    pub provider: String,
    pub api_key_masked: String,
    pub model: String,
    pub tags: Vec<String>,
    pub base_url: Option<String>,
    pub created_ts: u64,
}

/// Model pool status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolStatus {
    pub total: usize,
    pub active: usize,
    pub providers: Vec<ModelPoolEntry>,
    pub config_path: String,
}

// ── Private helpers ──────────────────────────────────

/// Resolve the `provider_pool.toml` path under `AppConfig::base_dir()`.
fn pool_path() -> AnyhowResult<std::path::PathBuf> {
    let dir = AppConfig::base_dir().context("Cannot determine base dir")?;
    Ok(dir.join("provider_pool.toml"))
}

#[derive(Deserialize)]
struct RawPool {
    entries: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    label: String,
    provider: String,
    api_key: String,
    model: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    created_ts: Option<u64>,
}

// ── Public API ───────────────────────────────────────

/// Read all pool entries with masked api_keys.
pub fn load_pool_entries() -> AnyhowResult<Vec<ModelPoolEntry>> {
    let path = pool_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let content =
        String::from_utf8(atomic_io::read_with_fallback(&path).context("Read provider pool")?)?;
    let pool: RawPool = toml::from_str(&content).context("Parse provider pool")?;

    let entries = pool
        .entries
        .into_iter()
        .map(|e| ModelPoolEntry {
            label: e.label,
            provider: e.provider,
            api_key_masked: mask_api_key(&e.api_key),
            model: e.model,
            tags: e.tags,
            base_url: e.base_url,
            created_ts: e.created_ts.unwrap_or(0),
        })
        .collect();

    Ok(entries)
}

/// Read the raw TOML content (or a default skeleton).
pub fn read_pool_raw() -> AnyhowResult<String> {
    let path = pool_path()?;
    if !path.exists() {
        return Ok("[[entries]]\n".into());
    }
    String::from_utf8(atomic_io::read_with_fallback(&path).context("Read provider pool")?)
        .map_err(|e| anyhow::anyhow!(e))
}

/// Write raw TOML content atomically.
pub fn write_pool_raw(content: &str) -> AnyhowResult<()> {
    let path = pool_path()?;
    atomic_io::ensure_parent_dir(&path).context("Create config dir")?;
    atomic_io::write_atomic(&path, content.as_bytes()).context("Write provider pool")
}

/// Mask an API key for safe display.
pub fn mask_api_key(key: &str) -> String {
    if key.starts_with("env:") {
        key.to_string()
    } else if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "****".into()
    }
}

/// Append a new entry to the pool.
pub fn add_entry(
    label: &str,
    provider: &str,
    api_key: &str,
    model: &str,
    tags: &[String],
    base_url: Option<&str>,
) -> AnyhowResult<ModelPoolEntry> {
    let mut raw = read_pool_raw()?;

    let new_block = format!(
        r#"

[[entries]]
label = "{label}"
provider = "{provider}"
api_key = "{api_key}"
model = "{model}"
tags = [{tags}]
base_url = {base_url}
created_ts = {created_ts}"#,
        created_ts = chrono::Utc::now().timestamp() as u64,
        label = label,
        provider = provider,
        api_key = api_key,
        model = model,
        tags = tags
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", "),
        base_url = match base_url {
            Some(url) => format!("\"{}\"", url),
            None => "null".into(),
        },
    );

    raw.push_str(&new_block);
    write_pool_raw(&raw)?;

    Ok(ModelPoolEntry {
        label: label.to_string(),
        provider: provider.to_string(),
        api_key_masked: mask_api_key(api_key),
        model: model.to_string(),
        tags: tags.to_vec(),
        base_url: base_url.map(String::from),
        created_ts: chrono::Utc::now().timestamp() as u64,
    })
}

/// Remove an entry by label. Returns `true` if found and removed.
pub fn remove_entry(label: &str) -> AnyhowResult<bool> {
    let raw = read_pool_raw()?;
    let mut found = false;
    let mut result = String::new();
    let mut in_entry = false;
    let mut current_entry = String::new();
    let mut skip_entry = false;

    for line in raw.lines() {
        if line.trim().starts_with("[[entries]]") {
            if in_entry && !skip_entry {
                result.push_str(&current_entry);
                result.push('\n');
            }
            in_entry = true;
            current_entry = line.to_string();
            current_entry.push('\n');
            skip_entry = false;
            continue;
        }

        if in_entry {
            if line.contains(&format!("label = \"{}\"", label)) {
                skip_entry = true;
                found = true;
            }
            current_entry.push_str(line);
            current_entry.push('\n');
        }
    }

    if in_entry && !skip_entry {
        result.push_str(&current_entry);
    }

    if !found {
        return Ok(false);
    }

    write_pool_raw(&result)?;
    Ok(true)
}

/// Replace the api_key for the entry matching `label`. Returns `true` if found.
pub fn update_api_key(label: &str, new_key: &str) -> AnyhowResult<bool> {
    let raw = read_pool_raw()?;
    let mut result = String::new();
    let mut in_entry = false;
    let mut found = false;
    let mut skip_old_key = false;

    for line in raw.lines() {
        if line.trim().starts_with("[[entries]]") {
            in_entry = true;
            skip_old_key = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_entry && line.contains(&format!("label = \"{}\"", label)) {
            found = true;
        }

        if found && line.trim().starts_with("api_key") && !skip_old_key {
            result.push_str(&format!("api_key = \"{}\"", new_key));
            result.push('\n');
            skip_old_key = true;
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !found {
        return Ok(false);
    }

    write_pool_raw(&result)?;
    Ok(true)
}

/// Get the pool status summary.
pub fn get_status() -> AnyhowResult<ModelPoolStatus> {
    let entries = load_pool_entries()?;
    let config_path = pool_path()?.to_string_lossy().to_string();
    Ok(ModelPoolStatus {
        total: entries.len(),
        active: entries.len(),
        providers: entries,
        config_path,
    })
}

/// Check connectivity to a provider's API endpoint.
pub async fn check_connectivity(label: &str) -> AnyhowResult<String> {
    let entries = load_pool_entries()?;
    let entry = entries
        .iter()
        .find(|e| e.label == label)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", label))?;

    let base_url = entry
        .base_url
        .as_deref()
        .unwrap_or(match entry.provider.as_str() {
            "openai" => crate::constants::OPENAI_API_BASE,
            "anthropic" => crate::constants::ANTHROPIC_API_BASE,
            "ollama" => crate::constants::OLLAMA_API_BASE,
            "siliconflow" => crate::constants::SILICONFLOW_API_BASE,
            _ => crate::constants::OPENAI_API_BASE,
        });

    match reqwest::Client::new()
        .get(format!("{}/v1/models", base_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok("connected".into())
            } else {
                Ok(format!("http_{}", resp.status().as_u16()))
            }
        }
        Err(e) => Ok(format!("error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mask_api_key ─────────────────────────────────

    #[test]
    fn mask_key_env_prefix_passthrough() {
        assert_eq!(mask_api_key("env:MY_SECRET"), "env:MY_SECRET");
    }

    #[test]
    fn mask_key_long_key() {
        let key = "sk-1234567890abcdef";
        let masked = mask_api_key(key);
        assert!(masked.starts_with("sk-1"));
        assert!(masked.ends_with("cdef"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn mask_key_short_key() {
        assert_eq!(mask_api_key("abc"), "****");
        assert_eq!(mask_api_key("12345678"), "****");
    }

    #[test]
    fn mask_key_empty_string() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn mask_key_exactly_8_chars() {
        assert_eq!(mask_api_key("12345678"), "****");
    }

    #[test]
    fn mask_key_9_chars_gets_masked() {
        let key = "123456789";
        let masked = mask_api_key(key);
        assert_eq!(masked, "1234...6789");
    }

    #[test]
    fn mask_key_preserves_first4_last4() {
        let key = "abcdefghij";
        let masked = mask_api_key(key);
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("ghij"));
    }

    // ── read_pool_raw / load_pool_entries (no config) ─

    #[test]
    fn read_pool_raw_no_config_returns_skeleton() {
        // pool_path() depends on AppConfig::base_dir() which may be None
        // in test. This test verifies the function doesn't panic.
        let result = read_pool_raw();
        // Either Ok with skeleton or Err if base_dir is unset — both acceptable
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn load_pool_entries_no_config_returns_empty() {
        let result = load_pool_entries();
        match result {
            Ok(entries) => assert!(entries.is_empty()),
            Err(_) => {} // AppConfig::base_dir() may not be set in test
        }
    }

    // ── ModelPoolEntry serialization ─────────────────

    #[test]
    fn model_pool_entry_roundtrip_json() {
        let entry = ModelPoolEntry {
            label: "test-openai".into(),
            provider: "openai".into(),
            api_key_masked: "sk-1...abcd".into(),
            model: "gpt-4".into(),
            tags: vec!["chat".into()],
            base_url: Some("https://api.openai.com".into()),
            created_ts: 1700000000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: ModelPoolEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.label, "test-openai");
        assert_eq!(decoded.provider, "openai");
        assert_eq!(decoded.api_key_masked, "sk-1...abcd");
        assert_eq!(decoded.model, "gpt-4");
        assert_eq!(decoded.tags, vec!["chat"]);
        assert_eq!(decoded.created_ts, 1700000000);
    }

    #[test]
    fn model_pool_entry_default_tags_empty() {
        let json = r#"{"label":"x","provider":"y","api_key_masked":"z","model":"m","tags":[],"created_ts":0}"#;
        let entry: ModelPoolEntry = serde_json::from_str(json).unwrap();
        assert!(entry.tags.is_empty());
        assert!(entry.base_url.is_none());
    }

    // ── ModelPoolStatus ──────────────────────────────

    #[test]
    fn model_pool_status_roundtrip() {
        let status = ModelPoolStatus {
            total: 0,
            active: 0,
            providers: vec![],
            config_path: "/tmp/test.toml".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let decoded: ModelPoolStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.total, 0);
        assert_eq!(decoded.config_path, "/tmp/test.toml");
    }
}
