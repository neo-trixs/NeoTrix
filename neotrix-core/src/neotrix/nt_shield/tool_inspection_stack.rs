use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use super::tool_permissions::{ToolPermission, ToolPermissionSet};

/// 5-layer security inspection result
pub enum InspectionResult {
    Allow,
    Deny(String),
    RequireApproval(String),
}

impl fmt::Display for InspectionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InspectionResult::Allow => write!(f, "Allow"),
            InspectionResult::Deny(reason) => write!(f, "Deny: {}", reason),
            InspectionResult::RequireApproval(reason) => write!(f, "RequireApproval: {}", reason),
        }
    }
}

pub trait ToolInspector: Send + Sync {
    fn name(&self) -> &str;
    fn inspect(&self, tool_name: &str, args: &serde_json::Value) -> InspectionResult;
}

/// Layer 1: Security check — is the tool itself safe?
pub struct SecurityInspector;

impl ToolInspector for SecurityInspector {
    fn name(&self) -> &str {
        "SecurityInspector"
    }

    fn inspect(&self, tool_name: &str, args: &serde_json::Value) -> InspectionResult {
        let dangerous_tools = [
            "rm", "mkfs", "dd", "format", "shutdown", "reboot",
            "poweroff", "halt", "init", "fdisk", "parted",
            "chmod", "chown", "passwd", "systemctl",
        ];
        if dangerous_tools.contains(&tool_name) {
            return InspectionResult::Deny(format!("Dangerous tool '{}' is blocked", tool_name));
        }

        let check_destructive = |s: &str| -> bool {
            let lower = s.to_lowercase();
            lower.contains("rm -rf /") || lower.contains("rm -rf /*")
        };

        if let Some(s) = args.as_str() {
            if check_destructive(s) {
                return InspectionResult::Deny("Potentially destructive command detected".into());
            }
        }

        if let Some(obj) = args.as_object() {
            for v in obj.values() {
                if let Some(s) = v.as_str() {
                    if check_destructive(s) {
                        return InspectionResult::Deny(
                            "Potentially destructive command detected in arguments".into(),
                        );
                    }
                }
            }
        }

        InspectionResult::Allow
    }
}

/// Layer 2: Egress check — does the tool exfiltrate data?
pub struct EgressInspector;

impl ToolInspector for EgressInspector {
    fn name(&self) -> &str {
        "EgressInspector"
    }

    fn inspect(&self, _tool_name: &str, _args: &serde_json::Value) -> InspectionResult {
        InspectionResult::Allow
    }
}

/// Layer 3: Permission check — does the user's permission set allow this?
pub struct PermissionInspector {
    user_perms: ToolPermissionSet,
}

impl PermissionInspector {
    pub fn new(user_perms: ToolPermissionSet) -> Self {
        Self { user_perms }
    }
}

impl ToolInspector for PermissionInspector {
    fn name(&self) -> &str {
        "PermissionInspector"
    }

    fn inspect(&self, tool_name: &str, _args: &serde_json::Value) -> InspectionResult {
        let tool_permissions: HashMap<&str, Vec<ToolPermission>> = [
            ("read", vec![ToolPermission::FileSystem]),
            ("write", vec![ToolPermission::FileSystem]),
            ("edit", vec![ToolPermission::FileSystem]),
            ("glob", vec![ToolPermission::FileSystem]),
            ("grep", vec![ToolPermission::FileSystem]),
            ("bash", vec![ToolPermission::Shell]),
            ("webfetch", vec![ToolPermission::Network]),
            ("websearch", vec![ToolPermission::Network]),
        ]
        .iter()
        .cloned()
        .collect();

        let Some(required) = tool_permissions.get(tool_name) else {
            return InspectionResult::Allow;
        };

        match self.user_perms.verify(required) {
            Ok(()) => InspectionResult::Allow,
            Err(e) => InspectionResult::Deny(format!("{}", e)),
        }
    }
}

/// Layer 4: Repetition check — is this tool being called too often?
pub struct RepetitionInspector {
    call_counts: Mutex<HashMap<String, usize>>,
    max_calls: usize,
}

impl RepetitionInspector {
    pub fn new(max_calls: usize) -> Self {
        Self {
            call_counts: Mutex::new(HashMap::new()),
            max_calls,
        }
    }
}

impl ToolInspector for RepetitionInspector {
    fn name(&self) -> &str {
        "RepetitionInspector"
    }

    fn inspect(&self, tool_name: &str, _args: &serde_json::Value) -> InspectionResult {
        let mut counts = self.call_counts.lock().unwrap();
        let count = counts.entry(tool_name.to_string()).or_insert(0);
        *count += 1;
        if *count > self.max_calls {
            InspectionResult::Deny(format!(
                "Tool '{}' called {} times (max {})",
                tool_name, *count, self.max_calls
            ))
        } else {
            InspectionResult::Allow
        }
    }
}

/// Layer 5: Build check — is this modifying critical system files?
pub struct BuildInspector;

impl ToolInspector for BuildInspector {
    fn name(&self) -> &str {
        "BuildInspector"
    }

    fn inspect(&self, tool_name: &str, args: &serde_json::Value) -> InspectionResult {
        let dangerous_paths = [
            "/etc/",
            "/usr/bin/",
            "/usr/lib/",
            "/boot/",
            "/dev/",
            "/proc/",
            "/sys/",
        ];

        let relevant_tools = ["write", "edit", "bash"];
        if !relevant_tools.contains(&tool_name) {
            return InspectionResult::Allow;
        }

        let paths_to_check: Vec<&str> = if let Some(s) = args.as_str() {
            vec![s]
        } else if let Some(obj) = args.as_object() {
            obj.values().filter_map(|v| v.as_str()).collect()
        } else {
            return InspectionResult::Allow;
        };

        for path in paths_to_check {
            for dangerous in &dangerous_paths {
                if path.starts_with(dangerous) {
                    return InspectionResult::Deny(format!(
                        "Blocked modification of critical system path: {}",
                        path
                    ));
                }
            }
        }

        InspectionResult::Allow
    }
}

