use std::collections::HashMap;

// ─── Minimal Deterministic PRNG ─────────────────────────────────────────────

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    fn shuffle(&mut self, arr: &mut [i32]) {
        for i in (1..arr.len()).rev() {
            let j = (self.next_u32() as usize) % (i + 1);
            arr.swap(i, j);
        }
    }
}

// ─── Permutation Table ──────────────────────────────────────────────────────

fn _permutation(seed: u64) -> [i32; 512] {
    let mut p = [0i32; 256];
    for i in 0..256 {
        p[i] = i as i32;
    }
    let mut rng = SimpleRng::new(seed);
    rng.shuffle(&mut p);
    let mut perm = [0i32; 512];
    for i in 0..512 {
        perm[i] = p[i & 255];
    }
    perm
}

// ─── Perlin Noise Primitives ────────────────────────────────────────────────

fn _fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn _lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn _grad2(perm: &[i32; 512], ix: i32, iy: i32, dx: f64, dy: f64) -> f64 {
    let h = perm[((perm[(ix as usize) & 255] + iy) & 255) as usize] & 3;
    let gx = if (h & 1) == 0 { dx } else { -dx };
    let gy = if (h & 2) == 0 { dy } else { -dy };
    gx + gy
}

fn _perlin_2d_point(perm: &[i32; 512], x: f64, y: f64) -> f64 {
    let ix = (x.floor() as i32) & 255;
    let iy = (y.floor() as i32) & 255;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let u = _fade(xf);
    let v = _fade(yf);

    let n00 = _grad2(perm, ix, iy, xf, yf);
    let n10 = _grad2(perm, ix + 1, iy, xf - 1.0, yf);
    let n01 = _grad2(perm, ix, iy + 1, xf, yf - 1.0);
    let n11 = _grad2(perm, ix + 1, iy + 1, xf - 1.0, yf - 1.0);

    let nx = _lerp(n00, n10, u);
    let ny = _lerp(n01, n11, u);
    _lerp(nx, ny, v)
}

// ─── Public API ─────────────────────────────────────────────────────────────

pub fn perlin_noise_2d(
    w: u32,
    h: u32,
    scale: f64,
    seed: u64,
    octaves: usize,
    lacunarity: f64,
    persistence: f64,
) -> Vec<f64> {
    let perm = _permutation(seed);
    let size = (w * h) as usize;
    let mut result = vec![0.0_f64; size];
    let mut max_amp = 0.0_f64;
    let mut amp = 1.0_f64;
    let mut freq = scale;

    for _ in 0..octaves {
        for y in 0..h {
            for x in 0..w {
                let px = x as f64 * freq;
                let py = y as f64 * freq;
                let n = _perlin_2d_point(&perm, px, py);
                result[(y * w + x) as usize] += n * amp;
            }
        }
        max_amp += amp;
        amp *= persistence;
        freq *= lacunarity;
    }

    if max_amp > 0.0 {
        for v in result.iter_mut() {
            *v = *v / max_amp * 0.5 + 0.5;
        }
    }

    result
}

pub fn domain_warp_noise(w: u32, h: u32, seed: u64, warp_strength: f64, scale: f64) -> Vec<f64> {
    let n1 = perlin_noise_2d(w, h, scale, seed, 3, 2.0, 0.5);
    let n2 = perlin_noise_2d(w, h, scale, seed + 1, 3, 2.0, 0.5);
    let n3 = perlin_noise_2d(w, h, scale, seed + 2, 3, 2.0, 0.5);

    let size = (w * h) as usize;
    let mut result = vec![0.0_f64; size];

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let shift_x = (n1[idx] - 0.5) * warp_strength;
            let shift_y = (n2[idx] - 0.5) * warp_strength;

            let sx = (x as f64 + shift_x).clamp(0.0, (w - 1) as f64) as u32;
            let sy = (y as f64 + shift_y).clamp(0.0, (h - 1) as f64) as u32;

            result[idx] = n3[(sy * w + sx) as usize];
        }
    }

    result
}

// ─── VSA Parameter Extraction ───────────────────────────────────────────────

fn _bits_to_float(vsa_vec: &[u8], start: usize, len: usize) -> f64 {
    let mut val = 0.0;
    for i in 0..len {
        if vsa_vec[(start + i) % 4096] != 0 {
            val += 2.0_f64.powi(-(i as i32 + 1));
        }
    }
    val
}

fn _bits_to_int(vsa_vec: &[u8], start: usize, len: usize, max_val: u32) -> u32 {
    let mut val = 0u32;
    for i in 0..len {
        if vsa_vec[(start + i) % 4096] != 0 {
            val |= 1 << i;
        }
    }
    val % (max_val + 1)
}

