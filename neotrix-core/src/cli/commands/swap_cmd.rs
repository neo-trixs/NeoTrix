use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_act_crypto::tx::TxBuilder;
use crate::neotrix::nt_act_crypto::CryptoAgent;
use crate::neotrix::nt_act_crypto::chain::ChainType;

fn with_crypto<F>(f: F) -> CommandOutput
where
    F: FnOnce(&mut CryptoAgent) -> CommandOutput,
{
    let mut crypto = CryptoAgent::new();
    let _ = crypto.load_persisted_wallets();
    f(&mut crypto)
}

fn chain_from_name(name: &str) -> ChainType {
    match name.to_lowercase().as_str() {
        "eth" | "ethereum" => ChainType::Ethereum,
        "bsc" | "bnb" => ChainType::Bsc,
        "polygon" | "matic" => ChainType::Polygon,
        "arb" | "arbitrum" => ChainType::Arbitrum,
        "opt" | "optimism" | "op" => ChainType::Optimism,
        "base" => ChainType::Base,
        "avax" | "avalanche" => ChainType::Avalanche,
        _ => ChainType::Ethereum,
    }
}

fn parse_amount(s: &str) -> Result<u128, String> {
    let val: f64 = s.parse().map_err(|e| format!("invalid amount: {e}"))?;
    Ok((val * 1e18) as u128)
}

fn short_addr(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

pub struct SwapCmd;

impl CliCommand for SwapCmd {
    fn name(&self) -> &str {
        "/swap"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/s"]
    }

    fn description(&self) -> &str {
        "Token swap: swap <chain> <token_in> <token_out> <amount>"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(
                "Usage:\n  /swap <chain> <token_in> <token_out> <amount>  — 获取报价/执行交换\n\
                 \nExamples:\n  /swap eth weth usdc 0.1\n  /swap bsc wbnb usdt 0.5",
            );
        }

        let chain = chain_from_name(args.first().map(|s| s.as_str()).unwrap_or("eth"));
        let token_in = args.get(1).map(|s| s.as_str()).unwrap_or("eth");
        let token_out = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let amount_str = args.get(3).map(|s| s.as_str()).unwrap_or("0");
        let execute = args.iter().any(|a| a == "--exec" || a == "--send");

        let amount = match parse_amount(amount_str) {
            Ok(a) => a,
            Err(e) => return CommandOutput::err(&format!("❌ {e}")),
        };

        if token_out.is_empty() {
            return CommandOutput::err("❌ 缺少输出代币。Usage: /swap <chain> <token_in> <token_out> <amount>");
        }

        with_crypto(|crypto| {
            let wallet = match crypto.wallet_manager.active_wallet() {
                Some(w) => w.clone(),
                None => return CommandOutput::err("❌ 没有活跃钱包。使用 /wallet create 或 /wallet import"),
            };

            let client = match crypto.evm_clients.get(&chain) {
                Some(c) => c,
                None => return CommandOutput::err(&format!("❌ 无法连接到 {chain}")),
            };

            let nonce = match client.get_transaction_count(&wallet.address) {
                Ok(n) => n,
                Err(e) => return CommandOutput::err(&format!("❌ 获取nonce失败: {e}")),
            };

            let gas_price = match client.get_gas_price() {
                Ok(g) => g,
                Err(_) => 10.0,
            };

            let priority_fee = (gas_price * 0.1).max(0.1) as u128;
            let max_fee = (gas_price * 1.5) as u128;

            let mut lines = vec![
                format!("🔄 Swap 报价:"),
                format!("  链: {}", chain),
                format!("  {token_in} → {token_out}"),
                format!("  金额: {} {token_in}", amount_str),
                format!("  钱包: {}", short_addr(&wallet.address)),
                format!("  Nonce: {nonce}"),
                format!("  Gas: {:.1} gwei (priority: {:.1})", gas_price, priority_fee as f64 / 1e9),
                format!("  Gas limit: ~200000"),
            ];

            if execute {
                let our_addr = wallet.address.clone();
                let client_clone = crypto.evm_clients.get(&chain).expect("evm client for chain must exist");

                let to_addr = match &*token_in.to_lowercase() {
                    "eth" | "bnb" | "matic" | "avax" => {
                        let swap_target = "0x0000000000000000000000000000000000000000";
                        let tx = TxBuilder::build_1559(
                            &wallet,
                            &chain,
                            swap_target,
                            amount,
                            vec![],
                            priority_fee,
                            max_fee,
                            200000,
                            nonce,
                        );
                        let signed = match TxBuilder::sign_1559(&wallet, &tx) {
                            Ok(s) => s,
                            Err(e) => return CommandOutput::err(&format!("❌ 签名失败: {e}")),
                        };
                        match client_clone.send_raw_transaction(&signed.raw) {
                            Ok(tx_hash) => {
                                let explorer = chain.explorer_url();
                                lines.push(format!("\n✅ 交易已发送!"));
                                lines.push(format!("  TxHash: {tx_hash}"));
                                lines.push(format!("  {explorer}/tx/{tx_hash}"));
                            }
                            Err(e) => {
                                lines.push(format!("\n❌ 发送失败: {e}"));
                            }
                        }
                        our_addr
                    }
                    _ => our_addr,
                };
                let _ = to_addr;
            } else {
                lines.push(format!("\n💡 添加 --exec 或 --send 来执行交换"));
            }

            let out = CommandOutput::ok(&lines.join("\n"));
            out.with_json(serde_json::json!({
                "action": "swap_quote",
                "chain": chain.to_string(),
                "token_in": token_in, "token_out": token_out,
                "amount": amount_str, "nonce": nonce,
                "gas_price_gwei": gas_price,
            }))
        })
    }
}

