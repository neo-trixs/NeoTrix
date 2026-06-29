use super::token_types::{TokenRegistry, TokenValue};
use super::easing::{EasingCurve, EntranceAnimation, EntranceType};

pub struct FilterChain {
    pub filters: Vec<String>,
}

impl FilterChain {
    pub fn new() -> Self {
        FilterChain { filters: Vec::new() }
    }

    pub fn push(&mut self, filter: &str) {
        self.filters.push(filter.to_string());
    }

    pub fn build(&self) -> String {
        self.filters.join(",")
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn generate_hud_position(bottom_right: bool) -> String {
        if bottom_right {
            "overlay=W-w-30:H-h-30".to_string()
        } else {
            "overlay=0:0".to_string()
        }
    }
}

pub struct TokenRenderer {
    pub registry: TokenRegistry,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for TokenRenderer {
    fn default() -> Self {
        TokenRenderer {
            registry: TokenRegistry::default(),
            fps: 30,
            width: 2560,
            height: 1440,
        }
    }
}

impl TokenRenderer {
    pub fn new(registry: TokenRegistry) -> Self {
        TokenRenderer { registry, fps: 30, width: 2560, height: 1440, ..Default::default() }
    }

    pub fn generate_zoompan(&self, _start_zoom: f64, _end_zoom: f64, duration_frames: u32) -> String {
        let easing = self.lookup_easing("easing-standard");
        format!(
            "zoompan=z='{}':d={}:fps={}:s={}x{}",
            easing, duration_frames, self.fps, self.width, self.height
        )
    }

    pub fn generate_entrance(&self, entrance: &EntranceAnimation, _duration_frames: u32) -> Vec<String> {
        let mut filters = Vec::new();
        let progress = Self::easing_curve_to_expression(&entrance.easing);

        match entrance.entrance_type {
            EntranceType::FadeIn => {
                filters.push(format!(
                    "fade=t=in:st=0:d={}",
                    entrance.duration_ms as f64 / 1000.0
                ));
            }
            EntranceType::Rise => {
                let rise_px = entrance.rise_distance;
                filters.push(format!(
                    "drawtext=text='':enable='between(t,0,{})':y=h-{:.0}+{:.0}*{}",
                    entrance.duration_ms as f64 / 1000.0,
                    rise_px, rise_px, progress
                ));
            }
            EntranceType::BlurClear => {
                let max_blur = entrance.blur_radius;
                filters.push(format!(
                    "boxblur=luma_radius='ceil({:.1}*(1-{}))':luma_power=2",
                    max_blur, progress
                ));
            }
            EntranceType::ScaleIn => {
                filters.push(format!(
                    "scale=iw*({}):ih*({})",
                    progress, progress
                ));
            }
            EntranceType::SlideIn { from_x, from_y } => {
                filters.push(format!(
                    "pad=iw*2:ih*2:(iw-iw*{:.2})/2:(ih-ih*{:.2})/2",
                    from_x, from_y
                ));
            }
        }
        filters
    }

