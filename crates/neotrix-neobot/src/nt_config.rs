//! `nt_config` — neobot 本地配置（全部本地默认值、零外部依赖）.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 策略模式（执行/演练两档）.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Enforce,
    DryRun,
}

impl PolicyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enforce => "enforce",
            Self::DryRun => "dry_run",
        }
    }
}

/// 本地引擎选择（可换引擎注册表本地版）.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    /// 零模型回显引擎（默认；不调任何外部模型，纯本地）.
    Echo,
    /// 本机 CLI 引擎 (`claude`/`codex` 等, 经 PATH 启动).
    Cli { command: String },
    /// opencode CLI 引擎（Zen 免费档直驱；本机已登录，免 key；model 如 `opencode/space-bunny-free`）.
    Opencode { model: String },
    /// OpenAI 兼容 HTTP 引擎 (Ollama/LM Studio/vLLM/自建网关).
    /// key 永不落盘, 运行时读 `NEOBOT_API_KEY`.
    Http { base_url: String, model: String },
}

/// neobot 配置 (文件 `~/.neobot/config.json` + 环境变量覆盖).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeobotConfig {
    pub data_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub policy_mode: PolicyMode,
    /// 为 `true` 时 Bot 动作一律拒绝（人接管语义）.
    pub human_has_control: bool,
    /// agent loop 上限（本地默认 8）.
    pub max_steps: u8,
    pub engine: EngineKind,
    /// computer 动作 allowlist (空 = 全拒, fail-closed; 如 ["navigate", "screenshot"]).
    #[serde(default)]
    pub computer_allow: Vec<String>,
    /// navigate 目标 host allowlist (空 = 全拒; 如 ["example.com"]).
    #[serde(default)]
    pub computer_hosts: Vec<String>,
    /// operator 自写 deny 规则（本地小 matcher，见
    /// `nt_policy::evaluate_extra_deny`；空 = 无。坏规则照拒不误）。
    #[serde(default)]
    pub extra_deny: Vec<String>,
    /// 写预算（嵌套预算：单次 ≤ 单轮 ≤ 单任务）。
    #[serde(default = "default_write_budget")]
    pub write_budget: WriteBudget,
}

/// 写预算（三界嵌套：单次写 ≤ 单轮累计 ≤ 单任务累计）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteBudget {
    pub max_single_write_bytes: usize,
    pub max_turn_write_bytes: usize,
    pub max_task_write_bytes: usize,
}

/// 默认预算：单次 512KiB / 单轮 1MiB / 单任务 2MiB（旧 WRITE_CAP 上限保留为任务级）。
pub fn default_write_budget() -> WriteBudget {
    WriteBudget {
        max_single_write_bytes: 512 * 1024,
        max_turn_write_bytes: 1024 * 1024,
        max_task_write_bytes: 2 * 1024 * 1024,
    }
}

impl NeobotConfig {
    /// 默认配置: 数据 `~/.neobot`, 工作区 `~/.neobot/workspace`.
    pub fn default_local() -> Result<Self, NtBotError> {
        let home = dirs_home().ok_or_else(|| NtBotError::Invalid("cannot locate home dir".to_owned()))?;
        let data_dir = home.join(".neobot");
        Ok(Self {
            workspace_dir: data_dir.join("workspace"),
            data_dir,
            policy_mode: PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 8,
            engine: EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: default_write_budget(),
        })
    }

    /// 环境变量覆盖: `NEOBOT_DATA_DIR` / `NEOBOT_POLICY` / `NEOBOT_ENGINE`.
    pub fn from_env() -> Result<Self, NtBotError> {
        let mut cfg = Self::default_local()?;
        if let Ok(dir) = std::env::var("NEOBOT_DATA_DIR") {
            if !dir.trim().is_empty() {
                cfg.data_dir = PathBuf::from(dir.trim());
                cfg.workspace_dir = cfg.data_dir.join("workspace");
            }
        }
        if let Ok(mode) = std::env::var("NEOBOT_POLICY") {
            cfg.policy_mode = match mode.trim() {
                "dry_run" => PolicyMode::DryRun,
                "enforce" => PolicyMode::Enforce,
                other => {
                    return Err(NtBotError::Invalid(format!("unknown NEOBOT_POLICY '{other}'")));
                }
            };
        }
        if let Ok(engine) = std::env::var("NEOBOT_ENGINE") {
            let trimmed = engine.trim();
            if trimmed == "http" {
                let (http, _) = crate::nt_http_engine::HttpEngineConfig::from_env()?;
                cfg.engine = EngineKind::Http {
                    base_url: http.base_url,
                    model: http.model,
                };
            } else if trimmed == "opencode" {
                let model = std::env::var("NEOBOT_MODEL")
                    .unwrap_or_default()
                    .trim()
                    .to_owned();
                cfg.engine = EngineKind::Opencode {
                    model: if model.is_empty() {
                        crate::nt_engine::DEFAULT_ZEN_MODEL.to_owned()
                    } else {
                        model
                    },
                };
            } else if !trimmed.is_empty() && trimmed != "echo" {
                cfg.engine = EngineKind::Cli {
                    command: trimmed.to_owned(),
                };
            }
        }
        cfg.validate()?;
        Ok(cfg)
    }

    /// 校验 + 建目录 (幂等).
    pub fn validate(&self) -> Result<(), NtBotError> {
        if self.max_steps == 0 || self.max_steps > 32 {
            return Err(NtBotError::Invalid(format!(
                "max_steps {} out of range 1..=32",
                self.max_steps
            )));
        }
        // 预算必须嵌套（rish 律：单次超轮、轮超任务即非法配置）。
        let budget = self.write_budget;
        if !(budget.max_single_write_bytes > 0
            && budget.max_single_write_bytes <= budget.max_turn_write_bytes
            && budget.max_turn_write_bytes <= budget.max_task_write_bytes)
        {
            return Err(NtBotError::Invalid(format!(
                "write_budget must nest single({}) <= turn({}) <= task({})",
                budget.max_single_write_bytes,
                budget.max_turn_write_bytes,
                budget.max_task_write_bytes
            )));
        }
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.workspace_dir)?;
        Ok(())
    }

    /// SQLite 文件路径.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("neobot.db")
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

#[cfg(test)]
mod tests {
    use super::NeobotConfig;

    #[test]
    fn rejects_bad_max_steps() {
        let mut cfg = NeobotConfig {
            data_dir: std::env::temp_dir().join("neobot-test-bad"),
            workspace_dir: std::env::temp_dir().join("neobot-test-bad-ws"),
            policy_mode: super::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 0,
            engine: super::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: super::default_write_budget(),
        };
        assert!(cfg.validate().is_err());
        cfg.max_steps = 33;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_unnested_budget() {
        let mut cfg = NeobotConfig::default_local().expect("default");
        cfg.write_budget.max_single_write_bytes = cfg.write_budget.max_turn_write_bytes + 1;
        assert!(cfg.validate().is_err());
        cfg.write_budget = super::default_write_budget();
        assert!(cfg.validate().is_ok());
    }
}
