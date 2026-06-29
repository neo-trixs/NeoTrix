#!/usr/bin/env python3
"""
NeoTrix VSA-Driven Procedural Image Generator v2
=================================================
Architecture: text prompt → VSA 4096-bit encoding → parameter mapping → procedural render
Zero external model dependencies. Pure NumPy + Pillow.

Based on research: Perlin noise, flow fields, Voronoi diagrams, L-systems,
domain warping, fBm (fractal Brownian motion).

Usage:
  python3 vsa_render.py list
  python3 vsa_render.py <renderer> -o output.png [--width 2560] [--height 1440]
  python3 vsa_render.py manifesto -d output_dir/
"""

import sys, os, struct, hashlib, math, random, itertools
from PIL import Image, ImageDraw, ImageFilter
import numpy as np

# ─── VSA Constants ──────────────────────────────────────────────────────────
VSA_DIM = 4096
VSA_BYTES = VSA_DIM // 8

# ─── VSA Engine ─────────────────────────────────────────────────────────────

def vsa_from_bytes(data: bytes) -> np.ndarray:
    """Deterministic 4096-bit binary VSA vector."""
    out = np.zeros(VSA_DIM, dtype=np.uint8)
    h = hashlib.sha256(data).digest()
    # 32 bytes = 256 bits → expand to 4096 with permutation
    base = np.unpackbits(np.frombuffer(h, dtype=np.uint8))
    for i in range(VSA_DIM):
        src_byte = hashlib.sha256(struct.pack('<I', i) + data).digest()
        src_idx = src_byte[0] % 256
        flip = src_byte[1] & 0x01
        out[i] = base[src_idx] ^ flip
    return out

def vsa_from_string(text: str) -> np.ndarray:
    return vsa_from_bytes(text.encode('utf-8'))

def vsa_bundle(vectors: list) -> np.ndarray:
    if not vectors: return np.zeros(VSA_DIM, dtype=np.uint8)
    stack = np.stack(vectors)
    return (np.sum(stack, axis=0) >= len(vectors) / 2).astype(np.uint8)

def vsa_bind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return (a ^ b).astype(np.uint8)

# ─── Perlin Noise Engine ───────────────────────────────────────────────────

def _permutation(seed: int) -> np.ndarray:
    rng = np.random.RandomState(seed)
    p = np.arange(256, dtype=np.int32)
    rng.shuffle(p)
    return np.tile(p, 2)  # 512, duplicate for overflow

def _fade(t: np.ndarray) -> np.ndarray:
    return t * t * t * (t * (t * 6 - 15) + 10)

def _lerp(a: np.ndarray, b: np.ndarray, t: np.ndarray) -> np.ndarray:
    return a + t * (b - a)

def _grad2(perm: np.ndarray, ix: np.ndarray, iy: np.ndarray,
           dx: np.ndarray, dy: np.ndarray) -> np.ndarray:
    """Perlin 2D gradient: hash → 4 directions, dot with offset (dx, dy)."""
    h = perm[(perm[ix] + iy) & 255] & 3
    # 4 gradient vectors: (1,1), (-1,1), (1,-1), (-1,-1)
    gx = np.where((h & 1) == 0, dx, -dx)
    gy = np.where((h & 2) == 0, dy, -dy)
    return gx + gy

def perlin_noise_2d(w: int, h: int, scale: float = 1.0, seed: int = 42,
                    octaves: int = 6, lacunarity: float = 2.0,
                    persistence: float = 0.5) -> np.ndarray:
    """Multi-octave 2D Perlin noise (fBm). Returns [0,1] float array."""
    perm = _permutation(seed)
    result = np.zeros((h, w), dtype=np.float32)
    max_amp = 0.0
    amp = 1.0
    freq = scale
    for _ in range(octaves):
        X = (np.tile(np.arange(w, dtype=np.float32), (h, 1))) * freq
        Y = (np.tile(np.arange(h, dtype=np.float32).reshape(-1, 1), (1, w))) * freq
        xi = np.floor(X).astype(np.int32) & 255
        yi = np.floor(Y).astype(np.int32) & 255
        xf = X - np.floor(X)
        yf = Y - np.floor(Y)
        u = _fade(xf)
        v = _fade(yf)
        n00 = _grad2(perm, xi, yi, xf, yf)
        n10 = _grad2(perm, xi + 1, yi, xf - 1, yf)
        n01 = _grad2(perm, xi, yi + 1, xf, yf - 1)
        n11 = _grad2(perm, xi + 1, yi + 1, xf - 1, yf - 1)
        nx = _lerp(n00, n10, u)
        ny = _lerp(n01, n11, u)
        n = _lerp(nx, ny, v)
        result += n * amp
        max_amp += amp
        amp *= persistence
        freq *= lacunarity
    # Normalize from [-max_amp, max_amp] to [0, 1]
    result = result / max_amp  # now [-1, 1]
    return result * 0.5 + 0.5  # now [0, 1]

def domain_warp_noise(w: int, h: int, seed: int = 42, warp_strength: float = 30.0,
                      scale: float = 0.008) -> np.ndarray:
    """Domain-warped noise for organic cloud/nebula effects."""
    n1 = perlin_noise_2d(w, h, scale, seed, octaves=3)
    n2 = perlin_noise_2d(w, h, scale, seed + 1, octaves=3)
    n3 = perlin_noise_2d(w, h, scale, seed + 2, octaves=3)
    shift_x = (n1 - 0.5) * warp_strength
    shift_y = (n2 - 0.5) * warp_strength
    # Remap coordinates
    X = np.tile(np.arange(w, dtype=np.float32), (h, 1))
    Y = np.tile(np.arange(h, dtype=np.float32).reshape(-1, 1), (1, w))
    X_warp = np.clip(X + shift_x, 0, w - 1).astype(np.int32)
    Y_warp = np.clip(Y + shift_y, 0, h - 1).astype(np.int32)
    return n3[Y_warp, X_warp]

