//! `nt_config` — neobot 本地配置.
//!
//! 对标 openbot `server/src/config.ts` (DeploymentConfig) 与 cumora
//! `server/src/env.ts`, 但全部本地默认值、零外部依赖.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 策略模式 — 对标 openbot `ActionPolicy{mode}`.
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

/// 本地引擎选择 — 对标 cumora `EngineAdapter` 注册表 (本地子集).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    /// 零模型回显引擎 (OpenMuse `sample` 模式对应物, 默认).
    Echo,
    /// 本机 CLI 引擎 (`claude`/`codex`/`opencode` 等, 经 PATH 启动).
    Cli { command: String },
}

/// neobot 配置 (文件 `~/.neobot/config.json` + 环境变量覆盖).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeobotConfig {
    pub data_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub policy_mode: PolicyMode,
    /// 为 `true` 时 Bot 动作一律拒绝 (openbot `HumanHasControl` 语义).
    pub human_has_control: bool,
    /// agent loop 上限 (OpenMuse chat=6 / model=16, 本地默认 8).
    pub max_steps: u8,
    pub engine: EngineKind,
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
            if !trimmed.is_empty() && trimmed != "echo" {
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
        };
        assert!(cfg.validate().is_err());
        cfg.max_steps = 33;
        assert!(cfg.validate().is_err());
    }
}
