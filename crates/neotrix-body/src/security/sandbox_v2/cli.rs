use std::sync::Arc;
use std::time::Duration;

use super::{CloudRuntime, CloudSandbox};
use super::docker::LocalDockerProvider;
use super::provider::NoopProvider;

fn create_sandbox(timeout: u64) -> CloudSandbox {
    CloudSandbox::new(
        "http://localhost".to_string(),
        None,
        Duration::from_secs(timeout),
        if CloudSandbox::docker_available() {
            Arc::new(LocalDockerProvider::new())
        } else {
            Arc::new(NoopProvider)
        },
    )
}

fn print_result(result: &super::CloudResult) {
    log::info!("[sandbox] exit_code={}", result.exit_code);
    log::info!("[sandbox] execution_time={:?}", result.execution_time);
    if !result.stdout.is_empty() {
        log::info!("--- stdout ---\n{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        log::info!("--- stderr ---\n{}", result.stderr);
    }
}

pub async fn handle_run(code: Option<&str>, runtime: Option<&str>, timeout: Option<u64>) {
    let rt = runtime
        .and_then(CloudRuntime::from_str)
        .unwrap_or(CloudRuntime::Python3);

    let mut sandbox = create_sandbox(timeout.unwrap_or(300));

    match code {
        Some(c) => {
            log::info!(
                "[sandbox] executing on {} (runtime: {})...",
                sandbox.provider_name(),
                rt.as_str()
            );
            match sandbox.run_code(c, rt).await {
                Ok(result) => print_result(&result),
                Err(e) => log::error!("[sandbox] error: {}", e),
            }
        }
        None => {
            use std::io::Read;
            let mut buf = String::new();
            if std::io::stdin().lock().read_to_string(&mut buf).is_ok() && !buf.trim().is_empty() {
                log::info!("[sandbox] executing {} bytes from stdin...", buf.len());
                match sandbox.run_code(&buf, rt).await {
                    Ok(result) => print_result(&result),
                    Err(e) => log::error!("[sandbox] error: {}", e),
                }
            } else {
                log::error!("[sandbox] error: no code provided. Pipe code via stdin or pass as argument.");
            }
        }
    }
}

pub fn handle_list() {
    // Sessions are in-memory; this shows current in-memory sessions.
    // For a persistent view, sessions would need to be stored on disk.
    let sandbox = create_sandbox(300);
    let sessions = sandbox.list_sessions();
    if sessions.is_empty() {
        log::info!("[sandbox] no active sessions (sessions are in-memory)");
        return;
    }
    log::info!("[sandbox] active sessions:");
    for s in sessions {
        log::info!(
            "  {} | runtime={} | status={:?}",
            s.session_id,
            s.runtime.as_str(),
            s.status
        );
    }
}

pub fn handle_cancel(session_id: &str) {
    let mut sandbox = create_sandbox(300);
    match sandbox.cancel_session(session_id) {
        Ok(()) => log::info!("[sandbox] session {} cancelled", session_id),
        Err(e) => log::error!("[sandbox] cancel failed: {}", e),
    }
}

pub async fn handle_upload(path: &str, session_id: &str) {
    let data = match tokio::fs::read(path).await {
        Ok(d) => d,
        Err(e) => {
            log::error!("[sandbox] read file '{}' failed: {}", path, e);
            return;
        }
    };

    let mut sandbox = create_sandbox(300);
    let sid = sandbox.create_session(CloudRuntime::GenericLinux);
    let sid_ref = if session_id.is_empty() { &sid } else { session_id };

    match sandbox.get_session_mut(sid_ref) {
        Some(session) => match session.upload_file(path, data).await {
            Ok(()) => log::info!("[sandbox] uploaded '{}' to session {}", path, sid_ref),
            Err(e) => log::error!("[sandbox] upload failed: {}", e),
        },
        None => {
            log::warn!(
                "[sandbox] session '{}' not found. Created session: {}",
                sid_ref, sid
            );
        }
    }
}
