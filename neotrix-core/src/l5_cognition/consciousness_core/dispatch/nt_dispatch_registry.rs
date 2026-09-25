//! 能力注册表装载与持久化（load / persist）（由 `dispatch.rs` 纯搬移拆分，行为零变更）

pub fn capability_registry_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = std::path::PathBuf::from(&home)
        .join(".neotrix")
        .join("capability_registry.json");
    let cwd_path = std::path::PathBuf::from(".neotrix").join("capability_registry.json");
    if cwd_path.exists() {
        cwd_path
    } else if home_path.exists() {
        home_path
    } else {
        cwd_path
    }
}

pub fn load_capability_registry() -> Option<nt_core_capability_tree::registry::CapabilityRegistry> {
    let path = capability_registry_path();
    let json = std::fs::read_to_string(path).ok()?;
    let export: nt_core_capability_tree::registry::RegistryExport =
        serde_json::from_str(&json).ok()?;
    let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
    for node in export.nodes {
        if registry.register(node).is_err() {
            return None;
        }
    }
    for (from, to) in export.edges {
        if registry.nodes.contains_key(&from) && registry.nodes.contains_key(&to) {
            let _ = registry.add_dependency(&from, &to);
        }
    }
    registry.experience_targets = export.experience_targets;
    let _ = nt_core_capability_tree::cad_node::register_cad_capability(&mut registry);
    let overlay_path = capability_registry_path()
        .parent()
        .map(|p| p.join("capability_overrides.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("capability_overrides.json"));
    if let Some(ov) =
        nt_core_capability_tree::registry::CapabilityRegistry::load_overlay_file(&overlay_path)
    {
        registry.merge_overlay(&ov);
    }
    Some(registry)
}

pub fn persist_capability_registry(
    registry: &nt_core_capability_tree::registry::CapabilityRegistry,
) -> Result<(), String> {
    let path = capability_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("registry dir: {}", e))?;
    }
    let export = registry.export();
    let json = serde_json::to_string_pretty(&export).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {}", e))
}
