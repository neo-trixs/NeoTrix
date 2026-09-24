#!/usr/bin/env python3
"""像素字体重建 v2：fusion-pixel（像素风优先）+ Noto（缺字移植）.

用法: python3 build_font_pixel.py
输入: /tmp/fusion-sub.ttf（像素底） /tmp/Noto.otf（Noto Sans CJK SC 全量 donor）
      ../data/*.ron + ../../src/*.rs（用字扫描） + charset.txt（基础语料）
输出: ../pixel-game.ttf
规则: ASCII 全收（可打印字符），CJK 按实际用字，缺字从 Noto 移植并报告。

2026-09-23 补记：原始 donor（/tmp/*）已失，现行补字走 surgical 路线——
  merge_pingfang.py（同目录）：PingFang SC Regular（系统 MobileAsset 内）
  subset 缺字集 → CFF 转 glyf（×1.2 upm 对齐）→ 合并。详见 ROUND16 报告。
"""
import os
import re
import string
import sys

from fontTools.ttLib import TTFont, newTable
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools import subset

BASE = os.path.dirname(os.path.abspath(__file__))
PIXEL_SRC = "/tmp/fusion-sub.ttf"
NOTO_SRC = "/tmp/Noto.otf"
RON_DIRS = [
    os.path.join(BASE, "..", "..", "data"),          # neotrix-swords/data
    os.path.join(BASE, "..", "..", "..", "crates", "neotrix-abilities", "data"),
]
SRC_DIR = os.path.join(BASE, "..", "..", "src")
CHARSET = os.path.join(BASE, "charset.txt")
OUT = os.path.join(BASE, "pixel-game.ttf")


def collect_corpus():
    chars = set(open(CHARSET, encoding="utf-8").read())
    chars.update(string.printable)  # ASCII 全收：数字/字母/标点/空格
    for ron_dir in RON_DIRS:
        for root, _, files in os.walk(ron_dir):
            for fn in files:
                if fn.endswith(".ron"):
                    chars.update(open(os.path.join(root, fn), encoding="utf-8").read())
    for root, _, files in os.walk(SRC_DIR):
        for fn in files:
            if fn.endswith(".rs"):
                chars.update(open(os.path.join(root, fn), encoding="utf-8").read())
    # 控制字符剔除（保留空格/换行只为子集播种，渲染无用）
    chars = {c for c in chars if c == " " or ord(c) >= 0x20}
    return "".join(sorted(chars))


def otf_to_ttf(font):
    """CFF → glyf（Noto OTF donor 用）。"""
    glyph_order = font.getGlyphOrder()
    glyph_set = font.getGlyphSet()
    tt_glyphs = {}
    for name in glyph_order:
        tt_pen = TTGlyphPen(glyph_set)
        cu_pen = Cu2QuPen(tt_pen, 1.0, True)
        glyph_set[name].draw(cu_pen)
        tt_glyphs[name] = tt_pen.glyph()
    font["loca"] = newTable("loca")
    font["glyf"] = glyf = newTable("glyf")
    glyf.glyphOrder = glyph_order
    glyf.glyphs = tt_glyphs
    for g in tt_glyphs.values():
        g.recalcBounds(glyf)
    del font["CFF "]
    if "VORG" in font:
        del font["VORG"]
    font.sfntVersion = "\000\001\000\000"
    font["maxp"].recalc(font)


def subset_to(font, text):
    ss = subset.Subsetter(subset.Options(
        name_IDs=["*"], hinting=False, desubroutinize=True,
        layout_features="*", glyph_names=False, legacy_cmap=True,
    ))
    ss.populate(text=text)
    ss.subset(font)
    return font


