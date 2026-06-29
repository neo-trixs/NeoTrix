use std::collections::HashMap;
use std::io::Write;

use base64::Engine;
use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::agent::tool::lifecycle::*;
use crate::core::nt_core_design_token::procedural::{
    extract_vsa_params, hsl_to_rgb, perlin_noise_2d,
};

// ─── Minimal PNG Encoder (zero external deps beyond flate2) ──────────────────

const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
};

fn crc32_bytes(data: &[u8]) -> u32 {
    let mut c = 0xffffffffu32;
    for &b in data {
        c = CRC32_TABLE[((c ^ b as u32) & 0xff) as usize] ^ (c >> 8);
    }
    c ^ 0xffffffff
}

pub(crate) fn png_encode(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        let len = data.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(chunk_type);
        out.extend_from_slice(data);
        let mut crc_buf = Vec::with_capacity(4 + data.len());
        crc_buf.extend_from_slice(chunk_type);
        crc_buf.extend_from_slice(data);
        let crc = crc32_bytes(&crc_buf);
        out.extend_from_slice(&crc.to_be_bytes());
    }

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(2);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(&mut out, b"IHDR", &ihdr);

    let mut raw = Vec::with_capacity((width as usize * 3 + 1) * height as usize);
    for y in 0..height {
        raw.push(0);
        let row_start = (y * width * 3) as usize;
        raw.extend_from_slice(&pixels[row_start..row_start + width as usize * 3]);
    }

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&raw)
        .expect("deflate encoder write failed");
    let compressed = encoder.finish().expect("deflate encoder finish failed");
    write_chunk(&mut out, b"IDAT", &compressed);

    write_chunk(&mut out, b"IEND", &[]);
    out
}

// ─── Procedural Generators ───────────────────────────────────────────────────

pub(crate) fn generate_mandelbrot(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let max_iter = 80 + (seed % 200) as u32;
    let zoom = 1.0 + (seed % 100) as f64 * 0.02;
    let xoff = -0.5 + (seed % 1000) as f64 * 0.001 - 0.5;
    let yoff = (seed % 1000) as f64 * 0.001 - 0.5;
    let hue_shift = (seed % 360) as f64 / 360.0;
    let mut pixels = vec![0u8; (width * height * 3) as usize];

    for py in 0..height {
        for px in 0..width {
            let x0 = (px as f64 / width as f64 - 0.5) * 3.5 / zoom + xoff;
            let y0 = (py as f64 / height as f64 - 0.5) * 2.5 / zoom + yoff;
            let mut x = 0.0f64;
            let mut y = 0.0f64;
            let mut iter = 0u32;
            while iter < max_iter {
                let xx = x * x - y * y + x0;
                let yy = 2.0 * x * y + y0;
                x = xx;
                y = yy;
                if x * x + y * y > 4.0 {
                    break;
                }
                iter += 1;
            }
            let idx = (py * width + px) as usize * 3;
            if iter == max_iter {
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
            } else {
                let t = iter as f64 / max_iter as f64;
                let hue = hue_shift + t * 0.7;
                let (r, g, b) = hsl_to_rgb(hue.fract(), 0.8, 0.5 + t * 0.4);
                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
            }
        }
    }
    pixels
}

