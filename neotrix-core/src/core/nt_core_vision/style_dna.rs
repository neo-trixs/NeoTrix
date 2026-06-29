use std::collections::HashMap;

/// A style DNA profile — deterministic style parameters from xiaohu-ip-studio
#[derive(Debug, Clone)]
pub struct StyleDna {
    pub id: String,
    pub name: String,
    pub rendering: RenderingProfile,
    pub lighting: LightingProfile,
    pub color_grading: ColorGradingProfile,
    pub post_process: PostProcessProfile,
}

#[derive(Debug, Clone)]
pub struct RenderingProfile {
    pub style_keywords: Vec<String>,
    pub line_art_weight: f64,
    pub shading: ShadingStyle,
    pub texture_emphasis: f64,
}

#[derive(Debug, Clone)]
pub enum ShadingStyle {
    Cel,
    Smooth,
    Crosshatch,
    Watercolor,
    None,
}

#[derive(Debug, Clone)]
pub struct LightingProfile {
    pub mood: LightingMood,
    pub key_light_dir: String,
}

#[derive(Debug, Clone)]
pub enum LightingMood {
    Warm,
    Cool,
    Dramatic,
    Soft,
    Neon,
    Natural,
    Golden,
}

#[derive(Debug, Clone)]
pub struct ColorGradingProfile {
    pub saturation: f64,
    pub warmth: f64,
    pub contrast: f64,
    pub dominant_hues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PostProcessProfile {
    pub film_grain: f64,
    pub bloom: f64,
    pub vignette: f64,
}

/// Style DNA registry
#[derive(Debug, Clone)]
pub struct StyleDnaRegistry {
    styles: HashMap<String, StyleDna>,
}

impl StyleDnaRegistry {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }

    pub fn register(&mut self, s: StyleDna) {
        self.styles.insert(s.id.clone(), s);
    }

    pub fn get(&self, id: &str) -> Option<&StyleDna> {
        self.styles.get(id)
    }

