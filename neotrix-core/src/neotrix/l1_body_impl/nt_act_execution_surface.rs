//! # NT-ACT execution_surface — 权威状态执行面
//!
//! 吸收源: cloudflare/computer — 虚拟文件系统执行面: Durable Object 持 SQLite
//! 权威状态, 经可插拔 `workspace.runtime` 投影到执行环境 (Container FUSE 挂载 /
//! Isolate shell), sandbox-side daemon 经 RPC 同步回写。
//!
//! 骨架阶段 (C0): 状态投影 + 可插拔 runtime + 同步接口, 已接 `/sandbox exec-surface`
//! 生产路径; 待完善: KB/SQLite 真权威状态, FUSE 容器后端, RPC 回写, 网络策略。
//!
//! 设计对齐: KB 为权威状态 (R-P79 单一事实源), 执行面是投影 — 与
//! ActionSandbox (审批) 互补: sandbox 决定"能否做", surface 决定"在哪执行"。

use std::collections::BTreeMap;
use std::fmt;

/// 执行结果。
#[derive(Debug, Clone)]
pub struct RuntimeResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl RuntimeResult {
    pub fn ok(stdout: &str) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }
    pub fn err(msg: &str) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: msg.to_string(),
        }
    }
}

/// 可插拔执行 runtime (Container / Isolate)。
pub trait ExecutionRuntime: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn run(&self, cmd: &str, cwd: &str) -> RuntimeResult;
    /// 默认工作目录 (状态投影根目录)。
    fn default_cwd(&self) -> String;
}

/// 骨架容器后端 — 占位实现，投影到本地受限目录。
pub struct ContainerRuntime {
    pub root: String,
}

impl Default for ContainerRuntime {
    fn default() -> Self {
        Self {
            root: "/tmp/nt_surface".to_string(),
        }
    }
}

impl ExecutionRuntime for ContainerRuntime {
    fn name(&self) -> &'static str {
        "container"
    }
    fn is_available(&self) -> bool {
        std::path::Path::new(&self.root).is_dir() || std::fs::create_dir_all(&self.root).is_ok()
    }
    fn default_cwd(&self) -> String {
        self.root.clone()
    }
    fn run(&self, cmd: &str, cwd: &str) -> RuntimeResult {
        use std::process::Command;
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(cwd)
            .output();
        match output {
            Ok(o) => RuntimeResult {
                exit_code: o.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
            },
            Err(e) => RuntimeResult::err(&format!("spawn failed: {e}")),
        }
    }
}

/// 骨架 isolate 后端 — 仅状态占位。
pub struct IsolateRuntime;
impl ExecutionRuntime for IsolateRuntime {
    fn name(&self) -> &'static str {
        "isolate"
    }
    fn is_available(&self) -> bool {
        false
    }
    fn run(&self, cmd: &str, _cwd: &str) -> RuntimeResult {
        RuntimeResult::err(&format!("isolate runtime not available (cmd={cmd})"))
    }
    fn default_cwd(&self) -> String {
        "/tmp/nt_surface".to_string()
    }
}

/// 权威状态 — 骨架用内存 BTreeMap, 提供与 KB 同步的挂点。
#[derive(Debug, Clone, Default)]
pub struct SurfaceState {
    /// 键 → 值 (文件系统投影的权威表示)。
    entries: BTreeMap<String, String>,
    revision: u64,
}

impl SurfaceState {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            revision: 0,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
        self.revision += 1;
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// 待接线: 与 KB nodes/metadata 双向同步 (R-P79 后续迭代)。
    pub fn sync_from_kb(&mut self, _kb: &str) {
        // TODO: 从 KB 拉权威键值。
    }
}

/// 执行面 — 权威状态 + 执行 runtime。
pub struct ExecutionSurface {
    pub runtime: Box<dyn ExecutionRuntime>,
    pub state: SurfaceState,
    pub cwd: String,
}

impl ExecutionSurface {
    pub fn new(runtime: Box<dyn ExecutionRuntime>) -> Self {
        let cwd = runtime.default_cwd();
        Self {
            runtime,
            state: SurfaceState::new(),
            cwd,
        }
    }

    pub fn with_state(mut self, state: SurfaceState) -> Self {
        self.state = state;
        self
    }

    /// 执行命令 (骨架: 状态投影到 runtime 根目录后执行)。
    pub fn execute(&mut self, cmd: &str) -> RuntimeResult {
        if !self.runtime.is_available() {
            return RuntimeResult::err(&format!("runtime '{}' unavailable", self.runtime.name()));
        }
        // 投影: 将权威状态写入 runtime 可见目录 (骨架简化)。
        self.state.set("last_cmd", cmd.to_string());
        let result = self.runtime.run(cmd, &self.cwd);
        self.state.set("last_exit", result.exit_code.to_string());
        self.state.set("last_stdout", result.stdout.clone());
        result
    }

    pub fn health(&self) -> f64 {
        if self.runtime.is_available() {
            1.0
        } else {
            0.0
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "ExecutionSurface(runtime={}, cwd={}, entries={}, revision={}, health={:.1})",
            self.runtime.name(),
            self.cwd,
            self.state.keys().len(),
            self.state.revision(),
            self.health()
        )
    }
}

impl fmt::Display for ExecutionSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_runtime_executes_and_returns_stdout() {
        let surface = ExecutionSurface::new(Box::new(ContainerRuntime::default()));
        let r = surface.runtime.run("echo nt-surface-ok", "/tmp");
        assert_eq!(r.exit_code, 0);
        assert!(r.stdout.contains("nt-surface-ok"));
    }

    #[test]
    fn execute_records_state_projection() {
        let mut surface = ExecutionSurface::new(Box::new(ContainerRuntime::default()));
        let r = surface.execute("echo hello");
        assert_eq!(r.exit_code, 0);
        assert_eq!(surface.state.get("last_exit"), Some("0"));
        assert_eq!(surface.state.get("last_cmd"), Some("echo hello"));
        assert!(surface.state.revision() >= 2);
    }

    #[test]
    fn unavailable_runtime_errors_cleanly() {
        let mut surface = ExecutionSurface::new(Box::new(IsolateRuntime));
        let r = surface.execute("rm -rf /");
        assert_eq!(r.exit_code, 1);
        assert!(r.stderr.contains("unavailable"));
    }

    #[test]
    fn health_reflects_runtime() {
        let container = ExecutionSurface::new(Box::new(ContainerRuntime::default()));
        let isolate = ExecutionSurface::new(Box::new(IsolateRuntime));
        assert_eq!(container.health(), 1.0);
        assert_eq!(isolate.health(), 0.0);
    }

    #[test]
    fn summary_reports_state() {
        let surface = ExecutionSurface::new(Box::new(ContainerRuntime::default()));
        let s = surface.summary();
        assert!(s.contains("runtime=container"));
        assert!(s.contains("entries="));
    }
}