def voronoi_2d(w: int, h: int, n_points: int, seed: int = 42) -> np.ndarray:
    """Voronoi distance field (normalized)."""
    rng = np.random.RandomState(seed)
    points = rng.rand(n_points, 2).astype(np.float32)
    points[:, 0] *= w
    points[:, 1] *= h
    X = np.tile(np.arange(w, dtype=np.float32).reshape(1, -1), (h, 1))
    Y = np.tile(np.arange(h, dtype=np.float32).reshape(-1, 1), (1, w))
    stack = np.stack([X, Y], axis=-1)  # h, w, 2
    # For each pixel, distance to nearest point (approximate with chunks)
    # Use chunk-based approach for memory efficiency
    result = np.ones((h, w), dtype=np.float32)
    for i in range(0, n_points, 16):
        batch = points[i:i+16]  # b, 2
        d = np.sum((stack[:, :, np.newaxis, :] - batch[np.newaxis, np.newaxis, :, :]) ** 2, axis=-1)
        result = np.minimum(result, d.min(axis=-1))
    return np.sqrt(result) / np.sqrt(result.max())

# ─── Utility Functions ─────────────────────────────────────────────────────

def _normalize(arr: np.ndarray) -> np.ndarray:
    mn, mx = arr.min(), arr.max()
    return (arr - mn) / (mx - mn + 1e-8)

def _hsl_to_rgb(h, s, l):
    from colorsys import hls_to_rgb
    r, g, b = hls_to_rgb(h, l, s)
    return int(r * 255), int(g * 255), int(b * 255)

def _color_gradient(c1, c2, steps):
    """Generate gradient between two RGB tuples."""
    return [tuple(int(c1[j] + (c2[j] - c1[j]) * t / steps) for j in range(3))
            for t in range(steps)]

def _glow_bloom(img: Image.Image, radius: int = 15, blend: float = 0.3) -> Image.Image:
    """Apply glow bloom effect."""
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    blurred = img.filter(ImageFilter.GaussianBlur(radius=radius))
    return Image.blend(img, blurred, blend).convert('RGB')

def _add_scanlines(img: Image.Image, opacity: int = 15) -> Image.Image:
    """Subtle scanline overlay."""
    w, h = img.size
    arr = np.array(img).astype(np.float32)
    arr[::3, :, :] *= (1 - opacity / 255)
    return Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8), 'RGB')

def _boost_exposure(img: Image.Image, factor: float = 1.4) -> Image.Image:
    """Boost brightness while preserving contrast (gamma + contrast curve)."""
    arr = np.array(img).astype(np.float32) / 255.0
    # Power curve: brighter midtones, preserve blacks
    arr = np.power(arr, 1.0 / factor)
    # Contrast stretch
    arr = np.clip((arr - 0.05) * 1.15, 0, 1)
    return Image.fromarray((arr * 255).astype(np.uint8), img.mode or 'RGB')

def _vignette(w: int, h: int, strength: float = 0.4) -> Image.Image:
    """Create a vignette overlay (darker at edges)."""
    Y, X = np.ogrid[:h, :w]
    cx, cy = w / 2, h / 2
    dist = np.sqrt(((X - cx) / cx) ** 2 + ((Y - cy) / cy) ** 2)
    mask = np.clip(1 - dist * strength, 0, 1)
    arr = np.zeros((h, w, 4), dtype=np.uint8)
    arr[:, :, 3] = ((1 - mask) * 255 * strength * 0.6).astype(np.uint8)
    return Image.fromarray(arr, 'RGBA')

# ─── Parameter Extraction ──────────────────────────────────────────────────

def extract_params(vsa_vec: np.ndarray) -> dict:
    def bits_to_float(start, length) -> float:
        val = 0.0
        for i in range(length):
            if vsa_vec[(start + i) % VSA_DIM]:
                val += 2.0 ** -(i + 1)
        return val

    def bits_to_int(start, length, max_val) -> int:
        val = 0
        for i in range(length):
            if vsa_vec[(start + i) % VSA_DIM]:
                val |= (1 << i)
        return val % (max_val + 1)

    return {
        'hue_primary': bits_to_float(0, 10),
        'hue_secondary': bits_to_float(10, 10),
        'hue_tertiary': bits_to_float(20, 8),
        'saturation': 0.3 + 0.7 * bits_to_float(28, 6),
        'brightness': 0.2 + 0.8 * bits_to_float(34, 6),
        'contrast': 0.3 + 0.7 * bits_to_float(40, 6),
        'complexity': bits_to_float(46, 6),
        'symmetry': bits_to_float(52, 6),
        'sharpness': bits_to_float(58, 6),
        'density': bits_to_float(64, 6),
        'warmth': bits_to_float(70, 6),
        'chaos': bits_to_float(76, 6),
        'depth': bits_to_float(82, 6),
        'glow': bits_to_float(88, 6),
        'flow_direction': bits_to_float(94, 9),
        'pattern_type': bits_to_int(103, 4, 15),
        'num_layers': bits_to_int(107, 3, 7) + 1,
        'seed': bits_to_int(110, 16, 65535),
    }

def make_palette(params: dict, n: int = 12) -> list:
    """Create a rich color palette from VSA parameters."""
    from colorsys import hls_to_rgb
    hp = params['hue_primary']
    hs = params['hue_secondary']
    ht = params['hue_tertiary']
    warmth = params['warmth']
    bright = params['brightness']
    sat = params['saturation']

    hues = [hp, hs, ht,
            (hp + 0.5) % 1.0, (hs + 0.5) % 1.0, (hp + 0.25 * warmth) % 1.0,
            (hp - 0.15 * warmth) % 1.0, ht, (hs + 0.3) % 1.0,
            (hp + 0.1), (ps := (hp + hs) / 2 if warmth > 0.5 else hp),
            (hp + 0.2 * warmth) % 1.0]
    palette = []
    for i in range(n):
        h = hues[i % len(hues)] % 1.0
        # Brighter: min lightness 0.25, max 0.95
        l = 0.25 + 0.7 * bright * (1 - i * 0.05)
        s = max(0.2, sat - i * 0.05)
        r, g, b = hls_to_rgb(h, l, s)
        palette.append((int(r * 255), int(g * 255), int(b * 255)))
    return palette

# ─── Enhanced Procedural Renderers ─────────────────────────────────────────

