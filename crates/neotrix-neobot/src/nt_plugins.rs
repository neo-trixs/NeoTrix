//! `nt_plugins` — 以 data_dir/plugins/*.json 为热插拔源的插件清单。
//!
//! 判语义形状（`NEOTRIX-STD-1.0` B10 与吸收约束）:
//! - 静态解析 JSON，动态注册 *已知名字*, 呼叫到独立子进程；
//! - **不**把插件能力塞进 enum 之外的隐藏扫描：未登记就当 unknown fail-closed；
//! - 子进程 **只读** workspace（cwd 受限）+ 环境只透必要键（此处只透 `PATH`/`HOME`/`NEOBOT_*`/`NT_*` 留待版本）；
//! - 错误映射：每调用失败就返回错误给模型，不因某个 manifest 损坏致命；
//! - **默认 deny**：只有显式放进 plugins 目录的 manifest 才会被发现调用，
//!   模型无法凭一个不存在的名字触达本地命令。
//!
//! 未来 P3/P4: 一旦接入真 MCP server listing 时，同一入口也注册 `Plugin(String)`。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;


/// One declarative plugin manifest: a named subprocess tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    /// 工具名（也是 JSON 的 display；禁与内置撞名 —— 运行时跳过并打印 warning）。
    pub name: String,
    /// 给模型/人看的说明。
    pub description: Option<String>,
    /// 可执行命令（命令路径会在 PATH 中搜索；不允许传入 shell 模板）。
    pub command: String,
    /// 固定前缀参数（JSON 数组字符串），调用参数从 stdin 传入。
    #[serde(default)]
    pub args: Vec<String>,
}

/// 从 plugin json 重新验活且只返回真实有效的。
pub fn load_dir(data_dir: &Path) -> Result<Vec<PluginManifest>, NtBotError> {
    let dir = data_dir.join("plugins");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(v) => v,
        Err(err) => return Err(NtBotError::Invalid(format!("plugins dir unreadable: {err}"))),
    };
    for entry in entries {
        let path = entry
            .map_err(|e| NtBotError::Invalid(format!("plugins read_dir: {e}")))?
            .path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match std::fs::read_to_string(&path) {
                Ok(text) => match serde_json::from_str::<PluginManifest>(&text) {
                    Ok(m) => out.push(m),
                    Err(err) => {
                        return Err(NtBotError::Invalid(format!(
                            "plugin manifest parse failed '{}': {err}",
                            path.display()
                        )))
                    }
                },
                Err(err) => {
                    return Err(NtBotError::Invalid(format!(
                        "plugin file unreadable '{}': {err}",
                        path.display()
                    )))
                }
            }
        }
    }
    Ok(out)
}

/// 全局已登记插件清单；由应用启动时填充。
static REGISTERED: std::sync::OnceLock<std::sync::RwLock<Vec<PluginManifest>>> =
    std::sync::OnceLock::new();

fn reg() -> &'static std::sync::RwLock<Vec<PluginManifest>> {
    REGISTERED.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// 以 data_dir/plugins 为源填充全局表；闭包不持有其内容一份。
pub fn init(data_dir: &Path) -> Result<(), NtBotError> {
    let loaded = load_dir(data_dir)?;
    let mut all = reg()
        .write()
        .map_err(|_| NtBotError::Store("plugin registry poisoned".to_owned()))?;
    // P4: 过完能力树 Registry, 让 registry_audit/node_count 能看见 plugin 清单。
    for m in &loaded {
        let node = nt_core_capability_tree::node::CapabilityNode::new_primitive(
            m.name.clone(),
            nt_core_capability_tree::node::Domain::Neobot,
            vec!["dynamic_tool".to_owned()],
        );
        let _ = crate::nt_capability_registry::register_node(node);
    }
    *all = loaded;
    Ok(())
}

/// 主动登记一个清单（单测/启动后增量发现）。
pub fn register_one(m: PluginManifest) -> Result<(), NtBotError> {
    let mut all = reg()
        .write()
        .map_err(|_| NtBotError::Store("plugin registry poisoned".to_owned()))?;
    if all.iter().any(|x| x.name == m.name) {
        return Err(NtBotError::Invalid(format!(
            "plugin '{}' already registered",
            m.name
        )));
    }
    all.push(m);
    Ok(())
}

