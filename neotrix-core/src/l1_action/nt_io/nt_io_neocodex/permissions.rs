// ── Permission System (from Claude Code: allow/deny/ask per tool) ──

#[derive(Debug, Clone)]
pub enum PermissionLevel {
    Allow,
    Deny,
    Ask,
}

pub struct PermissionSystem {
    pub permissions: Vec<(String, PermissionLevel)>,
    pub default_level: PermissionLevel,
}

impl Default for PermissionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionSystem {
    pub fn new() -> Self {
        let permissions = vec![
            ("read".to_string(), PermissionLevel::Allow),
            ("search".to_string(), PermissionLevel::Allow),
            ("write".to_string(), PermissionLevel::Allow),
            ("edit".to_string(), PermissionLevel::Allow),
            ("shell".to_string(), PermissionLevel::Ask),
        ];
        Self {
            permissions,
            default_level: PermissionLevel::Ask,
        }
    }

    /// Non-blocking policy gate for the streaming path (P0-2). Unlike
    /// `interactive_check` (stdin, CLI-only), this decides from the active
    /// permission mode so the desktop app can enforce Claude-style modes
    /// without a blocking terminal prompt.
    pub fn policy_gate(&self, tool: &str, permission_mode: &str) -> bool {
        match self.check(tool) {
            // read/search/write/edit are workspace-guarded → always allowed.
            PermissionLevel::Allow => true,
            // Deny is absolute regardless of mode.
            PermissionLevel::Deny => false,
            // shell (the only Ask tool): allowed in agentic modes, denied in
            // restrictive ones (manual/accept_edits/plan) where the UI review
            // layer is the enforcement point instead.
            PermissionLevel::Ask => !matches!(
                permission_mode,
                "manual" | "accept_edits" | "acceptEdits" | "plan"
            ),
        }
    }

    pub fn check(&self, tool: &str) -> PermissionLevel {
        for (pattern, level) in &self.permissions {
            if tool.starts_with(pattern) {
                return level.clone();
            }
        }
        self.default_level.clone()
    }

    pub fn set(&mut self, tool: &str, level: PermissionLevel) {
        if let Some((_, l)) = self.permissions.iter_mut().find(|(p, _)| p == tool) {
            *l = level;
        } else {
            self.permissions.push((tool.to_string(), level));
        }
    }

    pub fn interactive_check(&self, tool: &str, description: &str) -> bool {
        match self.check(tool) {
            PermissionLevel::Allow => true,
            PermissionLevel::Deny => {
                eprintln!("[permission] DENY: {} — {}", tool, description);
                false
            }
            PermissionLevel::Ask => {
                eprint!("[permission] Allow {} ({})? (y/N): ", tool, description);
                let mut buf = String::new();
                let _ = std::io::stdin().read_line(&mut buf);
                buf.trim().to_lowercase() == "y" || buf.trim().to_lowercase() == "yes"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_system() {
        let ps = PermissionSystem::new();
        assert!(ps.interactive_check("read", "read a file"));
        match ps.check("shell") {
            PermissionLevel::Ask => {}
            _ => panic!("shell should default to Ask"),
        }
    }
}