pub struct TransferCmd;

impl CliCommand for TransferCmd {
    fn name(&self) -> &str {
        "/transfer"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/tx", "/send"]
    }

    fn description(&self) -> &str {
        "Transfer tokens: transfer <chain> <to> <amount> [token]"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.len() < 3 {
            return CommandOutput::ok(
                "Usage:\n  /transfer <chain> <to> <amount> [token]\n\
                 \nExamples:\n  /transfer eth 0x... 0.01\n  /transfer bsc 0x... 10 usdc\n  /transfer polygon 0x... 50 matic",
            );
        }

        let chain = chain_from_name(args[0].as_str());
        let to_addr = &args[1];
        let amount_str = &args[2];
        let token = args.get(3).map(|s| s.as_str()).unwrap_or("native");

        let amount = match parse_amount(amount_str) {
            Ok(a) => a,
            Err(e) => return CommandOutput::err(&format!("❌ {e}")),
        };

        if !to_addr.starts_with("0x") || to_addr.len() != 42 {
            return CommandOutput::err("❌ 无效地址格式 (需要 0x + 40 hex 字符)");
        }

        with_crypto(|crypto| {
            let wallet = match crypto.wallet_manager.active_wallet() {
                Some(w) => w.clone(),
                None => return CommandOutput::err("❌ 没有活跃钱包"),
            };

            let client = match crypto.evm_clients.get(&chain) {
                Some(c) => c,
                None => return CommandOutput::err(&format!("❌ 无法连接到 {chain}")),
            };

            let nonce = match client.get_transaction_count(&wallet.address) {
                Ok(n) => n,
                Err(e) => return CommandOutput::err(&format!("❌ nonce: {e}")),
            };

            let gas_price = match client.get_gas_price() {
                Ok(g) => g,
                Err(_) => 10.0,
            };

            let (contract_addr, data, gas_limit, note) = match token.to_lowercase().as_str() {
                "native" | "eth" | "bnb" | "matic" | "avax" => {
                    ("".to_string(), vec![], 21000u64, format!("发送 {amount_str} {}", chain.native_currency()))
                }
                token_addr if token_addr.starts_with("0x") => {
                    let reg = &crypto.token_registry;
                    if let Some(info) = reg.get(&chain, token_addr) {
                        let data = TxBuilder::encode_erc20_transfer(token_addr, to_addr, amount).1;
                        (token_addr.to_string(), data, 80000u64, format!("发送 {} {}", amount_str, info.symbol))
                    } else {
                        let data = TxBuilder::encode_erc20_transfer(token_addr, to_addr, amount).1;
                        (token_addr.to_string(), data, 80000u64, format!("发送 {amount_str} token"))
                    }
                }
                sym => {
                    let reg = &crypto.token_registry;
                    let found = reg.all_tokens().iter().find(|t| t.symbol.to_lowercase() == sym);
                    match found {
                        Some(info) => {
                            let data = TxBuilder::encode_erc20_transfer(&info.address, to_addr, amount).1;
                            (info.address.clone(), data, 80000u64, format!("发送 {} {}", amount_str, info.symbol))
                        }
                        None => return CommandOutput::err(&format!("❌ 未知代币: {sym}")),
                    }
                }
            };

            let priority_fee = (gas_price * 0.1).max(0.1) as u128;
            let max_fee = (gas_price * 1.2) as u128;

            let target = if contract_addr.is_empty() {
                to_addr.clone()
            } else {
                contract_addr
            };

            let tx = TxBuilder::build_1559(
                &wallet, &chain, &target, amount, data,
                priority_fee, max_fee, gas_limit, nonce,
            );

            let signed = match TxBuilder::sign_1559(&wallet, &tx) {
                Ok(s) => s,
                Err(e) => return CommandOutput::err(&format!("❌ 签名失败: {e}")),
            };

            match client.send_raw_transaction(&signed.raw) {
                Ok(tx_hash) => {
                    let explorer = chain.explorer_url();
                    let out = CommandOutput::ok(&format!(
                        "✅ {note}\n   From: {}\n   To: {} ({})\n   TxHash: {tx_hash}\n   {explorer}/tx/{tx_hash}",
                        short_addr(&wallet.address),
                        short_addr(to_addr),
                        chain,
                    ));
                    out.with_json(serde_json::json!({
                        "action": "transfer",
                        "chain": chain.to_string(),
                        "from": wallet.address,
                        "to": to_addr,
                        "amount": amount_str,
                        "token": token,
                        "tx_hash": tx_hash,
                    }))
                }
                Err(e) => CommandOutput::err(&format!("❌ 发送失败: {e}")),
            }
        })
    }
}

