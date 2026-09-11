#!/usr/bin/env python3
"""Neotrix Icon Generator — Orange water-drop scattering, Mac squircle"""

import math
from PIL import Image, ImageDraw, ImageFilter

def squircle_mask(size, radius_ratio=0.22):
    """Create a Mac-style superellipse (squircle) mask"""
    img = Image.new('L', (size, size), 0)
    draw = ImageDraw.Draw(img)
    r = size * radius_ratio
    # Draw the squircle using rounded rectangle
    draw.rounded_rectangle([0, 0, size-1, size-1], radius=int(r), fill=255)
    return img

def create_neotrix_icon(size=1024, variant='main'):
    """
    variant: 'main', 'sim', 'core', 'mind', 'memory', 'world', 'act', 'io', 'shield'
    """
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    s = size  # shorthand
    cx, cy = s // 2, s // 2
    
    # === BACKGROUND: Orange gradient ===
    # Radial gradient from center
    for y in range(s):
        for x in range(s):
            dist = math.sqrt((x - cx)**2 + (y - cy)**2) / (s * 0.5)
            dist = min(dist, 1.0)
            # Orange gradient: center bright, edge darker
            r = int(255 - dist * 60)
            g = int(160 - dist * 80)
            b = int(30 + dist * 20)
            a = 255
            if variant == 'sim':
                # Darker, more red
                r = int(240 - dist * 50)
                g = int(100 - dist * 60)
                b = int(20 + dist * 30)
            elif variant == 'core':
                # More golden
                r = int(255 - dist * 40)
                g = int(180 - dist * 60)
                b = int(50 + dist * 30)
            elif variant == 'mind':
                # More purple-orange
                r = int(200 - dist * 50)
                g = int(120 - dist * 70)
                b = int(80 + dist * 40)
            elif variant == 'memory':
                # More blue-orange
                r = int(220 - dist * 50)
                g = int(140 - dist * 60)
                b = int(100 + dist * 50)
            elif variant == 'shield':
                # More red-orange
                r = int(245 - dist * 40)
                g = int(90 - dist * 50)
                b = int(25 + dist * 25)
            img.putpixel((x, y), (r, g, b, a))
    
    # === GLOW: Central bright spot ===
    glow = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    glow_r = int(s * 0.35)
    for i in range(glow_r, 0, -1):
        alpha = int(80 * (1 - i / glow_r))
        gd.ellipse([cx - i, cy - i, cx + i, cy + i], fill=(255, 220, 100, alpha))
    img = Image.alpha_composite(img, glow)
    draw = ImageDraw.Draw(img)
    
    # === MAIN ELEMENT: Water Drop (teardrop shape) ===
    drop_h = int(s * 0.28)  # height of drop
    drop_w = int(s * 0.16)  # width at widest
    drop_top = cy - int(s * 0.18)
    drop_bot = cy + int(s * 0.16)
    
    # Draw teardrop using polygon
    points = []
    # Top point
    points.append((cx, drop_top))
    # Right curve down
    steps = 30
    for i in range(steps + 1):
        t = i / steps
        angle = t * math.pi
        x = cx + drop_w * math.sin(angle) * (1 - t * 0.3)
        y = drop_top + (drop_bot - drop_top) * t
        points.append((int(x), int(y)))
    # Bottom center
    points.append((cx, drop_bot + int(s * 0.02)))
    # Left curve up
    for i in range(steps, -1, -1):
        t = i / steps
        angle = t * math.pi
        x = cx - drop_w * math.sin(angle) * (1 - t * 0.3)
        y = drop_top + (drop_bot - drop_top) * t
        points.append((int(x), int(y)))
    
    # Drop shadow
    shadow = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    sd = ImageDraw.Draw(shadow)
    shadow_pts = [(p[0] + 3, p[1] + 5) for p in points]
    sd.polygon(shadow_pts, fill=(0, 0, 0, 60))
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=8))
    img = Image.alpha_composite(img, shadow)
    draw = ImageDraw.Draw(img)
    
    # Drop body (gradient effect via multiple fills)
    for i in range(5):
        offset = i * 2
        alpha = 255 - i * 20
        shade = 255 - i * 30
        filled_pts = [(p[0] + offset // 2, p[1] + offset // 3) for p in points]
        draw.polygon(filled_pts, fill=(shade, int(shade * 0.7), int(shade * 0.2), alpha))
    
    # Drop highlight
    highlight = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    hd = ImageDraw.Draw(highlight)
    hx = cx - int(s * 0.03)
    hy = drop_top + int(s * 0.08)
    hr = int(s * 0.04)
    hd.ellipse([hx - hr, hy - hr, hx + hr, hy + int(hr * 1.5)], fill=(255, 255, 255, 180))
    highlight = highlight.filter(ImageFilter.GaussianBlur(radius=4))
    img = Image.alpha_composite(img, highlight)
    draw = ImageDraw.Draw(img)
    
    # === SCATTER DROPS: Radiating outward ===
    num_drops = 12
    for i in range(num_drops):
        angle = (2 * math.pi * i / num_drops) + 0.3  # offset for aesthetics
        dist = s * (0.22 + (i % 3) * 0.06)
        dx = cx + int(dist * math.cos(angle))
        dy = cy + int(dist * math.sin(angle))
        
        # Drop size decreases with distance
        dr = int(s * (0.025 - (i % 3) * 0.004))
        
        # Color varies slightly
        r_val = 255 - (i % 4) * 10
        g_val = 180 - (i % 3) * 15
        b_val = 50 + (i % 5) * 10
        
        # Shadow
        draw.ellipse([dx - dr + 2, dy - dr + 3, dx + dr + 2, dy + dr + 3], 
                     fill=(0, 0, 0, 40))
        # Drop
        draw.ellipse([dx - dr, dy - dr, dx + dr, dy + dr], 
                     fill=(r_val, g_val, b_val, 220))
        # Highlight
        draw.ellipse([dx - dr // 2, dy - dr // 2 - 1, dx + dr // 4, dy], 
                     fill=(255, 240, 200, 120))
    
    # === LIGHT RAYS: Subtle radial lines ===
    rays = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    rd = ImageDraw.Draw(rays)
    num_rays = 8
    for i in range(num_rays):
        angle = 2 * math.pi * i / num_rays + 0.2
        inner_r = int(s * 0.18)
        outer_r = int(s * 0.42)
        x1 = cx + int(inner_r * math.cos(angle))
        y1 = cy + int(inner_r * math.sin(angle))
        x2 = cx + int(outer_r * math.cos(angle))
        y2 = cy + int(outer_r * math.sin(angle))
        rd.line([(x1, y1), (x2, y2)], fill=(255, 220, 100, 35), width=max(1, s // 200))
    rays = rays.filter(ImageFilter.GaussianBlur(radius=2))
    img = Image.alpha_composite(img, rays)
    
    # === "N" LETTER: Centered in drop ===
    draw = ImageDraw.Draw(img)
    letter_size = int(s * 0.12)
    lx = cx
    ly = cy + int(s * 0.01)
    # Draw "N" with lines
    lw = max(2, s // 100)
    # Left vertical
    draw.line([(lx - letter_size // 2, ly - letter_size // 2),
               (lx - letter_size // 2, ly + letter_size // 2)], 
              fill=(255, 255, 255, 230), width=lw)
    # Diagonal
    draw.line([(lx - letter_size // 2, ly - letter_size // 2),
               (lx + letter_size // 2, ly + letter_size // 2)], 
              fill=(255, 255, 255, 230), width=lw)
    # Right vertical
    draw.line([(lx + letter_size // 2, ly - letter_size // 2),
               (lx + letter_size // 2, ly + letter_size // 2)], 
              fill=(255, 255, 255, 230), width=lw)
    
    # === APPLY MAC SQUIRCLE MASK ===
    mask = squircle_mask(s)
    # Create final output
    output = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    output.paste(img, mask=mask)
    
    return output

def generate_all():
    import os
    
    base_dir = '/Users/neo/Downloads/neotrix/assets/icons'
    os.makedirs(base_dir, exist_ok=True)
    
    # Main Neotrix icon
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
        
        # Save full size
        icon.save(os.path.join(var_dir, f'{var_name}-icon.png'))
        
        # Save scaled versions
        for sz in sizes:
            resized = icon.resize((sz, sz), Image.LANCZOS)
            resized.save(os.path.join(var_dir, f'{var_name}-icon-{sz}.png'))
        
        # Save ICO for Windows
        icon_ico = icon.resize((256, 256), Image.LANCZOS)
        icon_ico.save(os.path.join(var_dir, f'{var_name}.ico'), format='ICO', 
                      sizes=[(256,256),(128,128),(64,64),(32,32),(16,16)])
        
        print(f'  ✓ {var_name} done')
    
    # Also generate a large preview
    preview = create_neotrix_icon(2048, 'main')
    preview.save(os.path.join(base_dir, 'neotrix-preview.png'))
    print(f'\n✓ All icons generated in {base_dir}')
    print(f'✓ Preview: {base_dir}/neotrix-preview.png')

if __name__ == '__main__':
    generate_all()
