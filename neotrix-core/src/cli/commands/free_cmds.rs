//! /free 命令 — 免费 AI Provider 管理
//!
//! 子命令:
//!   list      列出所有免费 provider (keyless vs key-based)
//!   budget    查看免费 token 预算和已节省金额
//!   discover  重新发现免费模型

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_io_provider::free_catalog::FreeModelCatalog;
use crate::neotrix::nt_io_provider::factory::LlmProviderType;
use crate::neotrix::nt_io_provider::free_pool::global_free_pool;
use crate::neotrix::nt_io_provider::rate_profiles::free_provider_rate_profiles;
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct FreeCmd;

impl CliCommand for FreeCmd {
    fn name(&self) -> &str {
        "/free"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/freeproviders", "/free-list"]
    }

    fn description(&self) -> &str {
        "免费 AI 提供者管理: /free list | /free budget | /free discover"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" | "ls" => self.cmd_list(),
            "budget" | "b" => self.cmd_budget(),
            "discover" => self.cmd_discover(),
            _ => CommandOutput::err(
                "Usage:\n  /free list      列出所有免费 provider\n  /free budget    查看免费 token 预算\n  /free discover  重新发现免费模型",
            ),
        }
    }
}

impl FreeCmd {
    fn cmd_list(&self) -> CommandOutput {
        let profiles = free_provider_rate_profiles();
        let mut msg =
            String::from("╭─ Free LLM Providers ─────────────────────╮\n");

        let mut keyless: Vec<&str> = Vec::new();
        let mut key_based: Vec<&str> = Vec::new();
        for name in profiles.keys() {
            let typ = LlmProviderType::from_name(name);
            match typ {
                Some(t) if !t.is_free() => {} // skip paid (openai, anthropic)
                Some(t) if !t.needs_api_key() => keyless.push(name),
                _ => key_based.push(name),
            }
        }
        keyless.sort();
        key_based.sort();

        msg.push_str("  🔓 Keyless (No API key):\n");
        for name in &keyless {
            if let Some(profile) = profiles.get(name) {
                msg.push_str(&format!(
                    "    ◦ {}  ({} RPM, {} TPM)\n",
                    name, profile.rpm, profile.tpm
                ));
            }
        }

        msg.push_str("  🔑 Free Tier (API key required):\n");
        for name in &key_based {
            if let Some(profile) = profiles.get(name) {
                msg.push_str(&format!(
                    "    ◦ {}  ({} RPM, {} TPM)\n",
                    name, profile.rpm, profile.tpm
                ));
            }
        }

        msg.push_str("╰──────────────────────────────────────────╯");
        CommandOutput::ok(&msg)
    }

    fn cmd_budget(&self) -> CommandOutput {
        let pool = global_free_pool();
        let budgets = pool.all_budgets();
        let total_remaining = pool.total_free_tokens_remaining();
        let total_saved = pool.total_savings();

        let mut msg =
            String::from("╭─ Free Token Budget ───────────────────────╮\n");
        msg.push_str(&format!(
            "  Total remaining: ~{}K tokens/mo\n",
            total_remaining / 1000
        ));
        msg.push_str(&format!("  Total $ saved:   ~${:.2}\n\n", total_saved));

        for budget in &budgets {
            let status = if budget.is_active { "🟢" } else { "🔴" };
            let keyless = if budget.is_keyless { " (Keyless)" } else { "" };
            msg.push_str(&format!(
                "  {} {}{keyless}\n",
                status, budget.provider_name
            ));
            if budget.monthly_token_cap > 0 {
                let used_pct = if budget.monthly_token_cap > 0 {
                    (budget.tokens_used as f64 / budget.monthly_token_cap as f64 * 100.0) as u64
                } else {
                    0
                };
                msg.push_str(&format!(
                    "     📊 {}/{}K tokens ({}%)\n",
                    budget.tokens_used / 1000,
                    budget.monthly_token_cap / 1000,
                    used_pct,
                ));
            } else {
                msg.push_str("     📊 Unlimited\n");
            }
        }
        msg.push_str("╰──────────────────────────────────────────╯");
        CommandOutput::ok(&msg)
    }

    fn cmd_discover(&self) -> CommandOutput {
        let mut catalog = FreeModelCatalog::new();
        let fresh = catalog.refresh();
        let mut providers = std::collections::HashSet::new();
        for e in &fresh {
            providers.insert(&e.provider);
        }
        let result = format!(
            "Discovered {} free models across {} providers",
            fresh.len(),
            providers.len()
        );
        CommandOutput::ok(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_list_contains_keyless_section() {
        let cmd = FreeCmd;
        let output = cmd.cmd_list();
        assert!(
            output.message.contains("Keyless"),
            "list should contain Keyless section, got: {}",
            output.message
        );
        assert!(
            output.message.contains("Free Tier"),
            "list should contain Free Tier section, got: {}",
            output.message
        );
    }

    #[test]
    fn test_cmd_list_contains_rpm_tpm() {
        let cmd = FreeCmd;
        let output = cmd.cmd_list();
        assert!(
            output.message.contains("RPM"),
            "list should show RPM, got: {}",
            output.message
        );
        assert!(
            output.message.contains("TPM"),
            "list should show TPM, got: {}",
            output.message
        );
    }

    #[test]
    fn test_cmd_list_no_paid_providers() {
        let cmd = FreeCmd;
        let output = cmd.cmd_list();
        assert!(
            !output.message.contains("openai"),
            "list should not contain openai (paid)"
        );
        assert!(
            !output.message.contains("anthropic"),
            "list should not contain anthropic (paid)"
        );
    }

    #[test]
    fn test_cmd_budget_format() {
        let cmd = FreeCmd;
        let output = cmd.cmd_budget();
        assert!(
            output.message.contains("Token Budget"),
            "budget should show Token Budget header, got: {}",
            output.message
        );
        assert!(
            output.message.contains("remaining"),
            "budget should show remaining tokens"
        );
    }

    #[test]
    fn test_cmd_discover_format() {
        // This test calls refresh() which hits OpenRouter API — may fail without network
        let cmd = FreeCmd;
        let output = cmd.cmd_discover();
        // Even without API responses, the hardcoded fallbacks should produce some models
        assert!(
            output.message.starts_with("Discovered"),
            "discover output should start with 'Discovered', got: {}",
            output.message
        );
    }
}