def main():
    corpus = collect_corpus()
    print(f"语料 {len(corpus)} 字（含 ASCII 全套 + 全 RON + 源码字符串）")
    pix = TTFont(PIXEL_SRC)
    pix_cmap = pix.getBestCmap()
    noto = TTFont(NOTO_SRC)
    if "CFF " in noto:
        print("→ Noto CFF→glyf 转制...")
        otf_to_ttf(noto)
    noto_cmap = noto.getBestCmap()

    missing = sorted({c for c in corpus if ord(c) not in pix_cmap})
    print(f"像素底缺字 {len(missing)}（前20: {''.join(missing[:20])}）")
    unfixable = [c for c in missing if ord(c) not in noto_cmap]
    if unfixable:
        print(f"警告：Noto 也无此 {len(unfixable)} 字：{''.join(unfixable[:30])}")

    # 像素底按全语料子集化（缺字位留空，后续合并）
    print("→ 像素底子集化...")
    pix = subset_to(pix, corpus)
    # 两边裁到最小表交集，避免 Merger 在版本/可选表上断言失败
    KEEP = {"head", "hhea", "maxp", "OS/2", "hmtx", "cmap", "loca", "glyf", "name", "post"}
    for tag in list(pix.keys()):
        if tag not in KEEP:
            del pix[tag]
    pix_tmp = OUT + ".pix.ttf"
    pix.save(pix_tmp)

    # 从 Noto 取缺字子集（转制→缩放到像素 upm→子集化），落盘后官方 merge
    from fontTools.merge import Merger
    from fontTools.pens.transformPen import TransformPen
    PIXEL_UPM = pix["head"].unitsPerEm
    noto2 = TTFont(NOTO_SRC)
    if "CFF " in noto2:
        otf_to_ttf(noto2)
    NOTO_UPM = noto2["head"].unitsPerEm
    if NOTO_UPM != PIXEL_UPM:
        print(f"→ Noto upm {NOTO_UPM}→{PIXEL_UPM} 缩放...")
        k = PIXEL_UPM / NOTO_UPM
        gs = noto2.getGlyphSet()
        scaled = {}
        for name in noto2.getGlyphOrder():
            ttp = TTGlyphPen(gs)
            tp = TransformPen(ttp, (k, 0, 0, k, 0, 0))
            gs[name].draw(tp)
            scaled[name] = ttp.glyph()
        noto2["glyf"].glyphs = scaled
        noto2["glyf"].glyphOrder = list(noto2.getGlyphOrder())
        for key in list(noto2["hmtx"].metrics.keys()):
            adv, lsb = noto2["hmtx"].metrics[key]
            noto2["hmtx"].metrics[key] = (int(round(adv * k)), int(round(lsb * k)))
        noto2["head"].unitsPerEm = PIXEL_UPM
    graft = subset_to(noto2, "".join(missing))
    for tag in list(graft.keys()):
        if tag not in KEEP:
            del graft[tag]
    # maxp 对齐 1.0 + 缺失属性补 0，避免 Merger 在版本差异字段上断言
    for f in (pix, graft):
        if f["maxp"].tableVersion != 0x00010000:
            f["maxp"].tableVersion = 0x00010000
        for attr in ("maxZones", "maxTwilightPoints", "maxStorage", "maxFunctionDefs",
                     "maxInstructionDefs", "maxStackElements", "maxSizeOfInstructions",
                     "maxComponentElements", "maxComponentDepth"):
            if not hasattr(f["maxp"], attr):
                setattr(f["maxp"], attr, 0)
    graft_tmp = OUT + ".graft.ttf"
    graft.save(graft_tmp)

    print(f"→ 合并（像素 {len(pix.getBestCmap())} + 移植 {len(graft.getBestCmap())}）...")
    merged = Merger().merge([pix_tmp, graft_tmp])
    merged.save(OUT)
    for t in (pix_tmp, graft_tmp):
        try:
            os.remove(t)
        except OSError:
            pass
    size = os.path.getsize(OUT) // 1024

    # 验收：全语料零缺失
    final_cmap = TTFont(OUT).getBestCmap()
    still = sorted({c for c in corpus if ord(c) not in final_cmap})
    grafted = len(graft.getBestCmap())
    print(f"产出 pixel-game.ttf ({size}KB)，移植约 {grafted} 字，残缺 {len(still)}")
    if still:
        print("残缺字:", "".join(still[:50]))
        return 1
    print("验收通过：语料零缺失（含 ASCII 数字字母标点）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
