//! provider — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::io::{self, Write};

use super::{err, warn};

pub fn check_provider_config() -> bool {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if cfg.provider.is_some() && cfg.api_key.as_ref().is_some_and(|k| !k.is_empty()) {
        return true;
    }
    // 免费池子模式: default_model 指向 keyless 免费模型 (llm7/pollinations/:free) 时
    // 无需 API key — llm7/codestral-latest 是实测唯一匿名可用流式端点。
    if let Some(ref m) = cfg.default_model {
        if m.starts_with("llm7/")
            || m == "llm7"
            || m.starts_with("pollinations")
            || m.contains(":free")
        {
            return true;
        }
    }
    // LLM 代理池模式: provider_pool.toml 已注册第三方 key 时视为已配置。
    let pool = neotrix::l1_action::nt_io::nt_io_provider::global_provider_pool();
    if let Ok(guard) = pool.lock() {
        if !guard.entries.is_empty() {
            return true;
        }
    }
    false
}

pub fn run_provider_wizard() {
    println!("╔══════════════════════════════════════════╗");
    println!("║  NeoTrix — First-Time Provider Setup    ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("No LLM provider configured yet.");
    println!();

    println!("Available providers:");
    println!("  1) opencode.ai (free tier available)");
    println!("  2) xiaohuxing (OpenAI-compatible proxy)");
    println!("  3) OpenAI");
    println!("  4) Anthropic");
    println!("  5) Custom (OpenAI-compatible)");
    println!();

    let provider = loop {
        print!("Select provider [1-5]: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read stdin; using default provider 'opencode'.");
            break "opencode";
        }
        match input.trim() {
            "1" => break "opencode",
            "2" => break "xiaohuxing",
            "3" => break "openai",
            "4" => break "anthropic",
            "5" => break "custom",
            _ => {
                println!("Invalid selection, try again.");
                continue;
            }
        };
    };

    print!("Enter your API key (or press Enter to skip): ");
    let _ = io::stdout().flush();
    let mut api_key = String::new();
    if io::stdin().read_line(&mut api_key).is_err() {
        eprintln!("Failed to read stdin; skipping API key.");
    }
    let api_key = api_key.trim().to_string();

    let default_model = match provider {
        "opencode" => "opencode/gpt-4o-mini".to_string(),
        "xiaohuxing" => "gpt-4o-mini".to_string(),
        "openai" => "gpt-4o-mini".to_string(),
        "anthropic" => "claude-3-haiku-20240307".to_string(),
        "custom" => {
            print!("Enter default model name: ");
            let _ = io::stdout().flush();
            let mut model = String::new();
            if io::stdin().read_line(&mut model).is_err() {
                eprintln!("Failed to read stdin; using default model.");
                model.push_str("Agents-A1-4B-kimi-Preview-heretic-IQ4_NL");
            }
            model.trim().to_string()
        }
        _ => "Agents-A1-4B-kimi-Preview-heretic-IQ4_NL".to_string(),
    };

    let config_path = neotrix::config::NeoTrixConfig::path();
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "[config] warning: failed to create config directory ({}); continuing",
                e
            );
        }
    }

    let custom_endpoint = match provider {
        "xiaohuxing" => Some("https://api.xiaohuxing.eu.org/v1".to_string()),
        "custom" => {
            print!("Enter custom base URL: ");
            let _ = io::stdout().flush();
            let mut url = String::new();
            if io::stdin().read_line(&mut url).is_err() {
                eprintln!("Failed to read stdin; using default endpoint.");
            }
            let url = url.trim().to_string();
            if url.is_empty() {
                None
            } else {
                Some(url)
            }
        }
        _ => None,
    };

    // 隐私提示: 免费/代理端点靠日志/数据回灌维持免费, 可能将你的代码与对话用于训练。
    // NeoTrix 出网隐私守卫默认开启 (privacy_guard=true, 阻断未信任端点泄露内部指纹),
    // 但仍建议优先使用本地 (Ollama) 或付费签约云端。
    if matches!(provider, "opencode" | "xiaohuxing" | "custom") {
        println!();
        println!(
            "{} 隐私提醒: '{}' 属免费/代理端点, 可能记录并利用你的代码与对话训练模型。",
            warn("⚠"),
            provider
        );
        println!("  NeoTrix 已默认开启出网隐私守卫 (阻断未信任端点泄露内部源码/对话)。");
        println!("  生产建议: 改用本地 Ollama (provider = \"ollama\") 或付费签约云端。");
    }

    // Encrypt the API key before persisting to disk
    // 加密失败即拒绝保存，禁止明文回退 (fail-closed，防密钥落盘可读)
    let stored_key = if !api_key.is_empty() {
        match neotrix::nt_shield::shield_core::key_encryption::encrypt(&api_key) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!(
                    "{}: key encryption failed ({}); refusing to store plaintext key",
                    err("Error"),
                    e
                );
                return;
            }
        }
    } else {
        api_key.clone()
    };

    let mut content = format!(
        "# NeoTrix Configuration\n\
         provider = {:?}\n\
         api_key = {:?}\n\
         default_model = {:?}\n\
         # 出网隐私守卫: 阻止未信任 (免费/代理) 端点获取 NeoTrix 内部源码与对话\n\
         privacy_guard = true\n\
         privacy_block_untrusted = true\n",
        provider, stored_key, default_model,
    );
    if let Some(ref ep) = custom_endpoint {
        content.push_str(&format!("custom_endpoint = {:?}\n", ep));
    }

    if let Err(e) = std::fs::write(&config_path, content) {
        eprintln!(
            "{}: failed to write config file ({}); configuration not saved",
            err("Error"),
            e
        );
        return;
    }
    println!();
    println!("✅ Configuration saved to: {}", config_path.display());
    println!("   Provider: {}", provider);
    if !api_key.is_empty() {
        println!(
            "   API Key: ****{}",
            &api_key[api_key.len().saturating_sub(4)..]
        );
    }
    println!();
    println!("You can change these settings anytime by editing the config file.");
}
