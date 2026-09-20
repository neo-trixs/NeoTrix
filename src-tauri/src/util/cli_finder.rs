use std::process::Command;

pub fn find_neotrix_binary() -> Option<String> {
    if let Ok(output) = Command::new("which").arg("neotrix").output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }
    None
}
