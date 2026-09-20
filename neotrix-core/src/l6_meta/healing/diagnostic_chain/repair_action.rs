#![forbid(unsafe_code)]

use std::fmt;

/// A discrete repair action that can be executed against a component.
///
/// Each variant represents a different healing strategy:
/// - `Restart`: Stop and restart the target component.
/// - `Rollback`: Revert the component to a known-good version.
/// - `Degrade`: Switch to a fallback path while keeping the service alive.
/// - `Alert`: Emit an alert for human/operator intervention.
/// - `SelfHeal`: Execute a named self-healing script/procedure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepairAction {
    Restart { component: String },
    Rollback { to_version: String },
    Degrade { fallback: String },
    Alert { message: String },
    SelfHeal { script: String },
}

impl fmt::Display for RepairAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepairAction::Restart { component } => write!(f, "Restart({})", component),
            RepairAction::Rollback { to_version } => write!(f, "Rollback({})", to_version),
            RepairAction::Degrade { fallback } => write!(f, "Degrade({})", fallback),
            RepairAction::Alert { message } => write!(f, "Alert({})", message),
            RepairAction::SelfHeal { script } => write!(f, "SelfHeal({})", script),
        }
    }
}

impl RepairAction {
    pub fn restart(component: impl Into<String>) -> Self {
        RepairAction::Restart {
            component: component.into(),
        }
    }

    pub fn rollback(to_version: impl Into<String>) -> Self {
        RepairAction::Rollback {
            to_version: to_version.into(),
        }
    }

    pub fn degrade(fallback: impl Into<String>) -> Self {
        RepairAction::Degrade {
            fallback: fallback.into(),
        }
    }

    pub fn alert(message: impl Into<String>) -> Self {
        RepairAction::Alert {
            message: message.into(),
        }
    }

    pub fn self_heal(script: impl Into<String>) -> Self {
        RepairAction::SelfHeal {
            script: script.into(),
        }
    }

    /// Returns a human-readable label for this action type.
    pub fn action_type(&self) -> &'static str {
        match self {
            RepairAction::Restart { .. } => "restart",
            RepairAction::Rollback { .. } => "rollback",
            RepairAction::Degrade { .. } => "degrade",
            RepairAction::Alert { .. } => "alert",
            RepairAction::SelfHeal { .. } => "self_heal",
        }
    }
}

/// Simulates executing a repair action.
///
/// In production this would drive real component lifecycle operations.
/// The current implementation logs the action and returns a descriptive result.
pub fn execute_repair(action: &RepairAction) -> Result<String, String> {
    match action {
        RepairAction::Restart { component } => {
            log::info!(
                "[diagnostic_chain] executing restart for component: {}",
                component
            );
            Ok(format!("component '{}' restarted successfully", component))
        }
        RepairAction::Rollback { to_version } => {
            log::info!(
                "[diagnostic_chain] executing rollback to version: {}",
                to_version
            );
            Ok(format!("rolled back to version '{}'", to_version))
        }
        RepairAction::Degrade { fallback } => {
            log::warn!("[diagnostic_chain] degrading to fallback: {}", fallback);
            Ok(format!("degraded to fallback '{}'", fallback))
        }
        RepairAction::Alert { message } => {
            log::error!("[diagnostic_chain] alert emitted: {}", message);
            Ok(format!("alert emitted: {}", message))
        }
        RepairAction::SelfHeal { script } => {
            log::info!("[diagnostic_chain] executing self-heal script: {}", script);
            Ok(format!("self-heal script '{}' executed", script))
        }
    }
}

/// Selects the appropriate repair action from a diagnostic severity level.
///
/// Heuristic mapping:
/// - Critical → restart
/// - Warning → degrade
/// - Info/other → alert
pub fn select_action(source_component: &str, max_severity: &str) -> RepairAction {
    match max_severity {
        "critical" => RepairAction::restart(source_component),
        "warning" => RepairAction::degrade(source_component),
        _ => RepairAction::alert(format!("monitoring: {}", source_component)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_action() {
        let action = RepairAction::restart("nt_core_cache");
        assert_eq!(action.action_type(), "restart");
        let result = execute_repair(&action).unwrap();
        assert!(result.contains("nt_core_cache"));
    }

    #[test]
    fn test_rollback_action() {
        let action = RepairAction::rollback("v0.9.2");
        assert_eq!(action.action_type(), "rollback");
        let result = execute_repair(&action).unwrap();
        assert!(result.contains("v0.9.2"));
    }

    #[test]
    fn test_degrade_action() {
        let action = RepairAction::degrade("cache_readonly");
        assert_eq!(action.action_type(), "degrade");
        let result = execute_repair(&action).unwrap();
        assert!(result.contains("cache_readonly"));
    }

    #[test]
    fn test_alert_action() {
        let action = RepairAction::alert("disk full");
        assert_eq!(action.action_type(), "alert");
        let result = execute_repair(&action).unwrap();
        assert!(result.contains("disk full"));
    }

    #[test]
    fn test_self_heal_action() {
        let action = RepairAction::self_heal("scripts/repair_cache.sh");
        assert_eq!(action.action_type(), "self_heal");
        let result = execute_repair(&action).unwrap();
        assert!(result.contains("repair_cache.sh"));
    }

    #[test]
    fn test_display_formatting() {
        let action = RepairAction::restart("nt_world_crawl");
        assert_eq!(format!("{}", action), "Restart(nt_world_crawl)");
    }

    #[test]
    fn test_select_action_critical() {
        let action = select_action("nt_core_cache", "critical");
        assert_eq!(action.action_type(), "restart");
    }

    #[test]
    fn test_select_action_warning() {
        let action = select_action("nt_core_cache", "warning");
        assert_eq!(action.action_type(), "degrade");
    }

    #[test]
    fn test_select_action_info() {
        let action = select_action("nt_core_cache", "info");
        assert_eq!(action.action_type(), "alert");
    }

    #[test]
    fn test_action_equality() {
        let a = RepairAction::restart("x");
        let b = RepairAction::restart("x");
        assert_eq!(a, b);
    }

    #[test]
    fn test_action_clone() {
        let action = RepairAction::rollback("v1.0");
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_action_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(RepairAction::restart("a"));
        set.insert(RepairAction::rollback("b"));
        set.insert(RepairAction::degrade("c"));
        set.insert(RepairAction::alert("d"));
        set.insert(RepairAction::self_heal("e"));
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_all_action_types_have_display() {
        let actions = vec![
            RepairAction::restart("comp"),
            RepairAction::rollback("v1"),
            RepairAction::degrade("fb"),
            RepairAction::alert("msg"),
            RepairAction::self_heal("script"),
        ];
        for a in &actions {
            let s = format!("{}", a);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_all_execute_repair_return_ok() {
        let actions = vec![
            RepairAction::restart("comp"),
            RepairAction::rollback("v1"),
            RepairAction::degrade("fb"),
            RepairAction::alert("msg"),
            RepairAction::self_heal("script"),
        ];
        for a in &actions {
            assert!(execute_repair(a).is_ok());
        }
    }

    #[test]
    fn test_select_action_unknown_severity() {
        let action = select_action("comp", "unknown");
        assert_eq!(action.action_type(), "alert");
    }

    #[test]
    fn test_select_action_empty_severity() {
        let action = select_action("comp", "");
        assert_eq!(action.action_type(), "alert");
    }

    #[test]
    fn test_action_inequality() {
        let a = RepairAction::restart("x");
        let b = RepairAction::rollback("x");
        assert_ne!(a, b);
    }
}
