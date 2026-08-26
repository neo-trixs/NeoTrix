use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;

use super::{CloudResult, CloudRuntime, ResourceUsage};
use super::provider::CloudSandboxProvider;

pub struct LocalDockerProvider {
    log_buffers: Mutex<HashMap<String, Vec<String>>>,
    uploaded_files: Mutex<HashMap<String, HashMap<String, Vec<u8>>>>,
}

impl Default for LocalDockerProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalDockerProvider {
    pub fn new() -> Self {
        Self {
            log_buffers: Mutex::new(HashMap::new()),
            uploaded_files: Mutex::new(HashMap::new()),
        }
    }

    fn image_for(runtime: CloudRuntime) -> &'static str {
        match runtime {
            CloudRuntime::Python3 => "python:3.11-slim",
            CloudRuntime::Node18 => "node:18-alpine",
            CloudRuntime::RustStable => "rust:latest",
            CloudRuntime::Go1_21 => "golang:1.21-alpine",
            CloudRuntime::GenericLinux => "ubuntu:22.04",
        }
    }

    fn wrap_command(runtime: CloudRuntime, code: &str) -> (String, Vec<String>) {
        match runtime {
            CloudRuntime::Python3 => ("python3".into(), vec!["-c".into(), code.into()]),
            CloudRuntime::Node18 => ("node".into(), vec!["-e".into(), code.into()]),
            CloudRuntime::RustStable => {
                let script = format!(
                    "cat > /tmp/main.rs << 'RUSTEOF'\nfn main() {{ {} }}\nRUSTEOF\nrustc /tmp/main.rs -o /tmp/out && /tmp/out",
                    code
                );
                ("sh".into(), vec!["-c".into(), script])
            }
            CloudRuntime::Go1_21 => {
                let script = format!(
                    "cat > /tmp/main.go << 'GOEOF'\npackage main\n\nfunc main() {{ {} }}\nGOEOF\ngo run /tmp/main.go",
                    code
                );
                ("sh".into(), vec!["-c".into(), script])
            }
            CloudRuntime::GenericLinux => ("bash".into(), vec!["-c".into(), code.into()]),
        }
    }

    /// Pure assembly of the `docker run` argument vector for one execution.
    /// Hardened per the absorbed grok-bot local-sandbox pattern:
    /// - no network namespace egress (`--network none`);
    /// - read-only root filesystem; the only writable surface is a size-capped
    ///   tmpfs at `/tmp` (wrap_command scratch files live there);
    /// - host uploads are content-addressed per session and mounted `:ro`.
    fn run_args(
        session_id: &str,
        has_uploads: bool,
        env: &HashMap<String, String>,
    ) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "run".into(),
            "--rm".into(),
            "--network".into(),
            "none".into(),
            "--read-only".into(),
            "--tmpfs".into(),
            "/tmp:rw,size=64m,noexec,nosuid".into(),
            "--memory".into(),
            "512m".into(),
            "--cpus".into(),
            "1".into(),
            "--pids-limit".into(),
            "128".into(),
            "--label".into(),
            format!("neotrix-session={}", session_id),
        ];

        if has_uploads {
            let tmpdir = format!("/tmp/neotrix-upload-{}", session_id);
            args.push("-v".into());
            args.push(format!("{}:/workspace:ro", tmpdir));
        }

        // 注入 vault 凭据为容器环境变量。仅记录 key 名 — value 永不进日志/遥测。
        if !env.is_empty() {
            let mut keys: Vec<&str> = env.keys().map(|k| k.as_str()).collect();
            keys.sort();
            log::info!(
                "[sandbox] injecting {} vault secret(s) into container env (keys: {})",
                env.len(),
                keys.join(", ")
            );
            for (k, v) in env {
                args.push("--env".into());
                args.push(format!("{}={}", k, v));
            }
        }
        args
    }

    /// Structural fail-closed gate over docker args (defense-in-depth: even
    /// though we construct args ourselves, future edits cannot silently weaken
    /// the boundary). Enforces the absorbed loopback-only + read-only-mount
    /// invariants:
    /// - `--privileged` and `--network host` are hard denials;
    /// - every bind mount (`-v`/`--volume`/`--mount`) must be read-only;
    /// - any published port (`-p`/`--publish`) must be bound to `127.0.0.1` —
    ///   a bare `"host:container"` spec binds 0.0.0.0 and is denied.
    pub fn validate_docker_args(args: &[String]) -> Result<(), String> {
        let mut i = 0;
        while i < args.len() {
            let flag = args[i].as_str();
            match flag {
                "--privileged" => {
                    return Err("docker sandbox denies `--privileged`".into());
                }
                "--network=host" | "--net=host" => {
                    return Err("docker sandbox denies `--network host`".into());
                }
                "-v" | "--volume" | "--mount" => {
                    let spec =
                        args.get(i + 1).ok_or("docker sandbox: volume flag missing value")?;
                    Self::validate_mount_spec(spec)?;
                    i += 2;
                    continue;
                }
                "-p" | "--publish" => {
                    let spec =
                        args.get(i + 1).ok_or("docker sandbox: publish flag missing value")?;
                    Self::validate_publish_spec(spec)?;
                    i += 2;
                    continue;
                }
                "--network" | "--net" => {
                    let mode = args.get(i + 1).map(|s| s.as_str()).unwrap_or("");
                    if mode == "host" {
                        return Err("docker sandbox denies `--network host`".into());
                    }
                    i += 2;
                    continue;
                }
                _ => {}
            }
            if let Some(spec) = flag.strip_prefix("--publish=") {
                Self::validate_publish_spec(spec)?;
            } else if let Some(spec) = flag.strip_prefix("--volume=") {
                Self::validate_mount_spec(spec)?;
            }
            i += 1;
        }
        Ok(())
    }

    /// Bind mounts must be read-only. Accepted forms end with an option list
    /// containing the `ro` token (`:ro`, `:ro,z`, ...) or use the `--mount`
    /// long form with `,readonly`. A trailing `rw`, or a bare two-part spec
    /// (which Docker defaults to rw), is denied.
    fn validate_mount_spec(spec: &str) -> Result<(), String> {
        if spec.contains(",readonly") || spec.contains("=readonly") {
            return Ok(());
        }
        let opts = spec.rsplit(':').next().unwrap_or("");
        let has_ro = opts.split(',').any(|o| o.trim() == "ro");
        if has_ro {
            Ok(())
        } else {
            Err(format!(
                "docker sandbox mount must be read-only (`:ro`): got `{}`",
                spec
            ))
        }
    }

    /// Published ports must be loopback-bound. `"127.0.0.1:8080:80"` passes;
    /// `"8080:80"` or `"0.0.0.0:8080:80"` would expose on all interfaces and
    /// is denied fail-closed.
    fn validate_publish_spec(spec: &str) -> Result<(), String> {
        let host_part = spec.split(':').next().unwrap_or("");
        if host_part == "127.0.0.1" || host_part == "[::1]" || host_part == "localhost" {
            Ok(())
        } else {
            Err(format!(
                "docker sandbox port publish must bind loopback (127.0.0.1): got `{}`",
                spec
            ))
        }
    }
}

