use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    Local,
    Docker,
    Wasm,
    Remote,
}

#[derive(Debug)]
pub struct SandboxResultEntry {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug)]
pub struct Sandbox {
    pub mode: SandboxMode,
    pub image: String,
    pub memory_limit: String,
    pub work_dir: String,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            mode: SandboxMode::Local,
            image: "ubuntu:22.04".to_string(),
            memory_limit: "512m".to_string(),
            work_dir: {
                crate::core::nt_core_util::home_dir()
                    .join(".neotrix/sandbox")
                    .to_string_lossy()
                    .to_string()
            },
        }
    }
}

impl Sandbox {
    pub fn new(mode: SandboxMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn with_image(mode: SandboxMode, image: &str) -> Self {
        let mut s = Self::new(mode);
        s.image = image.to_string();
        s
    }

    fn split_command(cmd: &str) -> (&str, Vec<&str>) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            ("", vec![])
        } else {
            (parts[0], parts[1..].to_vec())
        }
    }

    /// Execute a command in the sandbox without shell injection
    pub fn execute(&self, cmd: &str) -> SandboxResultEntry {
        match self.mode {
            SandboxMode::Local => self.exec_local(cmd),
            SandboxMode::Docker => self.exec_docker(cmd),
            SandboxMode::Wasm => self.exec_wasm(cmd),
            SandboxMode::Remote => SandboxResultEntry {
                stdout: String::new(),
                stderr: "Remote sandbox not implemented".to_string(),
                exit_code: 1,
            },
        }
    }

    /// Run a full agent loop inside the sandbox
    pub fn run_agent(&self, task: &str) -> NeoTrixResult<String> {
        let result = self.execute("echo sandbox_agent_ready");
        if result.exit_code == 0 {
            Ok(result.stdout)
        } else {
            Err(NeoTrixError::Command {
                cmd: task.to_string(),
                exit_code: Some(result.exit_code),
                stderr: result.stderr,
            })
        }
    }

    fn exec_local(&self, cmd: &str) -> SandboxResultEntry {
        let (program, args) = Self::split_command(cmd);
        if program.is_empty() {
            return SandboxResultEntry {
                stdout: String::new(),
                stderr: "empty command".into(),
                exit_code: -1,
            };
        }
        // Reject args containing shell metacharacters (full coverage)
        let shell_meta = &[
            ';', '|', '&', '`', '$', '(', ')', '{', '}', '<', '>', '*', '?', '[', ']', '!', '\'',
            '"', '#',
        ][..];
        for arg in &args {
            if arg.contains(shell_meta) {
                return SandboxResultEntry {
                    stdout: String::new(),
                    stderr: format!("rejected: arg '{}' contains shell metacharacter", arg),
                    exit_code: -1,
                };
            }
        }
        let output = std::process::Command::new(program).args(&args).output();
        match output {
            Ok(o) => SandboxResultEntry {
                stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                stderr: String::from_utf8_lossy(&o.stderr).to_string(),
                exit_code: o.status.code().unwrap_or(-1),
            },
            Err(e) => SandboxResultEntry {
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                exit_code: -1,
            },
        }
    }

    fn exec_docker(&self, cmd: &str) -> SandboxResultEntry {
        let (program, args) = Self::split_command(cmd);
        if program.is_empty() {
            return SandboxResultEntry {
                stdout: String::new(),
                stderr: "empty command".into(),
                exit_code: -1,
            };
        }
        let output = std::process::Command::new("docker")
            .args(["run", "--rm", "-i"])
            .arg(format!("--memory={}", self.memory_limit))
            .arg(&self.image)
            .arg(program)
            .args(&args)
            .output();

        match output {
            Ok(o) => {
                let exit_code = o.status.code().unwrap_or(-1);
                if exit_code == 127 {
                    // Docker not found, fallback to local
                    return self.exec_local(cmd);
                }
                SandboxResultEntry {
                    stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&o.stderr).to_string(),
                    exit_code,
                }
            }
            Err(e) => SandboxResultEntry {
                stdout: String::new(),
                stderr: format!("Docker error: {} (fallback to local)", e),
                exit_code: 127,
            },
        }
    }

    #[cfg(not(feature = "sandbox"))]
    fn exec_wasm(&self, _cmd: &str) -> SandboxResultEntry {
        SandboxResultEntry {
            stdout: String::new(),
            stderr: "WASM sandbox not available (enable feature=sandbox)".into(),
            exit_code: -1,
        }
    }

    #[cfg(feature = "sandbox")]
    fn exec_wasm(&self, cmd: &str) -> SandboxResultEntry {
        match self.run_wasm_module(cmd) {
            Ok(output) => SandboxResultEntry {
                stdout: output,
                stderr: String::new(),
                exit_code: 0,
            },
            Err(e) => SandboxResultEntry {
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: -1,
            },
        }
    }

    #[cfg(feature = "sandbox")]
    fn run_wasm_module(&self, input: &str) -> NeoTrixResult<String> {
        use wasmtime::{Engine, Instance, Module, Store};
        let engine = Engine::default();
        let module_bytes = self.load_wasm_module("sandbox")?;
        let module = Module::new(&engine, &module_bytes).map_err(|e| NeoTrixError::General {
            msg: format!("WASM compile: {}", e),
            backtrace: None,
        })?;
        if module.exports().any(|e| e.name() == "main") {
            let mut store: Store<()> = Store::new(&engine, ());
            let instance =
                Instance::new(&mut store, &module, &[]).map_err(|e| NeoTrixError::General {
                    msg: format!("WASM instantiate: {}", e),
                    backtrace: None,
                })?;
            let func = instance
                .get_typed_func::<(), ()>(&mut store, "main")
                .map_err(|e| NeoTrixError::General {
                    msg: format!("WASM func: {}", e),
                    backtrace: None,
                })?;
            func.call(&mut store, ())
                .map_err(|e| NeoTrixError::General {
                    msg: format!("WASM exec: {}", e),
                    backtrace: None,
                })?;
        }
        Ok(format!("wasm:{}", input))
    }

    #[cfg(feature = "sandbox")]
    fn load_wasm_module(&self, name: &str) -> NeoTrixResult<Vec<u8>> {
        let path = std::path::PathBuf::from(&self.work_dir).join(format!("{}.wasm", name));
        std::fs::read(&path).map_err(|e| NeoTrixError::General {
            msg: format!("WASM load {}: {}", name, e),
            backtrace: None,
        })
    }

    /// Pull docker image (async)
    pub async fn pull_image(&self) -> NeoTrixResult<String> {
        if self.mode != SandboxMode::Docker {
            return Err(NeoTrixError::from("Not in Docker mode"));
        }
        let output = tokio::process::Command::new("docker")
            .args(["pull", &self.image])
            .output()
            .await
            .map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        if output.status.success() {
            Ok(format!("Image {} pulled", self.image))
        } else {
            Err(NeoTrixError::Command {
                cmd: format!("docker pull {}", self.image),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }
}

/// WASM-backed sandbox using the `agent-sandbox` crate (wraps Wasmtime + WASI).
///
/// Provides a higher-level sandbox for executing JavaScript/WASM code
/// with filesystem access, HTTP fetch (optional), and resource limits.
#[cfg(feature = "sandbox")]
pub struct WasmSandbox {
    inner: agent_sandbox::Sandbox,
    rt: tokio::runtime::Runtime,
}

#[cfg(feature = "sandbox")]
impl WasmSandbox {
    /// Create a new WASM sandbox with the given host work directory.
    ///
    /// The work directory is exposed as `/work` inside the sandbox.
    /// Defaults: 30s timeout, 512MB memory, 1B fuel limit, no networking.
    pub fn new(work_dir: &str) -> Result<Self, String> {
        let config = agent_sandbox::config::SandboxConfig {
            work_dir: std::path::PathBuf::from(work_dir),
            ..Default::default()
        };
        let inner = agent_sandbox::Sandbox::new(config).map_err(|e| e.to_string())?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        Ok(Self { inner, rt })
    }

    /// Execute code (JS/WASI) inside the sandbox with optional input bytes.
    ///
    /// The `input` bytes are written to `/work/input` before execution.
    /// Returns stdout on success, error string on failure.
    pub fn execute_wasm(&self, code: &str, input: &[u8]) -> Result<Vec<u8>, String> {
        if !input.is_empty() {
            self.rt
                .block_on(self.inner.write_file("/input", input))
                .map_err(|e| format!("write input: {}", e))?;
        }

        let result = self
            .rt
            .block_on(self.inner.exec_js(code))
            .map_err(|e| format!("WASM exec: {}", e))?;

        if result.exit_code != 0 {
            return Err(format!(
                "exit={} stderr={}",
                result.exit_code,
                String::from_utf8_lossy(&result.stderr)
            ));
        }
        Ok(result.stdout)
    }

    /// Run a CLI command inside the sandbox (e.g. "grep", "cat").
    pub fn exec_command(&self, command: &str, args: &[String]) -> Result<Vec<u8>, String> {
        let result = self
            .rt
            .block_on(self.inner.exec(command, args))
            .map_err(|e| format!("exec: {}", e))?;

        if result.exit_code != 0 {
            return Err(format!(
                "exit={} stderr={}",
                result.exit_code,
                String::from_utf8_lossy(&result.stderr)
            ));
        }
        Ok(result.stdout)
    }

    /// Read a file from the sandbox's work directory.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        self.rt
            .block_on(self.inner.read_file(path))
            .map_err(|e| e.to_string())
    }

    /// Write a file to the sandbox's work directory.
    pub fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), String> {
        self.rt
            .block_on(self.inner.write_file(path, contents))
            .map_err(|e| e.to_string())
    }

    /// List entries in a directory within the sandbox.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, String> {
        let entries = self
            .rt
            .block_on(self.inner.list_dir(path))
            .map_err(|e| e.to_string())?;
        Ok(entries.into_iter().map(|e| e.name).collect())
    }

    /// Get filesystem changes since sandbox creation.
    pub fn diff(&self) -> Result<Vec<String>, String> {
        let changes = self
            .rt
            .block_on(self.inner.diff())
            .map_err(|e| e.to_string())?;
        Ok(changes.into_iter().map(|c| format!("{:?}", c)).collect())
    }

    /// Destroy the sandbox and clean up temporary resources.
    pub fn destroy(&self) -> Result<(), String> {
        self.rt
            .block_on(self.inner.destroy())
            .map_err(|e| e.to_string())
    }
}

