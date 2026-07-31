use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub model: String,
    pub approval_mode: String,
    pub sandbox_mode: String,
    pub web_search_enabled: bool,
    pub context_compaction: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub theme: String,
    pub custom_instructions: Option<String>,
    pub mcp_servers: Vec<String>,
    pub plugins: Vec<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            model: "gpt-5.4".into(),
            approval_mode: "auto".into(),
            sandbox_mode: "workspace".into(),
            web_search_enabled: true,
            context_compaction: "medium".into(),
            max_tokens: 16384,
            temperature: 0.7,
            theme: "system".into(),
            custom_instructions: None,
            mcp_servers: Vec::new(),
            plugins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub config: ProfileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub total_profiles: u32,
    pub active_profile: String,
    pub default_profile: String,
    pub profiles_by_model: HashMap<String, u32>,
}

struct ProfileState {
    profiles: HashMap<String, ProfileInfo>,
    active_profile: String,
}

impl ProfileState {
    fn new() -> Self {
        let now = Utc::now().timestamp();
        let mut profiles = HashMap::new();

        let default_config = ProfileConfig::default();
        profiles.insert(
            "default".into(),
            ProfileInfo {
                name: "default".into(),
                description: "General purpose development".into(),
                is_active: true,
                is_default: true,
                created_at: now,
                updated_at: now,
                config: default_config,
            },
        );

        profiles.insert(
            "ci".into(),
            ProfileInfo {
                name: "ci".into(),
                description: "CI/CD pipeline, read-only".into(),
                is_active: false,
                is_default: false,
                created_at: now,
                updated_at: now,
                config: ProfileConfig {
                    model: "gpt-5.4-mini".into(),
                    approval_mode: "auto".into(),
                    sandbox_mode: "read-only".into(),
                    web_search_enabled: false,
                    context_compaction: "maximum".into(),
                    max_tokens: 4096,
                    temperature: 0.3,
                    ..Default::default()
                },
            },
        );

        profiles.insert(
            "exploration".into(),
            ProfileInfo {
                name: "exploration".into(),
                description: "Research and discovery".into(),
                is_active: false,
                is_default: false,
                created_at: now,
                updated_at: now,
                config: ProfileConfig {
                    model: "gpt-5.4".into(),
                    approval_mode: "suggest".into(),
                    sandbox_mode: "workspace".into(),
                    web_search_enabled: true,
                    context_compaction: "light".into(),
                    max_tokens: 32768,
                    temperature: 0.9,
                    ..Default::default()
                },
            },
        );

        Self {
            profiles,
            active_profile: "default".into(),
        }
    }
}

static PROFILE_STATE: LazyLock<Mutex<ProfileState>> =
    LazyLock::new(|| Mutex::new(ProfileState::new()));

fn lock_state() -> Result<std::sync::MutexGuard<'static, ProfileState>, String> {
    PROFILE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))
}

#[command]
pub fn profile_create(
    name: String,
    description: String,
    config: Option<ProfileConfig>,
) -> Result<String, String> {
    let mut state = lock_state()?;
    if state.profiles.contains_key(&name) {
        return Err(format!("Profile '{}' already exists", name));
    }
    let now = Utc::now().timestamp();
    state.profiles.insert(
        name.clone(),
        ProfileInfo {
            name: name.clone(),
            description,
            is_active: false,
            is_default: false,
            created_at: now,
            updated_at: now,
            config: config.unwrap_or_default(),
        },
    );
    Ok(name)
}

#[command]
pub fn profile_list() -> Result<Vec<ProfileInfo>, String> {
    let state = lock_state()?;
    let mut list: Vec<ProfileInfo> = state.profiles.values().cloned().collect();
    list.sort_by(|a, b| {
        if a.is_default { std::cmp::Ordering::Less }
        else if b.is_default { std::cmp::Ordering::Greater }
        else { a.name.cmp(&b.name) }
    });
    Ok(list)
}

#[command]
pub fn profile_get(name: String) -> Result<ProfileInfo, String> {
    let state = lock_state()?;
    state.profiles.get(&name).cloned().ok_or_else(|| format!("Profile '{}' not found", name))
}

#[command]
pub fn profile_update(
    name: String,
    description: Option<String>,
    config: Option<ProfileConfig>,
) -> Result<(), String> {
    let mut state = lock_state()?;
    let profile = state.profiles.get_mut(&name).ok_or_else(|| format!("Profile '{}' not found", name))?;
    if let Some(desc) = description {
        profile.description = desc;
    }
    if let Some(cfg) = config {
        profile.config = cfg;
    }
    profile.updated_at = Utc::now().timestamp();
    Ok(())
}

