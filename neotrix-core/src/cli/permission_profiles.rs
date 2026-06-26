use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Decision for a single action key within a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileDecision {
    Allow,
    Deny,
    Ask,
}

/// A named, inheritable permission profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub rules: HashMap<String, ProfileDecision>,
    /// Override the global approval mode while this profile is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_mode_override: Option<String>,
}

impl PermissionProfile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
            rules: HashMap::new(),
            approval_mode_override: None,
        }
    }
}

/// All profiles persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: HashMap<String, PermissionProfile>,
    pub active: String,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::builtin()
    }
}

impl ProfileStore {
    pub fn builtin() -> Self {
        let mut profiles = HashMap::new();

        // nt_shield (default)
        let mut nt_shield = PermissionProfile::new("nt_shield");
        nt_shield
            .rules
            .insert("write_file".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("delete_file".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("execute_command".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("network_request".into(), ProfileDecision::Allow);
        nt_shield
            .rules
            .insert("read_file".into(), ProfileDecision::Allow);
        nt_shield
            .rules
            .insert("read_secrets".into(), ProfileDecision::Deny);
        nt_shield
            .rules
            .insert("git_push".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("git_force_push".into(), ProfileDecision::Deny);
        nt_shield
            .rules
            .insert("compile_check".into(), ProfileDecision::Allow);
        nt_shield
            .rules
            .insert("modify_dependency".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("access_nt_world_browse_auto".into(), ProfileDecision::Ask);
        nt_shield
            .rules
            .insert("access_tor_network".into(), ProfileDecision::Ask);
        profiles.insert("nt_shield".into(), nt_shield);

        // strict-nt_shield
        let mut strict = PermissionProfile::new("strict-nt_shield");
        strict.parent = Some("nt_shield".into());
        strict
            .rules
            .insert("write_file".into(), ProfileDecision::Deny);
        strict
            .rules
            .insert("network_request".into(), ProfileDecision::Ask);
        strict
            .rules
            .insert("read_file".into(), ProfileDecision::Allow);
        strict
            .rules
            .insert("access_tor_network".into(), ProfileDecision::Deny);
        profiles.insert("strict-nt_shield".into(), strict);

        // general
        let mut general = PermissionProfile::new("general");
        general.parent = Some("nt_shield".into());
        general
            .rules
            .insert("write_file".into(), ProfileDecision::Allow);
        general
            .rules
            .insert("delete_file".into(), ProfileDecision::Ask);
        general
            .rules
            .insert("execute_command".into(), ProfileDecision::Allow);
        general
            .rules
            .insert("network_request".into(), ProfileDecision::Allow);
        general
            .rules
            .insert("modify_dependency".into(), ProfileDecision::Allow);
        profiles.insert("general".into(), general);

        // developer (most permissive)
        let mut developer = PermissionProfile::new("developer");
        developer.parent = Some("general".into());
        developer
            .rules
            .insert("delete_file".into(), ProfileDecision::Allow);
        developer
            .rules
            .insert("git_push".into(), ProfileDecision::Allow);
        developer
            .rules
            .insert("access_nt_world_browse_auto".into(), ProfileDecision::Allow);
        developer
            .rules
            .insert("access_tor_network".into(), ProfileDecision::Deny);
        developer.approval_mode_override = Some("auto-edit".into());
        profiles.insert("developer".into(), developer);

        Self {
            profiles,
            active: "nt_shield".into(),
        }
    }

    /// Resolve the effective rules for a profile (merging parent chain).
    pub fn resolve(&self, name: &str) -> Option<HashMap<String, ProfileDecision>> {
        let profile = self.profiles.get(name)?;
        let mut merged = HashMap::new();

        // Walk parent chain: root first, then child overrides
        let mut chain: Vec<&PermissionProfile> = vec![profile];
        let mut current = profile.parent.as_deref();
        while let Some(parent_name) = current {
            if let Some(parent) = self.profiles.get(parent_name) {
                chain.push(parent);
                current = parent.parent.as_deref();
            } else {
                break;
            }
        }
        for p in chain.into_iter().rev() {
            for (k, v) in &p.rules {
                merged.insert(k.clone(), *v);
            }
        }
        Some(merged)
    }

    /// Get the effective approval_mode_override, walking parent chain.
    pub fn resolve_approval_mode(&self, name: &str) -> Option<String> {
        let profile = self.profiles.get(name)?;
        if profile.approval_mode_override.is_some() {
            return profile.approval_mode_override.clone();
        }
        if let Some(ref parent) = profile.parent {
            return self.resolve_approval_mode(parent);
        }
        None
    }

    /// Evaluate a single action key against a resolved rule set.
    pub fn evaluate(&self, profile_name: &str, action_key: &str) -> Option<ProfileDecision> {
        let rules = self.resolve(profile_name)?;
        rules.get(action_key).copied()
    }
}

/// Config path: ~/.neotrix/profiles.toml
fn profiles_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".neotrix").join("profiles.toml")
}

/// Global profile manager.
static PROFILE_MANAGER: OnceLock<Mutex<ProfileStore>> = OnceLock::new();

pub fn global_profile_manager() -> &'static Mutex<ProfileStore> {
    PROFILE_MANAGER.get_or_init(|| {
        let path = profiles_path();
        let store = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str::<ProfileStore>(&s).ok())
                .unwrap_or_default()
        } else {
            let store = ProfileStore::builtin();
            let _ = save_profiles_to_disk(&store);
            store
        };
        Mutex::new(store)
    })
}