pub struct ToolInspectionStack {
    inspectors: Vec<Box<dyn ToolInspector>>,
}

impl ToolInspectionStack {
    pub fn new() -> Self {
        Self {
            inspectors: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut stack = Self::new();
        stack.add(Box::new(SecurityInspector));
        stack.add(Box::new(EgressInspector));
        stack.add(Box::new(PermissionInspector::new(
            ToolPermissionSet::all_permissions(),
        )));
        stack.add(Box::new(RepetitionInspector::new(50)));
        stack.add(Box::new(BuildInspector));
        stack
    }

    pub fn add(&mut self, inspector: Box<dyn ToolInspector>) {
        self.inspectors.push(inspector);
    }

    pub fn inspect(&self, tool_name: &str, args: &serde_json::Value) -> Vec<(String, InspectionResult)> {
        self.inspectors
            .iter()
            .map(|i| (i.name().to_string(), i.inspect(tool_name, args)))
            .collect()
    }

    /// Returns the first Deny, or the first RequireApproval, or Allow
    pub fn check_all(&self, tool_name: &str, args: &serde_json::Value) -> InspectionResult {
        let mut require_approval: Option<String> = None;
        for inspector in &self.inspectors {
            match inspector.inspect(tool_name, args) {
                InspectionResult::Deny(reason) => return InspectionResult::Deny(reason),
                InspectionResult::RequireApproval(reason) => {
                    if require_approval.is_none() {
                        require_approval = Some(reason);
                    }
                }
                InspectionResult::Allow => {}
            }
        }
        require_approval.map_or(InspectionResult::Allow, InspectionResult::RequireApproval)
    }
}

impl Default for ToolInspectionStack {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_inspector_allows_safe() {
        let inspector = SecurityInspector;
        let args = serde_json::json!("ls -la");
        assert!(matches!(
            inspector.inspect("ls", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_security_inspector_denies_dangerous_tool() {
        let inspector = SecurityInspector;
        let args = serde_json::json!("");
        assert!(matches!(
            inspector.inspect("dd", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_security_inspector_denies_rm_rf() {
        let inspector = SecurityInspector;
        let args = serde_json::json!("rm -rf /");
        assert!(matches!(
            inspector.inspect("bash", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_egress_inspector_always_allows() {
        let inspector = EgressInspector;
        let args = serde_json::json!({});
        assert!(matches!(
            inspector.inspect("webfetch", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_permission_inspector_allows_permitted() {
        let inspector = PermissionInspector::new(ToolPermissionSet::all_permissions());
        let args = serde_json::json!({});
        assert!(matches!(
            inspector.inspect("read", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_permission_inspector_denies_missing() {
        let restricted = ToolPermissionSet::new(vec![ToolPermission::FileSystem]);
        let inspector = PermissionInspector::new(restricted);
        let args = serde_json::json!({});
        assert!(matches!(
            inspector.inspect("bash", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_repetition_inspector_allows_under_limit() {
        let inspector = RepetitionInspector::new(3);
        let args = serde_json::json!({});
        assert!(matches!(
            inspector.inspect("read", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_repetition_inspector_denies_over_limit() {
        let inspector = RepetitionInspector::new(2);
        let args = serde_json::json!({});
        assert!(matches!(
            inspector.inspect("bash", &args),
            InspectionResult::Allow
        ));
        assert!(matches!(
            inspector.inspect("bash", &args),
            InspectionResult::Allow
        ));
        assert!(matches!(
            inspector.inspect("bash", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_build_inspector_allows_safe_path() {
        let inspector = BuildInspector;
        let args = serde_json::json!("/Users/test/file.txt");
        assert!(matches!(
            inspector.inspect("write", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_build_inspector_denies_etc() {
        let inspector = BuildInspector;
        let args = serde_json::json!("/etc/passwd");
        assert!(matches!(
            inspector.inspect("write", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_build_inspector_ignores_non_relevant_tool() {
        let inspector = BuildInspector;
        let args = serde_json::json!("/etc/passwd");
        assert!(matches!(
            inspector.inspect("webfetch", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_stack_check_all_returns_first_deny() {
        let mut stack = ToolInspectionStack::new();
        stack.add(Box::new(SecurityInspector));
        stack.add(Box::new(PermissionInspector::new(
            ToolPermissionSet::all_permissions(),
        )));
        let args = serde_json::json!("");
        assert!(matches!(
            stack.check_all("dd", &args),
            InspectionResult::Deny(_)
        ));
    }

    #[test]
    fn test_stack_check_all_returns_allow_when_all_pass() {
        let stack = ToolInspectionStack::with_defaults();
        let args = serde_json::json!("/Users/test/file.txt");
        assert!(matches!(
            stack.check_all("read", &args),
            InspectionResult::Allow
        ));
    }

    #[test]
    fn test_inspection_result_display() {
        assert_eq!(format!("{}", InspectionResult::Allow), "Allow");
        assert!(format!("{}", InspectionResult::Deny("bad".into())).contains("Deny"));
        assert!(format!("{}", InspectionResult::RequireApproval("ask".into())).contains("RequireApproval"));
    }
}