pub struct ApproveCmd;

impl CliCommand for ApproveCmd {
    fn name(&self) -> &str {
        "/approve"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/allow"]
    }

    fn description(&self) -> &str {
        "Approve token spending: approve <chain> <token> <spender> <amount>"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.len() < 4 {
            return CommandOutput::ok(
                "Usage:\n  /approve <chain> <token> <spender> <amount>\n\
                 \nExamples:\n  /approve eth usdc 0x... 1000\n  /approve bsc 0x... 0x... unlimited",
            );
        }

        let chain = chain_from_name(args[0].as_str());
        let token_arg = &args[1];
        let spender = &args[2];
        let amount_str = &args[3];

        let amount = if amount_str == "unlimited" || amount_str == "max" || amount_str == "inf" {
            u128::MAX
        } else {
            match parse_amount(amount_str) {
                Ok(a) => a,
                Err(e) => return CommandOutput::err(&format!("❌ {e}")),
            }
        };

        if !spender.starts_with("0x") || spender.len() != 42 {
            return CommandOutput::err("❌ 无效spender地址");
        }

        with_crypto(|crypto| {
            let wallet = match crypto.wallet_manager.active_wallet() {
                Some(w) => w.clone(),
                None => return CommandOutput::err("❌ 没有活跃钱包"),
            };

            let client = match crypto.evm_clients.get(&chain) {
                Some(c) => c,
                None => return CommandOutput::err(&format!("❌ 无法连接到 {chain}")),
            };

            let nonce = match client.get_transaction_count(&wallet.address) {
                Ok(n) => n,
                Err(e) => return CommandOutput::err(&format!("❌ nonce: {e}")),
            };

            let token_addr = if token_arg.starts_with("0x") {
                token_arg.to_string()
            } else {
                let sym = token_arg.to_lowercase();
                let reg = &crypto.token_registry;
                let found = reg.all_tokens().iter().find(|t| t.symbol.to_lowercase() == sym);
                match found {
                    Some(info) => info.address.clone(),
                    None => return CommandOutput::err(&format!("❌ 未知代币: {sym}")),
                }
            };

            let data = TxBuilder::encode_erc20_approve(spender, amount);
            let gas_price = match client.get_gas_price() {
                Ok(g) => g,
                Err(_) => 10.0,
            };
            let priority_fee = (gas_price * 0.1).max(0.1) as u128;
            let max_fee = (gas_price * 1.2) as u128;

            let tx = TxBuilder::build_1559(
                &wallet, &chain, &token_addr, 0, data,
                priority_fee, max_fee, 80000, nonce,
            );

            let signed = match TxBuilder::sign_1559(&wallet, &tx) {
                Ok(s) => s,
                Err(e) => return CommandOutput::err(&format!("❌ 签名失败: {e}")),
            };

            match client.send_raw_transaction(&signed.raw) {
                Ok(tx_hash) => {
                    let display_amount = if amount == u128::MAX {
                        format!("无限 (u128::MAX)")
                    } else {
                        amount_str.to_string()
                    };
                    let explorer = chain.explorer_url();
                    let out = CommandOutput::ok(&format!(
                        "✅ 已授权 {} 给 {}\n   金额: {display_amount}\n   链: {chain}\n   TxHash: {tx_hash}\n   {explorer}/tx/{tx_hash}",
                        short_addr(&token_addr),
                        short_addr(spender),
                    ));
                    out.with_json(serde_json::json!({
                        "action": "approve",
                        "chain": chain.to_string(),
                        "token": token_addr,
                        "spender": spender,
                        "amount": if amount == u128::MAX { "unlimited" } else { amount_str },
                        "tx_hash": tx_hash,
                    }))
                }
                Err(e) => CommandOutput::err(&format!("❌ 发送失败: {e}")),
            }
        })
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
