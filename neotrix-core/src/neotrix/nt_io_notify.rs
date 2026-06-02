use std::process::Command;

pub enum Level {
    Info,
    Success,
    Warning,
    Error,
}

impl Level {
    fn label(&self) -> &str {
        match self {
            Level::Info => "Info",
            Level::Success => "Success",
            Level::Warning => "Warning",
            Level::Error => "Error",
        }
    }
}

pub fn notify(title: &str, message: &str) {
    notify_with_level(title, message, Level::Info);
}

pub fn notify_with_level(title: &str, message: &str, level: Level) {
    let full_title = format!("NeoTrix - {}", level.label());
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display nt_io_notify \"{}\" with title \"{}\" subtitle \"{}\"",
            message.replace("\"", "\\\""),
            full_title.replace("\"", "\\\""),
            title.replace("\"", "\\\""),
        );
        if let Err(e) = Command::new("osascript").arg("-e").arg(&script).output() {
            log::warn!("通知失败: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = Command::new("notify-send")
            .arg(&full_title)
            .arg(message)
            .output()
        {
            log::warn!("通知失败: {}", e);
        }
    }
    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!(
            "[System.Windows.MessageBox]::show('{}','{}')",
            message.replace("'", "''"),
            full_title.replace("'", "''"),
        );
        if std::env::var("WT_SESSION").is_ok() {
            let _ = Command::new("powershell.exe")
                .arg("-c")
                .arg(&ps_cmd)
                .output();
        } else {
            let _ = Command::new("powershell")
                .arg("-c")
                .arg(&ps_cmd)
                .output();
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (title, message, level);
        log::info!("通知(未支持平台): {} - {}", title, message);
    }
}

/// Send nt_io_notify for long-running task completion
pub fn notify_task_complete(task_name: &str, success: bool) {
    let icon = if success { "✅" } else { "❌" };
    let status = if success { "completed" } else { "failed" };
    notify_with_level(
        &format!("{} Task {}", icon, status),
        &format!("'{}' {}", task_name, status),
        if success { Level::Success } else { Level::Error },
    );
}

/// Send nt_io_notify when agent approval is needed
pub fn notify_approval_needed(description: &str) {
    notify_with_level(
        "Approval Required",
        description,
        Level::Warning,
    );
}
