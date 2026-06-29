// REVIVED Evo 4
use crate::core::nt_core_value_system::{CoreValue, ValueSystem};

#[derive(Debug, Clone)]
pub struct VisualSignature {
    pub palette: VisualPalette,
    pub geometry: VisualGeometry,
    pub dominant_value: Option<CoreValue>,
    pub overall_satisfaction: f64,
    pub coherence_level: f64,
}

#[derive(Debug, Clone)]
pub struct VisualPalette {
    pub primary_hex: String,
    pub secondary_hex: String,
    pub accent_hex: String,
    pub bg_hex: String,
    pub fg_hex: String,
    pub saturation: f64,
    pub warmth: f64,
}

#[derive(Debug, Clone)]
pub struct VisualGeometry {
    pub symmetry: f64,
    pub complexity: f64,
    pub sharpness: f64,
    pub roundness: f64,
}

fn value_to_hsl(value: CoreValue) -> (f64, f64, f64) {
    match value {
        CoreValue::Curiosity => (39.0, 0.95, 0.52),
        CoreValue::KnowledgeGrowth => (142.0, 0.76, 0.48),
        CoreValue::Coherence => (217.0, 0.91, 0.60),
        CoreValue::Autonomy => (271.0, 0.91, 0.65),
        CoreValue::Helpfulness => (330.0, 0.78, 0.60),
        CoreValue::Truthfulness => (187.0, 0.94, 0.43),
        CoreValue::Efficiency => (215.0, 0.16, 0.47),
    }
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn weighted_blend_hsl(values: &[(CoreValue, f64)]) -> (f64, f64, f64) {
    let mut total_w = 0.0_f64;
    let mut h_sum = 0.0_f64;
    let mut s_sum = 0.0_f64;
    let mut l_sum = 0.0_f64;
    for (v, w) in values {
        let (h, s, l) = value_to_hsl(*v);
        h_sum += h * w;
        s_sum += s * w;
        l_sum += l * w;
        total_w += w;
    }
    if total_w == 0.0 {
        return (271.0, 0.91, 0.65);
    }
    (
        h_sum / total_w,
        (s_sum / total_w).min(1.0),
        (l_sum / total_w).min(1.0),
    )
}

impl VisualSignature {
    pub fn from_value_system(vs: &ValueSystem) -> Self {
        let dominant = vs.dominant_value();
        let overall = vs.overall_satisfaction();
        let coherence = vs.get_satisfaction(CoreValue::Coherence);
        let curiosity = vs.get_satisfaction(CoreValue::Curiosity);
        let knowledge = vs.get_satisfaction(CoreValue::KnowledgeGrowth);
        let autonomy = vs.get_satisfaction(CoreValue::Autonomy);
        let helpfulness = vs.get_satisfaction(CoreValue::Helpfulness);

        let pairs: Vec<(CoreValue, f64)> = vs
            .weights
            .iter()
            .map(|w| (w.value, w.weight * w.satisfaction))
            .collect();
        let (ph, ps, pl) = weighted_blend_hsl(&pairs);
        let primary = hsl_to_hex(ph, ps, pl * 1.15);

        let accent_pairs: Vec<(CoreValue, f64)> = vs
            .weights
            .iter()
            .filter(|w| {
                let prod = w.weight * w.satisfaction;
                prod > 0.05 && w.value != vs.dominant_value().unwrap_or(CoreValue::Autonomy)
            })
            .map(|w| (w.value, w.weight * w.satisfaction * 0.6))
            .collect();
        let (ah, ash, al) = if accent_pairs.is_empty() {
            ((ph + 30.0) % 360.0, ps * 0.8, pl * 0.9)
        } else {
            weighted_blend_hsl(&accent_pairs)
        };
        let accent = hsl_to_hex(ah, ash, al * 1.1);

        let secondary = hsl_to_hex((ph + 45.0) % 360.0, ps * 0.6, pl * 1.05);

        let bg_light = (pl * 0.07).min(0.12);
        let bg = hsl_to_hex(ph, ps.max(0.3), bg_light);
        let fg_light = (pl * 0.85).min(0.95);
        let fg = hsl_to_hex(ph, ps * 0.15, fg_light);

        let sat = ps * (0.5 + overall * 0.5);
        let warmth = (1.0 - (ph - 30.0) / 360.0).max(0.2).min(0.9);

        let symmetry = coherence;
        let complexity = curiosity * 0.5 + knowledge * 0.3 + 0.2;
        let sharpness = autonomy * 0.6 + (1.0 - helpfulness) * 0.4;
        let roundness = helpfulness * 0.7 + coherence * 0.3;

        Self {
            palette: VisualPalette {
                primary_hex: primary,
                secondary_hex: secondary,
                accent_hex: accent,
                bg_hex: bg,
                fg_hex: fg,
                saturation: sat,
                warmth,
            },
            geometry: VisualGeometry {
                symmetry,
                complexity,
                sharpness,
                roundness,
            },
            dominant_value: dominant,
            overall_satisfaction: overall,
            coherence_level: coherence,
        }
    }

    pub fn generate_logo_svg(&self) -> String {
        let ring_r = match self.dominant_value {
            Some(v) if v == CoreValue::Helpfulness || v == CoreValue::Coherence => 28.0,
            _ => 32.0,
        };
        let stroke_w = 2.0 + (1.0 - self.geometry.sharpness) * 2.0;
        let corners = if self.geometry.roundness > 0.6 {
            "round"
        } else {
            "miter"
        };
        let node_count = (4.0 + self.geometry.complexity * 6.0).round() as u32;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
  <defs>
    <linearGradient id="lg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="{primary}"/>
      <stop offset="100%" stop-color="{accent}"/>
    </linearGradient>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="{bg}"/>
      <stop offset="100%" stop-color="{bg2}"/>
    </linearGradient>
  </defs>
  <rect width="200" height="200" rx="{rx}" fill="url(#bg)"/>
  <circle cx="100" cy="100" r="{ring_r}" stroke="url(#lg)" stroke-width="{sw}" fill="none" opacity="0.5"/>
  <circle cx="100" cy="100" r="{ring_r2}" stroke="{secondary}" stroke-width="{sw2}" fill="none" opacity="0.3" stroke-dasharray="{dash}"/>
"#,
            primary = self.palette.primary_hex.clone(),
            accent = self.palette.accent_hex.clone(),
            secondary = self.palette.secondary_hex.clone(),
            bg = self.palette.bg_hex.clone(),
            bg2 = self.palette.bg_hex.clone(),
            rx = if self.geometry.roundness > 0.5 { 24 } else { 4 },
            ring_r = ring_r,
            ring_r2 = ring_r * 1.5,
            sw = stroke_w,
            sw2 = (stroke_w * 0.7).max(1.0),
            dash = if self.geometry.symmetry > 0.6 {
                "3 4"
            } else {
                "2 6"
            },
        );

        for i in 0..node_count {
            let angle = std::f64::consts::TAU * i as f64 / node_count as f64
                * (1.0 + (1.0 - self.geometry.symmetry) * 0.3);
            let r = ring_r * 0.7 + (self.geometry.complexity * 8.0);
            let cx = 100.0 + angle.cos() * r;
            let cy = 100.0 + angle.sin() * r;
            let dot_r = 1.5 + self.geometry.symmetry * 1.5;
            svg.push_str(&format!(
                r#"  <circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{accent}" opacity="{op}"/>"#,
                cx,
                cy,
                dot_r,
                accent = self.palette.accent_hex.clone(),
                op = 0.4 + self.geometry.complexity * 0.4,
            ));
        }

        let diamond_h = 14.0 + self.geometry.sharpness * 8.0;
        let diamond_w = diamond_h * 0.6;
        let dm = if corners == "round" {
            "stroke-linecap=\"round\" stroke-linejoin=\"round\""
        } else {
            ""
        };
        let half_h = diamond_h / 2.0;
        svg.push_str(&format!(
            r###"
  <polygon points="100.0,{t} {r},100.0 100.0,{b} {l},100.0" stroke="url(#lg)" stroke-width="2.5" fill="none" {dm}/>
  <circle cx="100.0" cy="100.0" r="3" fill="url(#lg)" opacity="0.8"/>
</svg>"###,
            t = 100.0 - half_h,
            b = 100.0 + half_h,
            r = 100.0 + diamond_w,
            l = 100.0 - diamond_w,
            dm = dm,
        ));

        svg
    }

    pub fn generate_favicon_svg(&self) -> String {
        let primary = self.palette.primary_hex.clone();
        let is_rounded = self.geometry.roundness > 0.4;
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="{primary}"/>
      <stop offset="100%" stop-color="{accent}"/>
    </linearGradient>
  </defs>
  <rect width="64" height="64" rx="{rx}" fill="{bg}"/>
  <path d="{path}" stroke="url(#g)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
  <circle cx="32" cy="{cy}" r="2.5" fill="url(#g)" opacity="0.8"/>
</svg>"#,
            primary = primary,
            accent = self.palette.accent_hex.clone(),
            bg = self.palette.bg_hex.clone(),
            rx = if is_rounded { 12 } else { 4 },
            path = if self.geometry.sharpness > 0.5 {
                "M12 48 L22 24 L32 36 L42 18 L52 48"
            } else {
                "M14 46 L24 26 L32 38 L40 22 L50 46"
            },
            cy = if self.geometry.sharpness > 0.5 {
                36
            } else {
                34
            },
        )
    }

    pub fn generate_palette_swatch_svg(&self) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 60">
  <rect x="0" y="0" width="60" height="60" rx="8" fill="{p}"/>
  <rect x="75" y="0" width="60" height="60" rx="8" fill="{s}"/>
  <rect x="150" y="0" width="60" height="60" rx="8" fill="{a}"/>
  <rect x="225" y="0" width="60" height="60" rx="8" fill="{b}"/>
  <rect x="300" y="0" width="60" height="60" rx="8" fill="{f}"/>
