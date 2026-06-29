#!/usr/bin/env python3
"""Generate text overlay PNGs for NeoTrix brand videos (2K+ variant).
Usage: python3 txt2k.py <text> <out.png> [fontsize=75] [color=#c084fc] [y=0.72] [--trim]
  --trim  Output tight-cropped text (transparent bg, minimal size) for slide-in animation
"""
import sys, os
from PIL import Image, ImageDraw, ImageFont

W, H = 2560, 1440
args = sys.argv[1:]
trim = '--trim' in args
if trim:
    args.remove('--trim')

text = args[0]
out = args[1]
fontsize = int(args[2]) if len(args) > 2 else 75
color_hex = args[3] if len(args) > 3 else "#c084fc"
y_ratio = float(args[4]) if len(args) > 4 else 0.72

hex_str = color_hex.lstrip('#')
r, g, b = int(hex_str[0:2], 16), int(hex_str[2:4], 16), int(hex_str[4:6], 16)

img = Image.new('RGBA', (0, 0), (0, 0, 0, 0))
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

bbox = font.getbbox(text)
tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]

if trim:
    pad = 40
    cw, ch = tw + pad * 2, th + pad * 2
    img = Image.new('RGBA', (cw, ch), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.text((pad - bbox[0], pad - bbox[1]), text, fill=(r, g, b, 255), font=font)
    img.save(out)
    print(f"Generated: {out} ({os.path.getsize(out)}b, {text} @ {fontsize}px, trim={cw}x{ch})", file=sys.stderr)
else:
    img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.text(((W - tw) // 2, int(H * y_ratio)), text, fill=(r, g, b, 255), font=font)
    img.save(out)
    print(f"Generated: {out} ({os.path.getsize(out)}b, {text} @ {fontsize}px)", file=sys.stderr)