pub(crate) fn generate_geometric(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut rng = fastrand::Rng::new();
    let bg_hue = seed as f64 / u64::MAX as f64;
    let mut pixels = vec![0u8; (width * height * 3) as usize];

    let (bg_r, bg_g, bg_b) = hsl_to_rgb(bg_hue, 0.3, 0.15);
    for p in pixels.chunks_exact_mut(3) {
        p[0] = bg_r;
        p[1] = bg_g;
        p[2] = bg_b;
    }

    let num_shapes = 8 + (seed % 20) as usize;
    for i in 0..num_shapes {
        let shape_type = rng.u32(0..4);
        let hue = bg_hue + (i as f64 * 0.13).fract();
        let (fr, fg, fb) = hsl_to_rgb(hue, 0.7 + rng.f64() * 0.3, 0.4 + rng.f64() * 0.4);
        let alpha = 0.3 + rng.f64() * 0.5;

        match shape_type {
            0 => {
                let cx = (rng.f64() * width as f64) as i32;
                let cy = (rng.f64() * height as f64) as i32;
                let r = (10.0 + rng.f64() * (width.min(height) as f64 * 0.3)) as i32;
                draw_circle(&mut pixels, width, height, cx, cy, r, fr, fg, fb, alpha);
            }
            1 => {
                let x = (rng.f64() * width as f64) as i32;
                let y = (rng.f64() * height as f64) as i32;
                let w = (10.0 + rng.f64() * (width as f64 * 0.4)) as u32;
                let h = (10.0 + rng.f64() * (height as f64 * 0.4)) as u32;
                draw_rect(&mut pixels, width, height, x, y, w, h, fr, fg, fb, alpha);
            }
            2 => {
                let pts: [(i32, i32); 3] = std::array::from_fn(|_| {
                    (
                        (rng.f64() * width as f64) as i32,
                        (rng.f64() * height as f64) as i32,
                    )
                });
                draw_triangle(&mut pixels, width, height, pts, fr, fg, fb, alpha);
            }
            3 => {
                let x1 = (rng.f64() * width as f64) as i32;
                let y1 = (rng.f64() * height as f64) as i32;
                let x2 = (rng.f64() * width as f64) as i32;
                let y2 = (rng.f64() * height as f64) as i32;
                let thickness = 1 + rng.u32(1..8) as i32;
                draw_line(
                    &mut pixels,
                    width,
                    height,
                    x1,
                    y1,
                    x2,
                    y2,
                    thickness,
                    fr,
                    fg,
                    fb,
                    alpha,
                );
            }
            _ => {}
        }
    }
    pixels
}

pub(crate) fn generate_perlin_art(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let params = extract_vsa_params(&create_vsa_seed(seed));
    let map: HashMap<&str, f64> = params.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let hue_base = *map.get("hue_primary").unwrap_or(&0.5);
    let hue_sec = *map.get("hue_secondary").unwrap_or(&0.3);
    let sat = *map.get("saturation").unwrap_or(&0.6);
    let bright = *map.get("brightness").unwrap_or(&0.5);
    let complexity = *map.get("complexity").unwrap_or(&0.5);

    let octaves = 2 + (complexity * 5.0) as usize;
    let scale = 0.02 + complexity * 0.08;
    let noise = perlin_noise_2d(width, height, scale, seed, octaves, 2.0, 0.6);
    let noise2 = perlin_noise_2d(
        width,
        height,
        scale * 0.4,
        seed.wrapping_add(777),
        3,
        2.0,
        0.5,
    );

    let mut pixels = vec![0u8; (width * height * 3) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let n = noise[idx];
            let n2 = noise2[idx];
            let hue = if n2 > 0.5 {
                hue_base + n * 0.2
            } else {
                hue_sec + n * 0.15
            };
            let sat_v = (sat * (0.5 + n * 0.5)).min(1.0);
            let lgt = (bright * (0.3 + n * 0.7)).min(1.0);
            let (r, g, b) = hsl_to_rgb(hue.fract(), sat_v, lgt);
            let pi = idx * 3;
            pixels[pi] = r;
            pixels[pi + 1] = g;
            pixels[pi + 2] = b;
        }
    }
    pixels
}

pub(crate) fn generate_combined(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let frac = generate_mandelbrot(width, height, seed);
    let geo = generate_geometric(width, height, seed.wrapping_add(333));
    let noise = generate_perlin_art(width, height, seed.wrapping_add(666));

    let mut pixels = vec![0u8; (width * height * 3) as usize];
    for i in 0..(width * height * 3) as usize {
        let v = frac[i] as f64 * 0.5 + geo[i] as f64 * 0.3 + noise[i] as f64 * 0.2;
        pixels[i] = v.min(255.0) as u8;
    }
    pixels
}

