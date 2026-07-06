pub fn resolve_daemon_path() -> Option<std::path::PathBuf> {
    let binary_name = if cfg!(target_os = "windows") {
        "neotrix-proxy-daemon.exe"
    } else {
        "neotrix-proxy-daemon"
    };

    if let Ok(path) = std::env::current_exe() {
        let dir = path.parent()?;
        let candidate = dir.join(binary_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let homebrew = dirs::home_dir()?.join(".cargo").join("bin").join(binary_name);
    if homebrew.exists() {
        return Some(homebrew);
    }

    None
}