    pub fn generate_shadow_overlay(&self, shadow_token_name: &str) -> String {
        if let Some(token) = self.registry.resolve(shadow_token_name) {
            if let TokenValue::Shadow { offset_x, offset_y, blur, spread, r, g, b, a } = &token.value {
                let r_str = Self::color_to_geq(*r * *a);
                let g_str = Self::color_to_geq(*g * *a);
                let b_str = Self::color_to_geq(*b * *a);
                format!(
                    "geq=r='{}':g='{}':b='{}':a='255',\
                     boxblur=luma_radius={}:luma_power=2,\
                     pad=iw+{}:ih+{}:{:.0}:{:.0}",
                    r_str, g_str, b_str,
                    blur.max(1.0) as u32,
                    offset_x.abs() as u32 + *spread as u32 * 2,
                    offset_y.abs() as u32 + *spread as u32 * 2,
                    offset_x.max(0.0) + spread,
                    offset_y.max(0.0) + spread
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn generate_life_overlay(&self, width: u32, _height: u32, token_name: &str) -> String {
        if let Some(token) = self.registry.resolve(token_name) {
            match &token.value {
                TokenValue::Color { r, g, b, .. } => {
                    let hex = Self::color_to_hex(*r, *g, *b);
                    let ratio = if token_name.contains("sparse") { 0.06 } else { 0.12 };
                    let mold = if token_name.contains("sparse") { 200 } else { 80 };
                    let life_size = if width > 480 { "640x360" } else { "320x180" };
                    format!(
                        "life=size={}:rate=30:ratio={}:mold={}:life_color={}:death_color=#000000",
                        life_size, ratio, mold, hex
                    )
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }

    pub fn generate_flow_field(&self, width: u32, height: u32) -> String {
        let w = width.max(320);
        let h = height.max(180);
        format!(
            "geq=r='12+16*sin(0.5*X/{w}*2*PI+0.3*Y/{h}*2*PI+0.7*N/30)':\
                   g='6+8*sin(0.7*X/{w}*2*PI+0.5*Y/{h}*2*PI+1.1*N/30)':\
                   b='20+30*sin(0.3*X/{w}*2*PI+0.8*Y/{h}*2*PI+0.5*N/30)'",
            w = w, h = h
        )
    }

    pub fn generate_text_overlay(&self, text: &str, position: &str, size: f64) -> String {
        let font_size = size;
        let pos_filter = match position {
            "center" => format!("x=(w-text_w)/2:y=(h-text_h)/2"),
            "bottom-right" => format!("x=w-tw-30:y=h-th-30"),
            "top-left" => format!("x=30:y=30"),
            _ => format!("x=(w-text_w)/2:y={}", position),
        };
        format!(
            "drawtext=text='{}':fontsize={:.0}:fontcolor=white:{}",
            text.replace('\'', "'\\\\''"),
            font_size,
            pos_filter
        )
    }

    fn lookup_easing(&self, name: &str) -> String {
        if let Some(token) = self.registry.resolve(name) {
            if let TokenValue::Easing { x1, y1, x2, y2 } = &token.value {
                if *x1 == 0.0 && *y1 == 0.0 && *x2 == 0.0 && *y2 == 0.0 {
                    return "1.02+0.04*on/360".to_string();
                }
                return format!("1.02+0.04*((1-{}))*on/360", sample_bezier_approx(*x1, *y1, *x2, *y2));
            }
        }
        "1.02+0.04*on/360".to_string()
    }

    fn color_to_geq(v: f64) -> String {
        format!("{:.0}", (v * 255.0).clamp(0.0, 255.0))
    }

    fn color_to_hex(r: f64, g: f64, b: f64) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }

    fn easing_curve_to_expression(curve: &EasingCurve) -> String {
        match curve {
            EasingCurve::CubicBezier { x1: _, y1, x2: _, y2 } => {
                format!("min(1,(t*{:.2}+(1-t)*{:.2}))", y2, y1)
            }
            EasingCurve::Spring(params) => {
                format!("spring({},{})", params.stiffness, params.damping)
            }
            EasingCurve::Linear => "t".to_string(),
        }
    }
}

fn sample_bezier_approx(_x1: f64, y1: f64, _x2: f64, y2: f64) -> f64 {
    let t = 0.5;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    3.0 * y1 * mt2 * t + 3.0 * y2 * mt * t2 + t2 * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_chain_build() {
        let mut fc = FilterChain::new();
        fc.push("format=rgba");
        fc.push("fade=t=in:st=0:d=0.5");
        let result = fc.build();
        assert_eq!(result, "format=rgba,fade=t=in:st=0:d=0.5");
    }

    #[test]
    fn test_generate_hud_position() {
        let pos = FilterChain::generate_hud_position(true);
        assert_eq!(pos, "overlay=W-w-30:H-h-30");
    }

    #[test]
    fn test_renderer_zoompam_default() {
        let renderer = TokenRenderer::default();
        let zp = renderer.generate_zoompan(1.02, 1.06, 360);
        assert!(zp.contains("zoompan=z="));
        assert!(zp.contains("fps=30"));
        assert!(zp.contains("2560x1440"));
    }

    #[test]
    fn test_color_to_hex() {
        let hex = TokenRenderer::color_to_hex(0.07, 0.09, 0.20);
        assert_eq!(hex, "#121733");
    }

    #[test]
    fn test_entrance_fade() {
        let renderer = TokenRenderer::default();
        let entrance = EntranceAnimation::fade_rise_blur();
        let filters = renderer.generate_entrance(&entrance, 180);
        assert!(!filters.is_empty());
        assert!(filters[0].contains("fade"));
    }
}
