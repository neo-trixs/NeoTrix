//! # cli_free_source — 外部 CLI 免费模型发现源（资源适配器）
//!
//! 池子（`UnifiedModelPool`）的 `ModelSource` 插件：跑外部模型 CLI 的列表命令
//! （默认 `opencode models`），把免费后缀的模型发现为 `is_free` 池条目。
//! 外部项目永远只是可替换资源：类型名不带外部标签，命令/参数/后缀全是构造参数。
//! 发现结果经 `pool.refresh()` 进缓存，`pool.free_models()` 即免费模型清单，
//! 晶体经 `NtFreePoolAsk` 智能调用（轮转 + 故障转移 + 冷却）。
//!
//! ```text
//! <cli> models ──▶ parse（provider/model 行，免费后缀过滤）──▶ cloud_free 条目
//!                                                                 ──▶ UnifiedModelPool
//! ```
//!
//! # Safety
//! - 只读子进程调用（`run_capture` 超时可杀），无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`。

use super::model_pool::{ModelSource, UnifiedModelEntry};
use super::provider_catalog::ProviderCategory;
use crate::l1_action::nt_io::nt_io_provider::common::factory::LlmProviderType;
use crate::l1_action::nt_model_cli::run_capture;
use std::time::Duration;

/// 外部 CLI 免费模型发现源（默认后端：opencode）。
pub struct CliFreeSource {
    command: String,
    /// 显式 argv（测试注入用）；`None` = 标准 `models` 调用。
    argv: Option<Vec<String>>,
    /// 免费后缀（默认 `-free`，随后端可配）。
    free_suffix: String,
    timeout: Duration,
}

impl CliFreeSource {
    pub fn new() -> Self {
        Self {
            command: "opencode".to_string(),
            argv: None,
            free_suffix: "-free".to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_argv(mut self, argv: Vec<String>) -> Self {
        self.argv = Some(argv);
        self
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    pub fn with_free_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.free_suffix = suffix.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for CliFreeSource {
    fn default() -> Self {
        Self::new()
    }
}

/// 纯函数：解析模型列表输出 → `(provider, model)`。
/// 只收含 `/` 且无空白的行（表头/空行/日志自动过滤）。
pub fn parse_models_list(output: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in output.lines() {
        let t = line.trim();
        if t.is_empty() || t.contains(char::is_whitespace) || !t.contains('/') {
            continue;
        }
        if let Some(slash) = t.find('/') {
            let (p, m) = t.split_at(slash);
            let model = &m[1..];
            if !p.is_empty() && !model.is_empty() {
                out.push((p.to_string(), model.to_string()));
            }
        }
    }
    out
}

/// 免费判定：模型名以免费后缀结尾（默认 `-free`，随后端可配）。
pub fn is_free_model_id(model: &str) -> bool {
    is_free_model_id_with(model, "-free")
}

/// 带后缀参数的免费判定（纯函数，源内复用）。
pub fn is_free_model_id_with(model: &str, suffix: &str) -> bool {
    !suffix.is_empty() && model.ends_with(suffix)
}

/// 免费档真实调用端点：opencode zen（OpenAI 兼容，匿名可用）。
/// 统一管理要求每个条目带真实 URL，不写占位符；
/// 调用方走 CLI 时忽略本字段，走 HTTP 时直连本地址。
pub fn zen_base_url() -> String {
    std::env::var("NEOTRIX_ZEN_URL")
        .unwrap_or_else(|_| "https://opencode.ai/zen/v1".to_string())
}

impl ModelSource for CliFreeSource {
    fn name(&self) -> &str {
        "cli-free"
    }

    fn category(&self) -> ProviderCategory {
        ProviderCategory::Cloud
    }

    fn discover(&self) -> Vec<UnifiedModelEntry> {
        let args = self
            .argv
            .clone()
            .unwrap_or_else(|| vec!["models".to_string()]);
        let output = match run_capture(&self.command, &args, self.timeout) {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };
        parse_models_list(&output)
            .into_iter()
            .filter(|(_, m)| is_free_model_id_with(m, &self.free_suffix))
            .map(|(p, m)| {
                let mut e = UnifiedModelEntry::cloud_free(
                    &p,
                    &m,
                    &m,
                    &zen_base_url(),
                    "free",
                    false,
                    None,
                    LlmProviderType::OpenCodeZen,
                );
                // 来源打真标签：cloud_free() 统一盖 "cloud_free" 章，
                // 这里纠正为发现源名，否则池展示/过滤认不出自家条目。
                e.source = "cli-free".to_string();
                e
            })
            .collect()
    }

    fn is_available(&self) -> bool {
        run_capture(
            &self.command,
            &["--version".to_string()],
            Duration::from_secs(5),
        )
        .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "opencode/big-pickle\nopencode/mimo-v2.5-free\n\nopencode/muse-spark-1.3-contributor-free\nbroken line here\n/opencode\n";

    #[test]
    fn test_parse_models_list_filters_noise() {
        let got = parse_models_list(SAMPLE);
        assert_eq!(
            got,
            vec![
                ("opencode".to_string(), "big-pickle".to_string()),
                ("opencode".to_string(), "mimo-v2.5-free".to_string()),
                (
                    "opencode".to_string(),
                    "muse-spark-1.3-contributor-free".to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_is_free_model_id() {
        assert!(is_free_model_id("mimo-v2.5-free"));
        assert!(!is_free_model_id("big-pickle"));
        assert!(!is_free_model_id("freewilly"));
    }

    #[test]
    fn test_zen_base_url_default() {
        // 只要不断言 env 覆盖分支（环境相关），默认分支必须真实可用
        let url = zen_base_url();
        assert!(url.starts_with("http"));
        assert!(!url.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_positive_via_sh() {
        // /bin/sh 内建 printf 输出多行，模拟 `opencode models` 真实输出
        //（macOS 无 /bin/printf 独立二进制，故走 sh -c）
        let src = CliFreeSource::new()
            .with_command("/bin/sh")
            .with_argv(vec![
                "-c".to_string(),
                "printf '%s\\n' opencode/mimo-v2.5-free opencode/big-pickle".to_string(),
            ]);
        let entries = src.discover();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "opencode/mimo-v2.5-free");
        assert!(entries[0].is_free);
        assert_eq!(entries[0].source, "cli-free");
        assert_eq!(entries[0].source, "cloud_free");
    }

    #[test]
    fn test_discover_unavailable_command_empty() {
        let src = CliFreeSource::new().with_command("/nonexistent-nt-xyz");
        assert!(src.discover().is_empty());
        assert!(!src.is_available());
    }

    #[test]
    fn test_entry_shape() {
        let e = UnifiedModelEntry::cloud_free(
            "opencode",
            "mimo-v2.5-free",
            "mimo-v2.5-free",
            "opencode-cli",
            "free",
            false,
            None,
            LlmProviderType::OpenCodeZen,
        );
        assert_eq!(e.id, "opencode/mimo-v2.5-free");
        assert!(e.is_free);
        assert!(!e.requires_api_key);
    }
}
