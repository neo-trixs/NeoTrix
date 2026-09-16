use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GlassConfig {
    pub blur_radius: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub tint_color: [f32; 4],
    pub tint_opacity: f32,
}

impl Default for GlassConfig {
    fn default() -> Self {
        Self {
            blur_radius: 20.0,
            saturation: 1.2,
            brightness: 1.0,
            contrast: 1.1,
            border_width: 0.5,
            border_color: [1.0, 1.0, 1.0, 0.3],
            tint_color: [0.9, 0.9, 1.0, 0.1],
            tint_opacity: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlassLayer {
    pub name: String,
    pub config: GlassConfig,
    pub opacity: f32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct GlassEffect {
    pub layers: Vec<GlassLayer>,
    pub composite_mode: CompositeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub background: [f32; 4],
}

pub struct LiquidGlassRenderer {
    effects: HashMap<String, GlassEffect>,
    context: RenderContext,
}

impl LiquidGlassRenderer {
    pub fn new(context: RenderContext) -> Self {
        Self {
            effects: HashMap::new(),
            context,
        }
    }

    pub fn create_glass(&mut self, name: &str, config: GlassConfig) -> &GlassLayer {
        let layer = GlassLayer {
            name: name.to_string(),
            config,
            opacity: 1.0,
            visible: true,
        };
        self.effects
            .entry(name.to_string())
            .or_insert_with(|| GlassEffect {
                layers: vec![],
                composite_mode: CompositeMode::Normal,
            })
            .layers
            .push(layer);
        self.effects.get(name).unwrap().layers.last().unwrap()
    }

    pub fn render(&self, effect_name: &str) -> Result<Vec<u8>, String> {
        let effect = self
            .effects
            .get(effect_name)
            .ok_or_else(|| format!("Effect not found: {}", effect_name))?;
        let pixel_count = (self.context.width * self.context.height * 4) as usize;
        let mut pixels = vec![0u8; pixel_count];
        for layer in &effect.layers {
            if !layer.visible {
                continue;
            }
            for chunk in pixels.chunks_exact_mut(4) {
                chunk[0] = (self.context.background[0] * layer.config.brightness * 255.0) as u8;
                chunk[1] = (self.context.background[1] * layer.config.brightness * 255.0) as u8;
                chunk[2] = (self.context.background[2] * layer.config.brightness * 255.0) as u8;
                chunk[3] = (layer.opacity * 255.0) as u8;
            }
        }
        Ok(pixels)
    }

    pub fn set_opacity(&mut self, effect_name: &str, opacity: f32) -> bool {
        if let Some(effect) = self.effects.get_mut(effect_name) {
            for layer in &mut effect.layers {
                layer.opacity = opacity;
            }
            true
        } else {
            false
        }
    }

    pub fn set_visible(&mut self, effect_name: &str, visible: bool) -> bool {
        if let Some(effect) = self.effects.get_mut(effect_name) {
            for layer in &mut effect.layers {
                layer.visible = visible;
            }
            true
        } else {
            false
        }
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    pub fn update_context(&mut self, context: RenderContext) {
        self.context = context;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_glass() {
        let mut r = LiquidGlassRenderer::new(RenderContext {
            width: 100,
            height: 100,
            scale_factor: 2.0,
            background: [0.0, 0.0, 0.0, 1.0],
        });
        let l = r.create_glass("blur", GlassConfig::default());
        assert_eq!(l.name, "blur");
        assert_eq!(r.effect_count(), 1);
    }

    #[test]
    fn test_render() {
        let mut r = LiquidGlassRenderer::new(RenderContext {
            width: 10,
            height: 10,
            scale_factor: 1.0,
            background: [0.5, 0.5, 0.5, 1.0],
        });
        r.create_glass("g", GlassConfig::default());
        let pixels = r.render("g").unwrap();
        assert_eq!(pixels.len(), 400);
    }

    #[test]
    fn test_not_found() {
        let r = LiquidGlassRenderer::new(RenderContext {
            width: 10,
            height: 10,
            scale_factor: 1.0,
            background: [0.0; 4],
        });
        assert!(r.render("missing").is_err());
    }
}
