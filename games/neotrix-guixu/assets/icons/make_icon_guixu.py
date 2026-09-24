#!/usr/bin/env python3
"""歸墟 游戏图标 — macOS 液态玻璃 squircle + 透明底 + 金色歸字徽.

用法: python3 make_icon_guixu.py <输出1024主图路径>
管线: 主图 → sips 下采样 iconset → iconutil -c icns → .app Resources 部署.
设计（沿用 make_icon.py v2 玻璃语言，徽章由太极换歸字）:
  透明背景 + 高透玻璃体 + 液态高光边 + 金环 + 玄青 discs + PingFang SC 歸字
  （浅金渐变 #F0E3C4→#D6AC58→#C2933F，品牌 token）。
"""
import sys
from PIL import Image, ImageDraw, ImageFilter, ImageFont

S = 1024
GOLD = (201, 162, 39, 255)
GOLD_TOP = (240, 227, 196, 255)
GOLD_MID = (214, 172, 88, 255)
GOLD_BOT = (194, 147, 63, 255)
INK = (16, 19, 28, 255)
RADIUS = 232
PINGFANG = ("/System/Library/AssetsV2/com_apple_MobileAsset_Font8/"
            "86ba2c91f017a3749571a82f2c6d890ac7ffb2fb.asset/AssetData/PingFang.ttc")
PF_INDEX = 3  # PingFang SC Regular


def squircle_mask(size: int, radius: int) -> Image.Image:
    m = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(m)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=255)
    return m


def masked(layer: Image.Image, mask: Image.Image) -> Image.Image:
    return Image.composite(layer, Image.new("RGBA", layer.size, (0, 0, 0, 0)), mask)


def gold_text(size_px: int, ch: str) -> Image.Image:
    """浅金渐变字（透明底，尺寸 size_px 见方画布）"""
    font = ImageFont.truetype(PINGFANG, size_px, index=PF_INDEX)
    mask = Image.new("L", (size_px, size_px), 0)
    d = ImageDraw.Draw(mask)
    bb = d.textbbox((0, 0), ch, font=font)
    tw, th = bb[2] - bb[0], bb[3] - bb[1]
    d.text(((size_px - tw) / 2 - bb[0], (size_px - th) / 2 - bb[1]), ch,
           font=font, fill=255)
    grad = Image.new("RGBA", (size_px, size_px), (0, 0, 0, 0))
    gd = ImageDraw.Draw(grad)
    for y in range(size_px):
        t = y / max(1, size_px - 1)
        if t < 0.5:
            k = t * 2
            c = tuple(int(GOLD_TOP[i] + (GOLD_MID[i] - GOLD_TOP[i]) * k) for i in range(3)) + (255,)
        else:
            k = (t - 0.5) * 2
            c = tuple(int(GOLD_MID[i] + (GOLD_BOT[i] - GOLD_MID[i]) * k) for i in range(3)) + (255,)
        gd.line([(0, y), (size_px, y)], fill=c)
    out = Image.new("RGBA", (size_px, size_px), (0, 0, 0, 0))
    out.paste(grad, (0, 0), mask)
    return out


def main(out: str) -> None:
    mask = squircle_mask(S, RADIUS)
    base = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # 1. 玻璃体：高透纵向渐变
    glass = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glass)
    for y in range(S):
        t = y / (S - 1)
        r = int(198 - 72 * t)
        g = int(212 - 68 * t)
        b = int(235 - 54 * t)
        a = int(172 - 36 * t)
        gd.line([(0, y), (S, y)], fill=(r, g, b, a))
    glass = glass.filter(ImageFilter.GaussianBlur(2))
    base = Image.alpha_composite(base, masked(glass, mask))

    # 2. 底部厚度阴影
    sh = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sh)
    for y in range(int(S * 0.62), S):
        frac = max(0.0, (y - S * 0.62) / (S * 0.38))
        sd.line([(0, y), (S, y)], fill=(20, 30, 55, int(96 * frac ** 1.5)))
    sh = sh.filter(ImageFilter.GaussianBlur(16))
    base = Image.alpha_composite(base, masked(sh, mask))

    # 3. 顶部液态 sheen
    sheen = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    hd = ImageDraw.Draw(sheen)
    hd.ellipse([-320, -420, S + 120, 300], fill=(255, 255, 255, 72))
    sheen = sheen.filter(ImageFilter.GaussianBlur(30))
    base = Image.alpha_composite(base, masked(sheen, mask))

    d = ImageDraw.Draw(base)
    # 4. 液态边缘光
    d.rounded_rectangle([4, 4, S - 5, S - 5], radius=RADIUS - 4,
                        outline=(255, 255, 255, 120), width=4)
    d.arc([28, 28, S - 29, S - 29], start=195, end=345,
          fill=(255, 255, 255, 210), width=10)
    d.arc([28, 28, S - 29, S - 29], start=15, end=165,
          fill=(255, 255, 255, 56), width=6)

    # 5. 金环（品牌连续性）
    C, R = S // 2, 300
    d.ellipse([C - R - 14, C - R - 14, C + R + 14, C + R + 14],
              outline=GOLD, width=11)
    d.ellipse([C - R + 26, C - R + 26, C + R - 26, C + R - 26],
              outline=(201, 162, 39, 90), width=3)

    # 6. 玄青 discs + 金色歸字（太极徽换字徽）
    d.ellipse([C - R, C - R, C + R, C + R], fill=INK)
    gt = gold_text(560, "歸")
    # 压纹：暗影错位 +4/+6
    sh = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    dark = Image.new("RGBA", (560, 560), (8, 10, 14, 255))
    dark.putalpha(gt.split()[3])
    dark = dark.filter(ImageFilter.GaussianBlur(3))
    sh.alpha_composite(dark, (C - 280 + 4, C - 280 + 6))
    base = Image.alpha_composite(base, masked(sh, mask))
    base.alpha_composite(gt, (C - 280, C - 280))

    # 7. 徽章顶部镜面高光
    gl = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gd = ImageDraw.Draw(gl)
    gd.ellipse([C - 220, C - 330, C + 220, C - 60], fill=(255, 255, 255, 64))
    gl = gl.filter(ImageFilter.GaussianBlur(26))
    base = Image.alpha_composite(base, masked(gl, mask))

    # 8. 轨道弧 + 左上点光
    d = ImageDraw.Draw(base)
    d.arc([120, 120, S - 121, S - 121], start=200, end=340,
          fill=(255, 255, 255, 200), width=7)
    sp = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sp)
    sd.ellipse([150, 130, 260, 240], fill=(255, 255, 255, 150))
    sp = sp.filter(ImageFilter.GaussianBlur(22))
    base = Image.alpha_composite(base, masked(sp, mask))

    base.save(out)
    print(f"guixu icon master -> {out}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "icon_master_1024.png")
