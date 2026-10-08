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
use crate::l1_action::nt_io::nt_io_plugin::PluginRegistry;

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
    /// 工作目录（相对/绝对路径均可）。`None` = 继承调用方 cwd。
    ///
    /// 为什么必须显式存在（F0）：descriptor 驱动 spawn 出的交互 agent 会按
    /// **自身 cwd** 划分项目/会话目录（如 freebuff 的 `projects/<目录名>/`），
    /// 继承调用方 cwd 意味着同一条插件在不同目录下跑会分裂成互不可见的历史。
    /// `#[serde(default)]` 保证旧 descriptor（无 cwd 字段）照旧载入。
    ///
    /// ⚠️ **F2 约束（项目身份 = 目录名）**：freebuff 把项目历史存在
    /// `~/.config/manicode/projects/<cwd 的目录名>/` —— 判据是**目录名**，不是
    /// 仓库内容。于是仓库根 `neotrix` 与 `neotrix-core` 是两段互不可见的历史。
    /// ⇒ 想用哪个项目的会话，就必须让 cwd **落在那个目录上**；descriptor 里的
    /// `cwd` 就是这个「项目身份」的声明位。
    /// ⛔ JSON 里放不了注释（加 `"//"` 键会被 descriptor 门的 C2「未知字段」
    /// 判红），故这条约束的文档落在这里与 `ntcode` 的 usage 字符串里。
    /// ⚠️ `ntcode --workdir` 的优先级**高于**本字段（见 `merge_cli_workdir`）
    /// ⇒ 显式 `--workdir` 会顶掉这里的项目绑定，等于换到另一个项目的历史。
    #[serde(default)]
    pub cwd: Option<PathBuf>,
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
    /// 显式设置工作目录（等价于 `cd <cwd> && <command>`）。
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// 合并 CLI 侧 `--workdir` 的**覆盖语义**（F0 唯一实现点，`ntcode` 调它）：
    /// CLI 显式给了就用 CLI 的；CLI 没给则保留 descriptor 自己写的 cwd。
    ///
    /// 抽成独立函数而不是内联在 `ntcode` 里，是为了这条优先级**可被单测覆盖**——
    /// 三个分支（都有 / 只有 descriptor / 只有 CLI）都能被断言到。
    pub fn merge_cli_workdir(&mut self, cli_workdir: Option<&std::path::Path>) {
        if let Some(dir) = cli_workdir {
            self.cwd = Some(dir.to_path_buf());
        }
    }

    /// 探活 argv：**只有** `probe_args`，缺省 `["--version"]`。
    ///
    /// 抽成独立函数，是为了让「探活永不混入运行期参数」这条约束**可被单测钉住**
    /// —— 见 `probe_argv_never_sees_launch_extra_args`。F1 的透传参数若被实现成
    /// 「合并进 `self.args`」，探活就会跟着拿到 `--continue <id>`，而那既可能让
    /// 探活失败、也和「插件是否可用」这个问题无关。
    pub fn probe_argv(&self) -> Vec<String> {
        if self.probe_args.is_empty() {
            vec!["--version".to_string()]
        } else {
            self.probe_args.clone()
        }
    }

    /// 探活：跑 `command + probe_args`（默认 `--version`）。
    pub fn probe_available(&self) -> bool {
        crate::l1_action::nt_model_cli::run_capture(
            &self.command,
            &self.probe_argv(),
            std::time::Duration::from_secs(10),
        )
        .map(|o| !o.trim().is_empty())
        .unwrap_or(false)
    }

    /// 启动 argv = `self.args` ++ `extra_args`（F1 透传口；`extra_args` 的顺序
    /// 就是命令行上的顺序，可重复传、按出现顺序累积）。
    ///
    /// 为什么**不**把透传参数并进 `self.args` 或加一个 serde 字段：
    /// - `self.args` 是**插件作者的声明**（descriptor 里写死的那几个），透传是
    ///   **调用方这一次会话的临时决定**；混进同一个字段后就再也分不清参数来源。
    /// - 加 serde 字段会改动 descriptor 的字段集，而 `nt_cli_plugin_probe.py`
    ///   的 C2 判据（未知字段）是从本 struct 抽的、`--self-test` 里还有一条
    ///   `TRUTH-rust-field-set-unchanged` 哨兵钉住字段集 ⇒ 新字段会让那扇门红，
    ///   而透传参数**根本不该是 descriptor 的一部分**。
    /// 走「调用参数」而非「结构体字段」，本 struct 的字段集保持不变。
    pub fn launch_argv(&self, extra_args: &[String]) -> Vec<String> {
        let mut argv = self.args.clone();
        argv.extend_from_slice(extra_args);
        argv
    }

    /// 启动：交互形态 spawn 子进程，stdio 直通 TTY，不解析输出。
    pub fn launch(&self) -> std::io::Result<std::process::Child> {
        self.launch_with(&[])
    }

    /// 同 `launch()`，但把 `extra_args` 追加到 `self.args` 之后（F1：调用方
    /// 透传给插件的启动参数，如 freebuff 的 `--continue <会话 id>`）。
    ///
    /// ⚠️ `extra_args` **只**进这里，**绝不**进 `probe_argv()`：探活必须仍然只跑
    /// `probe_args`。否则「续接一个已归档会话」这种一次性参数会让探活失败，
    /// 而探活失败的语义是「这个 CLI 装不上」，不是「这个会话 id 不存在」。
    pub fn launch_with(
        &self,
        extra_args: &[String],
    ) -> std::io::Result<std::process::Child> {
        let mut cmd = std::process::Command::new(&self.command);
        cmd.args(self.launch_argv(extra_args));
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
        // F0：cwd 必须显式落到子进程，否则交互 agent 会继承调用方 cwd，
        // 而项目/会话目录是按 cwd 划分的 ⇒ 同一插件在不同目录下跑会分裂历史。
        if let Some(dir) = &self.cwd {
            cmd.current_dir(dir);
        }
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
        if !path.extension().map(|e| e == "json").unwrap_or(false) {
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

/// 从共享 registry 取出全部 CLI 插件的 descriptor（cli_agent + external_cli 两类）。
///
/// 这是消费侧收口：ntcode 不再自己扫 plugins 目录，而是问 registry。
/// registry 登记了什么 CLI descriptor，ntcode 就能列出/启动什么 —— 「登记什么
/// 就跑什么」的语义对 CLI 能力同样成立。
pub async fn cli_descriptors_from_registry(
    registry: &PluginRegistry,
) -> Vec<ExternalCliPlugin> {
    let mut out = Vec::new();
    for cap in ["cli_agent", "external_cli"] {
        for handle in registry.shared_handles_by_capability(cap).await {
            if let Ok(arc) = handle.downcast::<ExternalCliPlugin>() {
                out.push((*arc).clone());
            }
        }
    }
    out
}

/// 按名字在 registry 里找交互式 CLI agent（capability=cli_agent）的 descriptor。
pub async fn find_cli_agent_by_name(
    registry: &PluginRegistry,
    name: &str,
) -> Option<ExternalCliPlugin> {
    cli_descriptors_from_registry(registry)
        .await
        .into_iter()
        .find(|p| p.name == name && p.mode == "interactive")
}

/// 校验：调用方透传参数**只对 `--agent` 路径生效**（F1）。
///
/// 为什么是 lib 函数而不是内联在 `ntcode` 的 `parse_args` 里：
/// 这条约束属于**插件启动契约**（透传参数的消费者是 `launch_with`，而
/// `launch_with` 只有 `--agent` 那条线会调），且它必须能被单测覆盖 ——
/// `ntcode` 是 bin，不在 `cargo test -p neotrix --lib` 的测试面内。
///
/// ⛔ 绝**不**静默忽略：模型池 / chat 那条线根本不 spawn 插件，用户在
/// 那里给的 `--agent-arg` 会被丢掉；静默丢掉等于让用户以为 `--continue`
/// 传进去了，实际没有任何子进程收到它。
pub fn ensure_agent_args_need_agent(
    agent: Option<&str>,
    agent_args: &[String],
) -> Result<(), String> {
    if agent_args.is_empty() {
        return Ok(());
    }
    if agent.is_none() {
        return Err(format!(
            "--agent-arg 只在 --agent <插件名> 路径下生效（本次没给 --agent，\
             却收到 {} 个透传参数）",
            agent_args.len()
        ));
    }
    Ok(())
}

/// 把 descriptor 适配成统一的 `Plugin` 实例（name/version 为登记时一次性
/// 静态化，cron `Plugin::name()` 需要 &'static str）。
pub struct CliDescriptorPlugin {
    name: &'static str,
    version: &'static str,
    descriptor: std::sync::Arc<ExternalCliPlugin>,
}

impl CliDescriptorPlugin {
    pub fn new(descriptor: ExternalCliPlugin) -> Self {
        let name = Box::leak(descriptor.name.clone().into_boxed_str());
        let version = Box::leak("0.0.0-desc".to_string().into_boxed_str());
        Self {
            name,
            version,
            descriptor: std::sync::Arc::new(descriptor),
        }
    }
    /// 取回共享的 descriptor 句柄：消费方 (ntcode) 由此拿到 command/args，
    /// 而不必再直读 plugins 目录 —— registry 登记了哪个，ntcode 就跑哪个。
    pub fn shared_descriptor(&self) -> std::sync::Arc<ExternalCliPlugin> {
        std::sync::Arc::clone(&self.descriptor)
    }
    /// 直接取 command/args（等价 `&*self.descriptor`，但不暴露字段）。
    pub fn descriptor_ref(&self) -> &ExternalCliPlugin {
        &self.descriptor
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
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn shared_handle(&self) -> Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> {
        // `ExternalCliPlugin: Any + Send + Sync`（blanket impl），故可做
        // 隐式 unsized coercion —— 这是合法的「具体类型 → trait object」转换，
        // 区别于禁止的 `dyn TraitA → dyn TraitB` 互转。注意 coercion 只能发生在
        // 赋值绑定位，不能透过 `Arc::clone` 的参数位反向推断。
        let arc: std::sync::Arc<ExternalCliPlugin> = std::sync::Arc::clone(&self.descriptor);
        let handle: std::sync::Arc<dyn std::any::Any + Send + Sync> = arc;
        Some(handle)
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

    // ── F0 / F5：cwd 覆盖链路 ──────────────────────────────────────────

    /// 进程内唯一临时目录名。
    ///
    /// 并行安全：同一进程内所有测试共享同一个 pid，只用 pid 会撞目录；
    /// 故再加一个原子计数器做第二维唯一。
    fn unique_tmp_dir(tag: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("neotrix-{tag}-{}-{n}", std::process::id()))
    }

    /// 测试用 descriptor。
    ///
    /// ⛔ 刻意不碰 `NEOTRIX_PLUGINS_DIR`：env 是**进程级**的，与既有
    /// `load_and_find_from_temp_plugins_dir` 并行跑会互相污染对方的插件目录。
    fn demo_plugin(command: &str, args: Vec<String>, probe_args: Vec<String>) -> ExternalCliPlugin {
        ExternalCliPlugin {
            name: "demo".to_string(),
            command: command.to_string(),
            args,
            cwd: None,
            probe_args,
            mode: "interactive".to_string(),
        }
    }

    /// F0 主断言：`launch()` 的 cwd **真生效**。
    ///
    /// 为什么这条不可能恒绿：子命令把 `pwd` 写进一个**相对路径**文件
    /// （`pwd > nt_cwd_probe.txt`），该文件必然落在**子进程自己的 cwd**。
    /// 若 `launch()` 漏掉 `current_dir`，文件就会落到进程 cwd（仓库根），
    /// `tmp` 下读不到 ⇒ 下面的读取断言必红。
    /// ⛔ 这里刻意**不**沿用既有测试「读不到就 return」的容错写法 ——
    /// 那样失败会被吞成绿，等于没测（L8：绿色≠有效）。
    ///
    /// 为什么不用「直接把子进程 stdout 捕获来比」：`launch()` 对 stdio 是
    /// `inherit`（交互 TUI 语义），不接管道，输出无法回收；改写成
    /// 「打印到 cwd 内的文件」既保持了断言强度，又不依赖管道。
    #[test]
    fn launch_applies_descriptor_cwd() {
        let tmp = unique_tmp_dir("cwd-launch");
        if std::fs::create_dir_all(&tmp).is_err() {
            eprintln!("跳过：无法创建 {}", tmp.display());
            return;
        }
        if !std::path::Path::new("/bin/sh").exists() {
            eprintln!("跳过：平台无 /bin/sh");
            let _ = std::fs::remove_dir_all(&tmp);
            return;
        }
        let plugin = demo_plugin(
            "/bin/sh",
            vec!["-c".to_string(), "pwd > nt_cwd_probe.txt".to_string()],
            Vec::new(),
        )
        .with_cwd(&tmp);
        let mut child = match plugin.launch() {
            Ok(c) => c,
            Err(e) => {
                // /bin/sh 存在却起不来 = 环境异常，按平台差异跳过。
                eprintln!("跳过：spawn /bin/sh 失败：{e}");
                let _ = std::fs::remove_dir_all(&tmp);
                return;
            }
        };
        let _ = child.wait();

        let printed = match std::fs::read_to_string(&tmp.join("nt_cwd_probe.txt")) {
            Ok(s) => s,
            Err(e) => {
                panic!(
                    "launch() 没把 cwd 设成 {}：cwd 内没有 marker 文件（{e}）",
                    tmp.display()
                );
            }
        };
        let printed = printed.trim();
        assert!(!printed.is_empty(), "子进程没打印出自己的 cwd（marker 为空）");
        // ⛔ macOS 上 `/var` 是 `/private/var` 的软链：子进程 `pwd` 报的是**物理**
        // 路径，我们手里的是**逻辑**路径 ⇒ 两侧都必须 canonicalize 后再比。
        let real = |p: &std::path::Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        assert_eq!(
            real(std::path::Path::new(printed)),
            real(&tmp),
            "子进程 cwd 应等于 descriptor 的 cwd"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 探活：不存在的命令必须 false。
    ///
    /// 同时钉一个**正例**（`/bin/echo` 必为 true），防止这条退化成
    /// 「恒 false」而照样绿（L8）。probe 参数写死为一个普通词，
    /// 不依赖各平台 `echo --version` 的行为差异。
    #[test]
    fn probe_available_is_false_for_missing_command() {
        let missing = demo_plugin("definitely-not-a-real-cli-xyz", Vec::new(), Vec::new());
        assert!(!missing.probe_available(), "不存在的命令探活竟返回 true");

        let real = demo_plugin(
            "/bin/echo",
            Vec::new(),
            vec!["neotrix-probe-ok".to_string()],
        );
        assert!(real.probe_available(), "/bin/echo 探活竟返回 false");
    }

    /// F0 第 4 步：`--workdir` **CLI 覆盖 descriptor** 的优先级，三个分支各钉一条。
    ///
    /// 为什么可能失败：任一分支写反（无条件覆盖成 `None`、或 `Some` 判断
    /// 写成「descriptor 有值才用 CLI」）都会让其中一条变红。
    #[test]
    fn merge_cli_workdir_cli_overrides_descriptor() {
        let desc = std::path::PathBuf::from("/from/descriptor");
        let cli = std::path::PathBuf::from("/from/cli");

        // 分支 1：两边都有 ⇒ CLI 赢。
        let mut p = demo_plugin("/bin/true", Vec::new(), Vec::new()).with_cwd(&desc);
        p.merge_cli_workdir(Some(&cli));
        assert_eq!(
            p.cwd.as_deref(),
            Some(cli.as_path()),
            "两边都有时必须 CLI 赢"
        );

        // 分支 2：只有 descriptor 有 ⇒ 保留 descriptor 的（CLI 不得把 cwd 清空）。
        let mut p = demo_plugin("/bin/true", Vec::new(), Vec::new()).with_cwd(&desc);
        p.merge_cli_workdir(None);
        assert_eq!(
            p.cwd.as_deref(),
            Some(desc.as_path()),
            "CLI 没给 --workdir 时必须保留 descriptor 的 cwd"
        );

        // 分支 3：只有 CLI 有 ⇒ 用 CLI 的（并把 descriptor 的 None 填上）。
        let mut p = demo_plugin("/bin/true", Vec::new(), Vec::new());
        assert!(p.cwd.is_none(), "分支 3 的前置条件：descriptor 的 cwd 应为空");
        p.merge_cli_workdir(Some(&cli));
        assert_eq!(
            p.cwd.as_deref(),
            Some(cli.as_path()),
            "只有 CLI 给了 --workdir 时必须用 CLI 的"
        );
    }

    /// descriptor 默认路径：无 `cwd` 字段的旧 descriptor 必须照旧载入为 `None`
    /// （= 继承调用方 cwd，向后兼容），写了 `cwd` 的必须解析出来。
    ///
    /// 为什么可能失败：把字段改成**无默认**（漏 `#[serde(default)]`）会让旧
    /// descriptor 直接反序列化失败 → 下面第一条 panic。
    #[test]
    fn descriptor_cwd_is_optional_and_parsed() {
        let legacy = r#"{"name":"legacy","command":"/bin/true","args":[]}"#;
        let p: ExternalCliPlugin = match serde_json::from_str(legacy) {
            Ok(p) => p,
            Err(e) => panic!("无 cwd 字段的旧 descriptor 应可载入：{e}"),
        };
        assert!(p.cwd.is_none(), "缺 cwd 字段时必须解析为 None（继承调用方 cwd）");

        let pinned = r#"{"name":"pinned","command":"/bin/true","cwd":"/srv/work"}"#;
        let q: ExternalCliPlugin = match serde_json::from_str(pinned) {
            Ok(p) => p,
            Err(e) => panic!("带 cwd 的 descriptor 应可载入：{e}"),
        };
        assert_eq!(
            q.cwd.as_deref(),
            Some(std::path::Path::new("/srv/work")),
            "descriptor 里写的 cwd 必须被解析出来"
        );
    }

    // ── F1：调用方透传参数（--agent-arg 的消费端）─────────────────────────

    /// F1 第 ① 条断言：多个透传参数**按调用方给的顺序**累积、**追加在**
    /// descriptor 的 `args` 之后。
    ///
    /// 为什么可能失败：实现里任何一次「排序 / 去重 / 前置到 descriptor 参数之前 /
    /// 只保留最后一个」都会让下面第一条变红。第二条钉住「无透传时行为不变」
    /// ——F1 是新增能力，不是替换旧行为。
    #[test]
    fn launch_argv_appends_extra_args_in_order() {
        let plugin = demo_plugin(
            "/bin/true",
            vec!["--trust-agents".to_string()],
            vec!["--version".to_string()],
        );
        let extra: Vec<String> = ["--continue", "conv-42", "--continue", "conv-7"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let want: Vec<String> = ["--trust-agents", "--continue", "conv-42", "--continue", "conv-7"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            plugin.launch_argv(&extra),
            want,
            "透传参数必须按出现顺序累积并追加在 descriptor args 之后（不排序/不去重/不前置）"
        );
        // 重复项**不得**被去重：`--continue` 两次出现是调用方的表达，
        // 这条断言就是为了让「顺手 dedup」的实现红掉。
        assert_eq!(
            plugin
                .launch_argv(&extra)
                .iter()
                .filter(|a| **a == "--continue")
                .count(),
            2,
            "重复的透传参数不得被去重"
        );
        assert_eq!(
            plugin.launch_argv(&[]),
            vec!["--trust-agents".to_string()],
            "没有透传参数时启动 argv 必须与 descriptor 声明逐字一致（F1 不改旧行为）"
        );
    }

    /// F1 第 ② 条断言（**实证**）：透传参数真的进了子进程的 argv，且顺序不变。
    ///
    /// 手法与 F0 的 `launch_applies_descriptor_cwd` 同款：`/bin/sh -c <script>
    /// sh <extra...>` 里 `$0=sh`、`$1..=$@` 即透传参数，脚本把 `"$@"` 逐行
    /// 写进 cwd 内的 marker 文件。纯函数断言（上面那条）只能证明**我们算出了**
    /// 什么，这一条证明**子进程真收到了**什么 —— 中间隔着 `Command::args` 与
    /// OS exec，漏掉任一环都红。
    /// ⚠️ 同时钉住 F0+F1 的组合：marker 落在 `tmp` ⇒ 带透传参数启动时 cwd 仍生效。
    /// ⛔ 刻意不沿用「读不到就 return」的容错写法（L8：绿色≠有效）。
    #[test]
    fn launch_with_passes_extra_args_to_child() {
        let tmp = unique_tmp_dir("argv-launch");
        if std::fs::create_dir_all(&tmp).is_err() {
            eprintln!("跳过：无法创建 {}", tmp.display());
            return;
        }
        if !std::path::Path::new("/bin/sh").exists() {
            eprintln!("跳过：平台无 /bin/sh");
            let _ = std::fs::remove_dir_all(&tmp);
            return;
        }
        let plugin = demo_plugin(
            "/bin/sh",
            vec![
                "-c".to_string(),
                "printf '%s\\n' \"$@\" > nt_argv_probe.txt".to_string(),
                "sh".to_string(),
            ],
            Vec::new(),
        )
        .with_cwd(&tmp);
        let extra: Vec<String> = ["--continue", "conv-42"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut child = match plugin.launch_with(&extra) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("跳过：spawn /bin/sh 失败：{e}");
                let _ = std::fs::remove_dir_all(&tmp);
                return;
            }
        };
        let _ = child.wait();

        let seen = match std::fs::read_to_string(&tmp.join("nt_argv_probe.txt")) {
            Ok(s) => s,
            Err(e) => {
                panic!(
                    "launch_with 没把透传参数送进子进程（cwd {} 内没有 marker 文件，{e}）",
                    tmp.display()
                );
            }
        };
        assert_eq!(
            seen.lines().collect::<Vec<&str>>(),
            vec!["--continue", "conv-42"],
            "子进程收到的透传参数必须与传进去的完全一致且顺序不变"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// F1 第 ② 条断言的另一半：透传参数**不污染**探活。
    ///
    /// 为什么可能失败：只要有人把透传参数实现成「并进 `self.args`」（这是最省事
    /// 的写法，且看起来能工作），`probe_argv()` 就会跟着拿到 `--continue <id>`，
    /// 而 `probe_args` 就形同虚设 —— 探活失败的语义会从「CLI 装不上」漂移成
    /// 「这次会话参数不对」。
    /// ⛔ 末尾的**正例对照**是防退化的关键（L8）：没有它，一个「透传压根没实现」
    /// 的版本也能让上面两条 `!contains` 断言恒绿。
    #[test]
    fn probe_argv_never_sees_launch_extra_args() {
        let plugin = demo_plugin(
            "freebuff",
            vec!["--trust-agents".to_string()],
            vec!["--version".to_string()],
        );
        let extra: Vec<String> = ["--continue", "no-such-conversation-xyz"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        for a in &extra {
            assert!(
                !plugin.probe_argv().iter().any(|p| p == a),
                "透传参数 `{a}` 不允许出现在探活 argv 里（探活只准跑 probe_args）"
            );
        }
        assert_eq!(
            plugin.probe_argv(),
            vec!["--version".to_string()],
            "探活 argv 必须逐字等于 probe_args（或缺省 --version）"
        );
        // 空 probe_args 时的缺省分支同样不许被透传参数顶掉。
        let bare = demo_plugin("freebuff", vec!["--trust-agents".to_string()], Vec::new());
        assert_eq!(
            bare.probe_argv(),
            vec!["--version".to_string()],
            "probe_args 为空时探活必须退回 --version"
        );

        // 正例对照：透传参数**确实**进了启动 argv ⇒ 上面两条不是空转。
        let launched = plugin.launch_argv(&extra);
        for a in &extra {
            assert!(
                launched.iter().any(|p| p == a),
                "对照：透传参数 `{a}` 必须在启动 argv 里（否则本测试的空转防护失效）"
            );
        }
    }

    /// F1 的第三条要求：透传参数只对 `--agent` 路径生效，缺 `--agent` 必须报错。
    ///
    /// 为什么可能失败：三个分支任一写反都红 —— 特别是把「无 agent」也放行的
    /// 实现，那会让 `--agent-arg` 在模型池/chat 线上被**静默丢弃**，而用户以为
    /// `--continue` 已经传进去了。
    #[test]
    fn ensure_agent_args_need_agent_rejects_args_without_agent() {
        let one = vec!["--continue".to_string()];

        // 分支 1：给了 --agent ⇒ 放行。
        assert!(
            ensure_agent_args_need_agent(Some("freebuff"), &one).is_ok(),
            "给了 --agent 时透传参数必须放行"
        );

        // 分支 2：没给 --agent 却给了透传 ⇒ 必须报错（⛔ 不得静默忽略）。
        let err = match ensure_agent_args_need_agent(None, &one) {
            Err(e) => e,
            Ok(()) => panic!("没给 --agent 却带透传参数时必须报错，否则参数被静默丢弃"),
        };
        assert!(
            err.contains("--agent"),
            "错误信息要指回 --agent（让用户知道怎么改）：{err}"
        );

        // 分支 3：两个都没给 ⇒ 放行（普通 goal 路径不受 F1 影响）。
        assert!(
            ensure_agent_args_need_agent(None, &[]).is_ok(),
            "既没 agent 也没透传参数时必须放行"
        );
    }
}