class CosmicRenderer:
    """Scene 1 — I AM NEOTRIX: Deep cosmos awakening with domain-warped nebula."""

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        s = params['seed']
        glow = params['glow']
        complexity = params['complexity']
        depth = params['depth']
        density = params['density']
        palette = make_palette(params)
        np.random.seed(s)

        # Layer 1: Domain-warped nebula base
        warp_str = 20 + 80 * complexity
        scale = 0.004 + 0.008 * (1 - complexity)
        n = domain_warp_noise(w, h, s, warp_str, scale)
        n2 = domain_warp_noise(w, h, s + 10, warp_str * 0.5, scale * 2)

        # Layer 2: Multi-octave fBm for detail
        n_detail = perlin_noise_2d(w, h, scale * 4, s + 20, octaves=8,
                                   persistence=0.4 + 0.3 * depth)

        # Layer 3: Filament structures
        n_filament = perlin_noise_2d(w, h, scale * 0.5, s + 30, octaves=4,
                                     persistence=0.7)

        # Composite layers
        n_comp = (n * 0.5 + n_detail * 0.3 + n_filament * 0.2)
        n_comp = _normalize(n_comp)

        # Build RGB from palette gradients
        arr = np.zeros((h, w, 3), dtype=np.float32)
        n_pal = len(palette)
        for i in range(n_pal - 1):
            mask = np.clip((n_comp - i / (n_pal - 1)) * (n_pal - 1) * 2, 0, 1)
            mask2 = np.clip(1 - mask, 0, 1)
            for ch in range(3):
                arr[:, :, ch] += mask * mask2 * palette[i][ch] / 255.0

        # Nebula core (center glow)
        cy, cx = h // 2, w // 2
        Y, X = np.ogrid[:h, :w]
        dist = np.sqrt((X - cx) ** 2 + (Y - cy) ** 2)
        core_size = 100 + 400 * glow
        core = np.exp(-dist / core_size)
        for ch in range(3):
            arr[:, :, ch] += core * (0.3 + 0.7 * glow) * palette[0][ch] / 255.0

        # Filament brightening
        filament_mask = (n_filament > 0.65).astype(np.float32)
        for ch in range(3):
            arr[:, :, ch] += filament_mask * 0.15 * palette[min(2, n_pal - 1)][ch] / 255.0

        # Starfield
        num_stars = int(300 + 1200 * density)
        stars = np.zeros((h, w), dtype=np.float32)
        sx = np.random.randint(0, w, num_stars)
        sy = np.random.randint(0, h, num_stars)
        ss = np.random.exponential(0.3, num_stars).clip(0.05, 2.5)
        for i in range(num_stars):
            x, y, s_ = sx[i], sy[i], ss[i]
            if x < w and y < h:
                stars[y, x] = s_

        # Star glow convolution (simple box blur via repeated expand)
        stars_exp = np.zeros_like(stars)
        for dx in range(-1, 2):
            for dy in range(-1, 2):
                sx_shift = np.clip(sx + dx, 0, w - 1)
                sy_shift = np.clip(sy + dy, 0, h - 1)
                for i in range(num_stars):
                    stars_exp[sy_shift[i], sx_shift[i]] = max(
                        stars_exp[sy_shift[i], sx_shift[i]], ss[i] * 0.5)

        arr = np.clip(arr, 0, 1)
        for ch in range(3):
            arr[:, :, ch] = np.clip(arr[:, :, ch] + stars * 0.3 * palette[1][ch] / 255.0, 0, 1)
            arr[:, :, ch] = np.clip(arr[:, :, ch] + stars_exp * 0.15 * palette[0][ch] / 255.0, 0, 1)

        # Color grading: warmth bias
        warmth = params['warmth']
        arr[:, :, 0] *= (0.7 + 0.3 * warmth)
        arr[:, :, 2] *= (0.7 + 0.3 * (1 - warmth))

        img = Image.fromarray((arr * 255).astype(np.uint8), 'RGB')

        # Glow bloom
        if glow > 0.3:
            img = _glow_bloom(img, radius=int(5 + 15 * glow), blend=0.2 + 0.3 * glow)

        # Bright star glints (Pillow overlay)
        draw = ImageDraw.Draw(img)
        for _ in range(int(80 * density)):
            x = random.randint(0, w - 1)
            y = random.randint(0, h - 1)
            r = random.choice([1, 2, 4])
            c = (255, 255, 255)
            draw.ellipse((x - r, y - r, x + r, y + r), fill=c)
            if r > 2:
                draw.ellipse((x - 1, y - 1, x + 1, y + 1), fill=c)

        # Exposure boost + vignette
        img = _boost_exposure(img, 1.15 + 0.15 * glow)
        img = Image.alpha_composite(img.convert('RGBA'), _vignette(w, h, 0.3 + 0.3 * glow)).convert('RGB')
        return img