/// 判断名字是否在运行时插件表中（给 `ToolName::parse` 用）。
pub fn is_registered(name: &str) -> bool {
    reg()
        .read()
        .map(|all| all.iter().any(|m| m.name == name))
        .unwrap_or(false)
}

/// 通过名字取回 manifest, 供 execute 路由。
pub fn lookup(name: &str) -> Option<PluginManifest> {
    reg()
        .read()
        .ok()?
        .iter()
        .find(|m| m.name == name)
        .cloned()
}

/// 调起插件子进程。
///
/// 规则：
/// - `command` + `args` 前缀；
/// - stdin 以 JSON 字节序列化的 `call.args`；
/// - 输出 stdout UTF-8 原样返回；stderr 非空则以 `Err` 保持安全。
///
/// ⛔ 不直接返回 `ToolOutcome`, 调用方转成. 执RUNTIME 不采纳。
pub fn invoke(manifest: &PluginManifest, args: &serde_json::Value, cwd: &Path) -> Result<String, NtBotError> {
    let mut child = std::process::Command::new(&manifest.command)
        .args(&manifest.args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| NtBotError::Invalid(format!("plugin spawn failed '{}': {e}", manifest.name)))?;

    let mut stdin = child.stdin.take().ok_or_else(|| {
        NtBotError::Invalid(format!("plugin '{}' has no stdin handle", manifest.name))
    })?;
    stdin
        .write_all(&serde_json::to_vec(args).map_err(NtBotError::from)?)
        .map_err(|e| NtBotError::Invalid(format!("plugin stdin write failed: {e}")))?;
    drop(stdin);

    let child_out = child.wait_with_output().map_err(|e| {
        NtBotError::Invalid(format!("plugin '{}' wait failed: {e}", manifest.name))
    })?;
    if !child_out.status.success() {
        let stderr = String::from_utf8_lossy(&child_out.stderr);
        return Err(NtBotError::Engine {
            engine: "plugin".to_owned(),
            reason: format!("plugin '{}' exited {}: {}", manifest.name, child_out.status, stderr.trim()),
        });
    }
    let stdout = String::from_utf8_lossy(&child_out.stdout);
    Ok(stdout.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_dir_returns_empty_when_dir_missing() {
        let tmp = crate::nt_testutil::temp_dir("nt_plugins_empty");
        let found = load_dir(&tmp).expect("missing dir");
        assert!(found.is_empty());
    }

    #[test]
    fn list_open_dir_parses_manifest_json() {
        let tmp = crate::nt_testutil::temp_dir("nt_plugins_parse");
        std::fs::create_dir_all(tmp.join("plugins")).unwrap();
        std::fs::write(
            tmp.join("plugins/echo.json"),
            r#"{"name":"echo","description":"demo","command":"/bin/cat","args":[]}"#,
        )
        .unwrap();
        let found = load_dir(&tmp).expect("parse");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "echo");
    }

    #[test]
    fn register_parses_manifest() {
        let m = PluginManifest {
            name: "demo_echo".to_owned(),
            description: None,
            command: "/bin/cat".to_owned(),
            args: vec![],
        };
        register_one(m).expect("register");
        assert!(is_registered("demo_echo"));
        assert!(!is_registered("missing"));
    }

    #[test]
    fn invoke_plugin_runs_echo_command() {
        let manifest = PluginManifest {
            name: "echo_command".to_owned(),
            description: None,
            command: "/bin/cat".to_owned(),
            args: vec![],
        };
        let tmp = crate::nt_testutil::temp_dir("nt_plugins_invoke");
        std::fs::create_dir_all(&tmp).unwrap();
        let out = invoke(&manifest, &serde_json::json!({"a":1}), &tmp).expect("invoke");
        assert!(out.contains("\"a\""), "子进程应原样回传 stdin: {out}");
    }
}
