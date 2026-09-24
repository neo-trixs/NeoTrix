//! 预烘焙字形图集文本（RENDER-MIN §1）：运行时零光栅化。
//!
//! 根治 macroquad font-atlas 在 Retina 2× 下膨胀至 32768² →
//! `TextureFormat::size` u32 溢出 abort（2026-09-24 实锤）。
//! 度量离线烘焙（`assets/fonts/bake_atlas.py`），此处只解析 + 排版 + 送显。

use std::collections::HashMap;

use macroquad::prelude::{Color, DrawTextureParams, Rect, Texture2D, draw_texture_ex, vec2};

/// 图集边长（px）与烘焙字号（px），须与 bake_atlas.py 一致。
pub const ATLAS_PX: f32 = 2048.0;
pub const BASE_PX: f32 = 32.0;
/// 基线换算：dest_top = baseline_y - size * BASELINE_K（SHOT 目检调定）。
pub const BASELINE_K: f32 = 0.84;

/// 单字形度量（图集 px）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub adv: f32,
    pub top: f32,
}

pub struct AtlasFont {
    tex: Texture2D,
    map: HashMap<char, Glyph>,
}

impl AtlasFont {
    /// 加载图集纹理 + 度量（GL 上下文内调用；失败回 None 走 macroquad 兜底）。
    pub fn load() -> Option<Self> {
        let tex = Texture2D::from_file_with_format(
            include_bytes!("../assets/fonts/glyph_atlas.png"),
            None,
        );
        let map = Self::parse_metrics(include_str!("../assets/fonts/glyph_metrics.ron"));
        if map.len() < 2000 || tex.width() != ATLAS_PX {
            return None;
        }
        println!("atlas baked font: {} glyphs", map.len());
        Some(Self { tex, map })
    }

    /// 解析度量 RON（纯函数）：`(ch:"X",x:..,y:..,w:..,h:..,adv:..,top:..)` 逐行。
    pub fn parse_metrics(ron: &str) -> HashMap<char, Glyph> {
        let mut map = HashMap::new();
        for line in ron.lines() {
            let t = line.trim().trim_matches(|c| c == '(' || c == ')' || c == ',');
            let rest = match t.strip_prefix("ch:\"") {
                Some(r) => r,
                None => continue,
            };
            // 字形字符取到 `",x:` 边界（`"` 本身亦可正确解析为单字符）。
            let end = match rest.find("\",x:") {
                Some(i) => i,
                None => continue,
            };
            let mut cs = rest[..end].chars();
            let ch = match (cs.next(), cs.next()) {
                (Some(c), None) => c,
                _ => continue,
            };
            let mut g = Glyph { x: 0.0, y: 0.0, w: 0.0, h: 0.0, adv: 0.0, top: 0.0 };
            let mut ok = true;
            // 首字段 x 在 `",x:` 后无键名，补上再统一解析。
            for kv in format!("x:{}", &rest[end + 4..]).split(',') {
                let mut it = kv.split(':');
                let (k, v) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
                let f: f32 = v.parse().unwrap_or(f32::NAN);
                if f.is_nan() {
                    ok = false;
                    break;
                }
                match k {
                    "x" => g.x = f,
                    "y" => g.y = f,
                    "w" => g.w = f,
                    "h" => g.h = f,
                    "adv" => g.adv = f,
                    "top" => g.top = f,
                    _ => {}
                }
            }
            if ok && g.w > 0.0 && g.h > 0.0 && g.adv > 0.0 {
                map.insert(ch, g);
            }
        }
        map
    }

    /// 行宽（未知字按 0.6em 垫，保证布局不断）。
    pub fn measure(&self, s: &str, size: f32) -> f32 {
        let sc = size / BASE_PX;
        s.chars()
            .map(|c| self.map.get(&c).map(|g| g.adv).unwrap_or(size * 0.6) * sc)
            .sum()
    }

    /// 绘制（y 为基线，与 macroquad draw_text 调用点兼容）。
    pub fn draw(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        let sc = size / BASE_PX;
        let mut pen = x;
        for c in s.chars() {
            if let Some(g) = self.map.get(&c) {
                draw_texture_ex(
                    &self.tex,
                    pen,
                    y - size * BASELINE_K + g.top * sc,
                    color,
                    DrawTextureParams {
                        dest_size: Some(vec2(g.w * sc, g.h * sc)),
                        source: Some(Rect::new(g.x, g.y, g.w, g.h)),
                        ..Default::default()
                    },
                );
                pen += g.adv * sc;
            } else {
                pen += size * 0.6 * sc;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_map() -> HashMap<char, Glyph> {
        AtlasFont::parse_metrics(include_str!("../assets/fonts/glyph_metrics.ron"))
    }

    #[test]
    fn metrics_count() {
        assert_eq!(full_map().len(), 2593);
    }

    #[test]
    fn lookup_gui() {
        let g = full_map()[&'歸'];
        assert_eq!((g.x, g.y, g.w, g.h), (800.0, 2000.0, 32.0, 30.0));
        assert!((g.adv - 32.0).abs() < 0.01);
    }

    #[test]
    fn quote_char_parses() {
        let m = full_map();
        assert!(m.contains_key(&'"'));
    }

    #[test]
    fn garbage_skipped() {
        let m = AtlasFont::parse_metrics("[\n(ch:\"好\",x:0,y:0,w:20,h:20,adv:20.0,top:2)\nbroken line\n]\n");
        assert_eq!(m.len(), 1);
    }
}
