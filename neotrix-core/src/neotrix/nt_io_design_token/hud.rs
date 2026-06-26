use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use crate::core::nt_core_util;

pub struct HUDOverlay {
    pub width: u32,
    pub height: u32,
    pub cycle: u64,
    pub drive_name: String,
    pub gdi: f64,
    pub handler_count: usize,
    pub entropy: f64,
    pub popcount: f64,
    pub cos_sim: f64,
}

impl Default for HUDOverlay {
    fn default() -> Self {
        let cycle = Self::read_cycle_counter();
        HUDOverlay {
            width: 2560,
            height: 1440,
            cycle,
            drive_name: "Explore".to_string(),
            gdi: 0.35,
            handler_count: 191,
            entropy: 0.72,
            popcount: 0.48,
            cos_sim: 0.63,
        }
    }
}

impl HUDOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metrics(
        cycle: u64,
        drive_name: &str,
        gdi: f64,
        handler_count: usize,
        entropy: f64,
        popcount: f64,
        cos_sim: f64,
    ) -> Self {
        HUDOverlay {
            width: 2560,
            height: 1440,
            cycle,
            drive_name: drive_name.to_string(),
            gdi,
            handler_count,
            entropy,
            popcount,
            cos_sim,
        }
    }

    pub fn render_to_ppm(&self) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut pixels = vec![0u8; w * h * 3];

        let panel_x = w as i32 - 400 - 30;
        let panel_y = h as i32 - 280 - 30;
        let panel_w = 400;
        let panel_h = 280;

        for py in 0..panel_h {
            for px in 0..panel_w {
                let dx = px as f64;
                let dy = py as f64;
                let corner_r = 12.0;
                let in_corner = if dx < corner_r && dy < corner_r {
                    (dx - corner_r).hypot(dy - corner_r) > corner_r
                } else if dx >= (panel_w as f64 - corner_r) && dy < corner_r {
                    (dx - (panel_w as f64 - corner_r)).hypot(dy - corner_r) > corner_r
                } else if dx < corner_r && dy >= (panel_h as f64 - corner_r) {
                    (dx - corner_r).hypot(dy - (panel_h as f64 - corner_r)) > corner_r
                } else if dx >= (panel_w as f64 - corner_r) && dy >= (panel_h as f64 - corner_r) {
                    (dx - (panel_w as f64 - corner_r)).hypot(dy - (panel_h as f64 - corner_r))
                        > corner_r
                } else {
                    false
                };
                if !in_corner {
                    let sx = (panel_x + px as i32).clamp(0, w as i32 - 1) as usize;
                    let sy = (panel_y + py as i32).clamp(0, h as i32 - 1) as usize;
                    let idx = (sy * w + sx) * 3;
                    pixels[idx] = 10;
                    pixels[idx + 1] = 12;
                    pixels[idx + 2] = 24;
                }
            }
        }

        let margin_x = panel_x + 20;
        let mut text_y = panel_y + 28;
        let line_h = 24;

        let metrics = vec![
            format!("VSA TELEMETRY  cyc:{}", self.cycle),
            format!("DIM       {}", 4096),
            format!("POPCOUNT  {:.3}", self.popcount),
            format!("ENTROPY   {:.3}", self.entropy),
            format!("COS.SIM   {:.3}", self.cos_sim),
            format!("DRIVE/θ   {}", self.drive_name),
            format!("HANDLERS  {}", self.handler_count),
            format!("GDI       {:.3}", self.gdi),
        ];

        for line in &metrics {
            let mut lx = margin_x;
            for ch in line.chars() {
                let glyph_w = match ch {
                    '0'..='9' | 'A'..='Z' | 'a'..='z' => 8,
                    '/' | '.' | ' ' => 5,
                    _ => 7,
                };
                let gx = lx;
                let gy = text_y;
                if ch != ' ' {
                    for dy in 0..14 {
                        for dx in 0..glyph_w {
                            let sx = (gx + dx).clamp(0, w as i32 - 1) as usize;
                            let sy = (gy + dy).clamp(0, h as i32 - 1) as usize;
                            let idx = (sy * w + sx) * 3;
                            pixels[idx] = 235;
                            pixels[idx + 1] = 237;
                            pixels[idx + 2] = 242;
                        }
                    }
                }
                lx += glyph_w + 1;
            }
            text_y += line_h;
        }

        let mut ppm = Vec::with_capacity(w * h * 3 + 256);
        let header = format!("P6\n{} {}\n255\n", w, h);
        ppm.extend_from_slice(header.as_bytes());
        ppm.extend_from_slice(&pixels);
        ppm
    }

    pub fn save_ppm(&self, path: &str) -> std::io::Result<()> {
        let data = self.render_to_ppm();
        fs::write(path, data)
    }

    pub fn ffmpeg_overlay_filter(&self) -> String {
        "overlay=W-w-30:H-h-30,fade=t=in:st=5:d=0.5".to_string()
    }

    pub fn ffmpeg_hud_input(path: &str) -> String {
        format!("-loop 1 -i {}", path)
    }

    fn hud_cycle_path() -> String {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        format!("{}/.neotrix/hud-cycle.txt", home)
    }

    fn read_cycle_counter() -> u64 {
        let path_str = Self::hud_cycle_path();
        let path = Path::new(&path_str);
        let current = if path.exists() {
            let mut buf = String::new();
            if let Ok(mut f) = fs::File::open(path) {
                let _ = f.read_to_string(&mut buf);
            }
            buf.trim().parse::<u64>().unwrap_or(0)
        } else {
            0
        };
        let next = current.wrapping_add(1);
        if let Ok(mut f) = fs::File::create(path) {
            let _ = write!(f, "{}", next);
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_default_dimensions() {
        let hud = HUDOverlay::new();
        assert_eq!(hud.width, 2560);
        assert_eq!(hud.height, 1440);
    }

    #[test]
    fn test_hud_filter_generation() {
        let hud = HUDOverlay::new();
        let filter = hud.ffmpeg_overlay_filter();
        assert!(filter.contains("overlay=W-w-30:H-h-30"));
        assert!(filter.contains("fade=t=in:st=5:d=0.5"));
    }

    #[test]
    fn test_hud_ppm_renders() {
        let hud = HUDOverlay::with_metrics(42, "Explore", 0.35, 191, 0.72, 0.48, 0.63);
        let ppm = hud.render_to_ppm();
        assert!(ppm.len() > 256);
        assert!(ppm.starts_with(b"P6"));
    }
}