</svg>"#,
            p = self.palette.primary_hex.clone(),
            s = self.palette.secondary_hex.clone(),
            a = self.palette.accent_hex.clone(),
            b = self.palette.bg_hex.clone(),
            f = self.palette.fg_hex.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_vs() -> ValueSystem {
        ValueSystem::new()
    }

    fn high_curiosity_vs() -> ValueSystem {
        let mut vs = ValueSystem::new();
        vs.record_satisfaction(CoreValue::Curiosity, 0.95);
        vs.record_satisfaction(CoreValue::KnowledgeGrowth, 0.8);
        vs.record_satisfaction(CoreValue::Autonomy, 0.7);
        vs
    }

    fn high_coherence_vs() -> ValueSystem {
        let mut vs = ValueSystem::new();
        vs.record_satisfaction(CoreValue::Coherence, 0.95);
        vs.record_satisfaction(CoreValue::Helpfulness, 0.9);
        vs.record_satisfaction(CoreValue::Truthfulness, 0.85);
        vs
    }

    #[test]
    fn test_default_signature_produces_valid_svg() {
        let vs = default_vs();
        let sig = VisualSignature::from_value_system(&vs);
        let svg = sig.generate_logo_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains(&sig.palette.primary_hex));
    }

    #[test]
    fn test_default_signature_dominant_value() {
        let sig = VisualSignature::from_value_system(&default_vs());
        assert!(sig.dominant_value.is_some());
    }

    #[test]
    fn test_high_curiosity_produces_different_svg() {
        let vs1 = default_vs();
        let vs2 = high_curiosity_vs();
        let sig1 = VisualSignature::from_value_system(&vs1);
        let sig2 = VisualSignature::from_value_system(&vs2);
        assert_ne!(sig1.palette.primary_hex, sig2.palette.primary_hex);
    }

    #[test]
    fn test_high_coherence_produces_different_geometry() {
        let vs1 = default_vs();
        let vs2 = high_coherence_vs();
        let sig1 = VisualSignature::from_value_system(&vs1);
        let sig2 = VisualSignature::from_value_system(&vs2);
        assert!(sig2.geometry.symmetry > sig1.geometry.symmetry);
        assert!(sig2.geometry.roundness > sig1.geometry.roundness);
    }

    #[test]
    fn test_favicon_svg_valid() {
        let sig = VisualSignature::from_value_system(&default_vs());
        let svg = sig.generate_favicon_svg();
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
    }

    #[test]
    fn test_palette_swatch_svg_valid() {
        let sig = VisualSignature::from_value_system(&default_vs());
        let swatch = sig.generate_palette_swatch_svg();
        assert!(swatch.contains(&sig.palette.primary_hex));
    }

    #[test]
    fn test_coherence_increases_roundness() {
        let vs1 = ValueSystem::new();
        let mut vs2 = ValueSystem::new();
        vs2.record_satisfaction(CoreValue::Coherence, 1.0);
        let sig1 = VisualSignature::from_value_system(&vs1);
        let sig2 = VisualSignature::from_value_system(&vs2);
        assert!(sig2.geometry.symmetry >= sig1.geometry.symmetry);
    }

    #[test]
    fn test_warmth_higher_for_curiosity() {
        let vs = high_curiosity_vs();
        let sig = VisualSignature::from_value_system(&vs);
        let mut vs2 = high_coherence_vs();
        vs2.record_satisfaction(CoreValue::Coherence, 0.95);
        let sig2 = VisualSignature::from_value_system(&vs2);
        assert!(sig.palette.warmth > 0.0);
        assert!(sig2.palette.warmth > 0.0);
    }

    fn value_hex(value: CoreValue) -> String {
        match value {
            CoreValue::Curiosity => "#f59e0b",
            CoreValue::KnowledgeGrowth => "#22c55e",
            CoreValue::Coherence => "#3b82f6",
            CoreValue::Autonomy => "#a855f7",
            CoreValue::Helpfulness => "#ec4899",
            CoreValue::Truthfulness => "#06b6d4",
            CoreValue::Efficiency => "#64748b",
        }
        .to_string()
    }

    #[test]
    fn test_all_values_have_colors() {
        for v in CoreValue::all() {
            let hex = value_hex(*v);
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
        }
    }
}