// ─── Shape Drawing Primitives ────────────────────────────────────────────────

fn blend_pixel(pixels: &mut [u8], idx: usize, r: u8, g: u8, b: u8, alpha: f64) {
    let a = alpha.min(1.0).max(0.0);
    let inv = 1.0 - a;
    pixels[idx] = (pixels[idx] as f64 * inv + r as f64 * a) as u8;
    pixels[idx + 1] = (pixels[idx + 1] as f64 * inv + g as f64 * a) as u8;
    pixels[idx + 2] = (pixels[idx + 2] as f64 * inv + b as f64 * a) as u8;
}

fn draw_circle(
    pixels: &mut [u8],
    w: u32,
    h: u32,
    cx: i32,
    cy: i32,
    r: i32,
    r_: u8,
    g: u8,
    b: u8,
    alpha: f64,
) {
    let r2 = r * r;
    let x0 = (cx - r).max(0);
    let x1 = (cx + r).min(w as i32 - 1);
    let y0 = (cy - r).max(0);
    let y1 = (cy + r).min(h as i32 - 1);
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px - cx;
            let dy = py - cy;
            if dx * dx + dy * dy <= r2 {
                let idx = (py as u32 * w + px as u32) as usize * 3;
                blend_pixel(pixels, idx, r_, g, b, alpha);
            }
        }
    }
}

fn draw_rect(
    pixels: &mut [u8],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    rw: u32,
    rh: u32,
    r_: u8,
    g: u8,
    b: u8,
    alpha: f64,
) {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + rw as i32 - 1).min(w as i32 - 1);
    let y1 = (y + rh as i32 - 1).min(h as i32 - 1);
    for py in y0..=y1 {
        for px in x0..=x1 {
            let idx = (py as u32 * w + px as u32) as usize * 3;
            blend_pixel(pixels, idx, r_, g, b, alpha);
        }
    }
}

fn draw_triangle(
    pixels: &mut [u8],
    w: u32,
    h: u32,
    pts: [(i32, i32); 3],
    r_: u8,
    g: u8,
    b: u8,
    alpha: f64,
) {
    let x0 = pts.iter().map(|p| p.0).min().unwrap_or(0).max(0);
    let x1 = pts.iter().map(|p| p.0).max().unwrap_or(0).min(w as i32 - 1);
    let y0 = pts.iter().map(|p| p.1).min().unwrap_or(0).max(0);
    let y1 = pts.iter().map(|p| p.1).max().unwrap_or(0).min(h as i32 - 1);

    for py in y0..=y1 {
        for px in x0..=x1 {
            if point_in_triangle(px, py, pts) {
                let idx = (py as u32 * w + px as u32) as usize * 3;
                blend_pixel(pixels, idx, r_, g, b, alpha);
            }
        }
    }
}