#[command]
pub fn profile_delete(name: String) -> Result<(), String> {
    let mut state = lock_state()?;
    let profile = state.profiles.get(&name).ok_or_else(|| format!("Profile '{}' not found", name))?;
    if profile.is_default {
        return Err("Cannot delete the default profile".into());
    }
    if profile.is_active {
        return Err("Cannot delete the active profile".into());
    }
    state.profiles.remove(&name);
    Ok(())
}

#[command]
pub fn profile_activate(name: String) -> Result<(), String> {
    let mut state = lock_state()?;
    if !state.profiles.contains_key(&name) {
        return Err(format!("Profile '{}' not found", name));
    }
    for (_, p) in state.profiles.iter_mut() {
        p.is_active = p.name == name;
    }
    state.active_profile = name;
    Ok(())
}

#[command]
pub fn profile_duplicate(
    name: String,
    new_name: String,
    new_description: Option<String>,
) -> Result<String, String> {
    let mut state = lock_state()?;
    let src = state.profiles.get(&name).cloned().ok_or_else(|| format!("Source profile '{}' not found", name))?;
    if state.profiles.contains_key(&new_name) {
        return Err(format!("Profile '{}' already exists", new_name));
    }
    let now = Utc::now().timestamp();
    state.profiles.insert(
        new_name.clone(),
        ProfileInfo {
            name: new_name.clone(),
            description: new_description.unwrap_or(src.description),
            is_active: false,
            is_default: false,
            created_at: now,
            updated_at: now,
            config: src.config,
        },
    );
    Ok(new_name)
}

#[command]
pub fn profile_reset(name: String) -> Result<(), String> {
    let mut state = lock_state()?;
    let profile = state.profiles.get_mut(&name).ok_or_else(|| format!("Profile '{}' not found", name))?;
    profile.config = ProfileConfig::default();
    profile.updated_at = Utc::now().timestamp();
    Ok(())
}

#[command]
pub fn profile_export(name: String) -> Result<String, String> {
    let state = lock_state()?;
    let profile = state.profiles.get(&name).ok_or_else(|| format!("Profile '{}' not found", name))?;
    serde_json::to_string_pretty(profile).map_err(|e| format!("Export failed: {}", e))
}

#[command]
pub fn profile_import(json_data: String) -> Result<String, String> {
    let mut state = lock_state()?;
    let mut profile: ProfileInfo =
        serde_json::from_str(&json_data).map_err(|e| format!("Import parse failed: {}", e))?;
    let final_name = if state.profiles.contains_key(&profile.name) {
        let imported = format!("{} (imported)", profile.name);
        imported
    } else {
        profile.name.clone()
    };
    let now = Utc::now().timestamp();
    profile.name = final_name.clone();
    profile.is_active = false;
    profile.is_default = false;
    profile.created_at = now;
    profile.updated_at = now;
    state.profiles.insert(final_name.clone(), profile);
    Ok(final_name)
}

#[command]
pub fn profile_summary() -> Result<ProfileSummary, String> {
    let state = lock_state()?;
    let total = state.profiles.len() as u32;
    let mut by_model: HashMap<String, u32> = HashMap::new();
    for (_, p) in state.profiles.iter() {
        *by_model.entry(p.config.model.clone()).or_default() += 1;
    }
    Ok(ProfileSummary {
        total_profiles: total,
        active_profile: state.active_profile.clone(),
        default_profile: "default".into(),
        profiles_by_model: by_model,
    })
}

fn template_config(
    model: &str,
    approval_mode: &str,
    sandbox_mode: &str,
    web_search: bool,
    compaction: &str,
) -> ProfileConfig {
    ProfileConfig {
        model: model.into(),
        approval_mode: approval_mode.into(),
        sandbox_mode: sandbox_mode.into(),
        web_search_enabled: web_search,
        context_compaction: compaction.into(),
        max_tokens: if model.contains("mini") { 8192 } else { 32768 },
        temperature: if compaction == "none" { 0.5 } else { 0.3 },
        ..Default::default()
    }
}

