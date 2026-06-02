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
    ) -> Result<CloudResult, String> {
        let image = Self::image_for(runtime);
        let (entrypoint, args) = Self::wrap_command(runtime, code);

        let has_uploads = self
            .uploaded_files
            .lock()
            .map(|f| f.contains_key(session_id))
            .unwrap_or(false);

        let mut docker_args: Vec<String> = vec![
            "run".into(),
            "--rm".into(),
            "--network".into(),
            "none".into(),
            "--memory".into(),
            "512m".into(),
            "--cpus".into(),
            "1".into(),
            "--label".into(),
            format!("neotrix-session={}", session_id),
        ];

        if has_uploads {
            let tmpdir = format!("/tmp/neotrix-upload-{}", session_id);
            docker_args.push("-v".into());
            docker_args.push(format!("{}:/workspace:ro", tmpdir));
        }

        docker_args.push(image.into());
        docker_args.push(entrypoint);
        docker_args.extend(args);

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
}
