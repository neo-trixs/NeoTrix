use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolPermission {
    Network,
    FileSystem,
    Shell,
    SystemConfig,
    UserData,
    McpServer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissionSet {
    permissions: HashSet<ToolPermission>,
}

impl ToolPermissionSet {
    pub fn new(permissions: Vec<ToolPermission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    pub fn verify(&self, required: &[ToolPermission]) -> Result<(), PermissionDenied> {
        for perm in required {
            if !self.permissions.contains(perm) {
                return Err(PermissionDenied {
                    tool_id: "unknown".into(),
                    missing: perm.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn contains(&self, perm: &ToolPermission) -> bool {
        self.permissions.contains(perm)
    }

    pub fn all_permissions() -> Self {
        use ToolPermission::*;
        Self {
            permissions: vec![Network, FileSystem, Shell, SystemConfig, UserData, McpServer]
                .into_iter()
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionDenied {
    pub tool_id: String,
    pub missing: ToolPermission,
}

impl fmt::Display for PermissionDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Permission denied for tool '{}': missing {:?}",
            self.tool_id, self.missing
        )
    }
}

impl std::error::Error for PermissionDenied {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_verify_allows_present() {
        let set = ToolPermissionSet::new(vec![ToolPermission::Network, ToolPermission::FileSystem]);
        assert!(set.verify(&[ToolPermission::Network]).is_ok());
    }

    #[test]
    fn test_permission_set_verify_denies_missing() {
        let set = ToolPermissionSet::new(vec![ToolPermission::Network]);
        assert!(set.verify(&[ToolPermission::Shell]).is_err());
    }

    #[test]
    fn test_permission_set_contains() {
        let set = ToolPermissionSet::new(vec![ToolPermission::FileSystem]);
        assert!(set.contains(&ToolPermission::FileSystem));
        assert!(!set.contains(&ToolPermission::McpServer));
    }

    #[test]
    fn test_permission_set_all_permissions() {
        let all = ToolPermissionSet::all_permissions();
        assert!(all.contains(&ToolPermission::Network));
        assert!(all.contains(&ToolPermission::McpServer));
    }

    #[test]
    fn test_permission_denied_display() {
        let err = PermissionDenied {
            tool_id: "test_tool".into(),
            missing: ToolPermission::SystemConfig,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test_tool"));
        assert!(msg.contains("SystemConfig"));
    }

    #[test]
    fn test_permission_denied_is_error() {
        let err = PermissionDenied {
            tool_id: "x".into(),
            missing: ToolPermission::UserData,
        };
        let err_ref: &dyn std::error::Error = &err;
        assert!(!err_ref.to_string().is_empty());
    }

    #[test]
    fn test_permission_verify_multiple_all() {
        let set = ToolPermissionSet::new(vec![
            ToolPermission::Network, ToolPermission::FileSystem, ToolPermission::Shell,
        ]);
        assert!(set.verify(&[ToolPermission::Network, ToolPermission::Shell]).is_ok());
    }

    #[test]
    fn test_permission_verify_multiple_one_missing() {
        let set = ToolPermissionSet::new(vec![ToolPermission::Network, ToolPermission::FileSystem]);
        let result = set.verify(&[ToolPermission::Network, ToolPermission::Shell]);
        assert!(result.is_err());
    }
}
