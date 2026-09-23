//! Agent Gallery — 浏览/安装预设 agent 角色
//!
//! 管理预设 persona 模板：浏览可用角色、安装到 registry、社区贡献。
//! 设计启发: Munder Difflin Agent Gallery + skills browsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::l6_meta::nt_agent_identity::{AgentPersona, AgentPreset, AgentStatus, AutonomyLevel};

// ─── Gallery ────────────────────────────────────────────────────────────────

/// Agent Gallery — manages preset personas
pub struct AgentGallery {
    /// Available presets (built-in + community)
    presets: Vec<GalleryPreset>,
    /// Installed persona IDs (mapped from presets)
    installed: HashMap<String, String>, // preset_id → persona_id
}

/// A gallery preset — extendable beyond built-in presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPreset {
    /// Unique preset ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Avatar emoji
    pub avatar: String,
    /// Description
    pub description: String,
    /// Specialty domain
    pub specialty: String,
    /// Personality description
    pub personality: String,
    /// Default autonomy level
    pub autonomy: AutonomyLevel,
    /// Default cost budget (USD)
    pub cost_budget: f64,
    /// Preferred LLM provider
    pub provider: Option<String>,
    /// Preferred model
    pub model: Option<String>,
    /// System prompt prefix
    pub system_prompt: Option<String>,
    /// Tags for search
    pub tags: Vec<String>,
    /// Source: "builtin" or "community"
    pub source: String,
    /// Author (for community presets)
    pub author: Option<String>,
    /// Version
    pub version: String,
    /// Rating (0.0 - 5.0)
    pub rating: f64,
    /// Install count
    pub install_count: u64,
}

impl AgentGallery {
    /// Create gallery with built-in presets
    pub fn new() -> Self {
        let mut gallery = Self {
            presets: Vec::new(),
            installed: HashMap::new(),
        };
        gallery.register_builtin_presets();
        gallery
    }

    /// Register built-in presets from AgentPreset enum
    fn register_builtin_presets(&mut self) {
        for preset in AgentPreset::all() {
            let description = preset.description().to_string();
            let persona = AgentPersona::from_preset(preset);
            self.presets.push(GalleryPreset {
                id: format!("builtin-{}", persona.specialty),
                name: persona.name.clone(),
                avatar: persona.avatar.clone(),
                description,
                specialty: persona.specialty.clone(),
                personality: persona.personality.clone(),
                autonomy: persona.autonomy.clone(),
                cost_budget: persona.cost_budget,
                provider: persona.provider.clone(),
                model: persona.model.clone(),
                system_prompt: persona.system_prompt.clone(),
                tags: persona.tags.clone(),
                source: "builtin".to_string(),
                author: Some("NeoTrix".to_string()),
                version: "1.0.0".to_string(),
                rating: 4.5,
                install_count: 0,
            });
        }
    }

    /// Feed gallery from L5 preset AgentCards (E1.2 T15).
    ///
    /// Field mapping uses only fields observed on both sides:
    /// `id / name / description / tags / version`.
    /// Remaining `GalleryPreset` fields take builtin-style defaults;
    /// `install()` logic is untouched.
    pub fn register_builtin_from_presets(&mut self) {
        for card in crate::l5_cognition::nt_agent::presets::all_presets() {
            if self.presets.iter().any(|p| p.id == card.id) {
                continue;
            }
            let specialty = match card.tags.first() {
                Some(t) => t.clone(),
                None => card.id.clone(),
            };
            self.presets.push(GalleryPreset {
                id: card.id.clone(),
                name: card.name.clone(),
                avatar: "🤖".to_string(),
                description: card.description.clone(),
                specialty,
                personality: card.description.clone(),
                autonomy: AutonomyLevel::ReadOnly,
                cost_budget: 0.0,
                provider: None,
                model: None,
                system_prompt: None,
                tags: card.tags.clone(),
                source: "builtin".to_string(),
                author: Some("NeoTrix".to_string()),
                version: card.version.clone(),
                rating: 0.0,
                install_count: 0,
            });
        }
    }

    /// Register a community preset
    pub fn register_community(&mut self, preset: GalleryPreset) {
        self.presets.push(preset);
    }

