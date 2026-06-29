use serde::{Deserialize, Serialize};

pub const VSA_DIM: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Color,
    Spacing,
    Easing,
    Shadow,
    Motion,
    Font,
    Radius,
    Opacity,
}

impl TokenType {
    pub fn name(&self) -> &str {
        match self {
            TokenType::Color => "color",
            TokenType::Spacing => "spacing",
            TokenType::Easing => "easing",
            TokenType::Shadow => "shadow",
            TokenType::Motion => "motion",
            TokenType::Font => "font",
            TokenType::Radius => "radius",
            TokenType::Opacity => "opacity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenValue {
    Color { r: f64, g: f64, b: f64, a: f64 },
    Spacing(f64),
    Easing { x1: f64, y1: f64, x2: f64, y2: f64 },
    Shadow { offset_x: f64, offset_y: f64, blur: f64, spread: f64, r: f64, g: f64, b: f64, a: f64 },
    Motion { duration_ms: u32, stiffness: f64, damping: f64 },
    Font { family: String, size: f64, weight: u16 },
    Radius(f64),
    Opacity(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignToken {
    pub name: String,
    pub description: String,
    pub token_type: TokenType,
    pub value: TokenValue,
    pub vsa_vector: Vec<u8>,
    pub semantic_path: Vec<String>,
}

impl DesignToken {
    pub fn new(name: &str, description: &str, token_type: TokenType, value: TokenValue) -> Self {
        let semantic_path = Self::build_semantic_path(name, &token_type);
        let vsa_vector = Vec::new();
        DesignToken {
            name: name.to_string(),
            description: description.to_string(),
            token_type,
            value,
            vsa_vector,
            semantic_path,
        }
    }

    fn build_semantic_path(name: &str, token_type: &TokenType) -> Vec<String> {
        let parts: Vec<&str> = name.split('-').collect();
        let mut path = vec![token_type.name().to_string()];
        for part in parts {
            path.push(part.to_string());
        }
        path
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyLevel {
    Global,
    Semantic(String),
    Component { component: String, variant: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRegistry {
    pub tokens: Vec<DesignToken>,
    pub hierarchy: Vec<(String, HierarchyLevel)>,
    aliases: Vec<(String, String)>,
}

impl Default for TokenRegistry {
    fn default() -> Self {
        let mut reg = TokenRegistry {
            tokens: Vec::new(),
            hierarchy: Vec::new(),
            aliases: Vec::new(),
        };
        reg.load_presets();
        reg
    }
}

impl TokenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, token: DesignToken, level: HierarchyLevel) {
        let name = token.name.clone();
        self.tokens.push(token);
        self.hierarchy.push((name, level));
    }

    pub fn resolve(&self, name: &str) -> Option<&DesignToken> {
        let resolved = self.resolve_alias(name);
        self.tokens.iter().find(|t| t.name == resolved)
    }

    pub fn resolve_by_type(&self, token_type: TokenType) -> Vec<&DesignToken> {
        self.tokens.iter().filter(|t| t.token_type == token_type).collect()
    }

    pub fn add_alias(&mut self, alias: &str, target: &str) {
        self.aliases.push((alias.to_string(), target.to_string()));
    }

    fn resolve_alias<'a>(&'a self, name: &'a str) -> &'a str {
        for (alias, target) in &self.aliases {
            if alias == name {
                return target;
            }
        }
        name
    }

    fn load_presets(&mut self) {
        // Colors — global palette
        for (name, desc, r, g, b, a) in &[
            ("color-brand-primary", "Primary brand color — deep indigo", 0.07, 0.09, 0.20, 1.0),
            ("color-brand-accent", "Accent — electric cyan", 0.0, 0.53, 0.82, 1.0),
            ("color-surface-dark", "Dark surface background", 0.04, 0.04, 0.08, 1.0),
            ("color-surface-mid", "Mid-tone surface", 0.10, 0.11, 0.18, 1.0),
            ("color-text-primary", "Primary text — white", 0.92, 0.93, 0.95, 1.0),
            ("color-text-secondary", "Secondary text — muted", 0.60, 0.62, 0.68, 1.0),
            ("color-feedback-success", "Success — emerald", 0.20, 0.78, 0.42, 1.0),
            ("color-feedback-warning", "Warning — amber", 0.95, 0.70, 0.12, 1.0),
            ("color-feedback-error", "Error — rose", 0.90, 0.22, 0.32, 1.0),
            ("color-life-purple", "Life overlay purple", 0.66, 0.33, 0.97, 0.08),
            ("color-life-cyan", "Life overlay cyan", 0.0, 0.83, 1.0, 0.05),
            ("color-flow-base", "Flow field base color", 0.05, 0.04, 0.10, 1.0),
        ] {
            let mut t = DesignToken::new(name, desc, TokenType::Color, TokenValue::Color { r: *r, g: *g, b: *b, a: *a });
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Spacing — 4px base scale
        for (name, desc, val) in &[
            ("spacing-xs", "4px", 4.0),
            ("spacing-sm", "8px", 8.0),
            ("spacing-md", "16px", 16.0),
            ("spacing-lg", "24px", 24.0),
            ("spacing-xl", "32px", 32.0),
            ("spacing-2xl", "48px", 48.0),
            ("spacing-3xl", "64px", 64.0),
        ] {
            let mut t = DesignToken::new(name, desc, TokenType::Spacing, TokenValue::Spacing(*val));
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Easing — cubic-bezier curves
        for (name, desc, x1, y1, x2, y2) in &[
            ("easing-standard", "Standard ease — both ends smooth", 0.4, 0.0, 0.2, 1.0),
            ("easing-entrance", "Entrance — decelerate into place", 0.0, 0.0, 0.2, 1.0),
            ("easing-exit", "Exit — accelerate away", 0.4, 0.0, 1.0, 1.0),
            ("easing-emphasis", "Emphasis — overshoot for attention", 0.4, 0.0, 0.6, 1.0),
            ("easing-spring-expressive", "Expressive spring — subtle bounce", 0.34, 1.56, 0.64, 1.0),
            ("easing-spring-standard", "Standard spring — minimal bounce", 0.34, 0.8, 0.64, 1.0),
        ] {
            let mut t = DesignToken::new(name, desc, TokenType::Easing,
                TokenValue::Easing { x1: *x1, y1: *y1, x2: *x2, y2: *y2 });
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Shadows — multi-layer depth system
        for (name, desc, ox, oy, blur, spread, r, g, b, a) in &[
            ("shadow-sm", "Subtle shadow — surface 1", 0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.08),
            ("shadow-md", "Medium shadow — surface 2", 0.0, 2.0, 6.0, 0.0, 0.0, 0.0, 0.0, 0.10),
            ("shadow-lg", "Large shadow — surface 3", 0.0, 4.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.12),
            ("shadow-hairline", "Hairline ring — edge definition", 0.0, 0.0, 0.5, 0.0, 1.0, 1.0, 1.0, 0.06),
            ("shadow-glow", "Glow — accent halo", 0.0, 0.0, 16.0, 0.0, 0.0, 0.53, 0.82, 0.08),
        ] {
            let mut t = DesignToken::new(name, desc, TokenType::Shadow,
                TokenValue::Shadow { offset_x: *ox, offset_y: *oy, blur: *blur, spread: *spread, r: *r, g: *g, b: *b, a: *a });
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Motion — spring-based animation parameters
        for (name, desc, dur, stiff, damp) in &[
            ("motion-quick", "Quick micro-interaction", 100, 300.0, 25.0),
            ("motion-moderate", "Standard transition", 200, 200.0, 20.0),
            ("motion-expressive", "Expressive entrance", 350, 180.0, 15.0),
            ("motion-slow", "Large transformation", 500, 120.0, 12.0),
        ] {
            let mut t = DesignToken::new(name, desc, TokenType::Motion,
                TokenValue::Motion { duration_ms: *dur, stiffness: *stiff, damping: *damp });
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Video rendering — zoom, flow field, grain, resolution parameters
        for (name, desc, val) in &[
            ("zoompan-speed", "Logo reveal zoom decay per frame", TokenValue::Opacity(0.015)),
            ("zoompan-fast", "Awakening fast zoom decay per frame", TokenValue::Opacity(0.04)),
            ("zoompan-slow", "Brand film slow zoom decay per frame", TokenValue::Opacity(0.01)),
            ("zoompan-moderate", "Hello world moderate zoom decay per frame", TokenValue::Opacity(0.025)),
            ("zoompan-manifesto", "Manifesto-60s zoom decay per frame", TokenValue::Opacity(0.03)),
            ("flow-field-width", "Flow/Conway life base render width px", TokenValue::Spacing(640.0)),
            ("flow-field-height", "Flow/Conway life base render height px", TokenValue::Spacing(360.0)),
            ("grain-opacity", "Film grain overlay opacity", TokenValue::Opacity(0.02)),
            ("resolution-hd-width", "HD video width px", TokenValue::Spacing(1920.0)),
            ("resolution-hd-height", "HD video height px", TokenValue::Spacing(1080.0)),
            ("resolution-2k-width", "2K video width px", TokenValue::Spacing(2560.0)),
            ("resolution-2k-height", "2K video height px", TokenValue::Spacing(1440.0)),
            ("zoompan-2k-base", "2K zoompan base zoom value", TokenValue::Opacity(1.02)),
            ("zoompan-2k-factor", "2K zoompan ramping factor", TokenValue::Opacity(0.04)),
        ] {
            let ttype = match val {
                TokenValue::Opacity(_) => TokenType::Opacity,
                TokenValue::Spacing(_) => TokenType::Spacing,
                _ => {
                    log::warn!("unexpected TokenValue variant in register_defaults, defaulting to Opacity");
                    TokenType::Opacity
                }
            };
            let mut t = DesignToken::new(name, desc, ttype, val.clone());
            t.vsa_vector = super::vsa_token::encode_token_name(name);
            self.register(t, HierarchyLevel::Global);
        }

        // Semantic-to-component mappings
        self.add_alias("color-button-primary-bg", "color-brand-primary");
        self.add_alias("color-button-primary-text", "color-text-primary");
        self.add_alias("spacing-button-padding", "spacing-md");
        self.add_alias("easing-button-press", "easing-spring-standard");
        self.add_alias("motion-button-press", "motion-quick");
        self.add_alias("shadow-button-rest", "shadow-sm");
        self.add_alias("shadow-button-hover", "shadow-md");
        self.add_alias("shadow-button-press", "shadow-hairline");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let reg = TokenRegistry::new();
        assert!(!reg.tokens.is_empty());
        assert!(reg.resolve("color-brand-primary").is_some());
    }

    #[test]
    fn test_alias_resolution() {
        let reg = TokenRegistry::new();
        let resolved = reg.resolve("color-button-primary-bg");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "color-brand-primary");
    }

    #[test]
    fn test_resolve_by_type() {
        let reg = TokenRegistry::new();
        let easings = reg.resolve_by_type(TokenType::Easing);
        assert!(easings.len() >= 4);
    }

    #[test]
    fn test_semantic_path() {
        let token = DesignToken::new("color-brand-primary", "test", TokenType::Color,
            TokenValue::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
        assert_eq!(token.semantic_path, vec!["color", "color-brand-primary"]);
    }
}
