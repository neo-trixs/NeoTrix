//! Permission Manager - 权限管理器
//!
//! 管理清理操作的权限检查
//! 域: NT-SHIELD (影卫)
//! 层: L3 Embodiment

use std::path::Path;

pub struct PermissionManager {
    fda_paths: Vec<String>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            fda_paths: vec![
                "~/Library/Mail".into(), "~/Library/Safari".into(),
                "~/Library/Messages".into(), "~/Library/AddressBook".into(),
            ],
        }
    }

    pub fn check_permission(&self, path: &Path) -> _PermissionCheck {
        let requires_sudo = self.requires_sudo(path);
        let requires_fda = self.requires_fda(path);
        _PermissionCheck {
            has_permission: !requires_sudo && !requires_fda,
            requires_sudo,
            requires_fda,
            error: None,
        }
    }

    fn requires_sudo(&self, path: &Path) -> bool {
        let s = path.to_string_lossy();
        s.starts_with("/System") || s.starts_with("/usr") || s.starts_with("/private/var/db")
    }

    fn requires_fda(&self, path: &Path) -> bool {
        let s = path.to_string_lossy().to_string();
        self.fda_paths.iter().any(|fda| s.starts_with(fda.as_str()))
    }

    pub fn _add_fda_path(&mut self, path: String) { self.fda_paths.push(path); }
}

pub struct _PermissionCheck {
    pub has_permission: bool,
    pub requires_sudo: bool,
    pub requires_fda: bool,
    pub error: Option<String>,
}

impl Default for PermissionManager { fn default() -> Self { Self::new() } }