    /// List all presets (with optional filter)
    pub fn list(&self, filter: Option<&str>) -> Vec<&GalleryPreset> {
        match filter {
            None => self.presets.iter().collect(),
            Some(f) => {
                let f_lower = f.to_lowercase();
                self.presets
                    .iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&f_lower)
                            || p.specialty.to_lowercase().contains(&f_lower)
                            || p.tags.iter().any(|t| t.to_lowercase().contains(&f_lower))
                    })
                    .collect()
            }
        }
    }

    /// Get a preset by ID
    pub fn get(&self, id: &str) -> Option<&GalleryPreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Install a preset → create AgentPersona
    pub fn install(&mut self, preset_id: &str) -> Result<AgentPersona, String> {
        let preset = self
            .presets
            .iter()
            .find(|p| p.id == preset_id)
            .ok_or_else(|| format!("Preset '{}' not found", preset_id))?
            .clone();

        let persona = AgentPersona {
            id: uuid_v4(),
            name: preset.name.clone(),
            avatar: preset.avatar.clone(),
            specialty: preset.specialty.clone(),
            personality: preset.personality.clone(),
            autonomy: preset.autonomy.clone(),
            cost_budget: preset.cost_budget,
            status: AgentStatus::Available,
            provider: preset.provider.clone(),
            model: preset.model.clone(),
            system_prompt: preset.system_prompt.clone(),
            tags: preset.tags.clone(),
            created_at: now_secs(),
            updated_at: now_secs(),
            session_count: 0,
            total_cost: 0.0,
            metadata: HashMap::new(),
        };

        // Record install
        if let Some(p) = self.presets.iter_mut().find(|p| p.id == preset_id) {
            p.install_count += 1;
        }

        self.installed.insert(preset_id.to_string(), persona.id.clone());
        Ok(persona)
    }

    /// Uninstall a preset (by persona ID)
    pub fn uninstall(&mut self, persona_id: &str) -> Option<String> {
        self.installed
            .iter()
            .find(|(_, pid)| *pid == persona_id)
            .map(|(preset_id, _)| preset_id.clone())
            .map(|preset_id| {
                self.installed.remove(&preset_id);
                preset_id
            })
    }

    /// Get installed persona IDs
    pub fn installed_ids(&self) -> Vec<&str> {
        self.installed.values().map(|s| s.as_str()).collect()
    }

    /// Get gallery stats
    pub fn stats(&self) -> GalleryStats {
        let builtin = self.presets.iter().filter(|p| p.source == "builtin").count();
        let community = self.presets.iter().filter(|p| p.source == "community").count();
        let installed_count = self.installed.len();
        let total_installs: u64 = self.presets.iter().map(|p| p.install_count).sum();
        GalleryStats {
            total_presets: self.presets.len(),
            builtin,
            community,
            installed: installed_count,
            total_installs,
        }
    }
}

impl Default for AgentGallery {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Stats ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryStats {
    pub total_presets: usize,
    pub builtin: usize,
    pub community: usize,
    pub installed: usize,
    pub total_installs: u64,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

use crate::l0_substrate::nt_core_time::now_secs;

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{:016x}-{:04x}-{:04x}-{:04x}-{:012x}",
        t,
        0x4000,
        0x8000,
        0xc000,
        t & 0xffffffffffff
    )
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_builtins() {
        let gallery = AgentGallery::new();
        let presets = gallery.list(None);
        assert_eq!(presets.len(), 6);
        assert!(presets.iter().all(|p| p.source == "builtin"));
    }

    #[test]
    fn test_gallery_filter() {
        let gallery = AgentGallery::new();
        let code_presets = gallery.list(Some("code"));
        assert!(code_presets.iter().all(|p| p.specialty.contains("code")));
    }

    #[test]
    fn test_install_preset() {
        let mut gallery = AgentGallery::new();
        let persona = gallery.install("builtin-code_generation").unwrap();
        assert_eq!(persona.name, "编码员");
        assert_eq!(persona.specialty, "code_generation");
        assert!(gallery.installed_ids().contains(&persona.id.as_str()));
    }

    #[test]
    fn test_install_nonexistent() {
        let mut gallery = AgentGallery::new();
        assert!(gallery.install("nonexistent").is_err());
    }

    #[test]
    fn test_uninstall() {
        let mut gallery = AgentGallery::new();
        let persona = gallery.install("builtin-code_generation").unwrap();
        assert!(gallery.uninstall(&persona.id).is_some());
        assert!(gallery.installed_ids().is_empty());
    }

    #[test]
    fn test_community_preset() {
        let mut gallery = AgentGallery::new();
        gallery.register_community(GalleryPreset {
            id: "community-translator".to_string(),
            name: "翻译员".to_string(),
            avatar: "🌐".to_string(),
            description: "多语言翻译助手".to_string(),
            specialty: "translation".to_string(),
            personality: "准确、自然".to_string(),
            autonomy: AutonomyLevel::ReadOnly,
            cost_budget: 2.0,
            provider: None,
            model: None,
            system_prompt: None,
            tags: vec!["translate".to_string()],
            source: "community".to_string(),
            author: Some("user123".to_string()),
            version: "1.0.0".to_string(),
            rating: 4.0,
            install_count: 5,
        });

        let stats = gallery.stats();
        assert_eq!(stats.community, 1);
        assert_eq!(stats.total_presets, 7);
    }

    #[test]
    fn test_gallery_stats() {
        let mut gallery = AgentGallery::new();
        gallery.install("builtin-code_generation").unwrap();
        gallery.install("builtin-code_review").unwrap();

        let stats = gallery.stats();
        assert_eq!(stats.builtin, 6);
        assert_eq!(stats.installed, 2);
    }
}