fn save_profiles_to_disk(store: &ProfileStore) -> Result<(), String> {
    let path = profiles_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let data = toml::to_string_pretty(store).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Reinitialize the profile manager (for testing).
pub fn reset_profile_manager() {
    if let Some(mutex) = PROFILE_MANAGER.get() {
        if let Ok(mut guard) = mutex.lock() {
            *guard = ProfileStore::builtin();
        }
    }
}

/// Public API: switch to a named profile.
/// Also applies the profile's approval_mode_override to the global approval engine.
pub fn switch_profile(name: &str) -> Result<String, String> {
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if !guard.profiles.contains_key(name) {
        return Err(format!(
            "Profile '{}' not found. Use /profile list to see available profiles.",
            name
        ));
    }
    guard.active = name.to_string();
    save_profiles_to_disk(&guard)?;

    // Apply approval mode override if set on this profile (or inherited)
    let mode_override = guard.resolve_approval_mode(name);
    drop(guard);
    if let Some(mode_str) = mode_override {
        if let Some(mode) = crate::cli::approval::ApprovalMode::from_str(&mode_str) {
            if let Ok(mut engine) = crate::cli::approval::global_approval().lock() {
                engine.set_mode(mode);
            }
        }
    }

    Ok(format!("Switched to profile: {}", name))
}

/// Public API: get active profile name.
pub fn active_profile_name() -> String {
    global_profile_manager()
        .lock()
        .map(|g| g.active.clone())
        .unwrap_or_else(|_| "nt_shield".to_string())
}

/// Public API: create a new profile (optional parent).
pub fn create_profile(name: &str, parent: Option<&str>) -> Result<String, String> {
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' already exists.", name));
    }
    if let Some(p) = parent {
        if !guard.profiles.contains_key(p) {
            return Err(format!("Parent profile '{}' not found.", p));
        }
    }
    let mut profile = PermissionProfile::new(name);
    profile.parent = parent.map(String::from);
    guard.profiles.insert(name.to_string(), profile);
    save_profiles_to_disk(&guard)?;
    Ok(format!("Created profile: {} (parent: {:?})", name, parent))
}

/// Public API: remove a profile (cannot remove built-in).
pub fn remove_profile(name: &str) -> Result<String, String> {
    let builtins = ["nt_shield", "strict-nt_shield", "general", "developer"];
    if builtins.contains(&name) {
        return Err(format!("Cannot remove built-in profile: {}", name));
    }
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if !guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' not found.", name));
    }
    if guard.active == name {
        guard.active = "nt_shield".into();
    }
    guard.profiles.remove(name);
    save_profiles_to_disk(&guard)?;
    Ok(format!("Removed profile: {}", name))
}

