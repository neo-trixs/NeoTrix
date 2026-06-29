pub fn run_sandbox_run(code: Option<&str>, runtime: &str, timeout: u64) {
    use neotrix::neotrix::nt_shield_sandbox::cli;
    let runtime = if runtime.is_empty() {
        None
    } else {
        Some(runtime)
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(cli::handle_run(code, runtime, Some(timeout)));
}

pub fn run_sandbox_list() {
    neotrix::neotrix::nt_shield_sandbox::cli::handle_list();
}

pub fn run_sandbox_cancel(session_id: &str) {
    neotrix::neotrix::nt_shield_sandbox::cli::handle_cancel(session_id);
}

pub fn run_sandbox_upload(path: &str, session_id: &str) {
    use neotrix::neotrix::nt_shield_sandbox::cli;
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(cli::handle_upload(path, session_id));
}
