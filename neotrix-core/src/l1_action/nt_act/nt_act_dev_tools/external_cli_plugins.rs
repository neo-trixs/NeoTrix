//! 外部 CLI 插件（descriptor 驱动，热插拔）
//!
//! NeoTrix 不把任何具体 CLI 名硬编码进发现/调度路径；外部 CLI 以一个
//! JSON descriptor 落盘到插件目录，加载即得。新增/卸载某个 CLI = 增删
//! 目录下的对应 `*.json` 文件。当前只消费 `mode: interactive` 的条目；
//! headless/chat 形态应走模型 provider，不在本文件。
//!
//! # Safety
//! - 仅 `std::fs` 读目录 + `std::process` spawn，无 unsafe (R-P1)。
//! - 生产路径无 `unwrap/expect/panic!`。

use serde::Deserialize;
use std::path::PathBuf;

use crate::l1_action::nt_io::nt_io_plugin::{Plugin, PluginEvent};

/// 一个外部 CLI 插件的 descriptor。
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalCliPlugin {
    /// 插件名，用于 `--agent <name>` 的键。
    pub name: String,
    /// 命令（PATH 可解析或绝对路径）。
    pub command: String,
    /// 启动参数（不含走管道 prompt 的形态）。
    #[serde(default)]
    pub args: Vec<String>,
    /// 探活参数，默认 `["--version"]`。
    #[serde(default)]
    pub probe_args: Vec<String>,
    /// 运行形态：`"interactive"`（交互 TUI）或 `"headless"`（连续命令）。
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "interactive".to_string()
}

impl ExternalCliPlugin {
    /// 探活：跑 `command + probe_args`（默认 `--version`）。
    pub fn probe_available(&self) -> bool {
        let args = if self.probe_args.is_empty() {
            vec!["--version".to_string()]
        } else {
            self.probe_args.clone()
        };
        crate::l1_action::nt_model_cli::run_capture(
            &self.command,
            &args,
            std::time::Duration::from_secs(10),
        )
        .map(|o| !o.trim().is_empty())
        .unwrap_or(false)
    }

    /// 启动：交互形态 spawn 子进程，stdio 直通 TTY，不解析输出。
    pub fn launch(&self) -> std::io::Result<std::process::Child> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(&self.args);
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
        cmd.spawn()
    }
}

/// 插件目录：`$NEOTRIX_PLUGINS_DIR`，否则 `~/.config/neotrix/plugins`。
pub fn plugins_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("NEOTRIX_PLUGINS_DIR") {
        return PathBuf::from(d);
    }
    let home = std::env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join(".config")
        .join("neotrix")
        .join("plugins")
}

/// 扫描插件目录，解析所有 `*.json` descriptor。错误条目跳过。
pub fn load_external_cli_plugins() -> Vec<ExternalCliPlugin> {
    let dir = plugins_dir();
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) ==false {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(p) = serde_json::from_str::<ExternalCliPlugin>(&text) {
                out.push(p);
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// 按名字查找插件。
pub fn find_external_cli_plugin(name: &str) -> Option<ExternalCliPlugin> {
    load_external_cli_plugins().into_iter().find(|p| p.name == name)
}

/// 把 descriptor 适配成统一的 `Plugin` 实例（name/version 为登记时一次性
/// 静态化，cron `Plugin::name()` 需要 &'static str）。
pub struct CliDescriptorPlugin {
    name: &'static str,
    version: &'static str,
    pub descriptor: ExternalCliPlugin,
}

impl CliDescriptorPlugin {
    pub fn new(descriptor: ExternalCliPlugin) -> Self {
        let name = Box::leak(descriptor.name.clone().into_boxed_str());
        let version = Box::leak("0.0.0-desc".to_string().into_boxed_str());
        Self {
            name,
            version,
            descriptor,
        }
    }
}

impl Plugin for CliDescriptorPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    fn version(&self) -> &'static str {
        self.version
    }
    fn capability(&self) -> &'static str {
        if self.descriptor.mode == "interactive" {
            "cli_agent"
        } else {
            "external_cli"
        }
    }
    fn on_load(&self) -> Result<(), String> {
        if self.descriptor.probe_available() {
            Ok(())
        } else {
            Err(format!("`{name}` 探活失败", name = self.name))
        }
    }
    fn on_unload(&self) -> Result<(), String> {
        Ok(())
    }
    fn on_event(&self, _event: &PluginEvent) -> Result<(), String> {
        Ok(())
    }
}

/// 所有 descriptor 插件 → `Box<dyn Plugin>` 列表，供统一注册表批量装载。
pub fn external_cli_as_plugins() -> Vec<Box<dyn Plugin>> {
    load_external_cli_plugins()
        .into_iter()
        .map(|p| Box::new(CliDescriptorPlugin::new(p)) as Box<dyn Plugin>)
        .collect()
}

/// 把外部 CLI descriptor 批量注册进统一的共享注册表。
pub async fn load_external_cli_into(
    registry: &crate::l1_action::nt_io::nt_io_plugin::PluginRegistry,
) -> Result<(), String> {
    registry.load_batch(external_cli_as_plugins()).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_and_find_from_temp_plugins_dir() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix-plugins-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&tmp);
        let f = tmp.join("demo.json");
        let mut fh = match std::fs::File::create(&f) {
            Ok(f) => f,
            Err(_) => return,
        };
        let _ = fh.write_all(
            br#"{"name":"demo","command":"/bin/echo","args":["hi"],"probe_args":["--version"],"mode":"interactive"}"#,
        );
        drop(fh);
        std::env::set_var("NEOTRIX_PLUGINS_DIR", &tmp);
        let all = load_external_cli_plugins();
        assert!(all.iter().any(|p| p.name == "demo"));
        assert!(find_external_cli_plugin("demo").is_some());
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::remove_var("NEOTRIX_PLUGINS_DIR");
    }

    #[test]
    fn unknown_plugin_is_none() {
        // 该名字不会出现在任何插件 descriptor 里，无需依赖 env 目录。
        assert!(find_external_cli_plugin("does-not-exist-xyz").is_none());
    }
}