    pub fn all_ids(&self) -> Vec<String> {
        self.styles.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.styles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Seed with 8+ default styles: Ghibli, Cyberpunk, Watercolor, OilPainting, LineArt, Anime, Realistic, Sketch
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();

        reg.register(StyleDna {
            id: "ghibli".to_string(),
            name: "Studio Ghibli".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "Studio Ghibli style".to_string(),
                    "hand-drawn animation".to_string(),
                    "soft watercolor backgrounds".to_string(),
                    "Miyazaki".to_string(),
                ],
                line_art_weight: 0.3,
                shading: ShadingStyle::Cel,
                texture_emphasis: 0.8,
            },
            lighting: LightingProfile {
                mood: LightingMood::Warm,
                key_light_dir: "golden hour".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.7,
                warmth: 0.8,
                contrast: 0.5,
                dominant_hues: vec!["green".to_string(), "blue".to_string(), "gold".to_string()],
            },
            post_process: PostProcessProfile {
                film_grain: 0.1,
                bloom: 0.3,
                vignette: 0.1,
            },
        });

        reg.register(StyleDna {
            id: "cyberpunk".to_string(),
            name: "Cyberpunk".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "cyberpunk style".to_string(),
                    "neon noir".to_string(),
                    "high contrast".to_string(),
                    "rainy city streets".to_string(),
                ],
                line_art_weight: 0.6,
                shading: ShadingStyle::Smooth,
                texture_emphasis: 0.9,
            },
            lighting: LightingProfile {
                mood: LightingMood::Neon,
                key_light_dir: "neon signs from above".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.9,
                warmth: 0.2,
                contrast: 0.9,
                dominant_hues: vec![
                    "magenta".to_string(),
                    "cyan".to_string(),
                    "black".to_string(),
                ],
            },
            post_process: PostProcessProfile {
                film_grain: 0.4,
                bloom: 0.7,
                vignette: 0.3,
            },
        });

        reg.register(StyleDna {
            id: "watercolor".to_string(),
            name: "Watercolor".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "watercolor painting".to_string(),
                    "wet on wet technique".to_string(),
                    "soft edges".to_string(),
                    "paper texture".to_string(),
                ],
                line_art_weight: 0.2,
                shading: ShadingStyle::Watercolor,
                texture_emphasis: 1.0,
            },
            lighting: LightingProfile {
                mood: LightingMood::Soft,
                key_light_dir: "diffuse overhead".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.6,
                warmth: 0.6,
                contrast: 0.4,
                dominant_hues: vec![
                    "pastel blue".to_string(),
                    "rose".to_string(),
                    "mint".to_string(),
                ],
            },
            post_process: PostProcessProfile {
                film_grain: 0.5,
                bloom: 0.2,
                vignette: 0.2,
            },
        });

        reg.register(StyleDna {
            id: "oil_painting".to_string(),
            name: "Oil Painting".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "oil painting".to_string(),
                    "impasto technique".to_string(),
                    "rich brushstrokes".to_string(),
                    "canvas texture".to_string(),
                ],
                line_art_weight: 0.1,
                shading: ShadingStyle::Smooth,
                texture_emphasis: 0.9,
            },
            lighting: LightingProfile {
                mood: LightingMood::Dramatic,
                key_light_dir: "chiaroscuro from left".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.8,
                warmth: 0.7,
                contrast: 0.8,
                dominant_hues: vec![
                    "burnt sienna".to_string(),
                    "ochre".to_string(),
                    "umber".to_string(),
                ],
            },
            post_process: PostProcessProfile {
                film_grain: 0.3,
                bloom: 0.1,
                vignette: 0.4,
            },
        });

        reg.register(StyleDna {
            id: "line_art".to_string(),
            name: "Line Art".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "line art".to_string(),
                    "ink drawing".to_string(),
                    "black and white".to_string(),
                    "clean strokes".to_string(),
                ],
                line_art_weight: 1.0,
                shading: ShadingStyle::Crosshatch,
                texture_emphasis: 0.5,
            },
            lighting: LightingProfile {
                mood: LightingMood::Natural,
                key_light_dir: "flat".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.0,
                warmth: 0.5,
                contrast: 1.0,
                dominant_hues: vec![],
            },
            post_process: PostProcessProfile {
                film_grain: 0.0,
                bloom: 0.0,
                vignette: 0.0,
            },
        });

        reg.register(StyleDna {
            id: "anime".to_string(),
            name: "Anime".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "anime style".to_string(),
                    "vibrant colors".to_string(),
                    "big expressive eyes".to_string(),
                    "cel shading".to_string(),
                ],
                line_art_weight: 0.5,
                shading: ShadingStyle::Cel,
                texture_emphasis: 0.3,
            },
            lighting: LightingProfile {
                mood: LightingMood::Soft,
                key_light_dir: "soft key light".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.9,
                warmth: 0.6,
                contrast: 0.6,
                dominant_hues: vec![
                    "pink".to_string(),
                    "lavender".to_string(),
                    "sky blue".to_string(),
                ],
            },
            post_process: PostProcessProfile {
                film_grain: 0.0,
                bloom: 0.5,
                vignette: 0.1,
            },
        });

        reg.register(StyleDna {
            id: "realistic".to_string(),
            name: "Realistic".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "photorealistic".to_string(),
                    "hyper-realistic".to_string(),
                    "detailed textures".to_string(),
                    "natural lighting".to_string(),
                ],
                line_art_weight: 0.0,
                shading: ShadingStyle::Smooth,
                texture_emphasis: 1.0,
            },
            lighting: LightingProfile {
                mood: LightingMood::Natural,
                key_light_dir: "natural window light".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.5,
                warmth: 0.5,
                contrast: 0.6,
                dominant_hues: vec!["neutral".to_string()],
            },
            post_process: PostProcessProfile {
                film_grain: 0.1,
                bloom: 0.0,
                vignette: 0.1,
            },
        });

        reg.register(StyleDna {
            id: "sketch".to_string(),
            name: "Sketch".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec![
                    "pencil sketch".to_string(),
                    "rough lines".to_string(),
                    "unfinished".to_string(),
                    "concept art".to_string(),
                ],
                line_art_weight: 0.8,
                shading: ShadingStyle::Crosshatch,
                texture_emphasis: 0.7,
            },
            lighting: LightingProfile {
                mood: LightingMood::Natural,
                key_light_dir: "top-down".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 0.1,
                warmth: 0.5,
                contrast: 0.7,
                dominant_hues: vec!["graphite".to_string(), "white".to_string()],
            },
            post_process: PostProcessProfile {
                film_grain: 0.6,
                bloom: 0.0,
                vignette: 0.2,
            },
        });

        reg
    }

    /// Compose style prompt from DNA for image gen backend
    pub fn to_style_prompt(&self, id: &str) -> Result<String, String> {
        let style = self
            .get(id)
            .ok_or_else(|| format!("Style '{}' not found", id))?;

        let mood_desc = match style.lighting.mood {
            LightingMood::Warm => "warm, cozy lighting".to_string(),
            LightingMood::Cool => "cool, blue-toned lighting".to_string(),
            LightingMood::Dramatic => "dramatic, high-contrast lighting".to_string(),
            LightingMood::Soft => "soft, diffused lighting".to_string(),
            LightingMood::Neon => "neon, colorful lighting".to_string(),
            LightingMood::Natural => "natural, even lighting".to_string(),
            LightingMood::Golden => "golden hour, warm glow".to_string(),
        };

        let keywords = style.rendering.style_keywords.join(", ");
        let shading_desc = match style.rendering.shading {
            ShadingStyle::Cel => "cel shaded".to_string(),
            ShadingStyle::Smooth => "smooth shading".to_string(),
            ShadingStyle::Crosshatch => "crosshatch shading".to_string(),
            ShadingStyle::Watercolor => "watercolor wash".to_string(),
            ShadingStyle::None => "flat colors".to_string(),
        };

        Ok(format!(
            "{}. Style: {}. Shading: {}. Lighting: {} ({}). Color: saturation {}, warmth {}, contrast {}. Post: grain {}, bloom {}, vignette {}.",
            keywords,
            style.name,
            shading_desc,
            mood_desc,
            style.lighting.key_light_dir,
            style.color_grading.saturation,
            style.color_grading.warmth,
            style.color_grading.contrast,
            style.post_process.film_grain,
            style.post_process.bloom,
            style.post_process.vignette,
        ))
    }
}

