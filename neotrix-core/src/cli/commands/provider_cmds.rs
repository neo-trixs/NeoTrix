//! /provider 命令 — 自我/客体 Provider 管理
//!
//! 子命令:
//!   list      列出所有注册 provider (按分类)
//!   status    显示当前 gateway 状态
//!   info <name> 显示指定 provider 详情

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_io_provider::provider_catalog::{
    ProviderCategory, lookup_provider, providers_by_category,
};

pub struct ProviderCmd;

impl CliCommand for ProviderCmd {
    fn name(&self) -> &str {
        "/provider"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/providers", "/p"]
    }

    fn description(&self) -> &str {
        "Provider 管理: /provider list | /provider info <name>"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" | "ls" => self.cmd_list(),
            "info" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                self.cmd_info(name)
            }
            "challenge" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                let task_type = args.get(2).map(|s| s.as_str()).unwrap_or("arithmetic");
                self.cmd_challenge(name, task_type)
            }
            _ => CommandOutput::err("Usage:\n  /provider list                  列出所有支持的 provider\n  /provider info <name>           查看 provider 详情\n  /provider challenge <name> [task]  运行 LLM Challenge 基准 (arithmetic|extract|boolean)"),
        }
    }
}

impl ProviderCmd {
    fn cmd_list(&self) -> CommandOutput {
        let mut msg = String::from("Provider 目录 — 自我(主体) vs 代理/云(客体)\n\n");

        // 本地推理 (主体)
        msg.push_str(&format!("【{}】 — 数据不出设备\n", ProviderCategory::Local.label()));
        for p in providers_by_category(ProviderCategory::Local) {
            let key_status = if p.api_key_env.is_none() { "免密钥" } else { "需密钥" };
            msg.push_str(&format!("  {} — {} ({})\n", p.display_name, p.base_url, key_status));
        }

        // 自定义代理 (客体)
        msg.push_str(&format!("\n【{}】 — 自定义中转\n", ProviderCategory::Proxy.label()));
        let proxy_base = std::env::var("NEOTRIX_PROXY_BASE_URL").unwrap_or_else(|_| "(未配置)".into());
        let proxy_key = std::env::var("NEOTRIX_PROXY_API_KEY").ok()
            .filter(|k| !k.is_empty())
            .map(|_| "已配置 ✓")
            .unwrap_or("未配置");
        msg.push_str(&format!("  custom-proxy — {}\n  API Key: {}\n", proxy_base, proxy_key));

        // 云端 API (客体)
        msg.push_str(&format!("\n【{}】 — 需 API Key\n", ProviderCategory::Cloud.label()));
        for p in providers_by_category(ProviderCategory::Cloud) {
            let key_status = match p.api_key_env {
                Some(env) => {
                    let val = std::env::var(env).unwrap_or_default();
                    if val.is_empty() { format!("{} 未设置 ✗", env) } else { "已配置 ✓".to_string() }
                }
                None => "免密钥 ✓".to_string(),
            };
            let free_label = if p.is_free { " [免费]" } else { "" };
            msg.push_str(&format!("  {:<16} — {:<20}{}  {}\n", p.display_name, p.default_model, free_label, key_status));
        }

        CommandOutput::ok(&msg)
    }

    fn cmd_info(&self, name: &str) -> CommandOutput {
        if name.is_empty() {
            return CommandOutput::err("Usage: /provider info <name>\n可用名称: openai, anthropic, gemini, groq, openrouter, ollama, ...");        }
        match lookup_provider(name) {
            Some(info) => {
                let key_status = match info.api_key_env {
                    Some(env) => {
                        let val = std::env::var(env).unwrap_or_default();
                        if val.is_empty() {
                            format!("{} (未设置)", env)
                        } else {
                            format!("{} ✓ (已配置)", env)
                        }
                    }
                    None => "免密钥 (本地/免费)".to_string(),
                };
                let free_label = if info.is_free { "免费" } else { "付费" };
                let mut msg = format!(
                    "Provider: {}\n  显示名: {}\n  分类: {}\n  基础 URL: {}\n  默认模型: {}\n  定价: {}\n  API Key: {}\n  模型列表:\n",
                    info.name, info.display_name, info.category.label(), info.base_url, info.default_model, free_label, key_status
                );
                for m in info.models {
                    msg.push_str(&format!("    - {}\n", m));
                }
                CommandOutput::ok(&msg)
            }
            None => CommandOutput::err(&format!("未知 provider: {}。使用 /provider list 查看可用列表。", name)),
        }
    }

    /// 运行 LLM Challenge 确定性基准 (P0-3, Unstract/LLM-Challenge pattern)。
    /// 接线 gateway.run_llm_challenge → ProviderBenchmark, 打分 accuracy/latency/cost。
    fn cmd_challenge(&self, name: &str, task_type: &str) -> CommandOutput {
        if name.is_empty() {
            return CommandOutput::err("Usage: /provider challenge <name> [task]\ntask: arithmetic | extraction | boolean (默认 arithmetic)");
        }
        let gateway = crate::neotrix::nt_io_provider::create_gateway();
        let available = gateway.providers();
        if !available.iter().any(|p| p == name) {
            return CommandOutput::err(&format!(
                "provider '{}' 未注册 (env 未配置或不可用)。已注册: {:?}",
                name, available
            ));
        }
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => return CommandOutput::err(&format!("tokio runtime init failed: {e}")),
        };
        match rt.block_on(gateway.run_llm_challenge(name, task_type)) {
            Ok(bench) => {
                let grade = if bench.accuracy >= 1.0 { "PASS" } else if bench.accuracy >= 0.5 { "PARTIAL" } else { "FAIL" };
                CommandOutput::ok(&format!(
                    "LLM Challenge [{}] — {}\n  模型: {}\n  task: {}\n  accuracy: {:.0}%\n  avg latency: {} ms\n  cost: ${:.4}\n  评级: {}",
                    name, grade, bench.model, bench.task_type,
                    bench.accuracy * 100.0, bench.latency_ms, bench.cost_usd, grade
                ))
            }
            Err(e) => CommandOutput::err(&format!("LLM Challenge 失败: {}", e)),
        }
    }
}
