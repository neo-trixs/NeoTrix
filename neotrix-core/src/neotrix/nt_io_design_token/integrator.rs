use super::easing::EntranceAnimation;
use super::renderer::{FilterChain, TokenRenderer};
use super::token_types::{DesignToken, HierarchyLevel, TokenRegistry, TokenType, TokenValue};
use super::vsa_token;

pub struct DesignTokenIntegrator {
    pub registry: TokenRegistry,
    pub last_diagnostic: String,
    pub renderer: TokenRenderer,
}

impl DesignTokenIntegrator {
    pub fn new() -> Self {
        let registry = TokenRegistry::default();
        let renderer = TokenRenderer::new(registry.clone());
        DesignTokenIntegrator {
            registry,
            last_diagnostic: String::new(),
            renderer,
        }
    }

    pub fn diagnostic(&self) -> String {
        let total = self.registry.tokens.len();
        let color_count = self.registry.resolve_by_type(TokenType::Color).len();
        let spacing_count = self.registry.resolve_by_type(TokenType::Spacing).len();
        let easing_count = self.registry.resolve_by_type(TokenType::Easing).len();
        let shadow_count = self.registry.resolve_by_type(TokenType::Shadow).len();
        let motion_count = self.registry.resolve_by_type(TokenType::Motion).len();
        let hierarchy_levels = self.registry.hierarchy.len();
        format!(
            "DesignToken|total={}|types=[color:{},spacing:{},easing:{},shadow:{},motion:{}]|hierarchy={}|easing_presets={}",
            total, color_count, spacing_count, easing_count, shadow_count, motion_count, hierarchy_levels, easing_count
        )
    }

    pub fn resolve(&self, name: &str) -> Option<&DesignToken> {
        self.registry.resolve(name)
    }

    pub fn generate_filter_chain(&self, scene_type: &str) -> String {
        let mut chain = FilterChain::new();
        match scene_type {
            s if s.starts_with("manifesto-s") => {
                let zoompan = self.renderer.generate_zoompan(1.02, 1.06, 360);
                chain.push(&zoompan);
                let entrance = EntranceAnimation::fade_rise_blur();
                for filter in self.renderer.generate_entrance(&entrance, 180) {
                    chain.push(&filter);
                }
                let shadow = self.renderer.generate_shadow_overlay("shadow-glow");
                if !shadow.is_empty() {
                    chain.push(&shadow);
                }
            }
            "hud" => {
                let pos = FilterChain::generate_hud_position(true);
                chain.push(&pos);
            }
            _ => {}
        }
        chain.build()
    }

    pub fn register_scene_tokens(&mut self) {
        let scene_tokens: [(&str, &str, TokenType, TokenValue); 5] = [
            (
                "motion-scene-entrance",
                "Scene entrance motion",
                TokenType::Motion,
                TokenValue::Motion {
                    duration_ms: 350,
                    stiffness: 180.0,
                    damping: 15.0,
                },
            ),
            (
                "easing-scene-zoom",
                "Scene zoom easing",
                TokenType::Easing,
                TokenValue::Easing {
                    x1: 0.4,
                    y1: 0.0,
                    x2: 0.2,
                    y2: 1.0,
                },
            ),
            (
                "color-scene-bg",
                "Scene background color",
                TokenType::Color,
                TokenValue::Color {
                    r: 0.04,
                    g: 0.04,
                    b: 0.08,
                    a: 1.0,
                },
            ),
            (
                "shadow-scene-overlay",
                "Scene overlay shadow",
                TokenType::Shadow,
                TokenValue::Shadow {
                    offset_x: 0.0,
                    offset_y: 0.0,
                    blur: 16.0,
                    spread: 0.0,
                    r: 0.0,
                    g: 0.53,
                    b: 0.82,
                    a: 0.08,
                },
            ),
            (
                "spacing-scene-padding",
                "Scene padding",
                TokenType::Spacing,
                TokenValue::Spacing(24.0),
            ),
        ];

        for (name, desc, ttype, value) in &scene_tokens {
            if self.registry.resolve(name).is_none() {
                let mut token = DesignToken::new(name, desc, ttype.clone(), value.clone());
                token.vsa_vector = vsa_token::encode_token_name(name);
                self.registry
                    .register(token, HierarchyLevel::Semantic("scene".to_string()));
            }
        }
    }
}
