use std::collections::HashMap;
use std::path::PathBuf;

/// Theme configuration (from Clawd theme.json)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub author: String,
    pub version: String,
    
    // States -> animation files
    pub states: HashMap<String, AnimationDef>,
    
    // Visual settings
    pub scale: f32,
    pub anchor: [f32; 2],        // Attachment point [x, y]
    pub shadow: ShadowDef,
    pub eye_tracking: bool,
}

/// Animation definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationDef {
    pub format: String,      // "gif", "apng", "webp", "svg", "png"
    pub path: String,        // Relative to theme dir
    pub frames: Option<u32>, // For sprite sheets
    pub fps: Option<f32>,    // Override default FPS
}

/// Shadow definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShadowDef {
    pub enabled: bool,
    pub offset: [f32; 2],
    pub blur: f32,
    pub color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert("idle".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "idle.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("thinking".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "thinking.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("typing".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "typing.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("building".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "building.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("groove".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "groove.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("juggling".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "juggling.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("error".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "error.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("happy".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "happy.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("notification".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "notification.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("sweeping".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "sweeping.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("carrying".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "carrying.gif".to_string(),
            frames: None,
            fps: None,
        });
        states.insert("sleeping".to_string(), AnimationDef {
            format: "gif".to_string(),
            path: "sleeping.gif".to_string(),
            frames: None,
            fps: None,
        });

        Self {
            name: "Default".to_string(),
            author: "NeoTrix".to_string(),
            version: "1.0.0".to_string(),
            states,
            scale: 1.0,
            anchor: [0.5, 0.5],
            shadow: ShadowDef {
                enabled: true,
                offset: [2.0, 2.0],
                blur: 4.0,
                color: "rgba(0,0,0,0.3)".to_string(),
            },
            eye_tracking: true,
        }
    }
}

/// Theme manager
pub struct ThemeManager {
    themes: HashMap<String, ThemeConfig>,
    current_theme: String,
    themes_dir: PathBuf,
}

impl ThemeManager {
    pub fn new(themes_dir: PathBuf) -> Self {
        let mut manager = Self {
            themes: HashMap::new(),
            current_theme: "default".to_string(),
            themes_dir,
        };
        
        // Load default theme
        manager.themes.insert("default".to_string(), ThemeConfig::default());
        
        manager
    }

    /// Load theme from directory
    pub fn load_theme(&mut self, name: &str, dir: PathBuf) -> Result<(), String> {
        let theme_json = dir.join("theme.json");
        if !theme_json.exists() {
            return Err(format!("Theme {} not found at {:?}", name, theme_json));
        }

        let content = std::fs::read_to_string(&theme_json)
            .map_err(|e| format!("Failed to read theme.json: {}", e))?;
        
        let config: ThemeConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse theme.json: {}", e))?;

        // Validate all animation files exist
        for (state, anim) in &config.states {
            let anim_path = dir.join(&anim.path);
            if !anim_path.exists() {
                return Err(format!("Missing animation for {}: {}", state, anim.path));
            }
        }

        self.themes.insert(name.to_string(), config);
        Ok(())
    }

    /// Set current theme
    pub fn set_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme {} not found", name))
        }
    }

    /// Get current theme
    pub fn current_theme(&self) -> &ThemeConfig {
        self.themes.get(&self.current_theme).unwrap()
    }

    /// Get theme by name
    pub fn get_theme(&self, name: &str) -> Option<&ThemeConfig> {
        self.themes.get(name)
    }

    /// List available themes
    pub fn list_themes(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Get animation path for state
    pub fn get_animation_path(&self, state: &str) -> Option<PathBuf> {
        let theme = self.current_theme();
        theme.states.get(state).map(|anim| {
            self.themes_dir.join(&self.current_theme).join(&anim.path)
        })
    }

    /// Get animation FPS for state
    pub fn get_animation_fps(&self, state: &str) -> f32 {
        let theme = self.current_theme();
        theme.states.get(state)
            .and_then(|anim| anim.fps)
            .unwrap_or(12.0) // Default 12 FPS for pixel art
    }

    /// Check if theme supports state
    pub fn supports_state(&self, state: &str) -> bool {
        let theme = self.current_theme();
        theme.states.contains_key(state)
    }

    /// Export theme to directory
    pub fn export_theme(&self, name: &str, dir: PathBuf) -> Result<(), String> {
        let theme = self.get_theme(name)
            .ok_or_else(|| format!("Theme {} not found", name))?;

        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        let theme_json = serde_json::to_string_pretty(theme)
            .map_err(|e| format!("Failed to serialize theme: {}", e))?;

        std::fs::write(dir.join("theme.json"), theme_json)
            .map_err(|e| format!("Failed to write theme.json: {}", e))?;

        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(PathBuf::from("themes"))
    }
}

/// Theme variant (Clawd has 3 built-in)
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeVariant {
    Clawd,      // Pixel crab
    Calico,     // 三花猫
    Cloudling,  // 云宝
    Custom(String),
}

impl ThemeVariant {
    pub fn name(&self) -> &str {
        match self {
            Self::Clawd => "Clawd",
            Self::Calico => "Calico",
            Self::Cloudling => "Cloudling",
            Self::Custom(name) => name,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Clawd" => Self::Clawd,
            "Calico" => Self::Calico,
            "Cloudling" => Self::Cloudling,
            _ => Self::Custom(name.to_string()),
        }
    }
}

/// Sprite sheet support
#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub texture_path: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub frames_per_row: u32,
    pub total_frames: u32,
}