/// Public API: set a rule for a profile (action_key → decision).
pub fn set_rule(profile_name: &str, action_key: &str, decision: &str) -> Result<String, String> {
    let decision = match decision {
        "allow" | "Allow" => ProfileDecision::Allow,
        "deny" | "Deny" => ProfileDecision::Deny,
        "ask" | "Ask" => ProfileDecision::Ask,
        _ => {
            return Err(format!(
                "Invalid decision: {}. Use allow|deny|ask.",
                decision
            ))
        }
    };
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    let profile = guard
        .profiles
        .get_mut(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found.", profile_name))?;
    profile.rules.insert(action_key.to_string(), decision);
    save_profiles_to_disk(&guard)?;
    Ok(format!(
        "Set rule: {} → {:?} in profile '{}'",
        action_key, decision, profile_name
    ))
}

/// Public API: get profile info (rules, parent, effective mode).
pub fn get_profile_info(name: &str) -> Result<serde_json::Value, String> {
    let guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    let profile = guard
        .profiles
        .get(name)
        .ok_or_else(|| format!("Profile '{}' not found.", name))?;
    let rules = guard.resolve(name).unwrap_or_default();
    let approval_mode = guard.resolve_approval_mode(name);
    let mut rules_json = serde_json::Map::new();
    for (k, v) in &rules {
        let v_str = match v {
            ProfileDecision::Allow => "allow",
            ProfileDecision::Deny => "deny",
            ProfileDecision::Ask => "ask",
        };
        rules_json.insert(k.clone(), serde_json::Value::String(v_str.to_string()));
    }
    Ok(serde_json::json!({
        "name": profile.name,
        "parent": profile.parent,
        "approval_mode_override": profile.approval_mode_override,
        "effective_approval_mode": approval_mode,
        "effective_rules": rules_json,
    }))
}

/// Check whether an action is denied by the active profile.
pub fn is_action_denied(action_key: &str) -> bool {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    match guard.evaluate(&guard.active, action_key) {
        Some(ProfileDecision::Deny) => true,
        _ => false,
    }
}

/// Check whether an action should be silently allowed (no approval needed).
pub fn is_action_allowed(action_key: &str) -> bool {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    match guard.evaluate(&guard.active, action_key) {
        Some(ProfileDecision::Allow) => true,
        _ => false,
    }
}

/// List all profiles (names).
pub fn list_profiles() -> Vec<String> {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    let mut names: Vec<String> = guard.profiles.keys().cloned().collect();
    names.sort();
    names
}

/// Map an ActionType to a profile action key string.
pub fn action_type_to_key(action: &crate::cli::approval::ActionType) -> &'static str {
    use crate::cli::approval::ActionType;
    match action {
        ActionType::FileWrite { .. } => "write_file",
        ActionType::FileCreate { .. } => "write_file",
        ActionType::FileEdit { .. } => "write_file",
        ActionType::ShellCommand { .. } => "execute_command",
        ActionType::GitOperation { .. } => "git_push",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_builtin_profiles_exist() {
        let store = ProfileStore::builtin();
        assert!(store.profiles.contains_key("nt_shield"));
        assert!(store.profiles.contains_key("strict-nt_shield"));
        assert!(store.profiles.contains_key("general"));
        assert!(store.profiles.contains_key("developer"));
        assert_eq!(store.active, "nt_shield");
    }

    #[test]
    fn test_resolve_inheritance() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("strict-nt_shield").unwrap();
        // inherited from nt_shield
        assert_eq!(rules.get("read_secrets"), Some(&ProfileDecision::Deny));
        // overridden in strict-nt_shield
        assert_eq!(rules.get("write_file"), Some(&ProfileDecision::Deny));
        // inherited
        assert_eq!(rules.get("compile_check"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_general_allows_write() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("general").unwrap();
        assert_eq!(rules.get("write_file"), Some(&ProfileDecision::Allow));
        assert_eq!(rules.get("execute_command"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_developer() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("developer").unwrap();
        assert_eq!(rules.get("delete_file"), Some(&ProfileDecision::Allow));
        assert_eq!(
            rules.get("access_tor_network"),
            Some(&ProfileDecision::Deny)
        );
        assert_eq!(rules.get("read_file"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_approval_mode_override() {
        let store = ProfileStore::builtin();
        assert_eq!(
            store.resolve_approval_mode("developer"),
            Some("auto-edit".to_string())
        );
        assert_eq!(store.resolve_approval_mode("nt_shield"), None);
    }

    #[test]
    fn test_evaluate() {
        let store = ProfileStore::builtin();
        assert_eq!(
            store.evaluate("nt_shield", "read_secrets"),
            Some(ProfileDecision::Deny)
        );
        assert_eq!(
            store.evaluate("nt_shield", "write_file"),
            Some(ProfileDecision::Ask)
        );
        assert_eq!(
            store.evaluate("general", "execute_command"),
            Some(ProfileDecision::Allow)
        );
    }

    #[serial]
    #[test]
    fn test_global_state_integration() {
        reset_profile_manager();
        let _ = crate::cli::approval::global_approval()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_mode(crate::cli::approval::ApprovalMode::Suggest);

        // create + remove custom
        assert!(create_profile("my-custom", Some("developer")).is_ok());
        let names = list_profiles();
        assert!(names.contains(&"my-custom".to_string()));
        assert!(remove_profile("my-custom").is_ok());
        let names = list_profiles();
        assert!(!names.contains(&"my-custom".to_string()));

        // remove builtin fails
        assert!(remove_profile("nt_shield").is_err());
        assert!(remove_profile("developer").is_err());

        // create duplicate fails
        assert!(create_profile("nt_shield", None).is_err());

        // set rule
        assert!(set_rule("nt_shield", "custom_action", "allow").is_ok());
        {
            let guard = global_profile_manager()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let profile = guard.profiles.get("nt_shield").unwrap();
            assert_eq!(
                profile.rules.get("custom_action"),
                Some(&ProfileDecision::Allow)
            );
        }

        // invalid decision
        assert!(set_rule("nt_shield", "foo", "maybe").is_err());

        // switch profile
        assert!(switch_profile("developer").is_ok());
        assert_eq!(active_profile_name(), "developer");
        assert!(switch_profile("nt_shield").is_ok());

        // switch nonexistent
        assert!(switch_profile("nonexistent").is_err());

        // is_action_denied
        assert!(switch_profile("strict-nt_shield").is_ok());
        assert!(is_action_denied("write_file"));
        assert!(!is_action_denied("compile_check"));

        // is_action_allowed
        assert!(switch_profile("general").is_ok());
        assert!(is_action_allowed("write_file"));
        assert!(!is_action_allowed("read_secrets"));

        // list
        let names = list_profiles();
        assert!(names.contains(&"nt_shield".to_string()));
        assert!(names.contains(&"developer".to_string()));
        assert!(names.len() >= 4);

        // get_profile_info
        let info = get_profile_info("developer").unwrap();
        assert_eq!(info["name"], "developer");
        assert_eq!(
            info["parent"],
            serde_json::Value::String("general".to_string())
        );
        assert!(info["effective_rules"].is_object());

        assert!(get_profile_info("does-not-exist").is_err());

        // switch_profile applies approval override
        reset_profile_manager();
        crate::cli::approval::global_approval()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_mode(crate::cli::approval::ApprovalMode::Suggest);
        assert!(switch_profile("developer").is_ok());
        {
            let engine = crate::cli::approval::global_approval()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            assert_eq!(
                engine.mode(),
                crate::cli::approval::ApprovalMode::AutoEdit,
                "switching to developer should apply auto-edit override"
            );
        }
        assert!(switch_profile("nt_shield").is_ok());
        crate::cli::approval::global_approval()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_mode(crate::cli::approval::ApprovalMode::Suggest);
    }

    #[test]
    fn test_action_type_to_key() {
        use crate::cli::approval::ActionType;
        assert_eq!(
            action_type_to_key(&ActionType::FileWrite {
                path: "x".into(),
                content_preview: "".into()
            }),
            "write_file"
        );
        assert_eq!(
            action_type_to_key(&ActionType::FileCreate { path: "x".into() }),
            "write_file"
        );
        assert_eq!(
            action_type_to_key(&ActionType::FileEdit {
                path: "x".into(),
                diff: "".into()
            }),
            "write_file"
        );
        assert_eq!(
            action_type_to_key(&ActionType::ShellCommand {
                command: "ls".into()
            }),
            "execute_command"
        );
        assert_eq!(
            action_type_to_key(&ActionType::GitOperation {
                description: "commit".into()
            }),
            "git_push"
        );
    }
}
