//! wallet — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::{err, info, success, warn};

pub fn run_wallet_create(label: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.persist_wallet(label) {
        Ok(lbl) => {
            if let Some(w) = crypto.wallet_manager.active_wallet() {
                println!("{}", success("Wallet created successfully"));
                println!("  Label:   {}", lbl);
                println!("  Address: {}", w.address);
                println!("  Path:    {:?}", crypto.wallet_store.dir_path());
            }
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_import(label: &str, private_key: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.import_wallet(private_key, label) {
        Ok(w) => {
            println!("{}", success("Wallet imported successfully"));
            println!("  Label:   {}", w.label);
            println!("  Address: {}", w.address);
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_list(json: bool) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.list_wallets() {
        Ok(wallets) => {
            if json {
                let list: Vec<serde_json::Value> = wallets
                    .iter()
                    .map(|w| {
                        serde_json::json!({
                            "label": w.label, "address": w.address,
                            "chain": w.chain, "created": w.created_at
                        })
                    })
                    .collect();
                match serde_json::to_string_pretty(&serde_json::json!({"wallets": list})) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("{}: JSON serialization failed: {}", err("Error"), e),
                }
            } else if wallets.is_empty() {
                println!(
                    "  {} No wallets found. Use {} to create one.",
                    info("ℹ"),
                    info("neotrix wallet create <label>")
                );
            } else {
                println!("  {} Wallets ({})", success("✓"), wallets.len());
                for w in &wallets {
                    let addr_short = if w.address.len() > 12 {
                        format!(
                            "{}...{}",
                            &w.address[..6],
                            &w.address[w.address.len() - 4..]
                        )
                    } else {
                        w.address.clone()
                    };
                    println!("    • {} [{}] {}", w.label, w.chain, addr_short);
                }
            }
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_balance(chain: &str) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    let addr = match crypto.wallet_manager.active_wallet() {
        Some(w) => w.address.clone(),
        None => {
            eprintln!(
                "{} No active wallet. Create or import one first.",
                err("Error:")
            );
            return;
        }
    };
    println!(
        "  {} Checking balance of {} on {}",
        info("ℹ"),
        &addr[..10],
        chain
    );
}

pub fn run_wallet_delete(label: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.delete_persisted_wallet(label) {
        Ok(_) => println!("{} Wallet '{}' deleted", success("✓"), label),
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_export(label: &str) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.load_wallet(label) {
        Ok(w) => {
            println!(
                "{}",
                warn("⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!")
            );
            println!();
            println!("🔑 {} 私钥:", w.label);
            println!("{}", w.private_key_hex());
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}
