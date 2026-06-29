#!/usr/bin/env python3
"""Generate text overlay PNGs for NeoTrix brand videos.
Usage: python3 txt.py <text> <out.png> [fontsize=56] [color=#c084fc] [y=0.72]"""
import sys, os, re
from PIL import Image, ImageDraw, ImageFont

W, H = 1920, 1080
text = sys.argv[1]
out = sys.argv[2]
fontsize = int(sys.argv[3]) if len(sys.argv) > 3 else 56
color_hex = sys.argv[4] if len(sys.argv) > 4 else "#c084fc"
y_ratio = float(sys.argv[5]) if len(sys.argv) > 5 else 0.72

# Parse hex color
hex_str = color_hex.lstrip('#')
r, g, b = int(hex_str[0:2], 16), int(hex_str[2:4], 16), int(hex_str[4:6], 16)

img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Find font
font = None
for fp in ['/System/Library/Fonts/Helvetica.ttc', '/System/Library/Fonts/Helvetica.ttf',
           '/System/Library/Fonts/SFNSDisplay.ttf', '/Library/Fonts/Arial.ttf']:
    if os.path.exists(fp):
        try:
            font = ImageFont.truetype(fp, fontsize)
            break
        except:
            continue
if not font:
    font = ImageFont.load_default()

bbox = draw.textbbox((0, 0), text, font=font)
tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
draw.text(((W - tw) // 2, int(H * y_ratio)), text, fill=(r, g, b, 255), font=font)
img.save(out)
print(f"Generated: {out} ({os.path.getsize(out)}b, {text} @ {fontsize}px)")
