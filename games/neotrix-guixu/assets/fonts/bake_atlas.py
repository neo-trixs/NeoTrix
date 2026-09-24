#!/usr/bin/env python3
"""预烘焙字形图集：charset.txt -> glyph_atlas.png(L单通道) + glyph_metrics.ron。
设计见 docs/architecture/RENDER-MIN-2026-09-24.md §1。
用法: python3 bake_atlas.py  (在 assets/fonts/ 下运行)
"""
from PIL import Image, ImageFont, ImageDraw

FONT = "pixel-game.ttf"
CHARSET = "charset.txt"
SIZE = 32
CELL = 40
CANVAS = 2048
COLS = CANVAS // CELL  # 51


def main():
    chars = [c for c in open(CHARSET, encoding="utf-8").read() if c.strip()]
    font = ImageFont.truetype(FONT, SIZE)
    ascent, descent = font.getmetrics()
    img = Image.new("L", (CANVAS, CANVAS), 0)
    d = ImageDraw.Draw(img)
    cmap = set()
    try:
        from fontTools.ttLib import TTFont
        cmap = set(TTFont(FONT).getBestCmap().keys())
    except Exception:
        pass
    recs, missing = [], []
    for i, ch in enumerate(chars):
        if cmap and ord(ch) not in cmap:
            missing.append(ch)
            continue
        cx, cy = (i % COLS) * CELL, (i // COLS) * CELL
        if cy + CELL > CANVAS:
            missing.append(ch)
            continue
        adv = font.getlength(ch)
        try:
            x0, y0, x1, y1 = font.getbbox(ch)
        except Exception:
            missing.append(ch)
            continue
        y_draw = cy + (CELL - (ascent + descent)) // 2 + ascent
        d.text((cx, y_draw), ch, font=font, fill=255)
        recs.append(
            f'(ch:"{ch}",x:{cx},y:{cy},w:{x1 - x0},h:{y1 - y0},'
            f"adv:{adv:.1f},top:{y_draw + y0 - cy})"
        )
    img.save("glyph_atlas.png")
    open("glyph_metrics.ron", "w", encoding="utf-8").write("[\n" + ",\n".join(recs) + "\n]\n")
    print(f"chars={len(chars)} baked={len(recs)} missing={len(missing)}")
    print("missing:", "".join(missing)[:60])
    import os
    print("png:", os.path.getsize("glyph_atlas.png") // 1024, "KB;",
          "ron:", os.path.getsize("glyph_metrics.ron") // 1024, "KB")


if __name__ == "__main__":
    main()