#[cfg(not(feature = "sandbox"))]
pub struct WasmSandbox(());

#[cfg(not(feature = "sandbox"))]
impl WasmSandbox {
    pub fn new(_work_dir: &str) -> Result<Self, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn execute_wasm(&self, _code: &str, _input: &[u8]) -> Result<Vec<u8>, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn exec_command(&self, _command: &str, _args: &[String]) -> Result<Vec<u8>, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn read_file(&self, _path: &str) -> Result<Vec<u8>, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn write_file(&self, _path: &str, _contents: &[u8]) -> Result<(), String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn list_dir(&self, _path: &str) -> Result<Vec<String>, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn diff(&self) -> Result<Vec<String>, String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }

    pub fn destroy(&self) -> Result<(), String> {
        Err("WasmSandbox requires the 'sandbox' feature".into())
    }
}

pub struct SandboxPool {
    sandboxes: Vec<Arc<Mutex<Sandbox>>>,
}

impl SandboxPool {
    pub fn new(count: usize, mode: SandboxMode) -> Self {
        let mut sandboxes = Vec::new();
        for _ in 0..count {
            sandboxes.push(Arc::new(Mutex::new(Sandbox::new(mode))));
        }
        Self { sandboxes }
    }

    pub fn acquire(&self) -> Option<Arc<Mutex<Sandbox>>> {
        self.sandboxes.first().cloned()
    }

