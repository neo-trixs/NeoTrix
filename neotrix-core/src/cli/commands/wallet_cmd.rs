//! Wallet command — /wallet
//!
//! Subcommands:
//!   /wallet create <label>         Create a new wallet
//!   /wallet import <label> <pk>    Import private key
//!   /wallet list                   List all wallets
//!   /wallet use <label>            Switch active wallet
//!   /wallet balance [chain]        Query balance
//!   /wallet export <label>         Export private key
//!   /wallet delete <label>         Delete wallet

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_act_crypto::CryptoAgent;

fn with_crypto<F>(f: F) -> CommandOutput
where
    F: FnOnce(&mut CryptoAgent) -> CommandOutput,
{
    let mut crypto = CryptoAgent::new();
    let _ = crypto.load_persisted_wallets();
    f(&mut crypto)
}

pub struct WalletCmd;

impl CliCommand for WalletCmd {
    fn name(&self) -> &str {
        "/wallet"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/w"]
    }

    fn description(&self) -> &str {
        "Wallet management: create | import | list | use | balance | export | delete"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain: Vec<&str> = args.iter().map(|s| s.as_str()).filter(|a| *a != "--json").collect();

        if plain.is_empty() {
            return CommandOutput::ok(
                "Usage: /wallet <subcommand> [args]\n\
                 Subcommands:\n  \
                 create <label>    创建新钱包\n  \
                 import <label> <pk>  导入私钥\n  \
                 list              列出所有钱包\n  \
                 use <label>       切换活跃钱包\n  \
                 balance [chain]   查询余额\n  \
                 export <label>    导出私钥\n  \
                 delete <label>    删除钱包",
            );
        }

        match plain[0] {
            "create" | "new" => {
                let label = plain.get(1).map(|s| *s).unwrap_or("default");
                let result = with_crypto(|crypto| {
                    match crypto.persist_wallet(label) {
                        Ok(label) => {
                            let w = crypto.wallet_manager.active_wallet().expect("active wallet must exist after persist");
                            CommandOutput::ok(&format!(
                                "✅ 创建钱包成功\n   Label: {}\n   Address: {}\n   📁 Saved to: {:?}",
                                label, w.address_short(), crypto.wallet_store.dir_path()
                            )).with_json(serde_json::json!({
                                "action": "create", "label": label,
                                "address": w.address, "chain": "evm"
                            }))
                        }
                        Err(e) => CommandOutput::err(&format!("创建失败: {}", e)),
                    }
                });
                if want_json { result } else { result }
            }

            "import" | "i" => {
                if plain.len() < 3 {
                    return CommandOutput::err("Usage: /wallet import <label> <private_key>");
                }
                let label = plain[1];
                let pk = plain[2];
                with_crypto(|crypto| match crypto.import_wallet(pk, label) {
                    Ok(w) => {
                        let msg = format!(
                            "✅ 导入钱包成功\n   Label: {}\n   Address: {}",
                            w.label, w.address_short()
                        );
                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "action": "import", "label": w.label,
                                "address": w.address, "chain": "evm"
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("导入失败: {}", e)),
                })
            }

            "list" | "ls" | "l" => {
                with_crypto(|crypto| {
                    let wallets = crypto.wallet_store.list_wallets().unwrap_or_default();
                    if wallets.is_empty() {
                        return CommandOutput::ok("📭 没有钱包。使用 /wallet create <label> 创建");
                    }

                    let mut lines = vec![format!("📒 钱包列表 (共{}个):", wallets.len())];
                    for (i, w) in wallets.iter().enumerate() {
                        let active = crypto.wallet_manager.active_wallet()
                            .map(|a| a.address == w.address).unwrap_or(false);
                        let marker = if active { " →" } else { "  " };
                        lines.push(format!(
                            "{}. {} [{}] {} {}",
                            i + 1,
                            marker,
                            w.chain,
                            w.label,
                            short_addr(&w.address)
                        ));
                    }
                    let out = CommandOutput::ok(&lines.join("\n"));
                    if want_json {
                        let json_list: Vec<serde_json::Value> = wallets.iter().map(|w| {
                            serde_json::json!({
                                "label": w.label, "address": w.address,
                                "chain": w.chain, "created": w.created_at
                            })
                        }).collect();
                        out.with_json(serde_json::json!({"wallets": json_list}))
                    } else {
                        out
                    }
                })
            }

            "use" | "switch" | "select" => {
                if plain.len() < 2 {
                    return CommandOutput::err("Usage: /wallet use <label>");
                }
                let label = plain[1];
                with_crypto(|crypto| {
                    let wallets = crypto.wallet_store.list_wallets().unwrap_or_default();
                    let idx = wallets.iter().position(|w| w.label == label);
                    match idx {
                        Some(i) => {
                            crypto.load_persisted_wallets().ok();
                            let _ = crypto.wallet_manager.set_active(i);
                            CommandOutput::ok(&format!("✅ 切换到钱包: {} ({})", label, wallets[i].address))
                        }
                        None => CommandOutput::err(&format!("❌ 未找到钱包: {}", label)),
                    }
                })
            }

            "balance" | "bal" | "b" => {
                let chain_name = plain.get(1).map(|s| *s);
                with_crypto(|crypto| {
                    let addr = match crypto.wallet_manager.active_wallet() {
                        Some(w) => w.address.clone(),
                        None => return CommandOutput::err("❌ 没有活跃钱包，请先创建或导入"),
                    };

                    let mut lines = vec![format!("💰 {} 余额:", short_addr(&addr))];
                    let mut total_usd = 0.0;

                    if let Some(name) = chain_name {
                        let chain = chain_from_name(name);
                        let result = crypto.scan_specific_chain(&chain, &addr);
                        for opp in &result {
                            lines.push(format!(
                                "  {}: {:.6} (${:.2})",
                                chain, opp.estimated_value_usd, opp.estimated_value_usd
                            ));
                            total_usd += opp.estimated_value_usd;
                        }
                    } else {
                        let balances = crypto.check_all_balances();
                        for (chain, bal) in &balances {
                            lines.push(format!("  {}: {:.6}", chain, bal));
                            total_usd += bal;
                        }
                        if balances.is_empty() {
                            lines.push("  (无法查询到余额或钱包为空)".into());
                        }
                    }

                    lines.push(format!("  合计估值: ${:.2}", total_usd));
                    let out = CommandOutput::ok(&lines.join("\n"));
                    if want_json {
                        out.with_json(serde_json::json!({
                            "address": addr, "total_usd": total_usd,
                            "chains": {}
                        }))
                    } else {
                        out
                    }
                })
            }

            "export" | "e" => {
                let label = plain.get(1).map(|s| *s).unwrap_or("");
                if label.is_empty() {
                    let w = CryptoAgent::new().wallet_manager.active_wallet().cloned();
                    return match w {
                        Some(_) => with_crypto(|crypto| {
                            if let Some(w) = crypto.wallet_manager.active_wallet() {
                                let msg = format!(
                                    "⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!\n\n🔑 {} 私钥:\n{}",
                                    w.label,
                                    w.private_key_hex()
                                );
                                CommandOutput::warn(&msg)
                            } else {
                                CommandOutput::err("no active wallet")
                            }
                        }),
                        None => CommandOutput::err("Usage: /wallet export <label>"),
                    };
                }
                with_crypto(|crypto| {
                    match crypto.wallet_store.load_wallet(label) {
                        Ok(w) => {
                            let msg = format!(
                                "⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!\n\n🔑 {} 私钥:\n{}",
                                w.label, w.private_key_hex()
                            );
                            CommandOutput::warn(&msg)
                        }
                        Err(e) => CommandOutput::err(&format!("导出失败: {}", e)),
                    }
                })
            }

            "delete" | "rm" => {
                if plain.len() < 2 {
                    return CommandOutput::err("Usage: /wallet delete <label>");
                }
                let label = plain[1];
                with_crypto(|crypto| match crypto.delete_persisted_wallet(label) {
                    Ok(_) => CommandOutput::ok(&format!("🗑️ 已删除钱包: {}", label)),
                    Err(e) => CommandOutput::err(&format!("删除失败: {}", e)),
                })
            }

            other => CommandOutput::err(&format!(
                "未知子命令: {}\n可用: create, import, list, use, balance, export, delete",
                other
            )),
        }
    }
}

fn short_addr(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

fn chain_from_name(name: &str) -> crate::neotrix::nt_act_crypto::ChainType {
    match name.to_lowercase().as_str() {
        "eth" | "ethereum" => crate::neotrix::nt_act_crypto::ChainType::Ethereum,
        "bsc" | "bnb" => crate::neotrix::nt_act_crypto::ChainType::Bsc,
        "polygon" | "matic" => crate::neotrix::nt_act_crypto::ChainType::Polygon,
        "arb" | "arbitrum" => crate::neotrix::nt_act_crypto::ChainType::Arbitrum,
        "opt" | "optimism" | "op" => crate::neotrix::nt_act_crypto::ChainType::Optimism,
        "base" => crate::neotrix::nt_act_crypto::ChainType::Base,
        "avax" | "avalanche" => crate::neotrix::nt_act_crypto::ChainType::Avalanche,
        _ => crate::neotrix::nt_act_crypto::ChainType::Ethereum,
    }
}