impl SpriteSheet {
    pub fn new(texture_path: &str, frame_width: u32, frame_height: u32, total_frames: u32) -> Self {
        Self {
            texture_path: texture_path.to_string(),
            frame_width,
            frame_height,
            frames_per_row: 0, // Will be calculated
            total_frames,
        }
    }

    pub fn get_frame_rect(&self, frame: u32) -> [f32; 4] {
        let frames_per_row = if self.frames_per_row > 0 {
            self.frames_per_row
        } else {
            // Assume square texture
            1 // Will be calculated from texture
        };

        let row = frame / frames_per_row;
        let col = frame % frames_per_row;

        [
            col as f32 * self.frame_width as f32,
            row as f32 * self.frame_height as f32,
            self.frame_width as f32,
            self.frame_height as f32,
        ]
    }
}

/// Animation player
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    pub current_frame: u32,
    pub frame_timer: f32,
    pub fps: f32,
    pub loop_animation: bool,
    pub playing: bool,
}

impl AnimationPlayer {
    pub fn new(fps: f32) -> Self {
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            fps,
            loop_animation: true,
            playing: true,
        }
    }

    pub fn update(&mut self, dt: f32, total_frames: u32) {
        if !self.playing {
            return;
        }

        self.frame_timer += dt;
        if self.frame_timer >= 1.0 / self.fps {
            self.frame_timer = 0.0;
            self.current_frame += 1;
            if self.current_frame >= total_frames {
                if self.loop_animation {
                    self.current_frame = 0;
                } else {
                    self.current_frame = total_frames - 1;
                    self.playing = false;
                }
            }
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.current_frame = 0;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new(12.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_config_default() {
        let config = ThemeConfig::default();
        assert_eq!(config.name, "Default");
        assert_eq!(config.states.len(), 12);
        assert!(config.eye_tracking);
    }

    #[test]
    fn test_animation_player() {
        let mut player = AnimationPlayer::new(12.0);
        assert_eq!(player.current_frame, 0);
        assert!(player.playing);

        player.update(0.1, 10);
        assert_eq!(player.current_frame, 1);

        player.stop();
        assert!(!player.playing);
    }

    #[test]
    fn test_sprite_sheet() {
        let sheet = SpriteSheet::new("texture.png", 32, 32, 16);
        let rect = sheet.get_frame_rect(5);
        assert_eq!(rect, [160.0, 0.0, 32.0, 32.0]);
    }

    #[test]
    fn test_theme_variant() {
        assert_eq!(ThemeVariant::Clawd.name(), "Clawd");
        assert_eq!(ThemeVariant::from_name("Calico"), ThemeVariant::Calico);
        assert_eq!(ThemeVariant::Custom("test".to_string()).name(), "test");
    }
}