#[command]
pub fn profile_templates() -> Result<Vec<ProfileInfo>, String> {
    let now = Utc::now().timestamp();
    let templates = vec![
        ProfileInfo {
            name: "CI/CD".into(),
            description: "CI/CD pipeline with read-only sandbox, minimal model, aggressive compaction".into(),
            is_active: false, is_default: false,
            created_at: now, updated_at: now,
            config: template_config("gpt-5.4-mini", "auto", "read-only", false, "maximum"),
        },
        ProfileInfo {
            name: "Exploration".into(),
            description: "Research mode with web search, suggestion-based approvals".into(),
            is_active: false, is_default: false,
            created_at: now, updated_at: now,
            config: template_config("gpt-5.4", "suggest", "workspace", true, "light"),
        },
        ProfileInfo {
            name: "Security Audit".into(),
            description: "Security audit with manual approvals, read-only sandbox".into(),
            is_active: false, is_default: false,
            created_at: now, updated_at: now,
            config: template_config("gpt-5.4", "manual", "read-only", false, "aggressive"),
        },
        ProfileInfo {
            name: "Deep Research".into(),
            description: "Deep research with full web search, workspace sandbox".into(),
            is_active: false, is_default: false,
            created_at: now, updated_at: now,
            config: template_config("gpt-5.4", "auto", "workspace", true, "medium"),
        },
        ProfileInfo {
            name: "Quick Fix".into(),
            description: "Quick fixes with full sandbox, no compaction, mini model".into(),
            is_active: false, is_default: false,
            created_at: now, updated_at: now,
            config: template_config("gpt-5.4-mini", "auto", "full", false, "none"),
        },
    ];
    Ok(templates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_create_and_list() {
        let name = profile_create("test-profile".into(), "A test profile".into(), None).unwrap();
        assert_eq!(name, "test-profile");

        let list = profile_list().unwrap();
        assert!(list.iter().any(|p| p.name == "test-profile"));
        assert!(list.iter().any(|p| p.is_default));

        let _cleanup = profile_delete("test-profile".into());
    }

    #[test]
    fn test_profile_activate() {
        let name = profile_create("activate-test".into(), "".into(), None).unwrap();
        assert!(profile_activate(name.clone()).is_ok());

        let info = profile_get(name.clone()).unwrap();
        assert!(info.is_active);

        let summary = profile_summary().unwrap();
        assert_eq!(summary.active_profile, name);

        let _cleanup = profile_delete(name);
    }

    #[test]
    fn test_profile_delete_guards() {
        assert!(profile_delete("default".into()).is_err());
        assert!(profile_delete("nonexistent".into()).is_err());
    }

    #[test]
    fn test_profile_export_import() {
        let json = profile_export("default".into()).unwrap();
        assert!(json.contains("default"));

        let imported_name = profile_import(json.clone()).unwrap();
        assert!(imported_name.contains("imported"));

        let info = profile_get(imported_name.clone()).unwrap();
        assert_eq!(info.description, "General purpose development");

        let _cleanup = profile_delete(imported_name);
    }

    #[test]
    fn test_profile_templates() {
        let templates = profile_templates().unwrap();
        assert_eq!(templates.len(), 5);
        assert!(templates.iter().any(|t| t.name == "CI/CD"));
        assert!(templates.iter().any(|t| t.name == "Deep Research"));

        let cicd = templates.iter().find(|t| t.name == "CI/CD").unwrap();
        assert_eq!(cicd.config.sandbox_mode, "read-only");
        assert!(!cicd.config.web_search_enabled);
        assert_eq!(cicd.config.context_compaction, "maximum");
    }

    #[test]
    fn test_profile_duplicate() {
        let dup_name = profile_duplicate("default".into(), "default-copy".into(), Some("Copy".into())).unwrap();
        assert_eq!(dup_name, "default-copy");

        let copy = profile_get(dup_name.clone()).unwrap();
        assert_eq!(copy.description, "Copy");
        assert!(!copy.is_default);

        let _cleanup = profile_delete(dup_name);
    }

    #[test]
    fn test_profile_update_and_reset() {
        let name = profile_create("update-test".into(), "original".into(), None).unwrap();
        
        let custom = ProfileConfig {
            model: "custom-model".into(),
            max_tokens: 999,
            ..Default::default()
        };
        assert!(profile_update(name.clone(), Some("updated".into()), Some(custom)).is_ok());
        
        let info = profile_get(name.clone()).unwrap();
        assert_eq!(info.description, "updated");
        assert_eq!(info.config.model, "custom-model");
        assert_eq!(info.config.max_tokens, 999);
        
        assert!(profile_reset(name.clone()).is_ok());
        let reset = profile_get(name.clone()).unwrap();
        assert_eq!(reset.config.model, "gpt-5.4");
        
        let _cleanup = profile_delete(name);
    }

    #[test]
    fn test_profile_summary() {
        let summary = profile_summary().unwrap();
        assert_eq!(summary.default_profile, "default");
        assert!(summary.total_profiles >= 3);
        assert!(summary.profiles_by_model.contains_key("gpt-5.4"));
    }
}