fn point_in_triangle(px: i32, py: i32, pts: [(i32, i32); 3]) -> bool {
    fn sign(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i64 {
        (a.0 as i64 - c.0 as i64) * (b.1 as i64 - c.1 as i64)
            - (a.1 as i64 - c.1 as i64) * (b.0 as i64 - c.0 as i64)
    }
    let d1 = sign((px, py), pts[0], pts[1]);
    let d2 = sign((px, py), pts[1], pts[2]);
    let d3 = sign((px, py), pts[2], pts[0]);
    let has_neg = d1 < 0 || d2 < 0 || d3 < 0;
    let has_pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(has_neg && has_pos)
}

fn draw_line(
    pixels: &mut [u8],
    w: u32,
    h: u32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    thickness: i32,
    r_: u8,
    g: u8,
    b: u8,
    alpha: f64,
) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x1, y1);
    let t2 = thickness / 2;

    loop {
        for ty in (y - t2).max(0)..=(y + t2).min(h as i32 - 1) {
            for tx in (x - t2).max(0)..=(x + t2).min(w as i32 - 1) {
                let dx2 = tx - x;
                let dy2 = ty - y;
                if dx2 * dx2 + dy2 * dy2 <= t2 * t2 {
                    let idx = (ty as u32 * w + tx as u32) as usize * 3;
                    blend_pixel(pixels, idx, r_, g, b, alpha);
                }
            }
        }

        if x == x2 && y == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn create_vsa_seed(seed: u64) -> Vec<u8> {
    let mut vsa = vec![0u8; 4096];
    let mut r = seed;
    for byte in vsa.iter_mut() {
        r = r.wrapping_mul(6364136223846793005).wrapping_add(1);
        *byte = (r >> 40) as u8;
    }
    vsa
}

// ─── Style Modes ─────────────────────────────────────────────────────────────

pub enum ImageStyle {
    Fractal,
    Geometric,
    Perlin,
    Combined,
}

pub(crate) fn parse_style(prompt: &str) -> ImageStyle {
    let lower = prompt.to_lowercase();
    if lower.contains("fractal") || lower.contains("mandelbrot") || lower.contains("julia") {
        ImageStyle::Fractal
    } else if lower.contains("geometric")
        || lower.contains("shape")
        || lower.contains("abstract shapes")
    {
        ImageStyle::Geometric
    } else if lower.contains("perlin") || lower.contains("noise") || lower.contains("texture") {
        ImageStyle::Perlin
    } else {
        ImageStyle::Combined
    }
}

// ─── Tool Implementation ─────────────────────────────────────────────────────

pub struct ImageGenTool {
    manifest: ToolManifest,
}

fn seed_from_prompt(prompt: &str) -> u64 {
    if prompt.is_empty() {
        return 42;
    }
    prompt
        .bytes()
        .fold(42u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

impl ImageGenTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "image_gen".into(),
                name: "Image Generator".into(),
                version: "0.2.0".into(),
                permissions: vec![],
                mcp: None,
                min_runtime: "0.1.0".into(),
                description: "Generate procedural images using VSA-driven fractal, geometric, and Perlin noise compositing. "
                    .to_string() + "Parameters: prompt, width (max 2048), height (max 2048), seed (optional). "
                    + "Style keywords: fractal, geometric, perlin, or leave empty for combined. "
                    + "Output is PNG (base64-encoded).",
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for ImageGenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for ImageGenTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        let _ = perlin_noise_2d(4, 4, 1.0, 0, 1, 2.0, 0.5);
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("abstract");
        let width = args
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(512)
            .min(2048)
            .max(32) as u32;
        let height = args
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(512)
            .min(2048)
            .max(32) as u32;
        let seed = args
            .get("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| seed_from_prompt(prompt));

        let style = parse_style(prompt);
        let pixels = match style {
            ImageStyle::Fractal => generate_mandelbrot(width, height, seed),
            ImageStyle::Geometric => generate_geometric(width, height, seed),
            ImageStyle::Perlin => generate_perlin_art(width, height, seed),
            ImageStyle::Combined => generate_combined(width, height, seed),
        };

        let png = png_encode(width, height, &pixels);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        let style_name = match style {
            ImageStyle::Fractal => "fractal",
            ImageStyle::Geometric => "geometric",
            ImageStyle::Perlin => "perlin",
            ImageStyle::Combined => "combined",
        };

        let result = serde_json::json!({
            "tool": "image_gen",
            "format": "png",
            "width": width,
            "height": height,
            "seed": seed,
            "style": style_name,
            "prompt": prompt,
            "size_bytes": png.len(),
            "data": b64,
        });
        let mut meta = HashMap::new();
        meta.insert("format".into(), "png".into());
        meta.insert("width".into(), width.to_string());
        meta.insert("height".into(), height.to_string());
        meta.insert("style".into(), style_name.into());
        meta.insert("prompt".into(), prompt.into());
        Ok(ToolOutput {
            result: serde_json::to_string_pretty(&result).unwrap_or_default(),
            metadata: meta,
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        Ok(())
    }
}