/// Render docker args for logging with secret values masked (Redactor).
/// Env values are never logged raw — only after secret redaction.
fn redact_docker_args(args: &[String]) -> String {
    crate::neotrix::l1_body_impl::nt_shield::redaction::redact_secrets(&args.join(" "))
}

#[async_trait]
impl CloudSandboxProvider for LocalDockerProvider {
    fn name(&self) -> &'static str {
        "docker"
    }

    async fn execute(
        &self,
        session_id: &str,
        code: &str,
        runtime: CloudRuntime,
        env: &HashMap<String, String>,
    ) -> Result<CloudResult, String> {
        let image = Self::image_for(runtime);
        let (entrypoint, cmd_args) = Self::wrap_command(runtime, code);

        let has_uploads = self
            .uploaded_files
            .lock()
            .map(|f| f.contains_key(session_id))
            .unwrap_or(false);

        let mut docker_args = Self::run_args(session_id, has_uploads, env);
        docker_args.push(image.into());
        docker_args.push(entrypoint);
        docker_args.extend(cmd_args);

        // Fail-closed structural gate: loopback-only publish / ro mounts /
        // no privileged. Runs on every execution before any container starts.
        Self::validate_docker_args(&docker_args)?;

        log::debug!("[sandbox] docker run: {}", redact_docker_args(&docker_args));

        let start = std::time::Instant::now();
        let output = tokio::process::Command::new("docker")
            .args(&docker_args)
            .output()
            .await
            .map_err(|e| format!("Docker exec failed: {}", e))?;

        let elapsed = start.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);

        if let Ok(mut bufs) = self.log_buffers.lock() {
            let logs = bufs.entry(session_id.to_string()).or_default();
            logs.push(format!(
                "[exit={}] {}",
                exit_code,
                String::from_utf8_lossy(&output.stdout).trim(),
            ));
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stderr.is_empty() {
                logs.push(format!("[stderr] {}", stderr));
            }
        }

        Ok(CloudResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            execution_time: elapsed,
            resource_usage: ResourceUsage {
                cpu_time: elapsed.as_secs_f64(),
                memory_mb: 0.0,
                network_kb: 0.0,
            },
        })
    }

    async fn upload_file(
        &self,
        session_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> Result<(), String> {
        let tmpdir = format!("/tmp/neotrix-upload-{}", session_id);
        tokio::fs::create_dir_all(&tmpdir)
            .await
            .map_err(|e| format!("create upload dir: {}", e))?;

        let file_path = format!("{}/{}", tmpdir, path.trim_start_matches('/'));
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("create parent dir: {}", e))?;
        }
        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|e| format!("write upload file: {}", e))?;

        if let Ok(mut files) = self.uploaded_files.lock() {
            files
                .entry(session_id.to_string())
                .or_default()
                .insert(path.to_string(), data);
        }
        Ok(())
    }

    async fn download_result(&self, session_id: &str) -> Result<CloudResult, String> {
        let logs = self
            .log_buffers
            .lock()
            .map_err(|e| e.to_string())?
            .get(session_id)
            .cloned()
            .unwrap_or_default();

        Ok(CloudResult {
            stdout: logs.join("\n"),
            stderr: String::new(),
            exit_code: 0,
            execution_time: Duration::from_secs(0),
            resource_usage: ResourceUsage::default(),
        })
    }

    fn stream_logs(&self, session_id: &str) -> BoxStream<'static, String> {
        let logs = self
            .log_buffers
            .lock()
            .ok()
            .and_then(|b| b.get(session_id).cloned())
            .unwrap_or_default();
        stream::iter(logs).boxed()
    }

    async fn cancel(&self, session_id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args([
                "kill",
                "--filter",
                &format!("label=neotrix-session={}", session_id),
            ])
            .output()
            .await
            .map_err(|e| format!("docker kill: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No such container") {
                return Err(format!("docker kill failed: {}", stderr));
            }
        }
        Ok(())
    }

    /// Validate-before-connect gate: the docker daemon must be reachable
    /// before any workload is dispatched to this provider. A broken local
    /// environment surfaces as a clean fail-closed denial, never as a
    /// mid-flight container failure.
    async fn validate_ready(&self) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["info", "--format", "{{.ServerVersion}}"])
            .output()
            .await
            .map_err(|e| format!("docker sandbox unavailable (daemon not reachable): {}", e))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            log::info!("[sandbox] docker daemon validated (server {})", version);
            Ok(())
        } else {
            Err(format!(
                "docker sandbox unavailable (daemon not reachable): {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}

#[cfg(all(test, feature = "sandbox"))]
mod tests {
    use super::*;

    /// 负例: 注入的 secret 出现在日志命令串中时必须被 Redactor 遮蔽,
    /// 禁止明文泄露到日志/遥测/LLM 上下文。
    #[test]
    fn test_redactor_masks_secret_in_logged_command() {
        let secret = "sk-supersecret";
        let args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--env".to_string(),
            format!("NEOTRIX_VAULT_API_KEY={}", secret),
            "python:3.11-slim".to_string(),
            "python3".to_string(),
            "-c".to_string(),
            "print(1)".to_string(),
        ];
        let rendered = redact_docker_args(&args);
        assert!(!rendered.contains("sk-supersecret"), "secret must be masked in command log");
        assert!(rendered.contains("[REDACTED]"));
    }

    fn to_owned(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    /// 生产形态参数 (run_args + image + entrypoint) 必须通过结构门。
    #[test]
    fn test_production_args_pass_structural_gate() {
        let mut env = HashMap::new();
        env.insert("NEOTRIX_VAULT_API_KEY".to_string(), "sk-x".to_string());
        let mut args = LocalDockerProvider::run_args("sess-1", true, &env);
        args.push("python:3.11-slim".into());
        args.push("python3".into());
        args.push("-c".into());
        args.push("print(1)".into());
        LocalDockerProvider::validate_docker_args(&args).expect("production args must pass");
    }

    /// run_args 硬化结构: 无网络 / 只读根 fs / 上传挂载 :ro。
    #[test]
    fn test_run_args_hardened_shape() {
        let args = LocalDockerProvider::run_args("sess-2", true, &HashMap::new());
        let joined = args.join(" ");
        assert!(joined.contains("--network none"), "no egress");
        assert!(joined.contains("--read-only"), "read-only rootfs");
        assert!(
            args.windows(2).any(|w| w[0] == "-v" && w[1].ends_with(":/workspace:ro")),
            "upload mount must be read-only"
        );
    }

    #[test]
    fn test_gate_denies_privileged() {
        let mut args = vec!["run".to_string(), "--privileged".to_string()];
        args.push("img".into());
        assert!(LocalDockerProvider::validate_docker_args(&args).is_err());
    }

    #[test]
    fn test_gate_denies_network_host() {
        for form in [
            to_owned(&["run", "--network", "host", "img"]),
            to_owned(&["run", "--network=host", "img"]),
            to_owned(&["run", "--net", "host", "img"]),
        ] {
            assert!(
                LocalDockerProvider::validate_docker_args(&form).is_err(),
                "network host must be denied"
            );
        }
    }

    #[test]
    fn test_gate_denies_rw_and_bare_mounts() {
        for spec in ["/tmp/data:/workspace", "/tmp/data:/workspace:rw"] {
            let args = to_owned(&["run", "-v", spec, "img"]);
            assert!(
                LocalDockerProvider::validate_docker_args(&args).is_err(),
                "mount `{}` must be denied (rw default/explicit)",
                spec
            );
        }
    }

    #[test]
    fn test_gate_allows_ro_mount() {
        for spec in ["/tmp/a:/workspace:ro", "/tmp/b:/workspace:ro,z"] {
            let args = to_owned(&["run", "-v", spec, "img"]);
            LocalDockerProvider::validate_docker_args(&args)
                .unwrap_or_else(|e| panic!("mount `{}` should pass: {}", spec, e));
        }
    }

    #[test]
    fn test_publish_requires_loopback_bind() {
        let ok = to_owned(&["run", "-p", "127.0.0.1:8080:80", "img"]);
        LocalDockerProvider::validate_docker_args(&ok).expect("loopback bind allowed");

        for bad in ["8080:80", "0.0.0.0:8080:80"] {
            let args = to_owned(&["run", "--publish", bad, "img"]);
            assert!(
                LocalDockerProvider::validate_docker_args(&args).is_err(),
                "publish `{}` must be denied (non-loopback bind)",
                bad
            );
        }
    }
}