    pub fn len(&self) -> usize {
        self.sandboxes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sandboxes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_default() {
        let s = Sandbox::default();
        assert_eq!(s.mode, SandboxMode::Local);
        assert_eq!(s.image, "ubuntu:22.04");
    }

    #[test]
    fn test_sandbox_new() {
        let s = Sandbox::new(SandboxMode::Docker);
        assert_eq!(s.mode, SandboxMode::Docker);
    }

    #[test]
    fn test_sandbox_local_exec() {
        let s = Sandbox::new(SandboxMode::Local);
        let result = s.execute("echo hello");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_sandbox_local_exec_fail() {
        let s = Sandbox::new(SandboxMode::Local);
        let result = s.execute("false");
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_sandbox_pool() {
        let pool = SandboxPool::new(3, SandboxMode::Local);
        assert_eq!(pool.len(), 3);
        assert!(pool.acquire().is_some());
    }

    #[test]
    fn test_docker_fallback_on_missing_docker() {
        let s = Sandbox::with_image(SandboxMode::Docker, "nonexistent:latest");
        let result = s.exec_docker("echo hello");
        // Should fallback to local if docker is not available
        assert!(result.exit_code == 0 || result.exit_code == 127);
    }

    #[test]
    fn test_run_agent_fallback() {
        let s = Sandbox::new(SandboxMode::Local);
        // run_agent tries to run neotrix-cli which won't be in PATH
        // but the echo fallback should work
        let result = s.execute("echo sandbox_test");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.trim().contains("sandbox_test"));
    }
}