impl Default for StyleDnaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_styles_count() {
        let reg = StyleDnaRegistry::with_defaults();
        assert_eq!(reg.len(), 8);
    }

    #[test]
    fn test_get_style() {
        let reg = StyleDnaRegistry::with_defaults();
        let s = reg.get("ghibli").unwrap();
        assert_eq!(s.name, "Studio Ghibli");
    }

    #[test]
    fn test_get_missing_style() {
        let reg = StyleDnaRegistry::new();
        assert!(reg.get("vaporwave").is_none());
    }

    #[test]
    fn test_to_style_prompt() {
        let reg = StyleDnaRegistry::with_defaults();
        let prompt = reg.to_style_prompt("cyberpunk").unwrap();
        assert!(prompt.contains("Cyberpunk"));
        assert!(prompt.contains("neon"));
    }

    #[test]
    fn test_to_style_prompt_missing() {
        let reg = StyleDnaRegistry::new();
        let result = reg.to_style_prompt("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_ids() {
        let reg = StyleDnaRegistry::with_defaults();
        let ids = reg.all_ids();
        assert!(ids.contains(&"anime".to_string()));
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn test_render_profile_fields() {
        let reg = StyleDnaRegistry::with_defaults();
        let s = reg.get("line_art").unwrap();
        assert_eq!(s.rendering.shading, ShadingStyle::Crosshatch);
        assert!((s.rendering.line_art_weight - 1.0).abs() < f64::EPSILON);
        assert!(s.rendering.style_keywords.contains(&"line art".to_string()));
    }

    #[test]
    fn test_lighting_mood_variants() {
        let reg = StyleDnaRegistry::with_defaults();
        let ghibli = reg.get("ghibli").unwrap();
        let sketch = reg.get("sketch").unwrap();
        assert!(matches!(ghibli.lighting.mood, LightingMood::Warm));
        assert!(matches!(sketch.lighting.mood, LightingMood::Natural));
    }

    #[test]
    fn test_post_process_defaults() {
        let reg = StyleDnaRegistry::with_defaults();
        let realistic = reg.get("realistic").unwrap();
        assert!((realistic.post_process.bloom - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_register_custom_style() {
        let mut reg = StyleDnaRegistry::new();
        reg.register(StyleDna {
            id: "vaporwave".to_string(),
            name: "Vaporwave".to_string(),
            rendering: RenderingProfile {
                style_keywords: vec!["vaporwave".to_string(), "80s retro".to_string()],
                line_art_weight: 0.3,
                shading: ShadingStyle::Smooth,
                texture_emphasis: 0.5,
            },
            lighting: LightingProfile {
                mood: LightingMood::Neon,
                key_light_dir: "pink grid horizon".to_string(),
            },
            color_grading: ColorGradingProfile {
                saturation: 1.0,
                warmth: 0.3,
                contrast: 0.7,
                dominant_hues: vec!["pink".to_string(), "cyan".to_string(), "purple".to_string()],
            },
            post_process: PostProcessProfile {
                film_grain: 0.2,
                bloom: 0.6,
                vignette: 0.3,
            },
        });
        assert_eq!(reg.len(), 1);
        assert!(reg.get("vaporwave").is_some());
    }

    #[test]
    fn test_empty_registry() {
        let reg = StyleDnaRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }
}
