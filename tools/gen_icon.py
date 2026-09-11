#!/usr/bin/env python3
"""Neotrix Icon Generator — 橙子黄色，水滴散射，Mac圆角"""

import math
from PIL import Image, ImageDraw, ImageFilter

def squircle_mask(size, radius_ratio=0.22):
    img = Image.new('L', (size, size), 0)
    draw = ImageDraw.Draw(img)
    r = size * radius_ratio
    draw.rounded_rectangle([0, 0, size-1, size-1], radius=int(r), fill=255)
    return img

def create_neotrix_icon(size=1024, variant='main'):
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    s = size
    cx, cy = s // 2, s // 2

    # ═══ BACKGROUND: 橙子黄色径向渐变 ═══
    for y in range(s):
        for x in range(s):
            dist = math.sqrt((x - cx)**2 + (y - cy)**2) / (s * 0.5)
            dist = min(dist, 1.0)
            # 橙子黄色: 中心亮黄橙 → 边缘深橙
            r = int(255 - dist * 30)       # 255→225
            g = int(180 - dist * 80)       # 180→100
            b = int(30 + dist * 10)        # 30→40
            # variant adjustments
            if variant == 'sim':
                r = int(250 - dist * 25)
                g = int(160 - dist * 70)
                b = int(25 + dist * 15)
            elif variant == 'core':
                r = int(255 - dist * 20)
                g = int(195 - dist * 70)
                b = int(40 + dist * 20)
            elif variant == 'mind':
                r = int(240 - dist * 30)
                g = int(150 - dist * 60)
                b = int(60 + dist * 25)
            elif variant == 'memory':
                r = int(245 - dist * 25)
                g = int(170 - dist * 65)
                b = int(50 + dist * 20)
            elif variant == 'shield':
                r = int(250 - dist * 20)
                g = int(140 - dist * 60)
                b = int(25 + dist * 15)
            img.putpixel((x, y), (r, g, b, 255))

    # ═══ GLOW: 中心亮斑 ═══
    glow = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    glow_r = int(s * 0.38)
    for i in range(glow_r, 0, -1):
        alpha = int(70 * (1 - i / glow_r))
        gd.ellipse([cx - i, cy - i, cx + i, cy + i], fill=(255, 230, 120, alpha))
    img = Image.alpha_composite(img, glow)

    # ═══ LIGHT RAYS: 8条放射线 ═══
    rays = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    rd = ImageDraw.Draw(rays)
    for i in range(8):
        angle = 2 * math.pi * i / 8 + 0.2
        inner_r = int(s * 0.16)
        outer_r = int(s * 0.44)
        x1 = cx + int(inner_r * math.cos(angle))
        y1 = cy + int(inner_r * math.sin(angle))
        x2 = cx + int(outer_r * math.cos(angle))
        y2 = cy + int(outer_r * math.sin(angle))
        rd.line([(x1, y1), (x2, y2)], fill=(255, 220, 100, 40), width=max(2, s // 150))
    rays = rays.filter(ImageFilter.GaussianBlur(radius=3))
    img = Image.alpha_composite(img, rays)

    # ═══ SCATTER DROPS: 12颗散射水滴 ═══
    draw = ImageDraw.Draw(img)
    import random
    random.seed(42)
    for i in range(12):
        angle = 2 * math.pi * i / 12 + 0.3
        dist = s * (0.20 + (i % 3) * 0.06)
        dx = cx + int(dist * math.cos(angle))
        dy = cy + int(dist * math.sin(angle))
        dr = int(s * (0.022 - (i % 3) * 0.003))

        # 颜色: 橙黄色系
        r_val = 255 - (i % 4) * 8
        g_val = 185 - (i % 3) * 12
        b_val = 40 + (i % 5) * 8

        # 阴影
        draw.ellipse([dx - dr + 2, dy - dr + 3, dx + dr + 2, dy + dr + 3],
                     fill=(0, 0, 0, 45))
        # 水滴主体
        draw.ellipse([dx - dr, dy - dr, dx + dr, dy + dr],
                     fill=(r_val, g_val, b_val, 230))
        # 高光
        draw.ellipse([dx - dr // 2, dy - dr // 2 - 1, dx + dr // 4, dy],
                     fill=(255, 245, 200, 140))

    # ═══ MAIN WATER DROP: 中心大水滴 ═══
    drop_h = int(s * 0.28)
    drop_w = int(s * 0.15)
    drop_top = cy - int(s * 0.20)
    drop_bot = cy + int(s * 0.15)

    points = []
    points.append((cx, drop_top))
    steps = 30
    for i in range(steps + 1):
        t = i / steps
        angle = t * math.pi
        x = cx + drop_w * math.sin(angle) * (1 - t * 0.3)
        y = drop_top + (drop_bot - drop_top) * t
        points.append((int(x), int(y)))
    points.append((cx, drop_bot + int(s * 0.02)))
    for i in range(steps, -1, -1):
        t = i / steps
        angle = t * math.pi
        x = cx - drop_w * math.sin(angle) * (1 - t * 0.3)
        y = drop_top + (drop_bot - drop_top) * t
        points.append((int(x), int(y)))

    # 水滴阴影
    shadow = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    sd = ImageDraw.Draw(shadow)
    shadow_pts = [(p[0] + 3, p[1] + 5) for p in points]
    sd.polygon(shadow_pts, fill=(0, 0, 0, 70))
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=10))
    img = Image.alpha_composite(img, shadow)
    draw = ImageDraw.Draw(img)

    # 水滴主体 (渐变效果)
    for i in range(6):
        offset = i * 2
        alpha = 255 - i * 18
        shade_r = 255 - i * 25
        shade_g = 190 - i * 20
        shade_b = 50 - i * 5
        filled_pts = [(p[0] + offset // 2, p[1] + offset // 3) for p in points]
        draw.polygon(filled_pts, fill=(shade_r, shade_g, max(0, shade_b), alpha))

    # 水滴高光
    highlight = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    hd = ImageDraw.Draw(highlight)
    hx = cx - int(s * 0.03)
    hy = drop_top + int(s * 0.08)
    hr = int(s * 0.035)
    hd.ellipse([hx - hr, hy - hr, hx + hr, hy + int(hr * 1.6)], fill=(255, 255, 255, 200))
    highlight = highlight.filter(ImageFilter.GaussianBlur(radius=5))
    img = Image.alpha_composite(img, highlight)
    draw = ImageDraw.Draw(img)

    # ═══ "N" LETTER: 白色N字 ═══
    draw = ImageDraw.Draw(img)
    letter_size = int(s * 0.10)
    lx = cx
    ly = cy + int(s * 0.01)
    lw = max(2, s // 90)
    # 左竖
    draw.line([(lx - letter_size // 2, ly - letter_size // 2),
               (lx - letter_size // 2, ly + letter_size // 2)],
              fill=(255, 255, 255, 240), width=lw)
    # 斜线
    draw.line([(lx - letter_size // 2, ly - letter_size // 2),
               (lx + letter_size // 2, ly + letter_size // 2)],
              fill=(255, 255, 255, 240), width=lw)
    # 右竖
    draw.line([(lx + letter_size // 2, ly - letter_size // 2),
               (lx + letter_size // 2, ly + letter_size // 2)],
              fill=(255, 255, 255, 240), width=lw)

    # ═══ MAC SQUIRCLE MASK ═══
    mask = squircle_mask(s)
    output = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    output.paste(img, mask=mask)
    return output

def generate_all():
    import os
    base_dir = '/Users/neo/Downloads/neotrix/assets/icons'
    os.makedirs(base_dir, exist_ok=True)

    variants = {
        'main': 'neotrix',
        'sim': 'nt-world-sim',
        'core': 'neotrix-core',
        'mind': 'nt-mind',
        'memory': 'nt-memory',
        'world': 'nt-world',
        'act': 'nt-act',
        'io': 'nt-io',
        'shield': 'nt-shield',
    }
    sizes = [1024, 512, 256, 128, 64, 32, 16]

    for var_key, var_name in variants.items():
        var_dir = os.path.join(base_dir, var_name)
        os.makedirs(var_dir, exist_ok=True)
        print(f'Generating {var_name}...')
        icon = create_neotrix_icon(1024, var_key)
        icon.save(os.path.join(var_dir, f'{var_name}-icon.png'))
        for sz in sizes:
            resized = icon.resize((sz, sz), Image.LANCZOS)
            resized.save(os.path.join(var_dir, f'{var_name}-icon-{sz}.png'))
        icon_ico = icon.resize((256, 256), Image.LANCZOS)
        icon_ico.save(os.path.join(var_dir, f'{var_name}.ico'), format='ICO',
                      sizes=[(256,256),(128,128),(64,64),(32,32),(16,16)])
        print(f'  ✓ {var_name}')

    preview = create_neotrix_icon(2048, 'main')
    preview.save(os.path.join(base_dir, 'neotrix-preview.png'))
    print(f'\n✓ All icons: {base_dir}')

if __name__ == '__main__':
    generate_all()
