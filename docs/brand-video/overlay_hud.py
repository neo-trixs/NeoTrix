#!/usr/bin/env python3
"""NeoTrix VSA Parameter HUD Overlay Generator.
Outputs a 2560x1440 RGBA PNG with telemetry-style HUD panel in the bottom-right corner.
Call repeatedly to generate frames with a cycling frame counter.
"""
import os, sys, random, struct
from PIL import Image, ImageDraw, ImageFont

W, H = 2560, 1440
PANEL_W, PANEL_H = 520, 340
MARGIN = 30
CYCLE_FILE = "/tmp/neotrix-hud-cycle.txt"

FONT_SIZE = 18
try:
    font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", FONT_SIZE)
except Exception:
    font = ImageFont.load_default()

def read_cycle():
    try:
        with open(CYCLE_FILE) as f: return int(f.read().strip()) + 1
    except Exception: return 1

def write_cycle(n):
    with open(CYCLE_FILE, "w") as f: f.write(str(n))

def random_vsa():
    return {
        "popcount": round(random.uniform(0.38, 0.52), 2),
        "entropy": round(random.uniform(0.82, 0.94), 2),
        "similarity": round(random.uniform(0.88, 0.98), 2),
        "drive": random.choice(["EXPLORE", "EXPLOIT", "REPAIR", "INNOVATE"]),
        "theta": round(random.uniform(0.55, 0.88), 2),
        "gdi": round(random.uniform(0.05, 0.25), 2),
        "handlers": random.randint(101, 107),
        "real_pct": random.randint(48, 54),
    }

def render_hud(output_path: str):
    cycle = read_cycle()
    vsa = random_vsa()

    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    px, py = W - PANEL_W - MARGIN, H - PANEL_H - MARGIN

    draw.rounded_rectangle(
        [px, py, px + PANEL_W, py + PANEL_H],
        radius=10, fill=(10, 0, 18, 180), outline=(60, 40, 80, 120), width=1
    )

    lines = [
        ("VSA PARAMETERS", (136, 68, 187)),
        ("", None),
        (f"DIM: 4096", (136, 136, 187)),
        (f"POPCOUNT: {vsa['popcount']:.2f}", (136, 136, 187)),
        (f"ENTROPY: {vsa['entropy']:.2f}", (136, 136, 187)),
        (f"COSINE SIM: {vsa['similarity']:.2f}", (136, 136, 187)),
        ("", None),
        (f"DRIVE: {vsa['drive']} (\u03b8={vsa['theta']:.2f})", (0, 212, 255)),
        (f"HANDLERS: {vsa['handlers']} ({vsa['real_pct']}% real)", (136, 136, 187)),
        (f"GDI: {vsa['gdi']:.2f} {'OK' if vsa['gdi'] < 0.2 else 'DRIFT'}", (0, 200, 100)),
        ("", None),
        (f"CYCLE: {cycle}", (80, 80, 120)),
    ]
    ly = py + 18
    for text, color in lines:
        if not text:
            ly += 4
            continue
        c = color or (136, 136, 187)
        draw.text((px + 18, ly), text, fill=c + (255,), font=font)
        ly += FONT_SIZE + 4

    write_cycle(cycle)
    img.save(output_path, "PNG")
    return output_path

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "/tmp/neotrix-hud.png"
    render_hud(out)
    print(out)