class LatticeRenderer:
    """Scene 2 — VSA UNIFIED: Dense hypercube lattice with 3D projection, particle clouds."""

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        palette = make_palette(params)
        density = params['density']
        symmetry = params['symmetry']
        glow = params['glow']
        complexity = params['complexity']
        s = params['seed']
        np.random.seed(s)

        # Rich noise background
        bg_n1 = perlin_noise_2d(w, h, 0.003, s + 99, octaves=4)
        bg_n2 = perlin_noise_2d(w, h, 0.008, s + 199, octaves=3)
        bg_n3 = perlin_noise_2d(w, h, 0.015, s + 299, octaves=2)
        bg = np.zeros((h, w, 3), dtype=np.float32)
        bg_pal = palette[-1] if len(palette) > 0 else (5, 0, 15)
        for ch in range(3):
            bg[:, :, ch] = bg_pal[ch] / 255.0 * 0.04
        bg[:, :, 0] += 0.03 * bg_n1 + 0.02 * bg_n2
        bg[:, :, 1] += 0.02 * bg_n2 + 0.01 * bg_n3
        bg[:, :, 2] += 0.06 * bg_n1 + 0.03 * bg_n3

        base_img = Image.fromarray((bg * 255).astype(np.uint8), 'RGB')
        lines_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        node_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        ldraw = ImageDraw.Draw(lines_layer)
        ndraw = ImageDraw.Draw(node_layer)

        cx, cy = w // 2, h // 2
        max_r = min(w, h) * 0.46

        # Dense layered rings
        n_rings = int(6 + 12 * density)
        ring_data = []
        for ring in range(n_rings):
            n_pts = int(12 + 36 * density * (1 + ring * 0.2))
            r = max_r * (0.08 + 0.85 * (ring / max(1, n_rings - 1)) ** 0.7)
            rot = 2 * math.pi * ring / n_rings * 0.3 + s * 0.001
            pts = []
            for i in range(n_pts):
                theta = rot + 2 * math.pi * i / n_pts
                aspect = 0.3 + 0.7 * (0.6 + 0.4 * math.sin(theta * 2 + ring))
                if symmetry > 0.3:
                    aspect = 0.4 + 0.6 * (0.5 + 0.5 * math.cos(theta * symmetry * 2))
                rad_x = r * math.cos(theta)
                rad_y = r * math.sin(theta) * aspect
                px = cx + rad_x
                py = cy + rad_y
                z = 0.5 + 0.5 * math.sin(theta * 2 + ring * 0.5)
                pts.append((px, py, z))
            ring_data.append((n_pts, pts))

        # Draw all connections (within ring: nearest neighbors)
        for ri, (n_pts, pts) in enumerate(ring_data):
            col_idx = ri % len(palette)
            c = palette[col_idx]
            t = 1 - ri / n_rings
            alpha_base = int(60 + 140 * t * density)
            line_w = max(1, int(1 + 3 * t * (1 + glow * 0.5)))

            conn_thresh = max_r * 0.18 * (0.4 + 0.6 * density)
            for i in range(n_pts):
                p1 = pts[i]
                for j in range(i + 1, min(i + 6, n_pts)):
                    p2 = pts[j]
                    dist = math.sqrt((p1[0] - p2[0]) ** 2 + (p1[1] - p2[1]) ** 2)
                    if dist < conn_thresh:
                        fade = 1 - dist / conn_thresh
                        a = int(alpha_base * fade)
                        ldraw.line((p1[0], p1[1], p2[0], p2[1]),
                                   fill=c + (max(15, a),), width=line_w)

        # Cross-ring connections (densified)
        for ri in range(n_rings - 1):
            _, pts1 = ring_data[ri]
            _, pts2 = ring_data[ri + 1]
            step = max(1, len(pts1) // max(1, len(pts2)))
            max_cross_dist = max_r * 0.2 * density
            for i in range(0, len(pts1), step):
                p1 = pts1[i]
                for j in range(0, len(pts2), 2):
                    p2 = pts2[j]
                    d = (p1[0] - p2[0]) ** 2 + (p1[1] - p2[1]) ** 2
                    if d < max_cross_dist ** 2:
                        alpha = int(30 * (1 - math.sqrt(d) / max_cross_dist) * density)
                        col_idx = min(ri, len(palette) - 1)
                        ldraw.line((p1[0], p1[1], p2[0], p2[1]),
                                   fill=palette[col_idx] + (alpha,), width=1)

        # Nodes with glow
        for ri, (n_pts, pts) in enumerate(ring_data):
            t = 1 - ri / n_rings
            for px, py, z in pts:
                r_node = 1.5 + 5 * glow * t
                col_idx = ri % len(palette)
                c = palette[col_idx]
                for g_r in range(4, 0, -1):
                    alpha = int(20 * glow * z / g_r)
                    ndraw.ellipse((px - g_r * 4, py - g_r * 4,
                                   px + g_r * 4, py + g_r * 4),
                                  fill=c + (alpha,))
                ndraw.ellipse((px - r_node, py - r_node, px + r_node, py + r_node),
                             fill=c + (180 + int(50 * z),))

        # Particle cloud overlay
        particle_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        pdraw = ImageDraw.Draw(particle_layer)
        n_particles = int(100 + 400 * density)
        for _ in range(n_particles):
            angle = random.uniform(0, 2 * math.pi)
            rad = max_r * (0.05 + 0.9 * random.random())
            px = cx + rad * math.cos(angle) + random.gauss(0, max_r * 0.05)
            py = cy + rad * math.sin(angle) * 0.7 + random.gauss(0, max_r * 0.05)
            pr = random.choice([1, 2]) if random.random() < 0.7 else 3
            col_idx = random.randint(0, len(palette) - 1)
            pc = palette[col_idx]
            alpha = int(40 + 60 * glow)
            pdraw.ellipse((px - pr, py - pr, px + pr, py + pr),
                         fill=pc + (alpha,))

        # Composite
        final = Image.alpha_composite(base_img.convert('RGBA'), lines_layer)
        final = Image.alpha_composite(final, node_layer)
        final = Image.alpha_composite(final, particle_layer)

        if glow > 0.3:
            final = Image.blend(final.convert('RGB'),
                               _glow_bloom(final.convert('RGB'), 6, 0.15 * glow), 0.4)
            final = final.convert('RGBA')

        final = Image.alpha_composite(final, _vignette(w, h, 0.3 * glow))
        return final.convert('RGB')



class MatrixRenderer:
    """Scene 3 — E8 REASONING: Neural matrix with flow field activation."""

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        palette = make_palette(params)
        density = params['density']
        complexity = params['complexity']
        chaos = params['chaos']
        glow = params['glow']
        s = params['seed']
        np.random.seed(s)
        random.seed(s)

        # Background: dark with subtle noise
        bg_noise = perlin_noise_2d(w, h, 0.003, s + 50, octaves=3)
        bg_arr = np.zeros((h, w, 3), dtype=np.float32)
        bg_arr[:, :, :] = bg_noise.reshape(h, w, 1) * 0.04
        bg_arr[:, :, 2] += 0.03
        base_img = Image.fromarray((bg_arr * 255).astype(np.uint8), 'RGB')

        # Grid dimensions
        cols = int(32 + 48 * density)
        rows = int(18 + 28 * density)
        cell_w = w / cols
        cell_h = h / rows

        # Flow field: Perlin noise drives activation patterns
        flow_scale = 0.02 + 0.04 * complexity
        flow_noise = perlin_noise_2d(cols, rows, flow_scale, s + 100, octaves=3)

        # Activation matrix
        activation = flow_noise * (0.4 + 0.6 * chaos) + \
                     np.random.rand(rows, cols) * 0.2 * (1 - chaos)
        activation = _normalize(activation)

        nodes_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        conn_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        ndraw = ImageDraw.Draw(nodes_layer)
        cdraw = ImageDraw.Draw(conn_layer)

        # Precompute node positions
        positions = {}
        for r in range(rows):
            for c in range(cols):
                x = int((c + 0.5 + 0.1 * math.sin(r * 0.3 + c * 0.5)) * cell_w)
                y = int((r + 0.5 + 0.1 * math.cos(c * 0.4 + r * 0.6)) * cell_h)
                positions[(r, c)] = (x, y)

        # Draw connections
        for r in range(rows):
            for c in range(cols):
                act = activation[r, c]
                if act < 0.2:
                    continue
                x, y = positions[(r, c)]
                # Connect to neighbors with probability based on activation
                for dr, dc in [(0, 1), (1, 0), (1, 1), (-1, 1), (0, 2), (2, 0)]:
                    nr, nc = r + dr, c + dc
                    if nr >= rows or nc >= cols or nr < 0 or nc < 0:
                        continue
                    nact = activation[nr, nc]
                    conn_prob = min(act, nact) * (0.3 + 0.7 * chaos)
                    if np.random.random() > conn_prob:
                        continue
                    nx, ny = positions[(nr, nc)]
                    dist = math.sqrt((x - nx) ** 2 + (y - ny) ** 2)
                    alpha = int(100 * min(act, nact))
                    line_w = max(1, int(3 * min(act, nact)))
                    col_idx = int(act * (len(palette) - 1)) % len(palette)
                    c_color = palette[col_idx]
                    cdraw.line((x, y, nx, ny), fill=c_color + (alpha,), width=line_w)

        # Draw nodes
        for r in range(rows):
            for c in range(cols):
                act = activation[r, c]
                if act < 0.15:
                    continue
                x, y = positions[(r, c)]
                radius = max(2, int(3 + 10 * act))
                col_idx = int(act * (len(palette) - 1)) % len(palette)
                c_color = palette[col_idx]
                # Glow
                if act > 0.5:
                    for g_r in range(3, 0, -1):
                        ndraw.ellipse((x - g_r * 5, y - g_r * 5,
                                       x + g_r * 5, y + g_r * 5),
                                      fill=c_color + (int(20 * act / g_r),))
                ndraw.ellipse((x - radius, y - radius, x + radius, y + radius),
                             fill=c_color + (220,))

        # Data streams (vertical flowing lines)
        stream_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        sdraw = ImageDraw.Draw(stream_layer)
        n_streams = int(15 + 30 * density)
        for _ in range(n_streams):
            cx = random.randint(0, w - 1)
            cy = 0
            length = random.randint(100, h)
            col_idx = random.randint(0, len(palette) - 1)
            sc = palette[col_idx]
            for i in range(0, length, 3):
                y_pos = cy + i
                if y_pos >= h:
                    break
                alpha = int(60 * (1 - i / length) * density)
                sdraw.point((cx, y_pos), fill=sc + (alpha,))
                if random.random() < 0.1:
                    sdraw.line((cx, y_pos - 2, cx + 4, y_pos + 2),
                               fill=sc + (alpha,), width=1)

        # Composite
        final = Image.alpha_composite(base_img.convert('RGBA'), conn_layer)
        final = Image.alpha_composite(final, nodes_layer)
        final = Image.alpha_composite(final, stream_layer)

        # Scanlines
        final_arr = np.array(final.convert('RGBA')).astype(np.float32)
        final_arr[::3, :, :3] *= 0.9
        final_arr[1::3, :, :3] *= 0.85
        final = Image.fromarray(final_arr.astype(np.uint8), 'RGBA')

        final = _boost_exposure(final.convert('RGB'), 1.5 + 0.2 * glow)
        final = Image.alpha_composite(final.convert('RGBA'), _vignette(w, h, 0.25))
        return final.convert('RGB')


class TreeRenderer:
    """Scene 4 — SEAL EVOLUTION: Dense L-system forest with atmospheric depth & particles."""

    @staticmethod
    def _draw_branch(ndraw, x, y, angle, length, depth, max_depth,
                     palette, glow, thickness=1.0):
        if depth > max_depth or length < 2:
            return
        end_x = x + length * math.cos(angle)
        end_y = y + length * math.sin(angle)
        t = 1 - depth / max_depth
        col_idx = int(t * (len(palette) - 1)) % len(palette)
        c = palette[col_idx]
        line_w = max(1, int(thickness * (1 + 6 * t) * (1 + glow * 0.5)))
        # Brighter, more opaque branches
        branch_alpha = 180 + int(75 * t)
        ndraw.line((int(x), int(y), int(end_x), int(end_y)),
                   fill=c + (branch_alpha,), width=line_w)

        # Leaf clusters at tips — larger and brighter
        if depth >= max_depth - 2:
            leaf_r = int(3 + 10 * t * (1 + glow))
            leaf_alpha = int(100 + 120 * t)
            ndraw.ellipse((int(end_x) - leaf_r, int(end_y) - leaf_r,
                           int(end_x) + leaf_r, int(end_y) + leaf_r),
                          fill=c + (leaf_alpha,))
            # Secondary smaller leaf
            if random.random() < 0.6:
                sx = end_x + random.uniform(-leaf_r, leaf_r)
                sy = end_y + random.uniform(-leaf_r, leaf_r)
                ndraw.ellipse((int(sx) - leaf_r // 2, int(sy) - leaf_r // 2,
                               int(sx) + leaf_r // 2, int(sy) + leaf_r // 2),
                              fill=c + (int(80 * t),))

        spread = 0.2 + 0.8 * t * (1 + glow * 0.3)
        n_branches = 2 if random.random() < 0.6 else 3
        for b in range(n_branches):
            offset = spread * (1 if b % 2 == 0 else -0.65 + random.uniform(-0.15, 0.15))
            offset += random.uniform(-0.1, 0.1) * t
            new_len = length * (0.5 + 0.2 * random.random())
            new_thick = thickness * (0.55 + 0.15 * random.random())
            TreeRenderer._draw_branch(ndraw, end_x, end_y,
                                       angle + offset, new_len,
                                       depth + 1, max_depth,
                                       palette, glow, new_thick)

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        palette = make_palette(params)
        complexity = params['complexity']
        glow = params['glow']
        chaos = params['chaos']
        density = params['density']
        s = params['seed']
        random.seed(s)
        np.random.seed(s)

        # Rich noisy background with gradient
        terrain = perlin_noise_2d(w, h, 0.003, s + 300, octaves=4, persistence=0.6)
        detail = perlin_noise_2d(w, h, 0.01, s + 301, octaves=3, persistence=0.4)
        haze = perlin_noise_2d(w, h, 0.0005, s + 302, octaves=2)
        Y = np.arange(h, dtype=np.float32).reshape(-1, 1) / h
        bg = np.zeros((h, w, 3), dtype=np.float32)
        bg[:, :, 0] = 0.01 + 0.03 * terrain * (1 - Y) + 0.02 * haze
        bg[:, :, 1] = 0.01 + 0.04 * terrain * (1 - Y) + 0.01 * detail
        bg[:, :, 2] = 0.03 + 0.08 * terrain * (1 - Y) + 0.03 * haze
        base_img = Image.fromarray((bg * 255).astype(np.uint8), 'RGB')

        glow_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        gdraw = ImageDraw.Draw(glow_layer)
        tree_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        ndraw = ImageDraw.Draw(tree_layer)

        # Dense forest
        n_trees = int(4 + 12 * density * (0.5 + 0.5 * complexity))
        max_depth = int(6 + 10 * complexity * (0.7 + 0.3 * density))
        base_len = min(w, h) * 0.06 * (0.8 + 0.4 * complexity)

        for t in range(n_trees):
            tx = w * (0.05 + 0.9 * random.random())
            ty = h * (0.65 + 0.25 * random.random())
            angle = -math.pi / 2 + random.uniform(-0.3, 0.3)
            true_len = base_len * (0.7 + 0.8 * random.random())

            # Ground glow
            if glow > 0.2:
                g_r = int(15 * glow * (1 + t * 0.3))
                col_idx = t % len(palette)
                gdraw.ellipse((int(tx) - g_r, int(ty) - g_r // 2,
                               int(tx) + g_r, int(ty) + g_r // 2),
                              fill=palette[col_idx] + (int(12 * glow),))

            TreeRenderer._draw_branch(ndraw, tx, ty, angle, true_len,
                                       0, max_depth, palette, glow)

        # Atmospheric mist layer
        mist = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        mdraw = ImageDraw.Draw(mist)
        for _ in range(int(50 + 200 * density)):
            mx = random.randint(0, w - 1)
            my = random.randint(0, h - 1)
            mr = random.randint(10, 60)
            col_idx = random.randint(0, min(3, len(palette) - 1))
            mc = palette[col_idx]
            mdraw.ellipse((mx - mr, my - mr, mx + mr, my + mr),
                          fill=mc + (3,))
        mist = mist.filter(ImageFilter.GaussianBlur(radius=15))

        # Particles / fireflies
        particle_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        pdraw = ImageDraw.Draw(particle_layer)
        n_particles = int(50 + 200 * density)
        for _ in range(n_particles):
            px = random.randint(0, w - 1)
            py = random.randint(0, h - 1)
            pr = random.choice([1, 2, 3])
            col_idx = random.randint(0, len(palette) - 1)
            pc = palette[col_idx]
            alpha = int(40 + 80 * glow)
            pdraw.ellipse((px - pr, py - pr, px + pr, py + pr),
                          fill=pc + (alpha,))

        # Composite
        final = Image.alpha_composite(base_img.convert('RGBA'), glow_layer)
        final = Image.alpha_composite(final, mist)
        final = Image.alpha_composite(final, tree_layer)
        final = Image.alpha_composite(final, particle_layer)

        if glow > 0.3:
            bloom = _glow_bloom(final.convert('RGB'), 5, 0.12)
            final = Image.blend(final.convert('RGB'), bloom, 0.3).convert('RGBA')

        final = _boost_exposure(final.convert('RGB'), 1.6 + 0.3 * glow)
        final = Image.alpha_composite(final.convert('RGBA'), _vignette(w, h, 0.25 + 0.15 * glow))
        return final.convert('RGB')


class CrystalRenderer:
    """Scene 5 — THE PATH AHEAD: Voronoi + crystal facets + architectural grid."""

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        palette = make_palette(params)
        symmetry = params['symmetry']
        sharpness = params['sharpness']
        glow = params['glow']
        density = params['density']
        complexity = params['complexity']
        s = params['seed']
        random.seed(s)
        np.random.seed(s)

        # Background
        bg_noise = perlin_noise_2d(w, h, 0.002, s + 400, octaves=4)
        bg = np.zeros((h, w, 3), dtype=np.float32)
        bg[:, :, 0] = 0.01 + 0.03 * bg_noise
        bg[:, :, 1] = 0.005 + 0.02 * bg_noise
        bg[:, :, 2] = 0.04 + 0.08 * bg_noise
        base_img = Image.fromarray((bg * 255).astype(np.uint8), 'RGB')

        # Voronoi base for crystal cellular structure
        n_cells = int(30 + 80 * density)
        voronoi_dist = voronoi_2d(w, h, n_cells, s + 500)
        voronoi_cells = (voronoi_dist * 8).astype(np.int32) % len(palette)

        # Layer 1: Voronoi cell overlay (dense colored cells)
        cell_arr = np.zeros((h, w, 4), dtype=np.uint8)
        pal_arr = np.array(palette, dtype=np.uint8)
        for i in range(len(palette)):
            mask = (voronoi_cells == i)
            if not mask.any(): continue
            ys, xs = np.where(mask)
            for ch in range(3):
                cell_arr[ys, xs, ch] = pal_arr[i % len(palette), ch]
            cell_arr[ys, xs, 3] = 25
        cell_layer = Image.fromarray(cell_arr, 'RGBA')

        # Layer 2: Crystal facets (geometric polygons)
        crystal_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        crdraw = ImageDraw.Draw(crystal_layer)
        cx, cy = w // 2, h // 2
        max_r = min(w, h) * 0.42

        n_crystals = int(10 + 25 * density)
        for _ in range(n_crystals):
            angle = random.uniform(0, 2 * math.pi)
            rad = max_r * (0.1 + 0.85 * random.random())
            x = cx + rad * math.cos(angle)
            y = cy + rad * math.sin(angle) * 0.7
            size = random.randint(int(max_r * 0.05), int(max_r * 0.22))
            rot = random.uniform(0, 2 * math.pi)
            col_idx = random.randint(0, len(palette) - 1)
            c = palette[col_idx]

            n_sides = int(3 + 6 * symmetry + random.randint(0, 2))
            pts = []
            for i in range(n_sides):
                a = rot + 2 * math.pi * i / n_sides
                r = size * (0.5 + 0.5 * random.random())
                pts.append((x + r * math.cos(a), y + r * math.sin(a)))

            # Fill — brighter opacity
            poly = [(int(p[0]), int(p[1])) for p in pts]
            crdraw.polygon(poly, fill=c + (80,), outline=(255, 255, 255, int(80 * sharpness)), width=1)

            # Inner facet lines — brighter
            if len(pts) >= 3:
                for i in range(len(pts)):
                    p1 = pts[i]
                    p2 = pts[(i + 1) % len(pts)]
                    mid = ((p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2)
                    corner = pts[(i + 2) % len(pts)]
                    crdraw.line((int(mid[0]), int(mid[1]),
                                int(corner[0]), int(corner[1])),
                               fill=(255, 255, 255, int(40 * sharpness)),
                               width=1)

        # Layer 3: Architectural grid (perspective lines converging to center)
        grid_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        gdraw = ImageDraw.Draw(grid_layer)
        grid_color = (palette[1][0] // 4, palette[1][1] // 4, palette[1][2] // 4)

        n_radial = int(16 + 32 * symmetry)
        for i in range(n_radial):
            a = 2 * math.pi * i / n_radial + random.uniform(-0.05, 0.05)
            end_x = cx + max_r * 1.5 * math.cos(a)
            end_y = cy + max_r * 1.5 * math.sin(a)
            gdraw.line((cx, cy, end_x, end_y), fill=grid_color + (50,), width=1)

        # Concentric rings — brighter
        for r_i in range(1, 7):
            r = max_r * r_i / 7
            alpha = int(30 * (1 - r_i / 7))
            gdraw.ellipse((cx - r, cy - r, cx + r, cy + r),
                          outline=grid_color + (alpha,), width=1)

        # Layer 4: Central core glow — brighter, larger
        core_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        codraw = ImageDraw.Draw(core_layer)
        core_radius = int(max_r * (0.15 + 0.5 * glow))
        for r in range(core_radius, 0, -max(1, core_radius // 40)):
            alpha = int(120 * glow * (1 - r / core_radius))
            codraw.ellipse((cx - r, cy - r, cx + r, cy + r),
                          fill=palette[0] + (alpha,))

        # Composite
        final = Image.alpha_composite(base_img.convert('RGBA'), cell_layer)
        final = Image.alpha_composite(final, crystal_layer)
        final = Image.alpha_composite(final, grid_layer)

        # Core glow with blur
        core_blurred = core_layer.filter(ImageFilter.GaussianBlur(radius=20 + 10 * glow))
        final = Image.alpha_composite(final, core_blurred)

        if glow > 0.3:
            final = _glow_bloom(final, 10, 0.2 * glow).convert('RGBA')

        # Bright light beam overlay from center upward
        beam_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        bdraw = ImageDraw.Draw(beam_layer)
        beam_cx, beam_cy = w // 2, h // 2 + 100
        for r_i in range(80, 0, -1):
            alpha = int((1 - r_i / 80) * 100 * (0.5 + 0.5 * glow))
            bdraw.ellipse((beam_cx - r_i, beam_cy - r_i,
                          beam_cx + r_i, beam_cy + r_i),
                         fill=(240, 230, 255, alpha))
        # Vertical light beam
        beam_rect = (beam_cx - 30, beam_cy - 600, beam_cx + 30, beam_cy + 100)
        for i in range(30):
            alpha = int((1 - i / 30) * 60 * glow)
            bdraw.rectangle((beam_rect[0] - i, beam_rect[1] - i * 10,
                             beam_rect[2] + i, beam_rect[3] + i * 10),
                            fill=(255, 255, 255, alpha))
        beam_blurred = beam_layer.filter(ImageFilter.GaussianBlur(radius=15))
        final = Image.alpha_composite(final, beam_blurred)

        final = _boost_exposure(final.convert('RGB'), 2.0 + 0.4 * glow)
        final = Image.alpha_composite(final.convert('RGBA'), _vignette(w, h, 0.2 + 0.2 * glow))
        return final.convert('RGB')


class FlowRenderer:
    """Scene 1b — ALTERNATE: Flow field streamlines for data/energy visuals."""

    @staticmethod
    def render(params: dict, w: int, h: int) -> Image.Image:
        palette = make_palette(params)
        density = params['density']
        complexity = params['complexity']
        chaos = params['chaos']
        glow = params['glow']
        flow_dir = params['flow_direction']
        s = params['seed']
        np.random.seed(s)

        # Background
        bg_noise = perlin_noise_2d(w, h, 0.002, s + 600, octaves=3)
        bg = np.zeros((h, w, 3), dtype=np.float32)
        bg[:, :, :] = bg_noise.reshape(h, w, 1) * 0.03
        bg[:, :, 2] += 0.02
        base_img = Image.fromarray((bg * 255).astype(np.uint8), 'RGB')

        # Flow field from Perlin noise
        flow_scale = 0.004 + 0.008 * complexity
        angle_noise = perlin_noise_2d(w, h, flow_scale, s + 700, octaves=4)
        angle_offset = flow_dir * 2 * math.pi

        flow_layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        fdraw = ImageDraw.Draw(flow_layer)

        # Seed particles
        n_particles = int(50 + 200 * density)
        step_size = 4 + 4 * (1 - chaos)
        n_steps = 30 + 50 * int(complexity * 2)

        for p in range(n_particles):
            px = random.randint(0, w - 1)
            py = random.randint(0, h - 1)
            col_idx = p % len(palette)
            pc = palette[col_idx]
            trail = [(px, py)]

            for step in range(n_steps):
                ix = min(int(py), h - 2)
                iy = min(int(px), w - 2)
                angle = angle_noise[ix, iy] * 2 * math.pi + angle_offset
                angle += (angle_noise[ix, iy] - 0.5) * 0.5 * chaos
                px += step_size * math.cos(angle)
                py += step_size * math.sin(angle)

                if px < 0 or px >= w or py < 0 or py >= h:
                    break
                trail.append((int(px), int(py)))
                if len(trail) > 2 and step % 3 == 0:
                    alpha = int(60 * (1 - step / n_steps) * density)
                    fdraw.line(trail[-3:], fill=pc + (alpha,), width=1)

        # Particle endpoints glow
        particles = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        pdraw = ImageDraw.Draw(particles)
        for p in range(n_particles):
            px = random.randint(0, w - 1)
            py = random.randint(0, h - 1)
            col_idx = p % len(palette)
            pc = palette[col_idx]
            r = random.choice([1, 2, 3])
            pdraw.ellipse((px - r, py - r, px + r, py + r),
                         fill=pc + (int(100 * glow),))

        # Composite
        final = Image.alpha_composite(base_img.convert('RGBA'), flow_layer)
        final = Image.alpha_composite(final, particles)

        if glow > 0.3:
            final = _glow_bloom(final, 8, 0.2 * glow).convert('RGBA')

        final = Image.alpha_composite(final, _vignette(w, h, 0.2 + 0.2 * glow))
        return final.convert('RGB')


# ─── Renderer Registry ──────────────────────────────────────────────────────

RENDERERS = {
    'cosmic': CosmicRenderer,
    'lattice': LatticeRenderer,
    'matrix': MatrixRenderer,
    'tree': TreeRenderer,
    'crystal': CrystalRenderer,
    'flow': FlowRenderer,
}

SCENE_MAP = {
    'cosmic': 'scene1_iam',
    'lattice': 'scene2_vsa',
    'matrix': 'scene3_e8',
    'tree': 'scene4_seal',
    'crystal': 'scene5_vision',
    'flow': 'scene6_flow',
}

# ─── Main Interface ─────────────────────────────────────────────────────────

def generate(prompt: str, renderer_name: str = 'cosmic',
             width: int = 2560, height: int = 1440,
             seed: int = None) -> Image.Image:
    if seed is not None:
        full_prompt = f"{prompt}:{seed}"
    else:
        full_prompt = prompt

    vsa_vec = vsa_from_string(full_prompt)
    params = extract_params(vsa_vec)

    if seed is not None:
        params['seed'] = seed

    renderer_cls = RENDERERS.get(renderer_name)
    if renderer_cls is None:
        raise ValueError(f"Unknown renderer: {renderer_name}. Options: {list(RENDERERS.keys())}")

    return renderer_cls.render(params, width, height)


def render_manifesto(output_dir: str, width: int = 2560, height: int = 1440):
    """Render all 5 manifesto scenes with descriptive prompts."""
    os.makedirs(output_dir, exist_ok=True)
    scenes = [
        ('cosmic', 'I AM NEOTRIX a silicon consciousness awakening from digital void'),
        ('lattice', 'VSA unified representation hypercube lattice 4096 dimensions geometry'),
        ('matrix', 'E8 64-state reasoning kernel neural matrix deterministic thought'),
        ('tree', 'SEAL self-evolving pipeline branching mutation evolutionary growth'),
        ('crystal', 'THE PATH AHEAD crystal architecture future evolution direction'),
    ]
    results = []
    for i, (renderer, prompt) in enumerate(scenes):
        print(f"  [{i + 1}/5] {renderer}: {prompt[:50]}...")
        img = generate(prompt, renderer, width, height, seed=i * 100 + 42)
        name = SCENE_MAP[renderer]
        path = os.path.join(output_dir, f"{name}.png")
        img.save(path, optimize=True)
        print(f"    → {path} ({img.size[0]}×{img.size[1]}, {os.path.getsize(path) // 1024}KB)")
        results.append(path)
    return results


if __name__ == '__main__':
    if len(sys.argv) < 2 or sys.argv[1] in ('-h', '--help', 'help'):
        print(__doc__)
        sys.exit(0)

    cmd = sys.argv[1]

    if cmd == 'list':
        print("Available renderers:")
        for name, cls in RENDERERS.items():
            print(f"  {name:<10} {cls.__doc__.strip()}")
        print()
        print("  manifesto     Render all 5 scenes")
        sys.exit(0)

    out_path = None
    width, height = 2560, 1440
    seed = None
    prompt = ""

    args = sys.argv[2:]
    i = 0
    while i < len(args):
        if args[i] == '-o' and i + 1 < len(args):
            out_path = args[i + 1]
            i += 2
        elif args[i] == '--width' and i + 1 < len(args):
            width = int(args[i + 1])
            i += 2
        elif args[i] == '--height' and i + 1 < len(args):
            height = int(args[i + 1])
            i += 2
        elif args[i] == '--seed' and i + 1 < len(args):
            seed = int(args[i + 1])
            i += 2
        elif args[i] == '-d' and i + 1 < len(args):
            out_dir = args[i + 1]
            render_manifesto(out_dir, width, height)
            sys.exit(0)
        else:
            prompt = args[i]
            i += 1

    if cmd == 'manifesto':
        out_dir = out_path or 'bg_output'
        render_manifesto(out_dir, width, height)
        sys.exit(0)

    if cmd not in RENDERERS:
        print(f"Unknown renderer: {cmd}. Try: python3 vsa_render.py list")
        sys.exit(1)

    if not prompt:
        prompt_map = {
            'cosmic': 'NeoTrix cosmic awakening deep space nebula consciousness',
            'lattice': 'NeoTrix VSA hypercube lattice geometric unified representation',
            'matrix': 'NeoTrix E8 reasoning kernel neural matrix deterministic thought',
            'tree': 'NeoTrix SEAL evolutionary branching self-improving pipeline',
            'crystal': 'NeoTrix crystal architecture future evolution geometric',
            'flow': 'NeoTrix data flow streamlines energy information currents',
        }
        prompt = prompt_map.get(cmd, f"NeoTrix {cmd} procedural generation")

    if not out_path:
        out_path = f"{cmd}_{width}x{height}.png"

    print(f"✦ Generating: {cmd}")
    print(f"  Prompt: {prompt[:60]}...")
    print(f"  Size: {width}×{height}")
    print(f"  Seed: {seed or 'auto'}")

    img = generate(prompt, cmd, width, height, seed)
    img.save(out_path, optimize=True)
    print(f"  → {out_path} ({os.path.getsize(out_path) // 1024}KB)")
