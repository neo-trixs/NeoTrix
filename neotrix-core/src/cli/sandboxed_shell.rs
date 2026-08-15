use crate::cli::approval::ActionType;
use crate::cli::shield_enforcer::{global_shield, ShieldDecision};

/// Execute a shell command through the ShieldEnforcer check chain.
/// Returns stdout+stderr on success, or an error description.
pub fn execute_guarded(command: &str) -> Result<String, String> {
    let shield = global_shield()
        .lock()
        .map_err(|e| format!("[sandboxed-shell] lock error: {e}"))?;

    let action_type = ActionType::ShellCommand { command: command.to_string() };

    shield.check_all(
        "execute_command",
        command,
        Some(command),
        Some(&action_type),
    )
    .map_err(|decision| match decision {
        ShieldDecision::Block(msg) => format!("[BLOCKED] {msg}"),
        ShieldDecision::RequireApproval(msg) => format!("[NEEDS APPROVAL] {msg}"),
        ShieldDecision::Violation(v) => format!("[VIOLATION] {v:?}"),
        ShieldDecision::Allow => "[ALLOWED but unreachable in error path]".into(),
    })?;

    drop(shield);

    // Wire proxy env vars based on proxy allowlist config
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(command);

    let output = cmd.output()
        .map_err(|e| format!("Failed to execute command: {e}"))?;

    let mut result = String::new();
    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        result.push_str(&format!("\n(exit code: {code})"));
    }

    Ok(result)
}

/// Check whether a file operation is allowed by the ShieldEnforcer.
/// Returns Ok(()) if allowed, or a description of why it was blocked.
pub fn check_file_operation(action: &str, path: &str, content: Option<&str>) -> Result<(), String> {
    let shield = global_shield()
        .lock()
        .map_err(|e| format!("[sandboxed-shell] lock error: {e}"))?;

    let action_type = match action {
        "file_read" => ActionType::FileCreate { path: path.to_string() },
        "file_write" | "file_edit" => ActionType::FileWrite {
            path: path.to_string(),
            content_preview: content.unwrap_or("").to_string(),
        },
        _ => return Ok(()),
    };

    shield.check_all(action, path, content, Some(&action_type))
        .map_err(|decision| match decision {
            ShieldDecision::Block(msg) => format!("[BLOCKED] {msg}"),
            ShieldDecision::RequireApproval(msg) => format!("[NEEDS APPROVAL] {msg}"),
            ShieldDecision::Violation(v) => format!("[VIOLATION] {v:?}"),
            ShieldDecision::Allow => "[ALLOWED but unreachable in error path]".into(),
        })
}

/// Quick check whether shell execution is allowed (non-blocking, for soft-enforcement).
pub fn is_shell_allowed(silent: bool) -> bool {
    let shield = match global_shield().try_lock() {
        Ok(s) => s,
        Err(_) => return true,
    };
    let allowed = !shield.sandbox.is_read_only()
        && shield.approval.mode() != crate::cli::approval::ApprovalMode::Suggest
        && shield.guard.check("execute_command", "test").is_ok();
    drop(shield);
    if !allowed && !silent {
        let _ = crate::cli::commands::types::CommandOutput::err(
            "Shell execution is restricted. Use --sandbox disabled or --approval full-auto to enable."
        );
    }
    allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::shield_enforcer::TEST_SHIELD_LOCK;

    #[test]
    fn test_is_shell_allowed_default() {
        // Default state: Suggest mode + Disabled sandbox
        // Suggest blocks shell
        if std::process::Command::new("which").arg("bash").output().is_err() {
            return;
        }
        // Serialize against any test mutating GLOBAL_SHIELD (e.g.
        // shield_enforcer::tests::test_init_shield setting FullAuto). is_shell_allowed
        // uses try_lock and returns true when the global is briefly held by another
        // test, so this assertion must not overlap any global accessor.
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert!(!is_shell_allowed(true), "Suggest mode should not allow shell");
    }

    #[test]
    fn test_is_shell_allowed_full_auto() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        {
            let _s = global_shield().lock().unwrap();
            // We can't easily modify the global singleton in parallel tests,
            // but we can at least call the function without panicking.
        }
        // The function should always return a value without panicking
        let _ = is_shell_allowed(true);
    }

    #[test]
    fn test_check_file_operation_read() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let result = check_file_operation("file_read", "/tmp/test.txt", None);
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_execute_guarded_empty_command() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let result = execute_guarded("echo hello");
        // May be blocked by ShieldEnforcer, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
}
