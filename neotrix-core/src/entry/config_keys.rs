//! config_keys — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::{err, info, success, warn};

pub fn run_config_encrypt_keys() {
    use neotrix::l3_embodiment::nt_shield::shield_core::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!(
            "{} No config file found at {}",
            err("Error:"),
            config_path.display()
        );
        return;
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to read config: {}", err("Error:"), e);
            return;
        }
    };
    let mut cfg: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} Failed to parse config: {}", err("Error:"), e);
            return;
        }
    };
    let mut changed = false;
    if let Some(table) = cfg.as_table_mut() {
        let keys_to_encrypt: Vec<String> = table
            .iter()
            .filter(|(k, v)| {
                let k_lower = k.to_lowercase();
                (k_lower.contains("api_key")
                    || k_lower.contains("apikey")
                    || k_lower.contains("secret"))
                    && v.is_str()
                    && !key_encryption::is_encrypted(v.as_str().unwrap_or_default())
            })
            .map(|(k, _)| k.clone())
            .collect();
        for key in &keys_to_encrypt {
            if let Some(toml::Value::String(plain)) = table.remove(key) {
                if plain.is_empty() {
                    table.insert(key.clone(), toml::Value::String(plain));
                    continue;
                }
                match key_encryption::encrypt(&plain) {
                    Ok(enc) => {
                        table.insert(key.clone(), toml::Value::String(enc));
                        println!("  {} Encrypted '{}'", success("✓"), key);
                        changed = true;
                    }
                    Err(e) => {
                        eprintln!("  {} Failed to encrypt '{}': {}", err("✗"), key, e);
                        table.insert(key.clone(), toml::Value::String(plain));
                    }
                }
            }
        }
    }
    if !changed {
        println!(
            "  {} No plaintext API keys or secrets found in config",
            info("ℹ")
        );
        return;
    }
    let output = toml::to_string_pretty(&cfg).unwrap_or(content);
    if let Err(e) = std::fs::write(&config_path, &output) {
        eprintln!("{} Failed to write config: {}", err("Error:"), e);
        return;
    }
    println!(
        "  {} Config written to {}",
        success("✓"),
        config_path.display()
    );
}

pub fn run_config_decrypt_keys() {
    use neotrix::l3_embodiment::nt_shield::shield_core::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!(
            "{} No config file found at {}",
            err("Error:"),
            config_path.display()
        );
        return;
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to read config: {}", err("Error:"), e);
            return;
        }
    };
    let mut cfg: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} Failed to parse config: {}", err("Error:"), e);
            return;
        }
    };
    let mut changed = false;
    if let Some(table) = cfg.as_table_mut() {
        let keys_to_decrypt: Vec<String> = table
            .iter()
            .filter(|(_, v)| {
                v.is_str() && key_encryption::is_encrypted(v.as_str().unwrap_or_default())
            })
            .map(|(k, _)| k.clone())
            .collect();
        for key in &keys_to_decrypt {
            if let Some(toml::Value::String(enc)) = table.remove(key) {
                match key_encryption::decrypt(&enc) {
                    Ok(plain) => {
                        table.insert(key.clone(), toml::Value::String(plain));
                        println!("  {} Decrypted '{}'", warn("⚠"), key);
                        changed = true;
                    }
                    Err(e) => {
                        eprintln!("  {} Failed to decrypt '{}': {}", err("✗"), key, e);
                        table.insert(key.clone(), toml::Value::String(enc));
                    }
                }
            }
        }
    }
    if !changed {
        println!("  {} No encrypted values found in config", info("ℹ"));
        return;
    }
    let output = toml::to_string_pretty(&cfg).unwrap_or(content);
    if let Err(e) = std::fs::write(&config_path, &output) {
        eprintln!("{} Failed to write config: {}", err("Error:"), e);
        return;
    }
    println!(
        "{} API keys are now stored in plaintext. Consider re-encrypting with `neotrix config encrypt-keys`.",
        warn("⚠")
    );
    println!(
        "  {} Config written to {}",
        success("✓"),
        config_path.display()
    );
}

// ── Wallet commands ──