pub fn extract_vsa_params(vsa_vec: &[u8]) -> Vec<(String, f64)> {
    vec![
        ("hue_primary".into(), _bits_to_float(vsa_vec, 0, 10)),
        ("hue_secondary".into(), _bits_to_float(vsa_vec, 10, 10)),
        ("hue_tertiary".into(), _bits_to_float(vsa_vec, 20, 8)),
        (
            "saturation".into(),
            0.3 + 0.7 * _bits_to_float(vsa_vec, 28, 6),
        ),
        (
            "brightness".into(),
            0.2 + 0.8 * _bits_to_float(vsa_vec, 34, 6),
        ),
        (
            "contrast".into(),
            0.3 + 0.7 * _bits_to_float(vsa_vec, 40, 6),
        ),
        ("complexity".into(), _bits_to_float(vsa_vec, 46, 6)),
        ("symmetry".into(), _bits_to_float(vsa_vec, 52, 6)),
        ("sharpness".into(), _bits_to_float(vsa_vec, 58, 6)),
        ("density".into(), _bits_to_float(vsa_vec, 64, 6)),
        ("warmth".into(), _bits_to_float(vsa_vec, 70, 6)),
        ("chaos".into(), _bits_to_float(vsa_vec, 76, 6)),
        ("depth".into(), _bits_to_float(vsa_vec, 82, 6)),
        ("glow".into(), _bits_to_float(vsa_vec, 88, 6)),
        ("flow_direction".into(), _bits_to_float(vsa_vec, 94, 9)),
        (
            "pattern_type".into(),
            _bits_to_int(vsa_vec, 103, 4, 15) as f64,
        ),
        (
            "num_layers".into(),
            (_bits_to_int(vsa_vec, 107, 3, 7) + 1) as f64,
        ),
        ("seed".into(), _bits_to_int(vsa_vec, 110, 16, 65535) as f64),
    ]
}

// ─── HSL / RGB ──────────────────────────────────────────────────────────────

fn _hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = _hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = _hue_to_rgb(p, q, h);
    let b = _hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

// ─── Palette ────────────────────────────────────────────────────────────────

pub fn make_palette(params: &[(String, f64)], n: usize) -> Vec<(u8, u8, u8)> {
    let map: HashMap<&str, f64> = params.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let hp = *map.get("hue_primary").unwrap_or(&0.0);
    let hs = *map.get("hue_secondary").unwrap_or(&0.0);
    let ht = *map.get("hue_tertiary").unwrap_or(&0.0);
    let warmth = *map.get("warmth").unwrap_or(&0.5);
    let bright = *map.get("brightness").unwrap_or(&0.5);
    let sat = *map.get("saturation").unwrap_or(&0.5);

    let hues = [
        hp,
        hs,
        ht,
        (hp + 0.5) % 1.0,
        (hs + 0.5) % 1.0,
        (hp + 0.25 * warmth) % 1.0,
        (hp - 0.15 * warmth) % 1.0,
        ht,
        (hs + 0.3) % 1.0,
        hp + 0.1,
        if warmth > 0.5 { (hp + hs) / 2.0 } else { hp },
        (hp + 0.2 * warmth) % 1.0,
    ];

    let mut palette = Vec::with_capacity(n);
    for i in 0..n {
        let h = hues[i % hues.len()] % 1.0;
        let l = 0.25 + 0.7 * bright * (1.0 - i as f64 * 0.05);
        let s = (0.2_f64).max(sat - i as f64 * 0.05);
        palette.push(hsl_to_rgb(h, s, l));
    }
    palette
}

// ─── Utility ────────────────────────────────────────────────────────────────

pub fn normalize(arr: &[f64]) -> Vec<f64> {
    let mn = arr.iter().copied().fold(f64::INFINITY, f64::min);
    let mx = arr.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = mx - mn;
    if range < 1e-10 {
        return vec![0.5; arr.len()];
    }
    arr.iter().map(|&v| (v - mn) / range).collect()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_range() {
        let noise = perlin_noise_2d(16, 16, 1.0, 42, 1, 2.0, 0.5);
        assert_eq!(noise.len(), 256);
        for &v in &noise {
            assert!(v >= 0.0 && v <= 1.0, "Perlin value {} out of [0, 1]", v);
        }
    }

    #[test]
    fn test_perlin_deterministic() {
        let a = perlin_noise_2d(32, 32, 0.5, 12345, 2, 2.0, 0.5);
        let b = perlin_noise_2d(32, 32, 0.5, 12345, 2, 2.0, 0.5);
        assert_eq!(a, b);
    }

    #[test]
    fn test_extract_params_keys() {
        let vsa = vec![1u8; 4096];
        let params = extract_vsa_params(&vsa);
        let keys: Vec<&str> = params.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(params.len(), 18);
        assert!(keys.contains(&"hue_primary"));
        assert!(keys.contains(&"hue_secondary"));
        assert!(keys.contains(&"hue_tertiary"));
        assert!(keys.contains(&"saturation"));
        assert!(keys.contains(&"brightness"));
        assert!(keys.contains(&"contrast"));
        assert!(keys.contains(&"complexity"));
        assert!(keys.contains(&"symmetry"));
        assert!(keys.contains(&"sharpness"));
        assert!(keys.contains(&"density"));
        assert!(keys.contains(&"warmth"));
        assert!(keys.contains(&"chaos"));
        assert!(keys.contains(&"depth"));
        assert!(keys.contains(&"glow"));
        assert!(keys.contains(&"flow_direction"));
        assert!(keys.contains(&"pattern_type"));
        assert!(keys.contains(&"num_layers"));
        assert!(keys.contains(&"seed"));
    }

    #[test]
    fn test_palette_length() {
        let params = vec![
            ("hue_primary".to_string(), 0.6),
            ("hue_secondary".to_string(), 0.3),
            ("hue_tertiary".to_string(), 0.1),
            ("warmth".to_string(), 0.7),
            ("brightness".to_string(), 0.8),
            ("saturation".to_string(), 0.6),
        ];
        let pal = make_palette(&params, 12);
        assert_eq!(pal.len(), 12);
        for &(r, g, b) in &pal {
            let _ = (r, g, b);
        }
    }
}
